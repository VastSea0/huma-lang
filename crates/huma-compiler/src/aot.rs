//! # Hüma AOT Derleyici — Cranelift Tabanlı Native Kod Üretimi
//!
//! Hüma kaynak kodunu Cranelift IR üzerinden gerçek native machine code'a
//! (ELF / Mach-O nesne dosyası) derler, ardından sistem bağlayıcısı (cc)
//! ile linkleyerek bağımsız bir binary üretir.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use cranelift_codegen::ir::{condcodes::FloatCC, types, AbiParam, InstBuilder};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

use huma_core::ast::{Ifade, Komut};
use huma_core::error::{HumaError, HumaResult};
use huma_core::lexer::Lexer;
use huma_core::parser::Parser;
use huma_core::token::Token;
use wait_timeout::ChildExt;

static AOT_BACKUP_SEQUENCE: AtomicU64 = AtomicU64::new(1);

fn bare_command(command: &Komut) -> &Komut {
    match command {
        Komut::Konumlu { komut, .. } => bare_command(komut),
        other => other,
    }
}

// ── Runtime C kaynak kodu ─────────────────────────────────────────────────────
const RUNTIME_C_SRC: &str = r#"
#include <stdio.h>
#include <math.h>

void huma_rt_print_f64(double v) {
    long long iv = (long long)v;
    if ((double)iv == v) {
        printf("%lld\n", iv);
    } else {
        printf("%.10g\n", v);
    }
}
"#;

fn command_status_with_timeout(mut command: Command, operation: &str) -> HumaResult<()> {
    let mut child = command
        .spawn()
        .map_err(|error| HumaError::CompileError(format!("{operation} başlatılamadı: {error}")))?;
    let status = match child.wait_timeout(Duration::from_secs(120)) {
        Ok(Some(status)) => status,
        Ok(None) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(HumaError::CompileError(format!(
                "{operation} 120 saniyede tamamlanmadı"
            )));
        }
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(HumaError::CompileError(format!(
                "{operation} beklenemedi: {error}"
            )));
        }
    };
    if status.success() {
        Ok(())
    } else {
        Err(HumaError::CompileError(format!(
            "{operation} başarısız oldu (çıkış: {})",
            status
                .code()
                .map_or_else(|| "sinyal".to_string(), |code| code.to_string())
        )))
    }
}

fn activate_binary(staged: &Path, output: &Path) -> HumaResult<()> {
    if std::fs::rename(staged, output).is_ok() {
        return Ok(());
    }
    if !output.exists() {
        return Err(HumaError::CompileError(format!(
            "AOT çıktısı etkinleştirilemedi: {}",
            output.display()
        )));
    }
    if output.is_dir() {
        return Err(HumaError::CompileError(format!(
            "AOT çıktı hedefi bir dizin: {}",
            output.display()
        )));
    }
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    let file_name = output
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| HumaError::CompileError("AOT çıktı adı geçerli UTF-8 değil".to_string()))?;
    let backup = (0..1_024)
        .find_map(|_| {
            let sequence = AOT_BACKUP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let candidate = parent.join(format!(
                ".{file_name}.aot-backup-{}-{sequence}",
                std::process::id()
            ));
            (!candidate.exists()).then_some(candidate)
        })
        .ok_or_else(|| {
            HumaError::CompileError(format!(
                "AOT çıktısı için benzersiz yedek yolu üretilemedi: {}",
                output.display()
            ))
        })?;
    std::fs::rename(output, &backup)?;
    if let Err(error) = std::fs::rename(staged, output) {
        let restore = std::fs::rename(&backup, output);
        return match restore {
            Ok(()) => Err(error.into()),
            Err(restore_error) => Err(HumaError::CompileError(format!(
                "AOT çıktısı etkinleştirilemedi ({error}) ve eski çıktı geri yüklenemedi \
                 ({restore_error}); yedek: {}",
                backup.display()
            ))),
        };
    }
    if let Err(error) = std::fs::remove_file(&backup) {
        eprintln!(
            "Uyarı: eski AOT yedeği temizlenemedi ({}): {}",
            backup.display(),
            error
        );
    }
    Ok(())
}

