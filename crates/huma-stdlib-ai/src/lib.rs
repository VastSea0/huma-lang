//! Deneysel AI/autograd/tokenizer adaptörü.
//!
//! Bu crate kararlı dil zemininin parçası değildir ve yeniden yazılabilir.

pub mod autograd;
pub mod tokenizer;

mod builtins;

pub use builtins::kayit_et;
