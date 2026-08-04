//! Hüma bytecode yürütücüsü.
//!
//! Bu paket yorumlayıcıyla tam diferansiyel eşlik kanıtlanana kadar deneysel
//! backend'dir.

pub mod bytecode {
    pub use huma_bytecode::*;
}

pub mod token {
    pub use huma_syntax::token::*;
}

pub mod value {
    pub use huma_runtime::value::*;
}

pub mod limits {
    pub use huma_runtime::limits::*;
}

pub mod gc {
    pub use huma_runtime::gc::*;
}

pub mod semantics {
    pub use huma_runtime::semantics::*;
}

pub mod capability {
    pub use huma_runtime::capability::*;
}

pub mod interpreter {
    pub use huma_runtime::interpreter::*;
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

mod vm;

pub use vm::{NativeCallHost, TaskHost, VM};