// ── Genel seçenekler ─────────────────────────────────────────────────────────

/// AOT derleme seçenekleri.
pub struct AotOptions<'a> {
    /// Üretilecek binary dosya yolu.
    pub output_bin: &'a Path,
    /// Optimizasyon seviyesi: 0 = yok, 1 = hız, 2 = hız+boyut.
    pub opt_level: u8,
}

// ── Ana giriş noktası ─────────────────────────────────────────────────────────

/// `source` Hüma kaynak kodunu native binary'ye derle.
pub fn compile_to_binary(source: &str, opts: &AotOptions<'_>) -> HumaResult<()> {
    if source.len() > crate::pipeline::MAX_SOURCE_BYTES {
        return Err(HumaError::CompileError(format!(
            "Kaynak {} bayt sınırını aşıyor",
            crate::pipeline::MAX_SOURCE_BYTES
        )));
    }
    // ── 1. Ayrıştırma ────────────────────────────────────────────────────────
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let (ast, diagnostics) = parser.parse_program_with_diagnostics();
    if let Some(first) = diagnostics.into_iter().next() {
        return Err(first);
    }
    validate_aot_subset(&ast)?;

    // ── 2. Cranelift kurulumu ────────────────────────────────────────────────
    let mut flag_builder = settings::builder();
    let opt_level = match opts.opt_level {
        0 => "none",
        1 => "speed",
        _ => "speed_and_size",
    };
    flag_builder.set("opt_level", opt_level).map_err(|error| {
        HumaError::CompileError(format!("AOT optimizasyon ayarı hatası: {error}"))
    })?;
    flag_builder
        .set("is_pic", "true")
        .map_err(|error| HumaError::CompileError(format!("AOT PIC ayarı hatası: {error}")))?;

    let flags = settings::Flags::new(flag_builder);
    let isa = cranelift_codegen::isa::lookup(target_lexicon::Triple::host())
        .map_err(|e| HumaError::CompileError(format!("ISA lookup hatası: {e}")))?
        .finish(flags)
        .map_err(|e| HumaError::CompileError(format!("ISA init hatası: {e}")))?;

    let obj_builder = ObjectBuilder::new(
        isa,
        "huma_program",
        cranelift_module::default_libcall_names(),
    )
    .map_err(|e| HumaError::CompileError(format!("ObjectBuilder hatası: {e}")))?;

    let mut module = ObjectModule::new(obj_builder);

    // ── 3. Runtime import: huma_rt_print_f64(f64) → void ───────────────────
    let mut print_sig = module.make_signature();
    print_sig.params.push(AbiParam::new(types::F64));
    let rt_print_id = module
        .declare_function("huma_rt_print_f64", Linkage::Import, &print_sig)
        .map_err(|e| HumaError::CompileError(format!("Runtime import hatası: {e}")))?;

    // ── 4. Kullanıcı fonksiyonlarını topla ve beyan et ──────────────────────
    let mut fn_registry: HashMap<String, (FuncId, Vec<String>)> = HashMap::new();
    let mut toplevel: Vec<Komut> = Vec::new();

    for komut in &ast {
        match bare_command(komut) {
            Komut::FonksiyonTanimla {
                ad, parametreler, ..
            } => {
                let mut sig = module.make_signature();
                for _ in parametreler {
                    sig.params.push(AbiParam::new(types::F64));
                }
                sig.returns.push(AbiParam::new(types::F64));
                let fid = module
                    .declare_function(ad, Linkage::Local, &sig)
                    .map_err(|e| {
                        HumaError::CompileError(format!("Fonksiyon beyan hatası '{ad}': {e}"))
                    })?;
                fn_registry.insert(ad.clone(), (fid, parametreler.clone()));
            }
            other => toplevel.push(other.clone()),
        }
    }

    let empty_sig = module.make_signature();
    let huma_main_id = module
        .declare_function("__huma_main", Linkage::Local, &empty_sig)
        .map_err(|e| HumaError::CompileError(format!("__huma_main beyan hatası: {e}")))?;

    // ── 5. Kullanıcı fonksiyonlarını tanımla ─────────────────────────────────
    let mut builder_ctx = FunctionBuilderContext::new();

    for komut in &ast {
        if let Komut::FonksiyonTanimla {
            ad,
            parametreler,
            govde,
        } = bare_command(komut)
        {
            let (fid, _) = fn_registry[ad];
            emit_user_function(
                ad,
                parametreler,
                govde,
                fid,
                &fn_registry,
                rt_print_id,
                huma_main_id,
                &mut module,
                &mut builder_ctx,
            )?;
        }
    }

    // ── 6. __huma_main tanımla ───────────────────────────────────────────────
    {
        let mut ctx = module.make_context();
        ctx.func.signature = module.make_signature();

        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);
        let entry = builder.create_block();
        builder.func.layout.append_block(entry);
        builder.switch_to_block(entry);
        builder.seal_block(entry);

        {
            let mut em = Emitter {
                builder: &mut builder,
                fn_registry: &fn_registry,
                rt_print_id,
                vars: HashMap::new(),
                var_counter: 0,
            };
            em.emit_stmts(&toplevel, &mut module);
            if !em.is_current_block_filled() {
                em.builder.ins().return_(&[]);
            }
        }
        builder.finalize();

        module
            .define_function(huma_main_id, &mut ctx)
            .map_err(|e| HumaError::CompileError(format!("__huma_main tanım hatası: {e}")))?;
        module.clear_context(&mut ctx);
    }

    // ── 7. C main → __huma_main çağır ───────────────────────────────────────
    {
        let mut c_main_sig = module.make_signature();
        c_main_sig.returns.push(AbiParam::new(types::I32));
        let c_main_id = module
            .declare_function("main", Linkage::Export, &c_main_sig)
            .map_err(|e| HumaError::CompileError(format!("main beyan hatası: {e}")))?;

        let mut ctx = module.make_context();
        ctx.func.signature = c_main_sig;

        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);
        let entry = builder.create_block();
        builder.func.layout.append_block(entry);
        builder.switch_to_block(entry);
        builder.seal_block(entry);

        let hm_ref = module.declare_func_in_func(huma_main_id, builder.func);
        builder.ins().call(hm_ref, &[]);
        let zero = builder.ins().iconst(types::I32, 0);
        builder.ins().return_(&[zero]);
        builder.finalize();

        module
            .define_function(c_main_id, &mut ctx)
            .map_err(|e| HumaError::CompileError(format!("main tanım hatası: {e}")))?;
        module.clear_context(&mut ctx);
    }

    // ── 8. Nesne dosyasına yaz ───────────────────────────────────────────────
    let obj_bytes = module
        .finish()
        .emit()
        .map_err(|e| HumaError::CompileError(format!("Nesne dosyası üretim hatası: {e}")))?;

    let parent = opts.output_bin.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent)?;
    let temporary = tempfile::Builder::new()
        .prefix(".huma-aot-")
        .tempdir_in(parent)
        .map_err(HumaError::IoError)?;
    let obj_path = temporary.path().join("program.o");
    let rt_c_path = temporary.path().join("runtime.c");
    let rt_o_path = temporary.path().join("runtime.o");
    let staged_output = temporary.path().join("program");

    std::fs::write(&obj_path, &obj_bytes)?;
    std::fs::write(&rt_c_path, RUNTIME_C_SRC)?;

    // ── 9. Runtime derleme ───────────────────────────────────────────────────
    let mut runtime_compile = Command::new("cc");
    runtime_compile
        .args(["-O2", "-c", "-o"])
        .arg(&rt_o_path)
        .arg(&rt_c_path);
    command_status_with_timeout(runtime_compile, "Runtime C derlemesi")?;

    // ── 10. Link ────────────────────────────────────────────────────────────
    let mut linker = Command::new("cc");
    linker
        .arg(&obj_path)
        .arg(&rt_o_path)
        .arg("-o")
        .arg(&staged_output)
        .arg("-lm");
    command_status_with_timeout(linker, "AOT bağlayıcısı")?;
    activate_binary(&staged_output, opts.output_bin)
}

