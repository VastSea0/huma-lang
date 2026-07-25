//! # Hüma Compiler
//!
//! High-level compilation operations: parse → compile → validated bytecode,
//! plus the explicitly limited Cranelift AOT path.

pub mod aot;
pub mod pipeline;

#[cfg(test)]
mod tests {
    use super::*;
    use huma_core::{compiler::Derleyici, lexer::Lexer, parser::Parser, vm::VM};
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
