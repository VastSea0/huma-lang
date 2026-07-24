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
pub mod builtin_files;

/// Re-export most-used items at the crate root for convenience.
pub use error::{HumaError, HumaResult};
