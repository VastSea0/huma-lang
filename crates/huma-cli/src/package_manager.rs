use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use unicode_normalization::UnicodeNormalization;
use wait_timeout::ChildExt;

// ─── Hüma Paket Standardı (HPS) ────────────────────────────────────────────

/// Hüma Paket Standardı (HPS) Metadata Dosyası
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
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
    /// Süreç dışı native/haricî modül için sürümlü HMI sözleşmesi.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hmi: Option<huma_hmi::PackageContract>,
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
#[serde(deny_unknown_fields)]
pub struct PaketKilit {
    pub paketler: HashMap<String, KilitBilgisi>,
    pub guncelleme_zamani: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct KilitBilgisi {
    pub surum: String,
    pub hash: String,
    /// Paketin kaynağı (GitHub URL, builtin, vb.)
    #[serde(default)]
    pub kaynak: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub imza: Option<PaketImzasi>,
}

/// Paket içeriğine bağlı, taşınabilir Ed25519 imza zarfı.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PaketImzasi {
    pub sema_surumu: u16,
    pub algoritma: String,
    pub paket: String,
    pub surum: String,
    pub sha256: String,
    pub anahtar_kimligi: String,
    pub acik_anahtar: String,
    pub imza: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DagitimImzasi {
    pub sema_surumu: u16,
    pub algoritma: String,
    pub dosya: String,
    pub sha256: String,
    pub anahtar_kimligi: String,
    pub acik_anahtar: String,
    pub imza: String,
}

// ─── Sabitler ───────────────────────────────────────────────────────────────

const PACKAGE_DIR: &str = "huma_modulleri";
const LOCK_FILE: &str = "huma.lock";
const PROJECT_FILE: &str = "huma.json";
const SIGNATURE_FILE: &str = "huma.sig";
const MANAGED_RUN_ROOT_ENV: &str = "HUMA_MANAGED_RUN_ROOT";
const SIGNATURE_DOMAIN: &[u8] = b"HUMA-PACKAGE-SIGNATURE-V1\0";
const ARTIFACT_SIGNATURE_DOMAIN: &[u8] = b"HUMA-ARTIFACT-SIGNATURE-V1\0";
const CURRENT_HUMA_VER: &str = env!("CARGO_PKG_VERSION");
const MAX_METADATA_BYTES: usize = 1024 * 1024;
const MAX_PACKAGE_FILES: usize = 10_000;
const MAX_PACKAGE_FILE_BYTES: usize = 64 * 1024 * 1024;
const MAX_PACKAGE_TOTAL_BYTES: u64 = 256 * 1024 * 1024;
const MAX_DEPENDENCIES: usize = 1024;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(1);

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
        return Err(anyhow!("Paket adı 128 bayttan uzun olamaz."));
    }
    if name.nfc().collect::<String>() != name {
        return Err(anyhow!(
            "Paket adı Unicode NFC biçiminde olmalıdır: '{}'.",
            name
        ));
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
    if !name.chars().next().is_some_and(char::is_alphanumeric)
        || !name.chars().last().is_some_and(char::is_alphanumeric)
    {
        return Err(anyhow!(
            "Paket adı harf veya rakamla başlayıp bitmelidir: '{}'.",
            name
        ));
    }

    Ok(())
}

fn validate_package_metadata(meta: &PaketMetadata, expected_name: Option<&str>) -> Result<()> {
    sanitize_package_name(&meta.ad)?;
    if let Some(expected_name) = expected_name {
        if meta.ad != expected_name {
            return Err(anyhow!(
                "Paket kimliği uyuşmuyor: '{}' istendi fakat metadata '{}' diyor.",
                expected_name,
                meta.ad
            ));
        }
    }
    sanitize_package_name(&meta.giris)?;
    if Path::new(&meta.giris)
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("hb")
    {
        return Err(anyhow!(
            "Paket giriş dosyası .hb uzantılı olmalıdır: '{}'.",
            meta.giris
        ));
    }
    Version::parse(&meta.surum)
        .with_context(|| format!("Paket sürümü geçerli SemVer değil: '{}'", meta.surum))?;
    for (field, value) in [
        ("açıklama", meta.aciklama.as_str()),
        ("yazar", meta.yazar.as_str()),
    ] {
        if value.len() > 64 * 1024 {
            return Err(anyhow!("Paket {field} alanı 65536 baytı aşamaz."));
        }
    }
    if let Some(requirement) = &meta.huma_surum {
        let requirement = VersionReq::parse(requirement)
            .with_context(|| format!("Geçersiz huma_surum ifadesi: '{requirement}'"))?;
        let current = Version::parse(CURRENT_HUMA_VER)?;
        if !requirement.matches(&current) {
            return Err(anyhow!(
                "Paket Hüma {} gerektiriyor; mevcut sürüm {}.",
                requirement,
                current
            ));
        }
    }
    if let Some(dependencies) = &meta.bagimliliklar {
        if dependencies.len() > MAX_DEPENDENCIES {
            return Err(anyhow!(
                "Paket bağımlılık sayısı {} sınırını aşıyor.",
                MAX_DEPENDENCIES
            ));
        }
        for (name, requirement) in dependencies {
            sanitize_package_name(name)?;
            VersionReq::parse(requirement).with_context(|| {
                format!("Geçersiz bağımlılık sürüm kısıtı: {name} -> '{requirement}'")
            })?;
        }
    }
    if let Some(scripts) = &meta.betikler {
        if scripts.len() > MAX_DEPENDENCIES {
            return Err(anyhow!(
                "Paket betik sayısı {} sınırını aşıyor.",
                MAX_DEPENDENCIES
            ));
        }
        for (name, command) in scripts {
            sanitize_package_name(name)?;
            if command.is_empty() || command.len() > 64 * 1024 {
                return Err(anyhow!("'{name}' betiği boş olamaz ve 65536 baytı aşamaz."));
            }
        }
    }
    if let Some(dependencies) = &meta.crate_bagimliliklari {
        if dependencies.len() > MAX_DEPENDENCIES {
            return Err(anyhow!(
                "Native crate bağımlılık sayısı {} sınırını aşıyor.",
                MAX_DEPENDENCIES
            ));
        }
        for (name, requirement) in dependencies {
            sanitize_package_name(name)?;
            VersionReq::parse(requirement).with_context(|| {
                format!("Geçersiz crate sürüm kısıtı: {name} -> '{requirement}'")
            })?;
        }
    }
    if let Some(contract) = &meta.hmi {
        contract
            .validate()
            .map_err(|error| anyhow!("Geçersiz HMI sözleşmesi: {error}"))?;
    }
    for (field, value) in [
        ("kaynak", meta.kaynak.as_deref()),
        ("github", meta.github.as_deref()),
        ("lisans", meta.lisans.as_deref()),
        ("yerleşik_rust", meta.yerleşik_rust.as_deref()),
    ] {
        if value.is_some_and(|value| value.len() > 64 * 1024) {
            return Err(anyhow!("Paket {field} alanı 65536 baytı aşamaz."));
        }
    }
    Ok(())
}

fn uzak_paket_girdisi_mi(input: &str) -> bool {
    input.contains('/') || input.starts_with("http:") || input.starts_with("https:")
}

fn read_bytes_limited(path: &Path, limit: usize, purpose: &str) -> Result<Vec<u8>> {
    let file =
        fs::File::open(path).with_context(|| format!("{purpose} açılamadı: {}", path.display()))?;
    if let Ok(metadata) = file.metadata() {
        if metadata.len() > limit as u64 {
            return Err(anyhow!(
                "{purpose} {} bayt sınırını aşıyor: {}",
                limit,
                path.display()
            ));
        }
    }
    let mut bytes = Vec::new();
    file.take((limit as u64) + 1)
        .read_to_end(&mut bytes)
        .with_context(|| format!("{purpose} okunamadı: {}", path.display()))?;
    if bytes.len() > limit {
        return Err(anyhow!(
            "{purpose} {} bayt sınırını aşıyor: {}",
            limit,
            path.display()
        ));
    }
    Ok(bytes)
}

fn read_text_limited(path: &Path, limit: usize, purpose: &str) -> Result<String> {
    String::from_utf8(read_bytes_limited(path, limit, purpose)?)
        .with_context(|| format!("{purpose} geçerli UTF-8 değil: {}", path.display()))
}

fn unique_sibling(path: &Path, label: &str) -> Result<PathBuf> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("Geçici yol için geçerli dosya adı yok: {}", path.display()))?;
    for _ in 0..64 {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(
            ".{file_name}.{label}-{}-{sequence}",
            std::process::id()
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(anyhow!(
        "Benzersiz geçici yol üretilemedi: {}",
        path.display()
    ))
}

// ─── [2] Native Kod Politikası ─────────────────────────────────────────────

/// 0.6 paket yöneticisi native kodu derlemez veya çalıştırmaz. Henüz
/// tanımlanmamış bir ABI'yi kabul etmiş gibi davranmak yerine açıkça reddeder.
fn check_native_code_safety(meta: &PaketMetadata, _trusted: bool) -> Result<()> {
    let has_native_rust = meta.yerleşik_rust.as_ref().is_some_and(|s| !s.is_empty());
    let has_crate_deps = meta
        .crate_bagimliliklari
        .as_ref()
        .is_some_and(|d| !d.is_empty());

    if (has_native_rust || has_crate_deps) && meta.hmi.is_some() {
        return Err(anyhow!(
            "'{}' paketi eski native Rust alanlarıyla HMI sözleşmesini birlikte kullanamaz.",
            meta.ad
        ));
    }
    if has_native_rust || has_crate_deps {
        return Err(anyhow!(
            "'{}' paketi süreç içi native Rust alanları içeriyor. Bu paket kurulamaz; \
             süreç dışı 'hmi' sözleşmesini kullanın. '--güvenilir' bu yapısal \
             güvenlik koşulunu atlamaz.",
            meta.ad
        ));
    }
    Ok(())
}

// ─── [3] Hash Doğrulaması ──────────────────────────────────────────────────

fn canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let value = serde_json::to_value(value)?;
    Ok(serde_json::to_vec(&value)?)
}

fn canonical_json_pretty_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let value = serde_json::to_value(value)?;
    Ok(serde_json::to_vec_pretty(&value)?)
}

fn canonical_json_pretty_string<T: Serialize>(value: &T) -> Result<String> {
    String::from_utf8(canonical_json_pretty_bytes(value)?)
        .map_err(|error| anyhow!("JSON serileştirmesi geçerli UTF-8 üretmedi: {error}"))
}

