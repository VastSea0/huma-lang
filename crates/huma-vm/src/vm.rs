use crate::bytecode::{validate_program, Constant, OpCode, Program};
use crate::error::{HumaError, HumaResult, RuntimeDiagnostic, SourceSpan, StackFrame};
use crate::gc::Gc;
use crate::gc::HeapSweepGuard;
use crate::token::Token;
use crate::value::{BuiltinRuntime, Deger};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

struct CallFrame {
    function_index: Option<usize>,
    function_name: Option<String>,
    function_location: Option<SourceSpan>,
    current_location: Option<SourceSpan>,
    ip: usize,
    locals: HashMap<String, Deger>,
    stack_base: usize,
    error_stack: Vec<(usize, usize)>,
}

struct SyncCallBoundary {
    frame_depth: usize,
    error: Option<RuntimeDiagnostic>,
}

/// Native ABI uygulamasını VM'den ayıran ana makine sözleşmesi.
pub trait NativeCallHost {
    fn call(
        &self,
        library: &str,
        function: &str,
        signature: &str,
        arguments: Vec<Deger>,
    ) -> Result<Deger, String>;
}

/// Asenkron görev uygulamasını VM'den ayıran ana makine sözleşmesi.
pub trait TaskHost {
    fn await_task(&self, id: u64) -> Deger;
}

pub struct VM {
    stack: Vec<Deger>,
    globals: HashMap<String, Deger>,
    program: Program,
    function_index: Option<usize>,
    function_name: Option<String>,
    function_location: Option<SourceSpan>,
    current_location: Option<SourceSpan>,
    ip: usize,
    locals: HashMap<String, Deger>,
    stack_base: usize,
    error_stack: Vec<(usize, usize)>,
    frames: Vec<CallFrame>,
    sync_call_boundaries: Vec<SyncCallBoundary>,
    pub call_depth: usize,
    runtime_error: Option<RuntimeDiagnostic>,
    output_buffer: Option<Rc<RefCell<String>>>,
    limits: crate::limits::ExecutionLimits,
    executed_steps: u64,
    output_bytes: usize,
    native_call_host: Option<Rc<dyn NativeCallHost>>,
    task_host: Option<Rc<dyn TaskHost>>,
    // VM kökleri bırakıldıktan sonra iş parçacığı heap'ini tarar.
    _heap_sweep: HeapSweepGuard,
}

impl VM {
    pub fn new(program: Program) -> Self {
        let validation_error = validate_program(&program)
            .err()
            .map(|error| RuntimeDiagnostic {
                message: error.to_string(),
                location: None,
                stack: Vec::new(),
            });
        Self {
            stack: Vec::new(),
            globals: crate::interpreter::varsayilan_global_degiskenler(),
            program,
            function_index: None,
            function_name: None,
            function_location: None,
            current_location: None,
            ip: 0,
            locals: HashMap::new(),
            stack_base: 0,
            error_stack: Vec::new(),
            frames: Vec::new(),
            sync_call_boundaries: Vec::new(),
            call_depth: 0,
            runtime_error: validation_error,
            output_buffer: None,
            limits: crate::limits::ExecutionLimits::default(),
            executed_steps: 0,
            output_bytes: 0,
            native_call_host: None,
            task_host: None,
            _heap_sweep: HeapSweepGuard,
        }
    }

    pub fn with_output_buffer(mut self, buffer: Rc<RefCell<String>>) -> Self {
        self.output_buffer = Some(buffer);
        self
    }

    pub fn with_native_call_host(mut self, host: Rc<dyn NativeCallHost>) -> Self {
        self.native_call_host = Some(host);
        self
    }

    pub fn with_task_host(mut self, host: Rc<dyn TaskHost>) -> Self {
        self.task_host = Some(host);
        self
    }

    pub fn with_limits(mut self, limits: crate::limits::ExecutionLimits) -> Result<Self, String> {
        self.limits = limits.validate()?;
        Ok(self)
    }

    pub fn run_checked(&mut self) -> HumaResult<()> {
        self.run();
        match self.runtime_error.take() {
            Some(message) => Err(HumaError::RuntimeError(message)),
            None => Ok(()),
        }
    }

