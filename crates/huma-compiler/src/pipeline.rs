//! Bytecode compilation pipeline.
//!
//! Orchestrates: source text → Lexer → Parser → Bytecode Compiler → versioned `.hbc`.

use crate::Derleyici;
use huma_bytecode::{validate_program, Program};
use huma_syntax::error::{HumaError, HumaResult};
use huma_syntax::lexer::Lexer;
use huma_syntax::parser::Parser;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

pub const BYTECODE_MAGIC: &[u8; 8] = b"HUMA-HBC";
pub const BYTECODE_FORMAT_VERSION: u16 = 4;
pub const MAX_BYTECODE_PAYLOAD_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_SOURCE_BYTES: usize = 64 * 1024 * 1024;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(1);

const BYTECODE_HEADER_BYTES: usize =
    BYTECODE_MAGIC.len() + std::mem::size_of::<u16>() + std::mem::size_of::<u64>() + 32;

fn read_file_limited(path: &str, limit: usize, kind: &str) -> HumaResult<Vec<u8>> {
    let file = fs::File::open(path)?;
    if let Ok(metadata) = file.metadata() {
        if metadata.len() > limit as u64 {
            return Err(HumaError::CompileError(format!(
                "{kind} {limit} bayt sınırını aşıyor: {path}"
            )));
        }
    }
    let mut bytes = Vec::new();
    file.take((limit as u64) + 1).read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(HumaError::CompileError(format!(
            "{kind} {limit} bayt sınırını aşıyor: {path}"
        )));
    }
    Ok(bytes)
}

pub fn read_source_file(path: &str) -> HumaResult<String> {
    String::from_utf8(read_file_limited(path, MAX_SOURCE_BYTES, "Kaynak dosyası")?)
        .map_err(|_| HumaError::CompileError(format!("Kaynak geçerli UTF-8 değil: {path}")))
}

fn atomic_write(path: &str, bytes: &[u8]) -> HumaResult<()> {
    let path = Path::new(path);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| HumaError::CompileError("Çıktı dosya adı geçerli UTF-8 değil".into()))?;
    let temporary = loop {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(
            ".{file_name}.tmp-{}-{sequence}",
            std::process::id()
        ));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => break (candidate, file),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    };
    let (temporary_path, mut temporary_file) = temporary;
    if let Err(error) = temporary_file
        .write_all(bytes)
        .and_then(|()| temporary_file.sync_all())
    {
        drop(temporary_file);
        let _ = fs::remove_file(&temporary_path);
        return Err(error.into());
    }
    drop(temporary_file);
    if fs::rename(&temporary_path, path).is_ok() {
        return Ok(());
    }
    if !path.exists() {
        let _ = fs::remove_file(&temporary_path);
        return Err(HumaError::CompileError(format!(
            "Bytecode çıktısı etkinleştirilemedi: {}",
            path.display()
        )));
    }
    let backup = (0..1_024).find_map(|_| {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(
            ".{file_name}.backup-{}-{sequence}",
            std::process::id()
        ));
        (!candidate.exists()).then_some(candidate)
    });
    let Some(backup) = backup else {
        let _ = fs::remove_file(&temporary_path);
        return Err(HumaError::CompileError(format!(
            "Bytecode çıktısı için benzersiz yedek yolu üretilemedi: {}",
            path.display()
        )));
    };
    if let Err(error) = fs::rename(path, &backup) {
        let _ = fs::remove_file(&temporary_path);
        return Err(error.into());
    }
    if let Err(error) = fs::rename(&temporary_path, path) {
        let restore = fs::rename(&backup, path);
        let _ = fs::remove_file(&temporary_path);
        return match restore {
            Ok(()) => Err(error.into()),
            Err(restore_error) => Err(HumaError::CompileError(format!(
                "Bytecode çıktısı yazılamadı ({error}) ve eski çıktı geri yüklenemedi \
                 ({restore_error}); yedek: {}",
                backup.display()
            ))),
        };
    }
    if let Err(error) = fs::remove_file(&backup) {
        eprintln!(
            "Uyarı: eski bytecode yedeği temizlenemedi ({}): {}",
            backup.display(),
            error
        );
    }
    Ok(())
}

/// Compile a `.hb` source file to an in-memory [`Program`].
pub fn compile_source(source: &str) -> HumaResult<Program> {
    if source.len() > MAX_SOURCE_BYTES {
        return Err(HumaError::CompileError(format!(
            "Kaynak {} bayt sınırını aşıyor",
            MAX_SOURCE_BYTES
        )));
    }
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
    validate_program(program)
}

