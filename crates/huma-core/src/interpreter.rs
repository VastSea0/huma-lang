use crate::ast::{Ifade, Komut};
use crate::builtin_files;
use crate::error::{HumaError, HumaResult};
use crate::token::Token;
use crate::value::Deger;
use futures_util::FutureExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use once_cell::sync::Lazy;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use regex::Regex;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};
use std::path::Path;
use std::rc::Rc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::task::LocalSet;
use unicode_normalization::UnicodeNormalization;

// Async server (hyper) state
struct IncomingRequest {
    id: u64,
    url: String,
    metot: String,
    govde: String,
    respond_to: oneshot::Sender<Response<Body>>,
}

static SUNUCULAR: Lazy<Mutex<HashMap<u64, mpsc::UnboundedSender<IncomingRequest>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static SUNUCU_RX: Lazy<tokio::sync::Mutex<HashMap<u64, mpsc::UnboundedReceiver<IncomingRequest>>>> =
    Lazy::new(|| tokio::sync::Mutex::new(HashMap::new()));
static YANITLAR: Lazy<Mutex<HashMap<u64, oneshot::Sender<Response<Body>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static SQL_CONNECTIONS: Lazy<Mutex<HashMap<u64, rusqlite::Connection>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static NEXT_ID: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(1));

struct YaprakExecutor {
    rt: Runtime,
    local: LocalSet,
    next_id: u64,
    tasks: HashMap<u64, JoinHandle<Deger>>,
}

impl YaprakExecutor {
    fn new() -> Self {
        Self {
            rt: Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio runtime init failed"),
            local: LocalSet::new(),
            next_id: 1,
            tasks: HashMap::new(),
        }
    }

    fn spawn<F>(&mut self, fut: F) -> Deger
    where
        F: std::future::Future<Output = Deger> + 'static,
    {
        let id = self.next_id;
        self.next_id += 1;
        let handle = self.local.spawn_local(fut);
        self.tasks.insert(id, handle);
        Deger::GorevId(id)
    }

    fn await_task(&mut self, id: u64) -> Deger {
        match self.tasks.remove(&id) {
            Some(handle) => match self.rt.block_on(self.local.run_until(handle)) {
                Ok(v) => v,
                Err(e) => Deger::Hata(format!("Görev hatası: {}", e)),
            },
            None => Deger::Hata(format!("Bilinmeyen görev: {}", id)),
        }
    }
}

thread_local! {
    static YAPRAK: std::cell::RefCell<YaprakExecutor> = std::cell::RefCell::new(YaprakExecutor::new());
}

const EN_FAZLA_TENSOR_ELEMANI: usize = 10_000_000;

fn boyut_dogrula(deger: f64, islem: &str, sifira_izin_ver: bool) -> Result<usize, String> {
    let alt_sinir = if sifira_izin_ver { 0.0 } else { 1.0 };
    if !deger.is_finite() || deger.fract() != 0.0 || deger < alt_sinir {
        let nitelik = if sifira_izin_ver {
            "negatif olmayan tamsayı"
        } else {
            "pozitif tamsayı"
        };
        return Err(format!("{islem}: boyut {nitelik} olmalı"));
    }
    if deger > EN_FAZLA_TENSOR_ELEMANI as f64 {
        return Err(format!(
            "{islem}: boyut güvenlik sınırını ({EN_FAZLA_TENSOR_ELEMANI}) aşıyor"
        ));
    }
    Ok(deger as usize)
}

fn eleman_sayisi_dogrula(satirlar: usize, sutunlar: usize, islem: &str) -> Result<usize, String> {
    let eleman_sayisi = satirlar
        .checked_mul(sutunlar)
        .ok_or_else(|| format!("{islem}: boyut çarpımı taştı"))?;
    if eleman_sayisi > EN_FAZLA_TENSOR_ELEMANI {
        return Err(format!(
            "{islem}: {eleman_sayisi} eleman güvenlik sınırını \
             ({EN_FAZLA_TENSOR_ELEMANI}) aşıyor"
        ));
    }
    Ok(eleman_sayisi)
}

fn indeks_dogrula(deger: f64, uzunluk: usize, islem: &str) -> Result<usize, String> {
    if !deger.is_finite() || deger < 0.0 || deger.fract() != 0.0 {
        return Err(format!("{islem}: indeks negatif olmayan tamsayı olmalı"));
    }
    if deger >= uzunluk as f64 {
        return Err(format!(
            "{islem}: indeks {deger} sınır dışında (uzunluk {uzunluk})"
        ));
    }
    Ok(deger as usize)
}

fn get_id() -> u64 {
    let mut id = NEXT_ID.lock().unwrap();
    let old = *id;
    *id += 1;
    old
}