/// Paket metadata'sı ve bütün dosyalarından tekrarlanabilir SHA-256 özeti hesaplar.
fn calculate_package_hash(package_path: &Path, meta: &PaketMetadata) -> Result<String> {
    fn collect_files(
        current: &Path,
        root: &Path,
        files: &mut Vec<PathBuf>,
        depth: usize,
    ) -> Result<()> {
        if depth > 64 {
            return Err(anyhow!("Paket dizin derinliği 64 sınırını aşıyor."));
        }
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
                collect_files(&path, root, files, depth + 1)?;
            } else if !file_type.is_file() {
                return Err(anyhow!(
                    "Paket yalnızca normal dosya ve dizin içerebilir: {}",
                    path.display()
                ));
            } else if path.parent() != Some(root)
                || !matches!(
                    path.file_name().and_then(|name| name.to_str()),
                    Some("huma.json" | "paket.json" | SIGNATURE_FILE)
                )
            {
                files.push(path);
                if files.len() > MAX_PACKAGE_FILES {
                    return Err(anyhow!(
                        "Paket dosya sayısı {} sınırını aşıyor.",
                        MAX_PACKAGE_FILES
                    ));
                }
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    collect_files(package_path, package_path, &mut files, 0)?;
    files.sort_by(|a, b| {
        a.strip_prefix(package_path)
            .unwrap_or(a)
            .cmp(b.strip_prefix(package_path).unwrap_or(b))
    });

    let mut hasher = Sha256::new();
    let meta_bytes = canonical_json_bytes(meta)?;
    if meta_bytes.len() > MAX_METADATA_BYTES {
        return Err(anyhow!(
            "Paket metadata'sı {} bayt sınırını aşıyor.",
            MAX_METADATA_BYTES
        ));
    }
    hasher.update((meta_bytes.len() as u64).to_le_bytes());
    hasher.update(meta_bytes);
    let mut total_bytes = 0u64;
    for path in files {
        let relative = path
            .strip_prefix(package_path)
            .with_context(|| format!("Paket yolu göreli hale getirilemedi: {}", path.display()))?;
        let relative_bytes = relative.to_str().ok_or_else(|| {
            anyhow!(
                "Paket dosya yolu geçerli UTF-8 olmalıdır: {}",
                path.display()
            )
        })?;
        let content = read_bytes_limited(&path, MAX_PACKAGE_FILE_BYTES, "Paket dosyası")?;
        total_bytes = total_bytes
            .checked_add(content.len() as u64)
            .ok_or_else(|| anyhow!("Paket toplam boyutu taştı."))?;
        if total_bytes > MAX_PACKAGE_TOTAL_BYTES {
            return Err(anyhow!(
                "Paket toplam boyutu {} bayt sınırını aşıyor.",
                MAX_PACKAGE_TOTAL_BYTES
            ));
        }
        hasher.update((relative_bytes.len() as u64).to_le_bytes());
        hasher.update(relative_bytes.as_bytes());
        hasher.update((content.len() as u64).to_le_bytes());
        hasher.update(content);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn package_signature_message(meta: &PaketMetadata, hash: &str) -> Result<Vec<u8>> {
    let digest = hex::decode(hash).with_context(|| "Paket SHA-256 özeti hex olarak okunamadı")?;
    if digest.len() != 32 {
        return Err(anyhow!("Paket SHA-256 özeti 32 bayt olmalıdır."));
    }
    let mut message = Vec::with_capacity(
        SIGNATURE_DOMAIN.len() + meta.ad.len() + meta.surum.len() + digest.len() + 16,
    );
    message.extend_from_slice(SIGNATURE_DOMAIN);
    message.extend_from_slice(&(meta.ad.len() as u64).to_be_bytes());
    message.extend_from_slice(meta.ad.as_bytes());
    message.extend_from_slice(&(meta.surum.len() as u64).to_be_bytes());
    message.extend_from_slice(meta.surum.as_bytes());
    message.extend_from_slice(&digest);
    Ok(message)
}

fn read_signing_key(path: &Path) -> Result<SigningKey> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("Ed25519 anahtar dosyası bulunamadı: {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(anyhow!(
            "Ed25519 anahtarı sembolik bağlantı olmayan normal dosya olmalıdır: {}",
            path.display()
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(anyhow!(
                "Ed25519 anahtar dosyası grup/diğer kullanıcı izinlerine açık; chmod 600 uygulayın: {}",
                path.display()
            ));
        }
    }
    let text = read_text_limited(path, 4_096, "Ed25519 anahtarı")?;
    let bytes = hex::decode(text.trim())
        .with_context(|| "Ed25519 anahtarı 32 baytlık küçük harfli hex tohum olmalıdır")?;
    let bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow!("Ed25519 anahtarı tam olarak 32 bayt olmalıdır."))?;
    Ok(SigningKey::from_bytes(&bytes))
}

fn artifact_hash(path: &Path) -> Result<String> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("Dağıtım dosyası bulunamadı: {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(anyhow!(
            "Dağıtım girdisi sembolik bağlantı olmayan normal dosya olmalıdır: {}",
            path.display()
        ));
    }
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn artifact_signature_message(file_name: &str, hash: &str) -> Result<Vec<u8>> {
    if file_name.is_empty() || file_name.len() > 4_096 || file_name.contains('\0') {
        return Err(anyhow!("Dağıtım dosya etiketi geçersiz."));
    }
    let digest = hex::decode(hash).with_context(|| "Dağıtım SHA-256 özeti geçerli hex değil")?;
    if digest.len() != 32 {
        return Err(anyhow!("Dağıtım SHA-256 özeti 32 bayt olmalıdır."));
    }
    let mut message =
        Vec::with_capacity(ARTIFACT_SIGNATURE_DOMAIN.len() + file_name.len() + digest.len() + 8);
    message.extend_from_slice(ARTIFACT_SIGNATURE_DOMAIN);
    message.extend_from_slice(&(file_name.len() as u64).to_be_bytes());
    message.extend_from_slice(file_name.as_bytes());
    message.extend_from_slice(&digest);
    Ok(message)
}

pub fn sign_distribution_artifact(input: &Path, key_path: &Path, output: &Path) -> Result<()> {
    let file_name = input
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("Dağıtım dosya adı geçerli UTF-8 olmalıdır."))?;
    let hash = artifact_hash(input)?;
    let signing_key = read_signing_key(key_path)?;
    let public_key = signing_key.verifying_key().to_bytes();
    let signature = signing_key.sign(&artifact_signature_message(file_name, &hash)?);
    let envelope = DagitimImzasi {
        sema_surumu: 1,
        algoritma: "Ed25519".to_string(),
        dosya: file_name.to_string(),
        sha256: hash,
        anahtar_kimligi: hex::encode(&Sha256::digest(public_key)[..8]),
        acik_anahtar: hex::encode(public_key),
        imza: hex::encode(signature.to_bytes()),
    };
    atomic_write(output, &canonical_json_pretty_bytes(&envelope)?)?;
    println!(
        "{} {} → {} (anahtar {}).",
        "Dağıtım imzalandı:".bright_green(),
        input.display(),
        output.display(),
        envelope.anahtar_kimligi
    );
    Ok(())
}

pub fn verify_distribution_artifact(input: &Path, signature_path: &Path) -> Result<()> {
    let content = read_text_limited(signature_path, MAX_METADATA_BYTES, "Dağıtım imza zarfı")?;
    let envelope: DagitimImzasi =
        serde_json::from_str(&content).with_context(|| "Dağıtım imza zarfı geçerli JSON değil")?;
    if envelope.sema_surumu != 1 || envelope.algoritma != "Ed25519" {
        return Err(anyhow!("Desteklenmeyen dağıtım imza zarfı."));
    }
    let file_name = input
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("Dağıtım dosya adı geçerli UTF-8 olmalıdır."))?;
    let hash = artifact_hash(input)?;
    if envelope.dosya != file_name || envelope.sha256 != hash {
        return Err(anyhow!(
            "Dağıtım imzası dosya adı veya SHA-256 özetiyle uyuşmuyor."
        ));
    }
    let public_key: [u8; 32] = hex::decode(&envelope.acik_anahtar)
        .with_context(|| "Dağıtım açık anahtarı geçerli hex değil")?
        .try_into()
        .map_err(|_| anyhow!("Ed25519 açık anahtarı 32 bayt olmalıdır."))?;
    let verifying_key = VerifyingKey::from_bytes(&public_key)
        .with_context(|| "Dağıtım Ed25519 açık anahtarı geçersiz")?;
    if verifying_key.is_weak() {
        return Err(anyhow!("Zayıf Ed25519 açık anahtarı reddedildi."));
    }
    if envelope.anahtar_kimligi != hex::encode(&Sha256::digest(public_key)[..8]) {
        return Err(anyhow!("Dağıtım anahtar kimliği uyuşmuyor."));
    }
    let signature: [u8; 64] = hex::decode(&envelope.imza)
        .with_context(|| "Dağıtım Ed25519 imzası geçerli hex değil")?
        .try_into()
        .map_err(|_| anyhow!("Ed25519 imzası 64 bayt olmalıdır."))?;
    verifying_key
        .verify_strict(
            &artifact_signature_message(file_name, &hash)?,
            &Signature::from_bytes(&signature),
        )
        .with_context(|| "Dağıtım Ed25519 imzası doğrulanamadı")?;
    println!(
        "{} {} (anahtar {}).",
        "Dağıtım doğrulandı:".bright_green(),
        input.display(),
        envelope.anahtar_kimligi
    );
    Ok(())
}

fn create_package_signature(
    meta: &PaketMetadata,
    hash: &str,
    signing_key: &SigningKey,
) -> Result<PaketImzasi> {
    let message = package_signature_message(meta, hash)?;
    let signature = signing_key.sign(&message);
    let public_key = signing_key.verifying_key().to_bytes();
    let key_digest = Sha256::digest(public_key);
    Ok(PaketImzasi {
        sema_surumu: 1,
        algoritma: "Ed25519".to_string(),
        paket: meta.ad.clone(),
        surum: meta.surum.clone(),
        sha256: hash.to_string(),
        anahtar_kimligi: hex::encode(&key_digest[..8]),
        acik_anahtar: hex::encode(public_key),
        imza: hex::encode(signature.to_bytes()),
    })
}

fn verify_package_signature(
    meta: &PaketMetadata,
    hash: &str,
    envelope: &PaketImzasi,
) -> Result<()> {
    if envelope.sema_surumu != 1 || envelope.algoritma != "Ed25519" {
        return Err(anyhow!(
            "Desteklenmeyen paket imza zarfı: şema {}, algoritma '{}'.",
            envelope.sema_surumu,
            envelope.algoritma
        ));
    }
    if envelope.paket != meta.ad || envelope.surum != meta.surum || envelope.sha256 != hash {
        return Err(anyhow!(
            "Paket imzası ad, sürüm veya SHA-256 özetiyle uyuşmuyor."
        ));
    }
    let public_key = hex::decode(&envelope.acik_anahtar)
        .with_context(|| "Paket imzasındaki açık anahtar geçerli hex değil")?;
    let public_key: [u8; 32] = public_key
        .try_into()
        .map_err(|_| anyhow!("Ed25519 açık anahtarı 32 bayt olmalıdır."))?;
    let verifying_key =
        VerifyingKey::from_bytes(&public_key).with_context(|| "Ed25519 açık anahtarı geçersiz")?;
    if verifying_key.is_weak() {
        return Err(anyhow!("Zayıf Ed25519 açık anahtarı reddedildi."));
    }
    let expected_key_id = hex::encode(&Sha256::digest(public_key)[..8]);
    if envelope.anahtar_kimligi != expected_key_id {
        return Err(anyhow!("Paket imzası anahtar kimliğiyle uyuşmuyor."));
    }
    let signature =
        hex::decode(&envelope.imza).with_context(|| "Paket Ed25519 imzası geçerli hex değil")?;
    let signature: [u8; 64] = signature
        .try_into()
        .map_err(|_| anyhow!("Ed25519 imzası 64 bayt olmalıdır."))?;
    let signature = Signature::from_bytes(&signature);
    verifying_key
        .verify_strict(&package_signature_message(meta, hash)?, &signature)
        .with_context(|| "Paket Ed25519 imzası doğrulanamadı")
}

