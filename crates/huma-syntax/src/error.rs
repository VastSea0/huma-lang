//! Hüma structured error types.
//!
//! Library crates use [`HumaError`] (via `thiserror`) so that callers — the CLI
//! or the IDE — can pattern-match on the variant and decide how to present it
//! (coloured terminal output, JSON for IDEs, etc.).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

/// Makine tarafından kararlı biçimde eşleştirilebilen hata ailesi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    Syntax,
    Runtime,
    Compile,
    Io,
    Serialization,
}

/// IDE, CLI JSON çıktısı ve HMI istemcileri için sürümlü hata zarfı.
///
/// `message` yerelleştirilebilir; otomasyonlar `code` ve `category` alanlarını
/// kullanmalıdır.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticEnvelope {
    pub schema_version: u16,
    pub code: String,
    pub category: ErrorCategory,
    pub message: String,
    pub location: Option<SourceSpan>,
    pub stack: Box<[StackFrame]>,
    /// Otomasyonların güvenle tüketebileceği, serbest biçimli olmayan ek alanlar.
    #[serde(default, skip_serializing_if = "boxed_map_is_empty")]
    pub details: Box<BTreeMap<String, String>>,
    /// En yakın nedenden başlayarak sınırlı ve döngüsüz neden zinciri.
    #[serde(default, skip_serializing_if = "boxed_slice_is_empty")]
    pub causes: Box<[DiagnosticCause]>,
}

fn boxed_map_is_empty(values: &BTreeMap<String, String>) -> bool {
    values.is_empty()
}

fn boxed_slice_is_empty(values: &[DiagnosticCause]) -> bool {
    values.is_empty()
}

impl DiagnosticEnvelope {
    pub const SCHEMA_VERSION: u16 = 1;
}

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

/// Üst tanının altında taşınabilen, kendi içinde yeniden dallanmayan neden.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticCause {
    pub code: String,
    pub category: ErrorCategory,
    pub message: String,
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

impl HumaError {
    /// Sürümler arasında anlamı değiştirilmeyecek hata kodu.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::SyntaxError { .. } => "HUMA-SYNTAX-0001",
            Self::RuntimeError(_) => "HUMA-RUNTIME-0001",
            Self::CompileError(_) => "HUMA-COMPILE-0001",
            Self::IoError(_) => "HUMA-IO-0001",
            Self::SerializationError(_) => "HUMA-SERIALIZE-0001",
        }
    }

    pub const fn category(&self) -> ErrorCategory {
        match self {
            Self::SyntaxError { .. } => ErrorCategory::Syntax,
            Self::RuntimeError(_) => ErrorCategory::Runtime,
            Self::CompileError(_) => ErrorCategory::Compile,
            Self::IoError(_) => ErrorCategory::Io,
            Self::SerializationError(_) => ErrorCategory::Serialization,
        }
    }

    pub fn diagnostic(&self) -> DiagnosticEnvelope {
        let (message, location, stack) = match self {
            Self::SyntaxError { line, col, message } => (
                message.clone(),
                Some(SourceSpan {
                    line: *line,
                    column: *col,
                }),
                Vec::new(),
            ),
            Self::RuntimeError(diagnostic) => (
                diagnostic.message.clone(),
                diagnostic.location,
                diagnostic.stack.clone(),
            ),
            Self::CompileError(message) | Self::SerializationError(message) => {
                (message.clone(), None, Vec::new())
            }
            Self::IoError(error) => (error.to_string(), None, Vec::new()),
        };
        let mut details = BTreeMap::new();
        let causes = match self {
            Self::IoError(error) => {
                details.insert("io_kind".to_string(), format!("{:?}", error.kind()));
                vec![DiagnosticCause {
                    code: "HUMA-IO-CAUSE-0001".to_string(),
                    category: ErrorCategory::Io,
                    message: error.to_string(),
                    location: None,
                }]
            }
            _ => Vec::new(),
        };
        DiagnosticEnvelope {
            schema_version: DiagnosticEnvelope::SCHEMA_VERSION,
            code: self.code().to_string(),
            category: self.category(),
            message,
            location,
            stack: stack.into_boxed_slice(),
            details: Box::new(details),
            causes: causes.into_boxed_slice(),
        }
    }
}

/// Convenience alias used throughout the core crate.
pub type HumaResult<T> = Result<T, HumaError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tanilama_zarfi_metin_gosteriminden_bagimsizdir() {
        let error = HumaError::SyntaxError {
            line: 3,
            col: 7,
            message: "beklenmeyen sözcük".to_string(),
        };
        let diagnostic = error.diagnostic();
        assert_eq!(diagnostic.schema_version, 1);
        assert_eq!(diagnostic.code, "HUMA-SYNTAX-0001");
        assert_eq!(diagnostic.category, ErrorCategory::Syntax);
        assert_eq!(diagnostic.location, Some(SourceSpan { line: 3, column: 7 }));
        assert_eq!(diagnostic.message, "beklenmeyen sözcük");
    }

    #[test]
    fn tanilama_zarfi_bilinmeyen_alanlari_reddeder() {
        let json = r#"{
            "schema_version": 1,
            "code": "HUMA-RUNTIME-0001",
            "category": "runtime",
            "message": "hata",
            "location": null,
            "stack": [],
            "surpriz": true
        }"#;
        assert!(serde_json::from_str::<DiagnosticEnvelope>(json).is_err());
    }

    #[test]
    fn eski_v1_zarfi_yeni_istege_uyumludur() {
        let json = r#"{
            "schema_version": 1,
            "code": "HUMA-RUNTIME-0001",
            "category": "runtime",
            "message": "hata",
            "location": null,
            "stack": []
        }"#;
        let envelope: DiagnosticEnvelope = serde_json::from_str(json).unwrap();
        assert!(envelope.details.is_empty());
        assert!(envelope.causes.is_empty());
    }
}
