//! # Hüma Core
//!
//! Core language primitives for the Hüma programming language:
//! tokens, AST definitions, lexer, parser, values, bytecode,
//! bytecode compiler, virtual machine, and tree-walking interpreter.

pub mod error;
pub mod token;
pub mod lexer;
pub mod ast;
pub mod parser;
pub mod value;
pub mod interpreter;
pub mod bytecode;
pub mod compiler;
pub mod vm;
pub mod gui;
pub mod ffi;
pub mod autograd;
pub mod tokenizer;
pub mod builtin_files;

/// Re-export most-used items at the crate root for convenience.
pub use error::{HumaError, HumaResult};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derin_rekursiyon_hatasi() {
        let kod = "rekursiyon fonksiyon olsun { rekursiyon() } rekursiyon()";
        let mut interp = interpreter::Yorumlayici::new();
        let lexer = lexer::Lexer::new(kod);
        let mut parser = parser::Parser::new(lexer);
        let program = parser.parse_program();
        interp.yorumla(program);
        assert_eq!(interp.call_depth, 0);
    }

    #[test]
    fn test_cagrilamayan_deger_hatasi() {
        let kod = "x = 42 olsun\nx()";
        let mut interp = interpreter::Yorumlayici::new();
        let lexer = lexer::Lexer::new(kod);
        let mut parser = parser::Parser::new(lexer);
        let program = parser.parse_program();
        interp.yorumla(program);
    }

    #[test]
    fn test_vm_fonksiyon_cagrisi() {
        let kod = "yardimci fonksiyon olsun { \"Merhaba\"'yı yazdır }\nselamla fonksiyon olsun { yardimci() }\nselamla()";
        let lexer = lexer::Lexer::new(kod);
        let mut parser = parser::Parser::new(lexer);
        let ast = parser.parse_program();
        let mut derleyici = compiler::Derleyici::new();
        let prog = derleyici.derle(ast);
        let mut vm = vm::VM::new(prog);
        vm.run();
    }
}

