use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use semver::{Version, VersionReq};
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

// ─── Hüma Paket Standardı (HPS) ────────────────────────────────────────────

/// Hüma Paket Standardı (HPS) Metadata Dosyası
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaketMetadata {
    pub ad: String,
    pub surum: String,
    pub aciklama: String,
    pub yazar: String,
    pub giris: String,
    /// Bu paket için gereken minimum Hüma versiyonu (isteğe bağlı)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub huma_surum: Option<String>,
    /// Projenin bağımlılıkları (paket adı -> sürüm kısıtlaması)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bagimliliklar: Option<HashMap<String, String>>,
    /// Çalıştırılabilir betikler (betik adı -> komut)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub betikler: Option<HashMap<String, String>>,
    /// Native Rust bağımlılıkları (crate adı -> sürüm)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crate_bagimliliklari: Option<HashMap<String, String>>,
    /// Gelecekteki sürümlü native paket ABI'si için ayrılmış alan.
    /// Hüma 0.6 bu kodu derlemez veya çalıştırmaz.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yerleşik_rust: Option<String>,
    /// Paketin GitHub kaynak URL'si (lock dosyasında izlenebilmesi için)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kaynak: Option<String>,
    /// GitHub kullanıcı/repo bilgisi (örn: "VastSea0/ag_istekleri")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github: Option<String>,
    /// Lisans türü (örn: "MIT")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lisans: Option<String>,
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
    /// Paketin kaynağı (GitHub URL, builtin, vb.)
    #[serde(default)]
    pub kaynak: Option<String>,
}

impl<'de> Deserialize<'de> for KilitBilgisi {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum TempKilit {
            String(String),
            Map {
                surum: String,
                hash: String,
                #[serde(default)]
                kaynak: Option<String>,
            },
        }

        match TempKilit::deserialize(deserializer)? {
            TempKilit::String(s) => Ok(KilitBilgisi {
                surum: s,
                hash: "".to_string(),
                kaynak: None,
            }),
            TempKilit::Map {
                surum,
                hash,
                kaynak,
            } => Ok(KilitBilgisi {
                surum,
                hash,
                kaynak,
            }),
        }
    }
}

// ─── Sabitler ───────────────────────────────────────────────────────────────

const PACKAGE_DIR: &str = "huma_modulleri";
const LOCK_FILE: &str = "huma.lock";
const PROJECT_FILE: &str = "huma.json";
const CURRENT_HUMA_VER: &str = env!("CARGO_PKG_VERSION");

/// Kaynak ağacında dağıtılan ve yerel olarak kurulabilen paketler.
const BUILTIN_PACKAGES: &[&str] = &[
    "nlp_temel",
    "nlp_ileri",
    "yapay_zeka",
    "ag_istekleri",
    "huma_sunucu",
    "huma_sqlite",
    "gui",
];

/// Paket adları ve dosya yollarında izin verilmeyen kalıplar
const FORBIDDEN_PATH_PATTERNS: &[&str] = &["..", "//", "\\", "\0", "~"];

/// Betiklerde tehlikeli kabuk meta-karakterleri
const DANGEROUS_SHELL_CHARS: &[&str] = &["&&", "||", ";", "|", "$(", "`", ">", "<", "&"];

// ─── [1] Path Sanitization ─────────────────────────────────────────────────

/// Paket adını ve dosya yollarını path traversal saldırılarına karşı doğrular.
///
/// Kötü niyetli bir paket `ad: "../../../etc"` veya `giris: "../../passwd"`
/// kullanarak sistemin kritik dosyalarına yazma yapabilir. Bu fonksiyon
/// tüm tehlikeli kalıpları reddeder.
fn sanitize_package_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow!("Paket adı boş olamaz."));
    }

    if name.len() > 128 {
        return Err(anyhow!("Paket adı 128 karakterden uzun olamaz."));
    }

    for pattern in FORBIDDEN_PATH_PATTERNS {
        if name.contains(pattern) {
            return Err(anyhow!(
                "Güvenlik Hatası: Paket adı '{}' tehlikeli karakter/kalıp içeriyor: '{}'",
                name,
                pattern
            ));
        }
    }

    // Sadece alfanümerik, alt çizgi, tire ve nokta (dosya uzantısı) karakterlerine izin ver
    let is_valid = name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.');

    if !is_valid {
        return Err(anyhow!(
            "Güvenlik Hatası: Paket adı '{}' geçersiz karakterler içeriyor. \
             Sadece harf, rakam, alt çizgi (_), tire (-) ve nokta (.) kullanılabilir.",
            name
        ));
    }

    Ok(())
}

fn uzak_paket_girdisi_mi(input: &str) -> bool {
    input.contains('/') || input.starts_with("http:") || input.starts_with("https:")
}

