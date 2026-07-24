//! Implementations for each CLI subcommand.

use anyhow::{Context, Result};
use colored::Colorize;
use huma_core::interpreter::Yorumlayici;
use huma_core::lexer::Lexer;
use huma_core::parser::Parser;
use huma_core::vm::VM;
use std::cell::RefCell;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;

/// Run a `.hb` source file through the tree-walking interpreter.
pub fn run_file(path: &str) -> Result<()> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("'{}' dosyası okunamadı", path))?;

    let interp = Yorumlayici::new();
    let mut interp = interp;
    execute_source(&source, &mut interp);

    // GUI isteği var mı kontrol et
    if huma_core::gui::gui_istegi_var_mi() {
        huma_core::gui::gui_calistir(interp);
    }

    Ok(())
}

/// Run a `.hb` source file by compiling to Bytecode and executing in Bytecode VM.
pub fn run_vm_file(path: &str) -> Result<()> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("'{}' dosyası okunamadı", path))?;

    let lexer = Lexer::new(&source);
    let mut parser = Parser::new(lexer);
    let program_ast = parser.parse_program();

    let mut derleyici = huma_core::compiler::Derleyici::new();
    let bytecode_prog = derleyici.derle(program_ast);

    let mut vm = VM::new(bytecode_prog);
    vm.run();
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
    vm.run();
    Ok(())
}

/// Generate a standalone Rust source file from a `.hb` file.
pub fn generate_standalone(input: &str, output_name: &str) -> Result<()> {
    let rs_file = huma_compiler::codegen::generate_standalone(input, output_name)
        .with_context(|| format!("'{}' standalone kod üretimi sırasında hata oluştu", input))?;

    println!(
        "{} {} oluşturuldu. Derlemek için: {}",
        "[Başarı]".bright_green().bold(),
        rs_file.bright_white(),
        format!("cd {} && cargo build", rs_file).bright_cyan(),
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

    loop {
        print!("{} ", "hüma ❯".bright_cyan().bold());
        io::stdout().flush()?;

        input.clear();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let trimmed = input.trim();
        if trimmed == "çıkış" || trimmed == "exit" {
            println!("{}", "Görüşmek üzere! 👋".dimmed());
            break;
        }
        if trimmed.is_empty() {
            continue;
        }

        execute_source(trimmed, &mut interp);
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
/// Test file matching:
/// - `*_test.hb`
/// - OR any `.hb` file under a `tests/` directory
pub fn run_tests(target: Option<&str>) -> Result<()> {
    let root = match target {
        Some(t) => PathBuf::from(t),
        None => {
            let tests_dir = PathBuf::from("tests");
            if tests_dir.exists() { tests_dir } else { PathBuf::from(".") }
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
        print!("{} {} ... ", "[TEST]".bright_cyan().bold(), file_str.bright_white());
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
                println!("  {} {}", "↳".bright_black(), format!("{:#}", e).bright_red());
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
        anyhow::bail!("Bazı test dosyaları başarısız oldu ({} adet).", failed_files);
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
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("'{}' test dosyası okunamadı", path.display()))?;

    // Run each test in its own thread with a timeout to avoid hangs
    // (e.g., accidental infinite loops or blocking IO).
    let (tx, rx) = mpsc::channel::<Result<String>>();
    let src = source.clone();

    std::thread::spawn(move || {
        let output = Rc::new(RefCell::new(String::new()));
        let mut interp = Yorumlayici::new().with_output_buffer(output.clone());

        let run_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            execute_source(&src, &mut interp);
        }));

        let send_res = match run_res {
            Ok(()) => Ok(output.borrow().clone()),
            Err(p) => {
                let msg = if let Some(s) = p.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = p.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Bilinmeyen panic".to_string()
                };
                Err(anyhow::anyhow!("Panic: {}", msg))
            }
        };

        // If receiver is gone, we can ignore.
        let _ = tx.send(send_res);
    });

    let out = match rx.recv_timeout(Duration::from_secs(3)) {
        Ok(res) => res?,
        Err(mpsc::RecvTimeoutError::Timeout) => {
            anyhow::bail!("Zaman aşımı (3s). Test dosyası kilitlenmiş olabilir.")
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            anyhow::bail!("Test iş parçacığı beklenmedik şekilde sonlandı.")
        }
    };

    if let Some(failed) = parse_birim_test_failed_count(&out) {
        return Ok(TestOutcome::Passed { failed_tests: failed });
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
    if !root.exists() {
        return Ok(());
    }

    if root.is_file() {
        if is_test_file(root) {
            out.push(root.to_path_buf());
        }
        return Ok(());
    }

    for entry in std::fs::read_dir(root)
        .with_context(|| format!("Dizin okunamadı: {}", root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_test_files(&path, out)?;
        } else if is_test_file(&path) {
            out.push(path);
        }
    }

    Ok(())
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
fn execute_source(source: &str, interp: &mut Yorumlayici) {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    interp.yorumla(program);
}
