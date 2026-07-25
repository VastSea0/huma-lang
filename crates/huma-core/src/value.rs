use crate::ast::{Ifade, Komut};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::rc::Rc;

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

impl std::fmt::Display for Deger {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Deger::Sayi(n) => {
                if *n == (*n as i64) as f64 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
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
                let p: Vec<String> = m_borrow
                    .iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, v))
                    .collect();
                write!(f, "{{{}}}", p.join(", "))
            }
            Deger::Hata(e) => write!(f, "Hata: {}", e),
            Deger::Vektor(v) => {
                let b = v.borrow();
                let mut s = String::from("vektor[");
                for (i, x) in b.iter().enumerate() {
                    if i > 0 {
                        s.push_str(", ");
                    }
                    let _ = write!(s, "{:.6}", x);
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
                let b = veri.borrow();
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
                let b = t.veri.lock().unwrap();
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
            _ => write!(f, "<dahili>"),
        }
    }
}

impl Deger {
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Deger::Sayi(n) => serde_json::Value::Number(
                serde_json::Number::from_f64(*n).unwrap_or(serde_json::Number::from(0)),
            ),
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
            Deger::Vektor(v) => {
                let arr: Vec<serde_json::Value> = v
                    .borrow()
                    .iter()
                    .map(|x| {
                        serde_json::Value::Number(
                            serde_json::Number::from_f64(*x).unwrap_or(serde_json::Number::from(0)),
                        )
                    })
                    .collect();
                serde_json::Value::Array(arr)
            }
            Deger::Matris {
                satirlar,
                sutunlar,
                veri,
            } => {
                let b = veri.borrow();
                let rows: Vec<serde_json::Value> = (0..*satirlar)
                    .map(|i| {
                        let cols: Vec<serde_json::Value> = (0..*sutunlar)
                            .map(|j| {
                                serde_json::Value::Number(
                                    serde_json::Number::from_f64(b[i * sutunlar + j])
                                        .unwrap_or(serde_json::Number::from(0)),
                                )
                            })
                            .collect();
                        serde_json::Value::Array(cols)
                    })
                    .collect();
                serde_json::Value::Array(rows)
            }
            _ => serde_json::Value::Null,
        }
    }

    pub fn from_json(v: &serde_json::Value) -> Deger {
        match v {
            serde_json::Value::Number(n) => Deger::Sayi(n.as_f64().unwrap_or(0.0)),
            serde_json::Value::String(s) => Deger::Metin(s.clone()),
            serde_json::Value::Array(a) => {
                let v: Vec<Deger> = a.iter().map(Deger::from_json).collect();
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