/// AOT backend currently targets a deliberately small, numeric subset.
///
/// Unsupported syntax must be rejected before code generation. Silently
/// replacing an expression with `0.0` would produce a valid-looking but
/// incorrect executable, which is never acceptable for a stable compiler.
fn validate_aot_subset(ast: &[Komut]) -> HumaResult<()> {
    let function_names: HashSet<String> = ast
        .iter()
        .filter_map(|command| match bare_command(command) {
            Komut::FonksiyonTanimla { ad, .. } => Some(ad.clone()),
            _ => None,
        })
        .collect();

    for command in ast {
        if let Komut::FonksiyonTanimla {
            ad,
            parametreler,
            govde,
        } = bare_command(command)
        {
            let mut scope: HashSet<String> = parametreler.iter().cloned().collect();
            validate_aot_commands(govde, &mut scope, &function_names).map_err(|message| {
                HumaError::CompileError(format!("AOT fonksiyonu '{}': {}", ad, message))
            })?;
        }
    }

    let mut top_scope = HashSet::new();
    validate_aot_commands(ast, &mut top_scope, &function_names).map_err(HumaError::CompileError)
}

fn validate_aot_commands(
    commands: &[Komut],
    scope: &mut HashSet<String>,
    function_names: &HashSet<String>,
) -> Result<(), String> {
    for command in commands {
        match bare_command(command) {
            Komut::DegiskenTanimla { ad, deger } => {
                validate_aot_expression(deger, scope, function_names)?;
                scope.insert(ad.clone());
            }
            Komut::Atama { ad, deger } => {
                if !scope.contains(ad) {
                    return Err(format!("tanımsız değişkene atama: {}", ad));
                }
                validate_aot_expression(deger, scope, function_names)?;
            }
            Komut::YazdirKomutu(expression) | Komut::DondurKomutu(expression) => {
                validate_aot_expression(expression, scope, function_names)?;
            }
            Komut::EgerKomutu {
                kosul,
                govde,
                degilse_govde,
            } => {
                validate_aot_expression(kosul, scope, function_names)?;
                validate_aot_commands(govde, scope, function_names)?;
                if let Some(otherwise) = degilse_govde {
                    validate_aot_commands(otherwise, scope, function_names)?;
                }
            }
            Komut::DonguKomutu { kosul, govde } => {
                validate_aot_expression(kosul, scope, function_names)?;
                validate_aot_commands(govde, scope, function_names)?;
            }
            Komut::AralikDongusu {
                degisken,
                baslangic,
                bitis,
                govde,
            } => {
                validate_aot_expression(baslangic, scope, function_names)?;
                validate_aot_expression(bitis, scope, function_names)?;
                scope.insert(degisken.clone());
                validate_aot_commands(govde, scope, function_names)?;
            }
            Komut::IfadeKomutu(Ifade::IkiliIslem {
                sol,
                operator: Token::Esittir,
                sag,
            }) => {
                let Ifade::Degisken(name) = sol.as_ref() else {
                    return Err("AOT atamasının sol tarafı değişken olmalıdır".to_string());
                };
                if !scope.contains(name) {
                    return Err(format!("tanımsız değişkene atama: {}", name));
                }
                validate_aot_expression(sag, scope, function_names)?;
            }
            Komut::IfadeKomutu(expression) => {
                validate_aot_expression(expression, scope, function_names)?;
            }
            Komut::FonksiyonTanimla { .. } => {}
            unsupported => {
                return Err(format!(
                    "AOT sayısal alt kümesinde desteklenmeyen komut: {:?}",
                    unsupported
                ));
            }
        }
    }
    Ok(())
}

