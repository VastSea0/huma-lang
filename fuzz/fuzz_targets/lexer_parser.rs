#![no_main]

use huma_core::lexer::Lexer;
use huma_core::parser::Parser;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let source = String::from_utf8_lossy(data);
    let mut parser = Parser::new(Lexer::new(&source));
    let _ = parser.parse_program_with_diagnostics();
});
