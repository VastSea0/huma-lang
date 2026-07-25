// huma_modulleri (nlp_temel, yapay_zeka) doğrulama testleri.
//
// Bu testler modüllerin yalnızca ayrıştırılmasını değil, gerçek çağrılarla
// çalışmasını da doğrular.

use huma::interpreter::Yorumlayici;
use huma::lexer::Lexer;
use huma::parser::Parser;
use huma_core as huma;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

/// Proje kökünü döndürür (huma-lang dizini).
fn proje_koku() -> PathBuf {
    // crate manifest dizininden 3 üst → workspace kökü
    let manifest = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest)
        .parent()
        .unwrap() // crates/
        .parent()
        .unwrap() // huma-lang/
        .to_path_buf()
}

/// Verilen kök dizin ve kaynak kodu ile yorumlayıcıyı çalıştırır.
/// `extra_yollar` listesi ek arama yolları ekler (modül dosyaları için).
fn eval_with_paths(kod: &str, extra_yollar: Vec<String>) -> (String, bool) {
    let buf = Rc::new(RefCell::new(String::new()));
    let mut yorumlayici = Yorumlayici::new().with_output_buffer(Rc::clone(&buf));
    for yol in extra_yollar {
        yorumlayici.arama_yolları.insert(0, yol);
    }
    let lx = Lexer::new(kod);
    let mut p = Parser::new(lx);
    let (prog, diagnostics) = p.parse_program_with_diagnostics();
    if let Some(first) = diagnostics.into_iter().next() {
        return (first.to_string(), true);
    }
    let runtime_result = yorumlayici.yorumla_kontrollu(prog);
    let out = buf.borrow().clone();
    match runtime_result {
        Ok(()) => (out, false),
        Err(error) => (format!("{}{}", out, error), true),
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// nlp_temel modülü testleri
// ══════════════════════════════════════════════════════════════════════════════

/// nlp_temel modülünün yüklenip yüklenmediğini kontrol eder.
/// Modül bozuksa hata mesajı görünür; test bu durumu kayıt altına alır.
#[test]
fn nlp_temel_modul_yuklenebilir() {
    let kok = proje_koku();
    let nlp_yolu = kok
        .join("huma_modulleri/nlp_temel")
        .to_string_lossy()
        .to_string();

    let kod = r#"yükle "nlp_temel.hb""#;
    // Paniklememeli — bozuk olsa bile hata mesajıyla devam etmeli
    let (cikti, hata_var) = eval_with_paths(kod, vec![nlp_yolu]);

    assert!(!hata_var, "nlp_temel yüklenemedi:\n{}", cikti);
}

/// nlp_temel sabitler.hb yüklenebilmeli (bağımsız en küçük parça).
#[test]
fn nlp_temel_sabitler_yuklenebilir() {
    let kok = proje_koku();
    let nlp_yolu = kok
        .join("huma_modulleri/nlp_temel")
        .to_string_lossy()
        .to_string();

    let kod = r#"yükle "sabitler.hb""#;
    let (cikti, hata_var) = eval_with_paths(kod, vec![nlp_yolu]);
    assert!(!hata_var, "nlp_temel/sabitler yüklenemedi:\n{}", cikti);
}

/// metin_islemci sınıfı oluşturulabilmeli ve temizle çağrılabilmeli.
#[test]
fn nlp_temel_metin_islemci_olusturma() {
    let kok = proje_koku();
    let nlp_yolu = kok
        .join("huma_modulleri/nlp_temel")
        .to_string_lossy()
        .to_string();

    let kod = r#"
        yükle "sabitler.hb"
        yükle "islemci.hb"
        islemci = metin_islemci() olsun
        "islemci oluşturuldu"'yu yazdır
    "#;
    let (cikti, hata_var) = eval_with_paths(kod, vec![nlp_yolu]);
    assert!(!hata_var, "metin_islemci oluşturulamadı:\n{}", cikti);
    assert!(
        cikti.contains("islemci oluşturuldu"),
        "metin_islemci nesnesi oluşturulmalıydı, çıktı: {}",
        cikti
    );
}

/// kok_bulucu (stemmer) sınıfı oluşturulabilmeli.
#[test]
fn nlp_temel_stemmer_olusturma() {
    let kok = proje_koku();
    let nlp_yolu = kok
        .join("huma_modulleri/nlp_temel")
        .to_string_lossy()
        .to_string();

    let kod = r#"
        yükle "sabitler.hb"
        yükle "stemmer.hb"
        stemmer = kok_bulucu() olsun
        "stemmer oluşturuldu"'yu yazdır
    "#;
    let (cikti, hata_var) = eval_with_paths(kod, vec![nlp_yolu]);
    assert!(!hata_var, "kok_bulucu oluşturulamadı:\n{}", cikti);
    assert!(
        cikti.contains("stemmer oluşturuldu"),
        "kok_bulucu nesnesi oluşturulmalıydı, çıktı: {}",
        cikti
    );
}

/// İleri NLP giriş noktası gerçek TF-IDF ve gömme işlemlerini çalıştırmalı.
#[test]
fn nlp_ileri_tfidf_ve_gomme_akisi_calisir() {
    let kok = proje_koku();
    let nlp_yolu = kok
        .join("huma_modulleri/nlp_ileri")
        .to_string_lossy()
        .to_string();

    let kod = r#"
        yükle "nlp_ileri.hb"

        corpus = [["elma", "armut"], ["elma"]] olsun
        tfidf = tfidf_matrisi(corpus) olsun
        matris_boyutu(tfidf["matris"])'ı yazdır

        gomme = gomme_tabakasi() olsun
        gomme.ilklendir(3, 4) olsun
        once = matris_al(gomme.E, 0, 0) olsun
        gradyan = vektor_olustur(4, 0.25) olsun
        gomme.guncelle(0, gradyan, 0.01) olsun
        sonra = matris_al(gomme.E, 0, 0) olsun
        ileri = gomme.ileri([0, 1]) olsun
        gomme.pozisyonel_kodla(ileri, 2) olsun
        matris_boyutu(ileri)'yi yazdır
        once'yi yazdır
        sonra'yı yazdır
    "#;
    let (cikti, hata_var) = eval_with_paths(kod, vec![nlp_yolu]);
    assert!(!hata_var, "İleri NLP akışı çalışmadı:\n{}", cikti);
    let satirlar = cikti.lines().collect::<Vec<_>>();
    assert_eq!(satirlar.len(), 4, "Dört doğrulama çıktısı bekleniyordu");
    assert_eq!(satirlar[0].trim(), "[2, 2]");
    assert_eq!(satirlar[1].trim(), "[2, 4]");
    let once = satirlar[2]
        .trim()
        .parse::<f64>()
        .expect("Önceki değer sayı");
    let sonra = satirlar[3]
        .trim()
        .parse::<f64>()
        .expect("Sonraki değer sayı");
    assert_ne!(
        once, sonra,
        "Adam güncellemesi gömme ağırlığını değiştirmeli"
    );
}

/// Yerleşik BPE, Türkçe UTF-8 metni ve boşlukları kayıpsız geri çözmeli.
#[test]
fn bpe_turkce_metni_kayipsiz_kodlar() {
    let kod = r#"
        metin = "İyi günler, şeker ölçümü!" olsun
        bpe_eğit(metin, 280) olsun
        tokenler = bpe_kodla(metin) olsun
        çözülmüş = bpe_çöz(tokenler) olsun
        çözülmüş'ü yazdır
    "#;
    let (cikti, hata_var) = eval_with_paths(kod, vec![]);
    assert!(!hata_var, "BPE akışı çalışmadı:\n{}", cikti);
    assert_eq!(cikti.trim(), "İyi günler, şeker ölçümü!");
}

// ══════════════════════════════════════════════════════════════════════════════
// yapay_zeka modülü testleri
// ══════════════════════════════════════════════════════════════════════════════

/// yapay_zeka modülünün yüklenip yüklenmediğini kontrol eder.
#[test]
fn yapay_zeka_modul_yuklenebilir() {
    let kok = proje_koku();
    let yz_yolu = kok
        .join("huma_modulleri/yapay_zeka")
        .to_string_lossy()
        .to_string();

    let kod = r#"yükle "yapay_zeka.hb""#;
    let (cikti, hata_var) = eval_with_paths(kod, vec![yz_yolu]);
    assert!(!hata_var, "yapay_zeka modülü yüklenemedi:\n{}", cikti);
}

/// Kanonik yoğun katman modülü yüklenebilmeli.
#[test]
fn yapay_zeka_yogun_katman_yuklenebilir() {
    let kok = proje_koku();
    let yz_yolu = kok
        .join("huma_modulleri/yapay_zeka")
        .to_string_lossy()
        .to_string();

    let kod = r#"yükle "yogun_katman.hb""#;
    let (cikti, hata_var) = eval_with_paths(kod, vec![yz_yolu]);
    assert!(!hata_var, "yapay_zeka/yogun_katman yüklenemedi:\n{}", cikti);
}

/// Küçük bir ağda gerçek geri yayılım ağırlıkları hedef yönünde değiştirmeli.
#[test]
fn yapay_zeka_egitim_adimi_tahmini_iyilestirir() {
    let kok = proje_koku();
    let yz_yolu = kok
        .join("huma_modulleri/yapay_zeka")
        .to_string_lossy()
        .to_string();

    let kod = r#"
        yükle "sinir_agi.hb"
        model = sinir_agi() olsun
        model.ilklendir() olsun
        model.katman_ekle(1, 1, "sigmoid") olsun
        giris = listeye_vektor([1.0]) olsun
        hedef = listeye_vektor([1.0]) olsun
        once = vektor_al(model.tahmin_et(giris), 0) olsun
        i = 1'den 40'a kadar {
            model.egitim_adimi(giris, hedef, 0.01) olsun
        }
        sonra = vektor_al(model.tahmin_et(giris), 0) olsun
        once'yi yazdır
        sonra'yı yazdır
    "#;
    let (cikti, hata_var) = eval_with_paths(kod, vec![yz_yolu]);
    assert!(!hata_var, "Yapay zekâ eğitim adımı çalışmadı:\n{}", cikti);
    let degerler = cikti
        .lines()
        .filter_map(|line| line.trim().parse::<f64>().ok())
        .collect::<Vec<_>>();
    assert_eq!(degerler.len(), 2, "İki tahmin bekleniyordu: {}", cikti);
    assert!(
        degerler[1] > degerler[0],
        "Hedef 1 iken eğitim tahmini artırmalıydı: {} -> {}",
        degerler[0],
        degerler[1]
    );
}

/// SQLite köprüsü sorguları çalıştırmalı ve hataları çalışma zamanı hatasına çevirmeli.
#[test]
fn sqlite_sorgu_ve_hata_akisi_calisir() {
    let kod = r#"
        id = dahili_sql_bağlan(":memory:") olsun
        dahili_sql_yürüt(id, "CREATE TABLE kisiler (ad TEXT NOT NULL)") olsun
        dahili_sql_yürüt(id, "INSERT INTO kisiler (ad) VALUES ('Hüma')") olsun
        satirlar = dahili_sql_sorgula(id, "SELECT ad FROM kisiler") olsun
        satirlar[0]["ad"]'ı yazdır

        dene {
            dahili_sql_yürüt(id, "GEÇERSİZ SQL") olsun
        } yakala sorun {
            "SQL hatası yakalandı"'yı yazdır
        }
    "#;
    let (cikti, hata_var) = eval_with_paths(kod, vec![]);
    assert!(!hata_var, "SQLite akışı çalışmadı:\n{}", cikti);
    assert_eq!(cikti.trim(), "Hüma\nSQL hatası yakalandı");
}

/// Dosya I/O başarısızlığı boş değere dönüşmek yerine yakalanabilir hata üretmeli.
#[test]
fn dosya_okuma_hatasi_yakalanabilir() {
    let kod = r#"
        dene {
            dosya_oku("/__huma_var_olmayan_dizin__/yok.txt") olsun
        } yakala sorun {
            "dosya hatası yakalandı"'yı yazdır
        }
    "#;
    let (cikti, hata_var) = eval_with_paths(kod, vec![]);
    assert!(!hata_var, "Dosya hatası yakalanamadı:\n{}", cikti);
    assert_eq!(cikti.trim(), "dosya hatası yakalandı");
}

// ══════════════════════════════════════════════════════════════════════════════
// Interpreter durum testleri — modül yükleme sonrası sağlık kontrolü
// ══════════════════════════════════════════════════════════════════════════════

/// Modül yükledikten sonra yorumlayıcı temel işlemlere devam edebilmeli.
#[test]
fn yorumlayici_modul_sonrasi_devam_edebilir() {
    let kok = proje_koku();
    let nlp_yolu = kok
        .join("huma_modulleri/nlp_temel")
        .to_string_lossy()
        .to_string();

    let kod = r#"
        yükle "sabitler.hb"
        x = 42 olsun
        x'i yazdır
    "#;
    let (cikti, hata_var) = eval_with_paths(kod, vec![nlp_yolu]);
    assert!(!hata_var, "Modül yükleme sonrası hata oluştu:\n{}", cikti);
    // Modül yüklensin ya da yüklenmesin, basit değişken işlemleri çalışmalı
    assert!(
        cikti.contains("42"),
        "Modül yüklemesi temel işlemleri bozmamalı, çıktı: {}",
        cikti
    );
}
