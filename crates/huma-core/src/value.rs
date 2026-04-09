use crate::ast::{Ifade, Komut};
use std::collections::{HashMap};
use std::rc::Rc;
use std::cell::RefCell;

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
    },
    DahiliFonksiyon(fn(Vec<Deger>) -> Deger),
    Sinif {
        ad: String,
        metotlar: HashMap<String, (Vec<String>, Vec<Komut>)>,
        alan_baslangic: Vec<(String, Ifade)>,
    },
    Nesne {
        sinif_adi: String,
        alanlar: Rc<RefCell<HashMap<String, Deger>>>,
    },
    Sozluk(Rc<RefCell<HashMap<String, Deger>>>),
    Hata(String),
}

impl std::fmt::Display for Deger {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Deger::Sayi(n) => {
                if *n == (*n as i64) as f64 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            },
            Deger::Metin(s) => write!(f, "{}", s),
            Deger::Bayt(b) => write!(f, "<bayt veri: {} bayt>", b.len()),
            Deger::Liste(l) => {
                let l_borrow = l.borrow();
                let p: Vec<String> = l_borrow.iter().map(|d| d.to_string()).collect();
                write!(f, "[{}]", p.join(", "))
            }
            Deger::GorevId(id) => write!(f, "<görev:{}>", id),
            Deger::Bos => write!(f, "Boş"),
            Deger::Nesne { sinif_adi, .. } => write!(f, "<{} nesnesi>", sinif_adi),
            Deger::Sozluk(m) => {
                let m_borrow = m.borrow();
                let p: Vec<String> = m_borrow.iter().map(|(k, v)| format!("\"{}\": {}", k, v)).collect();
                write!(f, "{{{}}}", p.join(", "))
            }
            Deger::Hata(e) => write!(f, "Hata: {}", e),
            _ => write!(f, "<dahili>"),
        }
    }
}

impl Deger {
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Deger::Sayi(n) => serde_json::Value::Number(serde_json::Number::from_f64(*n).unwrap_or(serde_json::Number::from(0))),
            Deger::Metin(s) => serde_json::Value::String(s.clone()),
            Deger::Liste(l) => {
                let l_borrow = l.borrow();
                let v: Vec<serde_json::Value> = l_borrow.iter().map(|d| d.to_json()).collect();
                serde_json::Value::Array(v)
            }
            Deger::GorevId(_) => serde_json::Value::Null,
            Deger::Bos => serde_json::Value::Null,
            Deger::Nesne { alanlar, .. } => {
                let mut map = serde_json::Map::new();
                for (k, v) in alanlar.borrow().iter() {
                    map.insert(k.clone(), v.to_json());
                }
                serde_json::Value::Object(map)
            }
            Deger::Sozluk(m) => {
                let mut map = serde_json::Map::new();
                for (k, v) in m.borrow().iter() {
                    map.insert(k.clone(), v.to_json());
                }
                serde_json::Value::Object(map)
            }
            Deger::Hata(e) => serde_json::Value::String(format!("Hata: {}", e)),
            _ => serde_json::Value::Null,
        }
    }

    pub fn from_json(v: &serde_json::Value) -> Deger {
        match v {
            serde_json::Value::Number(n) => Deger::Sayi(n.as_f64().unwrap_or(0.0)),
            serde_json::Value::String(s) => Deger::Metin(s.clone()),
            serde_json::Value::Array(a) => {
                let v: Vec<Deger> = a.iter().map(|item| Deger::from_json(item)).collect();
                Deger::Liste(Rc::new(RefCell::new(v)))
            }
            serde_json::Value::Bool(b) => Deger::Sayi(if *b { 1.0 } else { 0.0 }),
            serde_json::Value::Object(o) => {
                let mut map = HashMap::new();
                for (k, v) in o.iter() {
                    map.insert(k.clone(), Deger::from_json(v));
                }
                Deger::Sozluk(Rc::new(RefCell::new(map)))
            }
            _ => Deger::Bos,
        }
    }
}

