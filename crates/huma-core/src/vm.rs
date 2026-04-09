use crate::bytecode::{OpCode, Constant, Program};
use crate::value::Deger;
use std::collections::HashMap;

pub struct VM {
    stack: Vec<Deger>,
    globals: HashMap<String, Deger>,
    program: Program,
    ip: usize,
    error_stack: Vec<usize>,
}

impl VM {
    pub fn new(program: Program) -> Self {
        Self {
            stack: Vec::new(),
            globals: HashMap::new(),
            program,
            ip: 0,
            error_stack: Vec::new(),
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
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    if let (Deger::Sayi(a), Deger::Sayi(b)) = (l, r) {
                        self.stack.push(Deger::Sayi(a + b));
                    }
                }
                OpCode::Sub => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    if let (Deger::Sayi(a), Deger::Sayi(b)) = (l, r) {
                        self.stack.push(Deger::Sayi(a - b));
                    }
                }
                OpCode::Mul => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    if let (Deger::Sayi(a), Deger::Sayi(b)) = (l, r) {
                        self.stack.push(Deger::Sayi(a * b));
                    }
                }
                OpCode::Div => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    if let (Deger::Sayi(a), Deger::Sayi(b)) = (l, r) {
                        if b == 0.0 {
                            self.hata_firlat("Sıfıra bölme hatası".to_string());
                        } else {
                            self.stack.push(Deger::Sayi(a / b));
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
                OpCode::MakeMap(len) => {
                    let mut map = HashMap::new();
                    for _ in 0..*len {
                        let val = self.stack.pop().unwrap();
                        let key = self.stack.pop().unwrap();
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
                _ => {}
            }
        }
    }

    fn hata_firlat(&mut self, msg: String) {
        if let Some(handler_addr) = self.error_stack.pop() {
            self.ip = handler_addr;
            self.stack.push(Deger::Hata(msg));
        } else {
            panic!("Çalışma Zamanı Hatası: {}", msg);
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
