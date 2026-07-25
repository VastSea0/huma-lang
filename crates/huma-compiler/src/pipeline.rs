//! Bytecode compilation pipeline.
//!
//! Orchestrates: source text → Lexer → Parser → Bytecode Compiler → versioned `.hbc`.

use bincode::Options;
use huma_core::bytecode::{Constant, OpCode, Program};
use huma_core::compiler::Derleyici;
use huma_core::error::{HumaError, HumaResult};
use huma_core::lexer::Lexer;
use huma_core::parser::Parser;
use sha2::{Digest, Sha256};
use std::fs;

pub const BYTECODE_MAGIC: &[u8; 8] = b"HUMA-HBC";
pub const BYTECODE_FORMAT_VERSION: u16 = 1;
pub const MAX_BYTECODE_PAYLOAD_BYTES: usize = 64 * 1024 * 1024;

const BYTECODE_HEADER_BYTES: usize =
    BYTECODE_MAGIC.len() + std::mem::size_of::<u16>() + std::mem::size_of::<u64>() + 32;
const MAX_CONSTANTS: usize = 1_000_000;
const MAX_INSTRUCTIONS: usize = 10_000_000;
const MAX_COLLECTION_LITERAL_ITEMS: usize = 1_000_000;
const MAX_CALL_ARGUMENTS: usize = 65_535;

fn serialization_options() -> impl Options {
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .with_limit(MAX_BYTECODE_PAYLOAD_BYTES as u64)
        .reject_trailing_bytes()
}

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

/// Validate structural invariants that the VM relies on.
///
/// Source compilation already emits these invariants. This separate boundary is
/// required because `.hbc` files may come from an untrusted or older producer.
pub fn validate_bytecode(program: &Program) -> HumaResult<()> {
    if program.constants.len() > MAX_CONSTANTS {
        return Err(HumaError::SerializationError(format!(
            "Bytecode sabit havuzu sınırı aşıyor: {} > {}",
            program.constants.len(),
            MAX_CONSTANTS
        )));
    }
    if program.instructions.len() > MAX_INSTRUCTIONS {
        return Err(HumaError::SerializationError(format!(
            "Bytecode komut sınırı aşıyor: {} > {}",
            program.instructions.len(),
            MAX_INSTRUCTIONS
        )));
    }

    for (index, constant) in program.constants.iter().enumerate() {
        if let Constant::Sayi(value) = constant {
            if !value.is_finite() {
                return Err(HumaError::SerializationError(format!(
                    "Bytecode sabiti {index} sonlu bir sayı değil"
                )));
            }
        }
    }

    let instruction_count = program.instructions.len();
    for (index, instruction) in program.instructions.iter().enumerate() {
        match instruction {
            OpCode::PushConstant(constant_index) if *constant_index >= program.constants.len() => {
                return Err(HumaError::SerializationError(format!(
                    "Bytecode komutu {index} geçersiz sabit indeksi kullanıyor: {constant_index}"
                )));
            }
            OpCode::Jump(target) | OpCode::JumpIfFalse(target) | OpCode::TryBlockStart(target)
                if *target > instruction_count =>
            {
                return Err(HumaError::SerializationError(format!(
                    "Bytecode komutu {index} geçersiz atlama hedefi kullanıyor: {target}"
                )));
            }
            OpCode::MakeList(item_count) | OpCode::MakeMap(item_count)
                if *item_count > MAX_COLLECTION_LITERAL_ITEMS =>
            {
                return Err(HumaError::SerializationError(format!(
                    "Bytecode komutu {index} koleksiyon sınırını aşıyor: {item_count}"
                )));
            }
            OpCode::Call(argument_count)
            | OpCode::CallFFI {
                arg_len: argument_count,
                ..
            } if *argument_count > MAX_CALL_ARGUMENTS => {
                return Err(HumaError::SerializationError(format!(
                    "Bytecode komutu {index} argüman sınırını aşıyor: {argument_count}"
                )));
            }
            OpCode::MakeFunction { name, params, body } => {
                if name.is_empty() {
                    return Err(HumaError::SerializationError(format!(
                        "Bytecode komutu {index} boş fonksiyon adı içeriyor"
                    )));
                }
                if params.len() > MAX_CALL_ARGUMENTS {
                    return Err(HumaError::SerializationError(format!(
                        "Bytecode fonksiyonu '{name}' parametre sınırını aşıyor: {}",
                        params.len()
                    )));
                }
                if body.len() > MAX_INSTRUCTIONS {
                    return Err(HumaError::SerializationError(format!(
                        "Bytecode fonksiyonu '{name}' gövde sınırını aşıyor: {}",
                        body.len()
                    )));
                }
            }
            _ => {}
        }
    }

    Ok(())
}