fn validate_aot_expression(
    expression: &Ifade,
    scope: &HashSet<String>,
    function_names: &HashSet<String>,
) -> Result<(), String> {
    match expression {
        Ifade::Sayi(_) | Ifade::Dogru | Ifade::Yanlis => Ok(()),
        Ifade::Degisken(name) if scope.contains(name) => Ok(()),
        Ifade::Degisken(name) => Err(format!("tanımsız değişken: {}", name)),
        Ifade::IkiliIslem { sol, operator, sag } => {
            if !matches!(
                operator,
                Token::Arti
                    | Token::Eksi
                    | Token::Carpi
                    | Token::Bolnu
                    | Token::Kucuktur
                    | Token::Buyuktur
                    | Token::KucukEsit
                    | Token::BuyukEsit
                    | Token::EsitEsittir
                    | Token::Esittir
                    | Token::EsitDegil
            ) {
                return Err(format!(
                    "AOT tarafından desteklenmeyen operatör: {}",
                    operator
                ));
            }
            validate_aot_expression(sol, scope, function_names)?;
            validate_aot_expression(sag, scope, function_names)
        }
        Ifade::Cagri {
            fonksiyon,
            argumanlar,
            ..
        } => {
            let Ifade::Degisken(name) = fonksiyon.as_ref() else {
                return Err(
                    "AOT yalnızca adlandırılmış fonksiyon çağrılarını destekler".to_string()
                );
            };
            if !function_names.contains(name) {
                return Err(format!("AOT için bilinmeyen fonksiyon: {}", name));
            }
            for argument in argumanlar {
                validate_aot_expression(argument, scope, function_names)?;
            }
            Ok(())
        }
        unsupported => Err(format!(
            "AOT sayısal alt kümesinde desteklenmeyen ifade: {:?}",
            unsupported
        )),
    }
}

