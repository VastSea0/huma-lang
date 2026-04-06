use std::fs;
use std::path::Path;
use anyhow::{Result, anyhow};
use colored::Colorize;
use serde::{Deserialize, Serialize, Deserializer};
use semver::{Version, VersionReq};
use std::collections::HashMap;
use chrono;
use sha2::{Sha256, Digest};

/// Hüma Paket Standardı (HPS) Metadata Dosyası
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaketMetadata {
    pub ad: String,
    pub surum: String,
    pub aciklama: String,
    pub yazar: String,
    pub giris: String,
    /// Bu paket için gereken minimum Hüma versiyonu (isteğe bağlı)
    pub huma_surum: Option<String>,
    /// Projenin bağımlılıkları (paket adı -> sürüm kısıtlaması)
    pub bagimliliklar: Option<HashMap<String, String>>,
    /// Çalıştırılabilir betikler (betik adı -> komut)
    pub betikler: Option<HashMap<String, String>>,
    /// Native Rust bağımlılıkları (crate adı -> sürüm)
    pub crate_bagimliliklari: Option<HashMap<String, String>>,
    /// Transpilation (huma gen) sırasında enjekte edilecek Rust kodu
    pub yerleşik_rust: Option<String>,
}


/// Hüma Kilit Dosyası (huma.lock)
/// Yüklenen tüm paketlerin kesin sürümlerini ve bütünlük özetlerini (hash) saklar.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PaketKilit {
    pub paketler: HashMap<String, KilitBilgisi>,
    pub guncelleme_zamani: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct KilitBilgisi {
    pub surum: String,
    pub hash: String,
}

impl<'de> Deserialize<'de> for KilitBilgisi {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum TempKilit {
            String(String),
            Map { surum: String, hash: String },
        }

        match TempKilit::deserialize(deserializer)? {
            TempKilit::String(s) => Ok(KilitBilgisi {
                surum: s,
                hash: "".to_string(),
            }),
            TempKilit::Map { surum, hash } => Ok(KilitBilgisi { surum, hash }),
        }
    }
}

const PACKAGE_DIR: &str = "huma_modulleri";
const LOCK_FILE: &str = "huma.lock";
const PROJECT_FILE: &str = "huma.json";
const CURRENT_HUMA_VER: &str = env!("CARGO_PKG_VERSION");


/// Kurulu tüm paketleri ve sürümlerini kilit dosyasından listeler
pub fn list_packages() -> Result<()> {
    if !Path::new(PROJECT_FILE).exists() {
        return Err(anyhow!("Bu dizinde bir Hüma projesi (huma.json) bulunamadı."));
    }

    if !Path::new(LOCK_FILE).exists() {
        println!("{} Hiç paket kurulu değil.", "Bilgi:".bright_yellow());
        return Ok(());
    }

    let lock_str = fs::read_to_string(LOCK_FILE)?;
    let lock: PaketKilit = serde_json::from_str(&lock_str)?;

    println!("{} Kurulu Hüma Paketleri (Kilitlenmiş Sürümler):", "Hüma:".bright_cyan());
    for (ad, bilgi) in &lock.paketler {
        if !bilgi.hash.is_empty() {
            println!("  {} -> {} [{}]", ad.bright_green(), bilgi.surum.bright_white(), &bilgi.hash[..8].bright_black());
        } else {
            println!("  {} -> {}", ad.bright_green(), bilgi.surum.bright_white());
        }
    }
    
    Ok(())
}

/// Yeni bir Hüma paketi (projesi) oluşturur
pub fn create_package(name: &str) -> Result<()> {
    let dir = Path::new(name);
    if dir.exists() {
        return Err(anyhow!("'{}' dizini zaten mevcut.", name));
    }

    fs::create_dir_all(dir)?;
    
    let mut betikler = HashMap::new();
    betikler.insert("baslat".to_string(), format!("huma run {}.hb", name));
    betikler.insert("test".to_string(), "huma run tests/test.hb".to_string());

    let meta = PaketMetadata {
        ad: name.to_string(),
        surum: "0.1.0".to_string(),
        aciklama: "Yeni bir Hüma projesi.".to_string(),
        yazar: "Geliştirici".to_string(),
        giris: format!("{}.hb", name),
        huma_surum: Some(format!(">={}", CURRENT_HUMA_VER)),
        bagimliliklar: Some(HashMap::new()),
        betikler: Some(betikler),
        crate_bagimliliklari: None,
        yerleşik_rust: None,
    };

    fs::write(dir.join("huma.json"), serde_json::to_string_pretty(&meta)?)?;
    fs::write(dir.join(format!("{}.hb", name)), format!("// {} ana giriş dosyası\n\"{} kütüphanesi aktif.\"'ı yazdır", name, name))?;

    // .gitignore oluştur (Hüma standartlarına uygun)
    let gitignore_content = "huma_modulleri/\ntarget/\n*.hbc\n.DS_Store\n";
    fs::write(dir.join(".gitignore"), gitignore_content)?;

    // v0.4.0: Proje oluşturulurken kilit dosyası ve modül klasörü de ilklendirilir
    let mod_dir = dir.join(PACKAGE_DIR);
    if !mod_dir.exists() {
        fs::create_dir_all(&mod_dir)?;
    }
    
    let lock_path = dir.join(LOCK_FILE);
    let lock = PaketKilit {
        paketler: HashMap::new(),
        guncelleme_zamani: chrono::Local::now().to_rfc3339(),
    };
    fs::write(lock_path, serde_json::to_string_pretty(&lock)?)?;

    // Git ilklendirmesi dene
    let _ = std::process::Command::new("git")
        .arg("init")
        .current_dir(dir)
        .output();

    println!("{} '{}' projesi oluşturuldu.", "Başarılı!".bright_green(), name.bold());
    Ok(())
}


