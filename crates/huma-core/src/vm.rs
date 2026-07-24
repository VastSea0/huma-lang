use crate::bytecode::{OpCode, Constant, Program};
use crate::value::Deger;
use std::collections::HashMap;
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
                OpCode::LoadVar(ad) => {
                    let val = self.globals.get(ad).cloned().unwrap_or(Deger::Bos);
                    self.stack.push(val);
                }
                OpCode::StoreVar(ad) => {
                    let val = self.stack.pop().unwrap_or(Deger::Bos);
                    if self.globals.contains_key(ad) {
                        self.globals.insert(ad.clone(), val);
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
                        (Deger::Metin(a), b) => self.stack.push(Deger::Metin(format!("{}{}", a, b))),
                        (a, Deger::Metin(b)) => self.stack.push(Deger::Metin(format!("{}{}", a, b))),
                        _ => self.stack.push(Deger::Bos),
                    }
                }
                OpCode::Sub => {
                    let r = self.stack.pop().unwrap_or(Deger::Bos);
                    let l = self.stack.pop().unwrap_or(Deger::Bos);
                    if let (Deger::Sayi(a), Deger::Sayi(b)) = (l, r) {
                        self.stack.push(Deger::Sayi(a - b));
                    }
                }
                OpCode::Mul => {
                    let r = self.stack.pop().unwrap_or(Deger::Bos);
                    let l = self.stack.pop().unwrap_or(Deger::Bos);
                    if let (Deger::Sayi(a), Deger::Sayi(b)) = (l, r) {
                        self.stack.push(Deger::Sayi(a * b));
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
                    }
                }
                OpCode::Greater => {
                    let r = self.stack.pop().unwrap_or(Deger::Bos);
                    let l = self.stack.pop().unwrap_or(Deger::Bos);
                    match (l, r) {
                        (Deger::Sayi(a), Deger::Sayi(b)) => {
                            self.stack.push(Deger::Sayi(if a > b { 1.0 } else { 0.0 }))
                        }
                        _ => self.hata_firlat("Büyüktür karşılaştırması sadece sayılarda desteklenir".to_string()),
                    }
                }
                OpCode::Less => {
                    let r = self.stack.pop().unwrap_or(Deger::Bos);
                    let l = self.stack.pop().unwrap_or(Deger::Bos);
                    match (l, r) {
                        (Deger::Sayi(a), Deger::Sayi(b)) => {
                            self.stack.push(Deger::Sayi(if a < b { 1.0 } else { 0.0 }))
                        }
                        _ => self.hata_firlat("Küçüktür karşılaştırması sadece sayılarda desteklenir".to_string()),
                    }
                }
                OpCode::LessOrEqual => {
                    let r = self.stack.pop().unwrap_or(Deger::Bos);
                    let l = self.stack.pop().unwrap_or(Deger::Bos);
                    match (l, r) {
                        (Deger::Sayi(a), Deger::Sayi(b)) => {
                            self.stack.push(Deger::Sayi(if a <= b { 1.0 } else { 0.0 }))
                        }
                        _ => self.hata_firlat("Küçük eşittir karşılaştırması sadece sayılarda desteklenir".to_string()),
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
                            Deger::Fonksiyon { parametreler, govde } => {
                                let mut sub_compiler = crate::compiler::Derleyici::new();
                                let sub_prog = sub_compiler.derle(govde);
                                let mut sub_vm = VM::new(sub_prog);
                                sub_vm.call_depth = self.call_depth + 1;
                                sub_vm.globals.extend(self.globals.clone());
                                for (i, p) in parametreler.iter().enumerate() {
                                    if i < args.len() {
                                        sub_vm.globals.insert(p.clone(), args[i].clone());
                                    }
                                }
                                sub_vm.run();
                                self.globals.extend(sub_vm.globals);
                                let ret = sub_vm.stack.pop().unwrap_or(Deger::Bos);
                                self.stack.push(ret);
                            }
                            other => {
                                self.hata_firlat(format!("Çağrılamayan değer: {}", other));
                            }
                        }
                    }
                }
                OpCode::Print => {
                    let val = self.stack.pop().unwrap_or(Deger::Bos);
                    println!("{}", val);
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
                OpCode::Pop => { self.stack.pop(); }
                OpCode::Return => break,
                OpCode::Bos => self.stack.push(Deger::Bos),
                OpCode::MakeList(len) => {
                    let mut list = Vec::with_capacity(*len);
                    for _ in 0..*len {
                        list.push(self.stack.pop().unwrap_or(Deger::Bos));
                    }
                    list.reverse();
                    self.stack
                        .push(Deger::Liste(std::rc::Rc::new(std::cell::RefCell::new(list))));
                }
                OpCode::ListAccess => {
                    let index = self.stack.pop().unwrap_or(Deger::Bos);
                    let list = self.stack.pop().unwrap_or(Deger::Bos);
                    match (list, index) {
                        (Deger::Liste(items), Deger::Sayi(i)) => {
                            let idx = i as isize;
                            if idx < 0 || (idx as usize) >= items.borrow().len() {
                                self.hata_firlat("Liste erişiminde indeks sınır dışında".to_string());
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
                        _ => self.hata_firlat("Erişim için liste/sözlük ve geçerli indeks/anahtar gerekir".to_string()),
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
                    self.stack.push(Deger::Sozluk(std::rc::Rc::new(std::cell::RefCell::new(map))));
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
                            self.stack.push(Deger::Hata(format!("bekle: await edilemez değer: {}", other)));
                        }
                    }
                }
                OpCode::CallFFI { lib_ad, fn_ad, arg_len } => {
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
                    self.globals.insert(name.clone(), Deger::Fonksiyon { parametreler: params.clone(), govde: body.clone() });
                }
            }
        }
    }

    fn hata_firlat(&mut self, msg: String) {
        if let Some(handler_addr) = self.error_stack.pop() {
            self.ip = handler_addr;
            self.stack.push(Deger::Hata(msg));
        } else {
            eprintln!("[Hüma Hatası] {}", msg);
            self.ip = self.program.instructions.len();
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
