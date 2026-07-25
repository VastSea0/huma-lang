use crate::ast::{Ifade, Komut};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

const MAX_JSON_ITEMS: usize = 1_000_000;
const MAX_JSON_TEXT_BYTES: usize = 64 * 1024 * 1024;

/// Çalışma zamanı durumuna ihtiyaç duyan yerleşik fonksiyonlar için dar,
/// nesne-güvenli arayüz. Yerleşikler yorumlayıcıya veya VM'e ham işaretçiyle
/// yeniden girmek zorunda kalmaz.
pub trait BuiltinRuntime {
    fn call_value(&mut self, function: Deger, args: Vec<Deger>) -> Deger;
}

#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Debug, Clone, PartialEq)]
pub enum Deger {
    Sayi(f64),
    Metin(String),
    Bayt(Vec<u8>),
    Liste(Rc<RefCell<Vec<Deger>>>),
    GorevId(u64),
    Bos,
    Fonksiyon {
        parametreler: Vec<String>,
        govde: Vec<Komut>,
        /// Fonksiyonun tanımlandığı anda görünür olan sözcüksel kapsamlar.
        yakalanan_kapsamlar: Vec<HashMap<String, Deger>>,
        /// Ad alanlı modüllerde özel global bağların canlı ortam kimliği.
        module_kimligi: Option<String>,
    },
    /// VM içinde yürütülen, düz bytecode fonksiyon tablosuna bağlı closure.
    BytecodeFonksiyon {
        ad: Option<String>,
        function_index: usize,
        yakalanan_degiskenler: HashMap<String, Deger>,
    },
    DahiliFonksiyon(fn(Vec<Deger>) -> Deger),
    BaglamliDahiliFonksiyon(fn(&mut dyn BuiltinRuntime, Vec<Deger>) -> Deger),
    Sinif {
        ad: String,
        metotlar: HashMap<String, (Vec<String>, Vec<Komut>)>,
        alan_baslangic: Vec<(String, Ifade)>,
        module_kimligi: Option<String>,
    },
    Nesne {
        sinif_adi: String,
        alanlar: Rc<RefCell<HashMap<String, Deger>>>,
        module_kimligi: Option<String>,
    },
    Sozluk(Rc<RefCell<HashMap<String, Deger>>>),
    Hata(String),
    /// Bitişik f64 vektörü — boxing olmadan ML hesaplamaları için
    Vektor(Rc<RefCell<Vec<f64>>>),
    /// 2D matris — satır-önce (row-major) düzende saklanan f64 dizisi
    Matris {
        satirlar: usize,
        sutunlar: usize,
        veri: Rc<RefCell<Vec<f64>>>,
    },
    Tensor(crate::autograd::TensorData),
}

