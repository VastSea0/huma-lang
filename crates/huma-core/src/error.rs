//! Hüma structured error types.
//!
//! Library crates use [`HumaError`] (via `thiserror`) so that callers — the CLI
//! or the IDE — can pattern-match on the variant and decide how to present it
//! (coloured terminal output, JSON for IDEs, etc.).

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackFrame {
    pub function: String,
    pub location: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeDiagnostic {
    pub message: String,
    pub location: Option<SourceSpan>,
    pub stack: Vec<StackFrame>,
}

impl std::fmt::Display for RuntimeDiagnostic {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(location) = self.location {
            write!(
                formatter,
                "Satır {}, Sütun {}: {}",
                location.line, location.column, self.message
            )?;
        } else {
            write!(formatter, "{}", self.message)?;
        }
        if !self.stack.is_empty() {
            write!(formatter, "\nÇağrı izi: ")?;
            for (index, frame) in self.stack.iter().enumerate() {
                if index > 0 {
                    write!(formatter, " <- ")?;
                }
                write!(formatter, "{}", frame.function)?;
                if let Some(location) = frame.location {
                    write!(formatter, " ({}:{})", location.line, location.column)?;
                }
            }
        }
        Ok(())
    }
}

/// Top-level error type for the Hüma language toolkit.
#[derive(Debug, Error)]
pub enum HumaError {
    // ── Lexer / Parser ──────────────────────────────────────────────
    #[error("[Sözdizimi Hatası] Satır {line}, Sütun {col}: {message}")]
    SyntaxError {
        line: usize,
        col: usize,
        message: String,
    },

    // ── Runtime ─────────────────────────────────────────────────────
    #[error("[Çalışma Zamanı Hatası] {0}")]
    RuntimeError(RuntimeDiagnostic),

    // ── Compiler / Bytecode ─────────────────────────────────────────
    #[error("[Derleme Hatası] {0}")]
    CompileError(String),

    // ── I/O ─────────────────────────────────────────────────────────
    #[error("[Dosya Hatası] {0}")]
    IoError(#[from] std::io::Error),

    // ── Serialization ───────────────────────────────────────────────
    #[error("[Serileştirme Hatası] {0}")]
    SerializationError(String),
}

/// Convenience alias used throughout the core crate.
pub type HumaResult<T> = Result<T, HumaError>;

pub(crate) fn panik_mesaji(payload: Box<dyn std::any::Any + Send>) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|message| (*message).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "Bilinmeyen çalışma zamanı paniği".to_string())
}
