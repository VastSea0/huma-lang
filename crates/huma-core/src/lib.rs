//! # Hüma Core
//!
//! Core language primitives for the Hüma programming language:
//! tokens, AST definitions, lexer, parser, values, bytecode,
//! bytecode compiler, virtual machine, and tree-walking interpreter.

pub mod compiler {
    pub use huma_compiler::Derleyici;
}
pub mod autograd {
    pub use huma_stdlib_ai::autograd::*;
}

pub mod ai {
    pub use huma_stdlib_ai::*;
}

pub mod capability {
    pub use huma_runtime::capability::*;
}

pub mod ffi {
    pub use huma_stdlib_native::*;
}

pub mod file {
    pub use huma_stdlib_file::*;
}

pub mod sqlite {
    pub use huma_stdlib_sqlite::*;
}

pub mod net {
    pub use huma_stdlib_net::*;
}

pub mod process {
    pub use huma_stdlib_process::*;
}

pub mod gc {
    pub use huma_runtime::gc::*;
}

pub mod interpreter {
    pub use huma_runtime::interpreter::*;
}

pub mod limits {
    pub use huma_runtime::limits::*;
}

pub mod semantics {
    pub use huma_runtime::semantics::*;
}

pub mod tokenizer {
    pub use huma_stdlib_ai::tokenizer::*;
}

pub mod value {
    pub use huma_runtime::value::*;
}

pub mod vm {
    pub use huma_vm::*;
}

/// 0.6 geçiş uyumluluğu: alan bağımsız kaynak dili artık `huma-syntax`
/// paketindedir. Bu modüller eski Rust çağıranlarını kırmadan aynı tipleri
/// yeniden dışa aktarır.
pub mod ast {
    pub use huma_syntax::ast::*;
}

pub mod lexer {
    pub use huma_syntax::lexer::*;
}

pub mod morphology {
    pub use huma_syntax::morphology::*;
}

pub mod parser {
    pub use huma_syntax::parser::*;
}

pub mod token {
    pub use huma_syntax::token::*;
}

pub mod error {
    pub use huma_syntax::error::*;
}

pub mod bytecode {
    pub use huma_bytecode::*;
}

pub mod hmi {
    pub use huma_hmi::*;
}

pub mod builtin_files {
    pub use huma_stdlib::*;
}

