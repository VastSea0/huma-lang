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
use std::time::{SystemTime, UNIX_EPOCH};

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
    let _capability_guard = huma::capability::install(huma::capability::CapabilitySet::allow_all())
        .expect("Test yetenekleri kurulmalı");
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

fn gecici_modul_dizini(test_adi: &str) -> PathBuf {
    let benzersiz = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Sistem saati Unix epoch sonrasında olmalı")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "huma_modul_test_{}_{}_{}",
        test_adi,
        std::process::id(),
        benzersiz
    ));
    std::fs::create_dir_all(&path).expect("Geçici modül dizini oluşturulmalı");
    path
}

#[test]
fn ad_alanli_modul_yalniz_acik_dis_aktarimlari_gosterir() {
    let root = gecici_modul_dizini("ad_alani");
    let module_path = root.join("hesap.hb");
    std::fs::write(
        &module_path,
        r#"
            gizli = 40 olsun
            sayac = 0 olsun

            topla fonksiyon olsun deger alsın {
                gizli + deger'i döndür
            }
            artır fonksiyon olsun {
                sayac = sayac + 1
                sayac'ı döndür
            }

            topla'yı dışa aktar
            artır'ı dışa aktar
        "#,
    )
    .expect("Geçici modül yazılmalı");

    let source = r#"
        yükle "hesap.hb" olarak hesap
        hesap'ın topla(2)'yi yazdır
        hesap'ın artır()'ı yazdır
        hesap'ın artır()'ı yazdır
        hesap'ın gizli'yi yazdır
    "#;
    let (output, failed) = eval_with_paths(source, vec![root.to_string_lossy().to_string()]);
    assert!(!failed, "{output}");
    assert_eq!(output, "42\n1\n2\nBoş\n");

    std::fs::remove_dir_all(root).expect("Geçici modül dizini temizlenmeli");
}

#[test]
fn dis_a_aktar_modul_disinda_reddedilir() {
    let (output, failed) = eval_with_paths("deger = 1 olsun\ndeger'i dışa aktar", Vec::new());
    assert!(failed);
    assert!(output.contains("yalnızca yüklenen bir modül"));
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

#[test]
fn ayni_modul_farkli_goreli_yollardan_bir_kez_yuklenir() {
    let dizin = gecici_modul_dizini("kanonik");
    std::fs::write(dizin.join("tek.hb"), r#""yalnız bir kez"'i yazdır"#).expect("Modül yazılmalı");

    let kod = r#"
        yükle "tek.hb"
        yükle "./tek.hb"
    "#;
    let (cikti, hata_var) = eval_with_paths(kod, vec![dizin.to_string_lossy().to_string()]);
    assert!(!hata_var, "Kanonik modül yükleme başarısız: {cikti}");
    assert_eq!(cikti, "yalnız bir kez\n");

    std::fs::remove_dir_all(dizin).expect("Geçici modül dizini temizlenmeli");
}

#[test]
fn dongusel_modul_yukleme_acik_hata_verir() {
    let dizin = gecici_modul_dizini("dongu");
    std::fs::write(dizin.join("a.hb"), r#"yükle "b.hb""#).expect("a modülü yazılmalı");
    std::fs::write(dizin.join("b.hb"), r#"yükle "a.hb""#).expect("b modülü yazılmalı");

    let (cikti, hata_var) =
        eval_with_paths(r#"yükle "a.hb""#, vec![dizin.to_string_lossy().to_string()]);
    assert!(hata_var, "Döngüsel modül yükleme reddedilmeliydi");
    assert!(cikti.contains("Döngüsel modül yükleme"));

    std::fs::remove_dir_all(dizin).expect("Geçici modül dizini temizlenmeli");
}

#[test]
fn basarisiz_modul_duzeltildikten_sonra_yeniden_yuklenebilir() {
    let dizin = gecici_modul_dizini("yeniden");
    let modul = dizin.join("duzelt.hb");
    std::fs::write(&modul, "bu =").expect("Bozuk modül yazılmalı");

    let output = Rc::new(RefCell::new(String::new()));
    let mut yorumlayici = Yorumlayici::new().with_output_buffer(Rc::clone(&output));
    yorumlayici
        .arama_yolları
        .insert(0, dizin.to_string_lossy().to_string());

    let mut parser = Parser::new(Lexer::new(r#"yükle "duzelt.hb""#));
    let program = parser.parse_program();
    assert!(yorumlayici.yorumla_kontrollu(program).is_err());
    assert!(
        yorumlayici.yuklenen_dosyalar.is_empty(),
        "Başarısız modül yüklenmiş sayılmamalı"
    );

    std::fs::write(&modul, r#""düzeltildi"'yi yazdır"#).expect("Modül düzeltilmeli");
    let mut parser = Parser::new(Lexer::new(r#"yükle "duzelt.hb""#));
    let program = parser.parse_program();
    yorumlayici
        .yorumla_kontrollu(program)
        .expect("Düzeltilen modül yeniden yüklenebilmeli");
    assert_eq!(output.borrow().as_str(), "düzeltildi\n");

    std::fs::remove_dir_all(dizin).expect("Geçici modül dizini temizlenmeli");
}
