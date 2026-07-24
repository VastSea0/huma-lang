//! Standalone Rust code-generation from Hüma source.
//!
//! Generates a self-contained `.rs` file that embeds the bytecode as literal
//! Rust data structures and includes a minimal stack-based VM to run it.
//!
//! Design notes
//! ============
//! * `MakeFunction` carries a `Vec<Komut>` (AST) that cannot be serialised as
//!   a Rust literal.  We solve this by **pre-compiling every function body at
//!   gen time** into its own `(constants, instructions)` pair, stored in a
//!   `FUNCTIONS` table inside the generated binary.
//! * The generated VM uses a proper **call-frame stack** so recursive and
//!   mutually-recursive functions work correctly.
//! * Cargo package names must satisfy `[a-zA-Z][a-zA-Z0-9_-]*`; we derive
//!   the name from the last path segment of `output_name`.
//! * We emit `[workspace]` so the generated crate is not mistakenly absorbed
//!   into the parent workspace when built from inside the repository.

use huma_core::bytecode::{Constant, OpCode, Program};
use huma_core::compiler::Derleyici;
use huma_core::error::{HumaError, HumaResult};
use huma_core::lexer::Lexer;
use huma_core::parser::Parser;
use std::fs;

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Derive a valid Cargo package name from an arbitrary path string.
/// Takes the last non-empty path component and replaces any remaining
/// non-alphanumeric/dash/underscore characters with `_`.
fn safe_pkg_name(output_name: &str) -> String {
    // Take the last non-empty segment (handles paths like "build_scratch/bench_fib_gen")
    let base = output_name
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(output_name);

    // Replace characters that are invalid in a Cargo package name
    let sanitised: String = base
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();

    // Cargo package names must start with a letter; prepend 'p' if needed
    if sanitised.starts_with(|c: char| c.is_numeric()) {
        format!("p{}", sanitised)
    } else {
        sanitised
    }
}

/// Format a single `OpCode` as a Rust literal (for the *top-level* program).
/// All String-typed enum fields are emitted with `.to_string()` so the
/// generated code compiles without `&str` / `String` mismatches.
fn render_opcode(op: &OpCode) -> String {
    match op {
        OpCode::PushConstant(n)  => format!("I::K({})", n),
        OpCode::LoadVar(n)       => format!("I::LV({}.to_string())", quote(n)),
        OpCode::StoreVar(n)      => format!("I::SV({}.to_string())", quote(n)),
        OpCode::DefineVar(n)     => format!("I::DV({}.to_string())", quote(n)),
        OpCode::Add              => "I::Add".into(),
        OpCode::Sub              => "I::Sub".into(),
        OpCode::Mul              => "I::Mul".into(),
        OpCode::Div              => "I::Div".into(),
        OpCode::Greater          => "I::Gt".into(),
        OpCode::Less             => "I::Lt".into(),
        OpCode::LessOrEqual      => "I::Le".into(),
        OpCode::Equal            => "I::Eq".into(),
        OpCode::NotEqual         => "I::Ne".into(),
        OpCode::Jump(n)          => format!("I::J({})", n),
        OpCode::JumpIfFalse(n)   => format!("I::JF({})", n),
        OpCode::Call(n)          => format!("I::Call({})", n),
        OpCode::Return           => "I::Ret".into(),
        OpCode::Print            => "I::Print".into(),
        OpCode::Pop              => "I::Pop".into(),
        OpCode::Bos              => "I::Bos".into(),
        OpCode::MakeList(n)      => format!("I::ML({})", n),
        OpCode::ListAccess       => "I::LA".into(),
        OpCode::MakeMap(n)       => format!("I::MM({})", n),
        OpCode::TryBlockStart(n) => format!("I::TS({})", n),
        OpCode::TryBlockEnd      => "I::TE".into(),
        OpCode::Await            => "I::Aw".into(),
        OpCode::CallFFI { lib_ad, fn_ad, arg_len } =>
            format!("I::FFI({}.to_string(),{}.to_string(),{})", quote(lib_ad), quote(fn_ad), arg_len),
        // MakeFunction is pre-compiled; at the top level it emits a MkFn referencing the table
        OpCode::MakeFunction { name, .. } =>
            format!("I::MkFn({}.to_string())", quote(name)),
    }
}