/// Yazılacak dosyanın hedef dizin (huma_modulleri/) dışına çıkmadığını doğrular.
fn verify_path_within_boundary(file_path: &Path, boundary_dir: &Path) -> Result<()> {
    // Boundary'yi oluştur (henüz yoksa)
    fs::create_dir_all(boundary_dir)?;

    let canonical_boundary = boundary_dir.canonicalize().with_context(|| {
        format!(
            "Hedef dizin canonicalize edilemedi: {}",
            boundary_dir.display()
        )
    })?;

    // Dosyanın üst dizinini canonicalize et (dosya henüz yoksa üst dizini kullan)
    let parent = file_path.parent().unwrap_or(file_path);
    fs::create_dir_all(parent)?;
    let canonical_parent = parent
        .canonicalize()
        .with_context(|| format!("Dosya yolu canonicalize edilemedi: {}", parent.display()))?;

    if !canonical_parent.starts_with(&canonical_boundary) {
        return Err(anyhow!(
            "Güvenlik Hatası: Dosya yolu '{}' izin verilen sınırın ({}) dışına çıkıyor!",
            file_path.display(),
            canonical_boundary.display()
        ));
    }

    Ok(())
}

// ─── [2] Native Kod Uyarısı ────────────────────────────────────────────────

/// Paketin native (Rust) kodu içerip içermediğini kontrol eder ve
/// güvenilir modda değilse kullanıcıyı uyarır.
fn check_native_code_safety(meta: &PaketMetadata, trusted: bool) -> Result<()> {
    let has_native_rust = meta.yerleşik_rust.as_ref().is_some_and(|s| !s.is_empty());
    let has_crate_deps = meta
        .crate_bagimliliklari
        .as_ref()
        .is_some_and(|d| !d.is_empty());

    if !has_native_rust && !has_crate_deps {
        return Ok(());
    }

    println!();
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════╗".bright_red()
    );
    println!(
        "{}",
        "║  ⚠  GÜVENLİK UYARISI — NATIVE KOD TESPİT EDİLDİ  ⚠   ║"
            .bright_red()
            .bold()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════╝".bright_red()
    );

    if has_native_rust {
        println!(
            "  {} '{}' paketi {} içermektedir.",
            "↳".bright_yellow(),
            meta.ad.bold(),
            "yerleşik Rust kodu".bright_red().bold()
        );
        println!(
            "  {} Bu kod, derleme sırasında sisteminizde tam yetkiyle çalışabilir.",
            "↳".bright_yellow(),
        );
    }

    if has_crate_deps {
        let deps: Vec<String> = meta
            .crate_bagimliliklari
            .as_ref()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        println!(
            "  {} Harici Rust crate bağımlılıkları: {}",
            "↳".bright_yellow(),
            deps.join(", ").bright_white()
        );
    }

    println!();

    if trusted {
        println!(
            "  {} Güvenilir mod aktif (--güvenilir). Kuruluma devam ediliyor.",
            "✓".bright_green()
        );
        return Ok(());
    }

    // İnteraktif onay iste
    print!(
        "  {} Bu paketi kurmak istediğinize emin misiniz? [e/H]: ",
        "?".bright_cyan().bold()
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let answer = input.trim().to_lowercase();

    if answer == "e" || answer == "evet" {
        Ok(())
    } else {
        Err(anyhow!("Kurulum kullanıcı tarafından iptal edildi."))
    }
}

// ─── [3] Hash Doğrulaması ──────────────────────────────────────────────────