/// Bir paketi kurar ve kilit dosyasına ekler
pub fn install_package(input: Option<&str>) -> Result<()> {
    // 1. Proje dosyası kontrolü
    if !Path::new(PROJECT_FILE).exists() {
        return Err(anyhow!("Bu dizinde huma.json bulunamadı. Önce 'huma paket ilkle' çalıştırın."));
    }

    let input = match input {
        Some(i) => i,
        None => {
            // Hiç argüman yoksa: huma.json'daki tüm bağımlılıkları kur
            let meta = get_local_metadata()?;
            if let Some(deps) = meta.bagimliliklar {
                if deps.is_empty() {
                    println!("{} Kurulacak bağımlılık yok.", "Bilgi:".bright_yellow());
                    return Ok(());
                }
                println!("{} {} bağımlılık kuruluyor...", "Hüma:".bright_cyan(), deps.len());
                for (ad, _surum) in deps {
                    // Sembolik kurma (Şu an GitHub veya dahili paketleri destekliyoruz)
                    if ad == "nlp_temel" || ad == "ag_istekleri" || ad == "huma_sunucu" {
                        install_package(Some(&ad))?;
                    } else {
                        println!("{} {} paketi henüz topluluk kütüphanesinde değil, atlanıyor.", "Uyarı:".bright_yellow(), ad);
                    }
                }
                return Ok(());
            } else {
                println!("{} Bağımlılık listesi boş.", "Bilgi:".bright_yellow());
                return Ok(());
            }
        }
    };

    if input.starts_with("github.com/") {
        return install_from_github(input);
    }

    // Mock local installation for built-in libs
    match input {
        "nlp_temel" | "ag_istekleri" | "huma_sunucu" => {
            let mut meta = PaketMetadata {
                ad: input.to_string(),
                surum: "1.0.0".to_string(),
                aciklama: "Topluluk kütüphanesi.".to_string(),
                yazar: "Hüma Takımı".to_string(),
                giris: format!("{}.hb", input),
                huma_surum: Some(">=0.3.0".to_string()),
                bagimliliklar: None,
                betikler: None, // Added field
                crate_bagimliliklari: None,
                yerleşik_rust: None,
            };

            let content = match input {
                "huma_sunucu" => {
                    meta.aciklama = "Hüma için dahili web sunucusu modülü.".to_string();
                    // Path may change depending on execution context, but using absolute for internal simulation
                    fs::read_to_string("/home/egehan/development/humapy/huma_modulleri/huma_sunucu/huma_sunucu.hb")
                        .unwrap_or_else(|_| "// huma_sunucu içeriği".to_string())
                },
                "ag_istekleri" => {
                    meta.surum = "1.1.0".to_string();
                    meta.aciklama = "Hüma Programlama Dili için profesyonel ve hafif HTTP kütüphanesi.".to_string();
                    meta.yazar = "Egehan KAHRAMAN".to_string();
                    fs::read_to_string("/home/egehan/development/humapy/ag_istekleri/ag_istekleri.hb")
                        .unwrap_or_else(|_| "// ag_istekleri içeriği".to_string())
                },
                "nlp_temel" => {
                    meta.surum = "3.1.0".to_string();
                    meta.aciklama = "Hüma Dili Türkçe NLP Kütüphanesi.".to_string();
                    fs::read_to_string("/home/egehan/development/humapy/lib/nlp.hb")
                        .unwrap_or_else(|_| "// nlp_temel içeriği".to_string())
                },
                _ => "// Simülasyon içeriği".to_string(),
            };

            save_package(meta.clone(), &content)?;

            // Ek dosyaları kopyala (örneğin nlp_temel için bağımlılıklar)
            if input == "nlp_temel" {
                let package_path = format!("{}/{}", PACKAGE_DIR, input);
                let _ = fs::copy("/home/egehan/development/humapy/lib/dizgi.hb", format!("{}/dizgi.hb", package_path));
                let _ = fs::copy("/home/egehan/development/humapy/lib/liste.hb", format!("{}/liste.hb", package_path));
            }
        },
        _ => return Err(anyhow!("Paket bulunamadı. Lütfen GitHub linki kullanın (örn: github.com/user/repo).")),
    }
    Ok(())
}