// ── Kullanıcı fonksiyonu üretici ─────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn emit_user_function(
    ad: &str,
    parametreler: &[String],
    govde: &[Komut],
    fid: FuncId,
    fn_registry: &HashMap<String, (FuncId, Vec<String>)>,
    rt_print_id: FuncId,
    _huma_main_id: FuncId,
    module: &mut ObjectModule,
    builder_ctx: &mut FunctionBuilderContext,
) -> HumaResult<()> {
    let mut sig = module.make_signature();
    for _ in parametreler {
        sig.params.push(AbiParam::new(types::F64));
    }
    sig.returns.push(AbiParam::new(types::F64));

    let mut ctx = module.make_context();
    ctx.func.signature = sig;

    {
        let mut builder = FunctionBuilder::new(&mut ctx.func, builder_ctx);
        let entry = builder.create_block();
        builder.func.layout.append_block(entry);
        builder.switch_to_block(entry);
        builder.append_block_params_for_function_params(entry);
        builder.seal_block(entry);

        let params: Vec<cranelift_codegen::ir::Value> = builder.block_params(entry).to_vec();

        let mut vars: HashMap<String, Variable> = HashMap::new();
        let mut var_counter: u32 = 0;
        for (i, p) in parametreler.iter().enumerate() {
            let v = Variable::from_u32(var_counter);
            var_counter += 1;
            builder.declare_var(v, types::F64);
            builder.def_var(v, params[i]);
            vars.insert(p.clone(), v);
        }

        {
            let mut em = Emitter {
                builder: &mut builder,
                fn_registry,
                rt_print_id,
                vars,
                var_counter,
            };
            em.emit_stmts(govde, module);

            if !em.is_current_block_filled() {
                let zero = em.builder.ins().f64const(0.0);
                em.builder.ins().return_(&[zero]);
            }
        }
        builder.finalize();
    }

    module
        .define_function(fid, &mut ctx)
        .map_err(|e| HumaError::CompileError(format!("Fonksiyon tanım hatası '{ad}': {e}")))?;

    module.clear_context(&mut ctx);
    Ok(())
}

// ── IR yayıcı ────────────────────────────────────────────────────────────────

