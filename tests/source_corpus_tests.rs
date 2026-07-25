use huma_core::lexer::Lexer;
use huma_core::parser::Parser;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace kökü bulunmalı")
        .to_path_buf()
}

#[test]
fn izlenen_huma_kaynaklarinin_tumu_ayristirilabilir() {
    let root = workspace_root();
    let output = Command::new("git")
        .args(["ls-files", "*.hb"])
        .current_dir(&root)
        .output()
        .expect("izlenen Hüma kaynakları git ile listelenebilmeli");
    assert!(
        output.status.success(),
        "git ls-files başarısız: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let paths = String::from_utf8(output.stdout).expect("git çıktısı UTF-8 olmalı");
    let mut failures = Vec::new();
    for relative in paths.lines().filter(|line| !line.is_empty()) {
        let path = root.join(relative);
        if !path.exists() {
            continue;
        }
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} okunamadı: {}", path.display(), error));
        let mut parser = Parser::new(Lexer::new(&source));
        let (_, diagnostics) = parser.parse_program_with_diagnostics();
        for diagnostic in diagnostics {
            failures.push(format!("{}: {}", relative, diagnostic));
        }
    }

    assert!(
        failures.is_empty(),
        "Ayrıştırılamayan Hüma kaynakları:\n{}",
        failures.join("\n")
    );
}

#[derive(Deserialize)]
struct PaketMetadata {
    ad: String,
    surum: String,
    aciklama: String,
    yazar: String,
    giris: String,
}

#[test]
fn yerlesik_paket_metalari_gecerli_ve_girisleri_mevcut() {
    let modules = workspace_root().join("huma_modulleri");
    for entry in std::fs::read_dir(&modules).expect("huma_modulleri okunmalı") {
        let path = entry.expect("modül girdisi okunmalı").path();
        if !path.is_dir() {
            continue;
        }
        let metadata_path = path.join("huma.json");
        if !metadata_path.exists() {
            continue;
        }
        let source = std::fs::read_to_string(&metadata_path)
            .unwrap_or_else(|error| panic!("{} okunamadı: {}", metadata_path.display(), error));
        let metadata: PaketMetadata = serde_json::from_str(&source)
            .unwrap_or_else(|error| panic!("{} geçersiz: {}", metadata_path.display(), error));
        assert!(!metadata.ad.trim().is_empty());
        assert!(!metadata.surum.trim().is_empty());
        assert!(!metadata.aciklama.trim().is_empty());
        assert!(!metadata.yazar.trim().is_empty());
        assert!(
            path.join(&metadata.giris).is_file(),
            "{} giriş dosyası bulunamadı",
            metadata_path.display()
        );
    }
}