    pub fn run(&mut self) {
        if self.runtime_error.is_some() {
            return;
        }
        self.executed_steps = 0;
        self.output_bytes = 0;
        while self.tek_adim() {}
    }

    fn tek_adim(&mut self) -> bool {
        if self.runtime_error.is_some()
            || self
                .sync_call_boundaries
                .last()
                .is_some_and(|boundary| boundary.error.is_some())
        {
            return false;
        }
        let Some(opcode) = self.current_instructions().get(self.ip).cloned() else {
            if self.function_index.is_some() {
                self.fonksiyondan_don(Deger::Bos);
                return true;
            }
            return false;
        };
        self.current_location = self.current_spans().get(self.ip).copied().flatten();
        self.executed_steps = self.executed_steps.saturating_add(1);
        if self.executed_steps > self.limits.max_steps {
            self.hata_firlat(format!(
                "Çalıştırma adım sınırı aşıldı: {}",
                self.limits.max_steps
            ));
            return false;
        }
        self.ip += 1;
        self.execute(opcode);
        self.runtime_error.is_none()
            && self
                .sync_call_boundaries
                .last()
                .is_none_or(|boundary| boundary.error.is_none())
    }

    fn current_instructions(&self) -> &[OpCode] {
        match self.function_index {
            Some(index) => self
                .program
                .functions
                .get(index)
                .map_or(&[], |function| function.instructions.as_slice()),
            None => &self.program.instructions,
        }
    }

    fn current_spans(&self) -> &[Option<SourceSpan>] {
        match self.function_index {
            Some(index) => self
                .program
                .functions
                .get(index)
                .map_or(&[], |function| function.instruction_spans.as_slice()),
            None => &self.program.instruction_spans,
        }
    }