fn adam_matris_durumu(args: Vec<Deger>) -> Deger {
    let (satirlar, sutunlar) = match (args.first(), args.get(1)) {
        (Some(Deger::Sayi(r)), Some(Deger::Sayi(c))) => {
            let satirlar = match boyut_dogrula(*r, "adam_durum_olustur", false) {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            let sutunlar = match boyut_dogrula(*c, "adam_durum_olustur", false) {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            (satirlar, sutunlar)
        }
        _ => {
            return Deger::Hata("adam_durum_olustur: pozitif tamsayı boyutlar gerekir".to_string())
        }
    };
    let eleman_sayisi = match eleman_sayisi_dogrula(satirlar, sutunlar, "adam_durum_olustur") {
        Ok(deger) => deger,
        Err(hata) => return Deger::Hata(hata),
    };
    let mut durum = HashMap::new();
    durum.insert(
        "m".to_string(),
        Deger::Matris {
            satirlar,
            sutunlar,
            veri: Rc::new(RefCell::new(vec![0.0; eleman_sayisi])),
        },
    );
    durum.insert(
        "v".to_string(),
        Deger::Matris {
            satirlar,
            sutunlar,
            veri: Rc::new(RefCell::new(vec![0.0; eleman_sayisi])),
        },
    );
    durum.insert("adim".to_string(), Deger::Sayi(0.0));
    Deger::Sozluk(Rc::new(RefCell::new(durum)))
}

fn adam_vektor_durumu(args: Vec<Deger>) -> Deger {
    let boyut = match args.first() {
        Some(Deger::Sayi(n)) => match boyut_dogrula(*n, "adam_vektor_durum_olustur", false) {
            Ok(deger) => deger,
            Err(hata) => return Deger::Hata(hata),
        },
        _ => {
            return Deger::Hata(
                "adam_vektor_durum_olustur: pozitif tamsayı boyut gerekir".to_string(),
            )
        }
    };
    let mut durum = HashMap::new();
    durum.insert(
        "m".to_string(),
        Deger::Vektor(Rc::new(RefCell::new(vec![0.0; boyut]))),
    );
    durum.insert(
        "v".to_string(),
        Deger::Vektor(Rc::new(RefCell::new(vec![0.0; boyut]))),
    );
    durum.insert("adim".to_string(), Deger::Sayi(0.0));
    Deger::Sozluk(Rc::new(RefCell::new(durum)))
}

fn adam_matris_guncelle(args: Vec<Deger>) -> Deger {
    let (
        Deger::Matris {
            satirlar: wr,
            sutunlar: wc,
            veri: w,
        },
        Deger::Matris {
            satirlar: gr,
            sutunlar: gc,
            veri: g,
        },
        Deger::Sozluk(durum),
        Deger::Sayi(ogrenme_hizi),
    ) = (match (args.first(), args.get(1), args.get(2), args.get(3)) {
        (Some(w), Some(g), Some(d), Some(lr)) => (w, g, d, lr),
        _ => {
            return Deger::Hata(
                "adam_matris_guncelle: W, gradyan, durum ve öğrenme hızı gerekir".to_string(),
            )
        }
    })
    else {
        return Deger::Hata("adam_matris_guncelle: geçersiz argüman türleri".to_string());
    };
    if wr != gr || wc != gc {
        return Deger::Hata(
            "adam_matris_guncelle: ağırlık ve gradyan boyutları eşit olmalı".to_string(),
        );
    }
    if !ogrenme_hizi.is_finite() || *ogrenme_hizi <= 0.0 {
        return Deger::Hata(
            "adam_matris_guncelle: öğrenme hızı pozitif ve sonlu olmalı".to_string(),
        );
    }

    let (m_degeri, v_degeri, adim) = {
        let mut map = durum.borrow_mut();
        let adim = match map.get("adim") {
            Some(Deger::Sayi(t)) => *t + 1.0,
            _ => 1.0,
        };
        map.insert("adim".to_string(), Deger::Sayi(adim));
        (map.get("m").cloned(), map.get("v").cloned(), adim)
    };
    let (Some(Deger::Matris { veri: m, .. }), Some(Deger::Matris { veri: v, .. })) =
        (m_degeri, v_degeri)
    else {
        return Deger::Hata("adam_matris_guncelle: bozuk optimizör durumu".to_string());
    };

    let beta1: f64 = 0.9;
    let beta2: f64 = 0.999;
    let epsilon = 1e-8;
    let mut w = w.borrow_mut();
    let g = g.borrow();
    let mut m = m.borrow_mut();
    let mut v = v.borrow_mut();
    if w.len() != g.len() || w.len() != m.len() || w.len() != v.len() {
        return Deger::Hata("adam_matris_guncelle: durum boyutu uyuşmuyor".to_string());
    }
    if g.iter().any(|deger| !deger.is_finite()) {
        return Deger::Hata("adam_matris_guncelle: gradyanlar sonlu olmalı".to_string());
    }
    let duzeltme1 = 1.0 - beta1.powf(adim);
    let duzeltme2 = 1.0 - beta2.powf(adim);
    for index in 0..w.len() {
        m[index] = beta1 * m[index] + (1.0 - beta1) * g[index];
        v[index] = beta2 * v[index] + (1.0 - beta2) * g[index] * g[index];
        let m_hat = m[index] / duzeltme1;
        let v_hat = v[index] / duzeltme2;
        w[index] -= *ogrenme_hizi * m_hat / (v_hat.sqrt() + epsilon);
    }
    Deger::Bos
}

fn adam_vektor_guncelle(args: Vec<Deger>) -> Deger {
    let (Deger::Vektor(w), Deger::Vektor(g), Deger::Sozluk(durum), Deger::Sayi(ogrenme_hizi)) =
        (match (args.first(), args.get(1), args.get(2), args.get(3)) {
            (Some(w), Some(g), Some(d), Some(lr)) => (w, g, d, lr),
            _ => {
                return Deger::Hata(
                    "adam_vektor_guncelle: vektör, gradyan, durum ve öğrenme hızı gerekir"
                        .to_string(),
                )
            }
        })
    else {
        return Deger::Hata("adam_vektor_guncelle: geçersiz argüman türleri".to_string());
    };
    if !ogrenme_hizi.is_finite() || *ogrenme_hizi <= 0.0 {
        return Deger::Hata(
            "adam_vektor_guncelle: öğrenme hızı pozitif ve sonlu olmalı".to_string(),
        );
    }

    let (m_degeri, v_degeri, adim) = {
        let mut map = durum.borrow_mut();
        let adim = match map.get("adim") {
            Some(Deger::Sayi(t)) => *t + 1.0,
            _ => 1.0,
        };
        map.insert("adim".to_string(), Deger::Sayi(adim));
        (map.get("m").cloned(), map.get("v").cloned(), adim)
    };
    let (Some(Deger::Vektor(m)), Some(Deger::Vektor(v))) = (m_degeri, v_degeri) else {
        return Deger::Hata("adam_vektor_guncelle: bozuk optimizör durumu".to_string());
    };

    let beta1: f64 = 0.9;
    let beta2: f64 = 0.999;
    let epsilon = 1e-8;
    let mut w = w.borrow_mut();
    let g = g.borrow();
    let mut m = m.borrow_mut();
    let mut v = v.borrow_mut();
    if w.len() != g.len() || w.len() != m.len() || w.len() != v.len() {
        return Deger::Hata("adam_vektor_guncelle: durum boyutu uyuşmuyor".to_string());
    }
    if g.iter().any(|deger| !deger.is_finite()) {
        return Deger::Hata("adam_vektor_guncelle: gradyanlar sonlu olmalı".to_string());
    }
    let duzeltme1 = 1.0 - beta1.powf(adim);
    let duzeltme2 = 1.0 - beta2.powf(adim);
    for index in 0..w.len() {
        m[index] = beta1 * m[index] + (1.0 - beta1) * g[index];
        v[index] = beta2 * v[index] + (1.0 - beta2) * g[index] * g[index];
        let m_hat = m[index] / duzeltme1;
        let v_hat = v[index] / duzeltme2;
        w[index] -= *ogrenme_hizi * m_hat / (v_hat.sqrt() + epsilon);
    }
    Deger::Bos
}

pub struct Yorumlayici {
    pub global_degiskenler: HashMap<String, Deger>,
    pub yerel_scopes: Vec<HashMap<String, Deger>>,
    pub donus_degeri: Option<Deger>,
    pub yuklenen_dosyalar: HashSet<String>,
    pub arama_yolları: Vec<String>,
    pub output_buffer: Option<Rc<RefCell<String>>>,
    pub call_depth: usize,
    runtime_errors: Vec<String>,
    dongu_kontrolu: Option<DonguKontrolu>,
    dongu_derinligi: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DonguKontrolu {
    Devam,
    Kir,
}

pub fn varsayilan_global_degiskenler() -> HashMap<String, Deger> {
    let mut globals = HashMap::new();
    globals.insert("boş".to_string(), Deger::Bos);
    globals.insert("Boş".to_string(), Deger::Bos);
    globals.insert(
        "adam_durum_olustur".to_string(),
        Deger::DahiliFonksiyon(adam_matris_durumu),
    );
    globals.insert(
        "adam_vektor_durum_olustur".to_string(),
        Deger::DahiliFonksiyon(adam_vektor_durumu),
    );
    globals.insert(
        "adam_matris_guncelle".to_string(),
        Deger::DahiliFonksiyon(adam_matris_guncelle),
    );
    globals.insert(
        "adam_vektor_guncelle".to_string(),
        Deger::DahiliFonksiyon(adam_vektor_guncelle),
    );
    globals.insert(
        "uzunluk".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(value)] => Deger::Sayi(value.chars().count() as f64),
            [Deger::Bayt(value)] => Deger::Sayi(value.len() as f64),
            [Deger::Liste(value)] => Deger::Sayi(value.borrow().len() as f64),
            [Deger::Sozluk(value)] => Deger::Sayi(value.borrow().len() as f64),
            [Deger::Vektor(value)] => Deger::Sayi(value.borrow().len() as f64),
            [other] => Deger::Hata(format!(
                "uzunluk: metin, bayt, liste, sözlük veya vektör bekleniyordu; {} geldi",
                other
            )),
            _ => Deger::Hata(format!(
                "uzunluk: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );
    globals.insert(
        "oku".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(msg) = args.first() {
                print!("{}", msg);
                let _ = io::stdout().flush();
            }
            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_ok() {
                Deger::Metin(input.trim().to_string())
            } else {
                Deger::Bos
            }
        }),
    );
    globals.insert(
        "uyut".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Sayi(ms)] if ms.is_finite() && *ms >= 0.0 && *ms <= u64::MAX as f64 => {
                if *ms != 0.0 {
                    thread::sleep(Duration::from_millis(*ms as u64));
                }
                Deger::Bos
            }
            [Deger::Sayi(_)] => Deger::Hata(
                "uyut: milisaniye sonlu ve negatif olmayan bir sayı olmalıdır".to_string(),
            ),
            [other] => Deger::Hata(format!("uyut: sayı bekleniyordu; {} geldi", other)),
            _ => Deger::Hata(format!(
                "uyut: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );
    globals.insert(
        "zaman".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if !args.is_empty() {
                return Deger::Hata(format!(
                    "zaman: argüman beklenmiyordu; {} geldi",
                    args.len()
                ));
            }
            match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(duration) => Deger::Sayi(duration.as_secs_f64()),
                Err(error) => Deger::Hata(format!("zaman: sistem saati hatası: {}", error)),
            }
        }),
    );
    globals.insert(
        "listeye_ekle".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Liste(list), value] => {
                let mut yeni = list.borrow().clone();
                yeni.push(value.clone());
                Deger::Liste(Rc::new(RefCell::new(yeni)))
            }
            [other, _] => Deger::Hata(format!(
                "listeye_ekle: ilk argüman liste olmalıdır; {} geldi",
                other
            )),
            _ => Deger::Hata(format!(
                "listeye_ekle: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );
    globals.insert(
        "karekök".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Sayi(value)] if value.is_finite() && *value >= 0.0 => Deger::Sayi(value.sqrt()),
            [Deger::Sayi(_)] => {
                Deger::Hata("karekök: sonlu ve negatif olmayan sayı gerekir".to_string())
            }
            [other] => Deger::Hata(format!("karekök: sayı bekleniyordu; {} geldi", other)),
            _ => Deger::Hata(format!(
                "karekök: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );
    globals.insert(
        "rastgele".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if !args.is_empty() {
                return Deger::Hata(format!(
                    "rastgele: argüman beklenmiyordu; {} geldi",
                    args.len()
                ));
            }
            match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(duration) => {
                    let nanos = duration.as_nanos() as f64;
                    Deger::Sayi((nanos % 1_000_000.0) / 1_000_000.0)
                }
                Err(error) => Deger::Hata(format!("rastgele: sistem saati hatası: {}", error)),
            }
        }),
    );
    globals.insert(
        "dosya_oku".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let Some(Deger::Metin(yol)) = args.first() else {
                return Deger::Hata("dosya_oku: dosya yolu gerekir".to_string());
            };
            match std::fs::read_to_string(yol) {
                Ok(icerik) => Deger::Metin(icerik),
                Err(hata) => Deger::Hata(format!("dosya_oku: '{}': {}", yol, hata)),
            }
        }),
    );
    globals.insert(
        "dosya_yaz".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (Some(Deger::Metin(yol)), Some(Deger::Metin(icerik))) = (args.first(), args.get(1))
            else {
                return Deger::Hata("dosya_yaz: dosya yolu ve metin gerekir".to_string());
            };
            match std::fs::write(yol, icerik) {
                Ok(()) => Deger::Sayi(1.0),
                Err(hata) => Deger::Hata(format!("dosya_yaz: '{}': {}", yol, hata)),
            }
        }),
    );
    globals.insert(
        "ffi_yükle".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(name), Deger::Metin(path)] = args.as_slice() else {
                return Deger::Hata(
                    "ffi_yükle: kütüphane adı ve dosya yolu olmak üzere 2 metin gerekir"
                        .to_string(),
                );
            };
            let mut manager = match crate::ffi::FFI_YONETICI.lock() {
                Ok(manager) => manager,
                Err(_) => {
                    return Deger::Hata("ffi_yükle: FFI yöneticisi kilidi bozuldu".to_string())
                }
            };
            match manager.yukle(name, path) {
                Ok(()) => Deger::Sayi(1.0),
                Err(error) => Deger::Hata(format!("ffi_yükle: {}", error)),
            }
        }),
    );
    globals.insert(
        "ffi_çağır".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (Some(Deger::Metin(library)), Some(Deger::Metin(function))) =
                (args.first(), args.get(1))
            else {
                return Deger::Hata(
                    "ffi_çağır: kütüphane ve fonksiyon adı olmak üzere en az 2 metin gerekir"
                        .to_string(),
                );
            };
            let manager = match crate::ffi::FFI_YONETICI.lock() {
                Ok(manager) => manager,
                Err(_) => {
                    return Deger::Hata("ffi_çağır: FFI yöneticisi kilidi bozuldu".to_string())
                }
            };
            match manager.cagir_esnek(library, function, args[2..].to_vec()) {
                Ok(result) => result,
                Err(error) => Deger::Hata(format!("ffi_çağır: {}", error)),
            }
        }),
    );
    globals.insert(
        "tensor_olustur".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 3 {
                if let (Deger::Sayi(r), Deger::Sayi(c), Deger::Liste(l)) =
                    (&args[0], &args[1], &args[2])
                {
                    let req_grad = args
                        .get(3)
                        .map(|v| match v {
                            Deger::Sayi(n) => *n != 0.0,
                            _ => true,
                        })
                        .unwrap_or(true);
                    let veri: Vec<f64> = l
                        .borrow()
                        .iter()
                        .map(|d| match d {
                            Deger::Sayi(n) => *n,
                            _ => 0.0,
                        })
                        .collect();
                    if let Ok(mut g) = crate::autograd::AUTOGRAD_GRAF.lock() {
                        let t = g.tensor_olustur(*r as usize, *c as usize, veri, req_grad);
                        return Deger::Tensor(t);
                    }
                }
            }
            Deger::Bos
        }),
    );
    globals.insert(
        "tensor_topla".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Tensor(t1), Deger::Tensor(t2)) = (&args[0], &args[1]) {
                    if let Ok(mut g) = crate::autograd::AUTOGRAD_GRAF.lock() {
                        let res = g.topla(t1, t2);
                        return Deger::Tensor(res);
                    }
                }
            }
            Deger::Bos
        }),
    );
    globals.insert(
        "tensor_matris_carp".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Tensor(t1), Deger::Tensor(t2)) = (&args[0], &args[1]) {
                    if let Ok(mut g) = crate::autograd::AUTOGRAD_GRAF.lock() {
                        match g.matris_carp(t1, t2) {
                            Ok(res) => return Deger::Tensor(res),
                            Err(e) => return Deger::Hata(e),
                        }
                    }
                }
            }
            Deger::Bos
        }),
    );
    globals.insert(
        "tensor_relu".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Tensor(t1)) = args.first() {
                if let Ok(mut g) = crate::autograd::AUTOGRAD_GRAF.lock() {
                    let res = g.relu(t1);
                    return Deger::Tensor(res);
                }
            }
            Deger::Bos
        }),
    );
    globals.insert(
        "tensor_geri_yayilim".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Tensor(t1)) = args.first() {
                if let Ok(mut g) = crate::autograd::AUTOGRAD_GRAF.lock() {
                    if let Ok(()) = g.backward(t1.id) {
                        return Deger::Sayi(1.0);
                    }
                }
            }
            Deger::Sayi(0.0)
        }),
    );
    globals.insert(
        "tensor_gradyan".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Tensor(t1)) = args.first() {
                let grad_vec = t1
                    .gradyan
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|n| Deger::Sayi(*n))
                    .collect();
                return Deger::Liste(Rc::new(RefCell::new(grad_vec)));
            }
            Deger::Bos
        }),
    );
    globals.insert(
        "bpe_eğit".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (Some(Deger::Metin(metin)), Some(Deger::Sayi(vocab_boyutu))) =
                (args.first(), args.get(1))
            else {
                return Deger::Hata("bpe_eğit: metin ve sözlük boyutu gerekir".to_string());
            };
            if !vocab_boyutu.is_finite()
                || vocab_boyutu.fract() != 0.0
                || !(256.0..=65_536.0).contains(vocab_boyutu)
            {
                return Deger::Hata(
                    "bpe_eğit: sözlük boyutu 256 ile 65536 arasında bir tamsayı olmalı".to_string(),
                );
            }
            if let Ok(mut tok) = crate::tokenizer::BPE_TOKENIZER.lock() {
                tok.egit(metin, *vocab_boyutu as usize);
                return Deger::Sayi(1.0);
            }
            Deger::Hata("bpe_eğit: tokenizer kilidi alınamadı".to_string())
        }),
    );
    globals.insert(
        "bpe_kodla".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(metin)) = args.first() {
                if let Ok(tok) = crate::tokenizer::BPE_TOKENIZER.lock() {
                    let ids = tok.kodla(metin);
                    let list: Vec<Deger> =
                        ids.into_iter().map(|id| Deger::Sayi(id as f64)).collect();
                    return Deger::Liste(Rc::new(RefCell::new(list)));
                }
            }
            Deger::Hata("bpe_kodla: metin gerekir".to_string())
        }),
    );
    globals.insert(
        "bpe_çöz".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Liste(l)) = args.first() {
                let mut ids = Vec::with_capacity(l.borrow().len());
                for deger in l.borrow().iter() {
                    let Deger::Sayi(id) = deger else {
                        return Deger::Hata(
                            "bpe_çöz: token kimlikleri sayılardan oluşmalı".to_string(),
                        );
                    };
                    if !id.is_finite() || *id < 0.0 || id.fract() != 0.0 {
                        return Deger::Hata(
                            "bpe_çöz: token kimlikleri negatif olmayan tamsayılar olmalı"
                                .to_string(),
                        );
                    }
                    ids.push(*id as usize);
                }
                if let Ok(tok) = crate::tokenizer::BPE_TOKENIZER.lock() {
                    return match tok.coz(&ids) {
                        Ok(text) => Deger::Metin(text),
                        Err(hata) => Deger::Hata(format!("bpe_çöz: {}", hata)),
                    };
                }
            }
            Deger::Hata("bpe_çöz: token kimliği listesi gerekir".to_string())
        }),
    );
    globals.insert(
        "sistem".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(command)] = args.as_slice() else {
                return Deger::Hata("sistem: tam olarak 1 metin komutu gerekir".to_string());
            };

            let mut process = if cfg!(target_os = "windows") {
                let mut process = std::process::Command::new("cmd");
                process.args(["/C", command]);
                process
            } else {
                let mut process = std::process::Command::new("sh");
                process.args(["-c", command]);
                process
            };

            match process.output() {
                Ok(output) if output.status.success() => {
                    Deger::Metin(String::from_utf8_lossy(&output.stdout).trim().to_string())
                }
                Ok(output) => {
                    let code = output
                        .status
                        .code()
                        .map_or_else(|| "sinyal".to_string(), |value| value.to_string());
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    if stderr.is_empty() {
                        Deger::Hata(format!("sistem: komut başarısız oldu (çıkış: {})", code))
                    } else {
                        Deger::Hata(format!(
                            "sistem: komut başarısız oldu (çıkış: {}): {}",
                            code, stderr
                        ))
                    }
                }
                Err(error) => Deger::Hata(format!("sistem: komut başlatılamadı: {}", error)),
            }
        }),
    );
    // dahili_sunucu_baslat(port)
    globals.insert(
        "dahili_sunucu_baslat".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let port = match args.first() {
                Some(Deger::Sayi(n)) => *n as u16,
                _ => 8080,
            };
            let sid = get_id();

            let (tx, rx) = mpsc::unbounded_channel::<IncomingRequest>();
            SUNUCULAR.lock().unwrap().insert(sid, tx);
            SUNUCU_RX.blocking_lock().insert(sid, rx);

            // Spawn the hyper server on the Yaprak runtime.
            let addr = ([0, 0, 0, 0], port).into();
            let make_svc = make_service_fn(move |_conn| {
                let sid2 = sid;
                async move {
                    Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                        let sid3 = sid2;
                        async move {
                            let url = req.uri().to_string();
                            let metot = req.method().to_string();
                            let bytes = hyper::body::to_bytes(req.into_body())
                                .await
                                .unwrap_or_default();
                            let govde = String::from_utf8_lossy(&bytes).to_string();

                            let (resp_tx, resp_rx) = oneshot::channel::<Response<Body>>();
                            let rid = get_id();
                            let incoming = IncomingRequest {
                                id: rid,
                                url,
                                metot,
                                govde,
                                respond_to: resp_tx,
                            };

                            if let Some(sender) = SUNUCULAR.lock().unwrap().get(&sid3) {
                                let _ = sender.send(incoming);
                            }

                            match resp_rx.await {
                                Ok(resp) => Ok::<_, hyper::Error>(resp),
                                Err(_) => Ok::<_, hyper::Error>(
                                    Response::builder()
                                        .status(500)
                                        .body(Body::from("handler dropped"))
                                        .unwrap(),
                                ),
                            }
                        }
                    }))
                }
            });

            YAPRAK.with(|y| {
                let y = y.borrow_mut();
                let fut = hyper::Server::bind(&addr).serve(make_svc).map(|_| ());
                std::mem::drop(y.local.spawn_local(fut));
            });

            Deger::Sayi(sid as f64)
        }),
    );
    // dahili_sunucu_bekle(sid) -> görev (bekle ile alınır)
    globals.insert(
        "dahili_sunucu_bekle".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let sid = match args.first() {
                Some(Deger::Sayi(n)) => *n as u64,
                _ => return Deger::Bos,
            };
            YAPRAK.with(|y| {
                y.borrow_mut().spawn(async move {
                    let mut guard = SUNUCU_RX.lock().await;
                    let rx = match guard.get_mut(&sid) {
                        Some(r) => r,
                        None => return Deger::Bos,
                    };
                    match rx.recv().await {
                        Some(incoming) => {
                            YANITLAR
                                .lock()
                                .unwrap()
                                .insert(incoming.id, incoming.respond_to);
                            let mut fields = HashMap::new();
                            fields.insert("id".to_string(), Deger::Sayi(incoming.id as f64));
                            fields.insert("url".to_string(), Deger::Metin(incoming.url));
                            fields.insert("metot".to_string(), Deger::Metin(incoming.metot));
                            fields.insert("gövde".to_string(), Deger::Metin(incoming.govde));
                            Deger::Nesne {
                                sinif_adi: "İstek".to_string(),
                                alanlar: Rc::new(RefCell::new(fields)),
                            }
                        }
                        None => Deger::Bos,
                    }
                })
            })
        }),
    );
    // dahili_sunucu_yanitla(i_id, icerik, durum, tip, [basliklar])
    globals.insert(
        "dahili_sunucu_yanitla".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Sayi(0.0);
            }
            let i_id = match &args[0] {
                Deger::Sayi(n) => *n as u64,
                _ => return Deger::Sayi(0.0),
            };

            let (data, _len) = match &args[1] {
                Deger::Metin(s) => (s.as_bytes().to_vec(), s.len()),
                Deger::Bayt(b) => (b.clone(), b.len()),
                _ => (Vec::new(), 0),
            };

            let durum = match args.get(2) {
                Some(Deger::Sayi(n)) => *n as u16,
                _ => 200,
            };
            let tip = match args.get(3) {
                Some(Deger::Metin(s)) => s.as_str(),
                _ => "text/html; charset=utf-8",
            };

            if let Some(tx) = YANITLAR.lock().unwrap().remove(&i_id) {
                let mut builder = Response::builder()
                    .status(durum)
                    .header("content-type", tip);

                if args.len() >= 5 {
                    if let Deger::Nesne { alanlar, .. } = &args[4] {
                        for (k, v) in alanlar.borrow().iter() {
                            builder = builder.header(k.as_str(), v.to_string());
                        }
                    }
                }

                let body = Body::from(data);
                let resp = builder
                    .body(body)
                    .unwrap_or_else(|_| Response::new(Body::from("response build error")));
                let _ = tx.send(resp);
                return Deger::Sayi(1.0);
            }

            Deger::Sayi(0.0)
        }),
    );

    globals.insert(
        "dosya_oku_bayt".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let Some(Deger::Metin(yol)) = args.first() else {
                return Deger::Hata("dosya_oku_bayt: dosya yolu gerekir".to_string());
            };
            match std::fs::read(yol) {
                Ok(baytlar) => Deger::Bayt(baytlar),
                Err(hata) => Deger::Hata(format!("dosya_oku_bayt: '{}': {}", yol, hata)),
            }
        }),
    );
    // dahili_istek(metot, url, [gövde])

    globals.insert(
        "dahili_istek".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            let metot = match &args[0] {
                Deger::Metin(s) => s.to_uppercase(),
                _ => "GET".to_string(),
            };
            let url = match &args[1] {
                Deger::Metin(s) => s.clone(),
                _ => return Deger::Bos,
            };
            let govdeli = args.len() >= 3 && !matches!(args[2], Deger::Bos);
            let govde = if govdeli {
                match &args[2] {
                    Deger::Metin(s) => s.clone(),
                    _ => String::new(),
                }
            } else {
                String::new()
            };
            let headers = if args.len() >= 4 {
                args[3].clone()
            } else {
                Deger::Bos
            };

            YAPRAK.with(|y| {
                y.borrow_mut().spawn(async move {
                    let client = reqwest::Client::new();
                    let method = metot.parse().unwrap_or(reqwest::Method::GET);
                    let mut req = client.request(method, url);

                    if let Deger::Nesne { alanlar, .. } = headers {
                        for (k, v) in alanlar.borrow().iter() {
                            if let Ok(hn) = reqwest::header::HeaderName::from_bytes(k.as_bytes()) {
                                if let Ok(hv) =
                                    reqwest::header::HeaderValue::from_str(&v.to_string())
                                {
                                    req = req.header(hn, hv);
                                }
                            }
                        }
                    }

                    if govdeli {
                        req = req.body(govde);
                    }

                    match req.send().await {
                        Ok(res) => {
                            let durum = res.status().as_u16() as f64;
                            let icerik = res.text().await.unwrap_or_default();
                            let alanlar = HashMap::from([
                                ("durum".to_string(), Deger::Sayi(durum)),
                                ("içerik".to_string(), Deger::Metin(icerik)),
                            ]);
                            Deger::Nesne {
                                sinif_adi: "İstekCevabı".to_string(),
                                alanlar: Rc::new(RefCell::new(alanlar)),
                            }
                        }
                        Err(e) => {
                            let mut alanlar = HashMap::new();
                            alanlar.insert("durum".to_string(), Deger::Sayi(0.0));
                            alanlar.insert("hata".to_string(), Deger::Metin(e.to_string()));
                            Deger::Nesne {
                                sinif_adi: "İstekHatası".to_string(),
                                alanlar: Rc::new(RefCell::new(alanlar)),
                            }
                        }
                    }
                })
            })
        }),
    );

    globals.insert(
        "dosya_var_mı".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(yol)) = args.first() {
                return Deger::Sayi(if Path::new(yol).exists() { 1.0 } else { 0.0 });
            }
            Deger::Sayi(0.0)
        }),
    );
    // JSON Fonksiyonları
    globals.insert(
        "nesneden_metine".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(d) = args.first() {
                if let Ok(s) = serde_json::to_string_pretty(&d.to_json()) {
                    return Deger::Metin(s);
                }
            }
            Deger::Metin("null".to_string())
        }),
    );
    globals.insert(
        "metinden_nesneye".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(s)) = args.first() {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) {
                    return Deger::from_json(&v);
                }
            }
            Deger::Bos
        }),
    );
    globals.insert(
        "tipi".to_string(),
        Deger::DahiliFonksiyon(|args| match args.first() {
            Some(Deger::Sayi(_)) => Deger::Metin("Sayı".to_string()),
            Some(Deger::Metin(_)) => Deger::Metin("Metin".to_string()),
            Some(Deger::Liste(_)) => Deger::Metin("Liste".to_string()),
            Some(Deger::Fonksiyon { .. }) => Deger::Metin("Fonksiyon".to_string()),
            Some(Deger::Sinif { .. }) => Deger::Metin("Sınıf".to_string()),
            Some(Deger::Nesne { .. }) => Deger::Metin("Nesne".to_string()),
            _ => Deger::Metin("Boş".to_string()),
        }),
    );

    globals.insert(
        "ortam_değişkeni".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(anahtar)) = args.first() {
                if let Ok(val) = std::env::var(anahtar) {
                    return Deger::Metin(val);
                }
            }
            Deger::Bos
        }),
    );

    // ── NLP / Metin İşleme Built-in Fonksiyonları ──────────────────────────

    // küçük_harf(metin) → Türkçe-farkında küçük harf dönüşümü
    globals.insert(
        "küçük_harf".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(s)) = args.first() {
                let sonuc: String = s
                    .chars()
                    .map(|c| match c {
                        'I' => 'ı',
                        'İ' => 'i',
                        'Ğ' => 'ğ',
                        'Ş' => 'ş',
                        'Ç' => 'ç',
                        'Ö' => 'ö',
                        'Ü' => 'ü',
                        _ => c.to_lowercase().next().unwrap_or(c),
                    })
                    .collect();
                Deger::Metin(sonuc)
            } else {
                Deger::Bos
            }
        }),
    );

    // büyük_harf(metin) → Türkçe-farkında büyük harf dönüşümü
    globals.insert(
        "büyük_harf".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(s)) = args.first() {
                let sonuc: String = s
                    .chars()
                    .map(|c| match c {
                        'ı' => 'I',
                        'i' => 'İ',
                        'ğ' => 'Ğ',
                        'ş' => 'Ş',
                        'ç' => 'Ç',
                        'ö' => 'Ö',
                        'ü' => 'Ü',
                        _ => c.to_uppercase().next().unwrap_or(c),
                    })
                    .collect();
                Deger::Metin(sonuc)
            } else {
                Deger::Bos
            }
        }),
    );

    // böl(metin, ayraç) → Liste döndürür
    globals.insert(
        "böl".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Metin(s), Deger::Metin(ayrac)) = (&args[0], &args[1]) {
                    let parcalar: Vec<Deger> = if ayrac.is_empty() {
                        s.chars().map(|c| Deger::Metin(c.to_string())).collect()
                    } else {
                        s.split(ayrac.as_str())
                            .map(|p| Deger::Metin(p.to_string()))
                            .collect()
                    };
                    return Deger::Liste(Rc::new(RefCell::new(parcalar)));
                }
            }
            Deger::Bos
        }),
    );

    // birleştir(liste, ayraç) → birleştirilmiş metin
    globals.insert(
        "birleştir".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Liste(l), Deger::Metin(ayrac)) = (&args[0], &args[1]) {
                    let parcalar: Vec<String> = l.borrow().iter().map(|d| d.to_string()).collect();
                    return Deger::Metin(parcalar.join(ayrac.as_str()));
                }
            } else if let Some(Deger::Liste(l)) = args.first() {
                let parcalar: Vec<String> = l.borrow().iter().map(|d| d.to_string()).collect();
                return Deger::Metin(parcalar.join(""));
            }
            Deger::Bos
        }),
    );

    // değiştir(metin, aranan, yeni) → yeni metin
    globals.insert(
        "değiştir".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 3 {
                if let (Deger::Metin(s), Deger::Metin(aranan), Deger::Metin(yeni)) =
                    (&args[0], &args[1], &args[2])
                {
                    return Deger::Metin(s.replace(aranan.as_str(), yeni.as_str()));
                }
            }
            Deger::Bos
        }),
    );

    // kırp(metin) → baştaki ve sondaki boşlukları sil
    globals.insert(
        "kırp".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(s)) = args.first() {
                Deger::Metin(s.trim().to_string())
            } else {
                Deger::Bos
            }
        }),
    );

    // tekrar_sayısı(metin, aranan) → kaç kez geçiyor
    globals.insert(
        "tekrar_sayısı".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Metin(s), Deger::Metin(aranan)) = (&args[0], &args[1]) {
                    if aranan.is_empty() {
                        return Deger::Sayi(0.0);
                    }
                    return Deger::Sayi(s.matches(aranan.as_str()).count() as f64);
                }
            }
            Deger::Bos
        }),
    );

    // sayıya_çevir(metin) → Sayı değerine dönüştür
    globals.insert(
        "sayıya_çevir".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(s)) = args.first() {
                if let Ok(n) = s.trim().parse::<f64>() {
                    return Deger::Sayi(n);
                }
            } else if let Some(Deger::Sayi(n)) = args.first() {
                return Deger::Sayi(*n);
            }
            Deger::Bos
        }),
    );

    // metne_çevir(değer) → Metin değerine dönüştür
    globals.insert(
        "metne_çevir".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(d) = args.first() {
                Deger::Metin(d.to_string())
            } else {
                Deger::Bos
            }
        }),
    );

    // ascii_kodu(karakter) → Unicode kod noktası
    globals.insert(
        "ascii_kodu".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(s)) = args.first() {
                if let Some(c) = s.chars().next() {
                    return Deger::Sayi(c as u32 as f64);
                }
            }
            Deger::Bos
        }),
    );

    // karakterden(kod) → Unicode karakterini metin olarak döndür
    globals.insert(
        "karakterden".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(n)) = args.first() {
                if let Some(c) = char::from_u32(*n as u32) {
                    return Deger::Metin(c.to_string());
                }
            }
            Deger::Bos
        }),
    );

    // içeriyor(metin_veya_liste_veya_nesne, aranan) → 1 veya 0
    globals.insert(
        "içeriyor".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                match (&args[0], &args[1]) {
                    (Deger::Metin(s), Deger::Metin(sub)) => {
                        return Deger::Sayi(if s.contains(sub.as_str()) { 1.0 } else { 0.0 });
                    }
                    (Deger::Liste(l), target) => {
                        let has = l.borrow().iter().any(|item| item == target);
                        return Deger::Sayi(if has { 1.0 } else { 0.0 });
                    }
                    _ => {}
                }
            }
            Deger::Sayi(0.0)
        }),
    );
    globals.insert(
        "dahili_sunucu_baslat".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let port = match args.first() {
                Some(Deger::Sayi(n)) => *n as u16,
                _ => 8080,
            };
            let sid = get_id();

            let (tx, rx) = mpsc::unbounded_channel::<IncomingRequest>();
            SUNUCULAR.lock().unwrap().insert(sid, tx);
            SUNUCU_RX.blocking_lock().insert(sid, rx);

            let addr = ([0, 0, 0, 0], port).into();
            let make_svc = make_service_fn(move |_conn| {
                let sid2 = sid;
                async move {
                    Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                        let sid3 = sid2;
                        async move {
                            let url = req.uri().to_string();
                            let metot = req.method().to_string();
                            let bytes = hyper::body::to_bytes(req.into_body())
                                .await
                                .unwrap_or_default();
                            let govde = String::from_utf8_lossy(&bytes).to_string();

                            let (resp_tx, resp_rx) = oneshot::channel::<Response<Body>>();
                            let rid = get_id();
                            let incoming = IncomingRequest {
                                id: rid,
                                url,
                                metot,
                                govde,
                                respond_to: resp_tx,
                            };
                            if let Some(tx) = SUNUCULAR.lock().unwrap().get(&sid3) {
                                let _ = tx.send(incoming);
                            }

                            if let Ok(resp) = resp_rx.await {
                                Ok::<_, hyper::Error>(resp)
                            } else {
                                Ok::<_, hyper::Error>(
                                    Response::builder()
                                        .status(500)
                                        .body(Body::from("İç Sunucu Hatası"))
                                        .unwrap(),
                                )
                            }
                        }
                    }))
                }
            });

            tokio::spawn(async move {
                let server = Server::bind(&addr).serve(make_svc);
                let _ = server.await;
            });

            Deger::Sayi(sid as f64)
        }),
    );

    globals.insert(
        "dahili_sunucu_bekle".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(sid)) = args.first() {
                let sid = *sid as u64;
                let mut guard = SUNUCU_RX.blocking_lock();
                if let Some(rx) = guard.get_mut(&sid) {
                    if let Some(req) = rx.blocking_recv() {
                        let mut fields = HashMap::new();
                        fields.insert("id".to_string(), Deger::Sayi(req.id as f64));
                        fields.insert("url".to_string(), Deger::Metin(req.url));
                        fields.insert("metot".to_string(), Deger::Metin(req.metot));
                        fields.insert("gövde".to_string(), Deger::Metin(req.govde));

                        let rid = req.id;
                        YANITLAR.lock().unwrap().insert(rid, req.respond_to);

                        return Deger::Nesne {
                            sinif_adi: "İstek".to_string(),
                            alanlar: Rc::new(RefCell::new(fields)),
                        };
                    }
                }
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "dahili_sunucu_yanıtla".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 3 {
                if let (Deger::Sayi(rid), Deger::Sayi(durum), Deger::Metin(icerik)) =
                    (&args[0], &args[1], &args[2])
                {
                    let rid = *rid as u64;
                    let responder = YANITLAR.lock().unwrap().remove(&rid);
                    if let Some(tx) = responder {
                        let resp = Response::builder()
                            .status(*durum as u16)
                            .header("Content-Type", "text/html; charset=utf-8")
                            .body(Body::from(icerik.clone()))
                            .unwrap();
                        let _ = tx.send(resp);
                        return Deger::Sayi(1.0);
                    }
                }
            }
            Deger::Sayi(0.0)
        }),
    );

    globals.insert(
        "tekrar_sayısı".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Metin(metin), Deger::Metin(aranan)) = (&args[0], &args[1]) {
                    return Deger::Sayi(metin.matches(aranan).count() as f64);
                }
            }
            Deger::Sayi(0.0)
        }),
    );

    globals.insert(
        "ascii_kodu".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(s)) = args.first() {
                if let Some(c) = s.chars().next() {
                    return Deger::Sayi(c as u32 as f64);
                }
            }
            Deger::Sayi(0.0)
        }),
    );

    globals.insert(
        "karakterden".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(n)) = args.first() {
                if let Some(c) = std::char::from_u32(*n as u32) {
                    return Deger::Metin(c.to_string());
                }
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "değer_al".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let Deger::Nesne { alanlar, .. } = &args[0] {
                    if let Deger::Metin(key) = &args[1] {
                        if let Some(val) = alanlar.borrow().get(key) {
                            return val.clone();
                        }
                    }
                }
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "değer_ata".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 3 {
                if let Deger::Nesne { alanlar, .. } = &args[0] {
                    if let Deger::Metin(key) = &args[1] {
                        alanlar.borrow_mut().insert(key.clone(), args[2].clone());
                        return Deger::Sayi(1.0);
                    }
                }
            }
            Deger::Sayi(0.0)
        }),
    );

    globals.insert(
        "hızlı_içeriyor".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let Deger::Liste(l) = &args[0] {
                    let target = &args[1];
                    let contains = l.borrow().iter().any(|x| x == target);
                    return Deger::Sayi(if contains { 1.0 } else { 0.0 });
                }
            }
            Deger::Sayi(0.0)
        }),
    );

    globals.insert(
        "tipi".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(v) = args.first() {
                match v {
                    Deger::Sayi(_) => Deger::Metin("sayı".to_string()),
                    Deger::Metin(_) => Deger::Metin("metin".to_string()),
                    Deger::Liste(_) => Deger::Metin("liste".to_string()),
                    Deger::Sozluk(_) => Deger::Metin("sözlük".to_string()),
                    Deger::Fonksiyon { .. } | Deger::DahiliFonksiyon(_) => {
                        Deger::Metin("fonksiyon".to_string())
                    }
                    Deger::Nesne { sinif_adi, .. } => Deger::Metin(sinif_adi.clone()),
                    Deger::Sinif { ad, .. } => Deger::Metin(format!("sınıf_{}", ad)),
                    Deger::Bayt(_) => Deger::Metin("bayt".to_string()),
                    Deger::GorevId(_) => Deger::Metin("görev".to_string()),
                    Deger::Bos => Deger::Metin("boş".to_string()),
                    Deger::Hata(_) => Deger::Metin("hata".to_string()),
                    Deger::Vektor(v) => Deger::Metin(format!("vektör[{}]", v.borrow().len())),
                    Deger::Matris {
                        satirlar, sutunlar, ..
                    } => Deger::Metin(format!("matris[{}×{}]", satirlar, sutunlar)),
                    Deger::Tensor(t) => {
                        Deger::Metin(format!("tensor[{}×{}]", t.satirlar, t.sutunlar))
                    }
                }
            } else {
                Deger::Metin("bilinmeyen".to_string())
            }
        }),
    );

    globals.insert(
        "içeriyor".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                match (&args[0], &args[1]) {
                    (Deger::Metin(s), Deger::Metin(sub)) => {
                        return Deger::Sayi(if s.contains(sub.as_str()) { 1.0 } else { 0.0 });
                    }
                    (Deger::Liste(l), target) => {
                        let has = l.borrow().iter().any(|item| item == target);
                        return Deger::Sayi(if has { 1.0 } else { 0.0 });
                    }
                    _ => {}
                }
            }
            Deger::Sayi(0.0)
        }),
    );

    globals.insert(
        "başlıyor_mu".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Metin(s), Deger::Metin(onek)) = (&args[0], &args[1]) {
                    return Deger::Sayi(if s.starts_with(onek.as_str()) {
                        1.0
                    } else {
                        0.0
                    });
                }
            }
            Deger::Sayi(0.0)
        }),
    );

    globals.insert(
        "bitiyor_mu".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Metin(s), Deger::Metin(sonek)) = (&args[0], &args[1]) {
                    return Deger::Sayi(if s.ends_with(sonek.as_str()) {
                        1.0
                    } else {
                        0.0
                    });
                }
            }
            Deger::Sayi(0.0)
        }),
    );

    globals.insert(
        "dizi_dilim".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 3 {
                if let (Deger::Metin(s), Deger::Sayi(bas), Deger::Sayi(son)) =
                    (&args[0], &args[1], &args[2])
                {
                    let chars: Vec<char> = s.chars().collect();
                    let b = *bas as usize;
                    let e = (*son as usize).min(chars.len());
                    if b <= e {
                        return Deger::Metin(chars[b..e].iter().collect());
                    }
                }
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "dahili_sql_bağlan".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let Some(Deger::Metin(yol)) = args.first() else {
                return Deger::Hata("dahili_sql_bağlan: dosya yolu gerekir".to_string());
            };
            let conn = match rusqlite::Connection::open(yol) {
                Ok(conn) => conn,
                Err(hata) => {
                    return Deger::Hata(format!(
                        "dahili_sql_bağlan: veritabanı açılamadı: {}",
                        hata
                    ));
                }
            };
            let Ok(mut baglantilar) = SQL_CONNECTIONS.lock() else {
                return Deger::Hata(
                    "dahili_sql_bağlan: bağlantı tablosu kilitlenemedi".to_string(),
                );
            };
            let id = get_id();
            baglantilar.insert(id, conn);
            Deger::Sayi(id as f64)
        }),
    );

    globals.insert(
        "dahili_sql_yürüt".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (Some(Deger::Sayi(id)), Some(Deger::Metin(sql))) = (args.first(), args.get(1))
            else {
                return Deger::Hata(
                    "dahili_sql_yürüt: bağlantı kimliği ve SQL metni gerekir".to_string(),
                );
            };
            if !id.is_finite() || *id < 0.0 || id.fract() != 0.0 {
                return Deger::Hata(
                    "dahili_sql_yürüt: bağlantı kimliği negatif olmayan tamsayı olmalı".to_string(),
                );
            }
            let Ok(baglantilar) = SQL_CONNECTIONS.lock() else {
                return Deger::Hata("dahili_sql_yürüt: bağlantı tablosu kilitlenemedi".to_string());
            };
            let Some(conn) = baglantilar.get(&(*id as u64)) else {
                return Deger::Hata(format!(
                    "dahili_sql_yürüt: {} kimlikli bağlantı bulunamadı",
                    id
                ));
            };
            match conn.execute(sql, []) {
                Ok(etkilenen) => Deger::Sayi(etkilenen as f64),
                Err(hata) => Deger::Hata(format!("dahili_sql_yürüt: {}", hata)),
            }
        }),
    );

    globals.insert(
        "dahili_sql_sorgula".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (Some(Deger::Sayi(id)), Some(Deger::Metin(sql))) = (args.first(), args.get(1))
            else {
                return Deger::Hata(
                    "dahili_sql_sorgula: bağlantı kimliği ve SQL metni gerekir".to_string(),
                );
            };
            if !id.is_finite() || *id < 0.0 || id.fract() != 0.0 {
                return Deger::Hata(
                    "dahili_sql_sorgula: bağlantı kimliği negatif olmayan tamsayı olmalı"
                        .to_string(),
                );
            }
            let Ok(baglantilar) = SQL_CONNECTIONS.lock() else {
                return Deger::Hata(
                    "dahili_sql_sorgula: bağlantı tablosu kilitlenemedi".to_string(),
                );
            };
            let Some(conn) = baglantilar.get(&(*id as u64)) else {
                return Deger::Hata(format!(
                    "dahili_sql_sorgula: {} kimlikli bağlantı bulunamadı",
                    id
                ));
            };
            let mut stmt = match conn.prepare(sql) {
                Ok(stmt) => stmt,
                Err(hata) => return Deger::Hata(format!("dahili_sql_sorgula: {}", hata)),
            };
            let sutun_adlari = stmt
                .column_names()
                .iter()
                .map(|ad| ad.to_lowercase().trim().to_string())
                .collect::<Vec<_>>();
            let sorgu = stmt.query_map([], |satir| {
                let mut alanlar = HashMap::new();
                for (i, sutun_adi) in sutun_adlari.iter().enumerate() {
                    let deger: rusqlite::types::Value = satir.get(i)?;
                    let huma_degeri = match deger {
                        rusqlite::types::Value::Null => Deger::Bos,
                        rusqlite::types::Value::Integer(sayi) => Deger::Sayi(sayi as f64),
                        rusqlite::types::Value::Real(sayi) => Deger::Sayi(sayi),
                        rusqlite::types::Value::Text(metin) => Deger::Metin(metin),
                        rusqlite::types::Value::Blob(baytlar) => Deger::Bayt(baytlar),
                    };
                    alanlar.insert(sutun_adi.clone(), huma_degeri);
                }
                Ok(Deger::Nesne {
                    sinif_adi: "Satır".to_string(),
                    alanlar: Rc::new(RefCell::new(alanlar)),
                })
            });
            let satirlar = match sorgu {
                Ok(satirlar) => satirlar,
                Err(hata) => return Deger::Hata(format!("dahili_sql_sorgula: {}", hata)),
            };
            let mut sonuc = Vec::new();
            for satir in satirlar {
                match satir {
                    Ok(satir) => sonuc.push(satir),
                    Err(hata) => {
                        return Deger::Hata(format!(
                            "dahili_sql_sorgula: satır okunamadı: {}",
                            hata
                        ));
                    }
                }
            }
            Deger::Liste(Rc::new(RefCell::new(sonuc)))
        }),
    );

    let cli_args: Vec<Deger> = std::env::args().map(Deger::Metin).collect();
    globals.insert(
        "argümanlar".to_string(),
        Deger::Liste(Rc::new(RefCell::new(cli_args))),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // BLOK A — Genişletilmiş Matematik Built-in'leri
    // ═══════════════════════════════════════════════════════════════════════

    globals.insert(
        "üs".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Sayi(taban), Deger::Sayi(kuvvet)) = (&args[0], &args[1]) {
                    return Deger::Sayi(taban.powf(*kuvvet));
                }
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "ln".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(x)) = args.first() {
                return Deger::Sayi(x.ln());
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "log2".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(x)) = args.first() {
                return Deger::Sayi(x.log2());
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "log10".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(x)) = args.first() {
                return Deger::Sayi(x.log10());
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "sin".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(x)) = args.first() {
                Deger::Sayi(x.sin())
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "cos".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(x)) = args.first() {
                Deger::Sayi(x.cos())
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "tan".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(x)) = args.first() {
                Deger::Sayi(x.tan())
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "exp".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(x)) = args.first() {
                Deger::Sayi(x.exp())
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "tavan".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(x)) = args.first() {
                Deger::Sayi(x.ceil())
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "taban_sayı".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(x)) = args.first() {
                Deger::Sayi(x.floor())
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "mutlak_sayı".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(x)) = args.first() {
                Deger::Sayi(x.abs())
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "işaret".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(x)) = args.first() {
                Deger::Sayi(x.signum())
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "sonlu_mu".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(x)) = args.first() {
                Deger::Sayi(if x.is_finite() { 1.0 } else { 0.0 })
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "klamp".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 3 {
                if let (Deger::Sayi(x), Deger::Sayi(min), Deger::Sayi(max)) =
                    (&args[0], &args[1], &args[2])
                {
                    return Deger::Sayi(x.clamp(*min, *max));
                }
            }
            Deger::Bos
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // BLOK B — Aktivasyon Fonksiyonları & ML Primitifleri
    // ═══════════════════════════════════════════════════════════════════════

    globals.insert(
        "sigmoid".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(x)) = args.first() {
                Deger::Sayi(1.0 / (1.0 + (-x).exp()))
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "relu".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(x)) = args.first() {
                Deger::Sayi(x.max(0.0))
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "tanh_aktivasyon".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(x)) = args.first() {
                Deger::Sayi(x.tanh())
            } else {
                Deger::Bos
            }
        }),
    );

    // GELU — Gaussian Error Linear Unit (tanh approximation)
    globals.insert(
        "gelu".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(x)) = args.first() {
                let x = *x;
                let val =
                    0.5 * x * (1.0 + (0.7978845608028654 * (x + 0.044715 * x * x * x)).tanh());
                Deger::Sayi(val)
            } else {
                Deger::Bos
            }
        }),
    );

    // softmax(vektor) — her iki vektör tipi de kabul edilir
    globals.insert(
        "softmax".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let vals: Option<Vec<f64>> = match args.first() {
                Some(Deger::Vektor(v)) => Some(v.borrow().clone()),
                Some(Deger::Liste(l)) => Some(
                    l.borrow()
                        .iter()
                        .filter_map(|d| {
                            if let Deger::Sayi(n) = d {
                                Some(*n)
                            } else {
                                None
                            }
                        })
                        .collect(),
                ),
                _ => None,
            };
            if let Some(v) = vals {
                let max_val = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let exps: Vec<f64> = v.iter().map(|x| (x - max_val).exp()).collect();
                let toplam: f64 = exps.iter().sum();
                if toplam == 0.0 {
                    return Deger::Bos;
                }
                let sonuc: Vec<f64> = exps.iter().map(|e| e / toplam).collect();
                Deger::Vektor(Rc::new(RefCell::new(sonuc)))
            } else {
                Deger::Bos
            }
        }),
    );

    // log_softmax — numerically stable log-softmax
    globals.insert(
        "log_softmax".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let vals: Option<Vec<f64>> = match args.first() {
                Some(Deger::Vektor(v)) => Some(v.borrow().clone()),
                Some(Deger::Liste(l)) => Some(
                    l.borrow()
                        .iter()
                        .filter_map(|d| {
                            if let Deger::Sayi(n) = d {
                                Some(*n)
                            } else {
                                None
                            }
                        })
                        .collect(),
                ),
                _ => None,
            };
            if let Some(v) = vals {
                let max_val = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let log_sum_exp = v.iter().map(|x| (x - max_val).exp()).sum::<f64>().ln() + max_val;
                let sonuc: Vec<f64> = v.iter().map(|x| x - log_sum_exp).collect();
                Deger::Vektor(Rc::new(RefCell::new(sonuc)))
            } else {
                Deger::Bos
            }
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // BLOK C — Vektör Operasyonları
    // ═══════════════════════════════════════════════════════════════════════

    globals.insert(
        "vektor_olustur".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (Some(Deger::Sayi(n)), Some(Deger::Sayi(deger))) = (args.first(), args.get(1))
            else {
                return Deger::Hata(
                    "vektor_olustur: boyut ve başlangıç değeri gerekir".to_string(),
                );
            };
            let boyut = match boyut_dogrula(*n, "vektor_olustur", true) {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            if !deger.is_finite() {
                return Deger::Hata("vektor_olustur: başlangıç değeri sonlu olmalı".to_string());
            }
            Deger::Vektor(Rc::new(RefCell::new(vec![*deger; boyut])))
        }),
    );

    globals.insert(
        "vektor_uzunluk".to_string(),
        Deger::DahiliFonksiyon(|args| match args.first() {
            Some(Deger::Vektor(v)) => Deger::Sayi(v.borrow().len() as f64),
            Some(Deger::Liste(l)) => Deger::Sayi(l.borrow().len() as f64),
            _ => Deger::Bos,
        }),
    );

    globals.insert(
        "ic_carpim".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            let a: Option<Vec<f64>> = match &args[0] {
                Deger::Vektor(v) => Some(v.borrow().clone()),
                Deger::Liste(l) => Some(
                    l.borrow()
                        .iter()
                        .filter_map(|d| {
                            if let Deger::Sayi(n) = d {
                                Some(*n)
                            } else {
                                None
                            }
                        })
                        .collect(),
                ),
                _ => None,
            };
            let b: Option<Vec<f64>> = match &args[1] {
                Deger::Vektor(v) => Some(v.borrow().clone()),
                Deger::Liste(l) => Some(
                    l.borrow()
                        .iter()
                        .filter_map(|d| {
                            if let Deger::Sayi(n) = d {
                                Some(*n)
                            } else {
                                None
                            }
                        })
                        .collect(),
                ),
                _ => None,
            };
            if let (Some(va), Some(vb)) = (a, b) {
                if va.len() != vb.len() {
                    return Deger::Hata("ic_carpim: vektör boyutları eşit olmalı".to_string());
                }
                let sonuc: f64 = va.iter().zip(vb.iter()).map(|(x, y)| x * y).sum();
                Deger::Sayi(sonuc)
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "vektor_norm".to_string(),
        Deger::DahiliFonksiyon(|args| match args.first() {
            Some(Deger::Vektor(v)) => {
                let n: f64 = v.borrow().iter().map(|x| x * x).sum::<f64>().sqrt();
                Deger::Sayi(n)
            }
            Some(Deger::Liste(l)) => {
                let n: f64 = l
                    .borrow()
                    .iter()
                    .filter_map(|d| {
                        if let Deger::Sayi(x) = d {
                            Some(x * x)
                        } else {
                            None
                        }
                    })
                    .sum::<f64>()
                    .sqrt();
                Deger::Sayi(n)
            }
            _ => Deger::Bos,
        }),
    );

    globals.insert(
        "vektor_birim".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Vektor(v)) = args.first() {
                let b = v.borrow();
                let norm: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
                if norm == 0.0 {
                    return Deger::Hata(
                        "vektor_birim: sıfır vektörü normalize edilemez".to_string(),
                    );
                }
                let sonuc: Vec<f64> = b.iter().map(|x| x / norm).collect();
                Deger::Vektor(Rc::new(RefCell::new(sonuc)))
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "kosinus_benzerligi".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            let get_vec = |d: &Deger| -> Option<Vec<f64>> {
                match d {
                    Deger::Vektor(v) => Some(v.borrow().clone()),
                    Deger::Liste(l) => Some(
                        l.borrow()
                            .iter()
                            .filter_map(|x| {
                                if let Deger::Sayi(n) = x {
                                    Some(*n)
                                } else {
                                    None
                                }
                            })
                            .collect(),
                    ),
                    _ => None,
                }
            };
            if let (Some(va), Some(vb)) = (get_vec(&args[0]), get_vec(&args[1])) {
                if va.len() != vb.len() {
                    return Deger::Hata("kosinus_benzerligi: boyutlar eşit olmalı".to_string());
                }
                let dot: f64 = va.iter().zip(vb.iter()).map(|(a, b)| a * b).sum();
                let na: f64 = va.iter().map(|x| x * x).sum::<f64>().sqrt();
                let nb: f64 = vb.iter().map(|x| x * x).sum::<f64>().sqrt();
                if na == 0.0 || nb == 0.0 {
                    return Deger::Sayi(0.0);
                }
                Deger::Sayi(dot / (na * nb))
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "vektor_topla".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            let get_vec = |d: &Deger| -> Option<Vec<f64>> {
                match d {
                    Deger::Vektor(v) => Some(v.borrow().clone()),
                    Deger::Liste(l) => Some(
                        l.borrow()
                            .iter()
                            .filter_map(|x| {
                                if let Deger::Sayi(n) = x {
                                    Some(*n)
                                } else {
                                    None
                                }
                            })
                            .collect(),
                    ),
                    _ => None,
                }
            };
            if let (Some(va), Some(vb)) = (get_vec(&args[0]), get_vec(&args[1])) {
                if va.len() != vb.len() {
                    return Deger::Hata("vektor_topla: boyutlar eşit olmalı".to_string());
                }
                let sonuc: Vec<f64> = va.iter().zip(vb.iter()).map(|(a, b)| a + b).collect();
                Deger::Vektor(Rc::new(RefCell::new(sonuc)))
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "vektor_carpi".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            let get_vec = |d: &Deger| -> Option<Vec<f64>> {
                match d {
                    Deger::Vektor(v) => Some(v.borrow().clone()),
                    Deger::Liste(l) => Some(
                        l.borrow()
                            .iter()
                            .filter_map(|x| {
                                if let Deger::Sayi(n) = x {
                                    Some(*n)
                                } else {
                                    None
                                }
                            })
                            .collect(),
                    ),
                    _ => None,
                }
            };
            if let (Some(va), Some(vb)) = (get_vec(&args[0]), get_vec(&args[1])) {
                if va.len() != vb.len() {
                    return Deger::Hata("vektor_carpi: boyutlar eşit olmalı".to_string());
                }
                let sonuc: Vec<f64> = va.iter().zip(vb.iter()).map(|(a, b)| a * b).collect();
                Deger::Vektor(Rc::new(RefCell::new(sonuc)))
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "vektor_skalar_carp".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            let skalar = match &args[1] {
                Deger::Sayi(n) => *n,
                _ => return Deger::Bos,
            };
            match &args[0] {
                Deger::Vektor(v) => {
                    let sonuc: Vec<f64> = v.borrow().iter().map(|x| x * skalar).collect();
                    Deger::Vektor(Rc::new(RefCell::new(sonuc)))
                }
                _ => Deger::Bos,
            }
        }),
    );

    globals.insert(
        "listeye_vektor".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Liste(l)) = args.first() {
                let v: Vec<f64> = l
                    .borrow()
                    .iter()
                    .filter_map(|d| {
                        if let Deger::Sayi(n) = d {
                            Some(*n)
                        } else {
                            None
                        }
                    })
                    .collect();
                Deger::Vektor(Rc::new(RefCell::new(v)))
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "vektore_liste".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Vektor(v)) = args.first() {
                let l: Vec<Deger> = v.borrow().iter().map(|x| Deger::Sayi(*x)).collect();
                Deger::Liste(Rc::new(RefCell::new(l)))
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "vektor_dilim".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 3 {
                if let (Deger::Vektor(v), Deger::Sayi(bas), Deger::Sayi(son)) =
                    (&args[0], &args[1], &args[2])
                {
                    let b = v.borrow();
                    let start = *bas as usize;
                    let end = (*son as usize).min(b.len());
                    if start <= end {
                        let dilim: Vec<f64> = b[start..end].to_vec();
                        return Deger::Vektor(Rc::new(RefCell::new(dilim)));
                    }
                }
            }
            Deger::Bos
        }),
    );

    // vektor_ekle — vektöre eleman ekle
    globals.insert(
        "vektor_ekle".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Vektor(v), Deger::Sayi(val)) = (&args[0], &args[1]) {
                    v.borrow_mut().push(*val);
                    return Deger::Vektor(Rc::clone(v));
                }
            }
            Deger::Bos
        }),
    );

    // vektor_al — vektörden indeks ile eleman oku
    globals.insert(
        "vektor_al".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (Some(Deger::Vektor(vektor)), Some(Deger::Sayi(indeks))) =
                (args.first(), args.get(1))
            else {
                return Deger::Hata("vektor_al: vektör ve indeks gerekir".to_string());
            };
            let vektor = vektor.borrow();
            let indeks = match indeks_dogrula(*indeks, vektor.len(), "vektor_al") {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            Deger::Sayi(vektor[indeks])
        }),
    );

    // vektor_ata — vektöre indeks ile değer yaz
    globals.insert(
        "vektor_ata".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (Some(Deger::Vektor(vektor)), Some(Deger::Sayi(indeks)), Some(Deger::Sayi(deger))) =
                (args.first(), args.get(1), args.get(2))
            else {
                return Deger::Hata("vektor_ata: vektör, indeks ve sayı gerekir".to_string());
            };
            if !deger.is_finite() {
                return Deger::Hata("vektor_ata: atanacak değer sonlu olmalı".to_string());
            }
            let mut vektor = vektor.borrow_mut();
            let indeks = match indeks_dogrula(*indeks, vektor.len(), "vektor_ata") {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            vektor[indeks] = *deger;
            Deger::Sayi(1.0)
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // BLOK D — Matris Operasyonları (Naive GEMM — portable, zero-dep)
    // ═══════════════════════════════════════════════════════════════════════

    globals.insert(
        "matris_olustur".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (Some(Deger::Sayi(satirlar)), Some(Deger::Sayi(sutunlar))) =
                (args.first(), args.get(1))
            else {
                return Deger::Hata("matris_olustur: satır ve sütun sayısı gerekir".to_string());
            };
            let satirlar = match boyut_dogrula(*satirlar, "matris_olustur", true) {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            let sutunlar = match boyut_dogrula(*sutunlar, "matris_olustur", true) {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            let eleman_sayisi = match eleman_sayisi_dogrula(satirlar, sutunlar, "matris_olustur") {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            let baslangic = match args.get(2) {
                Some(Deger::Sayi(deger)) if deger.is_finite() => *deger,
                Some(_) => {
                    return Deger::Hata(
                        "matris_olustur: başlangıç değeri sonlu sayı olmalı".to_string(),
                    )
                }
                None => 0.0,
            };
            Deger::Matris {
                satirlar,
                sutunlar,
                veri: Rc::new(RefCell::new(vec![baslangic; eleman_sayisi])),
            }
        }),
    );

    globals.insert(
        "matris_al".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (
                Some(Deger::Matris {
                    satirlar,
                    sutunlar,
                    veri,
                }),
                Some(Deger::Sayi(satir)),
                Some(Deger::Sayi(sutun)),
            ) = (args.first(), args.get(1), args.get(2))
            else {
                return Deger::Hata("matris_al: matris, satır ve sütun gerekir".to_string());
            };
            let satir = match indeks_dogrula(*satir, *satirlar, "matris_al satır") {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            let sutun = match indeks_dogrula(*sutun, *sutunlar, "matris_al sütun") {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            Deger::Sayi(veri.borrow()[satir * sutunlar + sutun])
        }),
    );

    globals.insert(
        "matris_ata".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (
                Some(Deger::Matris {
                    satirlar,
                    sutunlar,
                    veri,
                }),
                Some(Deger::Sayi(satir)),
                Some(Deger::Sayi(sutun)),
                Some(Deger::Sayi(deger)),
            ) = (args.first(), args.get(1), args.get(2), args.get(3))
            else {
                return Deger::Hata("matris_ata: matris, satır, sütun ve sayı gerekir".to_string());
            };
            if !deger.is_finite() {
                return Deger::Hata("matris_ata: atanacak değer sonlu olmalı".to_string());
            }
            let satir = match indeks_dogrula(*satir, *satirlar, "matris_ata satır") {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            let sutun = match indeks_dogrula(*sutun, *sutunlar, "matris_ata sütun") {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            veri.borrow_mut()[satir * sutunlar + sutun] = *deger;
            Deger::Sayi(1.0)
        }),
    );

    // matris_carp — Naive O(n³) GEMM
    globals.insert(
        "matris_carp".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            if let (
                Deger::Matris {
                    satirlar: ra,
                    sutunlar: ca,
                    veri: va,
                },
                Deger::Matris {
                    satirlar: rb,
                    sutunlar: cb,
                    veri: vb,
                },
            ) = (&args[0], &args[1])
            {
                if ca != rb {
                    return Deger::Hata(format!(
                        "matris_carp: boyut uyumsuzluğu {}x{} * {}x{}",
                        ra, ca, rb, cb
                    ));
                }
                let (m, n, k) = (*ra, *cb, *ca);
                let a = va.borrow();
                let b = vb.borrow();
                let mut c = vec![0.0f64; m * n];
                for i in 0..m {
                    for j in 0..n {
                        let mut s = 0.0f64;
                        for p in 0..k {
                            s += a[i * k + p] * b[p * n + j];
                        }
                        c[i * n + j] = s;
                    }
                }
                Deger::Matris {
                    satirlar: m,
                    sutunlar: n,
                    veri: Rc::new(RefCell::new(c)),
                }
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "matris_transpoz".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Matris {
                satirlar,
                sutunlar,
                veri,
            }) = args.first()
            {
                let b = veri.borrow();
                let mut c = vec![0.0f64; satirlar * sutunlar];
                for i in 0..*satirlar {
                    for j in 0..*sutunlar {
                        c[j * satirlar + i] = b[i * sutunlar + j];
                    }
                }
                Deger::Matris {
                    satirlar: *sutunlar,
                    sutunlar: *satirlar,
                    veri: Rc::new(RefCell::new(c)),
                }
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "matris_satir_al".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (
                Some(Deger::Matris {
                    satirlar,
                    sutunlar,
                    veri,
                }),
                Some(Deger::Sayi(satir)),
            ) = (args.first(), args.get(1))
            else {
                return Deger::Hata("matris_satir_al: matris ve satır gerekir".to_string());
            };
            let satir = match indeks_dogrula(*satir, *satirlar, "matris_satir_al") {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            let baslangic = satir * sutunlar;
            let bitis = baslangic + sutunlar;
            Deger::Vektor(Rc::new(RefCell::new(
                veri.borrow()[baslangic..bitis].to_vec(),
            )))
        }),
    );

    globals.insert(
        "matris_sutun_al".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (
                Some(Deger::Matris {
                    satirlar,
                    sutunlar,
                    veri,
                }),
                Some(Deger::Sayi(sutun)),
            ) = (args.first(), args.get(1))
            else {
                return Deger::Hata("matris_sutun_al: matris ve sütun gerekir".to_string());
            };
            let sutun = match indeks_dogrula(*sutun, *sutunlar, "matris_sutun_al") {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            let veri = veri.borrow();
            let sonuc = (0..*satirlar)
                .map(|satir| veri[satir * sutunlar + sutun])
                .collect();
            Deger::Vektor(Rc::new(RefCell::new(sonuc)))
        }),
    );

    globals.insert(
        "matris_satir_ata".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (
                Some(Deger::Matris {
                    satirlar,
                    sutunlar,
                    veri,
                }),
                Some(Deger::Sayi(satir)),
                Some(Deger::Vektor(yeni_satir)),
            ) = (args.first(), args.get(1), args.get(2))
            else {
                return Deger::Hata(
                    "matris_satir_ata: matris, satır ve vektör gerekir".to_string(),
                );
            };
            let satir = match indeks_dogrula(*satir, *satirlar, "matris_satir_ata") {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            let yeni_satir = yeni_satir.borrow();
            if yeni_satir.len() != *sutunlar {
                return Deger::Hata(format!(
                    "matris_satir_ata: vektör uzunluğu {} olmalı",
                    sutunlar
                ));
            }
            let baslangic = satir * sutunlar;
            veri.borrow_mut()[baslangic..baslangic + sutunlar].copy_from_slice(&yeni_satir);
            Deger::Sayi(1.0)
        }),
    );

    globals.insert(
        "kimlik_matrisi".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(n)) = args.first() {
                let n = match boyut_dogrula(*n, "kimlik_matrisi", true) {
                    Ok(deger) => deger,
                    Err(hata) => return Deger::Hata(hata),
                };
                let eleman_sayisi = match eleman_sayisi_dogrula(n, n, "kimlik_matrisi") {
                    Ok(deger) => deger,
                    Err(hata) => return Deger::Hata(hata),
                };
                let mut v = vec![0.0f64; eleman_sayisi];
                for i in 0..n {
                    v[i * n + i] = 1.0;
                }
                Deger::Matris {
                    satirlar: n,
                    sutunlar: n,
                    veri: Rc::new(RefCell::new(v)),
                }
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "matris_boyutu".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Matris {
                satirlar, sutunlar, ..
            }) = args.first()
            {
                Deger::Liste(Rc::new(RefCell::new(vec![
                    Deger::Sayi(*satirlar as f64),
                    Deger::Sayi(*sutunlar as f64),
                ])))
            } else {
                Deger::Bos
            }
        }),
    );

    // matris_vektor_carp — M * v (2D matris ile 1D vektör çarpımı)
    globals.insert(
        "matris_vektor_carp".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            if let (
                Deger::Matris {
                    satirlar,
                    sutunlar,
                    veri,
                },
                Deger::Vektor(v),
            ) = (&args[0], &args[1])
            {
                let mb = veri.borrow();
                let vb = v.borrow();
                if *sutunlar != vb.len() {
                    return Deger::Hata(format!(
                        "matris_vektor_carp: matris sütun {} ≠ vektör boyutu {}",
                        sutunlar,
                        vb.len()
                    ));
                }
                let sonuc: Vec<f64> = (0..*satirlar)
                    .map(|i| (0..*sutunlar).map(|j| mb[i * sutunlar + j] * vb[j]).sum())
                    .collect();
                Deger::Vektor(Rc::new(RefCell::new(sonuc)))
            } else {
                Deger::Bos
            }
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // BLOK E — Düzenli İfade (Regex) Built-in'leri
    // ═══════════════════════════════════════════════════════════════════════

    globals.insert(
        "regex_eslestir".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Metin(metin), Deger::Metin(desen)) = (&args[0], &args[1]) {
                    match Regex::new(desen) {
                        Ok(re) => return Deger::Sayi(if re.is_match(metin) { 1.0 } else { 0.0 }),
                        Err(e) => {
                            return Deger::Hata(format!("regex_eslestir: geçersiz desen — {}", e))
                        }
                    }
                }
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "regex_bul".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Metin(metin), Deger::Metin(desen)) = (&args[0], &args[1]) {
                    match Regex::new(desen) {
                        Ok(re) => {
                            if let Some(m) = re.find(metin) {
                                return Deger::Metin(m.as_str().to_string());
                            }
                            return Deger::Bos;
                        }
                        Err(e) => return Deger::Hata(format!("regex_bul: geçersiz desen — {}", e)),
                    }
                }
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "regex_bul_tum".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Metin(metin), Deger::Metin(desen)) = (&args[0], &args[1]) {
                    match Regex::new(desen) {
                        Ok(re) => {
                            let eslesme: Vec<Deger> = re
                                .find_iter(metin)
                                .map(|m| Deger::Metin(m.as_str().to_string()))
                                .collect();
                            return Deger::Liste(Rc::new(RefCell::new(eslesme)));
                        }
                        Err(e) => {
                            return Deger::Hata(format!("regex_bul_tum: geçersiz desen — {}", e))
                        }
                    }
                }
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "regex_degistir".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 3 {
                if let (Deger::Metin(metin), Deger::Metin(desen), Deger::Metin(yeni)) =
                    (&args[0], &args[1], &args[2])
                {
                    match Regex::new(desen) {
                        Ok(re) => {
                            return Deger::Metin(re.replace_all(metin, yeni.as_str()).into_owned())
                        }
                        Err(e) => {
                            return Deger::Hata(format!("regex_degistir: geçersiz desen — {}", e))
                        }
                    }
                }
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "regex_bol".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Metin(metin), Deger::Metin(desen)) = (&args[0], &args[1]) {
                    match Regex::new(desen) {
                        Ok(re) => {
                            let parcalar: Vec<Deger> = re
                                .split(metin)
                                .map(|p| Deger::Metin(p.to_string()))
                                .collect();
                            return Deger::Liste(Rc::new(RefCell::new(parcalar)));
                        }
                        Err(e) => return Deger::Hata(format!("regex_bol: geçersiz desen — {}", e)),
                    }
                }
            }
            Deger::Bos
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // BLOK F — Gelişmiş Rastgele Sayı Üretimi (SmallRng — seed destekli)
    // ═══════════════════════════════════════════════════════════════════════

    // Thread-local SmallRng
    thread_local! {
        static RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy());
    }

    // Box-Muller dönüşümü ile Normal dağılım örneği
    globals.insert(
        "normal_rastgele".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let ortalama = match args.first() {
                Some(Deger::Sayi(n)) => *n,
                _ => 0.0,
            };
            let sapma = match args.get(1) {
                Some(Deger::Sayi(n)) => *n,
                _ => 1.0,
            };
            let (u1, u2) = RNG.with(|rng| {
                let mut r = rng.borrow_mut();
                (r.gen::<f64>(), r.gen::<f64>())
            });
            let u1 = u1.max(1e-10);
            let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            Deger::Sayi(ortalama + sapma * z)
        }),
    );

    globals.insert(
        "uniform_rastgele".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let min = match args.first() {
                Some(Deger::Sayi(n)) => *n,
                _ => 0.0,
            };
            let max = match args.get(1) {
                Some(Deger::Sayi(n)) => *n,
                _ => 1.0,
            };
            let val = RNG.with(|rng| rng.borrow_mut().gen::<f64>());
            Deger::Sayi(min + val * (max - min))
        }),
    );

    globals.insert(
        "rastgele_tohum_ata".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(seed)) = args.first() {
                RNG.with(|rng| {
                    *rng.borrow_mut() = SmallRng::seed_from_u64(*seed as u64);
                });
                Deger::Sayi(1.0)
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "rastgele_tamsayi".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let min = match args.first() {
                Some(Deger::Sayi(n)) => *n as i64,
                _ => 0,
            };
            let max = match args.get(1) {
                Some(Deger::Sayi(n)) => *n as i64,
                _ => 100,
            };
            if min >= max {
                return Deger::Sayi(min as f64);
            }
            let val = RNG.with(|rng| rng.borrow_mut().gen_range(min..=max));
            Deger::Sayi(val as f64)
        }),
    );

    globals.insert(
        "vektor_karistir".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Vektor(v)) = args.first() {
                let mut b = v.borrow_mut();
                let n = b.len();
                for i in (1..n).rev() {
                    let j = RNG.with(|rng| rng.borrow_mut().gen_range(0..=i));
                    b.swap(i, j);
                }
                Deger::Sayi(1.0)
            } else if let Some(Deger::Liste(l)) = args.first() {
                let mut b = l.borrow_mut();
                let n = b.len();
                for i in (1..n).rev() {
                    let j = RNG.with(|rng| rng.borrow_mut().gen_range(0..=i));
                    b.swap(i, j);
                }
                Deger::Sayi(1.0)
            } else {
                Deger::Bos
            }
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // BLOK G — Gelişmiş Metin & Unicode Built-in'leri
    // ═══════════════════════════════════════════════════════════════════════

    globals.insert(
        "unicode_normalize".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(s)) = args.first() {
                let normalized: String = s.nfc().collect();
                Deger::Metin(normalized)
            } else {
                Deger::Bos
            }
        }),
    );

    // FNV-1a hash — hızlı, non-kriptografik; lookup table indekslemesi için
    globals.insert(
        "metin_hash".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(s)) = args.first() {
                let mut hash: u64 = 0xcbf29ce484222325u64;
                for byte in s.bytes() {
                    hash ^= byte as u64;
                    hash = hash.wrapping_mul(0x100000001b3u64);
                }
                Deger::Sayi(hash as f64)
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "bayt_metin".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(s)) = args.first() {
                let bytes: Vec<Deger> = s.bytes().map(|b| Deger::Sayi(b as f64)).collect();
                Deger::Liste(Rc::new(RefCell::new(bytes)))
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "metin_bayt".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Liste(l)) = args.first() {
                let bytes: Vec<u8> = l
                    .borrow()
                    .iter()
                    .filter_map(|d| {
                        if let Deger::Sayi(n) = d {
                            Some(*n as u8)
                        } else {
                            None
                        }
                    })
                    .collect();
                match String::from_utf8(bytes) {
                    Ok(s) => Deger::Metin(s),
                    Err(_) => Deger::Hata("metin_bayt: geçersiz UTF-8 bayt dizisi".to_string()),
                }
            } else {
                Deger::Bos
            }
        }),
    );

    // metin_benzerlik — normalized Levenshtein mesafesi (0.0 = farklı, 1.0 = aynı)
    globals.insert(
        "metin_benzerlik".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Metin(s1), Deger::Metin(s2)) = (&args[0], &args[1]) {
                    let a: Vec<char> = s1.chars().collect();
                    let b: Vec<char> = s2.chars().collect();
                    let la = a.len();
                    let lb = b.len();
                    if la == 0 && lb == 0 {
                        return Deger::Sayi(1.0);
                    }
                    let max_len = la.max(lb);
                    let mut dp = vec![vec![0usize; lb + 1]; la + 1];
                    for (i, row) in dp.iter_mut().enumerate() {
                        row[0] = i;
                    }
                    for (j, cell) in dp[0].iter_mut().enumerate() {
                        *cell = j;
                    }
                    for i in 1..=la {
                        for j in 1..=lb {
                            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
                            dp[i][j] = (dp[i - 1][j] + 1)
                                .min(dp[i][j - 1] + 1)
                                .min(dp[i - 1][j - 1] + cost);
                        }
                    }
                    let dist = dp[la][lb];
                    Deger::Sayi(1.0 - (dist as f64 / max_len as f64))
                } else {
                    Deger::Bos
                }
            } else {
                Deger::Bos
            }
        }),
    );

    // metin_sablon — basit {anahtar} şablon dönüşümü (sözlük tabanlı)
    globals.insert(
        "metin_sablon".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            let sablon = match &args[0] {
                Deger::Metin(s) => s.clone(),
                _ => return Deger::Bos,
            };
            let mut sonuc = sablon;
            let ikinci = &args[1];
            match ikinci {
                Deger::Sozluk(m) => {
                    for (k, v) in m.borrow().iter() {
                        sonuc = sonuc.replace(&format!("{{{}}}", k), &v.to_string());
                    }
                }
                Deger::Nesne { alanlar, .. } => {
                    for (k, v) in alanlar.borrow().iter() {
                        sonuc = sonuc.replace(&format!("{{{}}}", k), &v.to_string());
                    }
                }
                _ => return Deger::Bos,
            }
            Deger::Metin(sonuc)
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // P1.2 — Tensor Broadcasting & Toplu Matris Operasyonları
    // (Hüma döngüsü olmadan Rust'ta tam vektörize — kritik hız kazanımı)
    // ═══════════════════════════════════════════════════════════════════════

    // matris_satirlara_ekle(M, v) — v vektörünü M'nin her satırına ekle (bias addition)
    globals.insert(
        "matris_satirlara_ekle".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            if let (
                Deger::Matris {
                    satirlar,
                    sutunlar,
                    veri,
                },
                Deger::Vektor(v),
            ) = (&args[0], &args[1])
            {
                let vb = v.borrow();
                if *sutunlar != vb.len() {
                    return Deger::Hata(format!(
                        "matris_satirlara_ekle: matris sütun {} ≠ vektör boyutu {}",
                        sutunlar,
                        vb.len()
                    ));
                }
                let mut mb = veri.borrow().clone();
                for i in 0..*satirlar {
                    for j in 0..*sutunlar {
                        mb[i * sutunlar + j] += vb[j];
                    }
                }
                Deger::Matris {
                    satirlar: *satirlar,
                    sutunlar: *sutunlar,
                    veri: Rc::new(RefCell::new(mb)),
                }
            } else {
                Deger::Bos
            }
        }),
    );

    // matris_sutunlara_ekle(M, v) — v vektörünü M'nin her sütununa ekle
    globals.insert(
        "matris_sutunlara_ekle".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            if let (
                Deger::Matris {
                    satirlar,
                    sutunlar,
                    veri,
                },
                Deger::Vektor(v),
            ) = (&args[0], &args[1])
            {
                let vb = v.borrow();
                if *satirlar != vb.len() {
                    return Deger::Hata(format!(
                        "matris_sutunlara_ekle: matris satır {} ≠ vektör boyutu {}",
                        satirlar,
                        vb.len()
                    ));
                }
                let mut mb = veri.borrow().clone();
                for i in 0..*satirlar {
                    for j in 0..*sutunlar {
                        mb[i * sutunlar + j] += vb[i];
                    }
                }
                Deger::Matris {
                    satirlar: *satirlar,
                    sutunlar: *sutunlar,
                    veri: Rc::new(RefCell::new(mb)),
                }
            } else {
                Deger::Bos
            }
        }),
    );

    // matris_skalar_carp(M, s) — matrisin tüm elemanlarını skalar ile çarp
    globals.insert(
        "matris_skalar_carp".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            if let (
                Deger::Matris {
                    satirlar,
                    sutunlar,
                    veri,
                },
                Deger::Sayi(s),
            ) = (&args[0], &args[1])
            {
                let sonuc: Vec<f64> = veri.borrow().iter().map(|x| x * s).collect();
                Deger::Matris {
                    satirlar: *satirlar,
                    sutunlar: *sutunlar,
                    veri: Rc::new(RefCell::new(sonuc)),
                }
            } else {
                Deger::Bos
            }
        }),
    );

    // matris_elemanlari_topla(M) — tüm elemanların toplamı
    globals.insert(
        "matris_elemanlari_topla".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Matris { veri, .. }) = args.first() {
                Deger::Sayi(veri.borrow().iter().sum())
            } else {
                Deger::Bos
            }
        }),
    );

    // matris_satir_toplamları(M) — her satırın toplamı → Vektor [satirlar]
    globals.insert(
        "matris_satir_toplamları".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Matris {
                satirlar,
                sutunlar,
                veri,
            }) = args.first()
            {
                let b = veri.borrow();
                let sonuc: Vec<f64> = (0..*satirlar)
                    .map(|i| b[i * sutunlar..(i + 1) * sutunlar].iter().sum())
                    .collect();
                Deger::Vektor(Rc::new(RefCell::new(sonuc)))
            } else {
                Deger::Bos
            }
        }),
    );

    // matris_sutun_toplamları(M) — her sütunun toplamı → Vektor [sutunlar]
    globals.insert(
        "matris_sutun_toplamları".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Matris {
                satirlar,
                sutunlar,
                veri,
            }) = args.first()
            {
                let b = veri.borrow();
                let mut sonuc = vec![0.0f64; *sutunlar];
                for i in 0..*satirlar {
                    for j in 0..*sutunlar {
                        sonuc[j] += b[i * sutunlar + j];
                    }
                }
                Deger::Vektor(Rc::new(RefCell::new(sonuc)))
            } else {
                Deger::Bos
            }
        }),
    );

    // batch_softmax(M) — her satıra softmax uygula, Matris döndür
    globals.insert(
        "batch_softmax".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Matris {
                satirlar,
                sutunlar,
                veri,
            }) = args.first()
            {
                let b = veri.borrow();
                let mut sonuc = vec![0.0f64; satirlar * sutunlar];
                for i in 0..*satirlar {
                    let satir = &b[i * sutunlar..(i + 1) * sutunlar];
                    let max_val = satir.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    let exps: Vec<f64> = satir.iter().map(|x| (x - max_val).exp()).collect();
                    let toplam: f64 = exps.iter().sum();
                    for (j, e) in exps.iter().enumerate() {
                        sonuc[i * sutunlar + j] = if toplam > 0.0 {
                            e / toplam
                        } else {
                            1.0 / *sutunlar as f64
                        };
                    }
                }
                Deger::Matris {
                    satirlar: *satirlar,
                    sutunlar: *sutunlar,
                    veri: Rc::new(RefCell::new(sonuc)),
                }
            } else {
                Deger::Bos
            }
        }),
    );

    // matris_satir_normlari(M) — her satırın L2 normunu hesapla → Vektor
    globals.insert(
        "matris_satir_normlari".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Matris {
                satirlar,
                sutunlar,
                veri,
            }) = args.first()
            {
                let b = veri.borrow();
                let sonuc: Vec<f64> = (0..*satirlar)
                    .map(|i| {
                        b[i * sutunlar..(i + 1) * sutunlar]
                            .iter()
                            .map(|x| x * x)
                            .sum::<f64>()
                            .sqrt()
                    })
                    .collect();
                Deger::Vektor(Rc::new(RefCell::new(sonuc)))
            } else {
                Deger::Bos
            }
        }),
    );

    // vektor_dis_carpim(v1, v2) — dış çarpım → Matris [n1×n2]
    globals.insert(
        "vektor_dis_carpim".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            if let (Deger::Vektor(v1), Deger::Vektor(v2)) = (&args[0], &args[1]) {
                let b1 = v1.borrow();
                let b2 = v2.borrow();
                let n1 = b1.len();
                let n2 = b2.len();
                let mut sonuc = Vec::with_capacity(n1 * n2);
                for i in 0..n1 {
                    for j in 0..n2 {
                        sonuc.push(b1[i] * b2[j]);
                    }
                }
                Deger::Matris {
                    satirlar: n1,
                    sutunlar: n2,
                    veri: Rc::new(RefCell::new(sonuc)),
                }
            } else {
                Deger::Bos
            }
        }),
    );

    // Element-wise aktivasyon fonksiyonları (tüm matris üzerinde — döngüsüz)
    globals.insert(
        "matris_relu".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Matris {
                satirlar,
                sutunlar,
                veri,
            }) = args.first()
            {
                let sonuc: Vec<f64> = veri.borrow().iter().map(|x| x.max(0.0)).collect();
                Deger::Matris {
                    satirlar: *satirlar,
                    sutunlar: *sutunlar,
                    veri: Rc::new(RefCell::new(sonuc)),
                }
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "matris_sigmoid".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Matris {
                satirlar,
                sutunlar,
                veri,
            }) = args.first()
            {
                let sonuc: Vec<f64> = veri
                    .borrow()
                    .iter()
                    .map(|x| 1.0 / (1.0 + (-x).exp()))
                    .collect();
                Deger::Matris {
                    satirlar: *satirlar,
                    sutunlar: *sutunlar,
                    veri: Rc::new(RefCell::new(sonuc)),
                }
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "matris_tanh_akt".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Matris {
                satirlar,
                sutunlar,
                veri,
            }) = args.first()
            {
                let sonuc: Vec<f64> = veri.borrow().iter().map(|x| x.tanh()).collect();
                Deger::Matris {
                    satirlar: *satirlar,
                    sutunlar: *sutunlar,
                    veri: Rc::new(RefCell::new(sonuc)),
                }
            } else {
                Deger::Bos
            }
        }),
    );

    globals.insert(
        "matris_gelu".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Matris {
                satirlar,
                sutunlar,
                veri,
            }) = args.first()
            {
                let sonuc: Vec<f64> = veri
                    .borrow()
                    .iter()
                    .map(|x| {
                        0.5 * x * (1.0 + (0.7978845608028654 * (x + 0.044715 * x * x * x)).tanh())
                    })
                    .collect();
                Deger::Matris {
                    satirlar: *satirlar,
                    sutunlar: *sutunlar,
                    veri: Rc::new(RefCell::new(sonuc)),
                }
            } else {
                Deger::Bos
            }
        }),
    );

    // matris_klamp(M, min, max) — tüm elemanları sınırla
    globals.insert(
        "matris_klamp".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 3 {
                if let (
                    Deger::Matris {
                        satirlar,
                        sutunlar,
                        veri,
                    },
                    Deger::Sayi(min),
                    Deger::Sayi(max),
                ) = (&args[0], &args[1], &args[2])
                {
                    let sonuc: Vec<f64> =
                        veri.borrow().iter().map(|x| x.clamp(*min, *max)).collect();
                    return Deger::Matris {
                        satirlar: *satirlar,
                        sutunlar: *sutunlar,
                        veri: Rc::new(RefCell::new(sonuc)),
                    };
                }
            }
            Deger::Bos
        }),
    );

    // matris_topla(M1, M2) — element-wise matris toplama
    globals.insert(
        "matris_topla".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            if let (
                Deger::Matris {
                    satirlar: r1,
                    sutunlar: c1,
                    veri: v1,
                },
                Deger::Matris {
                    satirlar: r2,
                    sutunlar: c2,
                    veri: v2,
                },
            ) = (&args[0], &args[1])
            {
                if r1 != r2 || c1 != c2 {
                    return Deger::Hata("matris_topla: boyutlar eşit olmalı".to_string());
                }
                let sonuc: Vec<f64> = v1
                    .borrow()
                    .iter()
                    .zip(v2.borrow().iter())
                    .map(|(a, b)| a + b)
                    .collect();
                Deger::Matris {
                    satirlar: *r1,
                    sutunlar: *c1,
                    veri: Rc::new(RefCell::new(sonuc)),
                }
            } else {
                Deger::Bos
            }
        }),
    );

    // matris_cikart(M1, M2) — element-wise matris çıkarma
    globals.insert(
        "matris_cikart".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            if let (
                Deger::Matris {
                    satirlar: r1,
                    sutunlar: c1,
                    veri: v1,
                },
                Deger::Matris {
                    satirlar: r2,
                    sutunlar: c2,
                    veri: v2,
                },
            ) = (&args[0], &args[1])
            {
                if r1 != r2 || c1 != c2 {
                    return Deger::Hata("matris_cikart: boyutlar eşit olmalı".to_string());
                }
                let sonuc: Vec<f64> = v1
                    .borrow()
                    .iter()
                    .zip(v2.borrow().iter())
                    .map(|(a, b)| a - b)
                    .collect();
                Deger::Matris {
                    satirlar: *r1,
                    sutunlar: *c1,
                    veri: Rc::new(RefCell::new(sonuc)),
                }
            } else {
                Deger::Bos
            }
        }),
    );

    // matris_carpi_elemanlari(M1, M2) — element-wise (Hadamard) çarpım
    globals.insert(
        "matris_carpi_elemanlari".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            if let (
                Deger::Matris {
                    satirlar: r1,
                    sutunlar: c1,
                    veri: v1,
                },
                Deger::Matris {
                    satirlar: r2,
                    sutunlar: c2,
                    veri: v2,
                },
            ) = (&args[0], &args[1])
            {
                if r1 != r2 || c1 != c2 {
                    return Deger::Hata(
                        "matris_carpi_elemanlari: boyutlar eşit olmalı".to_string(),
                    );
                }
                let sonuc: Vec<f64> = v1
                    .borrow()
                    .iter()
                    .zip(v2.borrow().iter())
                    .map(|(a, b)| a * b)
                    .collect();
                Deger::Matris {
                    satirlar: *r1,
                    sutunlar: *c1,
                    veri: Rc::new(RefCell::new(sonuc)),
                }
            } else {
                Deger::Bos
            }
        }),
    );

    // gradyan_kirp(vektor, maks_norm) — gradient clipping (exploding gradients için)
    globals.insert(
        "gradyan_kirp".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            let maks_norm = match &args[1] {
                Deger::Sayi(n) => *n,
                _ => return Deger::Bos,
            };
            match &args[0] {
                Deger::Vektor(v) => {
                    let b = v.borrow();
                    let norm: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
                    if norm > maks_norm {
                        let olcek = maks_norm / norm;
                        let kirpilmis: Vec<f64> = b.iter().map(|x| x * olcek).collect();
                        Deger::Vektor(Rc::new(RefCell::new(kirpilmis)))
                    } else {
                        Deger::Vektor(Rc::clone(v))
                    }
                }
                Deger::Matris {
                    satirlar,
                    sutunlar,
                    veri,
                } => {
                    let b = veri.borrow();
                    let norm: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
                    if norm > maks_norm {
                        let olcek = maks_norm / norm;
                        let kirpilmis: Vec<f64> = b.iter().map(|x| x * olcek).collect();
                        Deger::Matris {
                            satirlar: *satirlar,
                            sutunlar: *sutunlar,
                            veri: Rc::new(RefCell::new(kirpilmis)),
                        }
                    } else {
                        args[0].clone()
                    }
                }
                _ => Deger::Bos,
            }
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // P1.3 — Büyük Dosya Streaming I/O
    // ═══════════════════════════════════════════════════════════════════════

    // dosya_satir_oku(yol) → tüm satırları liste olarak döndür (lazy-reader tarzı)
    globals.insert(
        "dosya_satir_oku".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(yol)) = args.first() {
                match std::fs::read_to_string(yol) {
                    Ok(icerik) => {
                        let satirlar: Vec<Deger> = icerik
                            .lines()
                            .map(|s| Deger::Metin(s.to_string()))
                            .collect();
                        Deger::Liste(Rc::new(RefCell::new(satirlar)))
                    }
                    Err(e) => Deger::Hata(format!("dosya_satir_oku: {}", e)),
                }
            } else {
                Deger::Bos
            }
        }),
    );

    // dosya_satir_ekle(yol, satir) → dosyaya yeni satır ekle (append mode)
    globals.insert(
        "dosya_satir_ekle".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Metin(yol), Deger::Metin(satir)) = (&args[0], &args[1]) {
                    use std::io::Write;
                    match std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(yol)
                    {
                        Ok(mut f) => {
                            let _ = writeln!(f, "{}", satir);
                            return Deger::Sayi(1.0);
                        }
                        Err(e) => return Deger::Hata(format!("dosya_satir_ekle: {}", e)),
                    }
                }
            }
            Deger::Bos
        }),
    );

    // csv_oku(yol) → [[satır elemanları]] listesi — basit CSV (virgülle ayrılmış)
    globals.insert(
        "csv_oku".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let ayrac = match args.get(1) {
                Some(Deger::Metin(s)) => s.chars().next().unwrap_or(','),
                _ => ',',
            };
            if let Some(Deger::Metin(yol)) = args.first() {
                match std::fs::read_to_string(yol) {
                    Ok(icerik) => {
                        let satirlar: Vec<Deger> = icerik
                            .lines()
                            .map(|satir| {
                                let parcalar: Vec<Deger> = satir
                                    .split(ayrac)
                                    .map(|p| Deger::Metin(p.trim().to_string()))
                                    .collect();
                                Deger::Liste(Rc::new(RefCell::new(parcalar)))
                            })
                            .collect();
                        Deger::Liste(Rc::new(RefCell::new(satirlar)))
                    }
                    Err(e) => Deger::Hata(format!("csv_oku: {}", e)),
                }
            } else {
                Deger::Bos
            }
        }),
    );

    // csv_yaz(yol, veri_listesi) → [[satır]] → CSV dosyası
    globals.insert(
        "csv_yaz".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            let ayrac = match args.get(2) {
                Some(Deger::Metin(s)) => s.clone(),
                _ => ",".to_string(),
            };
            if let (Deger::Metin(yol), Deger::Liste(satirlar)) = (&args[0], &args[1]) {
                let mut icerik = String::new();
                for satir in satirlar.borrow().iter() {
                    if let Deger::Liste(parcalar) = satir {
                        let parcalar_b = parcalar.borrow();
                        let satir_str: Vec<String> =
                            parcalar_b.iter().map(|p| p.to_string()).collect();
                        icerik.push_str(&satir_str.join(&ayrac));
                        icerik.push('\n');
                    }
                }
                match std::fs::write(yol, icerik) {
                    Ok(_) => Deger::Sayi(1.0),
                    Err(e) => Deger::Hata(format!("csv_yaz: {}", e)),
                }
            } else {
                Deger::Bos
            }
        }),
    );

    // jsonl_oku(yol) → her satır bir JSON nesnesi → nesne listesi
    globals.insert(
        "jsonl_oku".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(yol)) = args.first() {
                match std::fs::read_to_string(yol) {
                    Ok(icerik) => {
                        let nesneler: Vec<Deger> = icerik
                            .lines()
                            .filter(|s| !s.trim().is_empty())
                            .map(
                                |satir| match serde_json::from_str::<serde_json::Value>(satir) {
                                    Ok(v) => Deger::from_json(&v),
                                    Err(_) => Deger::Metin(satir.to_string()),
                                },
                            )
                            .collect();
                        Deger::Liste(Rc::new(RefCell::new(nesneler)))
                    }
                    Err(e) => Deger::Hata(format!("jsonl_oku: {}", e)),
                }
            } else {
                Deger::Bos
            }
        }),
    );

    // jsonl_yaz(yol, nesne) → nesneyi JSON satırı olarak dosyaya ekle
    globals.insert(
        "jsonl_yaz".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            if let Deger::Metin(yol) = &args[0] {
                use std::io::Write;
                let json_str = serde_json::to_string(&args[1].to_json()).unwrap_or_default();
                match std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(yol)
                {
                    Ok(mut f) => {
                        let _ = writeln!(f, "{}", json_str);
                        return Deger::Sayi(1.0);
                    }
                    Err(e) => return Deger::Hata(format!("jsonl_yaz: {}", e)),
                }
            }
            Deger::Bos
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // P1.4 — Boyut Doğrulama (Erken Hata Tespiti)
    // ═══════════════════════════════════════════════════════════════════════

    // matris_dogrula(M, beklenen_satir, beklenen_sutun) → 1 ya da Hata
    globals.insert(
        "matris_dogrula".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 3 {
                return Deger::Bos;
            }
            if let (
                Deger::Matris {
                    satirlar, sutunlar, ..
                },
                Deger::Sayi(br),
                Deger::Sayi(bs),
            ) = (&args[0], &args[1], &args[2])
            {
                if *satirlar != *br as usize || *sutunlar != *bs as usize {
                    return Deger::Hata(format!(
                        "matris_dogrula: beklenen {}×{}, bulunan {}×{}",
                        br, bs, satirlar, sutunlar
                    ));
                }
                Deger::Sayi(1.0)
            } else {
                Deger::Hata("matris_dogrula: geçersiz argümanlar".to_string())
            }
        }),
    );

    // vektor_dogrula(v, beklenen_boyut) → 1 ya da Hata
    globals.insert(
        "vektor_dogrula".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            if let (Deger::Vektor(v), Deger::Sayi(b)) = (&args[0], &args[1]) {
                let n = v.borrow().len();
                if n != *b as usize {
                    return Deger::Hata(format!("vektor_dogrula: beklenen {}, bulunan {}", b, n));
                }
                Deger::Sayi(1.0)
            } else {
                Deger::Hata("vektor_dogrula: geçersiz argümanlar".to_string())
            }
        }),
    );

    // boyut_esit_mi(a, b) → 1 ya da 0 — iki vektör/matrisin boyutu eşit mi?
    globals.insert(
        "boyut_esit_mi".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            let esit = match (&args[0], &args[1]) {
                (Deger::Vektor(v1), Deger::Vektor(v2)) => v1.borrow().len() == v2.borrow().len(),
                (
                    Deger::Matris {
                        satirlar: r1,
                        sutunlar: c1,
                        ..
                    },
                    Deger::Matris {
                        satirlar: r2,
                        sutunlar: c2,
                        ..
                    },
                ) => r1 == r2 && c1 == c2,
                _ => false,
            };
            Deger::Sayi(if esit { 1.0 } else { 0.0 })
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // P2.3 — Profiling & İlerleme Takibi
    // ═══════════════════════════════════════════════════════════════════════

    // zamanlayici_baslat() → anlık zaman (f64 saniye, yüksek çözünürlük)
    globals.insert(
        "zamanlayici_baslat".to_string(),
        Deger::DahiliFonksiyon(|_| {
            Deger::Sayi(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64(),
            )
        }),
    );

    // zamanlayici_bitir(baslangic) → geçen süre ms cinsinden
    globals.insert(
        "zamanlayici_bitir".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(baslangic)) = args.first() {
                let simdi = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                Deger::Sayi((simdi - baslangic) * 1000.0)
            } else {
                Deger::Bos
            }
        }),
    );

    // ilerleme_cubugu(simdi, toplam, mesaj) → ASCII progress bar yazdır
    globals.insert(
        "ilerleme_cubugu".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Sayi(simdi), Deger::Sayi(toplam)) = (&args[0], &args[1]) {
                    let mesaj = match args.get(2) {
                        Some(Deger::Metin(s)) => s.as_str(),
                        _ => "",
                    };
                    let yuzde = if *toplam > 0.0 {
                        (*simdi / toplam * 100.0) as usize
                    } else {
                        0
                    };
                    let dolu = yuzde / 5;
                    let bos = 20usize.saturating_sub(dolu);
                    let cubuk: String = format!(
                        "[{}{}] {}% {}",
                        "█".repeat(dolu),
                        "░".repeat(bos),
                        yuzde,
                        mesaj
                    );
                    print!("\r{}", cubuk);
                    use std::io::Write;
                    let _ = std::io::stdout().flush();
                    if (*simdi as usize) >= (*toplam as usize) {
                        println!();
                    }
                    return Deger::Sayi(1.0);
                }
            }
            Deger::Bos
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // P2.4 — Model Değerlendirme Metrikleri
    // ═══════════════════════════════════════════════════════════════════════

    // f1_skoru(tahmin_listesi, gercek_listesi) → {f1, precision, recall} sözlüğü
    globals.insert(
        "f1_skoru".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 {
                return Deger::Bos;
            }
            if let (Deger::Liste(tahmin_l), Deger::Liste(gercek_l)) = (&args[0], &args[1]) {
                let t = tahmin_l.borrow();
                let g = gercek_l.borrow();
                if t.len() != g.len() {
                    return Deger::Hata("f1_skoru: liste uzunlukları eşit olmalı".to_string());
                }
                let mut tp = 0.0f64;
                let mut fp = 0.0f64;
                let mut fn_ = 0.0f64;
                for (ti, gi) in t.iter().zip(g.iter()) {
                    let tv = match ti {
                        Deger::Sayi(n) => *n >= 0.5,
                        _ => false,
                    };
                    let gv = match gi {
                        Deger::Sayi(n) => *n >= 0.5,
                        _ => false,
                    };
                    match (tv, gv) {
                        (true, true) => tp += 1.0,
                        (true, false) => fp += 1.0,
                        (false, true) => fn_ += 1.0,
                        _ => {}
                    }
                }
                let precision = if tp + fp > 0.0 { tp / (tp + fp) } else { 0.0 };
                let recall = if tp + fn_ > 0.0 { tp / (tp + fn_) } else { 0.0 };
                let f1 = if precision + recall > 0.0 {
                    2.0 * precision * recall / (precision + recall)
                } else {
                    0.0
                };
                let mut m = std::collections::HashMap::new();
                m.insert("f1".to_string(), Deger::Sayi(f1));
                m.insert("precision".to_string(), Deger::Sayi(precision));
                m.insert("recall".to_string(), Deger::Sayi(recall));
                m.insert("tp".to_string(), Deger::Sayi(tp));
                m.insert("fp".to_string(), Deger::Sayi(fp));
                m.insert("fn".to_string(), Deger::Sayi(fn_));
                Deger::Sozluk(Rc::new(RefCell::new(m)))
            } else {
                Deger::Bos
            }
        }),
    );

    // karisiklik_matrisi(tahmin, gercek, sinif_sayisi) → Matris [n×n]
    globals.insert(
        "karisiklik_matrisi".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 3 {
                return Deger::Bos;
            }
            if let (Deger::Liste(tahmin_l), Deger::Liste(gercek_l), Deger::Sayi(sinif_n)) =
                (&args[0], &args[1], &args[2])
            {
                let n = *sinif_n as usize;
                let mut matris = vec![0.0f64; n * n];
                let t = tahmin_l.borrow();
                let g = gercek_l.borrow();
                for (ti, gi) in t.iter().zip(g.iter()) {
                    let tv = match ti {
                        Deger::Sayi(n) => *n as usize,
                        _ => 0,
                    };
                    let gv = match gi {
                        Deger::Sayi(n) => *n as usize,
                        _ => 0,
                    };
                    if gv < n && tv < n {
                        matris[gv * n + tv] += 1.0;
                    }
                }
                Deger::Matris {
                    satirlar: n,
                    sutunlar: n,
                    veri: Rc::new(RefCell::new(matris)),
                }
            } else {
                Deger::Bos
            }
        }),
    );

    // perplexity(log_olasiliklar_listesi) → e^(-ortalama_log_olasilik)
    globals.insert(
        "perplexity".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Liste(l)) = args.first() {
                let b = l.borrow();
                let n = b.len();
                if n == 0 {
                    return Deger::Bos;
                }
                let ort_log: f64 = b
                    .iter()
                    .filter_map(|d| {
                        if let Deger::Sayi(x) = d {
                            Some(*x)
                        } else {
                            None
                        }
                    })
                    .sum::<f64>()
                    / n as f64;
                Deger::Sayi((-ort_log).exp())
            } else if let Some(Deger::Vektor(v)) = args.first() {
                let b = v.borrow();
                let n = b.len();
                if n == 0 {
                    return Deger::Bos;
                }
                let ort_log: f64 = b.iter().sum::<f64>() / n as f64;
                Deger::Sayi((-ort_log).exp())
            } else {
                Deger::Bos
            }
        }),
    );

    crate::gui::kayit_et(&mut globals);

    globals
}

