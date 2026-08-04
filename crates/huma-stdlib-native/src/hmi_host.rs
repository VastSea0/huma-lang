use huma_hmi::{HmiValue, ProcessClient, ProtocolVersion};
use huma_runtime::capability::{self, Capability};
use huma_runtime::gc::Gc;
use huma_runtime::value::Deger;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use unicode_normalization::UnicodeNormalization;

const MAX_MODULES: usize = 256;
const MAX_ITEMS: usize = 1_000_000;
const MAX_DEPTH: usize = 128;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct HmiManager {
    modules: HashMap<String, ProcessClient>,
    timeout: Duration,
}

impl Default for HmiManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HmiManager {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            timeout: DEFAULT_TIMEOUT,
        }
    }

    pub fn with_timeout(timeout: Duration) -> Result<Self, String> {
        if timeout.is_zero() {
            return Err("HMI zaman aşımı sıfır olamaz".to_string());
        }
        Ok(Self {
            modules: HashMap::new(),
            timeout,
        })
    }

    pub fn baslat(&mut self, name: &str, executable: &Path) -> Result<(), String> {
        validate_name(name, "HMI modül adı")?;
        if self.modules.contains_key(name) {
            return Err(format!("HMI modülü zaten çalışıyor: {name}"));
        }
        if self.modules.len() >= MAX_MODULES {
            return Err(format!(
                "Aynı anda en fazla {MAX_MODULES} HMI modülü çalışabilir"
            ));
        }
        let client = ProcessClient::spawn(executable, name, ProtocolVersion::V1_0, self.timeout)
            .map_err(|error| error.to_string())?;
        self.modules.insert(name.to_string(), client);
        Ok(())
    }

    pub fn cagir(
        &mut self,
        module: &str,
        function: &str,
        arguments: Vec<Deger>,
    ) -> Result<Deger, String> {
        validate_name(module, "HMI modül adı")?;
        validate_name(function, "HMI fonksiyon adı")?;
        let mut active = HashSet::new();
        let mut items = 0;
        let arguments = arguments
            .iter()
            .map(|value| to_hmi(value, &mut active, 0, &mut items))
            .collect::<Result<Vec<_>, _>>()?;
        let value = self
            .modules
            .get_mut(module)
            .ok_or_else(|| format!("HMI modülü çalışmıyor: {module}"))?
            .call(function, arguments)
            .map_err(|error| error.to_string())?;
        from_hmi(value, 0, &mut 0)
    }

    pub fn kapat(&mut self, name: &str) -> Result<(), String> {
        validate_name(name, "HMI modül adı")?;
        self.modules
            .remove(name)
            .ok_or_else(|| format!("HMI modülü çalışmıyor: {name}"))?
            .shutdown()
            .map_err(|error| error.to_string())
    }
}

fn validate_name(value: &str, field: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > 4_096 {
        return Err(format!("{field} boş olamaz ve 4096 baytı aşamaz"));
    }
    if value.contains('\0') || value.chars().any(char::is_control) {
        return Err(format!("{field} denetim karakteri içeremez"));
    }
    if value.nfc().collect::<String>() != value {
        return Err(format!("{field} NFC biçiminde olmalıdır"));
    }
    Ok(())
}

fn count_item(items: &mut usize) -> Result<(), String> {
    *items = items
        .checked_add(1)
        .ok_or_else(|| "HMI öğe sayısı taştı".to_string())?;
    if *items > MAX_ITEMS {
        return Err(format!("HMI değeri {MAX_ITEMS} öğe sınırını aşıyor"));
    }
    Ok(())
}