/// Encode a program into the stable Hüma bytecode container.
pub fn encode_bytecode(program: &Program) -> HumaResult<Vec<u8>> {
    validate_bytecode(program)?;
    let payload = serde_json::to_vec(program).map_err(|error| {
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
    let max_container_bytes = BYTECODE_HEADER_BYTES
        .checked_add(MAX_BYTECODE_PAYLOAD_BYTES)
        .ok_or_else(|| HumaError::SerializationError("Bytecode boyut sınırı taştı".into()))?;
    if bytes.len() > max_container_bytes {
        return Err(HumaError::SerializationError(format!(
            "Bytecode dosyası {max_container_bytes} bayt sınırını aşıyor"
        )));
    }
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
    let version_bytes: [u8; std::mem::size_of::<u16>()] = bytes
        .get(version_offset..version_offset + std::mem::size_of::<u16>())
        .and_then(|slice| slice.try_into().ok())
        .ok_or_else(|| HumaError::SerializationError("Bytecode sürüm alanı eksik".to_string()))?;
    let version = u16::from_le_bytes(version_bytes);
    if version != BYTECODE_FORMAT_VERSION {
        return Err(HumaError::SerializationError(format!(
            "Desteklenmeyen bytecode biçim sürümü: {version}; beklenen: {BYTECODE_FORMAT_VERSION}"
        )));
    }

    let length_offset = version_offset + std::mem::size_of::<u16>();
    let length_bytes: [u8; std::mem::size_of::<u64>()] = bytes
        .get(length_offset..length_offset + std::mem::size_of::<u64>())
        .and_then(|slice| slice.try_into().ok())
        .ok_or_else(|| HumaError::SerializationError("Bytecode uzunluk alanı eksik".to_string()))?;
    let payload_len_u64 = u64::from_le_bytes(length_bytes);
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

    let program: Program = serde_json::from_slice(payload).map_err(|error| {
        HumaError::SerializationError(format!("Bytecode okuma hatası: {error}"))
    })?;
    validate_bytecode(&program)?;
    Ok(program)
}

/// Compile a `.hb` file and write the resulting bytecode to `output_path`.
pub fn compile_file(input_path: &str, output_path: &str) -> HumaResult<()> {
    let source = read_source_file(input_path)?;
    let program = compile_source(&source)?;
    let encoded = encode_bytecode(&program)?;
    atomic_write(output_path, &encoded)?;
    Ok(())
}

/// Load a previously compiled `.hbc` bytecode file.
pub fn load_bytecode(path: &str) -> HumaResult<Program> {
    let bytes = read_file_limited(
        path,
        BYTECODE_HEADER_BYTES + MAX_BYTECODE_PAYLOAD_BYTES,
        "Bytecode dosyası",
    )?;
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
    let source = read_source_file(input_path)?;
    let program = compile_source(&source)?;

    let result = CompileResult {
        input: input_path.to_string(),
        output: output_path.to_string(),
        bytecode_format_version: BYTECODE_FORMAT_VERSION,
        instruction_count: program.instructions.len(),
        constant_count: program.constants.len(),
    };

    let encoded = encode_bytecode(&program)?;
    atomic_write(output_path, &encoded)?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{
        compile_source, decode_bytecode, encode_bytecode, read_source_file,
        BYTECODE_FORMAT_VERSION, BYTECODE_MAGIC, MAX_SOURCE_BYTES,
    };
    use huma_bytecode::{OpCode, Program};
    use std::fs::File;

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
        let raw = serde_json::to_vec(&program).expect("ham test verisi üretilmeli");
        let error = decode_bytecode(&raw).expect_err("ham payload reddedilmeli");
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
            functions: Vec::new(),
            instructions: vec![OpCode::PushConstant(0)],
            instruction_spans: vec![None],
        };
        let error = encode_bytecode(&program).expect_err("geçersiz program reddedilmeli");
        assert!(error.to_string().contains("sabit indeksi"));
    }

    #[test]
    fn seyrek_buyuk_kaynak_dosyasi_bellege_alinmadan_reddedilir() {
        let temporary = tempfile::NamedTempFile::new().expect("Geçici dosya oluşturulmalı");
        File::options()
            .write(true)
            .open(temporary.path())
            .expect("Geçici dosya açılmalı")
            .set_len((MAX_SOURCE_BYTES as u64) + 1)
            .expect("Seyrek dosya boyutlandırılmalı");
        let path = temporary
            .path()
            .to_str()
            .expect("Geçici dosya yolu UTF-8 olmalı");
        let error = read_source_file(path).expect_err("Büyük kaynak reddedilmeli");
        assert!(error.to_string().contains("sınırını aşıyor"));
    }

    #[test]
    fn gecersiz_utf8_kaynak_reddedilir() {
        let mut temporary = tempfile::NamedTempFile::new().expect("Geçici dosya oluşturulmalı");
        std::io::Write::write_all(&mut temporary, &[0xff, 0xfe]).expect("Geçersiz UTF-8 yazılmalı");
        let path = temporary
            .path()
            .to_str()
            .expect("Geçici dosya yolu UTF-8 olmalı");
        let error = read_source_file(path).expect_err("Geçersiz UTF-8 reddedilmeli");
        assert!(error.to_string().contains("geçerli UTF-8 değil"));
    }
}