/// Encode a program into the stable Hüma bytecode container.
pub fn encode_bytecode(program: &Program) -> HumaResult<Vec<u8>> {
    validate_bytecode(program)?;
    let payload = serialization_options()
        .serialize(program)
        .map_err(|error| {
            HumaError::SerializationError(format!("Bytecode serileştirme hatası: {error}"))
        })?;
    if payload.len() > MAX_BYTECODE_PAYLOAD_BYTES {
        return Err(HumaError::SerializationError(format!(
            "Bytecode payload sınırı aşıyor: {} bayt",
            payload.len()
        )));
    }

    let digest = Sha256::digest(&payload);
    let mut encoded = Vec::with_capacity(BYTECODE_HEADER_BYTES + payload.len());
    encoded.extend_from_slice(BYTECODE_MAGIC);
    encoded.extend_from_slice(&BYTECODE_FORMAT_VERSION.to_le_bytes());
    encoded.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    encoded.extend_from_slice(&digest);
    encoded.extend_from_slice(&payload);
    Ok(encoded)
}

/// Decode and validate a versioned Hüma bytecode container.
pub fn decode_bytecode(bytes: &[u8]) -> HumaResult<Program> {
    if bytes.len() < BYTECODE_HEADER_BYTES {
        return Err(HumaError::SerializationError(
            "Bytecode başlığı eksik veya dosya kesilmiş".to_string(),
        ));
    }
    if &bytes[..BYTECODE_MAGIC.len()] != BYTECODE_MAGIC {
        return Err(HumaError::SerializationError(
            "Geçersiz bytecode imzası; ham veya eski .hbc biçimi desteklenmiyor".to_string(),
        ));
    }

    let version_offset = BYTECODE_MAGIC.len();
    let version = u16::from_le_bytes(
        bytes[version_offset..version_offset + std::mem::size_of::<u16>()]
            .try_into()
            .expect("sabit uzunluk doğrulandı"),
    );
    if version != BYTECODE_FORMAT_VERSION {
        return Err(HumaError::SerializationError(format!(
            "Desteklenmeyen bytecode biçim sürümü: {version}; beklenen: {BYTECODE_FORMAT_VERSION}"
        )));
    }

    let length_offset = version_offset + std::mem::size_of::<u16>();
    let payload_len_u64 = u64::from_le_bytes(
        bytes[length_offset..length_offset + std::mem::size_of::<u64>()]
            .try_into()
            .expect("sabit uzunluk doğrulandı"),
    );
    let payload_len = usize::try_from(payload_len_u64).map_err(|_| {
        HumaError::SerializationError("Bytecode payload uzunluğu bu platformda taşma yaptı".into())
    })?;
    if payload_len > MAX_BYTECODE_PAYLOAD_BYTES {
        return Err(HumaError::SerializationError(format!(
            "Bytecode payload sınırı aşıyor: {payload_len} bayt"
        )));
    }
    let expected_total = BYTECODE_HEADER_BYTES
        .checked_add(payload_len)
        .ok_or_else(|| HumaError::SerializationError("Bytecode uzunluğu taşma yaptı".into()))?;
    if bytes.len() != expected_total {
        return Err(HumaError::SerializationError(format!(
            "Bytecode uzunluğu uyuşmuyor: başlık {expected_total}, dosya {} bayt",
            bytes.len()
        )));
    }

    let digest_offset = length_offset + std::mem::size_of::<u64>();
    let payload_offset = digest_offset + 32;
    let expected_digest = &bytes[digest_offset..payload_offset];
    let payload = &bytes[payload_offset..];
    let actual_digest = Sha256::digest(payload);
    if expected_digest != actual_digest.as_slice() {
        return Err(HumaError::SerializationError(
            "Bytecode bütünlük özeti uyuşmuyor".to_string(),
        ));
    }

    let program: Program = serialization_options()
        .deserialize(payload)
        .map_err(|error| {
            HumaError::SerializationError(format!("Bytecode okuma hatası: {error}"))
        })?;
    validate_bytecode(&program)?;
    Ok(program)
}