impl Default for Yorumlayici {
    fn default() -> Self {
        Self::new()
    }
}

impl Yorumlayici {
    pub fn new() -> Self {
        Self {
            global_degiskenler: varsayilan_global_degiskenler(),
            yerel_scopes: Vec::new(),
            donus_degeri: None,
            yuklenen_dosyalar: HashSet::new(),
            arama_yolları: vec![
                ".".to_string(),
                "./lib".to_string(),
                "./huma_modulleri".to_string(),
            ],
            output_buffer: None,
            call_depth: 0,
            runtime_errors: Vec::new(),
            dongu_kontrolu: None,
            dongu_derinligi: 0,
        }
    }

    pub fn fonksiyon_cagrisi(&mut self, f: Deger, args: Vec<Deger>) -> Deger {
        self.fonksiyon_cagrisi_detayli(f, args, None)
    }

    pub fn fonksiyon_cagrisi_detayli(
        &mut self,
        f: Deger,
        args: Vec<Deger>,
        nesne: Option<Deger>,
    ) -> Deger {
        if self.call_depth >= 50 {
            let message = "Azami özyineleme derinliği aşıldı".to_string();
            self.runtime_error_ekle(message.clone());
            return Deger::Hata(message);
        }
        self.call_depth += 1;

        let res = match f {
            Deger::Sinif {
                ad, alan_baslangic, ..
            } => {
                let alanlar = Rc::new(RefCell::new(HashMap::new()));
                for (alan_ad, alan_ifade) in alan_baslangic {
                    let val = self.ifade_hesapla(alan_ifade);
                    alanlar.borrow_mut().insert(alan_ad, val);
                }
                Deger::Nesne {
                    sinif_adi: ad,
                    alanlar,
                }
            }
            Deger::Fonksiyon {
                parametreler,
                govde,
            } => {
                let mut yerel = HashMap::new();
                if let Some(ins) = nesne {
                    yerel.insert("kendisi".to_string(), ins);
                }
                for (i, p) in parametreler.iter().enumerate() {
                    if i < args.len() {
                        yerel.insert(p.clone(), args[i].clone());
                    }
                }
                self.yerel_scopes.push(yerel);
                let eski = self.donus_degeri.take();
                let eski_dongu_kontrolu = self.dongu_kontrolu.take();
                let eski_dongu_derinligi = self.dongu_derinligi;
                self.dongu_derinligi = 0;
                for k in govde {
                    self.komut_calistir(k);
                    if self.donus_degeri.is_some()
                        || self.dongu_kontrolu.is_some()
                        || !self.runtime_errors.is_empty()
                    {
                        break;
                    }
                }
                let res = self.donus_degeri.take().unwrap_or(Deger::Bos);
                if self.dongu_kontrolu.take().is_some() {
                    self.runtime_error_ekle(
                        "devam/kır komutu bir fonksiyonun döngüsü dışında kullanılamaz".to_string(),
                    );
                }
                self.dongu_derinligi = eski_dongu_derinligi;
                self.dongu_kontrolu = eski_dongu_kontrolu;
                self.yerel_scopes.pop();
                self.donus_degeri = eski;
                res
            }
            Deger::DahiliFonksiyon(df) => df(args),
            _ => {
                let message = format!("Çağrılamayan değer: {}", f);
                self.runtime_error_ekle(message.clone());
                Deger::Hata(message)
            }
        };

        self.call_depth -= 1;
        res
    }