fn deger_yaz(
    value: &Deger,
    f: &mut std::fmt::Formatter<'_>,
    active: &mut HashSet<usize>,
    depth: usize,
) -> std::fmt::Result {
    if depth > 128 {
        return write!(f, "<azami gösterim derinliği aşıldı>");
    }
    match value {
        Deger::Sayi(n) => {
            if n.is_finite() && *n == (*n as i64) as f64 {
                write!(f, "{}", *n as i64)
            } else {
                write!(f, "{}", n)
            }
        }
        Deger::Metin(s) => write!(f, "{}", s),
        Deger::Bayt(b) => write!(f, "<bayt veri: {} bayt>", b.len()),
        Deger::Liste(l) => {
            let identity = Rc::as_ptr(l) as *const () as usize;
            if !active.insert(identity) {
                return write!(f, "<döngüsel liste>");
            }
            let borrowed = match l.try_borrow() {
                Ok(borrowed) => borrowed,
                Err(_) => {
                    active.remove(&identity);
                    return write!(f, "<liste kullanımda>");
                }
            };
            write!(f, "[")?;
            for (index, item) in borrowed.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }
                deger_yaz(item, f, active, depth + 1)?;
            }
            write!(f, "]")?;
            active.remove(&identity);
            Ok(())
        }
        Deger::GorevId(id) => write!(f, "<görev:{}>", id),
        Deger::Bos => write!(f, "Boş"),
        Deger::Nesne { sinif_adi, .. } => write!(f, "<{} nesnesi>", sinif_adi),
        Deger::Sozluk(m) => {
            let identity = Rc::as_ptr(m) as *const () as usize;
            if !active.insert(identity) {
                return write!(f, "<döngüsel sözlük>");
            }
            let borrowed = match m.try_borrow() {
                Ok(borrowed) => borrowed,
                Err(_) => {
                    active.remove(&identity);
                    return write!(f, "<sözlük kullanımda>");
                }
            };
            let mut entries = borrowed.iter().collect::<Vec<_>>();
            entries.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
            write!(f, "{{")?;
            for (index, (key, item)) in entries.into_iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{key:?}: ")?;
                deger_yaz(item, f, active, depth + 1)?;
            }
            write!(f, "}}")?;
            active.remove(&identity);
            Ok(())
        }
        Deger::Hata(e) => write!(f, "Hata: {}", e),
        Deger::Vektor(v) => {
            let b = match v.try_borrow() {
                Ok(borrowed) => borrowed,
                Err(_) => return write!(f, "<vektör kullanımda>"),
            };
            let mut s = String::from("vektor[");
            for (i, x) in b.iter().enumerate() {
                if i > 0 {
                    s.push_str(", ");
                }
                s.push_str(&format!("{x:.6}"));
            }
            s.push(']');
            write!(f, "{}", s)
        }
        Deger::Matris {
            satirlar,
            sutunlar,
            veri,
        } => {
            write!(f, "matris[{}×{}]", satirlar, sutunlar)?;
            let expected = match satirlar.checked_mul(*sutunlar) {
                Some(expected) => expected,
                None => return write!(f, " <bozuk boyut>"),
            };
            let b = match veri.try_borrow() {
                Ok(borrowed) => borrowed,
                Err(_) => return write!(f, " <veri kullanımda>"),
            };
            if b.len() != expected {
                return write!(
                    f,
                    " <bozuk veri: {} eleman bekleniyordu, {} bulundu>",
                    expected,
                    b.len()
                );
            }
            for i in 0..*satirlar {
                write!(f, "\n  [")?;
                for j in 0..*sutunlar {
                    if j > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:.4}", b[i * sutunlar + j])?;
                }
                write!(f, "]")?;
            }
            Ok(())
        }
        Deger::Tensor(t) => {
            write!(
                f,
                "tensor[{}×{}, id={}, requires_grad={}]",
                t.satirlar, t.sutunlar, t.id, t.requires_grad
            )?;
            let expected = match t.satirlar.checked_mul(t.sutunlar) {
                Some(expected) => expected,
                None => return write!(f, " <bozuk boyut>"),
            };
            let b = match t.veri.lock() {
                Ok(locked) => locked,
                Err(_) => return write!(f, " <veri kilidi bozuk>"),
            };
            if b.len() != expected {
                return write!(
                    f,
                    " <bozuk veri: {} eleman bekleniyordu, {} bulundu>",
                    expected,
                    b.len()
                );
            }
            for i in 0..t.satirlar {
                write!(f, "\n  [")?;
                for j in 0..t.sutunlar {
                    if j > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:.4}", b[i * t.sutunlar + j])?;
                }
                write!(f, "]")?;
            }
            Ok(())
        }
        Deger::Fonksiyon { .. } | Deger::BytecodeFonksiyon { .. } => {
            write!(f, "<fonksiyon>")
        }
        Deger::BaglamliDahiliFonksiyon(_) => write!(f, "<bağlamlı-dahili>"),
        _ => write!(f, "<dahili>"),
    }
}

impl std::fmt::Display for Deger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        deger_yaz(self, f, &mut HashSet::new(), 0)
    }
}

impl Deger {
    /// Değeri, ara belleğin denetimsiz büyümesine izin vermeden gösterir.
    pub fn to_string_limited(&self, limit: usize) -> Result<String, String> {
        struct LimitedFormatter {
            output: String,
            limit: usize,
        }

        impl std::fmt::Write for LimitedFormatter {
            fn write_str(&mut self, text: &str) -> std::fmt::Result {
                let next_size = self
                    .output
                    .len()
                    .checked_add(text.len())
                    .ok_or(std::fmt::Error)?;
                if next_size > self.limit {
                    return Err(std::fmt::Error);
                }
                self.output.push_str(text);
                Ok(())
            }
        }

        let mut formatter = LimitedFormatter {
            output: String::new(),
            limit,
        };
        std::fmt::write(&mut formatter, format_args!("{self}"))
            .map_err(|_| format!("Gösterim {limit} bayt sınırını aşıyor"))?;
        Ok(formatter.output)
    }