fn read_package_signature(package_root: &Path) -> Result<Option<PaketImzasi>> {
    let path = package_root.join(SIGNATURE_FILE);
    if !path.exists() {
        return Ok(None);
    }
    let metadata = fs::symlink_metadata(&path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(anyhow!(
            "Paket imza zarfı normal dosya olmalıdır: {}",
            path.display()
        ));
    }
    let content = read_text_limited(&path, MAX_METADATA_BYTES, "Paket imza zarfı")?;
    serde_json::from_str(&content)
        .with_context(|| format!("Paket imza zarfı geçersiz: {}", path.display()))
        .map(Some)
}

pub fn sign_current_package(key_path: &Path) -> Result<()> {
    let root = std::env::current_dir()?;
    let metadata_text = read_text_limited(
        &root.join(PROJECT_FILE),
        MAX_METADATA_BYTES,
        "Proje metadata'sı",
    )?;
    let meta: PaketMetadata =
        serde_json::from_str(&metadata_text).with_context(|| "huma.json parse edilemedi")?;
    validate_package_metadata(&meta, None)?;
    validate_package_tree(&root, &meta)?;
    let hash = calculate_package_hash(&root, &meta)?;
    let signing_key = read_signing_key(key_path)?;
    let envelope = create_package_signature(&meta, &hash, &signing_key)?;
    atomic_write(
        &root.join(SIGNATURE_FILE),
        &canonical_json_pretty_bytes(&envelope)?,
    )?;
    println!(
        "{} Paket '{}' v{} anahtar {} ile imzalandı.",
        "İmzalandı:".bright_green(),
        meta.ad,
        meta.surum,
        envelope.anahtar_kimligi
    );
    Ok(())
}

pub fn verify_current_package_signature() -> Result<()> {
    let root = std::env::current_dir()?;
    let metadata_text = read_text_limited(
        &root.join(PROJECT_FILE),
        MAX_METADATA_BYTES,
        "Proje metadata'sı",
    )?;
    let meta: PaketMetadata = serde_json::from_str(&metadata_text)?;
    validate_package_metadata(&meta, None)?;
    let hash = calculate_package_hash(&root, &meta)?;
    let envelope = read_package_signature(&root)?
        .ok_or_else(|| anyhow!("Paket imza zarfı bulunamadı: {SIGNATURE_FILE}"))?;
    verify_package_signature(&meta, &hash, &envelope)?;
    println!(
        "{} Paket '{}' v{} için Ed25519 imzası geçerli (anahtar {}).",
        "Doğrulandı:".bright_green(),
        meta.ad,
        meta.surum,
        envelope.anahtar_kimligi
    );
    Ok(())
}

/// İki paket sürümünün makinece okunabilir HMI sözleşmesini SemVer ile birlikte
/// denetler. Kırıcı değişiklik yalnız yeni bir ana paket sürümünde kabul edilir.
pub fn check_api_compatibility(previous_path: &Path, current_path: &Path) -> Result<()> {
    let previous_text = read_text_limited(
        previous_path,
        MAX_METADATA_BYTES,
        "Önceki paket metadata'sı",
    )?;
    let current_text =
        read_text_limited(current_path, MAX_METADATA_BYTES, "Yeni paket metadata'sı")?;
    let previous: PaketMetadata = serde_json::from_str(&previous_text)
        .with_context(|| "Önceki paket metadata'sı geçerli JSON değil")?;
    let current: PaketMetadata = serde_json::from_str(&current_text)
        .with_context(|| "Yeni paket metadata'sı geçerli JSON değil")?;
    validate_package_metadata(&previous, None)?;
    validate_package_metadata(&current, None)?;
    if previous.ad != current.ad {
        return Err(anyhow!(
            "API karşılaştırması aynı paket adını gerektirir: '{}' != '{}'",
            previous.ad,
            current.ad
        ));
    }
    let previous_version = Version::parse(&previous.surum)?;
    let current_version = Version::parse(&current.surum)?;
    if current_version <= previous_version {
        return Err(anyhow!(
            "Yeni paket sürümü eskisinden büyük olmalıdır: {} <= {}",
            current_version,
            previous_version
        ));
    }

    let compatibility = match (&previous.hmi, &current.hmi) {
        (None, _) => Ok(()),
        (Some(_), None) => Err(huma_hmi::HmiError::IncompatibleInterface(
            "paketin HMI sözleşmesi kaldırıldı".to_string(),
        )),
        (Some(previous), Some(current)) => {
            if previous.protocol.major != current.protocol.major {
                Err(huma_hmi::HmiError::IncompatibleInterface(format!(
                    "HMI protokol ana sürümü değişti: {} -> {}",
                    previous.protocol.major, current.protocol.major
                )))
            } else {
                current
                    .interface
                    .check_backward_compatible_with(&previous.interface)
            }
        }
    };

    match compatibility {
        Ok(()) => {
            println!(
                "{} {} {} → {} geriye uyumlu.",
                "API doğrulandı:".bright_green(),
                current.ad,
                previous_version,
                current_version
            );
            Ok(())
        }
        Err(error) if current_version.major > previous_version.major => {
            println!(
                "{} kırıcı değişiklik yeni ana sürümle kapsandı: {}",
                "API doğrulandı:".bright_green(),
                error
            );
            Ok(())
        }
        Err(error) => Err(anyhow!(
            "Kırıcı HMI/API değişikliği ana sürüm artırılmadan yayımlanamaz: {error}"
        )),
    }
}

// ─── [5] Atomik Dosya Yazımı ───────────────────────────────────────────────

/// Dosyayı atomik olarak yazar: önce `.tmp` uzantılı geçici dosyaya yaz,
/// sonra `rename()` ile hedef konuma taşı. Bu sayede yarım kalan yazımlarda
/// (güç kesintisi, çökme) dosya bozulması engellenir.
fn atomic_write(path: &Path, content: &[u8]) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let tmp_path = unique_sibling(path, "tmp")?;
    let mut temporary = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp_path)
        .with_context(|| format!("Geçici dosya oluşturulamadı: {}", tmp_path.display()))?;
    if let Err(error) = temporary
        .write_all(content)
        .and_then(|()| temporary.sync_all())
    {
        drop(temporary);
        let _ = fs::remove_file(&tmp_path);
        return Err(error)
            .with_context(|| format!("Geçici dosya yazılamadı: {}", tmp_path.display()));
    }
    drop(temporary);

    if fs::rename(&tmp_path, path).is_ok() {
        return Ok(());
    }

    // Bazı platformlarda `rename` var olan hedefin üzerine yazamaz. Hedefi
    // aynı dizinde kısa süreliğine yedekleyip başarısızlıkta geri yükle.
    if !path.exists() {
        let _ = fs::remove_file(&tmp_path);
        return Err(anyhow!(
            "Dosya taşıma başarısız: {} → {}",
            tmp_path.display(),
            path.display()
        ));
    }
    let backup_path = unique_sibling(path, "backup")?;
    fs::rename(path, &backup_path).with_context(|| {
        format!(
            "Mevcut dosya yedeklenemedi: {} → {}",
            path.display(),
            backup_path.display()
        )
    })?;
    if let Err(error) = fs::rename(&tmp_path, path) {
        let restore_result = fs::rename(&backup_path, path);
        let _ = fs::remove_file(&tmp_path);
        return match restore_result {
            Ok(()) => Err(error).with_context(|| {
                format!(
                    "Dosya taşıma başarısız; önceki dosya geri yüklendi: {}",
                    path.display()
                )
            }),
            Err(restore_error) => Err(anyhow!(
                "Dosya taşıma başarısız ({error}); önceki dosya da geri yüklenemedi \
                 ({restore_error}). Yedek: {}",
                backup_path.display()
            )),
        };
    }
    if let Err(error) = fs::remove_file(&backup_path) {
        eprintln!(
            "Uyarı: yeni dosya yazıldı fakat eski yedek temizlenemedi ({}): {}",
            backup_path.display(),
            error
        );
    }
    Ok(())
}

/// String içeriği atomik olarak yazar.
fn atomic_write_str(path: &Path, content: &str) -> Result<()> {
    atomic_write(path, content.as_bytes())
}

// ─── [9] Betik Güvenliği ───────────────────────────────────────────────────

