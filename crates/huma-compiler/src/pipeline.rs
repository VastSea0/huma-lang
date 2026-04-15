//! Bytecode compilation pipeline.
//!
//! Orchestrates: source text → Lexer → Parser → Bytecode Compiler → serialized `.hbc`.

use huma_core::bytecode::Program;
use huma_core::compiler::Derleyici;
use huma_core::error::{HumaError, HumaResult};
use huma_core::lexer::Lexer;
use huma_core::parser::Parser;
use std::fs;

/// Compile a `.hb` source file to an in-memory [`Program`].
pub fn compile_source(source: &str) -> HumaResult<Program> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let (ast, diagnostics) = parser.parse_program_with_diagnostics();
    if let Some(first) = diagnostics.into_iter().next() {
        return Err(first);
    }

    let mut compiler = Derleyici::new();
    let program = compiler
        .derle_kontrollu(ast)
        .map_err(HumaError::CompileError)?;
    Ok(program)
}

/// Compile a `.hb` file and write the resulting bytecode to `output_path`.
pub fn compile_file(input_path: &str, output_path: &str) -> HumaResult<()> {
    let source = fs::read_to_string(input_path)?;
    let program = compile_source(&source)?;

    let encoded: Vec<u8> = bincode::serialize(&program).map_err(|e| {
        HumaError::SerializationError(format!("Bytecode serileştirme hatası: {}", e))
    })?;

    fs::write(output_path, encoded)?;

    Ok(())
}

/// Load a previously compiled `.hbc` bytecode file.
pub fn load_bytecode(path: &str) -> HumaResult<Program> {
    let bytes = fs::read(path)?;
    let program: Program = bincode::deserialize(&bytes).map_err(|e| {
        HumaError::SerializationError(format!("Bytecode okuma hatası: {}", e))
    })?;
    Ok(program)
}

/// Diagnostic information returned after a successful compilation.
#[derive(Debug, serde::Serialize)]
pub struct CompileResult {
    pub input: String,
    pub output: String,
    pub instruction_count: usize,
    pub constant_count: usize,
}

/// Compile with full diagnostics (suitable for `--json` flag output).
pub fn compile_with_diagnostics(input_path: &str, output_path: &str) -> HumaResult<CompileResult> {
    let source = fs::read_to_string(input_path)?;
    let program = compile_source(&source)?;

    let result = CompileResult {
        input: input_path.to_string(),
        output: output_path.to_string(),
        instruction_count: program.instructions.len(),
        constant_count: program.constants.len(),
    };

    let encoded: Vec<u8> = bincode::serialize(&program).map_err(|e| {
        HumaError::SerializationError(format!("Bytecode serileştirme hatası: {}", e))
    })?;
    fs::write(output_path, encoded)?;

    Ok(result)
}
