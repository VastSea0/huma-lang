//! # Hüma Compiler
//!
//! High-level compilation operations: parse → compile → validated bytecode,
//! plus the explicitly limited Cranelift AOT path.

pub mod aot;
pub mod pipeline;

#[cfg(test)]
mod tests {
    use super::*;
    use huma_core::{
        compiler::Derleyici, interpreter::Yorumlayici, lexer::Lexer, parser::Parser, vm::VM,
    };
    use std::{cell::RefCell, rc::Rc};

    fn compile_and_run(kod: &str) {
        let lexer = Lexer::new(kod);
        let mut parser = Parser::new(lexer);
        let (ast, diagnostics) = parser.parse_program_with_diagnostics();
        assert!(diagnostics.is_empty());
        let mut derleyici = Derleyici::new();
        let prog = derleyici
            .derle_kontrollu(ast)
            .expect("Kaynak bytecode'a derlenmeli");
        let mut vm = VM::new(prog);
        vm.run_checked().expect("Bytecode hatasız çalışmalı");
    }

    fn vm_output(kod: &str) -> String {
        let program = pipeline::compile_source(kod).expect("kaynak bytecode'a derlenmeli");
        let output = Rc::new(RefCell::new(String::new()));
        let mut vm = VM::new(program).with_output_buffer(output.clone());
        vm.run_checked().expect("VM programı hatasız çalışmalı");
        let result = output.borrow().clone();
        result
    }

    fn yorumlayici_sonucu(kod: &str) -> Result<String, String> {
        let mut parser = Parser::new(Lexer::new(kod));
        let (program, diagnostics) = parser.parse_program_with_diagnostics();
        if let Some(error) = diagnostics.into_iter().next() {
            return Err(error.to_string());
        }
        let output = Rc::new(RefCell::new(String::new()));
        let mut yorumlayici = Yorumlayici::new().with_output_buffer(output.clone());
        yorumlayici
            .yorumla_kontrollu(program)
            .map_err(|error| error.to_string())?;
        let result = output.borrow().clone();
        Ok(result)
    }

    fn vm_sonucu(kod: &str) -> Result<String, String> {
        let program = pipeline::compile_source(kod).map_err(|error| error.to_string())?;
        let output = Rc::new(RefCell::new(String::new()));
        let mut vm = VM::new(program).with_output_buffer(output.clone());
        vm.run_checked().map_err(|error| error.to_string())?;
        let result = output.borrow().clone();
        Ok(result)
    }

    fn arka_uclar_esit(kod: &str) {
        assert_eq!(
            yorumlayici_sonucu(kod),
            vm_sonucu(kod),
            "yorumlayıcı ve VM farklı sonuç üretti"
        );
    }

    // ─────────────────────────────────────────────
    // pipeline::compile_source testleri
    // ─────────────────────────────────────────────

    #[test]
    fn pipeline_compile_source_temel() {
        let sonuc = pipeline::compile_source("x = 42 olsun");
        assert!(sonuc.is_ok(), "Temel kaynak derlenmeli: {:?}", sonuc);
    }

    #[test]
    fn pipeline_compile_source_fonksiyon() {
        let kod = r#"
            topla fonksiyon olsun a, b alsın {
                a + b'yi döndür
            }
            topla(3, 4)'ü yazdır
        "#;
        let sonuc = pipeline::compile_source(kod);
        assert!(sonuc.is_ok(), "Fonksiyonlu kaynak derlenmeli: {:?}", sonuc);
    }

    #[test]
    fn pipeline_compile_source_dongu() {
        let kod = r#"
            i = 0 olsun
            i < 3 olduğu sürece {
                i = i + 1 olsun
            }
        "#;
        let sonuc = pipeline::compile_source(kod);
        assert!(sonuc.is_ok(), "Döngülü kaynak derlenmeli: {:?}", sonuc);
    }

    // ─────────────────────────────────────────────
    // Compiler → VM happy-path testleri
    // ─────────────────────────────────────────────

    #[test]
    fn compiler_vm_sayi_yazdirma() {
        // Panik atmadan çalışmalı
        compile_and_run("42'yi yazdır");
    }

    #[test]
    fn compiler_vm_degisken_atama() {
        compile_and_run("x = 100 olsun");
    }