    /// Convert a runtime value to JSON without silently replacing invalid data.
    ///
    /// Cyclic values, non-finite numbers, borrowed containers and values with no
    /// JSON representation are explicit errors.
    pub fn to_json_checked(&self) -> Result<serde_json::Value, String> {
        struct Budget {
            items: usize,
            text_bytes: usize,
        }

        impl Budget {
            fn items(&mut self, count: usize) -> Result<(), String> {
                self.items = self
                    .items
                    .checked_add(count)
                    .ok_or_else(|| "JSON öğe sayısı taştı".to_string())?;
                if self.items > MAX_JSON_ITEMS {
                    return Err(format!(
                        "JSON öğe sayısı {MAX_JSON_ITEMS} güvenlik sınırını aşıyor"
                    ));
                }
                Ok(())
            }

            fn text(&mut self, count: usize) -> Result<(), String> {
                self.text_bytes = self
                    .text_bytes
                    .checked_add(count)
                    .ok_or_else(|| "JSON metin/bayt boyutu taştı".to_string())?;
                if self.text_bytes > MAX_JSON_TEXT_BYTES {
                    return Err(format!(
                        "JSON metin/bayt boyutu {MAX_JSON_TEXT_BYTES} güvenlik sınırını aşıyor"
                    ));
                }
                Ok(())
            }
        }

        fn number(value: f64) -> Result<serde_json::Value, String> {
            serde_json::Number::from_f64(value)
                .map(serde_json::Value::Number)
                .ok_or_else(|| "JSON yalnızca sonlu sayıları destekler".to_string())
        }

        fn convert(
            value: &Deger,
            active: &mut HashSet<usize>,
            depth: usize,
            budget: &mut Budget,
        ) -> Result<serde_json::Value, String> {
            if depth > 128 {
                return Err("JSON dönüşümünde azami iç içe değer derinliği aşıldı".to_string());
            }
            budget.items(1)?;
            match value {
                Deger::Sayi(n) => number(*n),
                Deger::Metin(s) => {
                    budget.text(s.len())?;
                    Ok(serde_json::Value::String(s.clone()))
                }
                Deger::Bayt(bytes) => {
                    budget.items(bytes.len())?;
                    budget.text(bytes.len())?;
                    Ok(serde_json::Value::Array(
                        bytes
                            .iter()
                            .map(|byte| serde_json::Value::Number((*byte).into()))
                            .collect(),
                    ))
                }
                Deger::Liste(items) => {
                    let identity = Rc::as_ptr(items) as *const () as usize;
                    if !active.insert(identity) {
                        return Err("Döngüsel liste JSON'a dönüştürülemez".to_string());
                    }
                    let borrowed = items
                        .try_borrow()
                        .map_err(|_| "Liste JSON dönüşümü sırasında kullanımda".to_string())?;
                    let result = borrowed
                        .iter()
                        .map(|item| convert(item, active, depth + 1, budget))
                        .collect::<Result<Vec<_>, _>>()
                        .map(serde_json::Value::Array);
                    active.remove(&identity);
                    result
                }
                Deger::Bos => Ok(serde_json::Value::Null),
                Deger::Nesne { alanlar, .. } => {
                    let identity = Rc::as_ptr(alanlar) as *const () as usize;
                    if !active.insert(identity) {
                        return Err("Döngüsel nesne JSON'a dönüştürülemez".to_string());
                    }
                    let borrowed = alanlar
                        .try_borrow()
                        .map_err(|_| "Nesne JSON dönüşümü sırasında kullanımda".to_string())?;
                    let mut map = serde_json::Map::new();
                    let result = (|| {
                        for (key, item) in borrowed.iter() {
                            budget.text(key.len())?;
                            map.insert(key.clone(), convert(item, active, depth + 1, budget)?);
                        }
                        Ok(serde_json::Value::Object(map))
                    })();
                    active.remove(&identity);
                    result
                }
                Deger::Sozluk(items) => {
                    let identity = Rc::as_ptr(items) as *const () as usize;
                    if !active.insert(identity) {
                        return Err("Döngüsel sözlük JSON'a dönüştürülemez".to_string());
                    }
                    let borrowed = items
                        .try_borrow()
                        .map_err(|_| "Sözlük JSON dönüşümü sırasında kullanımda".to_string())?;
                    let mut map = serde_json::Map::new();
                    let result = (|| {
                        for (key, item) in borrowed.iter() {
                            budget.text(key.len())?;
                            map.insert(key.clone(), convert(item, active, depth + 1, budget)?);
                        }
                        Ok(serde_json::Value::Object(map))
                    })();
                    active.remove(&identity);
                    result
                }
                Deger::Vektor(items) => {
                    let borrowed = items
                        .try_borrow()
                        .map_err(|_| "Vektör JSON dönüşümü sırasında kullanımda".to_string())?;
                    budget.items(borrowed.len())?;
                    borrowed
                        .iter()
                        .map(|item| number(*item))
                        .collect::<Result<Vec<_>, _>>()
                        .map(serde_json::Value::Array)
                }
                Deger::Matris {
                    satirlar,
                    sutunlar,
                    veri,
                } => {
                    let borrowed = veri
                        .try_borrow()
                        .map_err(|_| "Matris JSON dönüşümü sırasında kullanımda".to_string())?;
                    if borrowed.len() != satirlar.saturating_mul(*sutunlar) {
                        return Err("Matris veri uzunluğu boyutlarıyla uyuşmuyor".to_string());
                    }
                    budget.items(borrowed.len().saturating_add(*satirlar))?;
                    let mut rows = Vec::with_capacity(*satirlar);
                    for row in 0..*satirlar {
                        let mut cols = Vec::with_capacity(*sutunlar);
                        for col in 0..*sutunlar {
                            cols.push(number(borrowed[row * sutunlar + col])?);
                        }
                        rows.push(serde_json::Value::Array(cols));
                    }
                    Ok(serde_json::Value::Array(rows))
                }
                Deger::Tensor(tensor) => {
                    let borrowed = tensor
                        .veri
                        .lock()
                        .map_err(|_| "Tensor JSON dönüşüm kilidi bozuldu".to_string())?;
                    if borrowed.len() != tensor.satirlar.saturating_mul(tensor.sutunlar) {
                        return Err("Tensor veri uzunluğu boyutlarıyla uyuşmuyor".to_string());
                    }
                    budget.items(borrowed.len().saturating_add(tensor.satirlar))?;
                    let mut rows = Vec::with_capacity(tensor.satirlar);
                    for row in 0..tensor.satirlar {
                        let mut cols = Vec::with_capacity(tensor.sutunlar);
                        for col in 0..tensor.sutunlar {
                            cols.push(number(borrowed[row * tensor.sutunlar + col])?);
                        }
                        rows.push(serde_json::Value::Array(cols));
                    }
                    Ok(serde_json::Value::Array(rows))
                }
                Deger::GorevId(_)
                | Deger::Fonksiyon { .. }
                | Deger::BytecodeFonksiyon { .. }
                | Deger::DahiliFonksiyon(_)
                | Deger::BaglamliDahiliFonksiyon(_)
                | Deger::Sinif { .. } => {
                    Err("Bu çalışma zamanı değeri JSON ile temsil edilemez".to_string())
                }
                Deger::Hata(message) => Err(format!("Hata değeri JSON'a yazılamaz: {message}")),
            }
        }

        convert(
            self,
            &mut HashSet::new(),
            0,
            &mut Budget {
                items: 0,
                text_bytes: 0,
            },
        )
    }

