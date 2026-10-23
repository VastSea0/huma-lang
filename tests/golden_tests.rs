//! Davranış (golden) testleri: `tests/golden/<ad>.hb` programı yorumlayıcıda
//! ve bytecode VM'de ayrı ayrı çalıştırılır; her ikisinin çıktısı aynı
//! beklenen sonuçla karşılaştırılır.
//!
//! Her program için tam olarak biri bulunmalıdır:
//! - `<ad>.beklenen`: başarılı çalışmanın tam standart çıktısı.
//! - `<ad>.hata`: programın verdiği hatanın tam metni (öncesindeki çıktı
//!   varsa `<ad>.beklenen` ile birlikte de kullanılabilir).
//!
//! VM'in henüz desteklemediği bir yapı kullanan program ilk satırlarında
//! `// golden: yalnız-yorumlayıcı <gerekçe>` yazmalıdır. Gerekçesiz dışlama
//! reddedilir; böylece VM kapsamı bu dosyalardan ölçülebilir.

use huma_core as huma;

use huma::interpreter::Yorumlayici;
use huma::lexer::Lexer;
use huma::parser::Parser;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace kökü bulunmalı")
        .to_path_buf()
}

/// Çalıştırma sonucu: standart çıktı ve (varsa) hata metni.
#[derive(Debug, PartialEq)]
struct Sonuc {
    cikti: String,
    hata: Option<String>,
}

fn yorumlayicida_calistir(kaynak: &str, kok: &Path) -> Sonuc {
    let _yetenekler = huma::capability::install(huma::capability::CapabilitySet::deny_all())
        .expect("Test yetenekleri kurulmalı");
    let buffer = Rc::new(RefCell::new(String::new()));
    let mut yorumlayici = Yorumlayici::new().with_output_buffer(Rc::clone(&buffer));
    yorumlayici
        .arama_yolları
        .insert(0, kok.join("lib").to_string_lossy().to_string());
    let mut parser = Parser::new(Lexer::new(kaynak));
    let (program, tanilar) = parser.parse_program_with_diagnostics();
    let hata = match tanilar.into_iter().next() {
        Some(tani) => Some(tani.to_string()),
        None => yorumlayici
            .yorumla_kontrollu(program)
            .err()
            .map(|hata| hata.to_string()),
    };
    let cikti = buffer.borrow().clone();
    Sonuc { cikti, hata }
}

fn vm_de_calistir(kaynak: &str) -> Sonuc {
    let _yetenekler = huma::capability::install(huma::capability::CapabilitySet::deny_all())
        .expect("Test yetenekleri kurulmalı");
    let mut parser = Parser::new(Lexer::new(kaynak));
    let (ast, tanilar) = parser.parse_program_with_diagnostics();
    if let Some(tani) = tanilar.into_iter().next() {
        return Sonuc {
            cikti: String::new(),
            hata: Some(tani.to_string()),
        };
    }
    let program = match huma::compiler::Derleyici::new().derle_kontrollu(ast) {
        Ok(program) => program,
        Err(hata) => {
            return Sonuc {
                cikti: String::new(),
                hata: Some(format!("[VM derleme] {hata}")),
            }
        }
    };
    let buffer = Rc::new(RefCell::new(String::new()));
    let hata = huma::vm::VM::new(program)
        .with_output_buffer(Rc::clone(&buffer))
        .run_checked()
        .err()
        .map(|hata| hata.to_string());
    let cikti = buffer.borrow().clone();
    Sonuc { cikti, hata }
}

/// `// golden: yalnız-yorumlayıcı <gerekçe>` başlığını okur.
fn vm_dislama_gerekcesi(kaynak: &str) -> Result<Option<String>, String> {
    for satir in kaynak.lines().take_while(|satir| satir.starts_with("//")) {
        let Some(yonerge) = satir.strip_prefix("// golden:") else {
            continue;
        };
        let yonerge = yonerge.trim();
        let Some(gerekce) = yonerge.strip_prefix("yalnız-yorumlayıcı") else {
            return Err(format!("bilinmeyen golden yönergesi: {yonerge}"));
        };
        let gerekce = gerekce.trim();
        if gerekce.is_empty() {
            return Err("yalnız-yorumlayıcı yönergesi gerekçe içermeli".to_string());
        }
        return Ok(Some(gerekce.to_string()));
    }
    Ok(None)
}

fn oku_varsa(path: &Path) -> Option<String> {
    path.is_file().then(|| {
        std::fs::read_to_string(path)
            .unwrap_or_else(|hata| panic!("{} okunamadı: {hata}", path.display()))
    })
}

#[test]
fn golden_programlar_yorumlayici_ve_vm_de_beklenen_sonucu_verir() {
    let kok = workspace_root();
    let dizin = kok.join("tests").join("golden");
    let mut programlar: Vec<PathBuf> = std::fs::read_dir(&dizin)
        .expect("tests/golden okunmalı")
        .map(|girdi| girdi.expect("dizin girdisi okunmalı").path())
        .collect();
    programlar.sort();

    let mut hatalar = Vec::new();
    let mut vm_disi = Vec::new();
    let mut sayac = 0;
    for yol in &programlar {
        match yol.extension().and_then(|uzanti| uzanti.to_str()) {
            Some("hb") => {}
            Some("beklenen" | "hata") => {
                if !yol.with_extension("hb").is_file() {
                    hatalar.push(format!("{}: eşleşen .hb yok", yol.display()));
                }
                continue;
            }
            _ => {
                hatalar.push(format!("{}: tanınmayan golden dosyası", yol.display()));
                continue;
            }
        }
        sayac += 1;
        let ad = yol
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let kaynak = std::fs::read_to_string(yol).expect("golden kaynağı okunmalı");
        let beklenen_cikti = oku_varsa(&yol.with_extension("beklenen"));
        let beklenen_hata =
            oku_varsa(&yol.with_extension("hata")).map(|h| h.trim_end().to_string());
        if beklenen_cikti.is_none() && beklenen_hata.is_none() {
            hatalar.push(format!("{ad}: .beklenen veya .hata dosyası yok"));
            continue;
        }
        let beklenen = Sonuc {
            cikti: beklenen_cikti.unwrap_or_default(),
            hata: beklenen_hata,
        };

        let gerekce = match vm_dislama_gerekcesi(&kaynak) {
            Ok(gerekce) => gerekce,
            Err(hata) => {
                hatalar.push(format!("{ad}: {hata}"));
                continue;
            }
        };

        let yorumlayici = yorumlayicida_calistir(&kaynak, &kok);
        if yorumlayici != beklenen {
            hatalar.push(format!(
                "{ad} [yorumlayıcı]\n  beklenen: {beklenen:?}\n  gelen:    {yorumlayici:?}"
            ));
        }
        match gerekce {
            Some(gerekce) => vm_disi.push(format!("{ad}: {gerekce}")),
            None => {
                let vm = vm_de_calistir(&kaynak);
                if vm != beklenen {
                    hatalar.push(format!(
                        "{ad} [VM]\n  beklenen: {beklenen:?}\n  gelen:    {vm:?}"
                    ));
                }
            }
        }
    }

    assert!(sayac > 0, "tests/golden içinde program yok");
    // VM dışında bırakılan programlar `--nocapture` ile görünür.
    println!(
        "golden: {sayac} program, {} yalnız yorumlayıcıda:\n  {}",
        vm_disi.len(),
        vm_disi.join("\n  ")
    );
    assert!(hatalar.is_empty(), "\n{}", hatalar.join("\n"));
}
