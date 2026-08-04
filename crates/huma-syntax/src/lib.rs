//! Hüma'nın alan bağımsız Türkçe kaynak dili katmanı.
//!
//! Bu paket runtime, GUI, ağ, SQL veya alan kütüphanesi bağımlılığı taşımaz.

pub mod ast;
pub mod error;
pub mod lexer;
pub mod morphology;
pub mod parser;
pub mod token;

pub use error::{
    DiagnosticCause, DiagnosticEnvelope, ErrorCategory, HumaError, HumaResult, RuntimeDiagnostic,
    SourceSpan, StackFrame,
};
