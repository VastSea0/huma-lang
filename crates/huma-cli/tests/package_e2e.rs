use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn run_huma(project: &PathBuf, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_huma"))
        .args(arguments)
        .current_dir(project)
        .output()
        .expect("Hüma CLI başlatılmalı")
}

fn assert_success(output: &Output, operation: &str) {
    assert!(
        output.status.success(),
        "{operation} başarısız:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn gecisli_paket_kurulum_dogrulama_ve_kaldirma_uctan_uca_calisir() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Sistem saati Unix epoch sonrasında olmalı")
        .as_nanos();
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("Workspace kökü bulunmalı")
        .to_path_buf();
    let project =
        workspace
            .join("target")
            .join(format!("huma_package_e2e_{}_{}", std::process::id(), nonce));
    fs::create_dir_all(&project).expect("Geçici proje dizini oluşturulmalı");

    let initialize = run_huma(&project, &["ilkle"]);
    assert_success(&initialize, "Proje ilkleme");

    let unsigned = run_huma(&project, &["kur", "nlp_ileri"]);
    assert!(
        !unsigned.status.success(),
        "İmzasız paket açık güven onayı olmadan kurulmamalı"
    );
    assert!(String::from_utf8_lossy(&unsigned.stderr).contains("Ed25519 imzası taşımıyor"));

    let install = run_huma(&project, &["kur", "nlp_ileri", "--güvenilir"]);
    assert_success(&install, "Geçişli paket kurulumu");
    assert!(project.join("huma_modulleri/nlp_temel").is_dir());
    assert!(project.join("huma_modulleri/nlp_ileri").is_dir());

    let lock: Value = serde_json::from_slice(
        &fs::read(project.join("huma.lock")).expect("Kilit dosyası okunmalı"),
    )
    .expect("Kilit dosyası geçerli JSON olmalı");
    let packages = lock["paketler"]
        .as_object()
        .expect("Kilit paket haritası taşımalı");
    assert!(packages.contains_key("nlp_temel"));
    assert!(packages.contains_key("nlp_ileri"));
    assert_eq!(packages["nlp_ileri"]["kaynak"], "yerel-imzasız-güvenilir");

    let verify = run_huma(&project, &["paket", "doğrula"]);
    assert_success(&verify, "Paket doğrulama");

    let dependent_remove = run_huma(&project, &["paket", "sil", "nlp_temel"]);
    assert!(
        !dependent_remove.status.success(),
        "Kullanımdaki geçişli bağımlılık kaldırılamamalı"
    );
    assert!(String::from_utf8_lossy(&dependent_remove.stderr).contains("bağımlı"));

    assert_success(
        &run_huma(&project, &["paket", "sil", "nlp_ileri"]),
        "Kök paket kaldırma",
    );
    assert_success(
        &run_huma(&project, &["paket", "sil", "nlp_temel"]),
        "Artık kullanılmayan bağımlılığı kaldırma",
    );

    fs::remove_dir_all(&project).expect("Geçici proje temizlenmeli");
}

#[test]
fn run_komutu_bulunan_betigin_calisma_hatasini_ortmez() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Sistem saati Unix epoch sonrasında olmalı")
        .as_nanos();
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("Workspace kökü bulunmalı")
        .to_path_buf();
    let project =
        workspace
            .join("target")
            .join(format!("huma_script_e2e_{}_{}", std::process::id(), nonce));
    fs::create_dir_all(&project).expect("Geçici proje dizini oluşturulmalı");
    assert_success(&run_huma(&project, &["ilkle"]), "Proje ilkleme");

    let manifest_path = project.join("huma.json");
    let mut manifest: Value =
        serde_json::from_slice(&fs::read(&manifest_path).expect("Manifest okunmalı"))
            .expect("Manifest geçerli JSON olmalı");
    manifest["betikler"]["hata"] = Value::String("huma run bulunmayan.hb".to_string());
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).expect("Manifest serileştirilmeli"),
    )
    .expect("Manifest yazılmalı");

    let output = run_huma(&project, &["run", "hata"]);
    assert!(
        !output.status.success(),
        "Başarısız betik başarı sayılmamalı"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Betik 'hata' hata ile sonlandı"),
        "Gerçek betik hatası korunmalı:\n{stderr}"
    );
    assert!(!stderr.contains("'hata' adlı dosya veya proje betiği bulunamadı"));

    fs::remove_dir_all(&project).expect("Geçici proje temizlenmeli");
}

#[test]
fn program_yalniz_paket_yoneticisi_betigiyle_calistirilir() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Sistem saati Unix epoch sonrasında olmalı")
        .as_nanos();
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("Workspace kökü bulunmalı")
        .to_path_buf();
    let project = workspace.join("target").join(format!(
        "huma_managed_run_e2e_{}_{}",
        std::process::id(),
        nonce
    ));
    fs::create_dir_all(&project).expect("Geçici proje dizini oluşturulmalı");
    assert_success(&run_huma(&project, &["ilkle"]), "Proje ilkleme");

    let entry = project
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("{name}.hb"))
        .expect("Giriş adı üretilebilmeli");
    let direct = run_huma(&project, &["run", &entry]);
    assert!(
        !direct.status.success(),
        "Doğrudan dosya çalıştırma kapanmalı"
    );
    assert!(
        String::from_utf8_lossy(&direct.stderr).contains("yalnız paket yöneticisiyle"),
        "Hata paket yöneticisi kullanımını açıklamalı"
    );

    assert_success(
        &run_huma(&project, &["paket", "run", "baslat"]),
        "Yönetilen paket betiği",
    );
    fs::remove_dir_all(&project).expect("Geçici proje temizlenmeli");
}