struct Emitter<'a, 'b> {
    builder: &'a mut FunctionBuilder<'b>,
    fn_registry: &'a HashMap<String, (FuncId, Vec<String>)>,
    rt_print_id: FuncId,
    vars: HashMap<String, Variable>,
    var_counter: u32,
}

impl<'a, 'b> Emitter<'a, 'b> {
    fn is_current_block_filled(&self) -> bool {
        if let Some(block) = self.builder.current_block() {
            if let Some(inst) = self.builder.func.layout.last_inst(block) {
                return self.builder.func.dfg.insts[inst].opcode().is_terminator();
            }
        }
        false
    }

    fn get_or_create_var(&mut self, name: &str) -> Variable {
        if let Some(&v) = self.vars.get(name) {
            return v;
        }
        let v = Variable::from_u32(self.var_counter);
        self.var_counter += 1;
        self.builder.declare_var(v, types::F64);
        let zero = self.builder.ins().f64const(0.0);
        self.builder.def_var(v, zero);
        self.vars.insert(name.to_string(), v);
        v
    }

    fn emit_stmts(&mut self, stmts: &[Komut], module: &mut ObjectModule) {
        for stmt in stmts {
            if self.is_current_block_filled() {
                break;
            }
            self.emit_stmt(stmt, module);
        }
    }

