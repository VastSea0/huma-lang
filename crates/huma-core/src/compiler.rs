use crate::ast::{Ifade, Komut};
use crate::bytecode::{OpCode, Constant, Program};
use crate::token::Token;

pub struct Derleyici {
    constants: Vec<Constant>,
    instructions: Vec<OpCode>,
}

impl Derleyici {
    pub fn new() -> Self {
        Self {
            constants: Vec::new(),
            instructions: Vec::new(),
        }
    }

    pub fn derle(&mut self, program: Vec<Komut>) -> Program {
        for komut in program {
            self.komut_derle(komut);
        }
        Program {
            constants: self.constants.clone(),
            instructions: self.instructions.clone(),
        }
    }

    fn constant_ekle(&mut self, c: Constant) -> usize {
        if let Some(pos) = self.constants.iter().position(|x| match (x, &c) {
            (Constant::Sayi(a), Constant::Sayi(b)) => a == b,
            (Constant::Metin(a), Constant::Metin(b)) => a == b,
            _ => false,
        }) {
            pos
        } else {
            self.constants.push(c);
            self.constants.len() - 1
        }
    }

    fn ifade_derle(&mut self, ifade: Ifade) {
        match ifade {
            Ifade::Sayi(n) => {
                let idx = self.constant_ekle(Constant::Sayi(n));
                self.instructions.push(OpCode::PushConstant(idx));
            }
            Ifade::Metin(s) => {
                let idx = self.constant_ekle(Constant::Metin(s));
                self.instructions.push(OpCode::PushConstant(idx));
            }
            Ifade::Degisken(ad) => {
                self.instructions.push(OpCode::LoadVar(ad));
            }
            Ifade::Dogru => {
                let idx = self.constant_ekle(Constant::Sayi(1.0));
                self.instructions.push(OpCode::PushConstant(idx));
            }
            Ifade::Yanlis => {
                let idx = self.constant_ekle(Constant::Sayi(0.0));
                self.instructions.push(OpCode::PushConstant(idx));
            }
            Ifade::IkiliIslem { sol, operator, sag } => {
                self.ifade_derle(*sol);
                self.ifade_derle(*sag);
                match operator {
                    Token::Arti => self.instructions.push(OpCode::Add),
                    Token::Eksi => self.instructions.push(OpCode::Sub),
                    Token::Carpi => self.instructions.push(OpCode::Mul),
                    Token::Bolnu => self.instructions.push(OpCode::Div),
                    Token::Buyuktur => self.instructions.push(OpCode::Greater),
                    Token::Kucuktur => self.instructions.push(OpCode::Less),
                    Token::EsitEsittir | Token::Esittir => self.instructions.push(OpCode::Equal),
                    Token::EsitDegil => self.instructions.push(OpCode::NotEqual),
                    _ => {}
                }
            }
            Ifade::Liste(el) => {
                let len = el.len();
                for e in el {
                    self.ifade_derle(e);
                }
                self.instructions.push(OpCode::MakeList(len));
            }
            Ifade::Cagri { fonksiyon, argumanlar } => {
                let arg_len = argumanlar.len();
                for arg in argumanlar {
                    self.ifade_derle(arg);
                }
                self.ifade_derle(*fonksiyon);
                self.instructions.push(OpCode::Call(arg_len));
            }
            Ifade::Bekle(e) => {
                self.ifade_derle(*e);
                self.instructions.push(OpCode::Await);
            }
            Ifade::Bos => self.instructions.push(OpCode::Bos),
            Ifade::Sozluk(ciftler) => {
                let len = ciftler.len();
                for (k, v) in ciftler {
                    self.ifade_derle(k);
                    self.ifade_derle(v);
                }
                self.instructions.push(OpCode::MakeMap(len));
            }
            _ => {}
        }
    }

    fn komut_derle(&mut self, komut: Komut) {
        match komut {
            Komut::YazdirKomutu(ifade) => {
                self.ifade_derle(ifade);
                self.instructions.push(OpCode::Print);
            }
            Komut::DegiskenTanimla { ad, deger } => {
                self.ifade_derle(deger);
                self.instructions.push(OpCode::DefineVar(ad));
            }
            Komut::Atama { ad, deger } => {
                self.ifade_derle(deger);
                self.instructions.push(OpCode::StoreVar(ad));
            }
            Komut::IfadeKomutu(ifade) => {
                if let Ifade::IkiliIslem { sol, operator: Token::Esittir, sag } = ifade {
                    self.ifade_derle(*sag);
                    if let Ifade::Degisken(ad) = *sol {
                        self.instructions.push(OpCode::StoreVar(ad));
                    }
                } else {
                    self.ifade_derle(ifade);
                    self.instructions.push(OpCode::Pop);
                }
            }
            Komut::EgerKomutu { kosul, govde, degilse_govde } => {
                self.ifade_derle(kosul);
                let jump_if_false_idx = self.instructions.len();
                self.instructions.push(OpCode::JumpIfFalse(0));

                for k in govde { self.komut_derle(k); }
                
                if let Some(else_b) = degilse_govde {
                    let jump_idx = self.instructions.len();
                    self.instructions.push(OpCode::Jump(0));
                    
                    let else_start = self.instructions.len();
                    self.instructions[jump_if_false_idx] = OpCode::JumpIfFalse(else_start);
                    
                    for k in else_b { self.komut_derle(k); }
                    let end_idx = self.instructions.len();
                    self.instructions[jump_idx] = OpCode::Jump(end_idx);
                } else {
                    let end_idx = self.instructions.len();
                    self.instructions[jump_if_false_idx] = OpCode::JumpIfFalse(end_idx);
                }
            }
            Komut::DonguKomutu { kosul, govde } => {
                let start_idx = self.instructions.len();
                self.ifade_derle(kosul);
                let jump_if_false_idx = self.instructions.len();
                self.instructions.push(OpCode::JumpIfFalse(0));

                for k in govde { self.komut_derle(k); }
                
                self.instructions.push(OpCode::Jump(start_idx));
                let end_idx = self.instructions.len();
                self.instructions[jump_if_false_idx] = OpCode::JumpIfFalse(end_idx);
            }
            Komut::DondurKomutu(ifade) => {
                self.ifade_derle(ifade);
                self.instructions.push(OpCode::Return);
            }
            Komut::DeneKomutu { dene_govde, hata_degisken, hata_govde } => {
                let try_start_idx = self.instructions.len();
                self.instructions.push(OpCode::TryBlockStart(0));

                for k in dene_govde { self.komut_derle(k); }
                
                self.instructions.push(OpCode::TryBlockEnd);
                
                let jump_after_idx = self.instructions.len();
                self.instructions.push(OpCode::Jump(0));
                
                let catch_start = self.instructions.len();
                self.instructions[try_start_idx] = OpCode::TryBlockStart(catch_start);
                
                if let Some(ad) = hata_degisken {
                    self.instructions.push(OpCode::DefineVar(ad));
                } else {
                    self.instructions.push(OpCode::Pop);
                }
                
                for k in hata_govde { self.komut_derle(k); }
                
                let end_idx = self.instructions.len();
                self.instructions[jump_after_idx] = OpCode::Jump(end_idx);
            }
            _ => {
                // Placeholder for unimplemented commands in bytecode
            }
        }
    }
}
