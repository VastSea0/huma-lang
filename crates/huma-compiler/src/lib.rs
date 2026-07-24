//! # Hüma Compiler
//!
//! High-level compilation operations: parse → compile → serialize to bytecode,
//! and the standalone-binary code-generation path.

pub mod pipeline;
pub mod codegen;

#[cfg(test)]
mod tests {
    use super::*;
    use huma_core::{lexer::Lexer, parser::Parser, compiler::Derleyici, vm::VM};

    fn compile_and_run(kod: &str) {
        let lexer = Lexer::new(kod);
        let mut parser = Parser::new(lexer);
        let ast = parser.parse_program();
        let mut derleyici = Derleyici::new();
        let prog = derleyici.derle(ast);
        let mut vm = VM::new(prog);
        vm.run();
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
}