/// Escape a string as a Rust string literal (wrap in double-quotes, escape internals).
fn quote(s: &str) -> String {
    format!("{:?}", s)
}

/// Format a `Constant` as a Rust literal.
fn render_const(c: &Constant) -> String {
    match c {
        Constant::Sayi(n)  => format!("C::N({:?})", n),
        Constant::Metin(m) => format!("C::S({}.to_string())", quote(m)),
    }
}

/// Render a `Program` (instructions + constants) as two Rust `vec!` literals.
fn render_program(prog: &Program) -> (String, String) {
    let insts: Vec<String> = prog.instructions.iter().map(render_opcode).collect();
    let consts: Vec<String> = prog.constants.iter().map(render_const).collect();
    (
        format!("vec![{}]", insts.join(",")),
        format!("vec![{}]", consts.join(",")),
    )
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Read a `.hb` source file, compile it, and emit a standalone Cargo project
/// that can be compiled with `cargo run` into a native binary.
pub fn generate_standalone(input_path: &str, output_name: &str) -> HumaResult<String> {
    let source = fs::read_to_string(input_path)?;
    let lexer = Lexer::new(&source);
    let mut parser = Parser::new(lexer);
    let (program, diagnostics) = parser.parse_program_with_diagnostics();
    if let Some(first) = diagnostics.into_iter().next() {
        return Err(first);
    }

    let mut compiler = Derleyici::new();
    let bytecode = compiler
        .derle_kontrollu(program)
        .map_err(HumaError::CompileError)?;

    // ── Pre-compile function bodies ──────────────────────────────────────────
    // Walk the top-level instructions; every MakeFunction carries an AST body
    // that we compile separately into its own Program.
    let mut fn_entries: Vec<String> = Vec::new(); // Rust literal entries for FUNCTIONS table

    for op in &bytecode.instructions {
        if let OpCode::MakeFunction { name, params, body } = op {
            let mut fn_compiler = Derleyici::new();
            let fn_prog = fn_compiler.derle(body.clone());
            let (fn_insts, fn_consts) = render_program(&fn_prog);
            let params_lit: Vec<String> = params.iter().map(|p| format!("{:?}", p)).collect();
            fn_entries.push(format!(
                "({:?}, vec![{}], {}, {})",
                name,
                params_lit.join(","),
                fn_insts,
                fn_consts,
            ));
        }
    }

    let fn_table_lit = format!("vec![{}]", fn_entries.join(",\n    "));

    // ── Render top-level program ─────────────────────────────────────────────
    let (top_insts, top_consts) = render_program(&bytecode);

    // ── Native Package Discovery ─────────────────────────────────────────────
    let mut native_code = String::new();
    let mut extra_crates: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    if let Ok(mod_dir) = fs::read_dir("huma_modulleri") {
        for entry in mod_dir.flatten() {
            if entry.path().is_dir() {
                let pkg_name = entry.file_name().to_string_lossy().to_string();
                if source.contains(&format!("yükle \"{}\"", pkg_name))
                    || source.contains(&format!("yükle '{}'", pkg_name))
                {
                    let json_path  = entry.path().join("huma.json");
                    let json_path2 = entry.path().join("paket.json");
                    let target_json = if json_path.exists() {
                        Some(json_path)
                    } else if json_path2.exists() {
                        Some(json_path2)
                    } else {
                        None
                    };
                    if let Some(j) = target_json {
                        if let Ok(content) = fs::read_to_string(j) {
                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                                if let Some(code) = v.get("yerleşik_rust").and_then(|c| c.as_str()) {
                                    native_code.push_str("\n// --- Native Glue from ");
                                    native_code.push_str(&pkg_name);
                                    native_code.push_str(" ---\n");
                                    native_code.push_str(code);
                                    native_code.push('\n');
                                }
                                if let Some(deps) = v.get("crate_bagimliliklari").and_then(|d| d.as_object()) {
                                    for (k, ver) in deps {
                                        if let Some(v) = ver.as_str() {
                                            extra_crates.insert(k.clone(), v.to_string());
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

    // ── Build generated Rust source ──────────────────────────────────────────
    let rust_code = format!(
        r#"// Standalone Hüma Programı — Auto-generated by hüma compiler v{ver}
// DO NOT EDIT — regenerate with: huma gen {input}
#![allow(dead_code, unused_variables, unused_mut, non_camel_case_types, clippy::all)]

{native}

// ── Compact instruction enum ──────────────────────────────────────────────────
#[derive(Debug, Clone)]
enum I {{
    K(usize),      // PushConstant(index)
    LV(String),    // LoadVar
    SV(String),    // StoreVar
    DV(String),    // DefineVar
    Add, Sub, Mul, Div, Gt, Lt, Le, Eq, Ne,
    J(usize),      // Jump
    JF(usize),     // JumpIfFalse
    Call(usize),   // Call(arg_count)
    Ret,           // Return
    Print,
    Pop, Bos,
    ML(usize),     // MakeList(len)
    LA,            // ListAccess
    MM(usize),     // MakeMap(len)
    TS(usize),     // TryBlockStart(catch_addr)
    TE,            // TryBlockEnd
    Aw,            // Await (no-op in standalone)
    FFI(String, String, usize), // CallFFI(lib, fn, arg_len)
    MkFn(String),  // MakeFunction — registers fn from FUNCTIONS table
}}

// ── Constant pool ─────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
enum C {{
    N(f64),   // Number
    S(String), // String
}}

// ── Runtime value ─────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
enum V {{
    N(f64),
    S(String),
    List(Vec<V>),
    Map(std::collections::HashMap<String, V>),
    // Function: (params, instructions, constants)
    Fn(Vec<String>, Vec<I>, Vec<C>),
    Err(String),
    Nil,
}}

impl std::fmt::Display for V {{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {{
        match self {{
            V::N(n) => {{
                if *n == (*n as i64) as f64 {{ write!(f, "{{}}", *n as i64) }}
                else {{ write!(f, "{{}}", n) }}
            }},
            V::S(s)    => write!(f, "{{}}", s),
            V::List(l) => {{
                let parts: Vec<String> = l.iter().map(|x| x.to_string()).collect();
                write!(f, "[{{}}]", parts.join(", "))
            }},
            V::Map(_)  => write!(f, "<sözlük>"),
            V::Fn(..)  => write!(f, "<fonksiyon>"),
            V::Err(e)  => write!(f, "Hata: {{}}", e),
            V::Nil     => write!(f, "Boş"),
        }}
    }}
}}

// ── Function table: (name, params, instructions, constants) ───────────────────
fn make_fn_table() -> Vec<(&'static str, Vec<&'static str>, Vec<I>, Vec<C>)> {{
    // Each entry is pre-compiled at gen-time; params are &'static str slices.
    // We embed them inline and convert at runtime.
    {fn_table}
}}

// ── VM ────────────────────────────────────────────────────────────────────────
struct Frame {{
    insts:   Vec<I>,
    consts:  Vec<C>,
    locals:  std::collections::HashMap<String, V>,
    ip:      usize,
    ret_val: Option<V>,
}}

impl Frame {{
    fn new(insts: Vec<I>, consts: Vec<C>, locals: std::collections::HashMap<String, V>) -> Self {{
        Frame {{ insts, consts, locals, ip: 0, ret_val: None }}
    }}
}}

fn run_frame(
    frame: &mut Frame,
    globals: &mut std::collections::HashMap<String, V>,
    depth: usize,
) -> V {{
    if depth > 500 {{
        eprintln!("[Hüma] Azami özyineleme derinliği aşıldı");
        return V::Nil;
    }}
    let mut error_stack: Vec<usize> = Vec::new();
    let mut stack: Vec<V> = Vec::new();

    macro_rules! pop {{
        () => {{ stack.pop().unwrap_or(V::Nil) }};
    }}

    while frame.ip < frame.insts.len() {{
        // Clone the instruction to avoid borrow conflicts
        let op = frame.insts[frame.ip].clone();
        frame.ip += 1;

        match op {{
            I::K(i) => match &frame.consts[i] {{
                C::N(n) => stack.push(V::N(*n)),
                C::S(s) => stack.push(V::S(s.clone())),
            }},

            I::LV(ref a) => {{
                let v = frame.locals.get(a)
                    .or_else(|| globals.get(a))
                    .cloned()
                    .unwrap_or(V::Nil);
                stack.push(v);
            }},

            I::DV(ref a) => {{
                let v = pop!();
                frame.locals.insert(a.clone(), v.clone());
                globals.insert(a.clone(), v);
            }},

            I::SV(ref a) => {{
                let v = pop!();
                if frame.locals.contains_key(a) {{
                    frame.locals.insert(a.clone(), v.clone());
                }}
                if globals.contains_key(a) {{
                    globals.insert(a.clone(), v);
                }}
            }},

            I::Add => {{
                let r = pop!(); let l = pop!();
                match (l, r) {{
                    (V::N(a), V::N(b))   => stack.push(V::N(a + b)),
                    (V::S(a), V::S(b))   => stack.push(V::S(a + &b)),
                    (V::S(a), b)         => stack.push(V::S(format!("{{}}{{}}", a, b))),
                    (a, V::S(b))         => stack.push(V::S(format!("{{}}{{}}", a, b))),
                    _                    => stack.push(V::Nil),
                }}
            }},
            I::Sub => {{
                let r = pop!(); let l = pop!();
                if let (V::N(a), V::N(b)) = (l, r) {{ stack.push(V::N(a - b)); }}
            }},
            I::Mul => {{
                let r = pop!(); let l = pop!();
                if let (V::N(a), V::N(b)) = (l, r) {{ stack.push(V::N(a * b)); }}
            }},
            I::Div => {{
                let r = pop!(); let l = pop!();
                if let (V::N(a), V::N(b)) = (l, r) {{
                    if b == 0.0 {{ stack.push(V::Err("Sıfıra bölme".into())); }}
                    else {{ stack.push(V::N(a / b)); }}
                }}
            }},
            I::Gt => {{
                let r = pop!(); let l = pop!();
                if let (V::N(a), V::N(b)) = (l, r) {{ stack.push(V::N(if a > b {{ 1.0 }} else {{ 0.0 }})); }}
            }},
            I::Lt => {{
                let r = pop!(); let l = pop!();
                if let (V::N(a), V::N(b)) = (l, r) {{ stack.push(V::N(if a < b {{ 1.0 }} else {{ 0.0 }})); }}
            }},
            I::Le => {{
                let r = pop!(); let l = pop!();
                if let (V::N(a), V::N(b)) = (l, r) {{ stack.push(V::N(if a <= b {{ 1.0 }} else {{ 0.0 }})); }}
            }},
            I::Eq => {{
                let r = pop!(); let l = pop!();
                let eq = match (&l, &r) {{
                    (V::N(a), V::N(b))   => a == b,
                    (V::S(a), V::S(b))   => a == b,
                    (V::Nil, V::Nil)     => true,
                    _                    => false,
                }};
                stack.push(V::N(if eq {{ 1.0 }} else {{ 0.0 }}));
            }},
            I::Ne => {{
                let r = pop!(); let l = pop!();
                let eq = match (&l, &r) {{
                    (V::N(a), V::N(b))   => a == b,
                    (V::S(a), V::S(b))   => a == b,
                    (V::Nil, V::Nil)     => true,
                    _                    => false,
                }};
                stack.push(V::N(if !eq {{ 1.0 }} else {{ 0.0 }}));
            }},

            I::Print => println!("{{}}", pop!()),

            I::J(addr) => {{ frame.ip = addr; }},
            I::JF(addr) => {{
                let v = pop!();
                let truthy = match v {{
                    V::N(n)   => n != 0.0,
                    V::S(s)   => !s.is_empty(),
                    V::Nil    => false,
                    _         => true,
                }};
                if !truthy {{ frame.ip = addr; }}
            }},

            I::Ret => {{
                frame.ret_val = Some(pop!());
                break;
            }},

            I::Pop => {{ pop!(); }},

            I::Bos => stack.push(V::Nil),

            I::ML(len) => {{
                let mut list = Vec::with_capacity(len);
                for _ in 0..len {{ list.push(pop!()); }}
                list.reverse();
                stack.push(V::List(list));
            }},

            I::LA => {{
                let idx = pop!(); let container = pop!();
                match (container, idx) {{
                    (V::List(l), V::N(i)) => {{
                        let i = i as isize;
                        if i >= 0 && (i as usize) < l.len() {{
                            stack.push(l[i as usize].clone());
                        }} else {{
                            stack.push(V::Err("İndeks sınır dışı".into()));
                        }}
                    }},
                    (V::Map(m), V::S(k)) => stack.push(m.get(&k).cloned().unwrap_or(V::Nil)),
                    _ => stack.push(V::Err("Geçersiz erişim".into())),
                }}
            }},

            I::MM(len) => {{
                let mut map = std::collections::HashMap::new();
                for _ in 0..len {{
                    let v = pop!();
                    let k = match pop!() {{ V::S(s) => s, other => other.to_string() }};
                    map.insert(k, v);
                }}
                stack.push(V::Map(map));
            }},

            I::TS(addr) => error_stack.push(addr),
            I::TE       => {{ error_stack.pop(); }},
            I::Aw       => {{ /* no-op in standalone */ }},
            I::FFI(..)  => {{ /* FFI not supported in standalone */ }},

            I::MkFn(ref name) => {{
                // Register function from FUNCTIONS table into globals
                if let Some(v) = globals.get(name.as_str()).cloned() {{
                    // already registered (shouldn't happen but safe)
                    let _ = v;
                }}
                // The function is already in globals (registered in main before run)
            }},

            I::Call(arg_count) => {{
                let callable = pop!();
                let mut args: Vec<V> = (0..arg_count).map(|_| pop!()).collect();
                args.reverse();

                match callable {{
                    V::Fn(params, fn_insts, fn_consts) => {{
                        let mut locals: std::collections::HashMap<String, V> =
                            std::collections::HashMap::new();
                        for (p, a) in params.iter().zip(args.iter()) {{
                            locals.insert(p.clone(), a.clone());
                        }}
                        // Clone globals so the callee can read outer variables
                        let mut child_globals = globals.clone();
                        // Make the function itself available for recursion
                        // (it's already in globals by name via MkFn, so this is automatic)
                        let mut child_frame = Frame::new(fn_insts, fn_consts, locals);
                        let ret = run_frame(&mut child_frame, &mut child_globals, depth + 1);
                        // Propagate any new globals written by the callee back
                        // (selective: only update keys that already existed in parent globals)
                        for (k, v) in child_globals {{
                            if globals.contains_key(&k) {{
                                globals.insert(k, v);
                            }}
                        }}
                        stack.push(ret);
                    }},
                    other => {{
                        eprintln!("[Hüma] Çağrılamayan değer: {{}}", other);
                        stack.push(V::Nil);
                    }},
                }}
            }},
        }}
    }}

    frame.ret_val.take().unwrap_or_else(|| stack.pop().unwrap_or(V::Nil))
}}

fn main() {{
    // ── Build function table ─────────────────────────────────────────────────
    let raw_fn_table = make_fn_table();
    let mut globals: std::collections::HashMap<String, V> = std::collections::HashMap::new();
    for (name, params, insts, consts) in raw_fn_table {{
        let param_strings: Vec<String> = params.into_iter().map(|s| s.to_string()).collect();
        globals.insert(name.to_string(), V::Fn(param_strings, insts, consts));
    }}

    // ── Top-level program ────────────────────────────────────────────────────
    let top_insts: Vec<I> = {top_insts};
    let top_consts: Vec<C> = {top_consts};

    let mut frame = Frame::new(top_insts, top_consts, std::collections::HashMap::new());
    run_frame(&mut frame, &mut globals, 0);
}}
"#,
        ver        = env!("CARGO_PKG_VERSION"),
        input      = input_path,
        native     = native_code,
        fn_table   = fn_table_lit,
        top_insts  = top_insts,
        top_consts = top_consts,
    );

    // ── Project Structure Creation ────────────────────────────────────────────
    let pkg_name = safe_pkg_name(output_name);
    let project_path = format!("build_{}", output_name);
    fs::create_dir_all(format!("{}/src", project_path))?;

    // src/main.rs
    fs::write(format!("{}/src/main.rs", project_path), rust_code)?;

    // Cargo.toml — valid package name + [workspace] to opt out of parent workspace
    let mut cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

# Opt out of the parent huma-lang workspace
[workspace]

[dependencies]
"#,
        pkg_name
    );
    for (k, v) in &extra_crates {
        cargo_toml.push_str(&format!("{} = \"{}\"\n", k, v));
    }
    fs::write(format!("{}/Cargo.toml", project_path), cargo_toml)?;

    Ok(project_path)
}
