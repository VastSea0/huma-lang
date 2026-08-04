use std::cell::{Cell, RefCell};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    FileRead,
    FileWrite,
    NetworkClient,
    NetworkServer,
    Process,
    Ffi,
    Database,
    Gui,
}

impl Capability {
    pub const ALL: [Self; 8] = [
        Self::FileRead,
        Self::FileWrite,
        Self::NetworkClient,
        Self::NetworkServer,
        Self::Process,
        Self::Ffi,
        Self::Database,
        Self::Gui,
    ];

    pub fn turkce_adi(self) -> &'static str {
        match self {
            Self::FileRead => "dosya-okuma",
            Self::FileWrite => "dosya-yazma",
            Self::NetworkClient => "ağ-istemci",
            Self::NetworkServer => "ağ-sunucu",
            Self::Process => "süreç",
            Self::Ffi => "ffi",
            Self::Database => "veritabanı",
            Self::Gui => "gui",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilitySet {
    allowed: HashSet<Capability>,
}

impl CapabilitySet {
    pub fn deny_all() -> Self {
        Self::default()
    }

    pub fn allow_all() -> Self {
        Self {
            allowed: Capability::ALL.into_iter().collect(),
        }
    }

    pub fn allow(mut self, capability: Capability) -> Self {
        self.allowed.insert(capability);
        self
    }

    pub fn allows(&self, capability: Capability) -> bool {
        self.allowed.contains(&capability)
    }
}

thread_local! {
    static CURRENT_CAPABILITIES: RefCell<CapabilitySet> =
        RefCell::new(CapabilitySet::deny_all());
    static CAPABILITY_RESTORE_FAILED: Cell<bool> = const { Cell::new(false) };
}

/// Restores the previous thread-local policy when dropped.
pub struct CapabilityGuard {
    previous: Option<CapabilitySet>,
}

impl Drop for CapabilityGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.take() {
            CURRENT_CAPABILITIES.with(|current| {
                if let Ok(mut borrowed) = current.try_borrow_mut() {
                    *borrowed = previous;
                } else {
                    // Bir yetenek guard'ının geri yüklenememesi ayrıcalığın
                    // açık kalmasına yol açmamalı. Sonraki bütün denetimler,
                    // yeni bir politika başarıyla kurulana kadar kapalı kalır.
                    CAPABILITY_RESTORE_FAILED.with(|failed| failed.set(true));
                }
            });
        }
    }
}

pub fn install(capabilities: CapabilitySet) -> Result<CapabilityGuard, String> {
    CURRENT_CAPABILITIES.with(|current| {
        let mut borrowed = current
            .try_borrow_mut()
            .map_err(|_| "Yetenek politikası şu anda kullanımda".to_string())?;
        let previous = std::mem::replace(&mut *borrowed, capabilities);
        CAPABILITY_RESTORE_FAILED.with(|failed| failed.set(false));
        Ok(CapabilityGuard {
            previous: Some(previous),
        })
    })
}

pub fn require(capability: Capability, operation: &str) -> Result<(), String> {
    if CAPABILITY_RESTORE_FAILED.with(Cell::get) {
        return Err(format!(
            "{operation}: yetenek politikası güvenli biçimde geri yüklenemedi; bütün yetenekler kapalı"
        ));
    }
    CURRENT_CAPABILITIES.with(|current| {
        let borrowed = current
            .try_borrow()
            .map_err(|_| format!("{operation}: yetenek politikası kullanımda"))?;
        if borrowed.allows(capability) {
            Ok(())
        } else {
            Err(format!(
                "{operation}: '{}' yeteneği verilmedi",
                capability.turkce_adi()
            ))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{install, require, Capability, CapabilitySet, CURRENT_CAPABILITIES};

    #[test]
    fn politika_guard_onceki_yetenekleri_geri_yukler() {
        assert!(require(Capability::Process, "test").is_err());
        {
            let _guard = install(CapabilitySet::deny_all().allow(Capability::Process))
                .expect("Politika kurulmalı");
            assert!(require(Capability::Process, "test").is_ok());
        }
        assert!(require(Capability::Process, "test").is_err());
    }

    #[test]
    fn guard_geri_yukleme_cakismasinda_yetenekler_kapali_kalir() {
        let guard = install(CapabilitySet::deny_all().allow(Capability::Process))
            .expect("Politika kurulmalı");
        CURRENT_CAPABILITIES.with(|current| {
            let _borrow = current.borrow();
            drop(guard);
        });
        assert!(require(Capability::Process, "test")
            .expect_err("Geri yükleme hatası fail-closed olmalı")
            .contains("bütün yetenekler kapalı"));

        let recovery = install(CapabilitySet::deny_all()).expect("Kapalı politika kurulabilmeli");
        std::mem::forget(recovery);
        assert!(require(Capability::Process, "test").is_err());
    }
}