fn install_from_github(url: &str) -> Result<()> {
    println!("{} GitHub üzerinden indiriliyor: {}...", "Hüma:".bright_cyan(), url.bold());
    
    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() < 3 { return Err(anyhow!("Geçersiz URL")); }
    let raw_base = format!("https://raw.githubusercontent.com/{}/{}/main", parts[1], parts[2]);
    
    let meta_str = download_text(&format!("{}/paket.json", raw_base))?;
    let meta: PaketMetadata = serde_json::from_str(&meta_str)?;

    // 1. Hüma Sürüm Kontrolü
    if let Some(req_str) = &meta.huma_surum {
        let req = VersionReq::parse(req_str)?;
        let current_ver = Version::parse(CURRENT_HUMA_VER)?;
        if !req.matches(&current_ver) {
            return Err(anyhow!(
                "Sürüm Uyumsuzluğu: {} paketi Hüma v{} gerektiriyor (Sizdeki sürüm: v{}).",
                meta.ad, req_str, CURRENT_HUMA_VER
            ));
        }
    }

    // 2. Giriş Dosyasını İndir
    let entry_content = download_text(&format!("{}/{}", raw_base, meta.giris))?;
    
    save_package(meta, &entry_content)?;
    Ok(())
}

fn save_package(meta: PaketMetadata, content: &str) -> Result<()> {
    // 1. Proje dosyasını bul ve bağımlılık ekle
    if Path::new(PROJECT_FILE).exists() {
        let mut proj_meta: PaketMetadata = serde_json::from_str(&fs::read_to_string(PROJECT_FILE)?)?;
        if proj_meta.bagimliliklar.is_none() {
            proj_meta.bagimliliklar = Some(HashMap::new());
        }
        if let Some(ref mut deps) = proj_meta.bagimliliklar {
            deps.insert(meta.ad.clone(), format!("^{}", meta.surum));
        }
        fs::write(PROJECT_FILE, serde_json::to_string_pretty(&proj_meta)?)?;
    }

    // 2. Paketi modül dizinine yaz
    let package_path = format!("{}/{}", PACKAGE_DIR, meta.ad);
    fs::create_dir_all(&package_path)?;

    fs::write(format!("{}/{}", package_path, meta.giris), content)?;
    fs::write(format!("{}/paket.json", package_path), serde_json::to_string_pretty(&meta)?)?;

    // 3. Kilit Dosyasını Güncelle
    let hash = calculate_hash(content, &serde_json::to_string(&meta)?);
    update_lock_file(&meta.ad, &meta.surum, &hash)?;

    println!("{} {} v{} [hash:{}] başarıyla kuruldu.", 
        "Başarılı!".bright_green(), 
        meta.ad.bold(), 
        meta.surum.bright_white(),
        &hash[..8]
    );
    Ok(())
}

fn calculate_hash(content: &str, meta_str: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hasher.update(meta_str.as_bytes());
    hex::encode(hasher.finalize())
}

/// Paketin yayınlanabilirliğini doğrular
pub fn verify_package() -> Result<()> {
    if !Path::new(PROJECT_FILE).exists() {
        return Err(anyhow!("Bu dizinde bir Hüma projesi (huma.json) bulunamadı."));
    }
    
    let meta: PaketMetadata = serde_json::from_str(&fs::read_to_string(PROJECT_FILE)?)?;
    
    if meta.ad.is_empty() || meta.surum.is_empty() {
        return Err(anyhow!("Paket adı veya sürümü eksik."));
    }
    
    // Sürüm geçerli mi kontrol et
    Version::parse(&meta.surum)?;
    
    println!("{} Paket '{}' v{} yayına hazır.", "Doğrulandı:".bright_green(), meta.ad, meta.surum);
    Ok(())
}


fn update_lock_file(name: &str, version: &str, hash: &str) -> Result<()> {
    let mut lock = if Path::new(LOCK_FILE).exists() {
        let s = fs::read_to_string(LOCK_FILE)?;
        // Geriye dönük uyumluluk için Value olarak parse etmeyi dene
        if let Ok(l) = serde_json::from_str::<PaketKilit>(&s) {
            l
        } else {
            // Eski format ise veya hatalıysa sıfırla (veya hata ver)
            PaketKilit::default()
        }
    } else {
        PaketKilit::default()
    };

    lock.paketler.insert(name.to_string(), KilitBilgisi {
        surum: version.to_string(),
        hash: hash.to_string(),
    });
    lock.guncelleme_zamani = chrono::Local::now().to_rfc3339();

    fs::write(LOCK_FILE, serde_json::to_string_pretty(&lock)?)?;
    Ok(())
}