/// Paket metadata'sı ve bütün dosyalarından tekrarlanabilir SHA-256 özeti hesaplar.
fn calculate_package_hash(package_path: &Path, meta: &PaketMetadata) -> Result<String> {
    fn collect_files(current: &Path, root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                return Err(anyhow!(
                    "Paket bütünlüğü hesaplanırken sembolik bağlantı reddedildi: {}",
                    path.display()
                ));
            }
            if file_type.is_dir() {
                collect_files(&path, root, files)?;
            } else if path.parent() != Some(root)
                || !matches!(
                    path.file_name().and_then(|name| name.to_str()),
                    Some("huma.json" | "paket.json")
                )
            {
                files.push(path);
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    collect_files(package_path, package_path, &mut files)?;
    files.sort_by(|a, b| {
        a.strip_prefix(package_path)
            .unwrap_or(a)
            .cmp(b.strip_prefix(package_path).unwrap_or(b))
    });

    let mut hasher = Sha256::new();
    let meta_bytes = serde_json::to_vec(meta)?;
    hasher.update((meta_bytes.len() as u64).to_le_bytes());
    hasher.update(meta_bytes);
    for path in files {
        let relative = path
            .strip_prefix(package_path)
            .with_context(|| format!("Paket yolu göreli hale getirilemedi: {}", path.display()))?;
        let relative_bytes = relative.to_string_lossy();
        let content = fs::read(&path)?;
        hasher.update((relative_bytes.len() as u64).to_le_bytes());
        hasher.update(relative_bytes.as_bytes());
        hasher.update((content.len() as u64).to_le_bytes());
        hasher.update(content);
    }
    Ok(hex::encode(hasher.finalize()))
}

// ─── [5] Atomik Dosya Yazımı ───────────────────────────────────────────────

/// Dosyayı atomik olarak yazar: önce `.tmp` uzantılı geçici dosyaya yaz,
/// sonra `rename()` ile hedef konuma taşı. Bu sayede yarım kalan yazımlarda
/// (güç kesintisi, çökme) dosya bozulması engellenir.
fn atomic_write(path: &Path, content: &[u8]) -> Result<()> {
    let tmp_path = path.with_extension("tmp");

    // Üst dizini oluştur
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Geçici dosyaya yaz
    fs::write(&tmp_path, content)
        .with_context(|| format!("Geçici dosya yazılamadı: {}", tmp_path.display()))?;

    // Atomik taşıma
    fs::rename(&tmp_path, path).with_context(|| {
        format!(
            "Dosya taşıma başarısız: {} → {}",
            tmp_path.display(),
            path.display()
        )
    })?;

    Ok(())
}

/// String içeriği atomik olarak yazar.
fn atomic_write_str(path: &Path, content: &str) -> Result<()> {
    atomic_write(path, content.as_bytes())
}

// ─── [9] Betik Güvenliği ───────────────────────────────────────────────────

/// Betik komutunda tehlikeli kabuk meta-karakterlerini kontrol eder.
fn check_script_safety(command: &str) -> Result<()> {
    let mut detected = Vec::new();

    for pattern in DANGEROUS_SHELL_CHARS {
        if command.contains(pattern) {
            detected.push(*pattern);
        }
    }

    if !detected.is_empty() {
        println!(
            "\n  {} Betik komutu potansiyel tehlikeli kabuk meta-karakterleri içeriyor:",
            "⚠ UYARI:".bright_yellow().bold()
        );
        println!("    Komut: {}", command.bright_white());
        println!(
            "    Tespit: {}",
            detected
                .iter()
                .map(|s| format!("'{}'", s))
                .collect::<Vec<_>>()
                .join(", ")
                .bright_red()
        );
        print!(
            "  {} Yine de çalıştırmak istiyor musunuz? [e/H]: ",
            "?".bright_cyan().bold()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let answer = input.trim().to_lowercase();

        if answer != "e" && answer != "evet" {
            return Err(anyhow!(
                "Betik çalıştırma kullanıcı tarafından iptal edildi."
            ));
        }
    }

    Ok(())
}

// ─── Public API ─────────────────────────────────────────────────────────────

/// Kurulu tüm paketleri ve sürümlerini kilit dosyasından listeler
pub fn list_packages() -> Result<()> {
    if !Path::new(PROJECT_FILE).exists() {
        return Err(anyhow!(
            "Bu dizinde bir Hüma projesi (huma.json) bulunamadı."
        ));
    }

    if !Path::new(LOCK_FILE).exists() {
        println!("{} Hiç paket kurulu değil.", "Bilgi:".bright_yellow());
        return Ok(());
    }

    let lock_str = fs::read_to_string(LOCK_FILE)?;
    let lock: PaketKilit = serde_json::from_str(&lock_str)?;

    if lock.paketler.is_empty() {
        println!("{} Hiç paket kurulu değil.", "Bilgi:".bright_yellow());
        return Ok(());
    }

    println!(
        "{} Kurulu Hüma Paketleri (Kilitlenmiş Sürümler):",
        "Hüma:".bright_cyan()
    );
    for (ad, bilgi) in &lock.paketler {
        let source_info = bilgi.kaynak.as_deref().unwrap_or("yerel");
        if !bilgi.hash.is_empty() {
            println!(
                "  {} -> {} [{}] ({})",
                ad.bright_green(),
                bilgi.surum.bright_white(),
                &bilgi.hash[..8.min(bilgi.hash.len())].bright_black(),
                source_info.dimmed()
            );
        } else {
            println!(
                "  {} -> {} ({})",
                ad.bright_green(),
                bilgi.surum.bright_white(),
                source_info.dimmed()
            );
        }
    }

    Ok(())
}

/// Yeni bir Hüma paketi (projesi) oluşturur
pub fn create_package(name: &str) -> Result<()> {
    // [1] Güvenlik: Paket adı doğrulaması
    sanitize_package_name(name)?;

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
        kaynak: None,
        github: None,
        lisans: Some("MIT".to_string()),
    };

    // [5] Atomik yazım
    let meta_json = serde_json::to_string_pretty(&meta)?;
    atomic_write_str(&dir.join("huma.json"), &meta_json)?;

    let entry_content = format!(
        "// {} ana giriş dosyası\n\"Hüma projesi aktif.\"'ı yazdır",
        name
    );
    atomic_write_str(&dir.join(format!("{}.hb", name)), &entry_content)?;

    // .gitignore oluştur
    let gitignore_content = "huma_modulleri/\ntarget/\n*.hbc\n.DS_Store\n";
    atomic_write_str(&dir.join(".gitignore"), gitignore_content)?;

    // Modül klasörü ve kilit dosyası ilklendir
    let mod_dir = dir.join(PACKAGE_DIR);
    fs::create_dir_all(&mod_dir)?;

    let lock = PaketKilit {
        paketler: HashMap::new(),
        guncelleme_zamani: chrono::Local::now().to_rfc3339(),
    };
    let lock_json = serde_json::to_string_pretty(&lock)?;
    atomic_write_str(&dir.join(LOCK_FILE), &lock_json)?;

    // Git ilklendirmesi dene
    let _ = std::process::Command::new("git")
        .arg("init")
        .current_dir(dir)
        .output();

    println!(
        "{} '{}' projesi oluşturuldu.",
        "Başarılı!".bright_green(),
        name.bold()
    );
    Ok(())
}

/// Bir paketi kurar ve kilit dosyasına ekler.
/// `trusted`: `--güvenilir` bayrağı ile çağrıldıysa native kod onayı atlanır.
pub fn install_package(input: Option<&str>, trusted: bool) -> Result<()> {
    // 1. Proje dosyası kontrolü
    if !Path::new(PROJECT_FILE).exists() {
        return Err(anyhow!(
            "Bu dizinde huma.json bulunamadı. Önce 'huma paket ilkle' çalıştırın."
        ));
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
                println!(
                    "{} {} bağımlılık kuruluyor...",
                    "Hüma:".bright_cyan(),
                    deps.len()
                );
                let mut failures = Vec::new();
                for (ad, _surum) in deps {
                    if let Err(e) = install_package(Some(&ad), trusted) {
                        println!(
                            "{} {} paketi kurulurken hata: {}",
                            "Uyarı:".bright_yellow(),
                            ad,
                            e
                        );
                        failures.push(format!("{}: {}", ad, e));
                    }
                }
                if failures.is_empty() {
                    return Ok(());
                }
                return Err(anyhow!(
                    "Bazı bağımlılıklar kurulamadı:\n{}",
                    failures.join("\n")
                ));
            } else {
                println!("{} Bağımlılık listesi boş.", "Bilgi:".bright_yellow());
                return Ok(());
            }
        }
    };

    if uzak_paket_girdisi_mi(input) {
        return Err(anyhow!(
            "Uzak paket kurulumu 0.6.0'da devre dışıdır: imzalı kayıt ve çok dosyalı \
             paket doğrulaması henüz yok. Yalnızca kaynak ağacındaki yerel paketler kurulabilir."
        ));
    }
    sanitize_package_name(input)?;

    // Kaynak ağacındaki dağıtılmış paketlerden kurulum.
    if BUILTIN_PACKAGES.contains(&input) {
        println!(
            "{} '{}' yerel dağıtılmış paketlerden aranıyor...",
            "Hüma:".bright_cyan(),
            input.bold(),
        );

        let local_paths = find_local_package(input);
        if let Some(local_path) = local_paths {
            return install_from_local(&local_path, input, trusted);
        }
        return Err(anyhow!(
            "'{}' yerel kaynak ağacında bulunamadı. 0.6.0 uzak indirme yapmaz.",
            input
        ));
    }

    Err(anyhow!(
        "Paket '{}' bulunamadı. \n\
         Yerel kaynak ağacında dağıtılan paketler: {}",
        input,
        BUILTIN_PACKAGES.join(", ")
    ))
}

