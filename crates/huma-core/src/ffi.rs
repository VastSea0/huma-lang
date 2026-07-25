use crate::value::Deger;
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use unicode_normalization::UnicodeNormalization;

const EN_FAZLA_FFI_KUTUPHANE: usize = 256;
const EN_FAZLA_FFI_AD_BYTES: usize = 256;
const EN_FAZLA_FFI_YOL_BYTES: usize = 4096;
const EN_FAZLA_FFI_IMZA_BYTES: usize = 64;

fn metin_alani_dogrula(
    deger: &str,
    alan: &str,
    en_fazla_bytes: usize,
    nfc_zorunlu: bool,
) -> Result<(), String> {
    if deger.trim().is_empty() {
        return Err(format!("{alan} boş olamaz"));
    }
    if deger.len() > en_fazla_bytes {
        return Err(format!(
            "{alan} {en_fazla_bytes} baytlık güvenlik sınırını aşıyor"
        ));
    }
    if deger.contains('\0') {
        return Err(format!("{alan} NUL karakteri içeremez"));
    }
    if deger.chars().any(char::is_control) {
        return Err(format!("{alan} denetim karakteri içeremez"));
    }
    if nfc_zorunlu && deger.nfc().collect::<String>() != deger {
        return Err(format!("{alan} NFC ile normalize edilmiş olmalıdır"));
    }
    Ok(())
}

pub struct FFIManager {
    libraries: HashMap<String, Arc<Library>>,
}

impl Default for FFIManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FFIManager {
    pub fn new() -> Self {
        Self {
            libraries: HashMap::new(),
        }
    }

    pub fn yukle(&mut self, ad: &str, yol: &str) -> Result<(), String> {
        metin_alani_dogrula(ad, "FFI kütüphane adı", EN_FAZLA_FFI_AD_BYTES, true)?;
        metin_alani_dogrula(yol, "FFI kütüphane yolu", EN_FAZLA_FFI_YOL_BYTES, false)?;
        if self.libraries.contains_key(ad) {
            return Err(format!(
                "FFI kütüphane adı zaten yüklü: {ad}. Önce ffi_boşalt kullanın"
            ));
        }
        if self.libraries.len() >= EN_FAZLA_FFI_KUTUPHANE {
            return Err(format!(
                "Aynı anda en fazla {EN_FAZLA_FFI_KUTUPHANE} FFI kütüphanesi yüklenebilir"
            ));
        }
        // Güven sınırı: Library'nin ömrü yöneticide tutulur ve hiçbir Symbol bu
        // çağrıdan dışarı çıkarılmaz. Yanlış bir haricî ABI yine de ev sahibi
        // süreci çökertebilir; bu nedenle FFI ayrıca açık yetenek gerektirir.
        unsafe {
            match Library::new(yol) {
                Ok(lib) => {
                    self.libraries.insert(ad.to_string(), Arc::new(lib));
                    Ok(())
                }
                Err(e) => Err(format!(
                    "C/C++/CUDA kütüphanesi yüklenemedi ('{}'): {}",
                    yol, e
                )),
            }
        }
    }

    pub fn bosalt(&mut self, ad: &str) -> Result<(), String> {
        metin_alani_dogrula(ad, "FFI kütüphane adı", EN_FAZLA_FFI_AD_BYTES, true)?;
        self.libraries
            .remove(ad)
            .map(|_| ())
            .ok_or_else(|| format!("Yüklenmemiş FFI kütüphanesi: {ad}"))
    }

    pub fn cagir_f64_f64(&self, lib_ad: &str, fn_ad: &str, arg: f64) -> Result<f64, String> {
        match self.cagir_imzali(lib_ad, fn_ad, "f64(f64)", vec![Deger::Sayi(arg)])? {
            Deger::Sayi(value) => Ok(value),
            _ => Err("FFI iç hatası: sayısal sonuç bekleniyordu".to_string()),
        }
    }

    pub fn cagir_f64_f64_f64(
        &self,
        lib_ad: &str,
        fn_ad: &str,
        arg1: f64,
        arg2: f64,
    ) -> Result<f64, String> {
        match self.cagir_imzali(
            lib_ad,
            fn_ad,
            "f64(f64,f64)",
            vec![Deger::Sayi(arg1), Deger::Sayi(arg2)],
        )? {
            Deger::Sayi(value) => Ok(value),
            _ => Err("FFI iç hatası: sayısal sonuç bekleniyordu".to_string()),
        }
    }