    pub fn from_json_checked(v: &serde_json::Value) -> Result<Deger, String> {
        fn convert(
            value: &serde_json::Value,
            depth: usize,
            items: &mut usize,
            text_bytes: &mut usize,
        ) -> Result<Deger, String> {
            if depth > 128 {
                return Err("JSON dönüşümünde azami iç içe değer derinliği aşıldı".to_string());
            }
            *items = items
                .checked_add(1)
                .ok_or_else(|| "JSON öğe sayısı taştı".to_string())?;
            if *items > MAX_JSON_ITEMS {
                return Err(format!(
                    "JSON öğe sayısı {MAX_JSON_ITEMS} güvenlik sınırını aşıyor"
                ));
            }
            match value {
                serde_json::Value::Number(number) => number
                    .as_f64()
                    .filter(|value| value.is_finite())
                    .map(Deger::Sayi)
                    .ok_or_else(|| "JSON sayısı Hüma f64 değerine dönüştürülemedi".to_string()),
                serde_json::Value::String(text) => {
                    *text_bytes = text_bytes
                        .checked_add(text.len())
                        .ok_or_else(|| "JSON metin boyutu taştı".to_string())?;
                    if *text_bytes > MAX_JSON_TEXT_BYTES {
                        return Err(format!(
                            "JSON metin boyutu {MAX_JSON_TEXT_BYTES} güvenlik sınırını aşıyor"
                        ));
                    }
                    Ok(Deger::Metin(text.clone()))
                }
                serde_json::Value::Array(array) => {
                    let values = array
                        .iter()
                        .map(|item| convert(item, depth + 1, items, text_bytes))
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(Deger::Liste(Rc::new(RefCell::new(values))))
                }
                serde_json::Value::Bool(boolean) => {
                    Ok(Deger::Sayi(if *boolean { 1.0 } else { 0.0 }))
                }
                serde_json::Value::Object(object) => {
                    let mut map = HashMap::new();
                    for (key, item) in object {
                        *text_bytes = text_bytes
                            .checked_add(key.len())
                            .ok_or_else(|| "JSON metin boyutu taştı".to_string())?;
                        if *text_bytes > MAX_JSON_TEXT_BYTES {
                            return Err(format!(
                                "JSON metin boyutu {MAX_JSON_TEXT_BYTES} güvenlik sınırını aşıyor"
                            ));
                        }
                        map.insert(key.clone(), convert(item, depth + 1, items, text_bytes)?);
                    }
                    Ok(Deger::Sozluk(Rc::new(RefCell::new(map))))
                }
                serde_json::Value::Null => Ok(Deger::Bos),
            }
        }

        convert(v, 0, &mut 0, &mut 0)
    }
}