/// [4] Yerel workspace'te paketi arar (mono-repo senaryosu)
fn find_local_package(name: &str) -> Option<PathBuf> {
    let mut search_dirs = vec![PathBuf::from(".")];
    if let Ok(cwd) = std::env::current_dir() {
        let mut curr = cwd.as_path();
        while let Some(parent) = curr.parent() {
            search_dirs.push(parent.to_path_buf());
            curr = parent;
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut curr = exe.as_path();
        while let Some(parent) = curr.parent() {
            search_dirs.push(parent.to_path_buf());
            curr = parent;
        }
    }

    for base in &search_dirs {
        let candidates = [
            base.join(format!("{}/{}", PACKAGE_DIR, name)),
            base.join(format!("lib/{}", name)),
            base.join(name),
        ];

        for candidate in &candidates {
            let json_candidates = [candidate.join("huma.json"), candidate.join("paket.json")];

            for json_path in &json_candidates {
                if json_path.exists() {
                    return Some(candidate.clone());
                }
            }
        }
    }

    None
}

/// [4] Yerel dizinden paket kurar
fn install_from_local(local_path: &Path, name: &str, trusted: bool) -> Result<()> {
    println!(
        "{} '{}' yerel kaynaktan kuruluyor: {}",
        "Hüma:".bright_cyan(),
        name.bold(),
        local_path.display().to_string().dimmed()
    );

    // Metadata'yı oku
    let json_path = if local_path.join("huma.json").exists() {
        local_path.join("huma.json")
    } else if local_path.join("paket.json").exists() {
        local_path.join("paket.json")
    } else {
        return Err(anyhow!(
            "'{}' dizininde huma.json veya paket.json bulunamadı.",
            local_path.display()
        ));
    };

    let meta_str = fs::read_to_string(&json_path)?;
    let meta: PaketMetadata = serde_json::from_str(&meta_str)?;
    sanitize_package_name(&meta.ad)?;
    sanitize_package_name(&meta.giris)?;

    // [2] Güvenlik: Native kod kontrolü
    check_native_code_safety(&meta, trusted)?;

    let package_dir = PathBuf::from(PACKAGE_DIR);
    let package_path = package_dir.join(&meta.ad);
    let kaynak_hedefle_ayni = package_path.exists()
        && local_path
            .canonicalize()
            .ok()
            .zip(package_path.canonicalize().ok())
            .is_some_and(|(kaynak, hedef)| kaynak == hedef);
    if !kaynak_hedefle_ayni {
        copy_dir_contents(local_path, &package_path)?;
    }

    // Giriş dosyasını oku
    let entry_path = package_path.join(&meta.giris);
    let content = fs::read_to_string(&entry_path)
        .with_context(|| format!("Giriş dosyası okunamadı: {}", entry_path.display()))?;

    save_package(meta, &content, Some("yerel"), trusted)?;

    Ok(())
}

fn copy_dir_contents(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry.file_type()?.is_symlink() {
            return Err(anyhow!(
                "Paket kopyalanırken sembolik bağlantı reddedildi: {}",
                entry_path.display()
            ));
        }
        let target_path = dst.join(entry.file_name());
        if entry_path.is_dir() {
            let name = entry.file_name();
            if name != ".git" && name != "target" && name != "huma_modulleri" {
                copy_dir_contents(&entry_path, &target_path)?;
            }
        } else {
            fs::copy(&entry_path, &target_path)?;
        }
    }
    Ok(())
}