    pub fn with_output_buffer(mut self, buffer: Rc<RefCell<String>>) -> Self {
        self.output_buffer = Some(buffer);
        self
    }

    #[allow(dead_code)]
    fn yazdir(&self, content: &str) {
        if let Some(buf) = &self.output_buffer {
            buf.borrow_mut().push_str(content);
        } else {
            print!("{}", content);
            let _ = io::stdout().flush();
        }
    }

    fn satir_yazdir(&self, content: &str) {
        if let Some(buf) = &self.output_buffer {
            buf.borrow_mut().push_str(content);
            buf.borrow_mut().push('\n');
        } else {
            println!("{}", content);
        }
    }

    pub fn yorumla(&mut self, komutlar: Vec<Komut>) {
        for komut in komutlar {
            self.komut_calistir(komut);
            if self.donus_degeri.is_some() || !self.runtime_errors.is_empty() {
                break;
            }
        }
    }

    pub fn yorumla_kontrollu(&mut self, komutlar: Vec<Komut>) -> HumaResult<()> {
        self.runtime_errors.clear();
        self.yorumla(komutlar);
        match self.runtime_errors.first() {
            Some(message) => Err(HumaError::RuntimeError(message.clone())),
            None => Ok(()),
        }
    }

    pub fn runtime_hatalari(&self) -> &[String] {
        &self.runtime_errors
    }

