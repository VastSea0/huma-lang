//! Implementations for each CLI subcommand.

use anyhow::{Context, Result};
use colored::Colorize;
use huma_core::interpreter::Yorumlayici;
use huma_core::lexer::Lexer;
use huma_core::parser::Parser;
use huma_core::vm::VM;
use std::io::{self, BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

const MAX_TEST_OUTPUT_BYTES: usize = 16 * 1024 * 1024;
const MAX_REPL_INPUT_BYTES: usize = 1024 * 1024;
const MAX_TEST_FILES: usize = 10_000;

/// Run a `.hb` source file through the tree-walking interpreter.
pub fn run_file(path: &str) -> Result<()> {
    let source = huma_compiler::pipeline::read_source_file(path)
        .with_context(|| format!("'{}' dosyası okunamadı", path))?;

    let interp = Yorumlayici::new();
    let mut interp = interp;
    execute_source(&source, &mut interp)?;

    // GUI isteği var mı kontrol et
    if huma_core::gui::gui_istegi_var_mi().map_err(anyhow::Error::msg)? {
        huma_core::gui::gui_calistir(interp).map_err(anyhow::Error::msg)?;
    }

    Ok(())
}

/// Run a `.hb` source file by compiling to Bytecode and executing in Bytecode VM.
pub fn run_vm_file(path: &str) -> Result<()> {
    let source = huma_compiler::pipeline::read_source_file(path)
        .with_context(|| format!("'{}' dosyası okunamadı", path))?;

    let lexer = Lexer::new(&source);
    let mut parser = Parser::new(lexer);
    let (program_ast, diagnostics) = parser.parse_program_with_diagnostics();
    if let Some(first) = diagnostics.into_iter().next() {
        return Err(first.into());
    }

    let mut derleyici = huma_core::compiler::Derleyici::new();
    let bytecode_prog = derleyici
        .derle_kontrollu(program_ast)
        .map_err(huma_core::HumaError::CompileError)?;

    let mut vm = VM::new(bytecode_prog);
    vm.run_checked()?;
    Ok(())
}

/// Compile a `.hb` file to bytecode.
pub fn build_file(input: &str, output: &str, json_output: bool) -> Result<()> {
    if json_output {
        let result = huma_compiler::pipeline::compile_with_diagnostics(input, output)
            .with_context(|| format!("'{}' dosyası derlenirken hata oluştu", input))?;
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        huma_compiler::pipeline::compile_file(input, output)
            .with_context(|| format!("'{}' dosyası derlenirken hata oluştu", input))?;
        println!(
            "{} {} dosyası {} olarak derlendi.",
            "[Başarı]".bright_green().bold(),
            input.bright_white(),
            output.bright_cyan(),
        );
    }
    Ok(())
}

/// Execute a pre-compiled `.hbc` bytecode file in the VM.
pub fn exec_bytecode(path: &str) -> Result<()> {
    let program = huma_compiler::pipeline::load_bytecode(path)
        .with_context(|| format!("'{}' bytecode dosyası yüklenirken hata oluştu", path))?;

    let mut vm = VM::new(program);
    vm.run_checked()?;
    Ok(())
}

/// Compile a `.hb` file to a native machine code binary using Cranelift AOT.
pub fn compile_aot(input: &str, output_name: &str, opt_level: u8) -> Result<()> {
    let source = huma_compiler::pipeline::read_source_file(input)
        .with_context(|| format!("'{}' dosyası okunamadı", input))?;

    let out_path = std::path::Path::new(output_name);
    let opts = huma_compiler::aot::AotOptions {
        output_bin: out_path,
        opt_level,
    };

    huma_compiler::aot::compile_to_binary(&source, &opts)
        .with_context(|| format!("'{}' Cranelift AOT derlemesi sırasında hata oluştu", input))?;

    println!(
        "{} Native binary üretildi: {}",
        "[Başarı]".bright_green().bold(),
        output_name.bright_cyan().bold(),
    );
    Ok(())
}

/// Start the interactive REPL.
pub fn start_repl() -> Result<()> {
    println!(
        "\n{}  {}",
        "🌙 Hüma Programlama Dili".bright_cyan().bold(),
        format!("v{}", env!("CARGO_PKG_VERSION")).dimmed(),
    );
    println!(
        "{}",
        "   Etkileşimli REPL — Çıkmak için 'çıkış' veya 'exit' yazın.".dimmed()
    );
    println!();

    let mut interp = Yorumlayici::new();
    let mut input = String::new();
    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    loop {
        print!("{} ", "hüma ❯".bright_cyan().bold());
        io::stdout().flush()?;

        input.clear();
        let bytes_read = {
            let mut limited = (&mut stdin).take((MAX_REPL_INPUT_BYTES as u64) + 1);
            limited.read_line(&mut input)?
        };
        if bytes_read == 0 {
            break;
        }
        if input.len() > MAX_REPL_INPUT_BYTES {
            loop {
                let available = stdin.fill_buf()?;
                if available.is_empty() {
                    break;
                }
                if let Some(newline) = available.iter().position(|byte| *byte == b'\n') {
                    stdin.consume(newline + 1);
                    break;
                }
                let length = available.len();
                stdin.consume(length);
            }
            eprintln!(
                "REPL girdisi {} bayt sınırını aşıyor.",
                MAX_REPL_INPUT_BYTES
            );
            continue;
        }

        let trimmed = input.trim();
        if trimmed == "çıkış" || trimmed == "exit" {
            println!("{}", "Görüşmek üzere! 👋".dimmed());
            break;
        }
        if trimmed.is_empty() {
            continue;
        }

        if let Err(error) = execute_source(trimmed, &mut interp) {
            eprintln!("{}", error);
        }
    }
    Ok(())
}

/// Run project/unit tests.
///
/// Discovery rules (v0.6 roadmap friendly):
/// - If `target` is a file: run only that file.
/// - If `target` is a directory: scan it recursively.
/// - If `target` is not given:
///   - Prefer `tests/` if it exists, otherwise scan current directory.
///
/// Test file matching:
/// - `*_test.hb`
/// - OR any `.hb` file under a `tests/` directory
pub fn run_tests(target: Option<&str>) -> Result<()> {
    let root = match target {
        Some(t) => PathBuf::from(t),
        None => {
            let tests_dir = PathBuf::from("tests");
            if tests_dir.exists() {
                tests_dir
            } else {
                PathBuf::from(".")
            }
        }
    };

    let mut files = Vec::new();
    if root.is_file() {
        if root.extension().and_then(|s| s.to_str()) == Some("hb") {
            files.push(root.clone());
        } else {
            anyhow::bail!("Test hedefi bir .hb dosyası olmalı: {}", root.display());
        }
    } else {
        collect_test_files(&root, &mut files)?;
    }

    files.sort();
    files.dedup();

    if files.is_empty() {
        println!(
            "{} Çalıştırılacak test dosyası bulunamadı. (Arama: {})",
            "Bilgi:".bright_yellow(),
            root.display().to_string().dimmed()
        );
        return Ok(());
    }

    println!(
        "{} {} test dosyası bulundu.",
        "Hüma:".bright_cyan(),
        files.len().to_string().bright_white().bold(),
    );

    let mut passed_files = 0usize;
    let mut failed_files = 0usize;

    for file in &files {
        let file_str = file.display().to_string();
        print!(
            "{} {} ... ",
            "[TEST]".bright_cyan().bold(),
            file_str.bright_white()
        );
        io::stdout().flush().ok();

        match run_test_file(file) {
            Ok(TestOutcome::Passed { failed_tests }) => {
                if failed_tests > 0 {
                    failed_files += 1;
                    println!("{}", "FAIL".bright_red().bold());
                } else {
                    passed_files += 1;
                    println!("{}", "OK".bright_green().bold());
                }
            }
            Ok(TestOutcome::Unknown) => {
                passed_files += 1;
                println!("{}", "OK".bright_green().bold());
            }
            Err(e) => {
                failed_files += 1;
                println!("{}", "ERROR".bright_red().bold());
                println!(
                    "  {} {}",
                    "↳".bright_black(),
                    format!("{:#}", e).bright_red()
                );
            }
        }
    }

    println!();
    println!("{}", "-----------------------------".bright_black());
    println!("Toplam Dosya: {}", files.len());
    println!("Başarılı: {}", passed_files.to_string().bright_green());
    println!("Başarısız: {}", failed_files.to_string().bright_red());
    println!("{}", "-----------------------------".bright_black());

    if failed_files > 0 {
        anyhow::bail!(
            "Bazı test dosyaları başarısız oldu ({} adet).",
            failed_files
        );
    }

    Ok(())
}

#[derive(Debug)]
enum TestOutcome {
    /// We detected a `birim_test.hb` report, and parsed failed test count.
    Passed { failed_tests: usize },
    /// Test ran without panic, but we couldn't detect a report.
    Unknown,
}

fn run_test_file(path: &Path) -> Result<TestOutcome> {
    fn read_pipe_limited<R: Read>(pipe: R, name: &str) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        pipe.take((MAX_TEST_OUTPUT_BYTES as u64) + 1)
            .read_to_end(&mut bytes)
            .with_context(|| format!("Test {name} okunamadı"))?;
        if bytes.len() > MAX_TEST_OUTPUT_BYTES {
            anyhow::bail!(
                "Test {name} {} bayt sınırını aşıyor.",
                MAX_TEST_OUTPUT_BYTES
            );
        }
        Ok(bytes)
    }

    let executable =
        std::env::current_exe().with_context(|| "Hüma test yürütülebilirinin yolu bulunamadı")?;
    let mut child = Command::new(executable)
        .arg("run")
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("'{}' test süreci başlatılamadı", path.display()))?;
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            anyhow::bail!("Test standart çıktısı yakalanamadı")
        }
    };
    let stderr = match child.stderr.take() {
        Some(stderr) => stderr,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            anyhow::bail!("Test standart hatası yakalanamadı")
        }
    };
    let stdout_reader = match std::thread::Builder::new()
        .name("huma-test-stdout".to_string())
        .spawn(move || read_pipe_limited(stdout, "standart çıktısı"))
    {
        Ok(reader) => reader,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error).with_context(|| "Test çıktı okuyucusu başlatılamadı");
        }
    };
    let stderr_reader = match std::thread::Builder::new()
        .name("huma-test-stderr".to_string())
        .spawn(move || read_pipe_limited(stderr, "standart hatası"))
    {
        Ok(reader) => reader,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_reader.join();
            return Err(error).with_context(|| "Test hata okuyucusu başlatılamadı");
        }
    };
    let status = match child.wait_timeout(Duration::from_secs(3))? {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            anyhow::bail!("Zaman aşımı (3s). Test süreci sonlandırıldı.")
        }
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| anyhow::anyhow!("Test çıktı okuyucusu beklenmedik biçimde sonlandı"))??;
    let stderr = stderr_reader
        .join()
        .map_err(|_| anyhow::anyhow!("Test hata okuyucusu beklenmedik biçimde sonlandı"))??;
    let out =
        String::from_utf8(stdout).with_context(|| "Test standart çıktısı geçerli UTF-8 değil")?;
    let stderr =
        String::from_utf8(stderr).with_context(|| "Test standart hatası geçerli UTF-8 değil")?;
    if !status.success() {
        let detail = if stderr.trim().is_empty() {
            out.trim()
        } else {
            stderr.trim()
        };
        anyhow::bail!(
            "Test süreci başarısız oldu (çıkış: {}): {}",
            status
                .code()
                .map_or_else(|| "sinyal".to_string(), |code| code.to_string()),
            detail
        );
    }

    if let Some(failed) = parse_birim_test_failed_count(&out) {
        return Ok(TestOutcome::Passed {
            failed_tests: failed,
        });
    }

    // Fallback: If framework printed explicit failure marker, treat as failed.
    if out.contains("!!! HATA !!!") {
        return Ok(TestOutcome::Passed { failed_tests: 1 });
    }

    Ok(TestOutcome::Unknown)
}