/// Paketi diske kaydeder, kilit dosyasını günceller ve [7] alt bağımlılıkları kurar.
fn save_package(
    meta: PaketMetadata,
    content: &str,
    source: Option<&str>,
    trusted: bool,
) -> Result<()> {
    // [1] Güvenlik: Son kez doğrula
    sanitize_package_name(&meta.ad)?;
    sanitize_package_name(&meta.giris)?;

    let package_dir = PathBuf::from(PACKAGE_DIR);
    let package_path = package_dir.join(&meta.ad);

    // [1] Güvenlik: Canonical path doğrulaması
    let entry_file = package_path.join(&meta.giris);
    fs::create_dir_all(&package_path)?;
    verify_path_within_boundary(&entry_file, &package_dir)?;

    // 1. Proje dosyasını güncelle (bağımlılık ekle)
    if Path::new(PROJECT_FILE).exists() {
        let proj_str = fs::read_to_string(PROJECT_FILE)?;
        let mut proj_meta: PaketMetadata = serde_json::from_str(&proj_str)?;
        if proj_meta.bagimliliklar.is_none() {
            proj_meta.bagimliliklar = Some(HashMap::new());
        }
        if let Some(ref mut deps) = proj_meta.bagimliliklar {
            deps.insert(meta.ad.clone(), format!("^{}", meta.surum));
        }
        let proj_json = serde_json::to_string_pretty(&proj_meta)?;
        // [5] Atomik yazım
        atomic_write_str(Path::new(PROJECT_FILE), &proj_json)?;
    }

    // 2. Paketi modül dizinine yaz — [5] Atomik yazım
    atomic_write_str(&entry_file, content)?;

    let meta_json = serde_json::to_string_pretty(&meta)?;
    atomic_write_str(&package_path.join("paket.json"), &meta_json)?;

    // 3. [3] Kilit Dosyasını güncelle (hash ile)
    let hash = calculate_package_hash(&package_path, &meta)?;
    update_lock_file(&meta.ad, &meta.surum, &hash, source)?;

    println!(
        "{} {} v{} [hash:{}] başarıyla kuruldu.",
        "Başarılı!".bright_green(),
        meta.ad.bold(),
        meta.surum.bright_white(),
        &hash[..8]
    );

    // [7] Özyinelemeli bağımlılık kurulumu
    if let Some(deps) = &meta.bagimliliklar {
        if !deps.is_empty() {
            install_dependencies_recursive(deps, &mut HashSet::new(), trusted)?;
        }
    }

    Ok(())
}