    fn emit_stmt(&mut self, komut: &Komut, module: &mut ObjectModule) {
        let komut = bare_command(komut);
        match komut {
            Komut::DegiskenTanimla { ad, deger } | Komut::Atama { ad, deger } => {
                if let Some(val) = self.emit_expr(deger, module) {
                    let var = self.get_or_create_var(ad);
                    self.builder.def_var(var, val);
                }
            }

            Komut::YazdirKomutu(ifade) => {
                let val = self
                    .emit_expr(ifade, module)
                    .unwrap_or_else(|| self.builder.ins().f64const(0.0));
                let print_ref = module.declare_func_in_func(self.rt_print_id, self.builder.func);
                self.builder.ins().call(print_ref, &[val]);
            }

            Komut::EgerKomutu {
                kosul,
                govde,
                degilse_govde,
            } => {
                let cond_val = match self.emit_expr(kosul, module) {
                    Some(v) => v,
                    None => return,
                };

                let then_bb = self.builder.create_block();
                let else_bb = self.builder.create_block();
                let merge_bb = self.builder.create_block();

                let zero = self.builder.ins().f64const(0.0);
                let cond = self.builder.ins().fcmp(FloatCC::NotEqual, cond_val, zero);
                self.builder.ins().brif(cond, then_bb, &[], else_bb, &[]);

                self.builder.func.layout.append_block(then_bb);
                self.builder.switch_to_block(then_bb);
                self.builder.seal_block(then_bb);
                self.emit_stmts(govde, module);
                if !self.is_current_block_filled() {
                    self.builder.ins().jump(merge_bb, &[]);
                }

                self.builder.func.layout.append_block(else_bb);
                self.builder.switch_to_block(else_bb);
                self.builder.seal_block(else_bb);
                if let Some(else_stmts) = degilse_govde {
                    self.emit_stmts(else_stmts, module);
                }
                if !self.is_current_block_filled() {
                    self.builder.ins().jump(merge_bb, &[]);
                }

                self.builder.func.layout.append_block(merge_bb);
                self.builder.switch_to_block(merge_bb);
                self.builder.seal_block(merge_bb);
            }

            Komut::DonguKomutu { kosul, govde } => {
                let header_bb = self.builder.create_block();
                let body_bb = self.builder.create_block();
                let exit_bb = self.builder.create_block();

                self.builder.ins().jump(header_bb, &[]);

                self.builder.func.layout.append_block(header_bb);
                self.builder.switch_to_block(header_bb);
                let cond_val = self
                    .emit_expr(kosul, module)
                    .unwrap_or_else(|| self.builder.ins().f64const(0.0));
                let zero = self.builder.ins().f64const(0.0);
                let cond = self.builder.ins().fcmp(FloatCC::NotEqual, cond_val, zero);
                self.builder.ins().brif(cond, body_bb, &[], exit_bb, &[]);

                self.builder.func.layout.append_block(body_bb);
                self.builder.switch_to_block(body_bb);
                self.builder.seal_block(body_bb);
                self.emit_stmts(govde, module);
                if !self.is_current_block_filled() {
                    self.builder.ins().jump(header_bb, &[]);
                }

                self.builder.seal_block(header_bb);

                self.builder.func.layout.append_block(exit_bb);
                self.builder.switch_to_block(exit_bb);
                self.builder.seal_block(exit_bb);
            }

            Komut::AralikDongusu {
                degisken,
                baslangic,
                bitis,
                govde,
            } => {
                let start_val = match self.emit_expr(baslangic, module) {
                    Some(v) => v,
                    None => return,
                };
                let loop_var = self.get_or_create_var(degisken);
                self.builder.def_var(loop_var, start_val);

                let header_bb = self.builder.create_block();
                let body_bb = self.builder.create_block();
                let exit_bb = self.builder.create_block();

                self.builder.ins().jump(header_bb, &[]);
                self.builder.func.layout.append_block(header_bb);
                self.builder.switch_to_block(header_bb);

                let cur = self.builder.use_var(loop_var);
                let end_val = match self.emit_expr(bitis, module) {
                    Some(v) => v,
                    None => return,
                };
                let cond = self
                    .builder
                    .ins()
                    .fcmp(FloatCC::LessThanOrEqual, cur, end_val);
                self.builder.ins().brif(cond, body_bb, &[], exit_bb, &[]);

                self.builder.func.layout.append_block(body_bb);
                self.builder.switch_to_block(body_bb);
                self.builder.seal_block(body_bb);
                self.emit_stmts(govde, module);
                if !self.is_current_block_filled() {
                    let cur2 = self.builder.use_var(loop_var);
                    let one = self.builder.ins().f64const(1.0);
                    let next = self.builder.ins().fadd(cur2, one);
                    self.builder.def_var(loop_var, next);
                    self.builder.ins().jump(header_bb, &[]);
                }

                self.builder.seal_block(header_bb);
                self.builder.func.layout.append_block(exit_bb);
                self.builder.switch_to_block(exit_bb);
                self.builder.seal_block(exit_bb);
            }

            Komut::FonksiyonTanimla { .. } => {}

            Komut::DondurKomutu(ifade) => {
                let val = self
                    .emit_expr(ifade, module)
                    .unwrap_or_else(|| self.builder.ins().f64const(0.0));
                self.builder.ins().return_(&[val]);
            }

            Komut::IfadeKomutu(ifade) => {
                if let Ifade::IkiliIslem {
                    sol,
                    operator: Token::Esittir,
                    sag,
                } = ifade
                {
                    if let Ifade::Degisken(ad) = sol.as_ref() {
                        if let Some(val) = self.emit_expr(sag, module) {
                            let var = self.get_or_create_var(ad);
                            self.builder.def_var(var, val);
                            return;
                        }
                    }
                }
                self.emit_expr(ifade, module);
            }

            _ => {}
        }
    }

    fn emit_expr(
        &mut self,
        ifade: &Ifade,
        module: &mut ObjectModule,
    ) -> Option<cranelift_codegen::ir::Value> {
        match ifade {
            Ifade::Sayi(n) => Some(self.builder.ins().f64const(*n)),
            Ifade::Dogru => Some(self.builder.ins().f64const(1.0)),
            Ifade::Yanlis => Some(self.builder.ins().f64const(0.0)),
            Ifade::Bos => Some(self.builder.ins().f64const(0.0)),

            Ifade::Degisken(ad) => {
                let var = self.get_or_create_var(ad);
                Some(self.builder.use_var(var))
            }

            Ifade::IkiliIslem { sol, operator, sag } => {
                let l = self.emit_expr(sol, module)?;
                let r = self.emit_expr(sag, module)?;
                self.emit_binop(operator, l, r)
            }

            Ifade::Cagri {
                fonksiyon,
                argumanlar,
                ..
            } => {
                let mut arg_vals = Vec::with_capacity(argumanlar.len());
                for arg in argumanlar {
                    arg_vals.push(self.emit_expr(arg, module)?);
                }

                if let Ifade::Degisken(fn_name) = fonksiyon.as_ref() {
                    if let Some(&(fid, _)) = self.fn_registry.get(fn_name) {
                        let fref = module.declare_func_in_func(fid, self.builder.func);
                        let call = self.builder.ins().call(fref, &arg_vals);
                        let results = self.builder.inst_results(call);
                        return if results.is_empty() {
                            None
                        } else {
                            Some(results[0])
                        };
                    }
                }
                None
            }

            _ => None,
        }
    }