fn parse_birim_test_failed_count(output: &str) -> Option<usize> {
    // Example lines:
    // "Başarısız: 0"
    // "Başarısız: 3"
    for line in output.lines() {
        let l = line.trim();
        if let Some(rest) = l.strip_prefix("Başarısız:") {
            let n_str = rest.trim();
            if let Ok(n) = n_str.parse::<usize>() {
                return Some(n);
            }
        }
    }
    None
}

fn collect_test_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    fn collect(root: &Path, out: &mut Vec<PathBuf>, depth: usize) -> Result<()> {
        if depth > 32 {
            anyhow::bail!(
                "Test dizin derinliği 32 sınırını aşıyor: {}",
                root.display()
            );
        }
        if out.len() > MAX_TEST_FILES {
            anyhow::bail!("Test dosyası sayısı {} sınırını aşıyor.", MAX_TEST_FILES);
        }
        if !root.exists() {
            return Ok(());
        }
        let metadata = std::fs::symlink_metadata(root)
            .with_context(|| format!("Test yolu incelenemedi: {}", root.display()))?;
        if metadata.file_type().is_symlink() {
            anyhow::bail!(
                "Test keşfinde sembolik bağlantı reddedildi: {}",
                root.display()
            );
        }
        if metadata.is_file() {
            if is_test_file(root) {
                out.push(root.to_path_buf());
            }
            return Ok(());
        }
        if !metadata.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(root)
            .with_context(|| format!("Dizin okunamadı: {}", root.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                anyhow::bail!(
                    "Test keşfinde sembolik bağlantı reddedildi: {}",
                    path.display()
                );
            }
            if file_type.is_dir() {
                collect(&path, out, depth + 1)?;
            } else if file_type.is_file() && is_test_file(&path) {
                out.push(path);
                if out.len() > MAX_TEST_FILES {
                    anyhow::bail!("Test dosyası sayısı {} sınırını aşıyor.", MAX_TEST_FILES);
                }
            }
        }
        Ok(())
    }

    collect(root, out, 0)
}

fn is_test_file(path: &Path) -> bool {
    if path.extension().and_then(|s| s.to_str()) != Some("hb") {
        return false;
    }

    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    if file_name.ends_with("_test.hb") {
        return true;
    }

    // Any `.hb` file under a `tests/` folder is considered a test.
    path.components().any(|c| c.as_os_str() == "tests")
}

/// Helper: lex → parse → interpret.
fn execute_source(source: &str, interp: &mut Yorumlayici) -> Result<()> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let (program, diagnostics) = parser.parse_program_with_diagnostics();
    if let Some(first) = diagnostics.into_iter().next() {
        return Err(first.into());
    }
    interp.yorumla_kontrollu(program)?;
    Ok(())
}