fn to_hmi(
    value: &Deger,
    active: &mut HashSet<usize>,
    depth: usize,
    items: &mut usize,
) -> Result<HmiValue, String> {
    if depth > MAX_DEPTH {
        return Err("HMI değeri iç içelik sınırını aşıyor".to_string());
    }
    count_item(items)?;
    match value {
        Deger::Sayi(number) if number.is_finite() => Ok(HmiValue::Number(*number)),
        Deger::Sayi(_) => Err("HMI yalnız sonlu sayıları kabul eder".to_string()),
        Deger::Metin(text) => Ok(HmiValue::Text(text.clone())),
        Deger::Bayt(bytes) => Ok(HmiValue::Bytes(bytes.clone())),
        Deger::Bos => Ok(HmiValue::Empty),
        Deger::Liste(values) => {
            let identity = Gc::as_ptr(values) as usize;
            if !active.insert(identity) {
                return Err("Döngüsel liste HMI sınırından geçemez".to_string());
            }
            let borrowed = values
                .try_borrow()
                .map_err(|_| "Liste HMI dönüşümü sırasında kullanımda".to_string())?;
            let result = borrowed
                .iter()
                .map(|value| to_hmi(value, active, depth + 1, items))
                .collect::<Result<Vec<_>, _>>()
                .map(HmiValue::List);
            active.remove(&identity);
            result
        }
        Deger::Nesne { alanlar, .. } | Deger::Sozluk(alanlar) => {
            let identity = Gc::as_ptr(alanlar) as usize;
            if !active.insert(identity) {
                return Err("Döngüsel harita HMI sınırından geçemez".to_string());
            }
            let borrowed = alanlar
                .try_borrow()
                .map_err(|_| "Harita HMI dönüşümü sırasında kullanımda".to_string())?;
            let result = borrowed
                .iter()
                .map(|(key, value)| {
                    validate_name(key, "HMI harita anahtarı")?;
                    Ok((key.clone(), to_hmi(value, active, depth + 1, items)?))
                })
                .collect::<Result<BTreeMap<_, _>, String>>()
                .map(HmiValue::Map);
            active.remove(&identity);
            result
        }
        Deger::Vektor(values) => values
            .try_borrow()
            .map_err(|_| "Vektör HMI dönüşümü sırasında kullanımda".to_string())?
            .iter()
            .map(|value| {
                count_item(items)?;
                Ok(HmiValue::Number(*value))
            })
            .collect::<Result<Vec<_>, String>>()
            .map(HmiValue::List),
        Deger::Matris {
            satirlar,
            sutunlar,
            veri,
        } => {
            let borrowed = veri
                .try_borrow()
                .map_err(|_| "Matris HMI dönüşümü sırasında kullanımda".to_string())?;
            let expected = satirlar
                .checked_mul(*sutunlar)
                .ok_or_else(|| "Matris boyutu taştı".to_string())?;
            if borrowed.len() != expected {
                return Err("Matris veri boyutu geçersiz".to_string());
            }
            let mut rows = Vec::with_capacity(*satirlar);
            if *sutunlar == 0 {
                for _ in 0..*satirlar {
                    count_item(items)?;
                    rows.push(HmiValue::List(Vec::new()));
                }
                return Ok(HmiValue::List(rows));
            }
            for row in borrowed.chunks(*sutunlar) {
                count_item(items)?;
                let values = row
                    .iter()
                    .map(|value| {
                        count_item(items)?;
                        Ok(HmiValue::Number(*value))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                rows.push(HmiValue::List(values));
            }
            Ok(HmiValue::List(rows))
        }
        Deger::GorevId(_)
        | Deger::Fonksiyon { .. }
        | Deger::BytecodeFonksiyon { .. }
        | Deger::DahiliFonksiyon(_)
        | Deger::BaglamliDahiliFonksiyon(_)
        | Deger::Sinif { .. }
        | Deger::Hata(_)
        | Deger::Harici(_) => Err("Bu Hüma değeri HMI sınırından geçemez".to_string()),
    }
}

fn from_hmi(value: HmiValue, depth: usize, items: &mut usize) -> Result<Deger, String> {
    if depth > MAX_DEPTH {
        return Err("HMI yanıtı iç içelik sınırını aşıyor".to_string());
    }
    count_item(items)?;
    match value {
        HmiValue::Number(number) if number.is_finite() => Ok(Deger::Sayi(number)),
        HmiValue::Number(_) => Err("HMI yanıtı sonlu olmayan sayı içeriyor".to_string()),
        HmiValue::Boolean(value) => Ok(Deger::Sayi(if value { 1.0 } else { 0.0 })),
        HmiValue::Text(text) => Ok(Deger::Metin(text)),
        HmiValue::Bytes(bytes) => Ok(Deger::Bayt(bytes)),
        HmiValue::Empty => Ok(Deger::Bos),
        HmiValue::List(values) => values
            .into_iter()
            .map(|value| from_hmi(value, depth + 1, items))
            .collect::<Result<Vec<_>, _>>()
            .map(Gc::new)
            .map(Deger::Liste),
        HmiValue::Map(values) => values
            .into_iter()
            .map(|(key, value)| Ok((key, from_hmi(value, depth + 1, items)?)))
            .collect::<Result<HashMap<_, _>, String>>()
            .map(Gc::new)
            .map(Deger::Sozluk),
    }
}

pub static HMI_YONETICI: once_cell::sync::Lazy<Arc<Mutex<HmiManager>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HmiManager::new())));