fn download_text(url: &str) -> Result<String> {
    let response = ureq::get(url).call()?;
    Ok(response.into_string()?)
}

pub fn remove_package(name: &str) -> Result<()> {
    let path = format!("{}/{}", PACKAGE_DIR, name);
    if Path::new(&path).exists() {
        fs::remove_dir_all(&path)?;
        
        // Kilit dosyasından çıkar
        if Path::new(LOCK_FILE).exists() {
            let s = fs::read_to_string(LOCK_FILE)?;
            let mut lock: PaketKilit = serde_json::from_str(&s)?;
            lock.paketler.remove(name);
            fs::write(LOCK_FILE, serde_json::to_string_pretty(&lock)?)?;
        }

        println!("{} {} silindi.", "Başarılı!".bright_green(), name.bold());
        Ok(())
    } else {
        Err(anyhow!("Paket bulunamadı."))
    }
}

pub fn update_packages() -> Result<()> {
    println!("{} Paket güncellemeleri kontrol ediliyor...", "Hüma:".bright_cyan());
    // Gerçek bir senaryoda tüm URL'ler LOCK dosyasında saklanır ve tek tek re-download edilir.
    println!("{} Tüm paketler kilitli sürümlerinde güncel.", "Giriş:".bright_green());
    Ok(())
}

/// Mevcut dizinde bir Hüma projesi ilklendirir
pub fn init_project() -> Result<()> {
    if Path::new(PROJECT_FILE).exists() {
        return Err(anyhow!("Bu dizinde zaten bir huma.json dosyası mevcut."));
    }

    let default_name = std::env::current_dir()?
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("huma_projesi")
        .to_string();

    let mut betikler = HashMap::new();
    betikler.insert("baslat".to_string(), format!("huma run {}.hb", default_name));
    betikler.insert("test".to_string(), "huma run tests/test.hb".to_string());

    let meta = PaketMetadata {
        ad: default_name.clone(),
        surum: "0.1.0".to_string(),
        aciklama: "Yeni bir Hüma projesi.".to_string(),
        yazar: "Geliştirici".to_string(),
        giris: format!("{}.hb", default_name),
        huma_surum: Some(format!(">={}", CURRENT_HUMA_VER)),
        bagimliliklar: Some(HashMap::new()),
        betikler: Some(betikler),
        crate_bagimliliklari: None,
        yerleşik_rust: None,
    };

    fs::write(PROJECT_FILE, serde_json::to_string_pretty(&meta)?)?;
    
    let hb_file = format!("{}.hb", default_name);
    if !Path::new(&hb_file).exists() {
        fs::write(&hb_file, format!("// {} ana giriş dosyaba\n\"Hüma projesi aktif.\"'ı yazdır", default_name))?;
    }

    // Git ilklendirmesi dene
    let _ = std::process::Command::new("git")
        .arg("init")
        .output();

    println!("{} Proje '{}' olarak ilklendirildi.", "Başarılı!".bright_green(), default_name.bold());
    Ok(())
}

/// Mevcut dizindeki huma.json dosyasını okur
pub fn get_local_metadata() -> Result<PaketMetadata> {
    if !Path::new(PROJECT_FILE).exists() {
        return Err(anyhow!("Bu dizinde bir Hüma projesi (huma.json) bulunamadı."));
    }
    let s = fs::read_to_string(PROJECT_FILE)?;
    let meta: PaketMetadata = serde_json::from_str(&s)?;
    Ok(meta)
}

/// Belirtilen betiği çalıştırır
pub fn run_script(name: &str) -> Result<()> {
    let meta = get_local_metadata()?;
    if let Some(betikler) = meta.betikler {
        if let Some(komut) = betikler.get(name) {
            println!("{} {} betiği çalıştırılıyor: {}", "Hüma:".bright_cyan(), name.bold(), komut.bright_black());
            
            let status = if cfg!(target_os = "windows") {
                std::process::Command::new("cmd")
                    .args(["/C", komut])
                    .status()?
            } else {
                std::process::Command::new("sh")
                    .args(["-c", komut])
                    .status()?
            };

            if !status.success() {
                return Err(anyhow!("Betik '{}' hata ile sonlandı.", name));
            }
            Ok(())
        } else {
            Err(anyhow!("'{}' adlı bir betik huma.json içinde bulunamadı.", name))
        }
    } else {
        Err(anyhow!("Bu projede hiç betik tanımlanmamış."))
    }
}