    fn execute(&mut self, opcode: OpCode) {
        match opcode {
            OpCode::PushConstant(index) => match self.program.constants.get(index) {
                Some(Constant::Sayi(number)) => self.stack.push(Deger::Sayi(*number)),
                Some(Constant::Metin(text)) => self.stack.push(Deger::Metin(text.clone())),
                None => self.hata_firlat(format!("Geçersiz sabit indeksi: {index}")),
            },
            OpCode::LoadVar(name) => match self
                .locals
                .get(name.as_ref())
                .or_else(|| self.globals.get(name.as_ref()))
                .cloned()
            {
                Some(value) => self.stack.push(value),
                None => self.hata_firlat(format!("Tanımsız değişken: {name}")),
            },
            OpCode::StoreVar(name) => {
                let Some(value) = self.pop_value("değişken ataması") else {
                    return;
                };
                if let Some(current) = self.locals.get_mut(name.as_ref()) {
                    *current = value;
                } else if let Some(current) = self.globals.get_mut(name.as_ref()) {
                    *current = value;
                } else {
                    self.hata_firlat(format!("Tanımlanmamış değişkene atama yapılamaz: {name}"));
                }
            }
            OpCode::DefineVar(name) => {
                let Some(value) = self.pop_value("değişken tanımı") else {
                    return;
                };
                self.degisken_tanimla(name.to_string(), value);
            }
            OpCode::Add => self.ikili_islem_calistir(Token::Arti),
            OpCode::Sub => self.ikili_islem_calistir(Token::Eksi),
            OpCode::Mul => self.ikili_islem_calistir(Token::Carpi),
            OpCode::Div => self.ikili_islem_calistir(Token::Bolnu),
            OpCode::Mod => self.ikili_islem_calistir(Token::Mod),
            OpCode::Greater => self.ikili_islem_calistir(Token::Buyuktur),
            OpCode::Less => self.ikili_islem_calistir(Token::Kucuktur),
            OpCode::LessOrEqual => self.ikili_islem_calistir(Token::KucukEsit),
            OpCode::GreaterOrEqual => self.ikili_islem_calistir(Token::BuyukEsit),
            OpCode::Equal => self.ikili_islem_calistir(Token::EsitEsittir),
            OpCode::NotEqual => self.ikili_islem_calistir(Token::EsitDegil),
            OpCode::And | OpCode::Or => {
                let Some(right) = self.pop_value("mantıksal işlem") else {
                    return;
                };
                let Some(left) = self.pop_value("mantıksal işlem") else {
                    return;
                };
                let left = match crate::semantics::dogru_mu(&left) {
                    Ok(value) => value,
                    Err(error) => {
                        self.hata_firlat(error);
                        return;
                    }
                };
                let right = match crate::semantics::dogru_mu(&right) {
                    Ok(value) => value,
                    Err(error) => {
                        self.hata_firlat(error);
                        return;
                    }
                };
                let value = if matches!(opcode, OpCode::And) {
                    left && right
                } else {
                    left || right
                };
                self.stack.push(Deger::Sayi(if value { 1.0 } else { 0.0 }));
            }
            OpCode::Not => {
                let Some(value) = self.pop_value("mantıksal değil") else {
                    return;
                };
                match crate::semantics::dogru_mu(&value) {
                    Ok(value) => self.stack.push(Deger::Sayi(if value { 0.0 } else { 1.0 })),
                    Err(error) => self.hata_firlat(error),
                }
            }
            OpCode::Length => {
                let Some(value) = self.pop_value("uzunluk") else {
                    return;
                };
                match self.uzunluk(value) {
                    Ok(length) => self.stack.push(Deger::Sayi(length as f64)),
                    Err(error) => self.hata_firlat(error),
                }
            }
            OpCode::Call(argument_count) => self.cagri_calistir(argument_count),
            OpCode::Print => {
                let Some(value) = self.pop_value("yazdır") else {
                    return;
                };
                let text = match value.to_string_limited(self.limits.max_output_bytes) {
                    Ok(text) => text,
                    Err(error) => {
                        self.hata_firlat(error);
                        return;
                    }
                };
                if let Err(error) = self.satir_yazdir(&text) {
                    self.hata_firlat(error);
                }
            }
            OpCode::Jump(address) => self.ip = address,
            OpCode::JumpIfFalse(address) => {
                let Some(value) = self.pop_value("koşullu atlama") else {
                    return;
                };
                match crate::semantics::dogru_mu(&value) {
                    Ok(false) => self.ip = address,
                    Ok(true) => {}
                    Err(error) => self.hata_firlat(error),
                }
            }
            OpCode::Pop => {
                if self.stack.pop().is_none() {
                    self.hata_firlat("Pop komutu boş yığınla çalıştırılamaz".to_string());
                }
            }
            OpCode::Return => {
                let Some(value) = self.pop_value("döndür") else {
                    return;
                };
                self.fonksiyondan_don(value);
            }
            OpCode::Bos => self.stack.push(Deger::Bos),
            OpCode::MakeList(length) => {
                if length > self.limits.max_collection_items {
                    self.hata_firlat(format!(
                        "Liste eleman sınırı aşıldı: {} > {}",
                        length, self.limits.max_collection_items
                    ));
                    return;
                }
                let Some(values) = self.pop_values(length, "liste oluşturma") else {
                    return;
                };
                self.stack
                    .push(Deger::Liste(Gc::from_cell(RefCell::new(values))));
            }
            OpCode::ListAccess => {
                let Some(index) = self.pop_value("koleksiyon erişimi") else {
                    return;
                };
                let Some(container) = self.pop_value("koleksiyon erişimi") else {
                    return;
                };
                match self.koleksiyon_erisimi(container, index) {
                    Ok(value) => self.stack.push(value),
                    Err(error) => self.hata_firlat(error),
                }
            }
            OpCode::MakeMap(length) => {
                if length > self.limits.max_collection_items {
                    self.hata_firlat(format!(
                        "Sözlük eleman sınırı aşıldı: {} > {}",
                        length, self.limits.max_collection_items
                    ));
                    return;
                }
                let pair_value_count = match length.checked_mul(2) {
                    Some(value) => value,
                    None => {
                        self.hata_firlat("Sözlük eleman sayısı taştı".to_string());
                        return;
                    }
                };
                let Some(values) = self.pop_values(pair_value_count, "sözlük oluşturma") else {
                    return;
                };
                let mut map = HashMap::with_capacity(length);
                for pair in values.chunks_exact(2) {
                    match &pair[0] {
                        Deger::Metin(key) => {
                            map.insert(key.clone(), pair[1].clone());
                        }
                        other => {
                            self.hata_firlat(format!(
                                "Sözlük anahtarı metin olmalıdır; {other} geldi"
                            ));
                            return;
                        }
                    }
                }
                self.stack
                    .push(Deger::Sozluk(Gc::from_cell(RefCell::new(map))));
            }
            OpCode::TryBlockStart(address) => {
                self.error_stack.push((address, self.stack.len()));
            }
            OpCode::TryBlockEnd => {
                if self.error_stack.pop().is_none() {
                    self.hata_firlat("Eşleşmeyen TryBlockEnd komutu".to_string());
                }
            }
            OpCode::Await => {
                let Some(value) = self.pop_value("bekle") else {
                    return;
                };
                match value {
                    Deger::GorevId(id) => match &self.task_host {
                        Some(host) => match host.await_task(id) {
                            Deger::Hata(error) => self.hata_firlat(error),
                            result => self.stack.push(result),
                        },
                        None => self.hata_firlat(
                            "Asenkron görev ana makinesi yapılandırılmadı".to_string(),
                        ),
                    },
                    other => {
                        self.hata_firlat(format!("bekle: await edilemez değer: {other}"));
                    }
                }
            }
            OpCode::CallFFI {
                lib_ad,
                fn_ad,
                signature,
                arg_len,
            } => {
                if let Err(error) =
                    crate::capability::require(crate::capability::Capability::Ffi, "FFI çağrısı")
                {
                    self.hata_firlat(error);
                    return;
                }
                let Some(args) = self.pop_values(arg_len, "FFI çağrısı") else {
                    return;
                };
                let result = match &self.native_call_host {
                    Some(host) => host.call(&lib_ad, &fn_ad, &signature, args),
                    None => Err("FFI ana makinesi yapılandırılmadı".to_string()),
                };
                match result {
                    Ok(Deger::Hata(error)) | Err(error) => self.hata_firlat(error),
                    Ok(value) => self.stack.push(value),
                }
            }
            OpCode::DefineFunction {
                name,
                function_index,
            } => {
                let function = Deger::BytecodeFonksiyon {
                    ad: Some(name.to_string()),
                    function_index,
                    yakalanan_degiskenler: self.locals.clone(),
                };
                self.degisken_tanimla(name.to_string(), function);
            }
            OpCode::MakeClosure(function_index) => {
                self.stack.push(Deger::BytecodeFonksiyon {
                    ad: None,
                    function_index,
                    yakalanan_degiskenler: self.locals.clone(),
                });
            }
        }
    }