    fn runtime_error_ekle(&mut self, message: String) {
        if !self
            .runtime_errors
            .iter()
            .any(|existing| existing == &message)
        {
            self.runtime_errors.push(message);
        }
    }

    fn get_degisken(&mut self, ad: &str) -> Deger {
        for scope in self.yerel_scopes.iter().rev() {
            if let Some(val) = scope.get(ad) {
                return val.clone();
            }
        }
        match self.global_degiskenler.get(ad).cloned() {
            Some(value) => value,
            None => {
                let message = format!("Tanımsız değişken: {}", ad);
                self.runtime_error_ekle(message.clone());
                Deger::Hata(message)
            }
        }
    }

    fn degisken_ata(&mut self, ad: String, deger: Deger) {
        if ad.contains('.') {
            let parts: Vec<&str> = ad.splitn(2, '.').collect();
            let nesne_adi = parts[0];
            let alan_adi = parts[1];
            let obj = self.get_degisken(nesne_adi);
            if let Deger::Nesne { alanlar, .. } = obj {
                alanlar.borrow_mut().insert(alan_adi.to_string(), deger);
                return;
            }
        }
        for scope in self.yerel_scopes.iter_mut().rev() {
            if let Some(current) = scope.get_mut(&ad) {
                *current = deger;
                return;
            }
        }
        self.global_degiskenler.insert(ad, deger);
    }

