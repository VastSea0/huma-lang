//! # Hüma AOT Derleyici — Cranelift Tabanlı Native Kod Üretimi
//!
//! Hüma kaynak kodunu Cranelift IR üzerinden gerçek native machine code'a
//! (ELF / Mach-O nesne dosyası) derler, ardından sistem bağlayıcısı (cc)
//! ile linkleyerek bağımsız bir binary üretir.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use cranelift_codegen::ir::{
    condcodes::FloatCC, types, AbiParam, InstBuilder,
};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

use huma_core::ast::{Ifade, Komut};
use huma_core::error::{HumaError, HumaResult};
use huma_core::lexer::Lexer;
use huma_core::parser::Parser;
use huma_core::token::Token;

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
    // ── 1. Ayrıştırma ────────────────────────────────────────────────────────
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let (ast, diagnostics) = parser.parse_program_with_diagnostics();
    if let Some(first) = diagnostics.into_iter().next() {
        return Err(first);
    }

    // ── 2. Cranelift kurulumu ────────────────────────────────────────────────
    let mut flag_builder = settings::builder();
    match opts.opt_level {
        0 => flag_builder.set("opt_level", "none").unwrap(),
        1 => flag_builder.set("opt_level", "speed").unwrap(),
        _ => flag_builder.set("opt_level", "speed_and_size").unwrap(),
    }
    flag_builder.set("is_pic", "true").unwrap();

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
        match komut {
            Komut::FonksiyonTanimla { ad, parametreler, .. } => {
                let mut sig = module.make_signature();
                for _ in parametreler {
                    sig.params.push(AbiParam::new(types::F64));
                }
                sig.returns.push(AbiParam::new(types::F64));
                let fid = module
                    .declare_function(ad, Linkage::Local, &sig)
                    .map_err(|e| HumaError::CompileError(format!("Fonksiyon beyan hatası '{ad}': {e}")))?;
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
        if let Komut::FonksiyonTanimla { ad, parametreler, govde } = komut {
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

        let hm_ref = module.declare_func_in_func(huma_main_id, &mut builder.func);
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

    let stem = opts
        .output_bin
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("huma_prog");
    let parent = opts.output_bin.parent().unwrap_or(Path::new("."));

    let obj_path = parent.join(format!("{stem}.o"));
    let rt_c_path = parent.join("huma_rt_tmp.c");
    let rt_o_path = parent.join("huma_rt_tmp.o");

    std::fs::write(&obj_path, &obj_bytes)?;

    // ── 9. Runtime derleme ───────────────────────────────────────────────────
    std::fs::write(&rt_c_path, RUNTIME_C_SRC)?;

    let status = Command::new("cc")
        .args(["-O2", "-c", "-o"])
        .arg(&rt_o_path)
        .arg(&rt_c_path)
        .status()
        .map_err(|e| HumaError::CompileError(format!("cc (runtime) çalıştırılamadı: {e}")))?;
    if !status.success() {
        return Err(HumaError::CompileError(
            "Runtime C derlemesi başarısız".into(),
        ));
    }

    // ── 10. Link ────────────────────────────────────────────────────────────
    let status = Command::new("cc")
        .arg(&obj_path)
        .arg(&rt_o_path)
        .arg("-o")
        .arg(opts.output_bin)
        .arg("-lm")
        .status()
        .map_err(|e| HumaError::CompileError(format!("Bağlayıcı çalıştırılamadı: {e}")))?;
    if !status.success() {
        return Err(HumaError::CompileError("Bağlayıcı (linker) hatası".into()));
    }

    let _ = std::fs::remove_file(&obj_path);
    let _ = std::fs::remove_file(&rt_c_path);
    let _ = std::fs::remove_file(&rt_o_path);

    Ok(())
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

        let params: Vec<cranelift_codegen::ir::Value> =
            builder.block_params(entry).to_vec();

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
                let print_ref =
                    module.declare_func_in_func(self.rt_print_id, &mut self.builder.func);
                self.builder.ins().call(print_ref, &[val]);
            }

            Komut::EgerKomutu { kosul, govde, degilse_govde } => {
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

            Komut::AralikDongusu { degisken, baslangic, bitis, govde } => {
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
                let cond = self.builder.ins().fcmp(FloatCC::LessThanOrEqual, cur, end_val);
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
                if let Ifade::IkiliIslem { sol, operator: Token::Esittir, sag } = ifade {
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
            Ifade::Dogru  => Some(self.builder.ins().f64const(1.0)),
            Ifade::Yanlis => Some(self.builder.ins().f64const(0.0)),
            Ifade::Bos    => Some(self.builder.ins().f64const(0.0)),

            Ifade::Degisken(ad) => {
                let var = self.get_or_create_var(ad);
                Some(self.builder.use_var(var))
            }

            Ifade::IkiliIslem { sol, operator, sag } => {
                let l = self.emit_expr(sol, module)?;
                let r = self.emit_expr(sag, module)?;
                self.emit_binop(operator, l, r)
            }

            Ifade::Cagri { fonksiyon, argumanlar, .. } => {
                let mut arg_vals = Vec::with_capacity(argumanlar.len());
                for arg in argumanlar {
                    arg_vals.push(self.emit_expr(arg, module)?);
                }

                if let Ifade::Degisken(fn_name) = fonksiyon.as_ref() {
                    if let Some(&(fid, _)) = self.fn_registry.get(fn_name) {
                        let fref = module.declare_func_in_func(fid, &mut self.builder.func);
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
            Token::Arti   => Some(self.builder.ins().fadd(l, r)),
            Token::Eksi   => Some(self.builder.ins().fsub(l, r)),
            Token::Carpi  => Some(self.builder.ins().fmul(l, r)),
            Token::Bolnu  => Some(self.builder.ins().fdiv(l, r)),

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