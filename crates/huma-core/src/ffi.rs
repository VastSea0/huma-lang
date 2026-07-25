use crate::value::Deger;
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::sync::{Arc, Mutex};

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

    pub fn cagir_f64_f64(&self, lib_ad: &str, fn_ad: &str, arg: f64) -> Result<f64, String> {
        let lib = self
            .libraries
            .get(lib_ad)
            .ok_or_else(|| format!("Yüklenmemiş FFI kütüphanesi: {}", lib_ad))?;
        unsafe {
            let func: Symbol<unsafe extern "C" fn(f64) -> f64> = lib
                .get(fn_ad.as_bytes())
                .map_err(|e| format!("FFI Sembolü bulunamadı ('{}'): {}", fn_ad, e))?;
            Ok(func(arg))
        }
    }

    pub fn cagir_f64_f64_f64(
        &self,
        lib_ad: &str,
        fn_ad: &str,
        arg1: f64,
        arg2: f64,
    ) -> Result<f64, String> {
        let lib = self
            .libraries
            .get(lib_ad)
            .ok_or_else(|| format!("Yüklenmemiş FFI kütüphanesi: {}", lib_ad))?;
        unsafe {
            let func: Symbol<unsafe extern "C" fn(f64, f64) -> f64> = lib
                .get(fn_ad.as_bytes())
                .map_err(|e| format!("FFI Sembolü bulunamadı ('{}'): {}", fn_ad, e))?;
            Ok(func(arg1, arg2))
        }
    }

    pub fn cagir_esnek(
        &self,
        lib_ad: &str,
        fn_ad: &str,
        argumanlar: Vec<Deger>,
    ) -> Result<Deger, String> {
        let lib = self
            .libraries
            .get(lib_ad)
            .ok_or_else(|| format!("Yüklenmemiş FFI kütüphanesi: {}", lib_ad))?;

        unsafe {
            match argumanlar.len() {
                0 => {
                    let func: Symbol<unsafe extern "C" fn() -> f64> = lib
                        .get(fn_ad.as_bytes())
                        .map_err(|e| format!("FFI Sembolü bulunamadı ('{}'): {}", fn_ad, e))?;
                    Ok(Deger::Sayi(func()))
                }
                1 => match &argumanlar[0] {
                    Deger::Sayi(n) => {
                        let func: Symbol<unsafe extern "C" fn(f64) -> f64> = lib
                            .get(fn_ad.as_bytes())
                            .map_err(|e| format!("FFI Sembolü bulunamadı ('{}'): {}", fn_ad, e))?;
                        Ok(Deger::Sayi(func(*n)))
                    }
                    Deger::Metin(s) => {
                        let c_str = CString::new(s.as_str()).map_err(|e| e.to_string())?;
                        let func: Symbol<
                            unsafe extern "C" fn(
                                *const std::os::raw::c_char,
                            )
                                -> *const std::os::raw::c_char,
                        > = lib
                            .get(fn_ad.as_bytes())
                            .map_err(|e| format!("FFI Sembolü bulunamadı ('{}'): {}", fn_ad, e))?;
                        let res_ptr = func(c_str.as_ptr());
                        if res_ptr.is_null() {
                            Ok(Deger::Bos)
                        } else {
                            let res_str = CStr::from_ptr(res_ptr).to_string_lossy().into_owned();
                            Ok(Deger::Metin(res_str))
                        }
                    }
                    _ => Err("Desteklenmeyen FFI argüman tipi".to_string()),
                },
                2 => {
                    if let (Deger::Sayi(n1), Deger::Sayi(n2)) = (&argumanlar[0], &argumanlar[1]) {
                        let func: Symbol<unsafe extern "C" fn(f64, f64) -> f64> = lib
                            .get(fn_ad.as_bytes())
                            .map_err(|e| format!("FFI Sembolü bulunamadı ('{}'): {}", fn_ad, e))?;
                        Ok(Deger::Sayi(func(*n1, *n2)))
                    } else {
                        Err("İki argümanlı FFI çağrısında sayısal değerler bekleniyor".to_string())
                    }
                }
                _ => Err(format!(
                    "FFI çağrısında {} argüman henüz desteklenmiyor",
                    argumanlar.len()
                )),
            }
        }
    }
}

pub static FFI_YONETICI: once_cell::sync::Lazy<Arc<Mutex<FFIManager>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(FFIManager::new())));