/// Compile a `.hb` file and write the resulting bytecode to `output_path`.
pub fn compile_file(input_path: &str, output_path: &str) -> HumaResult<()> {
    let source = fs::read_to_string(input_path)?;
    let program = compile_source(&source)?;
    let encoded = encode_bytecode(&program)?;
    fs::write(output_path, encoded)?;
    Ok(())
}

/// Load a previously compiled `.hbc` bytecode file.
pub fn load_bytecode(path: &str) -> HumaResult<Program> {
    let bytes = fs::read(path)?;
    decode_bytecode(&bytes)
}

/// Diagnostic information returned after a successful compilation.
#[derive(Debug, serde::Serialize)]
pub struct CompileResult {
    pub input: String,
    pub output: String,
    pub bytecode_format_version: u16,
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
        bytecode_format_version: BYTECODE_FORMAT_VERSION,
        instruction_count: program.instructions.len(),
        constant_count: program.constants.len(),
    };

    let encoded = encode_bytecode(&program)?;
    fs::write(output_path, encoded)?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{
        compile_source, decode_bytecode, encode_bytecode, BYTECODE_FORMAT_VERSION, BYTECODE_MAGIC,
    };
    use huma_core::bytecode::{OpCode, Program};

    #[test]
    fn bytecode_container_round_trip() {
        let program = compile_source("x = 42 olsun\nx'i yazdır").expect("kaynak derlenmeli");
        let encoded = encode_bytecode(&program).expect("bytecode kodlanmalı");

        assert_eq!(&encoded[..BYTECODE_MAGIC.len()], BYTECODE_MAGIC);
        let decoded = decode_bytecode(&encoded).expect("bytecode çözülmeli");
        assert_eq!(decoded.instructions.len(), program.instructions.len());
        assert_eq!(decoded.constants.len(), program.constants.len());
    }

    #[test]
    fn bytecode_container_eski_ham_bicimi_reddeder() {
        let program = compile_source("42'yi yazdır").expect("kaynak derlenmeli");
        let raw = bincode::serialize(&program).expect("ham test verisi üretilmeli");
        let error = decode_bytecode(&raw).expect_err("ham bincode reddedilmeli");
        let message = error.to_string();
        assert!(
            message.contains("Bytecode başlığı") || message.contains("bytecode imzası"),
            "beklenmeyen hata: {message}"
        );
    }

    #[test]
    fn bytecode_container_bilinmeyen_surumu_reddeder() {
        let program = compile_source("42'yi yazdır").expect("kaynak derlenmeli");
        let mut encoded = encode_bytecode(&program).expect("bytecode kodlanmalı");
        let offset = BYTECODE_MAGIC.len();
        encoded[offset..offset + 2]
            .copy_from_slice(&BYTECODE_FORMAT_VERSION.saturating_add(1).to_le_bytes());

        let error = decode_bytecode(&encoded).expect_err("bilinmeyen sürüm reddedilmeli");
        assert!(error.to_string().contains("biçim sürümü"));
    }

    #[test]
    fn bytecode_container_manipulasyonu_yakalar() {
        let program = compile_source("42'yi yazdır").expect("kaynak derlenmeli");
        let mut encoded = encode_bytecode(&program).expect("bytecode kodlanmalı");
        let last = encoded.last_mut().expect("payload olmalı");
        *last ^= 0x01;

        let error = decode_bytecode(&encoded).expect_err("değiştirilmiş dosya reddedilmeli");
        assert!(error.to_string().contains("bütünlük"));
    }

    #[test]
    fn bytecode_container_gecersiz_sabit_indeksini_reddeder() {
        let program = Program {
            constants: Vec::new(),
            instructions: vec![OpCode::PushConstant(0)],
        };
        let error = encode_bytecode(&program).expect_err("geçersiz program reddedilmeli");
        assert!(error.to_string().contains("sabit indeksi"));
    }
}