/// Paket betikleri kabukta değil, doğrudan argv olarak çalıştırılır.
fn check_script_safety(command: &str) -> Result<()> {
    for pattern in DANGEROUS_SHELL_CHARS {
        if command.contains(pattern) {
            return Err(anyhow!(
                "Betik komutu kabuk operatörü içeremez ('{}'). Betikler doğrudan program ve \
                 argüman listesi olarak çalıştırılır.",
                pattern
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

    let lock = read_lock_or_default(Path::new(LOCK_FILE))?;

    if lock.paketler.is_empty() {
        println!("{} Hiç paket kurulu değil.", "Bilgi:".bright_yellow());
        return Ok(());
    }

    println!(
        "{} Kurulu Hüma Paketleri (Kilitlenmiş Sürümler):",
        "Hüma:".bright_cyan()
    );
    let mut packages = lock.paketler.iter().collect::<Vec<_>>();
    packages.sort_by(|left, right| left.0.cmp(right.0));
    for (ad, bilgi) in packages {
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

    let mut betikler = HashMap::new();
    betikler.insert("baslat".to_string(), format!("huma run {}.hb", name));
    betikler.insert("start".to_string(), format!("huma run {}.hb", name));
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
        hmi: None,
        kaynak: None,
        github: None,
        lisans: Some("MIT".to_string()),
    };
    validate_package_metadata(&meta, Some(name))?;
    let stage = unique_sibling(dir, "create")?;
    let create_result = (|| -> Result<()> {
        fs::create_dir_all(&stage)?;
        let meta_json = canonical_json_pretty_string(&meta)?;
        atomic_write_str(&stage.join("huma.json"), &meta_json)?;
        let entry_content = format!(
            "// {} ana giriş dosyası\n\"Hüma projesi aktif.\"'ı yazdır",
            name
        );
        atomic_write_str(&stage.join(format!("{}.hb", name)), &entry_content)?;
        atomic_write_str(
            &stage.join(".gitignore"),
            "huma_modulleri/\ntarget/\n*.hbc\n.DS_Store\n",
        )?;
        fs::create_dir_all(stage.join(PACKAGE_DIR))?;
        let lock = PaketKilit {
            paketler: HashMap::new(),
            guncelleme_zamani: chrono::Local::now().to_rfc3339(),
        };
        let lock_json = canonical_json_pretty_string(&lock)?;
        atomic_write_str(&stage.join(LOCK_FILE), &lock_json)?;
        fs::rename(&stage, dir).with_context(|| {
            format!(
                "Hazırlanan proje dizini etkinleştirilemedi: {}",
                dir.display()
            )
        })
    })();
    if let Err(error) = create_result {
        if stage.exists() {
            let _ = fs::remove_dir_all(&stage);
        }
        return Err(error);
    }

    println!(
        "{} '{}' projesi oluşturuldu.",
        "Başarılı!".bright_green(),
        name.bold()
    );
    Ok(())
}

/// Bir paketi kurar ve kilit dosyasına ekler.
/// `trusted`, yalnız yerel geliştirme sırasında imzasız pakete açık onay verir.
/// Hüma 0.6 süreç içi native paket ABI'si tanımlamadığından native kodu açmaz.
pub fn install_package(input: Option<&str>, trusted: bool) -> Result<()> {
    if !Path::new(PROJECT_FILE).exists() {
        return Err(anyhow!(
            "Bu dizinde huma.json bulunamadı. Önce 'huma paket ilkle' çalıştırın."
        ));
    }
    let project_bytes = read_bytes_limited(
        Path::new(PROJECT_FILE),
        MAX_METADATA_BYTES,
        "Proje metadata'sı",
    )?;
    let mut project: PaketMetadata = serde_json::from_slice(&project_bytes)
        .with_context(|| "huma.json geçerli paket metadata'sı değil")?;
    validate_package_metadata(&project, None)?;

    let (roots, explicitly_added) = match input {
        Some(input) => {
            if uzak_paket_girdisi_mi(input) {
                return Err(anyhow!(
                    "Uzak paket kurulumu 0.6.0'da devre dışıdır: imzalı kayıt ve çok dosyalı \
                     paket doğrulaması henüz yok. Yalnızca kaynak ağacındaki yerel paketler \
                     kurulabilir."
                ));
            }
            sanitize_package_name(input)?;
            (vec![(input.to_string(), None)], Some(input.to_string()))
        }
        None => {
            let dependencies = project.bagimliliklar.clone().unwrap_or_default();
            if dependencies.is_empty() {
                println!("{} Kurulacak bağımlılık yok.", "Bilgi:".bright_yellow());
                return Ok(());
            }
            if dependencies.len() > MAX_DEPENDENCIES {
                return Err(anyhow!(
                    "Proje bağımlılık sayısı {} sınırını aşıyor.",
                    MAX_DEPENDENCIES
                ));
            }
            let mut roots = dependencies.into_iter().collect::<Vec<_>>();
            roots.sort_by(|left, right| left.0.cmp(&right.0));
            let roots = roots
                .into_iter()
                .map(|(name, requirement)| {
                    let requirement = VersionReq::parse(&requirement).with_context(|| {
                        format!("Geçersiz proje bağımlılık sürüm kısıtı: {name} -> '{requirement}'")
                    })?;
                    Ok((name, Some(requirement)))
                })
                .collect::<Result<Vec<_>>>()?;
            (roots, None)
        }
    };

    let resolved = resolve_local_packages(&roots, trusted)?;
    if let Some(root_name) = explicitly_added {
        let root = resolved
            .iter()
            .find(|package| package.meta.ad == root_name)
            .ok_or_else(|| anyhow!("Çözümlenen kök paket bulunamadı: {root_name}"))?;
        project
            .bagimliliklar
            .get_or_insert_with(HashMap::new)
            .insert(root_name, format!("^{}", root.meta.surum));
    }
    install_resolved_transaction(resolved, project, &project_bytes, trusted)
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

#[derive(Clone)]
struct ResolvedPackage {
    meta: PaketMetadata,
    source_path: PathBuf,
}

fn package_metadata_path(package_path: &Path) -> Result<PathBuf> {
    for file_name in ["paket.json", "huma.json"] {
        let candidate = package_path.join(file_name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(anyhow!(
        "'{}' dizininde paket.json veya huma.json bulunamadı.",
        package_path.display()
    ))
}

fn validate_package_tree(package_path: &Path, meta: &PaketMetadata) -> Result<()> {
    let metadata = fs::symlink_metadata(package_path).with_context(|| {
        format!(
            "Paket kaynak dizini incelenemedi: {}",
            package_path.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(anyhow!(
            "Paket kaynağı sembolik bağlantı olmayan bir dizin olmalıdır: {}",
            package_path.display()
        ));
    }
    let canonical_root = package_path
        .canonicalize()
        .with_context(|| format!("Paket dizini çözümlenemedi: {}", package_path.display()))?;
    let entry_path = package_path.join(&meta.giris);
    let entry_metadata = fs::symlink_metadata(&entry_path)
        .with_context(|| format!("Paket giriş dosyası bulunamadı: {}", entry_path.display()))?;
    if entry_metadata.file_type().is_symlink() || !entry_metadata.is_file() {
        return Err(anyhow!(
            "Paket giriş yolu sembolik bağlantı olmayan normal dosya olmalıdır: {}",
            entry_path.display()
        ));
    }
    let canonical_entry = entry_path.canonicalize().with_context(|| {
        format!(
            "Paket giriş dosyası çözümlenemedi: {}",
            entry_path.display()
        )
    })?;
    if !canonical_entry.starts_with(&canonical_root) {
        return Err(anyhow!(
            "Paket giriş dosyası kaynak dizininin dışına çıkıyor: {}",
            entry_path.display()
        ));
    }
    if let Some(contract) = &meta.hmi {
        let executable = package_path.join(&contract.executable);
        let executable_metadata = fs::symlink_metadata(&executable)
            .with_context(|| format!("HMI yürütülebiliri bulunamadı: {}", executable.display()))?;
        if executable_metadata.file_type().is_symlink() || !executable_metadata.is_file() {
            return Err(anyhow!(
                "HMI yürütülebiliri sembolik bağlantı olmayan normal dosya olmalıdır: {}",
                executable.display()
            ));
        }
        let canonical_executable = executable.canonicalize()?;
        if !canonical_executable.starts_with(&canonical_root) {
            return Err(anyhow!(
                "HMI yürütülebiliri paket dizininin dışına çıkıyor: {}",
                executable.display()
            ));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if executable_metadata.permissions().mode() & 0o111 == 0 {
                return Err(anyhow!(
                    "HMI yürütülebilirinde çalıştırma izni yok: {}",
                    executable.display()
                ));
            }
        }
    }
    read_text_limited(&entry_path, MAX_PACKAGE_FILE_BYTES, "Paket giriş dosyası")?;
    calculate_package_hash(package_path, meta)?;
    Ok(())
}

fn load_local_package(name: &str, trusted: bool) -> Result<ResolvedPackage> {
    sanitize_package_name(name)?;
    if !BUILTIN_PACKAGES.contains(&name) {
        return Err(anyhow!(
            "Paket '{}' bulunamadı. Yerel kaynak ağacında dağıtılan paketler: {}",
            name,
            BUILTIN_PACKAGES.join(", ")
        ));
    }
    let source_path = find_local_package(name).ok_or_else(|| {
        anyhow!(
            "'{}' yerel kaynak ağacında bulunamadı. 0.6.0 uzak indirme yapmaz.",
            name
        )
    })?;
    let metadata_path = package_metadata_path(&source_path)?;
    let metadata_text = read_text_limited(&metadata_path, MAX_METADATA_BYTES, "Paket metadata'sı")?;
    let meta: PaketMetadata = serde_json::from_str(&metadata_text).with_context(|| {
        format!(
            "Paket metadata'sı geçerli JSON değil: {}",
            metadata_path.display()
        )
    })?;
    validate_package_metadata(&meta, Some(name))?;
    validate_package_tree(&source_path, &meta)?;
    check_native_code_safety(&meta, trusted)?;
    Ok(ResolvedPackage { meta, source_path })
}

fn resolve_local_packages(
    roots: &[(String, Option<VersionReq>)],
    trusted: bool,
) -> Result<Vec<ResolvedPackage>> {
    fn resolve_one(
        name: &str,
        requirement: Option<&VersionReq>,
        trusted: bool,
        resolved: &mut HashMap<String, ResolvedPackage>,
        visiting: &mut Vec<String>,
        order: &mut Vec<String>,
    ) -> Result<()> {
        if let Some(package) = resolved.get(name) {
            let version = Version::parse(&package.meta.surum)?;
            if requirement.is_some_and(|requirement| !requirement.matches(&version)) {
                return Err(anyhow!(
                    "Bağımlılık sürümü uyuşmuyor: '{}' için {} gerekiyor, {} çözümlendi.",
                    name,
                    requirement.map(ToString::to_string).unwrap_or_default(),
                    version
                ));
            }
            return Ok(());
        }
        if let Some(start) = visiting.iter().position(|current| current == name) {
            let mut cycle = visiting[start..].to_vec();
            cycle.push(name.to_string());
            return Err(anyhow!(
                "Döngüsel paket bağımlılığı algılandı: {}",
                cycle.join(" -> ")
            ));
        }

        let package = load_local_package(name, trusted)?;
        let version = Version::parse(&package.meta.surum)?;
        if requirement.is_some_and(|requirement| !requirement.matches(&version)) {
            return Err(anyhow!(
                "Bağımlılık sürümü uyuşmuyor: '{}' için {} gerekiyor, yerel sürüm {}.",
                name,
                requirement.map(ToString::to_string).unwrap_or_default(),
                version
            ));
        }

        visiting.push(name.to_string());
        let mut dependencies = package
            .meta
            .bagimliliklar
            .clone()
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>();
        dependencies.sort_by(|left, right| left.0.cmp(&right.0));
        for (dependency, requirement) in dependencies {
            let requirement = VersionReq::parse(&requirement)?;
            resolve_one(
                &dependency,
                Some(&requirement),
                trusted,
                resolved,
                visiting,
                order,
            )?;
        }
        let popped = visiting.pop();
        if popped.as_deref() != Some(name) {
            return Err(anyhow!(
                "İç hata: paket çözümleme yığını tutarsızlaştı ('{}').",
                name
            ));
        }
        resolved.insert(name.to_string(), package);
        order.push(name.to_string());
        Ok(())
    }

    let mut resolved = HashMap::new();
    let mut visiting = Vec::new();
    let mut order = Vec::new();
    for (name, requirement) in roots {
        resolve_one(
            name,
            requirement.as_ref(),
            trusted,
            &mut resolved,
            &mut visiting,
            &mut order,
        )?;
    }
    order
        .into_iter()
        .map(|name| {
            resolved
                .remove(&name)
                .ok_or_else(|| anyhow!("İç hata: çözümlenen paket kayboldu: {name}"))
        })
        .collect()
}

struct PreparedPackage {
    meta: PaketMetadata,
    target: PathBuf,
    staged: Option<PathBuf>,
    hash: String,
    signature: Option<PaketImzasi>,
}

fn cleanup_stages(packages: &mut [PreparedPackage]) {
    for package in packages {
        if let Some(stage) = package.staged.take() {
            if let Err(error) = fs::remove_dir_all(&stage) {
                eprintln!(
                    "Uyarı: geçici paket dizini temizlenemedi ({}): {}",
                    stage.display(),
                    error
                );
            }
        }
    }
}

fn prepare_packages(
    resolved: Vec<ResolvedPackage>,
    allow_unsigned_local: bool,
) -> Result<Vec<PreparedPackage>> {
    let package_root = PathBuf::from(PACKAGE_DIR);
    fs::create_dir_all(&package_root)?;
    let canonical_root = package_root.canonicalize().with_context(|| {
        format!(
            "Paket hedef dizini çözümlenemedi: {}",
            package_root.display()
        )
    })?;
    let mut prepared = Vec::with_capacity(resolved.len());
    for package in resolved {
        let target = package_root.join(&package.meta.ad);
        let target_parent = target.parent().unwrap_or(&package_root);
        if target_parent.canonicalize()? != canonical_root {
            cleanup_stages(&mut prepared);
            return Err(anyhow!(
                "Paket hedefi izin verilen dizinin dışına çıkıyor: {}",
                target.display()
            ));
        }
        let stage = match unique_sibling(&target, "stage") {
            Ok(stage) => stage,
            Err(error) => {
                cleanup_stages(&mut prepared);
                return Err(error);
            }
        };
        if let Err(error) = copy_dir_contents(&package.source_path, &stage)
            .and_then(|()| validate_package_tree(&stage, &package.meta))
        {
            let _ = fs::remove_dir_all(&stage);
            cleanup_stages(&mut prepared);
            return Err(error);
        }
        let staged = Some(stage);
        let hash_root = staged
            .as_deref()
            .ok_or_else(|| anyhow!("İç hata: paket hazırlama dizini kayboldu"))?;
        let hash = match calculate_package_hash(hash_root, &package.meta) {
            Ok(hash) => hash,
            Err(error) => {
                if let Some(stage) = &staged {
                    let _ = fs::remove_dir_all(stage);
                }
                cleanup_stages(&mut prepared);
                return Err(error);
            }
        };
        let signature = match read_package_signature(hash_root) {
            Ok(Some(signature)) => {
                if let Err(error) = verify_package_signature(&package.meta, &hash, &signature) {
                    if let Some(stage) = &staged {
                        let _ = fs::remove_dir_all(stage);
                    }
                    cleanup_stages(&mut prepared);
                    return Err(
                        error.context(format!("'{}' paketinin imzası geçersiz", package.meta.ad))
                    );
                }
                Some(signature)
            }
            Ok(None) if allow_unsigned_local => None,
            Ok(None) => {
                if let Some(stage) = &staged {
                    let _ = fs::remove_dir_all(stage);
                }
                cleanup_stages(&mut prepared);
                return Err(anyhow!(
                    "'{}' paketi Ed25519 imzası taşımıyor. Güvenli kurulum varsayılanı \n+                     imza ister; yalnız denetlediğiniz yerel geliştirme kaynağı için \n+                     açıkça '--güvenilir' verin.",
                    package.meta.ad
                ));
            }
            Err(error) => {
                if let Some(stage) = &staged {
                    let _ = fs::remove_dir_all(stage);
                }
                cleanup_stages(&mut prepared);
                return Err(error);
            }
        };
        prepared.push(PreparedPackage {
            meta: package.meta,
            target,
            staged,
            hash,
            signature,
        });
    }
    Ok(prepared)
}

struct DirectorySwap {
    target: PathBuf,
    backup: Option<PathBuf>,
}

fn rollback_directory_swaps(swaps: &mut Vec<DirectorySwap>) -> Vec<String> {
    let mut errors = Vec::new();
    while let Some(swap) = swaps.pop() {
        if swap.target.exists() {
            if let Err(error) = fs::remove_dir_all(&swap.target) {
                errors.push(format!("{} kaldırılamadı: {error}", swap.target.display()));
                continue;
            }
        }
        if let Some(backup) = swap.backup {
            if let Err(error) = fs::rename(&backup, &swap.target) {
                errors.push(format!(
                    "{} geri yüklenemedi (yedek {}): {error}",
                    swap.target.display(),
                    backup.display()
                ));
            }
        }
    }
    errors
}

fn install_resolved_transaction(
    resolved: Vec<ResolvedPackage>,
    project: PaketMetadata,
    original_project: &[u8],
    allow_unsigned_local: bool,
) -> Result<()> {
    let mut prepared = prepare_packages(resolved, allow_unsigned_local)?;
    let mut lock = read_lock_or_default(Path::new(LOCK_FILE))?;
    for package in &prepared {
        lock.paketler.insert(
            package.meta.ad.clone(),
            KilitBilgisi {
                surum: package.meta.surum.clone(),
                hash: package.hash.clone(),
                kaynak: Some(if package.signature.is_some() {
                    "yerel-imzalı".to_string()
                } else {
                    "yerel-imzasız-güvenilir".to_string()
                }),
                imza: package.signature.clone(),
            },
        );
    }
    lock.guncelleme_zamani = chrono::Local::now().to_rfc3339();
    let project_json = canonical_json_pretty_bytes(&project)?;
    let lock_json = canonical_json_pretty_bytes(&lock)?;
    if project_json.len() > MAX_METADATA_BYTES || lock_json.len() > MAX_METADATA_BYTES {
        cleanup_stages(&mut prepared);
        return Err(anyhow!(
            "Proje veya kilit metadata'sı {} bayt sınırını aşıyor.",
            MAX_METADATA_BYTES
        ));
    }

    let mut swaps = Vec::new();
    for index in 0..prepared.len() {
        let Some(stage) = prepared[index].staged.take() else {
            continue;
        };
        let target = prepared[index].target.clone();
        let backup = if target.exists() {
            let backup = match unique_sibling(&target, "backup") {
                Ok(backup) => backup,
                Err(error) => {
                    let _ = fs::remove_dir_all(&stage);
                    cleanup_stages(&mut prepared);
                    let rollback_errors = rollback_directory_swaps(&mut swaps);
                    return Err(anyhow!(
                        "Paket yedek yolu hazırlanamadı: {error}{}",
                        if rollback_errors.is_empty() {
                            String::new()
                        } else {
                            format!("; geri alma hataları: {}", rollback_errors.join("; "))
                        }
                    ));
                }
            };
            if let Err(error) = fs::rename(&target, &backup) {
                let _ = fs::remove_dir_all(&stage);
                cleanup_stages(&mut prepared);
                let rollback_errors = rollback_directory_swaps(&mut swaps);
                return Err(anyhow!(
                    "Mevcut paket yedeklenemedi: {} ({error}){}",
                    target.display(),
                    if rollback_errors.is_empty() {
                        String::new()
                    } else {
                        format!("; geri alma hataları: {}", rollback_errors.join("; "))
                    }
                ));
            }
            Some(backup)
        } else {
            None
        };
        if let Err(error) = fs::rename(&stage, &target) {
            let restore_error = backup
                .as_ref()
                .and_then(|backup| fs::rename(backup, &target).err());
            cleanup_stages(&mut prepared);
            let rollback_errors = rollback_directory_swaps(&mut swaps);
            return Err(anyhow!(
                "Hazırlanan paket etkinleştirilemedi: {} ({error}){}{}",
                target.display(),
                restore_error.map_or_else(String::new, |restore_error| format!(
                    "; önceki paket geri yüklenemedi: {restore_error}"
                )),
                if rollback_errors.is_empty() {
                    String::new()
                } else {
                    format!("; geri alma hataları: {}", rollback_errors.join("; "))
                }
            ));
        }
        swaps.push(DirectorySwap { target, backup });
    }

    let project_changed = project_json != original_project;
    if project_changed {
        if let Err(error) = atomic_write(Path::new(PROJECT_FILE), &project_json) {
            let rollback_errors = rollback_directory_swaps(&mut swaps);
            return Err(anyhow!(
                "Proje metadata'sı güncellenemedi: {error}{}",
                if rollback_errors.is_empty() {
                    String::new()
                } else {
                    format!("; geri alma hataları: {}", rollback_errors.join("; "))
                }
            ));
        }
    }
    if let Err(error) = atomic_write(Path::new(LOCK_FILE), &lock_json) {
        let mut rollback_errors = rollback_directory_swaps(&mut swaps);
        if project_changed {
            if let Err(restore_error) = atomic_write(Path::new(PROJECT_FILE), original_project) {
                rollback_errors.push(format!("huma.json geri yüklenemedi: {restore_error}"));
            }
        }
        return Err(anyhow!(
            "Kilit dosyası güncellenemedi: {error}{}",
            if rollback_errors.is_empty() {
                String::new()
            } else {
                format!("; geri alma hataları: {}", rollback_errors.join("; "))
            }
        ));
    }

    for swap in swaps {
        if let Some(backup) = swap.backup {
            if let Err(error) = fs::remove_dir_all(&backup) {
                eprintln!(
                    "Uyarı: eski paket yedeği temizlenemedi ({}): {}",
                    backup.display(),
                    error
                );
            }
        }
    }
    for package in &prepared {
        println!(
            "{} {} v{} [hash:{}] başarıyla kuruldu.",
            "Başarılı!".bright_green(),
            package.meta.ad.bold(),
            package.meta.surum.bright_white(),
            &package.hash[..8]
        );
    }
    Ok(())
}

fn copy_dir_contents(src: &Path, dst: &Path) -> Result<()> {
    fn copy_recursive(
        src: &Path,
        dst: &Path,
        depth: usize,
        file_count: &mut usize,
        total_bytes: &mut u64,
    ) -> Result<()> {
        if depth > 64 {
            return Err(anyhow!("Paket dizin derinliği 64 sınırını aşıyor."));
        }
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let entry_path = entry.path();
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                return Err(anyhow!(
                    "Paket kopyalanırken sembolik bağlantı reddedildi: {}",
                    entry_path.display()
                ));
            }
            let target_path = dst.join(entry.file_name());
            if file_type.is_dir() {
                let name = entry.file_name();
                if name != ".git" && name != "target" && name != "huma_modulleri" {
                    copy_recursive(
                        &entry_path,
                        &target_path,
                        depth + 1,
                        file_count,
                        total_bytes,
                    )?;
                }
            } else if file_type.is_file() {
                *file_count = file_count
                    .checked_add(1)
                    .ok_or_else(|| anyhow!("Paket dosya sayısı taştı."))?;
                if *file_count > MAX_PACKAGE_FILES {
                    return Err(anyhow!(
                        "Paket dosya sayısı {} sınırını aşıyor.",
                        MAX_PACKAGE_FILES
                    ));
                }
                let length = entry.metadata()?.len();
                if length > MAX_PACKAGE_FILE_BYTES as u64 {
                    return Err(anyhow!(
                        "Paket dosyası {} bayt sınırını aşıyor: {}",
                        MAX_PACKAGE_FILE_BYTES,
                        entry_path.display()
                    ));
                }
                *total_bytes = total_bytes
                    .checked_add(length)
                    .ok_or_else(|| anyhow!("Paket toplam boyutu taştı."))?;
                if *total_bytes > MAX_PACKAGE_TOTAL_BYTES {
                    return Err(anyhow!(
                        "Paket toplam boyutu {} bayt sınırını aşıyor.",
                        MAX_PACKAGE_TOTAL_BYTES
                    ));
                }
                fs::copy(&entry_path, &target_path)?;
            } else {
                return Err(anyhow!(
                    "Paket yalnızca normal dosya ve dizin içerebilir: {}",
                    entry_path.display()
                ));
            }
        }
        Ok(())
    }

    let mut file_count = 0usize;
    let mut total_bytes = 0u64;
    copy_recursive(src, dst, 0, &mut file_count, &mut total_bytes)
}

#[cfg(test)]
fn verify_dependency_version_at(package_root: &Path, name: &str, requirement: &str) -> Result<()> {
    sanitize_package_name(name)?;
    let requirement = VersionReq::parse(requirement)
        .with_context(|| format!("Geçersiz bağımlılık sürüm kısıtı: {name} -> '{requirement}'"))?;
    let package_path = package_root.join(name);
    let metadata_path = package_metadata_path(&package_path)?;
    let content = read_text_limited(
        &metadata_path,
        MAX_METADATA_BYTES,
        "Kurulu paket metadata'sı",
    )?;
    let metadata: PaketMetadata = serde_json::from_str(&content)
        .with_context(|| format!("Kurulu '{}' bağımlılığının metadata dosyası geçersiz", name))?;
    validate_package_metadata(&metadata, Some(name))?;
    let version = Version::parse(&metadata.surum)
        .with_context(|| format!("Kurulu '{}' paketinin sürümü geçersiz", name))?;
    if !requirement.matches(&version) {
        return Err(anyhow!(
            "Bağımlılık sürümü uyuşmuyor: '{}' için {} gerekiyor, {} kurulu.",
            name,
            requirement,
            version
        ));
    }
    Ok(())
}

fn validate_lock(lock: &PaketKilit) -> Result<()> {
    if lock.paketler.len() > MAX_DEPENDENCIES {
        return Err(anyhow!(
            "Kilitli paket sayısı {} sınırını aşıyor.",
            MAX_DEPENDENCIES
        ));
    }
    chrono::DateTime::parse_from_rfc3339(&lock.guncelleme_zamani)
        .with_context(|| "huma.lock güncelleme zamanı geçerli RFC 3339 değil")?;
    for (name, info) in &lock.paketler {
        sanitize_package_name(name)?;
        Version::parse(&info.surum)
            .with_context(|| format!("Kilitteki '{}' paket sürümü geçersiz", name))?;
        if info.hash.len() != 64
            || !info
                .hash
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(anyhow!(
                "Kilitteki '{}' paketi geçerli küçük harfli SHA-256 özeti taşımıyor.",
                name
            ));
        }
        if info
            .kaynak
            .as_deref()
            .is_none_or(|source| source.trim().is_empty() || source.len() > 4096)
        {
            return Err(anyhow!(
                "Kilitteki '{}' paketinin kaynak/provenans bilgisi geçersiz.",
                name
            ));
        }
    }
    Ok(())
}

fn read_lock_or_default(path: &Path) -> Result<PaketKilit> {
    if path.exists() {
        let content = read_text_limited(path, MAX_METADATA_BYTES, "Kilit dosyası")?;
        let lock = serde_json::from_str::<PaketKilit>(&content)
            .with_context(|| "Mevcut huma.lock geçersiz; kilit dosyası sessizce sıfırlanmadı")?;
        validate_lock(&lock)?;
        Ok(lock)
    } else {
        Ok(PaketKilit::default())
    }
}

/// Paketin yayınlanabilirliğini doğrular
pub fn verify_package() -> Result<()> {
    if !Path::new(PROJECT_FILE).exists() {
        return Err(anyhow!(
            "Bu dizinde bir Hüma projesi (huma.json) bulunamadı."
        ));
    }

    let meta_content = read_text_limited(
        Path::new(PROJECT_FILE),
        MAX_METADATA_BYTES,
        "Proje metadata'sı",
    )?;
    let meta: PaketMetadata =
        serde_json::from_str(&meta_content).with_context(|| "huma.json parse edilemedi")?;
    validate_package_metadata(&meta, None)?;
    let project_root = std::env::current_dir()?;
    let entry_path = project_root.join(&meta.giris);
    let entry_metadata = fs::symlink_metadata(&entry_path)
        .with_context(|| format!("Proje giriş dosyası bulunamadı: {}", entry_path.display()))?;
    if entry_metadata.file_type().is_symlink() || !entry_metadata.is_file() {
        return Err(anyhow!(
            "Proje giriş yolu sembolik bağlantı olmayan normal dosya olmalıdır: {}",
            entry_path.display()
        ));
    }
    if !entry_path
        .canonicalize()?
        .starts_with(project_root.canonicalize()?)
    {
        return Err(anyhow!(
            "Proje giriş dosyası proje dizininin dışına çıkıyor."
        ));
    }
    read_text_limited(&entry_path, MAX_PACKAGE_FILE_BYTES, "Proje giriş dosyası")?;

    let dependencies = meta.bagimliliklar.as_ref().cloned().unwrap_or_default();
    if !dependencies.is_empty() && !Path::new(LOCK_FILE).exists() {
        return Err(anyhow!(
            "Projenin bağımlılıkları var fakat huma.lock bulunamadı."
        ));
    }
    let lock = if Path::new(LOCK_FILE).exists() {
        let lock = read_lock_or_default(Path::new(LOCK_FILE))?;
        chrono::DateTime::parse_from_rfc3339(&lock.guncelleme_zamani)
            .with_context(|| "huma.lock güncelleme zamanı geçerli RFC 3339 değil")?;
        lock
    } else {
        PaketKilit::default()
    };

    for (dependency, requirement) in &dependencies {
        let lock_info = lock.paketler.get(dependency).ok_or_else(|| {
            anyhow!(
                "Bağımlılık '{}' huma.json içinde var fakat huma.lock içinde kilitlenmemiş.",
                dependency
            )
        })?;
        let requirement = VersionReq::parse(requirement)?;
        let version = Version::parse(&lock_info.surum)
            .with_context(|| format!("Kilitteki '{}' paket sürümü geçersiz", dependency))?;
        if !requirement.matches(&version) {
            return Err(anyhow!(
                "Kilitli bağımlılık sürümü uyuşmuyor: '{}' için {} gerekiyor, {} kilitli.",
                dependency,
                requirement,
                version
            ));
        }
    }

    let mut installed_metadata = HashMap::with_capacity(lock.paketler.len());
    for (package_name, lock_info) in &lock.paketler {
        sanitize_package_name(package_name)?;
        if lock_info.hash.len() != 64
            || !lock_info
                .hash
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(anyhow!(
                "Kilitteki '{}' paketi geçerli küçük harfli SHA-256 özeti taşımıyor.",
                package_name
            ));
        }
        if lock_info
            .kaynak
            .as_deref()
            .is_none_or(|source| source.trim().is_empty())
        {
            return Err(anyhow!(
                "Kilitteki '{}' paketinin kaynak/provenans bilgisi eksik.",
                package_name
            ));
        }
        let package_dir = PathBuf::from(PACKAGE_DIR).join(package_name);
        let metadata_path = package_metadata_path(&package_dir)?;
        let metadata_content =
            read_text_limited(&metadata_path, MAX_METADATA_BYTES, "Paket metadata'sı")?;
        let package_meta: PaketMetadata =
            serde_json::from_str(&metadata_content).with_context(|| {
                format!(
                    "Paket metadata'sı parse edilemedi: {}",
                    metadata_path.display()
                )
            })?;
        validate_package_metadata(&package_meta, Some(package_name))?;
        validate_package_tree(&package_dir, &package_meta)?;
        let computed_hash = calculate_package_hash(&package_dir, &package_meta)?;
        if lock_info.hash != computed_hash {
            return Err(anyhow!(
                "Bütünlük hatası: '{}' paketinin özeti uyuşmuyor (lock: {}, hesaplanan: {}).",
                package_name,
                &lock_info.hash[..16],
                &computed_hash[..16]
            ));
        }
        let signature_on_disk = read_package_signature(&package_dir)?;
        match (&lock_info.imza, signature_on_disk) {
            (Some(locked), Some(on_disk)) => {
                if locked != &on_disk {
                    return Err(anyhow!(
                        "Bütünlük hatası: '{}' paketinin imza zarfı kilitle uyuşmuyor.",
                        package_name
                    ));
                }
                verify_package_signature(&package_meta, &computed_hash, &on_disk).with_context(
                    || format!("'{}' paketinin Ed25519 imzası geçersiz", package_name),
                )?;
            }
            (Some(_), None) => {
                return Err(anyhow!(
                    "Bütünlük hatası: '{}' paketinin kilitli imza zarfı kayıp.",
                    package_name
                ));
            }
            (None, Some(_)) => {
                return Err(anyhow!(
                    "Bütünlük hatası: '{}' paketinde kilitlenmemiş imza zarfı var.",
                    package_name
                ));
            }
            (None, None) => {
                if lock_info.kaynak.as_deref() != Some("yerel-imzasız-güvenilir") {
                    return Err(anyhow!(
                        "Paket '{}' doğrulanmış Ed25519 imzası veya açık yerel-imzasız provenansı taşımıyor.",
                        package_name
                    ));
                }
            }
        }
        if lock_info.surum != package_meta.surum {
            return Err(anyhow!(
                "Sürüm uyuşmazlığı: lock '{}' için {} diyor, paket metadata {}.",
                package_name,
                lock_info.surum,
                package_meta.surum
            ));
        }
        installed_metadata.insert(package_name.clone(), package_meta);
    }

    for (package_name, package_meta) in &installed_metadata {
        for (dependency, requirement) in package_meta.bagimliliklar.as_ref().into_iter().flatten() {
            let dependency_meta = installed_metadata.get(dependency).ok_or_else(|| {
                anyhow!(
                    "'{}' paketinin '{}' bağımlılığı kilitli ve kurulu değil.",
                    package_name,
                    dependency
                )
            })?;
            let requirement = VersionReq::parse(requirement)?;
            let version = Version::parse(&dependency_meta.surum)?;
            if !requirement.matches(&version) {
                return Err(anyhow!(
                    "'{}' paketi '{}' için {} gerektiriyor; {} kurulu.",
                    package_name,
                    dependency,
                    requirement,
                    version
                ));
            }
        }
    }

    if let Some(signature) = read_package_signature(&project_root)? {
        let project_hash = calculate_package_hash(&project_root, &meta)?;
        verify_package_signature(&meta, &project_hash, &signature)
            .with_context(|| "Proje Ed25519 imzası geçersiz")?;
    }

    println!(
        "{} Paket '{}' v{} için yerel manifest ve dosya denetimleri geçti.",
        "Doğrulandı:".bright_green(),
        meta.ad,
        meta.surum
    );
    Ok(())
}

pub fn remove_package(name: &str) -> Result<()> {
    sanitize_package_name(name)?;
    if !Path::new(PROJECT_FILE).is_file() || !Path::new(LOCK_FILE).is_file() {
        return Err(anyhow!(
            "Paket kaldırmak için geçerli huma.json ve huma.lock gerekir."
        ));
    }
    let original_project = read_bytes_limited(
        Path::new(PROJECT_FILE),
        MAX_METADATA_BYTES,
        "Proje metadata'sı",
    )?;
    let original_lock =
        read_bytes_limited(Path::new(LOCK_FILE), MAX_METADATA_BYTES, "Kilit dosyası")?;
    let mut project: PaketMetadata =
        serde_json::from_slice(&original_project).with_context(|| "huma.json parse edilemedi")?;
    validate_package_metadata(&project, None)?;
    let mut lock: PaketKilit =
        serde_json::from_slice(&original_lock).with_context(|| "huma.lock parse edilemedi")?;
    validate_lock(&lock)?;
    if !lock.paketler.contains_key(name) {
        return Err(anyhow!("Paket huma.lock içinde bulunamadı: '{}'.", name));
    }

    for dependent in lock
        .paketler
        .keys()
        .filter(|package| package.as_str() != name)
    {
        let dependent_dir = PathBuf::from(PACKAGE_DIR).join(dependent);
        let metadata_path = package_metadata_path(&dependent_dir)?;
        let metadata_text =
            read_text_limited(&metadata_path, MAX_METADATA_BYTES, "Paket metadata'sı")?;
        let metadata: PaketMetadata = serde_json::from_str(&metadata_text)?;
        validate_package_metadata(&metadata, Some(dependent))?;
        if metadata
            .bagimliliklar
            .as_ref()
            .is_some_and(|dependencies| dependencies.contains_key(name))
        {
            return Err(anyhow!(
                "'{}' paketi '{}' paketine bağımlı; önce bağımlı paketi kaldırın.",
                dependent,
                name
            ));
        }
    }

    let package_root = PathBuf::from(PACKAGE_DIR);
    let target = package_root.join(name);
    let target_metadata = fs::symlink_metadata(&target)
        .with_context(|| format!("Kurulu paket dizini bulunamadı: {}", target.display()))?;
    if target_metadata.file_type().is_symlink() || !target_metadata.is_dir() {
        return Err(anyhow!(
            "Kurulu paket hedefi sembolik bağlantı olmayan dizin olmalıdır: {}",
            target.display()
        ));
    }
    let canonical_root = package_root.canonicalize()?;
    let canonical_target = target.canonicalize()?;
    if canonical_target == canonical_root || !canonical_target.starts_with(&canonical_root) {
        return Err(anyhow!(
            "Paket kaldırma hedefi güvenli sınırın dışında: {}",
            target.display()
        ));
    }

    if let Some(dependencies) = &mut project.bagimliliklar {
        dependencies.remove(name);
    }
    lock.paketler.remove(name);
    lock.guncelleme_zamani = chrono::Local::now().to_rfc3339();
    let project_json = canonical_json_pretty_bytes(&project)?;
    let lock_json = canonical_json_pretty_bytes(&lock)?;
    if project_json.len() > MAX_METADATA_BYTES || lock_json.len() > MAX_METADATA_BYTES {
        return Err(anyhow!(
            "Proje veya kilit metadata'sı {} bayt sınırını aşıyor.",
            MAX_METADATA_BYTES
        ));
    }

    let backup = unique_sibling(&target, "remove")?;
    fs::rename(&target, &backup).with_context(|| {
        format!(
            "Paket güvenli kaldırma alanına taşınamadı: {}",
            target.display()
        )
    })?;
    if let Err(error) = atomic_write(Path::new(PROJECT_FILE), &project_json) {
        let restore = fs::rename(&backup, &target);
        return Err(anyhow!(
            "huma.json güncellenemedi: {error}{}",
            restore
                .err()
                .map(|restore_error| format!(
                    "; paket geri yüklenemedi: {restore_error} (yedek: {})",
                    backup.display()
                ))
                .unwrap_or_default()
        ));
    }
    if let Err(error) = atomic_write(Path::new(LOCK_FILE), &lock_json) {
        let mut restore_errors = Vec::new();
        if let Err(restore_error) = atomic_write(Path::new(PROJECT_FILE), &original_project) {
            restore_errors.push(format!("huma.json geri yüklenemedi: {restore_error}"));
        }
        if let Err(restore_error) = fs::rename(&backup, &target) {
            restore_errors.push(format!(
                "paket geri yüklenemedi: {restore_error} (yedek: {})",
                backup.display()
            ));
        }
        return Err(anyhow!(
            "huma.lock güncellenemedi: {error}{}",
            if restore_errors.is_empty() {
                String::new()
            } else {
                format!("; {}", restore_errors.join("; "))
            }
        ));
    }
    if let Err(error) = fs::remove_dir_all(&backup) {
        eprintln!(
            "Uyarı: paket kaldırıldı fakat geri alma yedeği temizlenemedi ({}): {}",
            backup.display(),
            error
        );
    }
    println!("{} {} silindi.", "Başarılı!".bright_green(), name.bold());
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
    betikler.insert(
        "baslat".to_string(),
        format!("huma run {}.hb", default_name),
    );
    betikler.insert("start".to_string(), format!("huma run {}.hb", default_name));
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
        hmi: None,
        kaynak: None,
        github: None,
        lisans: Some("MIT".to_string()),
    };
    validate_package_metadata(&meta, Some(&default_name)).with_context(|| {
        format!(
            "Geçerli proje adı mevcut dizin adından üretilemedi: '{}'. \
             Dizini yeniden adlandırın veya 'huma yeni <ad>' kullanın.",
            default_name
        )
    })?;
    let initial_lock = PaketKilit {
        paketler: HashMap::new(),
        guncelleme_zamani: chrono::Local::now().to_rfc3339(),
    };
    let initial_lock_json = canonical_json_pretty_string(&initial_lock)?;
    let meta_json = canonical_json_pretty_string(&meta)?;

    let hb_file = format!("{}.hb", default_name);
    let entry_created = !Path::new(&hb_file).exists();
    if !Path::new(&hb_file).exists() {
        let content = format!(
            "// {} ana giriş dosyası\n\"Hüma projesi aktif.\"'ı yazdır",
            default_name
        );
        atomic_write_str(Path::new(&hb_file), &content)?;
    }
    let package_dir_created = !Path::new(PACKAGE_DIR).exists();
    if let Err(error) = fs::create_dir_all(PACKAGE_DIR) {
        if entry_created {
            let _ = fs::remove_file(&hb_file);
        }
        return Err(error.into());
    }
    let lock_created = !Path::new(LOCK_FILE).exists();
    if lock_created {
        if let Err(error) = atomic_write_str(Path::new(LOCK_FILE), &initial_lock_json) {
            if entry_created {
                let _ = fs::remove_file(&hb_file);
            }
            if package_dir_created {
                let _ = fs::remove_dir(PACKAGE_DIR);
            }
            return Err(error);
        }
    } else {
        read_lock_or_default(Path::new(LOCK_FILE))?;
    }
    // `huma.json` en son yazılır; varlığı başarılı ilklemeyi temsil eder.
    if let Err(error) = atomic_write_str(Path::new(PROJECT_FILE), &meta_json) {
        if entry_created {
            let _ = fs::remove_file(&hb_file);
        }
        if lock_created {
            let _ = fs::remove_file(LOCK_FILE);
        }
        if package_dir_created {
            let _ = fs::remove_dir(PACKAGE_DIR);
        }
        return Err(error);
    }

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
    let s = read_text_limited(
        Path::new(PROJECT_FILE),
        MAX_METADATA_BYTES,
        "Proje metadata'sı",
    )?;
    let meta: PaketMetadata = serde_json::from_str(&s)?;
    validate_package_metadata(&meta, None)?;
    Ok(meta)
}

/// Bir Hüma programının yalnız paket yöneticisi betiği içinden ve paket
/// kökünün dışına taşmadan çalıştırıldığını doğrular.
pub fn require_managed_run(target: &Path) -> Result<()> {
    let project_root = std::env::current_dir()?
        .canonicalize()
        .with_context(|| "Hüma proje kökü çözümlenemedi")?;
    let managed_root = std::env::var_os(MANAGED_RUN_ROOT_ENV).ok_or_else(|| {
        anyhow!(
            "Hüma programları yalnız paket yöneticisiyle çalıştırılır. Önce huma.json içinde bir betik tanımlayın, ardından 'huma paket run <betik>' kullanın."
        )
    })?;
    if Path::new(&managed_root) != project_root {
        return Err(anyhow!(
            "Paket yöneticisi çalışma kökü mevcut proje köküyle uyuşmuyor."
        ));
    }

    let metadata = get_local_metadata()?;
    if !Path::new(LOCK_FILE).is_file() {
        return Err(anyhow!(
            "Yönetilen çalıştırma için huma.lock zorunludur. 'huma paket kur' çalıştırın."
        ));
    }
    let lock = read_lock_or_default(Path::new(LOCK_FILE))?;
    for (dependency, requirement) in metadata.bagimliliklar.as_ref().into_iter().flatten() {
        let locked = lock.paketler.get(dependency).ok_or_else(|| {
            anyhow!(
                "Bağımlılık '{}' manifestte var fakat huma.lock içinde kilitli değil.",
                dependency
            )
        })?;
        let requirement = VersionReq::parse(requirement)?;
        let version = Version::parse(&locked.surum)?;
        if !requirement.matches(&version) {
            return Err(anyhow!(
                "Bağımlılık '{}' için kilitli {} sürümü {} kısıtını karşılamıyor.",
                dependency,
                version,
                requirement
            ));
        }
    }

    let candidate = if target.is_absolute() {
        target.to_path_buf()
    } else {
        project_root.join(target)
    };
    let canonical_target = candidate
        .canonicalize()
        .with_context(|| format!("Program yolu bulunamadı: {}", target.display()))?;
    if !canonical_target.starts_with(&project_root) {
        return Err(anyhow!(
            "Paket betiği proje kökünün dışındaki bir programı çalıştıramaz: {}",
            target.display()
        ));
    }
    Ok(())
}

/// Proje metadata'sında adı verilen bir betik bulunup bulunmadığını döndürür.
///
/// Metadata'nın bulunmaması veya geçersiz olması `false` değil hatadır; böylece
/// çağıran gerçek proje hatasını "betik yok" diye örtemez.
pub fn has_local_script(name: &str) -> Result<bool> {
    Ok(get_local_metadata()?
        .betikler
        .as_ref()
        .is_some_and(|scripts| scripts.contains_key(name)))
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

            let arguments = shell_words::split(komut)
                .with_context(|| format!("'{}' betiğinin argümanları ayrıştırılamadı", name))?;
            let (program, arguments) = arguments
                .split_first()
                .ok_or_else(|| anyhow!("'{}' betiğinin komutu boş.", name))?;
            let mut process = if program == "huma" {
                std::process::Command::new(
                    std::env::current_exe()
                        .with_context(|| "Çalışan Hüma yürütülebilirinin yolu bulunamadı")?,
                )
            } else {
                std::process::Command::new(program)
            };
            if program == "huma" {
                process.env(
                    MANAGED_RUN_ROOT_ENV,
                    std::env::current_dir()?
                        .canonicalize()
                        .with_context(|| "Paket betiği proje kökü çözümlenemedi")?,
                );
            } else {
                process.env_remove(MANAGED_RUN_ROOT_ENV);
            }
            let mut child = process
                .args(arguments)
                .spawn()
                .with_context(|| format!("'{}' betiği başlatılamadı", name))?;
            let status = match child.wait_timeout(Duration::from_secs(300))? {
                Some(status) => status,
                None => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(anyhow!("Betik '{}' 300 saniyede tamamlanmadı.", name));
                }
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
        calculate_package_hash, check_api_compatibility, check_native_code_safety,
        create_package_signature, read_lock_or_default, rollback_directory_swaps,
        sanitize_package_name, sign_distribution_artifact, uzak_paket_girdisi_mi,
        validate_package_metadata, verify_dependency_version_at, verify_distribution_artifact,
        verify_package_signature, DirectorySwap, PaketMetadata, SIGNATURE_FILE,
    };
    use ed25519_dalek::SigningKey;
    use std::collections::HashMap;
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

    #[test]
    fn bagimlilik_surumu_gercek_kurulu_surume_uygulanir() {
        let benzersiz = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Sistem saati Unix epoch sonrasında olmalı")
            .as_nanos();
        let kok = std::env::temp_dir().join(format!(
            "huma_paket_surumu_test_{}_{}",
            std::process::id(),
            benzersiz
        ));
        let paket = kok.join("ornek");
        fs::create_dir_all(&paket).expect("Geçici paket dizini oluşturulmalı");
        fs::write(
            paket.join("paket.json"),
            r#"{
                "ad": "ornek",
                "surum": "1.2.3",
                "aciklama": "test",
                "yazar": "test",
                "giris": "ana.hb"
            }"#,
        )
        .expect("Paket metadata'sı yazılmalı");

        verify_dependency_version_at(&kok, "ornek", "^1.0")
            .expect("Uyumlu bağımlılık sürümü kabul edilmeli");
        assert!(verify_dependency_version_at(&kok, "ornek", "^2.0").is_err());
        fs::remove_dir_all(&kok).expect("Geçici paket dizini temizlenmeli");
    }

    #[test]
    fn bozuk_kilit_dosyasi_sessizce_sifirlanmaz() {
        let benzersiz = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Sistem saati Unix epoch sonrasında olmalı")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "huma_bozuk_kilit_test_{}_{}.lock",
            std::process::id(),
            benzersiz
        ));
        fs::write(&path, "{geçersiz").expect("Bozuk kilit dosyası yazılmalı");
        let error = read_lock_or_default(&path).expect_err("Bozuk kilit reddedilmeli");
        assert!(error.to_string().contains("sessizce sıfırlanmadı"));
        fs::remove_file(path).expect("Geçici kilit dosyası temizlenmeli");
    }

    #[test]
    fn dizin_degisimleri_hata_enjeksiyonunda_onceki_surumu_geri_yukler() {
        let benzersiz = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Sistem saati Unix epoch sonrasında olmalı")
            .as_nanos();
        let kok = std::env::temp_dir().join(format!(
            "huma_rollback_test_{}_{}",
            std::process::id(),
            benzersiz
        ));
        let target = kok.join("paket");
        let backup = kok.join("paket.backup");
        fs::create_dir_all(&target).expect("Yeni paket dizini oluşturulmalı");
        fs::create_dir_all(&backup).expect("Yedek paket dizini oluşturulmalı");
        fs::write(target.join("surum"), "yeni").expect("Yeni sürüm yazılmalı");
        fs::write(backup.join("surum"), "eski").expect("Eski sürüm yazılmalı");

        let mut swaps = vec![DirectorySwap {
            target: target.clone(),
            backup: Some(backup.clone()),
        }];
        let errors = rollback_directory_swaps(&mut swaps);

        assert!(errors.is_empty(), "geri alma hataları: {errors:?}");
        assert_eq!(fs::read_to_string(target.join("surum")).unwrap(), "eski");
        assert!(!backup.exists());
        assert!(swaps.is_empty());
        fs::remove_dir_all(kok).expect("Geçici rollback dizini temizlenmeli");
    }

    #[test]
    fn metadata_semver_giris_ve_native_abi_sozlesmelerini_dogrular() {
        let mut meta: PaketMetadata = serde_json::from_value(serde_json::json!({
            "ad": "ornek",
            "surum": "1.0.0",
            "aciklama": "test",
            "yazar": "test",
            "giris": "ana.hb"
        }))
        .expect("Test metadata'sı geçerli olmalı");
        assert!(validate_package_metadata(&meta, Some("ornek")).is_ok());

        meta.surum = "sürüm-değil".to_string();
        assert!(validate_package_metadata(&meta, Some("ornek")).is_err());
        meta.surum = "1.0.0".to_string();
        meta.giris = "ana.txt".to_string();
        assert!(validate_package_metadata(&meta, Some("ornek")).is_err());
        meta.giris = "ana.hb".to_string();
        meta.yerleşik_rust = Some("fn tehlikeli() {}".to_string());
        assert!(check_native_code_safety(&meta, true).is_err());
    }

    #[test]
    fn paket_ozeti_harita_ekleme_sirasindan_bagimsizdir() {
        let benzersiz = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Sistem saati Unix epoch sonrasında olmalı")
            .as_nanos();
        let kok = std::env::temp_dir().join(format!(
            "huma_kanonik_hash_test_{}_{}",
            std::process::id(),
            benzersiz
        ));
        fs::create_dir_all(&kok).expect("Geçici paket dizini oluşturulmalı");
        fs::write(kok.join("ana.hb"), "\"tamam\"'ı yazdır").expect("Giriş yazılmalı");

        let mut first_dependencies = HashMap::new();
        first_dependencies.insert("a".to_string(), "^1".to_string());
        first_dependencies.insert("b".to_string(), "^2".to_string());
        let mut second_dependencies = HashMap::new();
        second_dependencies.insert("b".to_string(), "^2".to_string());
        second_dependencies.insert("a".to_string(), "^1".to_string());
        let base = |dependencies| PaketMetadata {
            ad: "ornek".to_string(),
            surum: "1.0.0".to_string(),
            aciklama: "test".to_string(),
            yazar: "test".to_string(),
            giris: "ana.hb".to_string(),
            huma_surum: None,
            bagimliliklar: Some(dependencies),
            betikler: None,
            crate_bagimliliklari: None,
            yerleşik_rust: None,
            hmi: None,
            kaynak: None,
            github: None,
            lisans: None,
        };
        let first =
            calculate_package_hash(&kok, &base(first_dependencies)).expect("İlk özet hesaplanmalı");
        let second = calculate_package_hash(&kok, &base(second_dependencies))
            .expect("İkinci özet hesaplanmalı");
        fs::remove_dir_all(&kok).expect("Geçici paket dizini temizlenmeli");
        assert_eq!(first, second);
    }

    #[test]
    fn ed25519_imzasi_paket_kimligi_ve_ozetine_baglanir() {
        let benzersiz = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Sistem saati Unix epoch sonrasında olmalı")
            .as_nanos();
        let kok = std::env::temp_dir().join(format!(
            "huma_imza_test_{}_{}",
            std::process::id(),
            benzersiz
        ));
        fs::create_dir_all(&kok).expect("Geçici paket dizini oluşturulmalı");
        fs::write(kok.join("ana.hb"), "\"tamam\"'ı yazdır").expect("Giriş yazılmalı");
        let meta: PaketMetadata = serde_json::from_value(serde_json::json!({
            "ad": "ornek",
            "surum": "1.0.0",
            "aciklama": "test",
            "yazar": "test",
            "giris": "ana.hb"
        }))
        .expect("Test metadata'sı geçerli olmalı");
        let hash = calculate_package_hash(&kok, &meta).expect("Özet hesaplanmalı");
        let key = SigningKey::from_bytes(&[7_u8; 32]);
        let envelope = create_package_signature(&meta, &hash, &key).expect("Paket imzalanmalı");
        verify_package_signature(&meta, &hash, &envelope).expect("İmza doğrulanmalı");

        fs::write(
            kok.join(SIGNATURE_FILE),
            serde_json::to_vec(&envelope).expect("Zarf serileştirilmeli"),
        )
        .expect("Zarf yazılmalı");
        assert_eq!(
            calculate_package_hash(&kok, &meta).expect("İmza sonrası özet hesaplanmalı"),
            hash,
            "imza zarfı kendi imzaladığı özeti değiştirmemeli"
        );

        let mut tampered = envelope;
        tampered.surum = "1.0.1".to_string();
        assert!(verify_package_signature(&meta, &hash, &tampered).is_err());
        fs::remove_dir_all(&kok).expect("Geçici paket dizini temizlenmeli");
    }

    #[test]
    fn kirici_hmi_degisiminde_ana_surum_artisi_zorunludur() {
        let benzersiz = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Sistem saati Unix epoch sonrasında olmalı")
            .as_nanos();
        let kok = std::env::temp_dir().join(format!(
            "huma_api_test_{}_{}",
            std::process::id(),
            benzersiz
        ));
        fs::create_dir_all(&kok).unwrap();
        let metadata = |version: &str, return_type: &str| {
            serde_json::json!({
                "ad": "ornek",
                "surum": version,
                "aciklama": "test",
                "yazar": "test",
                "giris": "ana.hb",
                "hmi": {
                    "protocol": { "major": 1, "minor": 0 },
                    "transport": "stdio-json-v1",
                    "executable": "ornek-host",
                    "interface": {
                        "schema_version": 1,
                        "module": "ornek",
                        "huma_version_requirement": ">=0.6,<1.0",
                        "functions": [{
                            "name": "topla",
                            "parameters": [{
                                "name": "deger",
                                "value_type": "number"
                            }],
                            "return_type": return_type
                        }]
                    }
                }
            })
        };
        let previous = kok.join("onceki.json");
        let minor = kok.join("minor.json");
        let major = kok.join("major.json");
        fs::write(
            &previous,
            serde_json::to_vec(&metadata("1.0.0", "number")).unwrap(),
        )
        .unwrap();
        fs::write(
            &minor,
            serde_json::to_vec(&metadata("1.1.0", "text")).unwrap(),
        )
        .unwrap();
        fs::write(
            &major,
            serde_json::to_vec(&metadata("2.0.0", "text")).unwrap(),
        )
        .unwrap();

        assert!(check_api_compatibility(&previous, &minor).is_err());
        check_api_compatibility(&previous, &major)
            .expect("ana sürüm artışı kırıcı HMI değişikliğini kapsamalı");
        fs::remove_dir_all(kok).unwrap();
    }

    #[test]
    fn dagitim_imzasi_dosya_adini_ve_icerigini_baglar() {
        let benzersiz = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Sistem saati Unix epoch sonrasında olmalı")
            .as_nanos();
        let kok = std::env::temp_dir().join(format!(
            "huma_dagitim_imza_test_{}_{}",
            std::process::id(),
            benzersiz
        ));
        fs::create_dir_all(&kok).unwrap();
        let artifact = kok.join("SHA256SUMS");
        let key = kok.join("release.key");
        let signature = kok.join("SHA256SUMS.sig");
        fs::write(&artifact, "abc  huma-linux\n").unwrap();
        fs::write(&key, hex::encode([9_u8; 32])).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&key, fs::Permissions::from_mode(0o600)).unwrap();
        }

        sign_distribution_artifact(&artifact, &key, &signature).unwrap();
        verify_distribution_artifact(&artifact, &signature).unwrap();
        fs::write(&artifact, "değiştirildi\n").unwrap();
        assert!(verify_distribution_artifact(&artifact, &signature).is_err());
        fs::remove_dir_all(kok).unwrap();
    }
}
