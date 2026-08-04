//! Hüma'nın normatif yorumlayıcısı ve çalışma zamanı sınırı.
//!
//! Alan yerleşikleri 0.7 geçişinde ayrı adaptör paketlerine çıkarılacaktır.

pub mod capability;
pub mod gc;
pub mod interpreter;
pub mod isolate;
pub mod limits;
pub mod semantics;
pub mod value;

pub mod ast {
    pub use huma_syntax::ast::*;
}

pub mod lexer {
    pub use huma_syntax::lexer::*;
}

pub mod parser {
    pub use huma_syntax::parser::*;
}

pub mod token {
    pub use huma_syntax::token::*;
}

pub mod bytecode {
    pub use huma_bytecode::*;
}

pub mod builtin_files {
    pub use huma_stdlib::*;
}

pub mod error {
    pub use huma_syntax::error::*;

    pub(crate) fn panik_mesaji(payload: Box<dyn std::any::Any + Send>) -> String {
        payload
            .downcast_ref::<&str>()
            .map(|message| (*message).to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "Bilinmeyen çalışma zamanı paniği".to_string())
    }
}

pub use huma_syntax::{HumaError, HumaResult, RuntimeDiagnostic, SourceSpan, StackFrame};