fn capability_error(operation: &str) -> Option<Deger> {
    capability::require(Capability::Ffi, operation)
        .err()
        .map(Deger::Hata)
}

pub fn kayit_et(globals: &mut HashMap<String, Deger>) {
    globals.insert(
        "hmi_başlat".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(name), Deger::Metin(path)] = args.as_slice() else {
                return Deger::Hata(
                    "hmi_başlat: modül adı ve yürütülebilir yol olmak üzere 2 metin gerekir"
                        .to_string(),
                );
            };
            if let Some(error) = capability_error("hmi_başlat") {
                return error;
            }
            match HMI_YONETICI
                .lock()
                .map_err(|_| "HMI yönetici kilidi bozuldu".to_string())
                .and_then(|mut manager| manager.baslat(name, Path::new(path)))
            {
                Ok(()) => Deger::Sayi(1.0),
                Err(error) => Deger::Hata(format!("hmi_başlat: {error}")),
            }
        }),
    );
    globals.insert(
        "hmi_çağır".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (Some(Deger::Metin(module)), Some(Deger::Metin(function))) =
                (args.first(), args.get(1))
            else {
                return Deger::Hata(
                    "hmi_çağır: modül ve fonksiyon adı olmak üzere en az 2 metin gerekir"
                        .to_string(),
                );
            };
            if let Some(error) = capability_error("hmi_çağır") {
                return error;
            }
            match HMI_YONETICI
                .lock()
                .map_err(|_| "HMI yönetici kilidi bozuldu".to_string())
                .and_then(|mut manager| manager.cagir(module, function, args[2..].to_vec()))
            {
                Ok(value) => value,
                Err(error) => Deger::Hata(format!("hmi_çağır: {error}")),
            }
        }),
    );
    globals.insert(
        "hmi_kapat".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(name)] = args.as_slice() else {
                return Deger::Hata("hmi_kapat: tam olarak bir modül adı gerekir".to_string());
            };
            if let Some(error) = capability_error("hmi_kapat") {
                return error;
            }
            match HMI_YONETICI
                .lock()
                .map_err(|_| "HMI yönetici kilidi bozuldu".to_string())
                .and_then(|mut manager| manager.kapat(name))
            {
                Ok(()) => Deger::Sayi(1.0),
                Err(error) => Deger::Hata(format!("hmi_kapat: {error}")),
            }
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dongusel_deger_surec_sinirindan_gecmez() {
        let list = Gc::new(Vec::new());
        list.borrow_mut().push(Deger::Liste(list.clone()));
        assert!(to_hmi(&Deger::Liste(list), &mut HashSet::new(), 0, &mut 0)
            .unwrap_err()
            .contains("Döngüsel"));
    }
}