    fn degisken_tanimla(&mut self, ad: String, deger: Deger) {
        if ad.contains('.') {
            let parts: Vec<&str> = ad.splitn(2, '.').collect();
            let nesne_adi = parts[0];
            let alan_adi = parts[1];
            let obj = self.get_degisken(nesne_adi);
            if let Deger::Nesne { alanlar, .. } = obj {
                alanlar.borrow_mut().insert(alan_adi.to_string(), deger);
                return;
            }
        }
        if let Some(scope) = self.yerel_scopes.last_mut() {
            scope.insert(ad, deger);
        } else {
            self.global_degiskenler.insert(ad, deger);
        }
    }

    fn komut_calistir(&mut self, komut: Komut) {
        if self.donus_degeri.is_some()
            || self.dongu_kontrolu.is_some()
            || !self.runtime_errors.is_empty()
        {
            return;
        }
        match komut {
            Komut::YazdirKomutu(ifade) => {
                let d = self.ifade_hesapla(ifade);
                if self.runtime_errors.is_empty() {
                    self.satir_yazdir(&format!("{}", d));
                }
            }
            Komut::DegiskenTanimla { ad, deger } => {
                let res = self.ifade_hesapla(deger);
                if self.runtime_errors.is_empty() {
                    self.degisken_tanimla(ad, res);
                }
            }
            Komut::Atama { ad, deger } => {
                let res = self.ifade_hesapla(deger);
                if self.runtime_errors.is_empty() {
                    self.degisken_ata(ad, res);
                }
            }
            Komut::EgerKomutu {
                kosul,
                govde,
                degilse_govde,
            } => {
                let r = self.ifade_hesapla(kosul);
                if self.dogruluk_kontrolu(r) {
                    for k in govde {
                        self.komut_calistir(k);
                        if self.donus_degeri.is_some() {
                            break;
                        }
                    }
                } else if let Some(d) = degilse_govde {
                    for k in d {
                        self.komut_calistir(k);
                        if self.donus_degeri.is_some() {
                            break;
                        }
                    }
                }
            }
            Komut::DonguKomutu { kosul, govde } => {
                self.dongu_derinligi += 1;
                loop {
                    let r = self.ifade_hesapla(kosul.clone());
                    if !self.dogruluk_kontrolu(r) || self.donus_degeri.is_some() {
                        break;
                    }
                    for k in &govde {
                        self.komut_calistir(k.clone());
                        if self.donus_degeri.is_some()
                            || self.dongu_kontrolu.is_some()
                            || !self.runtime_errors.is_empty()
                        {
                            break;
                        }
                    }
                    match self.dongu_kontrolu.take() {
                        Some(DonguKontrolu::Kir) => break,
                        Some(DonguKontrolu::Devam) | None => {}
                    }
                    if !self.runtime_errors.is_empty() {
                        break;
                    }
                }
                self.dongu_derinligi = self.dongu_derinligi.saturating_sub(1);
            }
            Komut::FonksiyonTanimla {
                ad,
                parametreler,
                govde,
            } => {
                self.degisken_tanimla(
                    ad,
                    Deger::Fonksiyon {
                        parametreler,
                        govde,
                    },
                );
            }
            Komut::SinifTanimla { ad, metotlar } => {
                let mut ms = HashMap::new();
                // Sınıf içindeki değişken tanımlarını da işle
                let mut init_fields: Vec<(String, Ifade)> = Vec::new();
                for m in metotlar {
                    if let Komut::FonksiyonTanimla {
                        ad: m_ad,
                        parametreler,
                        govde,
                    } = m
                    {
                        ms.insert(m_ad, (parametreler, govde));
                    } else if let Komut::DegiskenTanimla { ad: f_ad, deger } = m {
                        init_fields.push((f_ad, deger));
                    }
                }
                self.global_degiskenler.insert(
                    ad.clone(),
                    Deger::Sinif {
                        ad,
                        metotlar: ms,
                        alan_baslangic: init_fields,
                    },
                );
            }
            Komut::DondurKomutu(ifade) => {
                let v = self.ifade_hesapla(ifade);
                self.donus_degeri = Some(v);
            }
            Komut::YukleKomutu(yol) => self.modül_yükle(&yol),
            Komut::ListeOlustur { ad } => {
                self.degisken_tanimla(ad, Deger::Liste(Rc::new(RefCell::new(Vec::new()))));
            }
            Komut::ListeEkle { liste, deger } => {
                let deger_val = self.ifade_hesapla(deger);
                let liste_val = self.ifade_hesapla(liste);
                if let Deger::Liste(l) = liste_val {
                    if let Deger::Liste(vals) = &deger_val {
                        // Eğer değer bir listeyse (özellikle [x] syntax'ında), elemanlarını ekle
                        l.borrow_mut().extend(vals.borrow().iter().cloned());
                    } else {
                        l.borrow_mut().push(deger_val);
                    }
                }
            }
            Komut::ListeCikar { liste, indeks } => {
                let idx_val = self.ifade_hesapla(indeks);
                let liste_val = self.ifade_hesapla(liste);

                // Eğer indeks bir listeyse (özellikle [i] syntax'ında), ilk elemanı al
                let mut final_idx = idx_val.clone();
                if let Deger::Liste(l_idx) = &idx_val {
                    let b = l_idx.borrow();
                    if let Some(first) = b.first() {
                        final_idx = first.clone();
                    }
                }

                if let (Deger::Liste(l), Deger::Sayi(i)) = (liste_val, final_idx) {
                    let idx = i as usize;
                    let mut b = l.borrow_mut();
                    if idx < b.len() {
                        b.remove(idx);
                    }
                }
            }
            Komut::DeneKomutu {
                dene_govde,
                hata_degisken,
                hata_govde,
            } => {
                let onceki_hatalar = std::mem::take(&mut self.runtime_errors);
                let onceki_donus = self.donus_degeri.take();
                let onceki_dongu_kontrolu = self.dongu_kontrolu.take();
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    for k in dene_govde {
                        self.komut_calistir(k);
                        if !self.runtime_errors.is_empty()
                            || self.donus_degeri.is_some()
                            || self.dongu_kontrolu.is_some()
                        {
                            break;
                        }
                    }
                }));
                let yakalanan_hata = match result {
                    Ok(()) => self.runtime_errors.first().cloned(),
                    Err(error) => Some(
                        error
                            .downcast_ref::<&str>()
                            .map(|message| (*message).to_string())
                            .or_else(|| error.downcast_ref::<String>().cloned())
                            .unwrap_or_else(|| "Bilinmeyen çalışma zamanı paniği".to_string()),
                    ),
                };