    fn cagri_calistir(&mut self, argument_count: usize) {
        let Some(callable) = self.pop_value("fonksiyon çağrısı") else {
            return;
        };
        let Some(args) = self.pop_values(argument_count, "fonksiyon çağrısı") else {
            return;
        };
        match callable {
            Deger::DahiliFonksiyon(function) => {
                let result =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| function(args)));
                match result {
                    Ok(Deger::Hata(error)) => self.hata_firlat(error),
                    Ok(value) => self.stack.push(value),
                    Err(payload) => self.hata_firlat(format!(
                        "Yerleşik fonksiyon paniği güvenli biçimde yakalandı: {}",
                        crate::error::panik_mesaji(payload)
                    )),
                }
            }
            Deger::BaglamliDahiliFonksiyon(function) => {
                let result =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| function(self, args)));
                match result {
                    Ok(Deger::Hata(error)) => self.hata_firlat(error),
                    Ok(value) => self.stack.push(value),
                    Err(payload) => self.hata_firlat(format!(
                        "Bağlamlı yerleşik fonksiyon paniği güvenli biçimde yakalandı: {}",
                        crate::error::panik_mesaji(payload)
                    )),
                }
            }
            Deger::BytecodeFonksiyon {
                ad,
                function_index,
                yakalanan_degiskenler,
            } => {
                let function_name = ad
                    .clone()
                    .unwrap_or_else(|| format!("<anonim:{function_index}>"));
                if let Err(error) =
                    self.bytecode_cagrisi_baslat(ad, function_index, yakalanan_degiskenler, args)
                {
                    self.hata_firlat_cerceveli(
                        error,
                        Some(StackFrame {
                            function: function_name,
                            location: self.current_location,
                        }),
                    );
                }
            }
            Deger::Fonksiyon { .. } => self.hata_firlat(
                "AST fonksiyonu VM içinde yürütülemez; kaynak yeniden derlenmeli".to_string(),
            ),
            other => self.hata_firlat(format!("Çağrılamayan değer: {other}")),
        }
    }

    fn bytecode_cagrisi_baslat(
        &mut self,
        ad: Option<String>,
        function_index: usize,
        yakalanan_degiskenler: HashMap<String, Deger>,
        args: Vec<Deger>,
    ) -> Result<(), String> {
        if self.call_depth >= self.limits.max_call_depth {
            return Err("Azami özyineleme derinliği aşıldı".to_string());
        }
        let prototype = self
            .program
            .functions
            .get(function_index)
            .ok_or_else(|| format!("Geçersiz fonksiyon indeksi: {function_index}"))?;
        if args.len() != prototype.params.len() {
            return Err(format!(
                "Fonksiyon {} argüman bekliyor; {} argüman geldi",
                prototype.params.len(),
                args.len()
            ));
        }
        let params = prototype.params.clone();
        let callable_copy = Deger::BytecodeFonksiyon {
            ad: ad.clone(),
            function_index,
            yakalanan_degiskenler: yakalanan_degiskenler.clone(),
        };
        let mut locals = yakalanan_degiskenler;
        if let Some(name) = &ad {
            locals.insert(name.clone(), callable_copy);
        }
        for (parameter, value) in params.into_iter().zip(args) {
            locals.insert(parameter, value);
        }

        let frame = CallFrame {
            function_index: self.function_index,
            function_name: self.function_name.take(),
            function_location: self.function_location.take(),
            current_location: self.current_location,
            ip: self.ip,
            locals: std::mem::take(&mut self.locals),
            stack_base: self.stack_base,
            error_stack: std::mem::take(&mut self.error_stack),
        };
        self.frames.push(frame);
        self.function_index = Some(function_index);
        self.function_name = ad;
        self.function_location = self.current_location;
        self.current_location = None;
        self.ip = 0;
        self.locals = locals;
        self.stack_base = self.stack.len();
        self.call_depth += 1;
        Ok(())
    }

    fn bytecode_degerini_eszamanli_cagir(
        &mut self,
        ad: Option<String>,
        function_index: usize,
        yakalanan_degiskenler: HashMap<String, Deger>,
        args: Vec<Deger>,
    ) -> Deger {
        let frame_depth = self.frames.len();
        let stack_height = self.stack.len();
        self.sync_call_boundaries.push(SyncCallBoundary {
            frame_depth,
            error: None,
        });
        if let Err(error) =
            self.bytecode_cagrisi_baslat(ad, function_index, yakalanan_degiskenler, args)
        {
            self.sync_call_boundaries.pop();
            return Deger::Hata(error);
        }

        while self.frames.len() > frame_depth {
            if !self.tek_adim() {
                break;
            }
        }
        let Some(boundary) = self.sync_call_boundaries.pop() else {
            self.stack.truncate(stack_height);
            return Deger::Hata(
                "İç hata: eşzamanlı VM geri çağrı sınırı beklenmedik biçimde kayboldu".to_string(),
            );
        };
        if let Some(error) = boundary.error {
            self.stack.truncate(stack_height);
            return Deger::Hata(error.to_string());
        }
        if self.runtime_error.is_some() {
            self.stack.truncate(stack_height);
            return Deger::Hata(
                "Eşzamanlı VM geri çağrısı üst düzey çalışma zamanı hatasıyla sonlandı".to_string(),
            );
        }
        if self.frames.len() != frame_depth {
            self.stack.truncate(stack_height);
            return Deger::Hata(
                "İç hata: eşzamanlı VM geri çağrısı çağrı çerçevesini geri yüklemedi".to_string(),
            );
        }
        let Some(expected_stack_height) = stack_height.checked_add(1) else {
            self.stack.truncate(stack_height);
            return Deger::Hata(
                "İç hata: eşzamanlı VM geri çağrı yığın yüksekliği taştı".to_string(),
            );
        };
        if self.stack.len() != expected_stack_height {
            self.stack.truncate(stack_height);
            return Deger::Hata(
                "İç hata: eşzamanlı VM geri çağrısı tam olarak bir sonuç üretmedi".to_string(),
            );
        }
        match self.stack.pop() {
            Some(value) => value,
            None => Deger::Hata("İç hata: eşzamanlı VM geri çağrı sonucu kayboldu".to_string()),
        }
    }

    fn fonksiyondan_don(&mut self, value: Deger) {
        let Some(frame) = self.frames.pop() else {
            self.function_index = None;
            self.function_name = None;
            self.function_location = None;
            self.current_location = None;
            self.ip = self.program.instructions.len();
            self.stack.clear();
            self.stack.push(value);
            return;
        };
        self.stack.truncate(self.stack_base);
        self.function_index = frame.function_index;
        self.function_name = frame.function_name;
        self.function_location = frame.function_location;
        self.current_location = frame.current_location;
        self.ip = frame.ip;
        self.locals = frame.locals;
        self.stack_base = frame.stack_base;
        self.error_stack = frame.error_stack;
        self.call_depth = self.call_depth.saturating_sub(1);
        self.stack.push(value);
    }

    fn hata_firlat(&mut self, message: String) {
        self.hata_firlat_cerceveli(message, None);
    }

    fn hata_firlat_cerceveli(&mut self, message: String, initial_frame: Option<StackFrame>) {
        let error_location = self.current_location;
        let mut trace = initial_frame.into_iter().collect::<Vec<_>>();
        loop {
            if let Some((handler_address, stack_height)) = self.error_stack.pop() {
                self.stack.truncate(stack_height);
                self.ip = handler_address;
                let diagnostic = RuntimeDiagnostic {
                    message,
                    location: error_location,
                    stack: trace,
                };
                self.stack.push(Deger::Hata(diagnostic.to_string()));
                return;
            }

            if let Some(name) = self.function_name.take() {
                trace.push(StackFrame {
                    function: name,
                    location: self.function_location,
                });
            } else if let Some(index) = self.function_index {
                trace.push(StackFrame {
                    function: format!("<anonim:{index}>"),
                    location: self.function_location,
                });
            }

            let Some(frame) = self.frames.pop() else {
                self.runtime_error = Some(RuntimeDiagnostic {
                    message,
                    location: error_location,
                    stack: trace,
                });
                self.function_index = None;
                self.function_location = None;
                self.current_location = None;
                self.ip = self.program.instructions.len();
                self.stack.clear();
                self.locals.clear();
                self.error_stack.clear();
                self.call_depth = 0;
                return;
            };
            self.stack.truncate(self.stack_base);
            self.function_index = frame.function_index;
            self.function_name = frame.function_name;
            self.function_location = frame.function_location;
            self.current_location = frame.current_location;
            self.ip = frame.ip;
            self.locals = frame.locals;
            self.stack_base = frame.stack_base;
            self.error_stack = frame.error_stack;
            self.call_depth = self.call_depth.saturating_sub(1);
            if let Some(boundary) = self.sync_call_boundaries.last_mut() {
                if self.frames.len() == boundary.frame_depth {
                    boundary.error = Some(RuntimeDiagnostic {
                        message,
                        location: error_location,
                        stack: trace,
                    });
                    return;
                }
            }
        }
    }

    fn degisken_tanimla(&mut self, name: String, value: Deger) {
        if self.function_index.is_some() {
            self.locals.insert(name, value);
        } else {
            self.globals.insert(name, value);
        }
    }

    fn pop_value(&mut self, operation: &str) -> Option<Deger> {
        match self.stack.pop() {
            Some(value) => Some(value),
            None => {
                self.hata_firlat(format!("{operation}: çalışma yığını boş"));
                None
            }
        }
    }

    fn pop_values(&mut self, count: usize, operation: &str) -> Option<Vec<Deger>> {
        if self.stack.len() < count {
            self.hata_firlat(format!(
                "{operation}: {count} değer gerekiyor, yığında {} değer var",
                self.stack.len()
            ));
            return None;
        }
        let start = self.stack.len() - count;
        Some(self.stack.drain(start..).collect())
    }

    fn ikili_islem_calistir(&mut self, operator: Token) {
        let Some(right) = self.pop_value("ikili işlem") else {
            return;
        };
        let Some(left) = self.pop_value("ikili işlem") else {
            return;
        };
        match crate::semantics::ikili_islem(&operator, left, right) {
            Ok(result) => self.stack.push(result),
            Err(error) => self.hata_firlat(error),
        }
    }

    fn uzunluk(&self, value: Deger) -> Result<usize, String> {
        match value {
            Deger::Metin(text) => Ok(text.chars().count()),
            Deger::Bayt(bytes) => Ok(bytes.len()),
            Deger::Liste(items) => items
                .try_borrow()
                .map(|borrowed| borrowed.len())
                .map_err(|_| "Liste uzunluğu alınırken liste kullanımda".to_string()),
            Deger::Sozluk(items) => items
                .try_borrow()
                .map(|borrowed| borrowed.len())
                .map_err(|_| "Sözlük uzunluğu alınırken sözlük kullanımda".to_string()),
            Deger::Vektor(items) => items
                .try_borrow()
                .map(|borrowed| borrowed.len())
                .map_err(|_| "Vektör uzunluğu alınırken vektör kullanımda".to_string()),
            other => Err(format!("{other} değerinin uzunluğu alınamaz")),
        }
    }

    fn sayisal_indeks(value: f64, length: usize) -> Result<usize, String> {
        if !value.is_finite() || value < 0.0 || value.fract() != 0.0 {
            return Err("Koleksiyon indeksi negatif olmayan sonlu tamsayı olmalı".to_string());
        }
        if value >= length as f64 {
            return Err(format!(
                "Koleksiyon indeksi sınır dışında: {value} (uzunluk {length})"
            ));
        }
        Ok(value as usize)
    }

    fn koleksiyon_erisimi(&self, container: Deger, index: Deger) -> Result<Deger, String> {
        match (container, index) {
            (Deger::Liste(items), Deger::Sayi(number)) => {
                let borrowed = items
                    .try_borrow()
                    .map_err(|_| "Liste erişimi sırasında liste kullanımda".to_string())?;
                let index = Self::sayisal_indeks(number, borrowed.len())?;
                Ok(borrowed[index].clone())
            }
            (Deger::Metin(text), Deger::Sayi(number)) => {
                let length = text.chars().count();
                let index = Self::sayisal_indeks(number, length)?;
                text.chars()
                    .nth(index)
                    .map(|character| Deger::Metin(character.to_string()))
                    .ok_or_else(|| "Metin indeksi çözülemedi".to_string())
            }
            (Deger::Sozluk(map), Deger::Metin(key)) => map
                .try_borrow()
                .map_err(|_| "Sözlük erişimi sırasında sözlük kullanımda".to_string())
                .map(|borrowed| borrowed.get(&key).cloned().unwrap_or(Deger::Bos)),
            (Deger::Nesne { alanlar, .. }, Deger::Metin(key)) => alanlar
                .try_borrow()
                .map_err(|_| "Nesne erişimi sırasında nesne kullanımda".to_string())
                .map(|borrowed| borrowed.get(&key).cloned().unwrap_or(Deger::Bos)),
            (container, index) => Err(format!(
                "{container} değerine {index} indeksiyle erişilemez"
            )),
        }
    }

    fn satir_yazdir(&mut self, content: &str) -> Result<(), String> {
        let byte_count = content
            .len()
            .checked_add(1)
            .ok_or_else(|| "Çıktı boyutu hesaplanırken taştı".to_string())?;
        let next_output = self
            .output_bytes
            .checked_add(byte_count)
            .filter(|next| *next <= self.limits.max_output_bytes)
            .ok_or_else(|| {
                format!(
                    "Çıktı sınırı aşıldı: en fazla {} bayt",
                    self.limits.max_output_bytes
                )
            })?;
        if let Some(buffer) = self.output_buffer.clone() {
            let mut borrowed = buffer
                .try_borrow_mut()
                .map_err(|_| "Çıktı tamponu kullanımda".to_string())?;
            borrowed.push_str(content);
            borrowed.push('\n');
        } else {
            println!("{content}");
        }
        self.output_bytes = next_output;
        Ok(())
    }
}

