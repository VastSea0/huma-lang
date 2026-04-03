use crate::ast::{Ifade, Komut};
use crate::token::Token;
use crate::value::Deger;
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};
use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;
use std::time::Duration;
use crate::builtin_files;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::io::Read;

static SUNUCULAR: Lazy<Mutex<HashMap<u64, tiny_http::Server>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static ISTEKLER: Lazy<Mutex<HashMap<u64, tiny_http::Request>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static SQL_CONNECTIONS: Lazy<Mutex<HashMap<u64, rusqlite::Connection>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static NEXT_ID: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(1));

fn get_id() -> u64 {
    let mut id = NEXT_ID.lock().unwrap();
    let old = *id;
    *id += 1;
    old
}

pub struct Yorumlayici {

    pub global_degiskenler: HashMap<String, Deger>,
    pub yerel_scopes: Vec<HashMap<String, Deger>>,
    pub donus_degeri: Option<Deger>,
    pub yuklenen_dosyalar: HashSet<String>,
    pub arama_yolları: Vec<String>,
    pub output_buffer: Option<Rc<RefCell<String>>>,
}

impl Yorumlayici {
    pub fn new() -> Self {
        let mut globals = HashMap::new();
        globals.insert("uzunluk".to_string(), Deger::DahiliFonksiyon(|args| {
            match args.first() {
                Some(Deger::Metin(s)) => Deger::Sayi(s.chars().count() as f64),
                Some(Deger::Liste(l)) => Deger::Sayi(l.borrow().len() as f64),
                _ => Deger::Sayi(0.0),
            }
        }));
        globals.insert("oku".to_string(), Deger::DahiliFonksiyon(|args| {
            if let Some(msg) = args.first() { print!("{}", msg); let _ = io::stdout().flush(); }
            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_ok() { Deger::Metin(input.trim().to_string()) } else { Deger::Bos }
        }));
        globals.insert("uyut".to_string(), Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(ms)) = args.first() { if *ms > 0.0 { thread::sleep(Duration::from_millis(*ms as u64)); } }
            Deger::Bos
        }));
        globals.insert("zaman".to_string(), Deger::DahiliFonksiyon(|_| {
            Deger::Sayi(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64())
        }));
        globals.insert("listeye_ekle".to_string(), Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let Deger::Liste(l) = &args[0] {
                    // Semantik: Eski kodu bozmamak için kopyasını döndür (O(N))
                    // Ama NLP kütüphanesi mutation kullanacak şekilde güncellenecek.
                    let mut yeni = l.borrow().clone();
                    yeni.push(args[1].clone());
                    return Deger::Liste(Rc::new(RefCell::new(yeni)));
                }
            }
            Deger::Bos
        }));
        globals.insert("karekök".to_string(), Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(n)) = args.first() { Deger::Sayi(n.sqrt()) } else { Deger::Bos }
        }));
        globals.insert("rastgele".to_string(), Deger::DahiliFonksiyon(|_| {
            let n = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as f64;
            Deger::Sayi((n % 1000000.0) / 1000000.0)
        }));
        globals.insert("dosya_oku".to_string(), Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(yol)) = args.first() {
                if let Ok(s) = std::fs::read_to_string(yol) { return Deger::Metin(s); }
            }
            Deger::Bos
        }));
        globals.insert("dosya_yaz".to_string(), Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Metin(yol), Deger::Metin(icerik)) = (&args[0], &args[1]) {
                    if std::fs::write(yol, icerik).is_ok() { return Deger::Sayi(1.0); }
                }
            }
            Deger::Sayi(0.0)
        }));
        globals.insert("sistem".to_string(), Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(komut)) = args.first() {
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(komut)
                    .output();
                match output {
                    Ok(o) => {
                        let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                        return Deger::Metin(s);
                    }
                    Err(_) => return Deger::Bos,
                }
            }
            Deger::Bos
        }));
            // dahili_sunucu_baslat(port)
        globals.insert("dahili_sunucu_baslat".to_string(), Deger::DahiliFonksiyon(|args| {
            let port = match args.first() { Some(Deger::Sayi(n)) => *n as u16, _ => 8080 };
            match tiny_http::Server::http(format!("0.0.0.0:{}", port)) {
                Ok(server) => {
                    let id = get_id();
                    SUNUCULAR.lock().unwrap().insert(id, server);
                    Deger::Sayi(id as f64)
                }
                Err(_) => Deger::Bos
            }
        }));
        // dahili_sunucu_bekle(id)
        globals.insert("dahili_sunucu_bekle".to_string(), Deger::DahiliFonksiyon(|args| {
            let id = match args.first() { Some(Deger::Sayi(n)) => *n as u64, _ => return Deger::Bos };
            if let Some(server) = SUNUCULAR.lock().unwrap().get(&id) {
                if let Ok(mut istek) = server.recv() {
                    let mut govde = String::new();
                    let _ = istek.as_reader().read_to_string(&mut govde);
                    
                    let i_id = get_id();

                    let url = istek.url().to_string();
                    let metot = istek.method().to_string();
                    
                    ISTEKLER.lock().unwrap().insert(i_id, istek);
                    
                    let mut fields = HashMap::new();
                    fields.insert("id".to_string(), Deger::Sayi(i_id as f64));
                    fields.insert("url".to_string(), Deger::Metin(url));
                    fields.insert("metot".to_string(), Deger::Metin(metot));
                    fields.insert("gövde".to_string(), Deger::Metin(govde));
                    
                    return Deger::Nesne { sinif_adi: "İstek".to_string(), alanlar: Rc::new(RefCell::new(fields)) };
                }
            }
            Deger::Bos
        }));
        // dahili_sunucu_yanitla(i_id, icerik, durum, tip, [basliklar])
        globals.insert("dahili_sunucu_yanitla".to_string(), Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 { return Deger::Sayi(0.0); }
            let i_id = match &args[0] { Deger::Sayi(n) => *n as u64, _ => return Deger::Sayi(0.0) };
            
            let (data, len) = match &args[1] {
                Deger::Metin(s) => (s.as_bytes().to_vec(), s.len()),
                Deger::Bayt(b) => (b.clone(), b.len()),
                _ => (Vec::new(), 0),
            };

            let durum = match args.get(2) { Some(Deger::Sayi(n)) => *n as u16, _ => 200 };
            let tip = match args.get(3) { Some(Deger::Metin(s)) => s.as_str(), _ => "text/html; charset=utf-8" };
            
            if let Some(istek) = ISTEKLER.lock().unwrap().remove(&i_id) {
                let mut response = tiny_http::Response::new(
                    tiny_http::StatusCode(durum),
                    vec![tiny_http::Header::from_bytes(&b"Content-Type"[..], tip.as_bytes()).unwrap()],
                    std::io::Cursor::new(data),
                    Some(len),
                    None,
                );
                
                // Ek başlıkları ekle (5. argüman)
                if args.len() >= 5 {
                    if let Deger::Nesne { alanlar, .. } = &args[4] {
                        for (k, v) in alanlar.borrow().iter() {
                            if let Ok(header) = tiny_http::Header::from_bytes(k.as_bytes(), v.to_string().as_bytes()) {
                                response.add_header(header);
                            }
                        }
                    }
                }

                let _ = istek.respond(response);
                return Deger::Sayi(1.0);
            }
            Deger::Sayi(0.0)
        }));

        globals.insert("dosya_oku_bayt".to_string(), Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(yol)) = args.first() {
                if let Ok(b) = std::fs::read(yol) { return Deger::Bayt(b); }
            }
            Deger::Bos
        }));
        // dahili_istek(metot, url, [gövde])

        globals.insert("dahili_istek".to_string(), Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 { return Deger::Bos; }
            let metot = match &args[0] { Deger::Metin(s) => s.to_uppercase(), _ => "GET".to_string() };
            let url = match &args[1] { Deger::Metin(s) => s, _ => return Deger::Bos };
            let govdeli = args.len() >= 3 && !matches!(args[2], Deger::Bos);
            let govde = if govdeli { match &args[2] { Deger::Metin(s) => s.clone(), _ => String::new() } } else { String::new() };
            
            let mut req = ureq::request(&metot, url);
            
            // Başlıkları ekle (4. argüman)
            if args.len() >= 4 {
                if let Deger::Nesne { alanlar, .. } = &args[3] {
                    for (k, v) in alanlar.borrow().iter() {
                        req = req.set(k, &v.to_string());
                    }
                }
            }
            
            // Eğer gövde varsa ve Content-Type belirtilmemişse varsayılan olarak JSON denebilir
            // Ama şimdilik kullanıcıya bırakmak daha esnek.

            let response = if govdeli && (metot == "POST" || metot == "PUT" || metot == "PATCH" || metot == "DELETE") {
                req.send_string(&govde)
            } else {
                req.call()
            };

            match response {
                Ok(res) => {
                    let durum = res.status() as f64;
                    let icerik = res.into_string().unwrap_or_default();
                    let alanlar = HashMap::from([
                        ("durum".to_string(), Deger::Sayi(durum)),
                        ("içerik".to_string(), Deger::Metin(icerik)),
                    ]);
                    Deger::Nesne { sinif_adi: "İstekCevabı".to_string(), alanlar: Rc::new(RefCell::new(alanlar)) }
                }
                Err(e) => {
                    let mut alanlar = HashMap::new();
                    alanlar.insert("durum".to_string(), Deger::Sayi(0.0));
                    alanlar.insert("hata".to_string(), Deger::Metin(e.to_string()));
                    Deger::Nesne { sinif_adi: "İstekHatası".to_string(), alanlar: Rc::new(RefCell::new(alanlar)) }
                }
            }
        }));

        globals.insert("dosya_var_mı".to_string(), Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(yol)) = args.first() {
                return Deger::Sayi(if Path::new(yol).exists() { 1.0 } else { 0.0 });
            }
            Deger::Sayi(0.0)
        }));
        // JSON Fonksiyonları
        globals.insert("nesneden_metine".to_string(), Deger::DahiliFonksiyon(|args| {
            if let Some(d) = args.first() {
                if let Ok(s) = serde_json::to_string_pretty(&d.to_json()) {
                    return Deger::Metin(s);
                }
            }
            Deger::Metin("null".to_string())
        }));
        globals.insert("metinden_nesneye".to_string(), Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(s)) = args.first() {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) {
                    return Deger::from_json(&v);
                }
            }
            Deger::Bos
        }));
        globals.insert("tipi".to_string(), Deger::DahiliFonksiyon(|args| {

            match args.first() {
                Some(Deger::Sayi(_)) => Deger::Metin("Sayı".to_string()),
                Some(Deger::Metin(_)) => Deger::Metin("Metin".to_string()),
                Some(Deger::Liste(_)) => Deger::Metin("Liste".to_string()),
                Some(Deger::Fonksiyon { .. }) => Deger::Metin("Fonksiyon".to_string()),
                Some(Deger::Sinif { .. }) => Deger::Metin("Sınıf".to_string()),
                Some(Deger::Nesne { .. }) => Deger::Metin("Nesne".to_string()),
                _ => Deger::Metin("Boş".to_string()),
            }
        }));

        globals.insert("ortam_değişkeni".to_string(), Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(anahtar)) = args.first() {
                if let Ok(val) = std::env::var(anahtar) {
                    return Deger::Metin(val);
                }
            }
            Deger::Bos
        }));

        // ── NLP / Metin İşleme Built-in Fonksiyonları ──────────────────────────

        // küçük_harf(metin) → Türkçe-farkında küçük harf dönüşümü
        globals.insert("küçük_harf".to_string(), Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(s)) = args.first() {
                let sonuc: String = s.chars().map(|c| match c {
                    'I' => 'ı', 'İ' => 'i', 'Ğ' => 'ğ', 'Ş' => 'ş',
                    'Ç' => 'ç', 'Ö' => 'ö', 'Ü' => 'ü',
                    _ => c.to_lowercase().next().unwrap_or(c),
                }).collect();
                Deger::Metin(sonuc)
            } else { Deger::Bos }
        }));

        // büyük_harf(metin) → Türkçe-farkında büyük harf dönüşümü
        globals.insert("büyük_harf".to_string(), Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(s)) = args.first() {
                let sonuc: String = s.chars().map(|c| match c {
                    'ı' => 'I', 'i' => 'İ', 'ğ' => 'Ğ', 'ş' => 'Ş',
                    'ç' => 'Ç', 'ö' => 'Ö', 'ü' => 'Ü',
                    _ => c.to_uppercase().next().unwrap_or(c),
                }).collect();
                Deger::Metin(sonuc)
            } else { Deger::Bos }
        }));

        // böl(metin, ayraç) → Liste döndürür
        globals.insert("böl".to_string(), Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Metin(s), Deger::Metin(ayrac)) = (&args[0], &args[1]) {
                    let parcalar: Vec<Deger> = if ayrac.is_empty() {
                        s.chars().map(|c| Deger::Metin(c.to_string())).collect()
                    } else {
                        s.split(ayrac.as_str()).map(|p| Deger::Metin(p.to_string())).collect()
                    };
                    return Deger::Liste(Rc::new(RefCell::new(parcalar)));
                }
            }
            Deger::Bos
        }));

        // birleştir(liste, ayraç) → birleştirilmiş metin
        globals.insert("birleştir".to_string(), Deger::DahiliFonksiyon(|args| {
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
        }));

        // değiştir(metin, aranan, yeni) → yeni metin
        globals.insert("değiştir".to_string(), Deger::DahiliFonksiyon(|args| {
            if args.len() >= 3 {
                if let (Deger::Metin(s), Deger::Metin(aranan), Deger::Metin(yeni)) =
                    (&args[0], &args[1], &args[2])
                {
                    return Deger::Metin(s.replace(aranan.as_str(), yeni.as_str()));
                }
            }
            Deger::Bos
        }));

        // kırp(metin) → baştaki ve sondaki boşlukları sil
        globals.insert("kırp".to_string(), Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(s)) = args.first() {
                Deger::Metin(s.trim().to_string())
            } else { Deger::Bos }
        }));

        // tekrar_sayısı(metin, aranan) → kaç kez geçiyor
        globals.insert("tekrar_sayısı".to_string(), Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Metin(s), Deger::Metin(aranan)) = (&args[0], &args[1]) {
                    if aranan.is_empty() { return Deger::Sayi(0.0); }
                    return Deger::Sayi(s.matches(aranan.as_str()).count() as f64);
                }
            }
            Deger::Bos
        }));

        // sayıya_çevir(metin) → Sayı değerine dönüştür
        globals.insert("sayıya_çevir".to_string(), Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(s)) = args.first() {
                if let Ok(n) = s.trim().parse::<f64>() { return Deger::Sayi(n); }
            } else if let Some(Deger::Sayi(n)) = args.first() {
                return Deger::Sayi(*n);
            }
            Deger::Bos
        }));

        // metne_çevir(değer) → Metin değerine dönüştür
        globals.insert("metne_çevir".to_string(), Deger::DahiliFonksiyon(|args| {
            if let Some(d) = args.first() {
                Deger::Metin(d.to_string())
            } else { Deger::Bos }
        }));

        // ascii_kodu(karakter) → Unicode kod noktası
        globals.insert("ascii_kodu".to_string(), Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(s)) = args.first() {
                if let Some(c) = s.chars().next() {
                    return Deger::Sayi(c as u32 as f64);
                }
            }
            Deger::Bos
        }));

        // karakterden(kod) → Unicode karakterini metin olarak döndür
        globals.insert("karakterden".to_string(), Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(n)) = args.first() {
                if let Some(c) = char::from_u32(*n as u32) {
                    return Deger::Metin(c.to_string());
                }
            }
            Deger::Bos
        }));

        // içeriyor(metin_veya_liste_veya_nesne, aranan) → 1 veya 0
        globals.insert("içeriyor".to_string(), Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                match (&args[0], &args[1]) {
                    (Deger::Metin(s), Deger::Metin(aranan)) => {
                        return Deger::Sayi(if s.contains(aranan.as_str()) { 1.0 } else { 0.0 });
                    }
                    (Deger::Liste(l), aranan) => {
                        return Deger::Sayi(if l.borrow().contains(aranan) { 1.0 } else { 0.0 });
                    }
                    (Deger::Nesne { alanlar, .. }, Deger::Metin(anahtar)) => {
                        return Deger::Sayi(if alanlar.borrow().contains_key(anahtar) { 1.0 } else { 0.0 });
                    }
                    _ => {}
                }
            }
            Deger::Sayi(0.0)
        }));

        // değer_al(nesne, anahtar) → değer
        globals.insert("değer_al".to_string(), Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Nesne { alanlar, .. }, Deger::Metin(anahtar)) = (&args[0], &args[1]) {
                    return alanlar.borrow().get(anahtar).cloned().unwrap_or(Deger::Bos);
                }
            }
            Deger::Bos
        }));

        // değer_ata(nesne, anahtar, değer)
        globals.insert("değer_ata".to_string(), Deger::DahiliFonksiyon(|args| {
            if args.len() >= 3 {
                if let (Deger::Nesne { alanlar, .. }, Deger::Metin(anahtar)) = (&args[0], &args[1]) {
                    alanlar.borrow_mut().insert(anahtar.clone(), args[2].clone());
                    return Deger::Sayi(1.0);
                }
            }
            Deger::Sayi(0.0)
        }));


        // hızlı_içeriyor(liste, eleman) → O(1) arama (HashSet kullanarak)
        // NOT: Bu fonksiyon ilk çağrıda listeyi bir küme gibi önbelleğe alırsa daha da hızlanır,
        // ancak KISS prensibi gereği şimdilik her seferinde HashSet oluşturuyoruz (geçici çözüm).
        // Gerçek çözüm: Liste'yi küme tipine çeviren bir built-in eklemek.
        globals.insert("hızlı_içeriyor".to_string(), Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Liste(l), eleman) = (&args[0], &args[1]) {
                    let set: HashSet<String> = l.borrow().iter().map(|d| d.to_string()).collect();
                    return Deger::Sayi(if set.contains(&eleman.to_string()) { 1.0 } else { 0.0 });
                }
            }
            Deger::Bos
        }));

        // başlıyor_mu(metin, ön_ek) → 1 veya 0
        globals.insert("başlıyor_mu".to_string(), Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Metin(s), Deger::Metin(onek)) = (&args[0], &args[1]) {
                    return Deger::Sayi(if s.starts_with(onek.as_str()) { 1.0 } else { 0.0 });
                }
            }
            Deger::Bos
        }));

        // bitiyor_mu(metin, son_ek) → 1 veya 0
        globals.insert("bitiyor_mu".to_string(), Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Metin(s), Deger::Metin(sonek)) = (&args[0], &args[1]) {
                    return Deger::Sayi(if s.ends_with(sonek.as_str()) { 1.0 } else { 0.0 });
                }
            }
            Deger::Bos
        }));

        // dizi_dilim(metin, baş, son) → alt metin (char-bazlı, Unicode güvenli)
        globals.insert("dizi_dilim".to_string(), Deger::DahiliFonksiyon(|args| {
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
        }));

        // ── SQL / Veritabanı Fonksiyonları ───────────────────────────────
        globals.insert("dahili_sql_bağlan".to_string(), Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(yol)) = args.first() {
                if let Ok(conn) = rusqlite::Connection::open(yol) {
                    let id = get_id();
                    SQL_CONNECTIONS.lock().unwrap().insert(id, conn);
                    return Deger::Sayi(id as f64);
                }
            }
            Deger::Bos
        }));

        globals.insert("dahili_sql_yürüt".to_string(), Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 { return Deger::Sayi(0.0); }
            if let (Deger::Sayi(id), Deger::Metin(sql)) = (&args[0], &args[1]) {
                let conn_id = *id as u64;
                let mut conns = SQL_CONNECTIONS.lock().unwrap();
                if let Some(conn) = conns.get(&conn_id) {
                    match conn.execute(sql, []) {
                        Ok(_) => return Deger::Sayi(1.0),
                        Err(e) => eprintln!("[Hüma SQL Hatası] Yürütme: {}", e),
                    }
                } else {
                    eprintln!("[Hüma SQL Hatası] Bağlantı ID bulunamadı: {} (Haritadaki boyut: {})", conn_id, conns.len());
                }
            }
            Deger::Sayi(0.0)
        }));

        globals.insert("dahili_sql_sorgula".to_string(), Deger::DahiliFonksiyon(|args| {
            if args.len() < 2 { return Deger::Bos; }
            if let (Deger::Sayi(id), Deger::Metin(sql)) = (&args[0], &args[1]) {
                let conn_id = *id as u64;
                let mut conns = SQL_CONNECTIONS.lock().unwrap();
                if let Some(conn) = conns.get(&conn_id) {
                    match conn.prepare(sql) {
                        Ok(mut stmt) => {
                            let col_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
                            let rows: Vec<Deger> = stmt.query_map([], |row| {
                                let mut fields = HashMap::new();
                                for i in 0..col_names.len() {
                                    let val: rusqlite::types::Value = row.get(i).unwrap_or(rusqlite::types::Value::Null);
                                    let d_val = match val {
                                        rusqlite::types::Value::Integer(i) => Deger::Sayi(i as f64),
                                        rusqlite::types::Value::Real(f) => Deger::Sayi(f),
                                        rusqlite::types::Value::Text(t) => Deger::Metin(t),
                                        _ => Deger::Bos,
                                    };
                                    let col_name = col_names[i].to_lowercase().trim().to_string();
                                    fields.insert(col_name, d_val);
                                }
                                Ok(Deger::Nesne { sinif_adi: "Satır".to_string(), alanlar: Rc::new(RefCell::new(fields)) })
                            }).unwrap().flatten().collect();
                            
                            return Deger::Liste(Rc::new(RefCell::new(rows)));
                        }
                        Err(e) => eprintln!("[Hüma SQL Hatası] Sorgulama: {}", e),
                    }
                } else {
                    eprintln!("[Hüma SQL Hatası] Sorgulama - Bağlantı ID bulunamadı: {} (Haritadaki boyut: {})", conn_id, conns.len());
                }
            }
            Deger::Bos
        }));

        // ── Argümanları al ──────────────────────────────────────────────────────
        let cli_args: Vec<Deger> = std::env::args().map(|s| Deger::Metin(s)).collect();
        globals.insert("argümanlar".to_string(), Deger::Liste(Rc::new(RefCell::new(cli_args))));

        // ── GUI Fonksiyonları ─────────────────────────────────────────────────
        crate::gui::kayit_et(&mut globals);

        Self { 
            global_degiskenler: globals, 
            yerel_scopes: Vec::new(), 
            donus_degeri: None, 
            yuklenen_dosyalar: HashSet::new(), 
            arama_yolları: vec![".".to_string(), "./lib".to_string(), "./huma_modulleri".to_string()],
            output_buffer: None,
        }
    }

    pub fn fonksiyon_cagrisi(&mut self, f: Deger, args: Vec<Deger>) -> Deger {
        self.fonksiyon_cagrisi_detayli(f, args, None)
    }

    pub fn fonksiyon_cagrisi_detayli(&mut self, f: Deger, args: Vec<Deger>, nesne: Option<Deger>) -> Deger {
        match f {
            Deger::Sinif { ad, alan_baslangic, .. } => {
                let alanlar = Rc::new(RefCell::new(HashMap::new()));
                for (alan_ad, alan_ifade) in alan_baslangic {
                    let val = self.ifade_hesapla(alan_ifade);
                    alanlar.borrow_mut().insert(alan_ad, val);
                }
                Deger::Nesne { sinif_adi: ad, alanlar }
            },
            Deger::Fonksiyon { parametreler, govde } => {
                let mut yerel = HashMap::new();
                if let Some(ins) = nesne { yerel.insert("kendisi".to_string(), ins); }
                for (i, p) in parametreler.iter().enumerate() {
                    if i < args.len() {
                        yerel.insert(p.clone(), args[i].clone());
                    }
                }
                self.yerel_scopes.push(yerel);
                let eski = self.donus_degeri.take();
                for k in govde { 
                    self.komut_calistir(k); 
                    if self.donus_degeri.is_some() { break; } 
                }
                let res = self.donus_degeri.take().unwrap_or(Deger::Bos);
                self.yerel_scopes.pop(); 
                self.donus_degeri = eski; 
                res
            }
            Deger::DahiliFonksiyon(df) => {
                df(args)
            }
            _ => Deger::Bos
        }
    }

    pub fn with_output_buffer(mut self, buffer: Rc<RefCell<String>>) -> Self {
        self.output_buffer = Some(buffer);
        self
    }

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
            if self.donus_degeri.is_some() { break; }
        }
    }

    fn get_degisken(&self, ad: &str) -> Deger {
        for scope in self.yerel_scopes.iter().rev() {
            if let Some(val) = scope.get(ad) { return val.clone(); }
        }
        self.global_degiskenler.get(ad).cloned().unwrap_or(Deger::Bos)
    }

    fn degisken_ata(&mut self, ad: String, deger: Deger) {
        for scope in self.yerel_scopes.iter_mut().rev() {
            if scope.contains_key(&ad) { scope.insert(ad, deger); return; }
        }
        self.global_degiskenler.insert(ad, deger);
    }

    fn degisken_tanimla(&mut self, ad: String, deger: Deger) {
        if let Some(scope) = self.yerel_scopes.last_mut() {
            scope.insert(ad, deger);
        } else {
            self.global_degiskenler.insert(ad, deger);
        }
    }

    fn komut_calistir(&mut self, komut: Komut) {
        if self.donus_degeri.is_some() { return; }
        match komut {
            Komut::YazdirKomutu(ifade) => {
                let d = self.ifade_hesapla(ifade);
                self.satir_yazdir(&format!("{}", d));
            }
            Komut::DegiskenTanimla { ad, deger } => {
                let res = self.ifade_hesapla(deger);
                self.degisken_tanimla(ad, res);
            }
            Komut::Atama { ad, deger } => {
                let res = self.ifade_hesapla(deger);
                self.degisken_ata(ad, res);
            }
            Komut::EgerKomutu { kosul, govde, degilse_govde } => {
                let r = self.ifade_hesapla(kosul);
                if self.dogruluk_kontrolu(r) {
                    for k in govde { self.komut_calistir(k); if self.donus_degeri.is_some() { break; } }
                } else if let Some(d) = degilse_govde {
                    for k in d { self.komut_calistir(k); if self.donus_degeri.is_some() { break; } }
                }
            }
            Komut::DonguKomutu { kosul, govde } => {
                loop {
                    let r = self.ifade_hesapla(kosul.clone());
                    if !self.dogruluk_kontrolu(r) || self.donus_degeri.is_some() { break; }
                    for k in &govde { self.komut_calistir(k.clone()); if self.donus_degeri.is_some() { break; } }
                }
            }
            Komut::FonksiyonTanimla { ad, parametreler, govde } => {
                self.degisken_tanimla(ad, Deger::Fonksiyon { parametreler, govde });
            }
            Komut::SinifTanimla { ad, metotlar } => {
                let mut ms = HashMap::new();
                // Sınıf içindeki değişken tanımlarını da işle
                let mut init_fields: Vec<(String, Ifade)> = Vec::new();
                for m in metotlar {
                    if let Komut::FonksiyonTanimla { ad: m_ad, parametreler, govde } = m {
                        ms.insert(m_ad, (parametreler, govde));
                    } else if let Komut::DegiskenTanimla { ad: f_ad, deger } = m {
                        init_fields.push((f_ad, deger));
                    }
                }
                self.global_degiskenler.insert(ad.clone(), Deger::Sinif { ad, metotlar: ms, alan_baslangic: init_fields });
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
                    l.borrow_mut().push(deger_val);
                }
            }
            Komut::ListeCikar { liste, indeks } => {
                let idx_val = self.ifade_hesapla(indeks);
                let liste_val = self.ifade_hesapla(liste);
                if let (Deger::Liste(l), Deger::Sayi(i)) = (liste_val, idx_val) {
                    let idx = i as usize;
                    let mut b = l.borrow_mut();
                    if idx < b.len() {
                        b.remove(idx);
                    }
                }
            }
            Komut::DeneKomutu { dene_govde, hata_govde } => {
                // Basit hata yönetimi — dene bloğunu çalıştır, panic olursa hata bloğunu çalıştır
                // Rust'ta gerçek try-catch yok, bu yüzden basit bir yaklaşım:
                // Şimdilik dene bloğunu çalıştır, hata olursa hata bloğunu çalıştır
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let mut temp_interp = Yorumlayici::new();
                    // Global değişkenleri kopyala
                    temp_interp.global_degiskenler = self.global_degiskenler.clone();
                    temp_interp.yerel_scopes = self.yerel_scopes.clone();
                    for k in dene_govde.clone() {
                        temp_interp.komut_calistir(k);
                    }
                    temp_interp
                }));
                match result {
                    Ok(temp) => {
                        self.global_degiskenler = temp.global_degiskenler;
                        self.yerel_scopes = temp.yerel_scopes;
                    }
                    Err(_) => {
                        for k in hata_govde { self.komut_calistir(k); if self.donus_degeri.is_some() { break; } }
                    }
                }
            }
            Komut::NesneAlaniAtama { nesne, ozellik, deger } => {
                let deger_val = self.ifade_hesapla(deger);
                let nesne_val = self.ifade_hesapla(nesne);
                if let Deger::Nesne { alanlar, .. } = nesne_val {
                    alanlar.borrow_mut().insert(ozellik, deger_val);
                }
            }
            Komut::IfadeKomutu(ifade) => {
                if let Ifade::IkiliIslem { sol, operator: Token::Esittir, sag } = ifade {
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
                                (Deger::Liste(l), Deger::Sayi(i)) => {
                                    let idx = i as usize;
                                    let mut b = l.borrow_mut();
                                    if idx < b.len() {
                                        b[idx] = d;
                                    }
                                }
                                (Deger::Nesne { alanlar, .. }, Deger::Metin(key)) => {
                                    alanlar.borrow_mut().insert(key, d);
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                } else { self.ifade_hesapla(ifade); }
            }
        }
    }

    fn modül_yükle(&mut self, dosya_adı: &str) {
        // Önce gömülü kütüphaneleri kontrol et
        for (ad, icerik) in builtin_files::get_lib_files() {
            if ad == dosya_adı {
                if self.yuklenen_dosyalar.contains(ad) { return; }
                self.yuklenen_dosyalar.insert(ad.to_string());
                let mut parser = crate::parser::Parser::new(crate::lexer::Lexer::new(icerik));
                let prog = parser.parse_program();
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
            if path.is_file() { bulundu = Some(tam_yol); break; }
            
            // Paket yöneticisi için destek: modul/modul.hb pattern'ini kontrol et
            let paket_yol = format!("{}/{}/{}.hb", temel, dosya_adı, dosya_adı);
            if Path::new(&paket_yol).is_file() { bulundu = Some(paket_yol); break; }

            // Uzantı ekleyerek kontrol et
            if !dosya_adı.ends_with(".hb") {
                let hb_yol = format!("{}.hb", tam_yol);
                if Path::new(&hb_yol).is_file() { bulundu = Some(hb_yol); break; }
            }
        }


        if let Some(yol) = bulundu {
            if self.yuklenen_dosyalar.contains(&yol) { return; }
            self.yuklenen_dosyalar.insert(yol.clone());
            if let Ok(icerik) = std::fs::read_to_string(&yol) {
                let mut parser = crate::parser::Parser::new(crate::lexer::Lexer::new(&icerik));
                let prog = parser.parse_program();
                let eski = self.donus_degeri.take();
                self.yorumla(prog);
                self.donus_degeri = eski;
            }
        } else {
            eprintln!("[Hüma Hatası] Modül bulunamadı: {}", dosya_adı);
        }
    }

    fn ifade_hesapla(&mut self, ifade: Ifade) -> Deger {
        match ifade {
            Ifade::Sayi(n) => Deger::Sayi(n),
            Ifade::Metin(s) => Deger::Metin(s),
            Ifade::Bos => Deger::Bos,
            Ifade::Dogru => Deger::Sayi(1.0),
            Ifade::Yanlis => Deger::Sayi(0.0),
            Ifade::Degisken(ad) => self.get_degisken(&ad),
            Ifade::Liste(el) => Deger::Liste(Rc::new(RefCell::new(el.into_iter().map(|e| self.ifade_hesapla(e)).collect()))),
            Ifade::ListeErisim { liste, indeks } => {
                let l_val = self.ifade_hesapla(*liste);
                let i_val = self.ifade_hesapla(*indeks);
                match (l_val, i_val) {
                    (Deger::Liste(l), Deger::Sayi(i)) => l.borrow().get(i as usize).cloned().unwrap_or(Deger::Bos),
                    (Deger::Metin(s), Deger::Sayi(i)) => s.chars().nth(i as usize).map(|c| Deger::Metin(c.to_string())).unwrap_or(Deger::Bos),
                    (Deger::Nesne { alanlar, .. }, Deger::Metin(key)) => alanlar.borrow().get(&key).cloned().unwrap_or(Deger::Bos),
                    _ => Deger::Bos
                }
            }
            Ifade::NesneErisim { nesne, ozellik } => {
                let inst = self.ifade_hesapla(*nesne);
                if let Deger::Nesne { alanlar, .. } = inst { alanlar.borrow().get(&ozellik).cloned().unwrap_or(Deger::Bos) }
                else { Deger::Bos }
            }
            Ifade::KendisiErisim { ozellik } => {
                let kendisi = self.get_degisken("kendisi");
                if let Deger::Nesne { alanlar, .. } = kendisi {
                    alanlar.borrow().get(&ozellik).cloned().unwrap_or(Deger::Bos)
                } else {
                    Deger::Bos
                }
            }
            Ifade::Uzunluk(ifade) => {
                let val = self.ifade_hesapla(*ifade);
                match val {
                    Deger::Liste(l) => Deger::Sayi(l.borrow().len() as f64),
                    Deger::Metin(s) => Deger::Sayi(s.chars().count() as f64),
                    _ => Deger::Sayi(0.0),
                }
            }
            Ifade::FonksiyonIfadesi { parametreler, govde } => Deger::Fonksiyon { parametreler, govde },
            Ifade::Cagri { fonksiyon, argumanlar } => {
                let mut method_instance = None;
                let f = if let Ifade::NesneErisim { nesne, ozellik } = *fonksiyon.clone() {
                    let instance = self.ifade_hesapla(*nesne);
                    if let Deger::Nesne { ref sinif_adi, .. } = instance {
                        if let Some(Deger::Sinif { metotlar, .. }) = self.global_degiskenler.get(sinif_adi) {
                            if let Some((ps, bd)) = metotlar.get(&ozellik) {
                                method_instance = Some(instance.clone());
                                Deger::Fonksiyon { parametreler: ps.clone(), govde: bd.clone() }
                            } else { self.ifade_hesapla(*fonksiyon) }
                        } else { self.ifade_hesapla(*fonksiyon) }
                    } else { self.ifade_hesapla(*fonksiyon) }
                } else { self.ifade_hesapla(*fonksiyon) };

                let args = argumanlar.into_iter().map(|a| self.ifade_hesapla(a)).collect();
                self.fonksiyon_cagrisi_detayli(f, args, method_instance)
            }
            Ifade::IkiliIslem { sol, operator, sag } => {
                let mut l = self.ifade_hesapla(*sol);
                let mut r = self.ifade_hesapla(*sag);
                
                // Tip zorlama (Coercion) - Arti hariç diğer sayısal işlemlerde zorla
                if matches!(operator, Token::Eksi | Token::Carpi | Token::Bolnu | Token::Mod | Token::Kucuktur | Token::Buyuktur | Token::KucukEsit | Token::BuyukEsit) {
                    if let Deger::Metin(ref s) = l { if let Ok(n) = s.parse::<f64>() { l = Deger::Sayi(n); } }
                    if let Deger::Metin(ref s) = r { if let Ok(n) = s.parse::<f64>() { r = Deger::Sayi(n); } }
                }

                match operator {
                    Token::Ve => Deger::Sayi(if self.dogruluk_kontrolu(l.clone()) && self.dogruluk_kontrolu(r.clone()) { 1.0 } else { 0.0 }),
                    Token::Veya => Deger::Sayi(if self.dogruluk_kontrolu(l.clone()) || self.dogruluk_kontrolu(r.clone()) { 1.0 } else { 0.0 }),
                    Token::EsitEsittir | Token::Esittir => Deger::Sayi(if l == r { 1.0 } else { 0.0 }),
                    Token::EsitDegil => Deger::Sayi(if l != r { 1.0 } else { 0.0 }),
                    _ => match (l, r) {
                        (Deger::Sayi(a), Deger::Sayi(b)) => match operator {
                            Token::Arti => Deger::Sayi(a + b),
                            Token::Eksi => Deger::Sayi(a - b),
                            Token::Carpi => Deger::Sayi(a * b),
                            Token::Bolnu => Deger::Sayi(a / b),
                            Token::Mod => Deger::Sayi(a % b),
                            Token::Kucuktur => Deger::Sayi(if a < b { 1.0 } else { 0.0 }),
                            Token::Buyuktur => Deger::Sayi(if a > b { 1.0 } else { 0.0 }),
                            Token::KucukEsit => Deger::Sayi(if a <= b { 1.0 } else { 0.0 }),
                            Token::BuyukEsit => Deger::Sayi(if a >= b { 1.0 } else { 0.0 }),
                            _ => Deger::Bos
                        },
                        (l_val, r_val) => match operator {
                            Token::Arti => Deger::Metin(format!("{}{}", l_val, r_val)),
                            Token::Kucuktur => Deger::Sayi(if l_val.to_string() < r_val.to_string() { 1.0 } else { 0.0 }),
                            Token::Buyuktur => Deger::Sayi(if l_val.to_string() > r_val.to_string() { 1.0 } else { 0.0 }),
                            _ => Deger::Bos
                        }
                    }
                }
            }
            Ifade::MantıksalDegil(i) => {
                let v = self.ifade_hesapla(*i);
                Deger::Sayi(if self.dogruluk_kontrolu(v) { 0.0 } else { 1.0 })
            }
            _ => Deger::Bos
        }
    }

    fn dogruluk_kontrolu(&self, deger: Deger) -> bool {
        match deger {
            Deger::Sayi(n) => n != 0.0,
            Deger::Metin(s) => !s.is_empty(),
            Deger::Liste(l) => !l.borrow().is_empty(),
            Deger::Bos => false,
            _ => true
        }
    }
}
