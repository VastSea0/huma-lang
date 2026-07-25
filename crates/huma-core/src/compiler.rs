use crate::ast::{Ifade, Komut};
use crate::bytecode::{Constant, OpCode, Program};
use crate::token::Token;

pub struct Derleyici {
    constants: Vec<Constant>,
    instructions: Vec<OpCode>,
    errors: Vec<String>,
}

impl Default for Derleyici {
    fn default() -> Self {
        Self::new()
    }
}

impl Derleyici {
    pub fn new() -> Self {
        Self {
            constants: Vec::new(),
            instructions: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn derle(&mut self, program: Vec<Komut>) -> Program {
        self.constants.clear();
        self.instructions.clear();
        self.errors.clear();
        for komut in program {
            self.komut_derle(komut);
        }
        Program {
            constants: self.constants.clone(),
            instructions: self.instructions.clone(),
        }
    }

    pub fn derle_kontrollu(&mut self, program: Vec<Komut>) -> Result<Program, String> {
        let compiled = self.derle(program);
        if self.errors.is_empty() {
            Ok(compiled)
        } else {
            Err(self.errors.join("\n"))
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
                if operator == Token::Ve {
                    self.ifade_derle(*sol);
                    let false_jump = self.instructions.len();
                    self.instructions.push(OpCode::JumpIfFalse(0));
                    self.ifade_derle(*sag);
                    self.instructions.push(OpCode::Not);
                    self.instructions.push(OpCode::Not);
                    let end_jump = self.instructions.len();
                    self.instructions.push(OpCode::Jump(0));
                    let false_branch = self.instructions.len();
                    let false_idx = self.constant_ekle(Constant::Sayi(0.0));
                    self.instructions.push(OpCode::PushConstant(false_idx));
                    let end = self.instructions.len();
                    self.instructions[false_jump] = OpCode::JumpIfFalse(false_branch);
                    self.instructions[end_jump] = OpCode::Jump(end);
                    return;
                }
                if operator == Token::Veya {
                    self.ifade_derle(*sol);
                    let right_jump = self.instructions.len();
                    self.instructions.push(OpCode::JumpIfFalse(0));
                    let true_idx = self.constant_ekle(Constant::Sayi(1.0));
                    self.instructions.push(OpCode::PushConstant(true_idx));
                    let end_jump = self.instructions.len();
                    self.instructions.push(OpCode::Jump(0));
                    let right_branch = self.instructions.len();
                    self.ifade_derle(*sag);
                    self.instructions.push(OpCode::Not);
                    self.instructions.push(OpCode::Not);
                    let end = self.instructions.len();
                    self.instructions[right_jump] = OpCode::JumpIfFalse(right_branch);
                    self.instructions[end_jump] = OpCode::Jump(end);
                    return;
                }

                self.ifade_derle(*sol);
                self.ifade_derle(*sag);
                match operator {
                    Token::Arti => self.instructions.push(OpCode::Add),
                    Token::Eksi => self.instructions.push(OpCode::Sub),
                    Token::Carpi => self.instructions.push(OpCode::Mul),
                    Token::Bolnu => self.instructions.push(OpCode::Div),
                    Token::Buyuktur => self.instructions.push(OpCode::Greater),
                    Token::Kucuktur => self.instructions.push(OpCode::Less),
                    Token::BuyukEsit => self.instructions.push(OpCode::GreaterOrEqual),
                    Token::KucukEsit => self.instructions.push(OpCode::LessOrEqual),
                    Token::EsitEsittir | Token::Esittir => self.instructions.push(OpCode::Equal),
                    Token::EsitDegil => self.instructions.push(OpCode::NotEqual),
                    Token::Mod => self.instructions.push(OpCode::Mod),
                    other => {
                        self.errors
                            .push(format!("Desteklenmeyen ikili işlem operatörü: {}", other));
                    }
                }
            }
            Ifade::Liste(el) => {
                let len = el.len();
                for e in el {
                    self.ifade_derle(e);
                }
                self.instructions.push(OpCode::MakeList(len));
            }
            Ifade::Cagri {
                fonksiyon,
                argumanlar,
                ..
            } => {
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
            Ifade::MantıksalDegil(e) => {
                self.ifade_derle(*e);
                self.instructions.push(OpCode::Not);
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
            Ifade::ListeErisim { liste, indeks } => {
                self.ifade_derle(*liste);
                self.ifade_derle(*indeks);
                self.instructions.push(OpCode::ListAccess);
            }
            Ifade::Uzunluk(deger) => {
                self.ifade_derle(*deger);
                self.instructions.push(OpCode::Length);
            }
            Ifade::NesneErisim { nesne, ozellik } => {
                self.ifade_derle(*nesne);
                let idx = self.constant_ekle(Constant::Metin(ozellik));
                self.instructions.push(OpCode::PushConstant(idx));
                self.instructions.push(OpCode::ListAccess);
            }
            unsupported => {
                self.errors.push(format!(
                    "Bytecode derleyici tarafından henüz desteklenmeyen ifade: {:?}",
                    unsupported
                ));
            }
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
                if let Ifade::IkiliIslem {
                    sol,
                    operator: Token::Esittir,
                    sag,
                } = ifade
                {
                    self.ifade_derle(*sag);
                    match *sol {
                        Ifade::Degisken(ad) => self.instructions.push(OpCode::StoreVar(ad)),
                        other => self.errors.push(format!(
                            "Bytecode derleyici bu atama hedefini desteklemiyor: {:?}",
                            other
                        )),
                    }
                } else {
                    self.ifade_derle(ifade);
                    self.instructions.push(OpCode::Pop);
                }
            }
            Komut::EgerKomutu {
                kosul,
                govde,
                degilse_govde,
            } => {
                self.ifade_derle(kosul);
                let jump_if_false_idx = self.instructions.len();
                self.instructions.push(OpCode::JumpIfFalse(0));

                for k in govde {
                    self.komut_derle(k);
                }

                if let Some(else_b) = degilse_govde {
                    let jump_idx = self.instructions.len();
                    self.instructions.push(OpCode::Jump(0));

                    let else_start = self.instructions.len();
                    self.instructions[jump_if_false_idx] = OpCode::JumpIfFalse(else_start);

                    for k in else_b {
                        self.komut_derle(k);
                    }
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

                for k in govde {
                    self.komut_derle(k);
                }

                self.instructions.push(OpCode::Jump(start_idx));
                let end_idx = self.instructions.len();
                self.instructions[jump_if_false_idx] = OpCode::JumpIfFalse(end_idx);
            }
            Komut::DondurKomutu(ifade) => {
                self.ifade_derle(ifade);
                self.instructions.push(OpCode::Return);
            }
            Komut::AralikDongusu {
                degisken,
                baslangic,
                bitis,
                govde,
            } => {
                // Initialize loop variable
                self.ifade_derle(baslangic);
                self.instructions.push(OpCode::DefineVar(degisken.clone()));

                let start_idx = self.instructions.len();
                // Condition check: var <= bitis
                self.instructions.push(OpCode::LoadVar(degisken.clone()));
                self.ifade_derle(bitis);
                self.instructions.push(OpCode::LessOrEqual);

                let jump_if_false_idx = self.instructions.len();
                self.instructions.push(OpCode::JumpIfFalse(0));

                for k in govde {
                    self.komut_derle(k);
                }

                // Increment var = var + 1
                self.instructions.push(OpCode::LoadVar(degisken.clone()));
                let one_idx = self.constant_ekle(Constant::Sayi(1.0));
                self.instructions.push(OpCode::PushConstant(one_idx));
                self.instructions.push(OpCode::Add);
                self.instructions.push(OpCode::StoreVar(degisken));

                self.instructions.push(OpCode::Jump(start_idx));
                let end_idx = self.instructions.len();
                self.instructions[jump_if_false_idx] = OpCode::JumpIfFalse(end_idx);
            }
            Komut::FonksiyonTanimla {
                ad,
                parametreler,
                govde,
            } => {
                self.instructions.push(OpCode::MakeFunction {
                    name: ad,
                    params: parametreler,
                    body: govde,
                });
            }
            Komut::DeneKomutu {
                dene_govde,
                hata_degisken,
                hata_govde,
            } => {
                let try_start_idx = self.instructions.len();
                self.instructions.push(OpCode::TryBlockStart(0));

                for k in dene_govde {
                    self.komut_derle(k);
                }

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

                for k in hata_govde {
                    self.komut_derle(k);
                }

                let end_idx = self.instructions.len();
                self.instructions[jump_after_idx] = OpCode::Jump(end_idx);
            }
            unsupported => {
                self.errors.push(format!(
                    "Bytecode derleyici tarafından henüz desteklenmeyen komut: {:?}",
                    unsupported
                ));
            }
        }
    }
}
