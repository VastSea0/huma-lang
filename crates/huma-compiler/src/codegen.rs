//! Standalone Rust code-generation from Hüma source.
//!
//! Generates a self-contained `.rs` file that embeds the bytecode as literal
//! Rust data structures and includes a minimal VM to run it.

use huma_core::bytecode::{Constant, OpCode};
use huma_core::compiler::Derleyici;
use huma_core::error::HumaResult;
use huma_core::lexer::Lexer;
use huma_core::parser::Parser;
use std::fs;

/// Read a `.hb` source file, compile it, and emit a standalone Cargo project
/// that can be compiled with `cargo run` into a native binary with all dependencies.
pub fn generate_standalone(input_path: &str, output_name: &str) -> HumaResult<String> {
    let source = fs::read_to_string(input_path)?;
    let lexer = Lexer::new(&source);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    let mut compiler = Derleyici::new();
    let bytecode = compiler.derle(program);

    // ── Native Package Discovery ───────────────────────────────────
    let mut native_code = String::new();
    let mut extra_crates = std::collections::HashMap::new();
    
    // Simple scan for 'yükle' statements to find used packages
    if let Ok(mod_dir) = fs::read_dir("huma_modulleri") {
        for entry in mod_dir.flatten() {
            if entry.path().is_dir() {
                let pkg_name = entry.file_name().to_string_lossy().to_string();
                // Check if this package is actually used in the source
                if source.contains(&format!("yükle \"{}\"", pkg_name)) || source.contains(&format!("yükle '{}'", pkg_name)) {
                    let json_path = entry.path().join("huma.json");
                    let json_path2 = entry.path().join("paket.json");
                    let target_json = if json_path.exists() { Some(json_path) } else if json_path2.exists() { Some(json_path2) } else { None };
                    
                    if let Some(j) = target_json {
                        if let Ok(json_content) = fs::read_to_string(j) {
                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json_content) {
                                // Extract yerleşik_rust
                                if let Some(code) = v.get("yerleşik_rust").and_then(|c| c.as_str()) {
                                    native_code.push_str("\n// --- Native Glue from ");
                                    native_code.push_str(&pkg_name);
                                    native_code.push_str(" ---\n");
                                    native_code.push_str(code);
                                    native_code.push_str("\n");
                                }
                                // Extract crate_bagimliliklari
                                if let Some(deps) = v.get("crate_bagimliliklari").and_then(|d| d.as_object()) {
                                    for (k, v) in deps {
                                        if let Some(ver) = v.as_str() {
                                            extra_crates.insert(k.clone(), ver.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // ── Render instructions as Rust literals ────────────────────────
    let inst_items: Vec<String> = bytecode
        .instructions
        .iter()
        .map(|op| match op {
            OpCode::PushConstant(n) => format!("OpCode::PushConstant({})", n),
            OpCode::LoadVar(n) => format!("OpCode::LoadVar(\"{}\".to_string())", n),
            OpCode::StoreVar(n) => format!("OpCode::StoreVar(\"{}\".to_string())", n),
            OpCode::DefineVar(n) => format!("OpCode::DefineVar(\"{}\".to_string())", n),
            OpCode::Add => "OpCode::Add".to_string(),
            OpCode::Sub => "OpCode::Sub".to_string(),
            OpCode::Mul => "OpCode::Mul".to_string(),
            OpCode::Div => "OpCode::Div".to_string(),
            OpCode::Greater => "OpCode::Greater".to_string(),
            OpCode::Less => "OpCode::Less".to_string(),
            OpCode::Equal => "OpCode::Equal".to_string(),
            OpCode::NotEqual => "OpCode::NotEqual".to_string(),
            OpCode::Jump(n) => format!("OpCode::Jump({})", n),
            OpCode::JumpIfFalse(n) => format!("OpCode::JumpIfFalse({})", n),
            OpCode::Call(n) => format!("OpCode::Call({})", n),
            OpCode::Return => "OpCode::Return".to_string(),
            OpCode::Print => "OpCode::Print".to_string(),
            OpCode::MakeList(n) => format!("OpCode::MakeList({})", n),
            OpCode::ListAccess => "OpCode::ListAccess".to_string(),
            OpCode::Pop => "OpCode::Pop".to_string(),
            OpCode::Bos => "OpCode::Bos".to_string(),
        })
        .collect();
    let inst_str = format!("vec![{}]", inst_items.join(", "));

    // ── Render constants as Rust literals ────────────────────────────
    let const_items: Vec<String> = bytecode
        .constants
        .iter()
        .map(|c| match c {
            Constant::Sayi(n) => format!("Constant::Sayi({:?})", n),
            Constant::Metin(m) => format!("Constant::Metin(\"{}\".to_string())", m),
        })
        .collect();
    let const_str = format!("vec![{}]", const_items.join(", "));

    // ── Template (main.rs) ──────────────────────────────────────────
    let rust_code = format!(
        r#"
// Standalone Hüma Programı — Auto-generated by hüma compiler v{}
#![allow(dead_code, unused_variables, unused_mut)]

#[derive(Debug, Clone)]
enum OpCode {{
    PushConstant(usize), LoadVar(String), StoreVar(String), DefineVar(String),
    Add, Sub, Mul, Div, Greater, Less, Equal, NotEqual,
    Jump(usize), JumpIfFalse(usize), Call(usize), Return, Print,
    MakeList(usize), ListAccess, Pop, Bos,
}}

#[derive(Debug, Clone)]
enum Constant {{ Sayi(f64), Metin(String) }}

#[derive(Debug, Clone)]
enum Deger {{ 
    Sayi(f64), 
    Metin(String), 
    Liste(Vec<Deger>), 
    Nesne(std::collections::HashMap<String, Deger>),
    Bos 
}}

{}

impl std::fmt::Display for Deger {{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {{
        match self {{
            Deger::Sayi(n) => write!(f, "{{}}", n),
            Deger::Metin(s) => write!(f, "{{}}", s),
            Deger::Liste(l) => {{
                let p: Vec<String> = l.iter().map(|d| d.to_string()).collect();
                write!(f, "[{{}}]", p.join(", "))
            }},
            Deger::Nesne(m) => write!(f, "<Nesne>"),
            Deger::Bos => write!(f, "Boş"),
        }}
    }}
}}

fn main() {{
    let inst = {};
    let cons = {};
    let mut stack: Vec<Deger> = Vec::new();
    let mut globals = std::collections::HashMap::new();
    let mut ip = 0;
    
    while ip < inst.len() {{
        let op = &inst[ip]; ip += 1;
        match op {{
            OpCode::PushConstant(i) => match &cons[*i] {{
                Constant::Sayi(n) => stack.push(Deger::Sayi(*n)),
                Constant::Metin(s) => stack.push(Deger::Metin(s.clone())),
            }},
            OpCode::LoadVar(a) => stack.push(globals.get(a).cloned().unwrap_or(Deger::Bos)),
            OpCode::DefineVar(a) => {{ let v = stack.pop().unwrap(); globals.insert(a.clone(), v); }},
            OpCode::StoreVar(a) => {{ let v = stack.pop().unwrap(); globals.insert(a.clone(), v); }},
            OpCode::Add => {{ let (r, l) = (stack.pop().unwrap(), stack.pop().unwrap()); if let (Deger::Sayi(a), Deger::Sayi(b)) = (l, r) {{ stack.push(Deger::Sayi(a+b)); }} }},
            OpCode::Sub => {{ let (r, l) = (stack.pop().unwrap(), stack.pop().unwrap()); if let (Deger::Sayi(a), Deger::Sayi(b)) = (l, r) {{ stack.push(Deger::Sayi(a-b)); }} }},
            OpCode::Mul => {{ let (r, l) = (stack.pop().unwrap(), stack.pop().unwrap()); if let (Deger::Sayi(a), Deger::Sayi(b)) = (l, r) {{ stack.push(Deger::Sayi(a*b)); }} }},
            OpCode::Div => {{ let (r, l) = (stack.pop().unwrap(), stack.pop().unwrap()); if let (Deger::Sayi(a), Deger::Sayi(b)) = (l, r) {{ stack.push(Deger::Sayi(a/b)); }} }},
            OpCode::Less => {{ let (r, l) = (stack.pop().unwrap(), stack.pop().unwrap()); if let (Deger::Sayi(a), Deger::Sayi(b)) = (l, r) {{ stack.push(Deger::Sayi(if a < b {{ 1.0 }} else {{ 0.0 }})); }} }},
            OpCode::Greater => {{ let (r, l) = (stack.pop().unwrap(), stack.pop().unwrap()); if let (Deger::Sayi(a), Deger::Sayi(b)) = (l, r) {{ stack.push(Deger::Sayi(if a > b {{ 1.0 }} else {{ 0.0 }})); }} }},
            OpCode::Print => println!("{{}}", stack.pop().unwrap()),
            OpCode::Jump(a) => ip = *a,
            OpCode::JumpIfFalse(a) => {{
                let v = stack.pop().unwrap();
                let t = match v {{ Deger::Sayi(n) => n != 0.0, Deger::Bos => false, _ => true }};
                if !t {{ ip = *a; }}
            }},
            OpCode::Return => break,
            OpCode::Pop => {{ stack.pop(); }},
            _ => {{}}
        }}
    }}
}}
"#,
        env!("CARGO_PKG_VERSION"),
        native_code,
        inst_str,
        const_str
    );

    // ── Project Structure Creation ──────────────────────────────────
    let project_path = format!("build_{}", output_name);
    fs::create_dir_all(format!("{}/src", project_path))?;
    
    // Write src/main.rs
    fs::write(format!("{}/src/main.rs", project_path), rust_code)?;
    
    // Generate Cargo.toml
    let mut cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        output_name
    );
    
    for (k, v) in extra_crates {
        cargo_toml.push_str(&format!("{} = \"{}\"\n", k, v));
    }
    
    fs::write(format!("{}/Cargo.toml", project_path), cargo_toml)?;

    Ok(project_path)
}