    pub fn cagir_imzali(
        &self,
        lib_ad: &str,
        fn_ad: &str,
        imza: &str,
        argumanlar: Vec<Deger>,
    ) -> Result<Deger, String> {
        metin_alani_dogrula(lib_ad, "FFI kütüphane adı", EN_FAZLA_FFI_AD_BYTES, true)?;
        metin_alani_dogrula(fn_ad, "FFI fonksiyon adı", EN_FAZLA_FFI_AD_BYTES, false)?;
        metin_alani_dogrula(imza, "FFI imzası", EN_FAZLA_FFI_IMZA_BYTES, false)?;
        let lib = self
            .libraries
            .get(lib_ad)
            .ok_or_else(|| format!("Yüklenmemiş FFI kütüphanesi: {}", lib_ad))?;

        let numbers = argumanlar
            .iter()
            .enumerate()
            .map(|(index, value)| match value {
                Deger::Sayi(number) if number.is_finite() => Ok(*number),
                Deger::Sayi(_) => Err(format!("FFI {index}. argümanı sonlu sayı olmalıdır")),
                other => Err(format!(
                    "FFI {index}. argümanı sayı olmalıdır; {other} geldi"
                )),
            })
            .collect::<Result<Vec<_>, _>>()?;

        unsafe {
            let result = match (imza, numbers.as_slice()) {
                ("f64()", []) => {
                    let func: Symbol<unsafe extern "C" fn() -> f64> = lib
                        .get(fn_ad.as_bytes())
                        .map_err(|e| format!("FFI Sembolü bulunamadı ('{}'): {}", fn_ad, e))?;
                    func()
                }
                ("f64(f64)", [arg]) => {
                    let func: Symbol<unsafe extern "C" fn(f64) -> f64> = lib
                        .get(fn_ad.as_bytes())
                        .map_err(|e| format!("FFI Sembolü bulunamadı ('{}'): {}", fn_ad, e))?;
                    func(*arg)
                }
                ("f64(f64,f64)", [left, right]) => {
                    let func: Symbol<unsafe extern "C" fn(f64, f64) -> f64> = lib
                        .get(fn_ad.as_bytes())
                        .map_err(|e| format!("FFI Sembolü bulunamadı ('{}'): {}", fn_ad, e))?;
                    func(*left, *right)
                }
                ("f64()", _) | ("f64(f64)", _) | ("f64(f64,f64)", _) => {
                    return Err(format!(
                        "FFI imzası '{imza}' ile {} argüman uyuşmuyor",
                        numbers.len()
                    ))
                }
                _ => {
                    return Err(format!(
                        "Desteklenmeyen FFI imzası: '{imza}'. Desteklenenler: \
                         f64(), f64(f64), f64(f64,f64)"
                    ))
                }
            };
            if result.is_finite() {
                Ok(Deger::Sayi(result))
            } else {
                Err("FFI sonlu olmayan sayısal sonuç döndürdü".to_string())
            }
        }
    }
}

pub static FFI_YONETICI: once_cell::sync::Lazy<Arc<Mutex<FFIManager>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(FFIManager::new())));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffi_metin_sinirlari_yuklemeden_once_dogrulanir() {
        let mut manager = FFIManager::new();
        assert!(manager.yukle("", "kitaplik").unwrap_err().contains("boş"));
        assert!(manager
            .yukle("e\u{301}", "kitaplik")
            .unwrap_err()
            .contains("NFC"));
        assert!(manager
            .yukle("ad", "kötü\0yol")
            .unwrap_err()
            .contains("NUL"));
        assert!(manager
            .cagir_imzali("yok", "işlev\0", "f64()", vec![])
            .unwrap_err()
            .contains("NUL"));
    }

    #[test]
    fn ffi_bosalt_yuklenmemis_adi_sessizce_yutmaz() {
        let mut manager = FFIManager::new();
        assert!(manager.bosalt("yok").unwrap_err().contains("Yüklenmemiş"));
    }
}