    #[test]
    fn compiler_vm_fonksiyon_cagrisi() {
        let kod = r#"
            merhaba fonksiyon olsun {
                "Selam!"'ı yazdır
            }
            merhaba()
        "#;
        compile_and_run(kod);
    }

    #[test]
    fn compiler_vm_ic_ice_fonksiyon() {
        let kod = r#"
            ic fonksiyon olsun { "iç"'i yazdır }
            dis fonksiyon olsun { ic() }
            dis()
        "#;
        compile_and_run(kod);
    }

    #[test]
    fn compiler_derle_kontrollu_basarili() {
        let lexer = Lexer::new("x = 5 olsun");
        let mut parser = Parser::new(lexer);
        let ast = parser.parse_program();
        let mut derleyici = Derleyici::new();
        let sonuc = derleyici.derle_kontrollu(ast);
        assert!(sonuc.is_ok(), "Kontrollü derleme başarılı olmalı");
    }

    #[test]
    fn compiler_vm_ozyinelemeli_fibonacci_dogru() {
        let kod = r#"
            fibonacci fonksiyon olsun n alsın {
                n <= 1 ise { n'i döndür }
                fibonacci(n - 1) + fibonacci(n - 2)'yi döndür
            }
            fibonacci(0)'ı yazdır
            fibonacci(1)'i yazdır
            fibonacci(2)'yi yazdır
            fibonacci(7)'yi yazdır
        "#;
        assert_eq!(vm_output(kod), "0\n1\n1\n13\n");
    }

    #[test]
    fn compiler_vm_closure_bytecode_olarak_kapsam_yakalar() {
        let kod = r#"
            toplayici_uret fonksiyon olsun taban alsın {
                toplayici = fonksiyon olsun deger alsın {
                    taban + deger'i döndür
                } olsun
                toplayici'yi döndür
            }
            on_ekle = toplayici_uret(10) olsun
            on_ekle(5)'i yazdır
        "#;
        arka_uclar_esit(kod);
        let program = pipeline::compile_source(kod).expect("Closure bytecode'a derlenmeli");
        assert_eq!(
            program.functions.len(),
            2,
            "İç ve dış fonksiyonlar düz fonksiyon tablosunda bulunmalı"
        );
        let output = Rc::new(RefCell::new(String::new()));
        let mut vm = VM::new(program).with_output_buffer(output.clone());
        vm.run_checked().expect("Bytecode closure çalışmalı");
        assert_eq!(output.borrow().as_str(), "15\n");
    }

    #[test]
    fn compiler_vm_hata_izi_fonksiyon_cercevelerini_gosterir() {
        let kod = r#"
            ic fonksiyon olsun { 1 / 0'ı döndür }
            dis fonksiyon olsun { ic()'i döndür }
            dis()
        "#;
        let program = pipeline::compile_source(kod).expect("Kaynak bytecode'a derlenmeli");
        let error = VM::new(program)
            .run_checked()
            .expect_err("Sıfıra bölme hata vermeli")
            .to_string();
        assert!(error.contains("Çağrı izi: ic"), "{error}");
        assert!(error.contains("<- dis"), "{error}");
    }

    #[test]
    fn yorumlayici_vm_normatif_deger_semantiginde_esittir() {
        for kod in [
            r#"{} ise { "dolu"'yu yazdır } yoksa { "boş"'u yazdır }"#,
            r#""2" < "10"'u yazdır"#,
            r#""değer: " + 42'yi yazdır"#,
            r#"f fonksiyon olsun x alsın { x'i döndür } f()'ı yazdır"#,
            r#"{1: "geçersiz"}'i yazdır"#,
            r#""2" * 3'ü yazdır"#,
            r#"1e308 * 1e308'i yazdır"#,
        ] {
            arka_uclar_esit(kod);
        }
    }

    #[test]
    fn pipeline_desteklenmeyen_komutu_reddeder() {
        let sonuc = pipeline::compile_source(r#""matematik.hb"'yi yükle"#);
        assert!(
            sonuc.is_err(),
            "Bytecode modül yüklemeyi sessizce yutmamalı"
        );
    }

    #[test]
    fn pipeline_desteklenmeyen_liste_atamasini_reddeder() {
        let error = pipeline::compile_source("dizi = [1] olsun\ndizi[0] = 2 olsun")
            .expect_err("Liste ataması bytecode alt kümesinde reddedilmeli");
        assert!(error.to_string().contains("atama hedefini desteklemiyor"));
    }
}
