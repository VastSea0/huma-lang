//! # Hüma Core
//!
//! Core language primitives for the Hüma programming language:
//! tokens, AST definitions, lexer, parser, values, bytecode,
//! bytecode compiler, virtual machine, and tree-walking interpreter.

pub mod ast;
pub mod autograd;
pub mod builtin_files;
pub mod bytecode;
pub mod compiler;
pub mod error;
pub mod ffi;
pub mod gui;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod token;
pub mod tokenizer;
pub mod value;
pub mod vm;

/// Re-export most-used items at the crate root for convenience.
pub use error::{HumaError, HumaResult};

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    // ─────────────────────────────────────────────
    // Yardımcı: kodu yorumlayıp string çıktısını döndürür
    // ─────────────────────────────────────────────
    fn eval(kod: &str) -> String {
        let buf = Rc::new(RefCell::new(String::new()));
        let mut yorumlayici = interpreter::Yorumlayici::new().with_output_buffer(Rc::clone(&buf));
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
        let t = graf.tensor_olustur(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], true);
        assert_eq!(t.satirlar, 2);
        assert_eq!(t.sutunlar, 3);
        assert!(t.requires_grad);
        let veri = t.veri.lock().unwrap();
        assert_eq!(*veri, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn autograd_topla_ileri() {
        let mut graf = autograd::AutogradGraph::new();
        let a = graf.tensor_olustur(1, 3, vec![1.0, 2.0, 3.0], true);
        let b = graf.tensor_olustur(1, 3, vec![4.0, 5.0, 6.0], true);
        let c = graf.topla(&a, &b);
        let veri = c.veri.lock().unwrap();
        assert_eq!(*veri, vec![5.0, 7.0, 9.0]);
    }

    #[test]
    fn autograd_topla_geri_yayilim() {
        let mut graf = autograd::AutogradGraph::new();
        let a = graf.tensor_olustur(1, 2, vec![1.0, 2.0], true);
        let b = graf.tensor_olustur(1, 2, vec![3.0, 4.0], true);
        let c = graf.topla(&a, &b);
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
        let a = graf.tensor_olustur(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], true);
        let b = graf.tensor_olustur(3, 1, vec![1.0, 0.0, -1.0], true);
        let c = graf.matris_carp(&a, &b).expect("MatMul başarısız oldu");
        let veri = c.veri.lock().unwrap();
        // [1*1+2*0+3*(-1), 4*1+5*0+6*(-1)] = [-2, -2]
        assert_eq!(*veri, vec![-2.0, -2.0]);
    }

    #[test]
    fn autograd_matmul_boyut_uyumsuzlugu() {
        let mut graf = autograd::AutogradGraph::new();
        let a = graf.tensor_olustur(2, 3, vec![0.0; 6], false);
        let b = graf.tensor_olustur(2, 2, vec![0.0; 4], false);
        let sonuc = graf.matris_carp(&a, &b);
        assert!(sonuc.is_err(), "Boyut uyumsuzluğu hata döndürmeli");
    }

    #[test]
    fn autograd_relu_ileri() {
        let mut graf = autograd::AutogradGraph::new();
        let a = graf.tensor_olustur(1, 4, vec![-2.0, -1.0, 0.0, 3.0], true);
        let r = graf.relu(&a);
        let veri = r.veri.lock().unwrap();
        assert_eq!(*veri, vec![0.0, 0.0, 0.0, 3.0]);
    }

    #[test]
    fn autograd_relu_geri_yayilim() {
        let mut graf = autograd::AutogradGraph::new();
        let a = graf.tensor_olustur(1, 4, vec![-1.0, 2.0, -3.0, 4.0], true);
        let r = graf.relu(&a);
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
}