impl BuiltinRuntime for VM {
    fn call_value(&mut self, function: Deger, args: Vec<Deger>) -> Deger {
        match function {
            Deger::DahiliFonksiyon(function) => {
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| function(args))) {
                    Ok(value) => value,
                    Err(payload) => Deger::Hata(format!(
                        "Yerleşik fonksiyon paniği güvenli biçimde yakalandı: {}",
                        crate::error::panik_mesaji(payload)
                    )),
                }
            }
            Deger::BaglamliDahiliFonksiyon(function) => {
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    function(self, args)
                })) {
                    Ok(value) => value,
                    Err(payload) => Deger::Hata(format!(
                        "Bağlamlı yerleşik fonksiyon paniği güvenli biçimde yakalandı: {}",
                        crate::error::panik_mesaji(payload)
                    )),
                }
            }
            Deger::BytecodeFonksiyon {
                ad,
                function_index,
                yakalanan_degiskenler,
            } => self.bytecode_degerini_eszamanli_cagir(
                ad,
                function_index,
                yakalanan_degiskenler,
                args,
            ),
            Deger::Fonksiyon { .. } => {
                Deger::Hata("AST fonksiyonu VM içinde yürütülemez".to_string())
            }
            other => Deger::Hata(format!("Çağrılamayan değer: {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VM;
    use crate::bytecode::{Constant, FunctionPrototype, OpCode, Program};
    use crate::value::{BuiltinRuntime, Deger};
    use std::cell::RefCell;
    use std::rc::Rc;

    fn geri_cagir(runtime: &mut dyn BuiltinRuntime, args: Vec<Deger>) -> Deger {
        let [callback] = args.as_slice() else {
            return Deger::Hata("test geri çağrısı bir fonksiyon bekliyor".to_string());
        };
        runtime.call_value(callback.clone(), Vec::new())
    }

    fn callback_program(function_instructions: Vec<OpCode>) -> Program {
        Program {
            constants: vec![Constant::Sayi(7.0)],
            functions: vec![FunctionPrototype {
                params: Vec::new(),
                instruction_spans: vec![None; function_instructions.len()],
                instructions: function_instructions,
            }],
            instructions: vec![
                OpCode::DefineFunction {
                    name: "sonuc".into(),
                    function_index: 0,
                },
                OpCode::LoadVar("sonuc".into()),
                OpCode::LoadVar("geri_cagir".into()),
                OpCode::Call(1),
                OpCode::Print,
            ],
            instruction_spans: vec![None; 5],
        }
    }

    #[test]
    fn baglamli_yerlesik_bytecode_callbackini_eszamanli_calistirir() {
        let output = Rc::new(RefCell::new(String::new()));
        let mut vm = VM::new(callback_program(vec![
            OpCode::PushConstant(0),
            OpCode::Return,
        ]))
        .with_output_buffer(output.clone());
        vm.globals.insert(
            "geri_cagir".to_string(),
            Deger::BaglamliDahiliFonksiyon(geri_cagir),
        );

        vm.run_checked()
            .expect("Bağlamlı bytecode geri çağrısı çalışmalı");
        assert_eq!(output.borrow().as_str(), "7\n");
        assert_eq!(vm.call_depth, 0);
        assert!(vm.frames.is_empty());
    }

    #[test]
    fn baglamli_bytecode_callback_hatasi_cagri_cercevesini_bozmaz() {
        let mut vm = VM::new(callback_program(vec![
            OpCode::LoadVar("tanimsiz".into()),
            OpCode::Return,
        ]));
        vm.globals.insert(
            "geri_cagir".to_string(),
            Deger::BaglamliDahiliFonksiyon(geri_cagir),
        );

        let error = vm
            .run_checked()
            .expect_err("Callback çalışma zamanı hatası vermeli");
        assert!(error.to_string().contains("Tanımsız değişken"));
        assert_eq!(vm.call_depth, 0);
        assert!(vm.frames.is_empty());
        assert!(vm.sync_call_boundaries.is_empty());
    }
}