// ─── [7] Özyinelemeli Bağımlılık Kurulumu ──────────────────────────────────

/// Alt bağımlılıkları özyinelemeli olarak kurar. Döngüsel bağımlılıkları `visited`
/// seti ile engeller.
fn install_dependencies_recursive(
    deps: &HashMap<String, String>,
    visited: &mut HashSet<String>,
    trusted: bool,
) -> Result<()> {
    let mut failures = Vec::new();
    for dep_name in deps.keys() {
        if visited.contains(dep_name) {
            println!(
                "  {} '{}' zaten işlendi, döngüsel bağımlılık atlandı.",
                "↳".bright_black(),
                dep_name.dimmed()
            );
            continue;
        }

        visited.insert(dep_name.clone());

        // Zaten kurulu mu kontrol et
        let pkg_dir = PathBuf::from(PACKAGE_DIR).join(dep_name);
        if pkg_dir.exists() {
            continue;
        }

        println!(
            "  {} Alt bağımlılık kuruluyor: {}",
            "↳".bright_cyan(),
            dep_name.bold()
        );

        if let Err(e) = install_package(Some(dep_name), trusted) {
            println!(
                "  {} Alt bağımlılık '{}' kurulamadı: {}",
                "Uyarı:".bright_yellow(),
                dep_name,
                e
            );
            failures.push(format!("{}: {}", dep_name, e));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(anyhow!(
            "Alt bağımlılık kurulumu kısmen başarısız:\n{}",
            failures.join("\n")
        ))
    }
}

/// Paketin yayınlanabilirliğini doğrular
pub fn verify_package() -> Result<()> {
    if !Path::new(PROJECT_FILE).exists() {
        return Err(anyhow!(
            "Bu dizinde bir Hüma projesi (huma.json) bulunamadı."
        ));
    }

    let meta_content = fs::read_to_string(PROJECT_FILE)?;
    let meta: PaketMetadata = serde_json::from_str(&meta_content)?;

    if meta.ad.is_empty() || meta.surum.is_empty() {
        return Err(anyhow!("Paket adı veya sürümü eksik."));
    }

    // Sürüm geçerli mi kontrol et
    Version::parse(&meta.surum)?;

    // [1] Güvenlik: Paket adı doğrulaması
    sanitize_package_name(&meta.ad)?;
    sanitize_package_name(&meta.giris)?;

    // Giriş dosyası var mı?
    if !Path::new(&meta.giris).exists() {
        return Err(anyhow!(
            "Giriş dosyası bulunamadı: '{}'. huma.json içindeki 'giris' alanını kontrol edin.",
            meta.giris
        ));
    }

    // Hüma sürüm kısıtı tanımlıysa parse et ve mevcut sürümle karşılaştır
    if let Some(req_str) = &meta.huma_surum {
        let req = VersionReq::parse(req_str)
            .with_context(|| format!("Geçersiz huma_surum ifadesi: '{}'", req_str))?;
        let current = Version::parse(CURRENT_HUMA_VER)?;
        if !req.matches(&current) {
            return Err(anyhow!(
                "Sürüm uyumsuzluğu: proje Hüma {} gerektiriyor, mevcut sürüm v{}.",
                req_str,
                CURRENT_HUMA_VER
            ));
        }
    }

    // Bağımlılık semver kısıtlarını doğrula
    if let Some(deps) = &meta.bagimliliklar {
        for (dep_name, dep_req) in deps {
            sanitize_package_name(dep_name)?;
            VersionReq::parse(dep_req).with_context(|| {
                format!(
                    "Geçersiz bağımlılık sürüm kısıtı: {} -> '{}'",
                    dep_name, dep_req
                )
            })?;
        }

        if Path::new(LOCK_FILE).exists() {
            let lock_content = fs::read_to_string(LOCK_FILE)?;
            let lock: PaketKilit =
                serde_json::from_str(&lock_content).with_context(|| "huma.lock parse edilemedi")?;
            for dep_name in deps.keys() {
                if !lock.paketler.contains_key(dep_name) {
                    return Err(anyhow!(
                        "Bağımlılık '{}' huma.json içinde var fakat huma.lock içinde kilitlenmemiş.",
                        dep_name
                    ));
                }
            }
        }
    }

    // Lock dosyası varsa parse et ve paket bütünlüğünü doğrula
    if Path::new(LOCK_FILE).exists() {
        let lock_content = fs::read_to_string(LOCK_FILE)?;
        let lock: PaketKilit =
            serde_json::from_str(&lock_content).with_context(|| "huma.lock parse edilemedi")?;

        for (pkg_name, lock_info) in &lock.paketler {
            let pkg_dir = PathBuf::from(PACKAGE_DIR).join(pkg_name);
            let pkg_meta_path = pkg_dir.join("paket.json");
            if !pkg_meta_path.exists() {
                return Err(anyhow!(
                    "Kilit dosyasında '{}' var ancak metadata dosyası eksik: {}",
                    pkg_name,
                    pkg_meta_path.display()
                ));
            }

            let pkg_meta_content = fs::read_to_string(&pkg_meta_path)?;
            let pkg_meta: PaketMetadata =
                serde_json::from_str(&pkg_meta_content).with_context(|| {
                    format!(
                        "Paket metadata parse edilemedi: {}",
                        pkg_meta_path.display()
                    )
                })?;
            sanitize_package_name(&pkg_meta.ad)?;
            sanitize_package_name(&pkg_meta.giris)?;
            let pkg_entry_path = pkg_dir.join(&pkg_meta.giris);
            verify_path_within_boundary(&pkg_entry_path, &pkg_dir)?;
            if !pkg_entry_path.exists() {
                return Err(anyhow!(
                    "Kilitteki '{}' paketinin giriş dosyası eksik: {}",
                    pkg_name,
                    pkg_entry_path.display()
                ));
            }

            let computed_hash = calculate_package_hash(&pkg_dir, &pkg_meta)?;

            if !lock_info.hash.is_empty() && lock_info.hash != computed_hash {
                return Err(anyhow!(
                    "Bütünlük hatası: '{}' paketinin hash değeri uyuşmuyor (lock: {}, hesaplanan: {}).",
                    pkg_name,
                    &lock_info.hash[..16.min(lock_info.hash.len())],
                    &computed_hash[..16.min(computed_hash.len())]
                ));
            }

            if lock_info.surum != pkg_meta.surum {
                return Err(anyhow!(
                    "Sürüm uyuşmazlığı: lock '{}' için {} diyor, paket metadata {}.",
                    pkg_name,
                    lock_info.surum,
                    pkg_meta.surum
                ));
            }
        }
    }

    println!(
        "{} Paket '{}' v{} için yerel manifest ve dosya denetimleri geçti.",
        "Doğrulandı:".bright_green(),
        meta.ad,
        meta.surum
    );
    Ok(())
}

/// Kilit dosyasını atomik olarak günceller
fn update_lock_file(name: &str, version: &str, hash: &str, source: Option<&str>) -> Result<()> {
    let mut lock = if Path::new(LOCK_FILE).exists() {
        let s = fs::read_to_string(LOCK_FILE)?;
        serde_json::from_str::<PaketKilit>(&s).unwrap_or_default()
    } else {
        PaketKilit::default()
    };

    lock.paketler.insert(
        name.to_string(),
        KilitBilgisi {
            surum: version.to_string(),
            hash: hash.to_string(),
            kaynak: source.map(|s| s.to_string()),
        },
    );
    lock.guncelleme_zamani = chrono::Local::now().to_rfc3339();

    // [5] Atomik yazım
    let lock_json = serde_json::to_string_pretty(&lock)?;
    atomic_write_str(Path::new(LOCK_FILE), &lock_json)?;

    Ok(())
}

pub fn remove_package(name: &str) -> Result<()> {
    // [1] Güvenlik: Paket adı doğrulaması
    sanitize_package_name(name)?;

    let path = format!("{}/{}", PACKAGE_DIR, name);
    if Path::new(&path).exists() {
        fs::remove_dir_all(&path)?;

        // Kilit dosyasından çıkar — [5] atomik yazım
        if Path::new(LOCK_FILE).exists() {
            let s = fs::read_to_string(LOCK_FILE)?;
            let mut lock: PaketKilit = serde_json::from_str(&s)?;
            lock.paketler.remove(name);
            lock.guncelleme_zamani = chrono::Local::now().to_rfc3339();
            let lock_json = serde_json::to_string_pretty(&lock)?;
            atomic_write_str(Path::new(LOCK_FILE), &lock_json)?;
        }

        // Proje dosyasından bağımlılığı kaldır
        if Path::new(PROJECT_FILE).exists() {
            let proj_str = fs::read_to_string(PROJECT_FILE)?;
            if let Ok(mut proj_meta) = serde_json::from_str::<PaketMetadata>(&proj_str) {
                if let Some(ref mut deps) = proj_meta.bagimliliklar {
                    deps.remove(name);
                }
                let proj_json = serde_json::to_string_pretty(&proj_meta)?;
                atomic_write_str(Path::new(PROJECT_FILE), &proj_json)?;
            }
        }

        println!("{} {} silindi.", "Başarılı!".bright_green(), name.bold());
        Ok(())
    } else {
        Err(anyhow!("Paket bulunamadı."))
    }
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
    betikler.insert(
        "baslat".to_string(),
        format!("huma run {}.hb", default_name),
    );
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
        kaynak: None,
        github: None,
        lisans: Some("MIT".to_string()),
    };

    // [5] Atomik yazım
    let meta_json = serde_json::to_string_pretty(&meta)?;
    atomic_write_str(Path::new(PROJECT_FILE), &meta_json)?;

    let hb_file = format!("{}.hb", default_name);
    if !Path::new(&hb_file).exists() {
        let content = format!(
            "// {} ana giriş dosyası\n\"Hüma projesi aktif.\"'ı yazdır",
            default_name
        );
        atomic_write_str(Path::new(&hb_file), &content)?;
    }

    // Git ilklendirmesi dene
    let _ = std::process::Command::new("git").arg("init").output();

    println!(
        "{} Proje '{}' olarak ilklendirildi.",
        "Başarılı!".bright_green(),
        default_name.bold()
    );
    Ok(())
}