/// Re-export most-used items at the crate root for convenience.
pub use error::{HumaError, HumaResult, RuntimeDiagnostic, SourceSpan, StackFrame};

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::{SystemTime, UNIX_EPOCH};

    // ─────────────────────────────────────────────
    // Yardımcı: kodu yorumlayıp string çıktısını döndürür
    // ─────────────────────────────────────────────
    fn eval(kod: &str) -> String {
        let buf = Rc::new(RefCell::new(String::new()));
        let mut yorumlayici = interpreter::Yorumlayici::new().with_output_buffer(Rc::clone(&buf));
        file::kayit_et(&mut yorumlayici.global_degiskenler);
        ffi::kayit_et(&mut yorumlayici.global_degiskenler);
        ai::kayit_et(&mut yorumlayici.global_degiskenler);
        sqlite::kayit_et(&mut yorumlayici.global_degiskenler);
        net::yorumlayiciyi_yapilandir(&mut yorumlayici);
        process::kayit_et(&mut yorumlayici.global_degiskenler);
        let lx = lexer::Lexer::new(kod);
        let mut p = parser::Parser::new(lx);
        let (prog, diagnostics) = p.parse_program_with_diagnostics();
        assert!(
            diagnostics.is_empty(),
            "Kaynak ayrıştırılamadı: {:?}",
            diagnostics
        );
        yorumlayici
            .yorumla_kontrollu(prog)
            .expect("Kaynak çalışma zamanı hatası vermemeli");
        let sonuc = buf.borrow().clone();
        sonuc
    }

    fn eval_hatasi(kod: &str) -> String {
        let mut yorumlayici = interpreter::Yorumlayici::new();
        file::kayit_et(&mut yorumlayici.global_degiskenler);
        ffi::kayit_et(&mut yorumlayici.global_degiskenler);
        ai::kayit_et(&mut yorumlayici.global_degiskenler);
        sqlite::kayit_et(&mut yorumlayici.global_degiskenler);
        net::yorumlayiciyi_yapilandir(&mut yorumlayici);
        process::kayit_et(&mut yorumlayici.global_degiskenler);
        let mut parser = parser::Parser::new(lexer::Lexer::new(kod));
        let (program, diagnostics) = parser.parse_program_with_diagnostics();
        assert!(
            diagnostics.is_empty(),
            "Kaynak ayrıştırılamadı: {:?}",
            diagnostics
        );
        yorumlayici
            .yorumla_kontrollu(program)
            .expect_err("Kaynak çalışma zamanı hatası vermeliydi")
            .to_string()
    }

    // ─────────────────────────────────────────────
    // Görev 1 — Regresyon testleri
    // ─────────────────────────────────────────────

    #[test]
    fn test_derin_rekursiyon_hatasi() {
        let kod = "rekursiyon fonksiyon olsun { rekursiyon() } rekursiyon()";
        let mut interp = interpreter::Yorumlayici::new();
        let lx = lexer::Lexer::new(kod);
        let mut p = parser::Parser::new(lx);
        let program = p.parse_program();
        let error = interp
            .yorumla_kontrollu(program)
            .expect_err("Sınırsız özyineleme çalışma zamanı hatası vermeli");
        assert!(error.to_string().contains("özyineleme"));
        assert_eq!(
            interp.call_depth, 0,
            "call_depth çıkıştan sonra sıfırlanmalı"
        );
    }

    #[test]
    fn test_cagrilamayan_deger_hatasi() {
        let kod = "x = 42 olsun\nx()";
        let mut interp = interpreter::Yorumlayici::new();
        let lx = lexer::Lexer::new(kod);
        let mut p = parser::Parser::new(lx);
        let program = p.parse_program();
        let error = interp
            .yorumla_kontrollu(program)
            .expect_err("Çağrılamayan değer çalışma zamanı hatası vermeli");
        assert!(error.to_string().contains("Çağrılamayan değer"));
    }

    #[test]
    fn temel_yerlesikler_gecersiz_girdiyi_sessizce_yutmaz() {
        assert!(eval_hatasi("uzunluk(42)").contains("uzunluk"));
        assert!(eval_hatasi("listeye_ekle(42, 1)").contains("ilk argüman liste"));
        assert!(eval_hatasi("karekök(\"değil\")").contains("sayı bekleniyordu"));
        assert!(eval_hatasi("karekök(-1)").contains("negatif olmayan"));
        assert!(eval_hatasi("uyut(-1)").contains("negatif olmayan"));
        assert!(eval_hatasi("zaman(1)").contains("argüman beklenmiyordu"));
        assert!(eval_hatasi("rastgele(1)").contains("argüman beklenmiyordu"));
        assert!(eval_hatasi("sistem(42)").contains("metin komutu"));
        assert!(eval_hatasi("hmi_başlat(1, 2)").contains("2 metin"));
        assert!(eval_hatasi("hmi_çağır(1, 2)").contains("en az 2 metin"));
        assert!(eval_hatasi("hmi_kapat(1)").contains("modül adı"));
        assert!(eval_hatasi("küçük_harf(1)").contains("metin bekleniyordu"));
        assert!(eval_hatasi(r#"böl("a")"#).contains("tam olarak 2"));
        assert!(eval_hatasi("birleştir([1])").contains("yalnızca metin"));
        assert!(eval_hatasi(r#"değiştir("a", "a")"#).contains("tam olarak 3"));
        assert!(eval_hatasi(r#"sayıya_çevir("sayı değil")"#).contains("geçerli bir sayı değil"));
        assert!(eval_hatasi(r#"ascii_kodu("iki")"#).contains("tam olarak bir Unicode"));
        assert!(eval_hatasi("karakterden(-1)").contains("sonlu bir tamsayı"));
        assert!(eval_hatasi("içeriyor(1, 2)").contains("metin, liste, nesne veya sözlük"));
        assert!(eval_hatasi("tipi()").contains("tam olarak 1"));
        assert!(eval_hatasi(r#"dizi_dilim("abc", 0, 4)"#).contains("geçerli aralık"));
    }

    #[test]
    fn unicode_buyuk_harf_birden_cok_kod_noktasi_uretebilir() {
        assert_eq!(eval(r#"büyük_harf("straße ıi")'yi yazdır"#), "STRASSE Iİ\n");
    }

    #[test]
    fn regex_degistirme_metnini_grup_sablonu_gibi_yorumlamaz() {
        assert_eq!(
            eval(r#"regex_degistir("a-a", "a", "$1")'ı yazdır"#),
            "$1-$1\n"
        );
    }

    #[test]
    fn metrikler_uyumsuz_ve_gecersiz_veriyi_reddeder() {
        assert!(eval_hatasi("f1_skoru([1], [1, 0])").contains("uzunlukları eşit"));
        assert!(eval_hatasi("f1_skoru([], [])").contains("boş veri"));
        assert!(eval_hatasi("f1_skoru([1.1], [1])").contains("0 ile 1"));
        assert!(eval_hatasi("karisiklik_matrisi([0.5], [0], 2)").contains("tamsayı"));
    }

    #[test]
    fn adam_durumu_alias_hatasinda_kismi_guncellenmez() {
        let output = eval(
            r#"
                ağırlıklar = vektor_olustur(1, 1) olsun
                gradyan = vektor_olustur(1, 1) olsun
                durum = adam_vektor_durum_olustur(1) olsun
                değer_ata(durum, "m", ağırlıklar) olsun
                dene {
                    adam_vektor_guncelle(ağırlıklar, gradyan, durum, 0.1) olsun
                } yakala hata { }
                vektor_al(ağırlıklar, 0)'ı yazdır
                değer_al(durum, "adim")'ı yazdır
            "#,
        );
        assert_eq!(output, "1\n0\n");
    }

    #[test]
    fn csv_alintili_alanlari_kayipsiz_yazar_ve_okur() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Sistem saati Unix epoch sonrasında olmalı")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "huma_csv_roundtrip_{}_{}.csv",
            std::process::id(),
            nonce
        ));
        let escaped_path = path
            .to_string_lossy()
            .replace('\\', "\\\\")
            .replace('"', "\\\"");
        let _guard = capability::install(
            capability::CapabilitySet::deny_all()
                .allow(capability::Capability::FileRead)
                .allow(capability::Capability::FileWrite),
        )
        .expect("Dosya yetenekleri kurulmalı");
        let output = eval(&format!(
            r#"
                satırlar = [["a,b", "iki\nsatır", "\"alıntı\""], [42, boş, "ç"]] olsun
                csv_yaz("{escaped_path}", satırlar) olsun
                okunan = csv_oku("{escaped_path}") olsun
                nesneden_metine(okunan)'ı yazdır
            "#
        ));
        let _ = std::fs::remove_file(&path);
        assert_eq!(
            output,
            "[\n  [\n    \"a,b\",\n    \"iki\\nsatır\",\n    \"\\\"alıntı\\\"\"\n  ],\n  [\n    \"42\",\n    \"\",\n    \"ç\"\n  ]\n]\n"
        );
        assert!(eval_hatasi(r#"csv_yaz("x", [], "ğ")"#).contains("tek baytlık ASCII"));
        assert!(eval_hatasi(r#"csv_yaz("x", [[[]]])"#).contains("metin, sayı veya boş"));
    }

    #[test]
    fn sozluk_deger_api_sozlesmesi_gercekten_calisir() {
        let output = eval(
            r#"
                harita = {} olsun
                değer_ata(harita, "anahtar", 7) olsun
                değer_al(harita, "anahtar")'ı yazdır
                içeriyor(harita, "anahtar")'ı yazdır
                değer_al(harita, "yok")'u yazdır
            "#,
        );
        assert_eq!(output, "7\n1\nBoş\n");
    }

    #[test]
    fn dis_dunya_yetenekleri_varsayilan_olarak_kapalidir() {
        assert!(eval_hatasi(r#"dosya_oku("gizli.txt")"#).contains("yeteneği verilmedi"));
        assert!(eval_hatasi(r#"sistem("echo güvenli-değil")"#).contains("yeteneği verilmedi"));
        assert!(eval_hatasi(r#"dahili_istek("GET", "https://example.com")"#)
            .contains("yeteneği verilmedi"));
    }

    #[test]
    fn uzunluk_desteklenen_koleksiyonlarda_tutarlidir() {
        let output = eval(
            r#"
                uzunluk("Türkçe")'yi yazdır
                uzunluk([1, 2, 3])'ü yazdır
                uzunluk({"a": 1, "b": 2})'yi yazdır
            "#,
        );
        assert_eq!(output, "6\n3\n2\n");
    }

    #[test]
    fn anonim_fonksiyon_tanimlandigi_kapsami_yakalar() {
        let output = eval(
            r#"
                toplayici_uret fonksiyon olsun taban alsın {
                    toplayici = fonksiyon olsun deger alsın {
                        taban + deger'i döndür
                    } olsun
                    toplayici'yi döndür
                }
                on_ekle = toplayici_uret(10) olsun
                on_ekle(5)'i yazdır
            "#,
        );
        assert_eq!(output, "15\n");
    }

    #[test]
    fn fonksiyon_arguman_sayisi_tam_eslesmelidir() {
        let eksik = eval_hatasi(
            r#"
                topla fonksiyon olsun a, b alsın { a + b'yi döndür }
                topla(1)
            "#,
        );
        assert!(eksik.contains("2 argüman bekliyor; 1 argüman geldi"));

        let fazla = eval_hatasi(
            r#"
                kimlik fonksiyon olsun x alsın { x'i döndür }
                kimlik(1, 2)
            "#,
        );
        assert!(fazla.contains("1 argüman bekliyor; 2 argüman geldi"));
    }

    #[test]
    fn uc_argumanli_dogal_turkce_cagri_gercekten_calisir() {
        let output = eval(
            r#"
                topla3 fonksiyon olsun a, b, c alsın {
                    (a + b + c)'yi döndür
                }
                1 ile 2 ve 3'ü topla3'ü yazdır
            "#,
        );
        assert_eq!(output, "6\n");
    }

    #[test]
    fn dogal_turkce_liste_ekleme_gercekten_calisir() {
        let output = eval(
            r#"
                sayılar = [] olsun
                sayılar'a [1, 2]'yi ekle
                sayılar'a 3'ü ekle
                nesneden_metine(sayılar)'ı yazdır
            "#,
        );
        assert_eq!(output, "[\n  1.0,\n  2.0,\n  3.0\n]\n");
    }

    #[test]
    fn dogal_turkce_liste_cikarma_gercekten_calisir() {
        let output = eval(
            r#"
                öğeler = [10, 20, 30] olsun
                öğeler'den 1'i çıkar
                nesneden_metine(öğeler)'i yazdır
            "#,
        );
        assert_eq!(output, "[\n  10.0,\n  30.0\n]\n");
    }

    #[test]
    fn sinif_kurucusu_tanimsiz_argumanlari_reddeder() {
        let error = eval_hatasi(
            r#"
                Ornek sınıf olsun { deger = 1 olsun }
                Ornek(1)
            "#,
        );
        assert!(error.contains("kurucu argümanı kabul etmiyor"));
    }

    #[test]
    fn json_donusumu_veri_kaybini_ve_donguyu_reddeder() {
        assert!(value::Deger::Sayi(f64::INFINITY)
            .to_json_checked()
            .expect_err("sonsuz sayı JSON'a dönüşmemeli")
            .contains("sonlu"));

        let liste = gc::Gc::new(Vec::new());
        liste
            .borrow_mut()
            .push(value::Deger::Liste(gc::Gc::clone(&liste)));
        assert!(value::Deger::Liste(liste)
            .to_json_checked()
            .expect_err("döngüsel liste JSON'a dönüşmemeli")
            .contains("Döngüsel"));

        assert!(eval_hatasi(r#"metinden_nesneye("{")"#).contains("geçersiz JSON"));
        assert!(eval_hatasi(
            r#"
                f fonksiyon olsun { boş'u döndür }
                nesneden_metine(f)
            "#
        )
        .contains("temsil edilemez"));
    }

    #[test]
    fn ai_boyutlari_guvenli_sinirlarla_dogrulanir() {
        assert!(eval_hatasi("vektor_olustur(-1, 0)").contains("boyut"));
        assert!(eval_hatasi("matris_olustur(10000000, 2)").contains("güvenlik sınırını"));
        assert!(eval_hatasi("adam_durum_olustur(10000000, 2)").contains("güvenlik sınırını"));
    }

    #[test]
    fn ai_indeksleri_sessizce_yutulmaz() {
        assert!(
            eval_hatasi("v = vektor_olustur(2, 0) olsun\nvektor_al(v, 2)")
                .contains("sınır dışında")
        );
        assert!(
            eval_hatasi("m = matris_olustur(2, 2) olsun\nmatris_ata(m, 0, 2, 1)")
                .contains("sınır dışında")
        );
        assert!(eval_hatasi(
            "m = matris_olustur(2, 2) olsun\n\
                 k = vektor_olustur(1, 0) olsun\n\
                 matris_satir_ata(m, 0, k)"
        )
        .contains("vektör uzunluğu"));
    }

    #[test]
    fn test_vm_fonksiyon_cagrisi() {
        let kod = "yardimci fonksiyon olsun { \"Merhaba\"'yı yazdır }\nselamla fonksiyon olsun { yardimci() }\nselamla()";
        let lx = lexer::Lexer::new(kod);
        let mut p = parser::Parser::new(lx);
        let (ast, diagnostics) = p.parse_program_with_diagnostics();
        assert!(diagnostics.is_empty());
        let mut derleyici = compiler::Derleyici::new();
        let prog = derleyici
            .derle_kontrollu(ast)
            .expect("Fonksiyonlar bytecode'a derlenmeli");
        let mut vm = vm::VM::new(prog);
        vm.run_checked().expect("VM programı hatasız çalışmalı");
    }

    // ─────────────────────────────────────────────
    // Lexer unit testleri
    // ─────────────────────────────────────────────

    #[test]
    fn lexer_sayi_tokenize() {
        let mut lx = lexer::Lexer::new("42");
        assert_eq!(lx.next_token(), token::Token::Sayi(42.0));
        assert_eq!(lx.next_token(), token::Token::Son);
    }

    #[test]
    fn lexer_bilimsel_sayi_tokenize() {
        let mut lx = lexer::Lexer::new("1e-7 1.0e308 2E+3");
        assert_eq!(lx.next_token(), token::Token::Sayi(1e-7));
        assert_eq!(lx.next_token(), token::Token::Sayi(1.0e308));
        assert_eq!(lx.next_token(), token::Token::Sayi(2e3));
        assert_eq!(lx.next_token(), token::Token::Son);
    }

    #[test]
    fn lexer_sonlu_olmayan_sayiyi_reddeder() {
        let mut lx = lexer::Lexer::new("1e309");
        assert!(matches!(lx.next_token(), token::Token::Hata(_)));
    }

    #[test]
    fn lexer_bilinmeyen_metin_ekini_reddeder() {
        let mut lx = lexer::Lexer::new(r#""metin"'xyz"#);
        assert_eq!(lx.next_token(), token::Token::Metin("metin".to_string()));
        assert!(matches!(lx.next_token(), token::Token::Hata(_)));
    }

    #[test]
    fn lexer_metin_tokenize() {
        let mut lx = lexer::Lexer::new("\"merhaba\"");
        assert_eq!(lx.next_token(), token::Token::Metin("merhaba".to_string()));
    }

    #[test]
    fn lexer_anahtar_kelimeler() {
        let mut lx = lexer::Lexer::new("olsun fonksiyon");
        assert_eq!(lx.next_token(), token::Token::Olsun);
        assert_eq!(lx.next_token(), token::Token::Fonksiyon);
    }

    #[test]
    fn lexer_aritmetik_operatorler() {
        let mut lx = lexer::Lexer::new("+ - * /");
        assert_eq!(lx.next_token(), token::Token::Arti);
        assert_eq!(lx.next_token(), token::Token::Eksi);
        assert_eq!(lx.next_token(), token::Token::Carpi);
        assert_eq!(lx.next_token(), token::Token::Bolnu);
    }

    #[test]
    fn lexer_karsilastirma_operatorleri() {
        let mut lx = lexer::Lexer::new("> < >= <= == !=");
        assert_eq!(lx.next_token(), token::Token::Buyuktur);
        assert_eq!(lx.next_token(), token::Token::Kucuktur);
        assert_eq!(lx.next_token(), token::Token::BuyukEsit);
        assert_eq!(lx.next_token(), token::Token::KucukEsit);
        assert_eq!(lx.next_token(), token::Token::EsitEsittir);
        assert_eq!(lx.next_token(), token::Token::EsitDegil);
    }

    #[test]
    fn lexer_tanimlayici() {
        let mut lx = lexer::Lexer::new("degisken_adi");
        match lx.next_token() {
            token::Token::Tanimlayici(s) => assert_eq!(s, "degisken_adi"),
            t => panic!("Beklenen Tanimlayici, gelen: {:?}", t),
        }
    }

    #[test]
    fn lexer_yorum_satiri_atlanir() {
        let mut lx = lexer::Lexer::new("// bu yorum\n42");
        assert_eq!(lx.next_token(), token::Token::Sayi(42.0));
    }

    // ─────────────────────────────────────────────
    // Interpreter happy-path testleri
    // ─────────────────────────────────────────────

    #[test]
    fn interpreter_degisken_atama() {
        let out = eval("x = 5 olsun\nx'i yazdır");
        assert_eq!(out.trim(), "5");
    }

    #[test]
    fn interpreter_temel_aritmetik() {
        let out = eval("(3 + 4 * 2)'yi yazdır");
        assert_eq!(out.trim(), "11");
    }

    #[test]
    fn interpreter_esitlik_karsilastirma() {
        let out = eval(
            r#"
            a = 5 olsun
            a == 5 ise { "evet"'i yazdır }
        "#,
        );
        assert_eq!(out.trim(), "evet");
    }

    #[test]
    fn interpreter_yoksa_kolu() {
        let out = eval(
            r#"
            x = 3 olsun
            x > 10 ise {
                "buyuk"'u yazdır
            } yoksa {
                "kucuk"'u yazdır
            }
        "#,
        );
        assert_eq!(out.trim(), "kucuk");
    }

    #[test]
    fn interpreter_fonksiyon_parametreli() {
        let out = eval(
            r#"
            kare fonksiyon olsun n alsın {
                n * n'i döndür
            }
            kare(7)'yi yazdır
        "#,
        );
        assert_eq!(out.trim(), "49");
    }

    #[test]
    fn interpreter_liste_islemi() {
        let out = eval(
            r#"
            dizi = [10, 20, 30] olsun
            dizi[2]'yi yazdır
        "#,
        );
        assert_eq!(out.trim(), "30");
    }

    #[test]
    fn interpreter_liste_elemani_atama() {
        let out = eval(
            r#"
            dizi = [10, 20, 30] olsun
            dizi[1] = 99 olsun
            dizi[1]'i yazdır
        "#,
        );
        assert_eq!(out.trim(), "99");
    }

    #[test]
    fn interpreter_aralik_devam_ve_kir() {
        let out = eval(
            r#"
            toplam = 0 olsun
            i = 0'dan 5'e kadar {
                i = 2 ise { devam }
                i = 4 ise { kır }
                toplam = toplam + i olsun
            }
            toplam'ı yazdır
        "#,
        );
        assert_eq!(out.trim(), "4");
    }

    #[test]
    fn interpreter_dene_yakala_calisma_zamani_hatasi() {
        let out = eval(
            r#"
            dene {
                tanimsiz_degisken'i yazdır
            } yakala sorun {
                "yakalandı"'yı yazdır
            }
            "devam"'ı yazdır
        "#,
        );
        assert_eq!(out.trim(), "yakalandı\ndevam");
    }

    #[test]
    fn interpreter_mantiksal_atama_ve_kisa_devre() {
        let out = eval(
            r#"
            sonuc = yanlış ve tanimsiz_degisken olsun
            diger = doğru veya baska_tanimsiz_degisken olsun
            sonuc'u yazdır
            diger'i yazdır
        "#,
        );
        assert_eq!(out.trim(), "0\n1");
    }

    #[test]
    fn parser_ayrilmis_sozcugu_atama_hedefi_olarak_reddeder() {
        let mut parser = parser::Parser::new(lexer::Lexer::new("dogru = 1 olsun"));
        let (_, diagnostics) = parser.parse_program_with_diagnostics();
        assert!(diagnostics
            .iter()
            .any(|error| error.to_string().contains("atamanın sol tarafı")));
    }

    #[test]
    fn interpreter_liste_atamasinda_sinir_hatasi() {
        let mut parser =
            parser::Parser::new(lexer::Lexer::new("dizi = [1] olsun\ndizi[2] = 9 olsun"));
        let (program, diagnostics) = parser.parse_program_with_diagnostics();
        assert!(diagnostics.is_empty());
        let mut interp = interpreter::Yorumlayici::new();
        let error = interp
            .yorumla_kontrollu(program)
            .expect_err("Sınır dışı atama hata vermeli");
        assert!(error.to_string().contains("indeks sınır dışında"));
    }

    #[test]
    fn interpreter_metin_birlestirme() {
        let out = eval(
            r#"
            ad = "Dünya" olsun
            "Merhaba " + ad'ı yazdır
        "#,
        );
        assert_eq!(out.trim(), "Merhaba Dünya");
    }

    #[test]
    fn interpreter_dongude_toplam() {
        let out = eval(
            r#"
            toplam = 0 olsun
            i = 1 olsun
            i <= 5 olduğu sürece {
                toplam = toplam + i olsun
                i = i + 1 olsun
            }
            toplam'ı yazdır
        "#,
        );
        assert_eq!(out.trim(), "15");
    }

    // ─────────────────────────────────────────────
    // Autograd unit testleri
    // ─────────────────────────────────────────────

    #[test]
    fn autograd_tensor_olustur() {
        let mut graf = autograd::AutogradGraph::new();
        let t = graf
            .tensor_olustur(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], true)
            .expect("Geçerli tensor oluşturulmalı");
        assert_eq!(t.satirlar, 2);
        assert_eq!(t.sutunlar, 3);
        assert!(t.requires_grad);
        let veri = t.veri.lock().unwrap();
        assert_eq!(*veri, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn autograd_topla_ileri() {
        let mut graf = autograd::AutogradGraph::new();
        let a = graf
            .tensor_olustur(1, 3, vec![1.0, 2.0, 3.0], true)
            .expect("Sol tensor oluşturulmalı");
        let b = graf
            .tensor_olustur(1, 3, vec![4.0, 5.0, 6.0], true)
            .expect("Sağ tensor oluşturulmalı");
        let c = graf.topla(&a, &b).expect("Tensorlar toplanmalı");
        let veri = c.veri.lock().unwrap();
        assert_eq!(*veri, vec![5.0, 7.0, 9.0]);
    }

    #[test]
    fn autograd_topla_geri_yayilim() {
        let mut graf = autograd::AutogradGraph::new();
        let a = graf
            .tensor_olustur(1, 2, vec![1.0, 2.0], true)
            .expect("Sol tensor oluşturulmalı");
        let b = graf
            .tensor_olustur(1, 2, vec![3.0, 4.0], true)
            .expect("Sağ tensor oluşturulmalı");
        let c = graf.topla(&a, &b).expect("Tensorlar toplanmalı");
        let c_id = c.id;
        let a_id = a.id;
        let b_id = b.id;
        graf.backward(c_id).unwrap();
        // Add geri yayılımında hem a hem b gradyanı 1.0 olmalı
        let ga: Vec<f64> = graf.nodes[&a_id].gradyan.lock().unwrap().clone();
        let gb: Vec<f64> = graf.nodes[&b_id].gradyan.lock().unwrap().clone();
        assert_eq!(ga, vec![1.0, 1.0]);
        assert_eq!(gb, vec![1.0, 1.0]);
    }

    #[test]
    fn autograd_matmul_ileri() {
        let mut graf = autograd::AutogradGraph::new();
        // 2x3 * 3x1 = 2x1
        let a = graf
            .tensor_olustur(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], true)
            .expect("Sol tensor oluşturulmalı");
        let b = graf
            .tensor_olustur(3, 1, vec![1.0, 0.0, -1.0], true)
            .expect("Sağ tensor oluşturulmalı");
        let c = graf.matris_carp(&a, &b).expect("MatMul başarısız oldu");
        let veri = c.veri.lock().unwrap();
        // [1*1+2*0+3*(-1), 4*1+5*0+6*(-1)] = [-2, -2]
        assert_eq!(*veri, vec![-2.0, -2.0]);
    }

    #[test]
    fn autograd_matmul_boyut_uyumsuzlugu() {
        let mut graf = autograd::AutogradGraph::new();
        let a = graf
            .tensor_olustur(2, 3, vec![0.0; 6], false)
            .expect("Sol tensor oluşturulmalı");
        let b = graf
            .tensor_olustur(2, 2, vec![0.0; 4], false)
            .expect("Sağ tensor oluşturulmalı");
        let sonuc = graf.matris_carp(&a, &b);
        assert!(sonuc.is_err(), "Boyut uyumsuzluğu hata döndürmeli");
    }

    #[test]
    fn autograd_relu_ileri() {
        let mut graf = autograd::AutogradGraph::new();
        let a = graf
            .tensor_olustur(1, 4, vec![-2.0, -1.0, 0.0, 3.0], true)
            .expect("Tensor oluşturulmalı");
        let r = graf.relu(&a).expect("ReLU çalışmalı");
        let veri = r.veri.lock().unwrap();
        assert_eq!(*veri, vec![0.0, 0.0, 0.0, 3.0]);
    }

    #[test]
    fn autograd_relu_geri_yayilim() {
        let mut graf = autograd::AutogradGraph::new();
        let a = graf
            .tensor_olustur(1, 4, vec![-1.0, 2.0, -3.0, 4.0], true)
            .expect("Tensor oluşturulmalı");
        let r = graf.relu(&a).expect("ReLU çalışmalı");
        let r_id = r.id;
        let a_id = a.id;
        graf.backward(r_id).unwrap();
        // ReLU: pozitif değerlerin gradyanı 1, negatiflerin 0
        let ga: Vec<f64> = graf.nodes[&a_id].gradyan.lock().unwrap().clone();
        assert_eq!(ga, vec![0.0, 1.0, 0.0, 1.0]);
    }

    #[test]
    fn autograd_statik_graf_alinabiilir() {
        // Global static AUTOGRAD_GRAF kilit alınabilmeli
        let _graf = autograd::AUTOGRAD_GRAF.lock().unwrap();
    }

    // ─────────────────────────────────────────────
    // Compiler + VM happy-path testleri
    // ─────────────────────────────────────────────

    #[test]
    fn compiler_vm_temel_aritmetik() {
        // Compiler → VM akışı panik atmadan tamamlanmalı
        let kod = "x = 3 + 4 olsun";
        let lx = lexer::Lexer::new(kod);
        let mut p = parser::Parser::new(lx);
        let (ast, diagnostics) = p.parse_program_with_diagnostics();
        assert!(diagnostics.is_empty());
        let mut dc = compiler::Derleyici::new();
        let prog = dc.derle_kontrollu(ast).expect("Bytecode derlenmeli");
        let mut vm = vm::VM::new(prog);
        vm.run_checked().expect("VM programı çalışmalı");
    }

    #[test]
    fn compiler_vm_yazdır() {
        let kod = "\"test\"'i yazdır";
        let lx = lexer::Lexer::new(kod);
        let mut p = parser::Parser::new(lx);
        let (ast, diagnostics) = p.parse_program_with_diagnostics();
        assert!(diagnostics.is_empty());
        let mut dc = compiler::Derleyici::new();
        let prog = dc.derle_kontrollu(ast).expect("Bytecode derlenmeli");
        let mut vm = vm::VM::new(prog);
        vm.run_checked().expect("VM programı çalışmalı");
    }

    #[test]
    fn compiler_vm_mantiksal_kisa_devre() {
        let kod = r#"
            sonuc = yanlış ve tanimsiz_degisken olsun
            diger = doğru veya baska_tanimsiz_degisken olsun
            sonuc'u yazdır
            diger'i yazdır
        "#;
        let mut parser = parser::Parser::new(lexer::Lexer::new(kod));
        let (ast, diagnostics) = parser.parse_program_with_diagnostics();
        assert!(diagnostics.is_empty());
        let program = compiler::Derleyici::new()
            .derle_kontrollu(ast)
            .expect("Mantıksal ifadeler bytecode'a derlenmeli");
        let buffer = Rc::new(RefCell::new(String::new()));
        let mut vm = vm::VM::new(program).with_output_buffer(Rc::clone(&buffer));
        vm.run_checked().expect("VM kısa devre uygulamalı");
        assert_eq!(buffer.borrow().trim(), "0\n1");
    }

    #[test]
    fn compiler_vm_sifira_bolmeyi_hata_yapar() {
        let mut parser = parser::Parser::new(lexer::Lexer::new("10 / 0'ı yazdır"));
        let (ast, diagnostics) = parser.parse_program_with_diagnostics();
        assert!(diagnostics.is_empty());
        let program = compiler::Derleyici::new()
            .derle_kontrollu(ast)
            .expect("Kaynak bytecode'a derlenmeli");
        let error = vm::VM::new(program)
            .run_checked()
            .expect_err("VM sıfıra bölmeyi reddetmeli");
        assert!(error.to_string().contains("Sıfıra bölme"));
    }

    #[test]
    fn yorumlayici_sonsuz_donguyu_adim_sinirinda_durdurur() {
        let mut parser = parser::Parser::new(lexer::Lexer::new("doğru olduğu sürece { }"));
        let (program, diagnostics) = parser.parse_program_with_diagnostics();
        assert!(diagnostics.is_empty());
        let limits = limits::ExecutionLimits {
            max_steps: 10,
            ..limits::ExecutionLimits::default()
        };
        let mut interpreter = interpreter::Yorumlayici::new()
            .with_limits(limits)
            .expect("Sınırlar geçerli olmalı");
        let error = interpreter
            .yorumla_kontrollu(program)
            .expect_err("Sonsuz döngü sınırda durmalı");
        assert!(error.to_string().contains("adım sınırı"));
    }

    #[test]
    fn vm_sonsuz_donguyu_adim_sinirinda_durdurur() {
        let mut parser = parser::Parser::new(lexer::Lexer::new("doğru olduğu sürece { }"));
        let (ast, diagnostics) = parser.parse_program_with_diagnostics();
        assert!(diagnostics.is_empty());
        let program = compiler::Derleyici::new()
            .derle_kontrollu(ast)
            .expect("Döngü bytecode'a derlenmeli");
        let limits = limits::ExecutionLimits {
            max_steps: 10,
            ..limits::ExecutionLimits::default()
        };
        let mut vm = vm::VM::new(program)
            .with_limits(limits)
            .expect("Sınırlar geçerli olmalı");
        let error = vm.run_checked().expect_err("Sonsuz döngü sınırda durmalı");
        assert!(error.to_string().contains("adım sınırı"));
    }

    #[test]
    fn cikti_siniri_iki_arka_ucta_uygulanir() {
        let source = r#""1234"'ü yazdır"#;
        let limits = limits::ExecutionLimits {
            max_output_bytes: 4,
            ..limits::ExecutionLimits::default()
        };
        let mut parser = parser::Parser::new(lexer::Lexer::new(source));
        let (ast, diagnostics) = parser.parse_program_with_diagnostics();
        assert!(diagnostics.is_empty());

        let mut interpreter = interpreter::Yorumlayici::new()
            .with_limits(limits)
            .expect("Sınırlar geçerli olmalı");
        assert!(interpreter
            .yorumla_kontrollu(ast.clone())
            .expect_err("Yorumlayıcı çıktı sınırını uygulamalı")
            .to_string()
            .contains("Çıktı sınırı"));

        let bytecode = compiler::Derleyici::new()
            .derle_kontrollu(ast)
            .expect("Kaynak bytecode'a derlenmeli");
        let mut vm = vm::VM::new(bytecode)
            .with_limits(limits)
            .expect("Sınırlar geçerli olmalı");
        assert!(vm
            .run_checked()
            .expect_err("VM çıktı sınırını uygulamalı")
            .to_string()
            .contains("Çıktı sınırı"));
    }
}
