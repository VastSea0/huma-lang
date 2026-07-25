use crate::bytecode::{Constant, OpCode, Program};
use crate::error::{HumaError, HumaResult};
use crate::value::Deger;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use tokio::runtime::{Builder, Runtime};
use tokio::task::JoinHandle;
use tokio::task::LocalSet;

pub struct VM {
    stack: Vec<Deger>,
    globals: HashMap<String, Deger>,
    program: Program,
    ip: usize,
    error_stack: Vec<usize>,
    yaprak: YaprakExecutor,
    pub call_depth: usize,
    runtime_error: Option<String>,
    output_buffer: Option<Rc<RefCell<String>>>,
}

#[allow(dead_code)]
struct YaprakExecutor {
    rt: Runtime,
    local: LocalSet,
    next_id: u64,
    tasks: HashMap<u64, JoinHandle<Deger>>,
}

#[allow(dead_code)]
impl YaprakExecutor {
    fn new() -> Self {
        Self {
            rt: Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio runtime init failed"),
            local: LocalSet::new(),
            next_id: 1,
            tasks: HashMap::new(),
        }
    }

    fn insert(&mut self, task: JoinHandle<Deger>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.tasks.insert(id, task);
        id
    }

    fn await_task(&mut self, id: u64) -> Deger {
        match self.tasks.remove(&id) {
            Some(handle) => match self.rt.block_on(self.local.run_until(handle)) {
                Ok(v) => v,
                Err(e) => Deger::Hata(format!("Görev hatası: {}", e)),
            },
            None => Deger::Hata(format!("Bilinmeyen görev: {}", id)),
        }
    }
}

impl VM {
    pub fn new(program: Program) -> Self {
        Self {
            stack: Vec::new(),
            globals: crate::interpreter::varsayilan_global_degiskenler(),
            program,
            ip: 0,
            error_stack: Vec::new(),
            yaprak: YaprakExecutor::new(),
            call_depth: 0,
            runtime_error: None,
            output_buffer: None,
        }
    }

    pub fn with_output_buffer(mut self, buffer: Rc<RefCell<String>>) -> Self {
        self.output_buffer = Some(buffer);
        self
    }

    pub fn run_checked(&mut self) -> HumaResult<()> {
        self.run();
        match self.runtime_error.take() {
            Some(message) => Err(HumaError::RuntimeError(message)),
            None => Ok(()),
        }
    }