/// Mevcut dizindeki huma.json dosyasını okur
pub fn get_local_metadata() -> Result<PaketMetadata> {
    if !Path::new(PROJECT_FILE).exists() {
        return Err(anyhow!(
            "Bu dizinde bir Hüma projesi (huma.json) bulunamadı."
        ));
    }
    let s = fs::read_to_string(PROJECT_FILE)?;
    let meta: PaketMetadata = serde_json::from_str(&s)?;
    Ok(meta)
}

/// [9] Belirtilen betiği güvenlik kontrolleriyle çalıştırır
pub fn run_script(name: &str) -> Result<()> {
    let meta = get_local_metadata()?;
    if let Some(betikler) = meta.betikler {
        if let Some(komut) = betikler.get(name) {
            // [9] Güvenlik: Tehlikeli karakter kontrolü
            check_script_safety(komut)?;

            println!(
                "{} {} betiği çalıştırılıyor: {}",
                "Hüma:".bright_cyan(),
                name.bold(),
                komut.bright_black()
            );

            let exe_str = std::env::current_exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "huma".to_string());
            let final_cmd = if komut.starts_with("huma ") {
                komut.replacen("huma ", &format!("\"{}\" ", exe_str), 1)
            } else {
                komut.to_string()
            };

            let status = if cfg!(target_os = "windows") {
                std::process::Command::new("cmd")
                    .args(["/C", &final_cmd])
                    .status()?
            } else {
                std::process::Command::new("sh")
                    .args(["-c", &final_cmd])
                    .status()?
            };

            if !status.success() {
                return Err(anyhow!("Betik '{}' hata ile sonlandı.", name));
            }
            Ok(())
        } else {
            Err(anyhow!(
                "'{}' adlı bir betik huma.json içinde bulunamadı.",
                name
            ))
        }
    } else {
        Err(anyhow!("Bu projede hiç betik tanımlanmamış."))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        calculate_package_hash, sanitize_package_name, uzak_paket_girdisi_mi, PaketMetadata,
    };
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn paket_adi_dizin_gecisini_reddeder() {
        assert!(sanitize_package_name("../../etc").is_err());
        assert!(sanitize_package_name("paket/alt").is_err());
        assert!(sanitize_package_name("paket\\alt").is_err());
        assert!(sanitize_package_name("gecerli_paket-1.0").is_ok());
    }

    #[test]
    fn uzak_paket_kaynaklari_taninir() {
        assert!(uzak_paket_girdisi_mi("github.com/kullanici/paket"));
        assert!(uzak_paket_girdisi_mi("https://example.com/paket"));
        assert!(!uzak_paket_girdisi_mi("nlp_temel"));
    }

    #[test]
    fn paket_ozeti_alt_dosya_degisimini_yakalar() {
        let benzersiz = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Sistem saati Unix epoch sonrasında olmalı")
            .as_nanos();
        let kok = std::env::temp_dir().join(format!(
            "huma_paket_hash_test_{}_{}",
            std::process::id(),
            benzersiz
        ));
        fs::create_dir_all(kok.join("alt")).expect("Geçici paket dizini oluşturulmalı");
        fs::write(kok.join("ana.hb"), "\"tamam\"'ı yazdır").expect("Giriş yazılmalı");
        fs::write(kok.join("alt/yardimci.hb"), "x = 1 olsun").expect("Alt dosya yazılmalı");
        let meta: PaketMetadata = serde_json::from_value(serde_json::json!({
            "ad": "ornek",
            "surum": "1.0.0",
            "aciklama": "test",
            "yazar": "test",
            "giris": "ana.hb"
        }))
        .expect("Test metadata'sı geçerli olmalı");

        let once = calculate_package_hash(&kok, &meta).expect("İlk özet hesaplanmalı");
        fs::write(kok.join("alt/yardimci.hb"), "x = 2 olsun").expect("Alt dosya değiştirilmeli");
        let sonra = calculate_package_hash(&kok, &meta).expect("İkinci özet hesaplanmalı");
        fs::remove_dir_all(&kok).expect("Geçici paket dizini temizlenmeli");

        assert_ne!(once, sonra);
    }
}
