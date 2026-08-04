//! Hüma'nın isteğe bağlı yerel kütüphane adaptörü.
//!
//! Bu sınır güvenilir olmayan yerel ABI'yi normatif çalışma zamanından ayırır.
//! Varsayılan sınır sürümlü HMI üzerinden ayrı süreçtir. Süreç içi FFI yalnız
//! ayrıca güvenilir olarak etkinleştirilen eski/dar ABI uyumluluğu içindir.

mod ffi;
mod hmi_host;

pub use ffi::{FFIManager, FFI_YONETICI};
pub use hmi_host::{HmiManager, HMI_YONETICI};

use huma_runtime::capability::{self, Capability};
use huma_runtime::value::Deger;
use std::collections::HashMap;
use std::rc::Rc;

fn yetenek_hatasi(operation: &str) -> Option<Deger> {
    capability::require(Capability::Ffi, operation)
        .err()
        .map(Deger::Hata)
}

/// Güvenli, süreç dışı HMI yerleşiklerini ekler.
pub fn kayit_et(globals: &mut HashMap<String, Deger>) {
    hmi_host::kayit_et(globals);
}

/// Yalnız açıkça güvenilen kod için süreç içi FFI yerleşiklerini ekler.
pub fn guvenilir_ffi_kayit_et(globals: &mut HashMap<String, Deger>) {
    globals.insert(
        "ffi_yükle".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(name), Deger::Metin(path)] = args.as_slice() else {
                return Deger::Hata(
                    "ffi_yükle: kütüphane adı ve dosya yolu olmak üzere 2 metin gerekir"
                        .to_string(),
                );
            };
            if let Some(error) = yetenek_hatasi("ffi_yükle") {
                return error;
            }
            let mut manager = match FFI_YONETICI.lock() {
                Ok(manager) => manager,
                Err(_) => {
                    return Deger::Hata("ffi_yükle: FFI yöneticisi kilidi bozuldu".to_string())
                }
            };
            match manager.yukle(name, path) {
                Ok(()) => Deger::Sayi(1.0),
                Err(error) => Deger::Hata(format!("ffi_yükle: {error}")),
            }
        }),
    );
    globals.insert(
        "ffi_çağır".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (
                Some(Deger::Metin(library)),
                Some(Deger::Metin(function)),
                Some(Deger::Metin(signature)),
            ) = (args.first(), args.get(1), args.get(2))
            else {
                return Deger::Hata(
                    "ffi_çağır: kütüphane adı, fonksiyon adı ve açık ABI imzası gerekir"
                        .to_string(),
                );
            };
            if let Some(error) = yetenek_hatasi("ffi_çağır") {
                return error;
            }
            let manager = match FFI_YONETICI.lock() {
                Ok(manager) => manager,
                Err(_) => {
                    return Deger::Hata("ffi_çağır: FFI yöneticisi kilidi bozuldu".to_string())
                }
            };
            match manager.cagir_imzali(library, function, signature, args[3..].to_vec()) {
                Ok(result) => result,
                Err(error) => Deger::Hata(format!("ffi_çağır: {error}")),
            }
        }),
    );
    globals.insert(
        "ffi_boşalt".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(name)] = args.as_slice() else {
                return Deger::Hata(
                    "ffi_boşalt: tam olarak bir kütüphane adı metni gerekir".to_string(),
                );
            };
            if let Some(error) = yetenek_hatasi("ffi_boşalt") {
                return error;
            }
            let mut manager = match FFI_YONETICI.lock() {
                Ok(manager) => manager,
                Err(_) => {
                    return Deger::Hata("ffi_boşalt: FFI yöneticisi kilidi bozuldu".to_string())
                }
            };
            match manager.bosalt(name) {
                Ok(()) => Deger::Sayi(1.0),
                Err(error) => Deger::Hata(format!("ffi_boşalt: {error}")),
            }
        }),
    );
}

struct NativeHost;

impl huma_vm::NativeCallHost for NativeHost {
    fn call(
        &self,
        library: &str,
        function: &str,
        signature: &str,
        arguments: Vec<Deger>,
    ) -> Result<Deger, String> {
        let manager = FFI_YONETICI
            .lock()
            .map_err(|_| "FFI yöneticisi kilidi bozuldu".to_string())?;
        manager.cagir_imzali(library, function, signature, arguments)
    }
}

/// VM'nin native çağrı komutları için açıkça enjekte edilen ana makineyi kurar.
pub fn vm_call_host() -> Rc<dyn huma_vm::NativeCallHost> {
    Rc::new(NativeHost)
}