    pub fn run(&mut self) {
        while self.ip < self.program.instructions.len() {
            let op = &self.program.instructions[self.ip];
            self.ip += 1;
            match op {
                OpCode::PushConstant(idx) => {
                    let c = &self.program.constants[*idx];
                    match c {
                        Constant::Sayi(n) => self.stack.push(Deger::Sayi(*n)),
                        Constant::Metin(s) => self.stack.push(Deger::Metin(s.clone())),
                    }
                }
                OpCode::LoadVar(ad) => match self.globals.get(ad).cloned() {
                    Some(val) => self.stack.push(val),
                    None => self.hata_firlat(format!("Tanımsız değişken: {}", ad)),
                },
                OpCode::StoreVar(ad) => {
                    let val = self.stack.pop().unwrap_or(Deger::Bos);
                    if self.globals.contains_key(ad) {
                        self.globals.insert(ad.clone(), val);
                    } else {
                        self.hata_firlat(format!(
                            "Tanımlanmamış değişkene atama yapılamaz: {}",
                            ad
                        ));
                    }
                }
                OpCode::DefineVar(ad) => {
                    let val = self.stack.pop().unwrap_or(Deger::Bos);
                    self.globals.insert(ad.clone(), val);
                }
                OpCode::Add => {
                    let r = self.stack.pop().unwrap_or(Deger::Bos);
                    let l = self.stack.pop().unwrap_or(Deger::Bos);
                    match (l, r) {
                        (Deger::Sayi(a), Deger::Sayi(b)) => self.stack.push(Deger::Sayi(a + b)),
                        (Deger::Metin(a), Deger::Metin(b)) => self.stack.push(Deger::Metin(a + &b)),
                        (Deger::Metin(a), b) => {
                            self.stack.push(Deger::Metin(format!("{}{}", a, b)))
                        }
                        (a, Deger::Metin(b)) => {
                            self.stack.push(Deger::Metin(format!("{}{}", a, b)))
                        }
                        (a, b) => self.hata_firlat(format!(
                            "Toplama işlemi bu değerlerde desteklenmez: {} ve {}",
                            a, b
                        )),
                    }
                }
                OpCode::Sub => {
                    let r = self.stack.pop().unwrap_or(Deger::Bos);
                    let l = self.stack.pop().unwrap_or(Deger::Bos);
                    if let (Deger::Sayi(a), Deger::Sayi(b)) = (l, r) {
                        self.stack.push(Deger::Sayi(a - b));
                    } else {
                        self.hata_firlat(
                            "Çıkarma işlemi yalnızca sayılarda desteklenir".to_string(),
                        );
                    }
                }
                OpCode::Mul => {
                    let r = self.stack.pop().unwrap_or(Deger::Bos);
                    let l = self.stack.pop().unwrap_or(Deger::Bos);
                    if let (Deger::Sayi(a), Deger::Sayi(b)) = (l, r) {
                        self.stack.push(Deger::Sayi(a * b));
                    } else {
                        self.hata_firlat(
                            "Çarpma işlemi yalnızca sayılarda desteklenir".to_string(),
                        );
                    }
                }
                OpCode::Div => {
                    let r = self.stack.pop().unwrap_or(Deger::Bos);
                    let l = self.stack.pop().unwrap_or(Deger::Bos);
                    if let (Deger::Sayi(a), Deger::Sayi(b)) = (l, r) {
                        if b == 0.0 {
                            self.hata_firlat("Sıfıra bölme hatası".to_string());
                        } else {
                            self.stack.push(Deger::Sayi(a / b));
                        }
                    } else {
                        self.hata_firlat("Bölme işlemi yalnızca sayılarda desteklenir".to_string());
                    }
                }
                OpCode::Mod => {
                    let r = self.stack.pop().unwrap_or(Deger::Bos);
                    let l = self.stack.pop().unwrap_or(Deger::Bos);
                    match (l, r) {
                        (Deger::Sayi(_), Deger::Sayi(0.0)) => {
                            self.hata_firlat("Sıfıra göre kalan hesaplanamaz".to_string());
                        }
                        (Deger::Sayi(a), Deger::Sayi(b)) => self.stack.push(Deger::Sayi(a % b)),
                        _ => self
                            .hata_firlat("Kalan işlemi yalnızca sayılarda desteklenir".to_string()),
                    }
                }
                OpCode::Greater => {
                    let r = self.stack.pop().unwrap_or(Deger::Bos);
                    let l = self.stack.pop().unwrap_or(Deger::Bos);
                    match (l, r) {
                        (Deger::Sayi(a), Deger::Sayi(b)) => {
                            self.stack.push(Deger::Sayi(if a > b { 1.0 } else { 0.0 }))
                        }
                        _ => self.hata_firlat(
                            "Büyüktür karşılaştırması sadece sayılarda desteklenir".to_string(),
                        ),
                    }
                }
                OpCode::Less => {
                    let r = self.stack.pop().unwrap_or(Deger::Bos);
                    let l = self.stack.pop().unwrap_or(Deger::Bos);
                    match (l, r) {
                        (Deger::Sayi(a), Deger::Sayi(b)) => {
                            self.stack.push(Deger::Sayi(if a < b { 1.0 } else { 0.0 }))
                        }
                        _ => self.hata_firlat(
                            "Küçüktür karşılaştırması sadece sayılarda desteklenir".to_string(),
                        ),
                    }
                }
                OpCode::LessOrEqual => {
                    let r = self.stack.pop().unwrap_or(Deger::Bos);
                    let l = self.stack.pop().unwrap_or(Deger::Bos);
                    match (l, r) {
                        (Deger::Sayi(a), Deger::Sayi(b)) => {
                            self.stack.push(Deger::Sayi(if a <= b { 1.0 } else { 0.0 }))
                        }
                        _ => self.hata_firlat(
                            "Küçük eşittir karşılaştırması sadece sayılarda desteklenir"
                                .to_string(),
                        ),
                    }
                }
                OpCode::GreaterOrEqual => {
                    let r = self.stack.pop().unwrap_or(Deger::Bos);
                    let l = self.stack.pop().unwrap_or(Deger::Bos);
                    match (l, r) {
                        (Deger::Sayi(a), Deger::Sayi(b)) => {
                            self.stack.push(Deger::Sayi(if a >= b { 1.0 } else { 0.0 }))
                        }
                        _ => self.hata_firlat(
                            "Büyük eşittir karşılaştırması sadece sayılarda desteklenir"
                                .to_string(),
                        ),
                    }
                }
                OpCode::Equal => {
                    let r = self.stack.pop().unwrap_or(Deger::Bos);
                    let l = self.stack.pop().unwrap_or(Deger::Bos);
                    self.stack.push(Deger::Sayi(if l == r { 1.0 } else { 0.0 }));
                }
                OpCode::NotEqual => {
                    let r = self.stack.pop().unwrap_or(Deger::Bos);
                    let l = self.stack.pop().unwrap_or(Deger::Bos);
                    self.stack.push(Deger::Sayi(if l != r { 1.0 } else { 0.0 }));
                }
                OpCode::And => {
                    let r = self.stack.pop().unwrap_or(Deger::Bos);
                    let l = self.stack.pop().unwrap_or(Deger::Bos);
                    let result = self.is_truthy(l) && self.is_truthy(r);
                    self.stack.push(Deger::Sayi(if result { 1.0 } else { 0.0 }));
                }
                OpCode::Or => {
                    let r = self.stack.pop().unwrap_or(Deger::Bos);
                    let l = self.stack.pop().unwrap_or(Deger::Bos);
                    let result = self.is_truthy(l) || self.is_truthy(r);
                    self.stack.push(Deger::Sayi(if result { 1.0 } else { 0.0 }));
                }
                OpCode::Not => {
                    let value = self.stack.pop().unwrap_or(Deger::Bos);
                    let result = !self.is_truthy(value);
                    self.stack.push(Deger::Sayi(if result { 1.0 } else { 0.0 }));
                }
                OpCode::Length => {
                    let value = self.stack.pop().unwrap_or(Deger::Bos);
                    match value {
                        Deger::Metin(s) => self.stack.push(Deger::Sayi(s.chars().count() as f64)),
                        Deger::Liste(items) => {
                            self.stack.push(Deger::Sayi(items.borrow().len() as f64))
                        }
                        Deger::Sozluk(items) => {
                            self.stack.push(Deger::Sayi(items.borrow().len() as f64))
                        }
                        other => self.hata_firlat(format!("{} değerinin uzunluğu alınamaz", other)),
                    }
                }
                OpCode::Call(arg_len) => {
                    if self.call_depth >= 50 {
                        self.hata_firlat("Azami özyineleme derinliği aşıldı".to_string());
                    } else {
                        let callable = self.stack.pop().unwrap_or(Deger::Bos);
                        let mut args = Vec::with_capacity(*arg_len);
                        for _ in 0..*arg_len {
                            args.push(self.stack.pop().unwrap_or(Deger::Bos));
                        }
                        args.reverse();

                        match callable {
                            Deger::DahiliFonksiyon(f) => {
                                self.stack.push(f(args));
                            }
                            Deger::Fonksiyon {
                                parametreler,
                                govde,
                            } => {
                                // Fonksiyon gövdeleri AST olarak saklanır. Aynı AST semantiğini
                                // kullanan yorumlayıcı, yerel kapsamları ve özyinelemeyi yönetir.
                                // Bu sayede çağrı parametreleri global değişkenleri kirletmez.
                                let mut interp = crate::interpreter::Yorumlayici::new();
                                interp.global_degiskenler = self.globals.clone();
                                interp.call_depth = self.call_depth;
                                if let Some(buffer) = &self.output_buffer {
                                    interp = interp.with_output_buffer(buffer.clone());
                                }
                                let ret = interp.fonksiyon_cagrisi(
                                    Deger::Fonksiyon {
                                        parametreler,
                                        govde,
                                    },
                                    args,
                                );
                                self.globals = interp.global_degiskenler;
                                match ret {
                                    Deger::Hata(message) => self.hata_firlat(message),
                                    value => self.stack.push(value),
                                }
                            }
                            other => {
                                self.hata_firlat(format!("Çağrılamayan değer: {}", other));
                            }
                        }
                    }
                }
                OpCode::Print => {
                    let val = self.stack.pop().unwrap_or(Deger::Bos);
                    self.satir_yazdir(&val.to_string());
                }
                OpCode::Jump(addr) => {
                    self.ip = *addr;
                }
                OpCode::JumpIfFalse(addr) => {
                    let val = self.stack.pop().unwrap_or(Deger::Bos);
                    if !self.is_truthy(val) {
                        self.ip = *addr;
                    }
                }
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::Return => break,
                OpCode::Bos => self.stack.push(Deger::Bos),
                OpCode::MakeList(len) => {
                    let mut list = Vec::with_capacity(*len);
                    for _ in 0..*len {
                        list.push(self.stack.pop().unwrap_or(Deger::Bos));
                    }
                    list.reverse();
                    self.stack
                        .push(Deger::Liste(std::rc::Rc::new(std::cell::RefCell::new(
                            list,
                        ))));
                }
                OpCode::ListAccess => {
                    let index = self.stack.pop().unwrap_or(Deger::Bos);
                    let list = self.stack.pop().unwrap_or(Deger::Bos);
                    match (list, index) {
                        (Deger::Liste(items), Deger::Sayi(i)) => {
                            let idx = i as isize;
                            if idx < 0 || (idx as usize) >= items.borrow().len() {
                                self.hata_firlat(
                                    "Liste erişiminde indeks sınır dışında".to_string(),
                                );
                            } else {
                                self.stack.push(items.borrow()[idx as usize].clone());
                            }
                        }
                        (Deger::Sozluk(map), Deger::Metin(k)) => {
                            if let Some(v) = map.borrow().get(&k) {
                                self.stack.push(v.clone());
                            } else {
                                self.stack.push(Deger::Bos);
                            }
                        }
                        (Deger::Nesne { alanlar, .. }, Deger::Metin(k)) => {
                            if let Some(v) = alanlar.borrow().get(&k) {
                                self.stack.push(v.clone());
                            } else {
                                self.stack.push(Deger::Bos);
                            }
                        }
                        _ => self.hata_firlat(
                            "Erişim için liste/sözlük ve geçerli indeks/anahtar gerekir"
                                .to_string(),
                        ),
                    }
                }
                OpCode::MakeMap(len) => {
                    let mut map = HashMap::new();
                    for _ in 0..*len {
                        let val = self.stack.pop().unwrap_or(Deger::Bos);
                        let key = self.stack.pop().unwrap_or(Deger::Bos);
                        if let Deger::Metin(k) = key {
                            map.insert(k, val);
                        }
                    }
                    self.stack
                        .push(Deger::Sozluk(std::rc::Rc::new(std::cell::RefCell::new(
                            map,
                        ))));
                }
                OpCode::TryBlockStart(addr) => {
                    self.error_stack.push(*addr);
                }
                OpCode::TryBlockEnd => {
                    self.error_stack.pop();
                }
                OpCode::Await => {
                    let val = self.stack.pop().unwrap_or(Deger::Bos);
                    match val {
                        Deger::GorevId(id) => {
                            let out = self.yaprak.await_task(id);
                            self.stack.push(out);
                        }
                        other => {
                            self.hata_firlat(format!("bekle: await edilemez değer: {}", other));
                        }
                    }
                }
                OpCode::CallFFI {
                    lib_ad,
                    fn_ad,
                    arg_len,
                } => {
                    let mut args = Vec::with_capacity(*arg_len);
                    for _ in 0..*arg_len {
                        args.push(self.stack.pop().unwrap_or(Deger::Bos));
                    }
                    args.reverse();
                    if let Ok(mgr) = crate::ffi::FFI_YONETICI.lock() {
                        match mgr.cagir_esnek(lib_ad, fn_ad, args) {
                            Ok(res) => self.stack.push(res),
                            Err(e) => self.hata_firlat(e),
                        }
                    }
                }
                OpCode::MakeFunction { name, params, body } => {
                    self.globals.insert(
                        name.clone(),
                        Deger::Fonksiyon {
                            parametreler: params.clone(),
                            govde: body.clone(),
                        },
                    );
                }
            }
        }
    }

    fn hata_firlat(&mut self, msg: String) {
        if let Some(handler_addr) = self.error_stack.pop() {
            self.ip = handler_addr;
            self.stack.push(Deger::Hata(msg));
        } else {
            self.runtime_error = Some(msg);
            self.ip = self.program.instructions.len();
        }
    }

    fn satir_yazdir(&self, content: &str) {
        if let Some(buffer) = &self.output_buffer {
            let mut buffer = buffer.borrow_mut();
            buffer.push_str(content);
            buffer.push('\n');
        } else {
            println!("{}", content);
        }
    }

    fn is_truthy(&self, d: Deger) -> bool {
        match d {
            Deger::Sayi(n) => n != 0.0,
            Deger::Metin(s) => !s.is_empty(),
            Deger::Liste(l) => !l.borrow().is_empty(),
            Deger::Bos => false,
            _ => true,
        }
    }
}