                if let Some(message) = yakalanan_hata {
                    self.runtime_errors = onceki_hatalar;
                    self.donus_degeri = onceki_donus;
                    self.dongu_kontrolu = onceki_dongu_kontrolu;
                    if let Some(var) = hata_degisken {
                        self.degisken_tanimla(var, Deger::Metin(message));
                    }
                    for k in hata_govde {
                        self.komut_calistir(k);
                        if self.donus_degeri.is_some()
                            || self.dongu_kontrolu.is_some()
                            || !self.runtime_errors.is_empty()
                        {
                            break;
                        }
                    }
                } else {
                    self.runtime_errors = onceki_hatalar;
                    if self.donus_degeri.is_none() {
                        self.donus_degeri = onceki_donus;
                    }
                    if self.dongu_kontrolu.is_none() {
                        self.dongu_kontrolu = onceki_dongu_kontrolu;
                    }
                }
            }
            Komut::AralikDongusu {
                degisken,
                baslangic,
                bitis,
                govde,
            } => {
                let start_val = self.ifade_hesapla(baslangic);
                let end_val = self.ifade_hesapla(bitis);
                if let (Deger::Sayi(s), Deger::Sayi(e)) = (start_val, end_val) {
                    if !s.is_finite() || !e.is_finite() {
                        self.runtime_error_ekle(
                            "Aralık sınırları sonlu sayılar olmalıdır".to_string(),
                        );
                        return;
                    }
                    self.dongu_derinligi += 1;
                    let mut i = s;
                    while i <= e {
                        self.degisken_tanimla(degisken.clone(), Deger::Sayi(i));
                        for k in &govde {
                            self.komut_calistir(k.clone());
                            if self.donus_degeri.is_some()
                                || self.dongu_kontrolu.is_some()
                                || !self.runtime_errors.is_empty()
                            {
                                break;
                            }
                        }
                        if self.donus_degeri.is_some() {
                            break;
                        }
                        match self.dongu_kontrolu.take() {
                            Some(DonguKontrolu::Kir) => break,
                            Some(DonguKontrolu::Devam) | None => {}
                        }
                        if !self.runtime_errors.is_empty() {
                            break;
                        }
                        i += 1.0;
                    }
                    self.dongu_derinligi = self.dongu_derinligi.saturating_sub(1);
                } else if self.runtime_errors.is_empty() {
                    self.runtime_error_ekle(
                        "Aralık başlangıcı ve bitişi sayı olmalıdır".to_string(),
                    );
                }
            }
            Komut::Devam => {
                if self.dongu_derinligi == 0 {
                    self.runtime_error_ekle(
                        "devam komutu yalnızca döngü içinde kullanılabilir".to_string(),
                    );
                } else {
                    self.dongu_kontrolu = Some(DonguKontrolu::Devam);
                }
            }
            Komut::Kir => {
                if self.dongu_derinligi == 0 {
                    self.runtime_error_ekle(
                        "kır komutu yalnızca döngü içinde kullanılabilir".to_string(),
                    );
                } else {
                    self.dongu_kontrolu = Some(DonguKontrolu::Kir);
                }
            }
            Komut::NesneAlaniAtama {
                nesne,
                ozellik,
                deger,
            } => {
                let deger_val = self.ifade_hesapla(deger);
                let nesne_val = self.ifade_hesapla(nesne);
                if let Deger::Nesne { alanlar, .. } = nesne_val {
                    alanlar.borrow_mut().insert(ozellik, deger_val);
                } else if self.runtime_errors.is_empty() {
                    self.runtime_error_ekle(
                        "Alan atamasının hedefi bir nesne olmalıdır".to_string(),
                    );
                }
            }
            Komut::IfadeKomutu(ifade) => {
                if let Ifade::IkiliIslem {
                    sol,
                    operator: Token::Esittir,
                    sag,
                } = ifade
                {
                    let d = self.ifade_hesapla(*sag);
                    match *sol {
                        Ifade::Degisken(ad) => self.degisken_ata(ad, d),
                        Ifade::NesneErisim { nesne, ozellik } => {
                            if let Deger::Nesne { alanlar, .. } = self.ifade_hesapla(*nesne) {
                                alanlar.borrow_mut().insert(ozellik, d);
                            }
                        }
                        Ifade::KendisiErisim { ozellik } => {
                            let kendisi = self.get_degisken("kendisi");
                            if let Deger::Nesne { alanlar, .. } = kendisi {
                                alanlar.borrow_mut().insert(ozellik, d);
                            }
                        }
                        Ifade::ListeErisim { liste, indeks } => {
                            let l_val = self.ifade_hesapla((*liste).clone());
                            let i_val = self.ifade_hesapla(*indeks);
                            match (l_val, i_val) {
                                (Deger::Liste(l), Deger::Sayi(i))
                                    if i >= 0.0 && i.fract() == 0.0 =>
                                {
                                    let idx = i as usize;
                                    let mut b = l.borrow_mut();
                                    if idx < b.len() {
                                        b[idx] = d.clone();
                                    } else {
                                        self.runtime_error_ekle(format!(
                                            "Liste atamasında indeks sınır dışında: {}",
                                            i
                                        ));
                                    }
                                }
                                (Deger::Sozluk(m), Deger::Metin(key)) => {
                                    m.borrow_mut().insert(key, d.clone());
                                }
                                (Deger::Nesne { alanlar, .. }, Deger::Metin(key)) => {
                                    alanlar.borrow_mut().insert(key, d.clone());
                                }
                                (container, index) if self.runtime_errors.is_empty() => {
                                    self.runtime_error_ekle(format!(
                                        "{} değerine {} indeksiyle atama yapılamaz",
                                        container, index
                                    ));
                                }
                                _ => {}
                            }
                        }
                        _ => self.runtime_error_ekle("Geçersiz atama hedefi".to_string()),
                    }
                } else {
                    self.ifade_hesapla(ifade);
                }
            }
        }
    }

    fn modül_yükle(&mut self, dosya_adı: &str) {
        // Önce gömülü kütüphaneleri kontrol et
        for (ad, icerik) in builtin_files::get_lib_files() {
            if ad == dosya_adı {
                if self.yuklenen_dosyalar.contains(ad) {
                    return;
                }
                self.yuklenen_dosyalar.insert(ad.to_string());
                let mut parser = crate::parser::Parser::new(crate::lexer::Lexer::new(icerik));
                let (prog, diagnostics) = parser.parse_program_with_diagnostics();
                if let Some(first) = diagnostics.into_iter().next() {
                    self.runtime_error_ekle(format!("{} modülü: {}", dosya_adı, first));
                    return;
                }
                let eski = self.donus_degeri.take(); // Save return value state
                self.yorumla(prog);
                self.donus_degeri = eski; // Restore return value state
                return;
            }
        }

        let mut bulundu = None;
        for temel in &self.arama_yolları {
            let tam_yol = format!("{}/{}", temel, dosya_adı);
            let path = Path::new(&tam_yol);
            if path.is_file() {
                bulundu = Some(tam_yol);
                break;
            }

            // Paket yöneticisi için destek: modul/modul.hb pattern'ini kontrol et
            let paket_yol = format!("{}/{}/{}.hb", temel, dosya_adı, dosya_adı);
            if Path::new(&paket_yol).is_file() {
                bulundu = Some(paket_yol);
                break;
            }

            // Uzantı ekleyerek kontrol et
            if !dosya_adı.ends_with(".hb") {
                let hb_yol = format!("{}.hb", tam_yol);
                if Path::new(&hb_yol).is_file() {
                    bulundu = Some(hb_yol);
                    break;
                }
            }
        }

        if let Some(yol) = bulundu {
            if self.yuklenen_dosyalar.contains(&yol) {
                return;
            }
            self.yuklenen_dosyalar.insert(yol.clone());
            match std::fs::read_to_string(&yol) {
                Ok(icerik) => {
                    let mut parser = crate::parser::Parser::new(crate::lexer::Lexer::new(&icerik));
                    let (prog, diagnostics) = parser.parse_program_with_diagnostics();
                    if let Some(first) = diagnostics.into_iter().next() {
                        self.runtime_error_ekle(format!("{} modülü: {}", dosya_adı, first));
                        return;
                    }
                    let eski = self.donus_degeri.take();

                    let mut pushed = false;
                    if let Some(parent) = Path::new(&yol).parent() {
                        let parent_str = parent.to_string_lossy().to_string();
                        if !parent_str.is_empty() && !self.arama_yolları.contains(&parent_str) {
                            self.arama_yolları.insert(0, parent_str);
                            pushed = true;
                        }
                    }

                    self.yorumla(prog);

                    if pushed {
                        self.arama_yolları.remove(0);
                    }
                    self.donus_degeri = eski;
                }
                Err(error) => {
                    self.runtime_error_ekle(format!("Modül okunamadı ({}): {}", dosya_adı, error));
                }
            }
        } else {
            self.runtime_error_ekle(format!("Modül bulunamadı: {}", dosya_adı));
        }
    }

    fn ifade_hesapla(&mut self, ifade: Ifade) -> Deger {
        let value = self.ifade_hesapla_ic(ifade);
        if let Deger::Hata(message) = &value {
            self.runtime_error_ekle(message.clone());
        }
        value
    }

    fn ifade_hesapla_ic(&mut self, ifade: Ifade) -> Deger {
        match ifade {
            Ifade::Bekle(inner) => {
                let v = self.ifade_hesapla(*inner);
                match v {
                    Deger::GorevId(id) => YAPRAK.with(|y| y.borrow_mut().await_task(id)),
                    other => Deger::Hata(format!("bekle: await edilemez değer: {}", other)),
                }
            }
            Ifade::Sayi(n) => Deger::Sayi(n),
            Ifade::Metin(s) => Deger::Metin(s),
            Ifade::Bos => Deger::Bos,
            Ifade::Dogru => Deger::Sayi(1.0),
            Ifade::Yanlis => Deger::Sayi(0.0),
            Ifade::Degisken(ad) => self.get_degisken(&ad),
            Ifade::Liste(el) => Deger::Liste(Rc::new(RefCell::new(
                el.into_iter().map(|e| self.ifade_hesapla(e)).collect(),
            ))),
            Ifade::Sozluk(el) => {
                let mut map = HashMap::new();
                for (k, v) in el {
                    let key = self.ifade_hesapla(k).to_string();
                    let val = self.ifade_hesapla(v);
                    map.insert(key, val);
                }
                Deger::Sozluk(Rc::new(RefCell::new(map)))
            }
            Ifade::ListeErisim { liste, indeks } => {
                let l_val = self.ifade_hesapla(*liste);
                let i_val = self.ifade_hesapla(*indeks);
                match (l_val, i_val) {
                    (Deger::Liste(l), Deger::Sayi(i)) if i >= 0.0 && i.fract() == 0.0 => {
                        l.borrow().get(i as usize).cloned().unwrap_or_else(|| {
                            Deger::Hata(format!("Liste indeksi sınır dışında: {}", i))
                        })
                    }
                    (Deger::Metin(s), Deger::Sayi(i)) if i >= 0.0 && i.fract() == 0.0 => s
                        .chars()
                        .nth(i as usize)
                        .map(|c| Deger::Metin(c.to_string()))
                        .unwrap_or_else(|| {
                            Deger::Hata(format!("Metin indeksi sınır dışında: {}", i))
                        }),
                    (Deger::Sozluk(m), Deger::Metin(key)) => {
                        m.borrow().get(&key).cloned().unwrap_or(Deger::Bos)
                    }
                    (Deger::Nesne { alanlar, .. }, Deger::Metin(key)) => {
                        alanlar.borrow().get(&key).cloned().unwrap_or(Deger::Bos)
                    }
                    (container, index) => Deger::Hata(format!(
                        "{} değeri {} indeksiyle erişilemez",
                        container, index
                    )),
                }
            }
            Ifade::NesneErisim { nesne, ozellik } => {
                let inst = self.ifade_hesapla(*nesne);
                if let Deger::Nesne { alanlar, .. } = inst {
                    alanlar.borrow().get(&ozellik).cloned().unwrap_or_else(|| {
                        Deger::Hata(format!("Nesne özelliği bulunamadı: {}", ozellik))
                    })
                } else if let Deger::Sozluk(m) = inst {
                    m.borrow().get(&ozellik).cloned().unwrap_or(Deger::Bos)
                } else {
                    Deger::Hata(format!("Nesne özelliğine erişilemez: {}", ozellik))
                }
            }
            Ifade::KendisiErisim { ozellik } => {
                let kendisi = self.get_degisken("kendisi");
                if let Deger::Nesne { alanlar, .. } = kendisi {
                    alanlar.borrow().get(&ozellik).cloned().unwrap_or_else(|| {
                        Deger::Hata(format!("Nesne özelliği bulunamadı: {}", ozellik))
                    })
                } else {
                    Deger::Hata("kendisi yalnızca sınıf metotlarında kullanılabilir".to_string())
                }
            }
            Ifade::Uzunluk(ifade) => {
                let val = self.ifade_hesapla(*ifade);
                match val {
                    Deger::Liste(l) => Deger::Sayi(l.borrow().len() as f64),
                    Deger::Metin(s) => Deger::Sayi(s.chars().count() as f64),
                    Deger::Sozluk(m) => Deger::Sayi(m.borrow().len() as f64),
                    other => Deger::Hata(format!("{} değerinin uzunluğu alınamaz", other)),
                }
            }
            Ifade::FonksiyonIfadesi {
                parametreler,
                govde,
            } => Deger::Fonksiyon {
                parametreler,
                govde,
            },
            Ifade::NesneOlustur { sinif_adi, .. } => Deger::Hata(format!(
                "Eski nesne oluşturma AST biçimi desteklenmiyor: {}",
                sinif_adi
            )),
            Ifade::Cagri {
                fonksiyon,
                argumanlar,
                pos,
            } => {
                let mut method_instance = None;
                let f = if let Ifade::NesneErisim { nesne, ozellik } = *fonksiyon.clone() {
                    let instance = self.ifade_hesapla(*nesne);
                    if let Deger::Nesne {
                        ref sinif_adi,
                        ref alanlar,
                    } = instance
                    {
                        // 1. Önce sınıf metotlarını kontrol et
                        if let Some(Deger::Sinif { metotlar, .. }) =
                            self.global_degiskenler.get(sinif_adi)
                        {
                            if let Some((ps, bd)) = metotlar.get(&ozellik) {
                                method_instance = Some(instance.clone());
                                Deger::Fonksiyon {
                                    parametreler: ps.clone(),
                                    govde: bd.clone(),
                                }
                            } else {
                                // 2. Sınıf metodu yoksa alanlara bak
                                if let Some(field_val) = alanlar.borrow().get(&ozellik) {
                                    if matches!(
                                        field_val,
                                        Deger::Fonksiyon { .. } | Deger::DahiliFonksiyon(_)
                                    ) {
                                        method_instance = Some(instance.clone());
                                    }
                                    field_val.clone()
                                } else {
                                    self.ifade_hesapla(*fonksiyon)
                                }
                            }
                        } else {
                            // 3. Sınıf yoksa (düz nesne) alanlara bak
                            if let Some(field_val) = alanlar.borrow().get(&ozellik) {
                                if matches!(
                                    field_val,
                                    Deger::Fonksiyon { .. } | Deger::DahiliFonksiyon(_)
                                ) {
                                    method_instance = Some(instance.clone());
                                }
                                field_val.clone()
                            } else {
                                self.ifade_hesapla(*fonksiyon)
                            }
                        }
                    } else if let Deger::Sozluk(ref m) = instance {
                        if ozellik == "getir" {
                            let args = argumanlar
                                .into_iter()
                                .map(|a| self.ifade_hesapla(a))
                                .collect::<Vec<_>>();
                            if let Some(Deger::Metin(k)) = args.first() {
                                return m.borrow().get(k).cloned().unwrap_or(Deger::Bos);
                            }
                            return Deger::Bos;
                        } else if ozellik == "ayarla" {
                            let args = argumanlar
                                .into_iter()
                                .map(|a| self.ifade_hesapla(a))
                                .collect::<Vec<_>>();
                            if args.len() >= 2 {
                                if let Deger::Metin(k) = &args[0] {
                                    m.borrow_mut().insert(k.clone(), args[1].clone());
                                }
                            }
                            return Deger::Bos;
                        } else {
                            self.ifade_hesapla(*fonksiyon)
                        }
                    } else {
                        self.ifade_hesapla(*fonksiyon)
                    }
                } else {
                    self.ifade_hesapla(*fonksiyon)
                };

                let args = argumanlar
                    .into_iter()
                    .map(|a| self.ifade_hesapla(a))
                    .collect();
                if !matches!(
                    f,
                    Deger::Fonksiyon { .. } | Deger::DahiliFonksiyon(_) | Deger::Sinif { .. }
                ) {
                    return Deger::Hata(format!(
                        "Satır {}, Sütun {}: Çağrılamayan değer: {}",
                        pos.0, pos.1, f
                    ));
                }
                self.fonksiyon_cagrisi_detayli(f, args, method_instance)
            }
            Ifade::IkiliIslem { sol, operator, sag } => {
                let mut l = self.ifade_hesapla(*sol);
                if operator == Token::Ve {
                    if !self.dogruluk_kontrolu(l) {
                        return Deger::Sayi(0.0);
                    }
                    let r = self.ifade_hesapla(*sag);
                    return Deger::Sayi(if self.dogruluk_kontrolu(r) { 1.0 } else { 0.0 });
                }
                if operator == Token::Veya {
                    if self.dogruluk_kontrolu(l) {
                        return Deger::Sayi(1.0);
                    }
                    let r = self.ifade_hesapla(*sag);
                    return Deger::Sayi(if self.dogruluk_kontrolu(r) { 1.0 } else { 0.0 });
                }
                let mut r = self.ifade_hesapla(*sag);

                // Tip zorlama (Coercion) - Arti hariç diğer sayısal işlemlerde zorla
                if matches!(
                    operator,
                    Token::Eksi
                        | Token::Carpi
                        | Token::Bolnu
                        | Token::Mod
                        | Token::Kucuktur
                        | Token::Buyuktur
                        | Token::KucukEsit
                        | Token::BuyukEsit
                ) {
                    if let Deger::Metin(ref s) = l {
                        if let Ok(n) = s.parse::<f64>() {
                            l = Deger::Sayi(n);
                        }
                    }
                    if let Deger::Metin(ref s) = r {
                        if let Ok(n) = s.parse::<f64>() {
                            r = Deger::Sayi(n);
                        }
                    }
                }

                match operator {
                    Token::EsitEsittir | Token::Esittir => {
                        Deger::Sayi(if l == r { 1.0 } else { 0.0 })
                    }
                    Token::EsitDegil => Deger::Sayi(if l != r { 1.0 } else { 0.0 }),
                    _ => match (l, r) {
                        (Deger::Sayi(a), Deger::Sayi(b)) => match operator {
                            Token::Arti => Deger::Sayi(a + b),
                            Token::Eksi => Deger::Sayi(a - b),
                            Token::Carpi => Deger::Sayi(a * b),
                            Token::Bolnu if b == 0.0 => {
                                Deger::Hata("Sıfıra bölme hatası".to_string())
                            }
                            Token::Bolnu => Deger::Sayi(a / b),
                            Token::Mod if b == 0.0 => {
                                Deger::Hata("Sıfıra göre kalan hesaplanamaz".to_string())
                            }
                            Token::Mod => Deger::Sayi(a % b),
                            Token::Kucuktur => Deger::Sayi(if a < b { 1.0 } else { 0.0 }),
                            Token::Buyuktur => Deger::Sayi(if a > b { 1.0 } else { 0.0 }),
                            Token::KucukEsit => Deger::Sayi(if a <= b { 1.0 } else { 0.0 }),
                            Token::BuyukEsit => Deger::Sayi(if a >= b { 1.0 } else { 0.0 }),
                            _ => Deger::Hata(format!(
                                "Desteklenmeyen sayısal operatör: {}",
                                operator
                            )),
                        },
                        (l_val, r_val) => match operator {
                            Token::Arti => {
                                // Sayısal olmayan kombinasyonlarda tip kontrolü:
                                // - Sayı + Boş veya Boş + Sayı → Tip Hatası (sessiz string birleştirme engellendi)
                                // - Metin + herhangi → string birleştirme (izin verilir)
                                let l_is_num = matches!(l_val, Deger::Sayi(_));
                                let r_is_num = matches!(r_val, Deger::Sayi(_));
                                let l_is_bos = matches!(l_val, Deger::Bos);
                                let r_is_bos = matches!(r_val, Deger::Bos);

                                if (l_is_num && r_is_bos) || (l_is_bos && r_is_num) {
                                    // Sayı ile Boş toplanamaz — Tip Hatası
                                    Deger::Hata(format!(
                                        "Tip Hatası: '{}' ile '{}' toplanamaz. Sayısal işlemde Boş değer kullanılamaz.",
                                        l_val, r_val
                                    ))
                                } else {
                                    // Metin birleştirme (Metin + herhangi veya herhangi + Metin)
                                    Deger::Metin(format!("{}{}", l_val, r_val))
                                }
                            }
                            Token::Eksi | Token::Carpi | Token::Bolnu | Token::Mod => {
                                // Sayısal operatörlerde Boş değer kullanılamaz → Tip Hatası
                                Deger::Hata(format!(
                                    "Tip Hatası: '{}' ve '{}' arasında sayısal işlem yapılamaz. Her iki değer de sayı olmalıdır.",
                                    l_val, r_val
                                ))
                            }
                            Token::Kucuktur => {
                                Deger::Sayi(if l_val.to_string() < r_val.to_string() {
                                    1.0
                                } else {
                                    0.0
                                })
                            }
                            Token::Buyuktur => {
                                Deger::Sayi(if l_val.to_string() > r_val.to_string() {
                                    1.0
                                } else {
                                    0.0
                                })
                            }
                            _ => Deger::Hata(format!(
                                "Desteklenmeyen işlem: {} ile {} arasında {}",
                                l_val, r_val, operator
                            )),
                        },
                    },
                }
            }
            Ifade::MantıksalDegil(i) => {
                let v = self.ifade_hesapla(*i);
                Deger::Sayi(if self.dogruluk_kontrolu(v) { 0.0 } else { 1.0 })
            }
        }
    }

    fn dogruluk_kontrolu(&self, deger: Deger) -> bool {
        match deger {
            Deger::Sayi(n) => n != 0.0,
            Deger::Metin(s) => !s.is_empty(),
            Deger::Liste(l) => !l.borrow().is_empty(),
            Deger::Sozluk(m) => !m.borrow().is_empty(),
            Deger::Bos => false,
            _ => true,
        }
    }
}