    fn emit_binop(
        &mut self,
        op: &Token,
        l: cranelift_codegen::ir::Value,
        r: cranelift_codegen::ir::Value,
    ) -> Option<cranelift_codegen::ir::Value> {
        match op {
            Token::Arti => Some(self.builder.ins().fadd(l, r)),
            Token::Eksi => Some(self.builder.ins().fsub(l, r)),
            Token::Carpi => Some(self.builder.ins().fmul(l, r)),
            Token::Bolnu => Some(self.builder.ins().fdiv(l, r)),

            Token::Kucuktur => {
                let b = self.builder.ins().fcmp(FloatCC::LessThan, l, r);
                Some(self.bool_to_f64(b))
            }
            Token::Buyuktur => {
                let b = self.builder.ins().fcmp(FloatCC::GreaterThan, l, r);
                Some(self.bool_to_f64(b))
            }
            Token::KucukEsit => {
                let b = self.builder.ins().fcmp(FloatCC::LessThanOrEqual, l, r);
                Some(self.bool_to_f64(b))
            }
            Token::BuyukEsit => {
                let b = self.builder.ins().fcmp(FloatCC::GreaterThanOrEqual, l, r);
                Some(self.bool_to_f64(b))
            }
            Token::EsitEsittir | Token::Esittir => {
                let b = self.builder.ins().fcmp(FloatCC::Equal, l, r);
                Some(self.bool_to_f64(b))
            }
            Token::EsitDegil => {
                let b = self.builder.ins().fcmp(FloatCC::NotEqual, l, r);
                Some(self.bool_to_f64(b))
            }

            _ => None,
        }
    }

    fn bool_to_f64(&mut self, b: cranelift_codegen::ir::Value) -> cranelift_codegen::ir::Value {
        let i = self.builder.ins().uextend(types::I64, b);
        self.builder.ins().fcvt_from_uint(types::F64, i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn aot_sayisal_alt_kume_gercek_ikili_uretir() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Sistem saati Unix epoch sonrasında olmalı")
            .as_nanos();
        let output =
            std::env::temp_dir().join(format!("huma_aot_test_{}_{}", std::process::id(), nonce));
        let source = r#"
            kare fonksiyon olsun x alsın {
                x * x'i döndür
            }
            sonuc = kare(5) olsun
            sonuc'u yazdır
        "#;

        compile_to_binary(
            source,
            &AotOptions {
                output_bin: &output,
                opt_level: 0,
            },
        )
        .expect("AOT sayısal alt kümesi ikili üretmeli");

        let run = Command::new(&output)
            .output()
            .expect("Üretilen AOT ikilisi çalışmalı");
        let _ = std::fs::remove_file(&output);
        assert!(run.status.success());
        assert_eq!(String::from_utf8_lossy(&run.stdout).trim(), "25");
    }

    #[test]
    fn aot_desteklenmeyen_ifadeyi_acikca_reddeder() {
        let output = std::env::temp_dir().join("huma_aot_reddet_test");
        let error = compile_to_binary(
            r#""metin"'i yazdır"#,
            &AotOptions {
                output_bin: &output,
                opt_level: 0,
            },
        )
        .expect_err("AOT metin ifadelerini kabul etmemeli");
        assert!(error.to_string().contains("desteklenmeyen ifade"));
        assert!(!output.exists());
    }
}
