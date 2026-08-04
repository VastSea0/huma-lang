#![no_main]

use huma_bytecode::{validate_program, Program};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(program) = serde_json::from_slice::<Program>(data) {
        let _ = validate_program(&program);
    }
});
