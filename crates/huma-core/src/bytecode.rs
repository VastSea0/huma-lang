use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpCode {
    PushConstant(usize),
    LoadVar(String),
    StoreVar(String),
    DefineVar(String),
    Add,
    Sub,
    Mul,
    Div,
    Greater,
    Less,
    LessOrEqual,
    Equal,
    NotEqual,
    Jump(usize),
    JumpIfFalse(usize),
    Call(usize),
    Return,
    Print,
    MakeList(usize),
    ListAccess,
    MakeMap(usize),
    TryBlockStart(usize),
    TryBlockEnd,
    Await,
    Pop,
    Bos,
    CallFFI { lib_ad: String, fn_ad: String, arg_len: usize },
    MakeFunction { name: String, params: Vec<String>, body: Vec<crate::ast::Komut> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constant {
    Sayi(f64),
    Metin(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub constants: Vec<Constant>,
    pub instructions: Vec<OpCode>,
}
