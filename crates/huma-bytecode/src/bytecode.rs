use crate::error::{HumaError, HumaResult, SourceSpan};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::rc::Rc;

pub const MAX_CONSTANTS: usize = 1_000_000;
pub const MAX_INSTRUCTIONS: usize = 10_000_000;
pub const MAX_COLLECTION_LITERAL_ITEMS: usize = 1_000_000;
pub const MAX_CALL_ARGUMENTS: usize = 65_535;
pub const MAX_STACK_HEIGHT: usize = 1_000_000;
pub const MAX_FUNCTIONS: usize = 100_000;
pub const MAX_SYMBOL_BYTES: usize = 1_048_576;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpCode {
    PushConstant(usize),
    LoadVar(Rc<str>),
    StoreVar(Rc<str>),
    DefineVar(Rc<str>),
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
    Equal,
    NotEqual,
    And,
    Or,
    Not,
    Length,
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
    CallFFI {
        lib_ad: String,
        fn_ad: String,
        signature: String,
        arg_len: usize,
    },
    DefineFunction {
        name: Rc<str>,
        function_index: usize,
    },
    MakeClosure(usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constant {
    Sayi(f64),
    Metin(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionPrototype {
    pub params: Vec<String>,
    pub instructions: Vec<OpCode>,
    pub instruction_spans: Vec<Option<SourceSpan>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub constants: Vec<Constant>,
    pub functions: Vec<FunctionPrototype>,
    pub instructions: Vec<OpCode>,
    pub instruction_spans: Vec<Option<SourceSpan>>,
}

fn serialization_error(message: impl Into<String>) -> HumaError {
    HumaError::SerializationError(message.into())
}

fn stack_effect(opcode: &OpCode) -> HumaResult<(usize, usize)> {
    let effect = match opcode {
        OpCode::PushConstant(_) | OpCode::LoadVar(_) | OpCode::Bos => (0, 1),
        OpCode::StoreVar(_) | OpCode::DefineVar(_) | OpCode::Print | OpCode::Pop => (1, 0),
        OpCode::Add
        | OpCode::Sub
        | OpCode::Mul
        | OpCode::Div
        | OpCode::Mod
        | OpCode::Greater
        | OpCode::GreaterOrEqual
        | OpCode::Less
        | OpCode::LessOrEqual
        | OpCode::Equal
        | OpCode::NotEqual
        | OpCode::And
        | OpCode::Or
        | OpCode::ListAccess => (2, 1),
        OpCode::Not | OpCode::Length | OpCode::Await => (1, 1),
        OpCode::Jump(_)
        | OpCode::TryBlockStart(_)
        | OpCode::TryBlockEnd
        | OpCode::DefineFunction { .. } => (0, 0),
        OpCode::MakeClosure(_) => (0, 1),
        OpCode::JumpIfFalse(_) => (1, 0),
        OpCode::Call(argument_count) => (
            argument_count
                .checked_add(1)
                .ok_or_else(|| serialization_error("Bytecode çağrı yığın etkisi taştı"))?,
            1,
        ),
        OpCode::Return => (1, 0),
        OpCode::MakeList(item_count) => (*item_count, 1),
        OpCode::MakeMap(item_count) => (
            item_count
                .checked_mul(2)
                .ok_or_else(|| serialization_error("Bytecode sözlük yığın etkisi taştı"))?,
            1,
        ),
        OpCode::CallFFI { arg_len, .. } => (*arg_len, 1),
    };
    Ok(effect)
}

fn enqueue_height(
    queue: &mut VecDeque<(usize, usize)>,
    heights: &mut [Option<usize>],
    target: usize,
    height: usize,
) -> HumaResult<()> {
    if height > MAX_STACK_HEIGHT {
        return Err(serialization_error(format!(
            "Bytecode yığın sınırını aşıyor: {height} > {MAX_STACK_HEIGHT}"
        )));
    }
    match heights[target] {
        Some(previous) if previous != height => Err(serialization_error(format!(
            "Bytecode kontrol akışı {target} konumunda uyumsuz yığın yükseklikleri birleştiriyor: \
             {previous} ve {height}"
        ))),
        Some(_) => Ok(()),
        None => {
            heights[target] = Some(height);
            queue.push_back((target, height));
            Ok(())
        }
    }
}

fn validate_stack(instructions: &[OpCode], context: &str) -> HumaResult<()> {
    let instruction_count = instructions.len();
    let mut heights = vec![None; instruction_count + 1];
    let mut queue = VecDeque::new();
    enqueue_height(&mut queue, &mut heights, 0, 0)?;

    while let Some((index, height)) = queue.pop_front() {
        if index == instruction_count {
            continue;
        }
        let opcode = &instructions[index];
        let (required, pushed) = stack_effect(opcode)?;
        if height < required {
            return Err(serialization_error(format!(
                "{context} komutu {index} yığın eksilmesine yol açıyor: {required} değer \
                 gerekiyor, {height} değer var"
            )));
        }
        let next_height = height - required + pushed;
        if next_height > MAX_STACK_HEIGHT {
            return Err(serialization_error(format!(
                "{context} komutu {index} yığın sınırını aşıyor: {next_height}"
            )));
        }

        match opcode {
            OpCode::Jump(target) => {
                enqueue_height(&mut queue, &mut heights, *target, next_height)?;
            }
            OpCode::JumpIfFalse(target) => {
                enqueue_height(&mut queue, &mut heights, *target, next_height)?;
                enqueue_height(&mut queue, &mut heights, index + 1, next_height)?;
            }
            OpCode::TryBlockStart(handler) => {
                enqueue_height(&mut queue, &mut heights, index + 1, next_height)?;
                let error_height = height
                    .checked_add(1)
                    .ok_or_else(|| serialization_error("Bytecode hata yakalama yığını taştı"))?;
                enqueue_height(&mut queue, &mut heights, *handler, error_height)?;
            }
            OpCode::Return => {}
            _ => {
                enqueue_height(&mut queue, &mut heights, index + 1, next_height)?;
            }
        }
    }

    Ok(())
}

fn enqueue_handlers(
    queue: &mut VecDeque<(usize, Vec<usize>)>,
    states: &mut [Option<Vec<usize>>],
    target: usize,
    handlers: Vec<usize>,
    context: &str,
) -> HumaResult<()> {
    match &states[target] {
        Some(previous) if previous != &handlers => Err(serialization_error(format!(
            "{context} kontrol akışı {target} konumunda uyumsuz hata bloklarını birleştiriyor"
        ))),
        Some(_) => Ok(()),
        None => {
            states[target] = Some(handlers.clone());
            queue.push_back((target, handlers));
            Ok(())
        }
    }
}

fn validate_error_blocks(instructions: &[OpCode], context: &str) -> HumaResult<()> {
    let mut states = vec![None; instructions.len() + 1];
    let mut queue = VecDeque::new();
    enqueue_handlers(&mut queue, &mut states, 0, Vec::new(), context)?;

    while let Some((index, handlers)) = queue.pop_front() {
        if index == instructions.len() {
            if !handlers.is_empty() {
                return Err(serialization_error(format!(
                    "{context} açık bir hata bloğuyla sona eriyor"
                )));
            }
            continue;
        }

        match &instructions[index] {
            OpCode::TryBlockStart(handler) => {
                enqueue_handlers(&mut queue, &mut states, *handler, handlers.clone(), context)?;
                let mut nested = handlers;
                nested.push(*handler);
                enqueue_handlers(&mut queue, &mut states, index + 1, nested, context)?;
            }
            OpCode::TryBlockEnd => {
                let mut outer = handlers;
                if outer.pop().is_none() {
                    return Err(serialization_error(format!(
                        "{context} komutu {index} eşleşmeyen TryBlockEnd içeriyor"
                    )));
                }
                enqueue_handlers(&mut queue, &mut states, index + 1, outer, context)?;
            }
            OpCode::Jump(target) => {
                enqueue_handlers(&mut queue, &mut states, *target, handlers, context)?;
            }
            OpCode::JumpIfFalse(target) => {
                enqueue_handlers(&mut queue, &mut states, *target, handlers.clone(), context)?;
                enqueue_handlers(&mut queue, &mut states, index + 1, handlers, context)?;
            }
            OpCode::Return => {}
            _ => {
                enqueue_handlers(&mut queue, &mut states, index + 1, handlers, context)?;
            }
        }
    }
    Ok(())
}

fn validate_instructions(
    instructions: &[OpCode],
    constant_count: usize,
    function_count: usize,
    context: &str,
) -> HumaResult<()> {
    for (index, instruction) in instructions.iter().enumerate() {
        match instruction {
            OpCode::PushConstant(constant_index) if *constant_index >= constant_count => {
                return Err(serialization_error(format!(
                    "{context} komutu {index} geçersiz sabit indeksi kullanıyor: {constant_index}"
                )));
            }
            OpCode::Jump(target) | OpCode::JumpIfFalse(target) | OpCode::TryBlockStart(target)
                if *target > instructions.len() =>
            {
                return Err(serialization_error(format!(
                    "{context} komutu {index} geçersiz atlama hedefi kullanıyor: {target}"
                )));
            }
            OpCode::MakeList(item_count) | OpCode::MakeMap(item_count)
                if *item_count > MAX_COLLECTION_LITERAL_ITEMS =>
            {
                return Err(serialization_error(format!(
                    "{context} komutu {index} koleksiyon sınırını aşıyor: {item_count}"
                )));
            }
            OpCode::Call(argument_count)
            | OpCode::CallFFI {
                arg_len: argument_count,
                ..
            } if *argument_count > MAX_CALL_ARGUMENTS => {
                return Err(serialization_error(format!(
                    "{context} komutu {index} argüman sınırını aşıyor: {argument_count}"
                )));
            }
            OpCode::DefineFunction {
                name,
                function_index,
            } => {
                if name.is_empty() || name.len() > MAX_SYMBOL_BYTES {
                    return Err(serialization_error(format!(
                        "{context} komutu {index} geçersiz fonksiyon adı içeriyor"
                    )));
                }
                if *function_index >= function_count {
                    return Err(serialization_error(format!(
                        "{context} komutu {index} geçersiz fonksiyon indeksi kullanıyor: \
                         {function_index}"
                    )));
                }
            }
            OpCode::MakeClosure(function_index) if *function_index >= function_count => {
                return Err(serialization_error(format!(
                    "{context} komutu {index} geçersiz closure indeksi kullanıyor: {function_index}"
                )));
            }
            OpCode::LoadVar(name) | OpCode::StoreVar(name) | OpCode::DefineVar(name)
                if name.is_empty() || name.len() > MAX_SYMBOL_BYTES =>
            {
                return Err(serialization_error(format!(
                    "{context} komutu {index} geçersiz değişken adı içeriyor"
                )));
            }
            OpCode::CallFFI {
                lib_ad,
                fn_ad,
                signature,
                arg_len,
            } if lib_ad.is_empty()
                || fn_ad.is_empty()
                || lib_ad.len() > MAX_SYMBOL_BYTES
                || fn_ad.len() > MAX_SYMBOL_BYTES
                || !matches!(
                    (signature.as_str(), *arg_len),
                    ("f64()", 0) | ("f64(f64)", 1) | ("f64(f64,f64)", 2)
                ) =>
            {
                return Err(serialization_error(format!(
                    "{context} komutu {index} geçersiz FFI adı, imzası veya argüman sayısı içeriyor"
                )));
            }
            _ => {}
        }
    }
    validate_stack(instructions, context)?;
    validate_error_blocks(instructions, context)
}

/// Validate every invariant required before a [`Program`] may enter the VM.
pub fn validate_program(program: &Program) -> HumaResult<()> {
    if program.constants.len() > MAX_CONSTANTS {
        return Err(serialization_error(format!(
            "Bytecode sabit havuzu sınırı aşıyor: {} > {}",
            program.constants.len(),
            MAX_CONSTANTS
        )));
    }
    if program.instructions.len() > MAX_INSTRUCTIONS {
        return Err(serialization_error(format!(
            "Bytecode komut sınırı aşıyor: {} > {}",
            program.instructions.len(),
            MAX_INSTRUCTIONS
        )));
    }
    if program.instruction_spans.len() != program.instructions.len() {
        return Err(serialization_error(format!(
            "Bytecode ana program konum tablosu uzunluğu uyuşmuyor: {} != {}",
            program.instruction_spans.len(),
            program.instructions.len()
        )));
    }
    if program.functions.len() > MAX_FUNCTIONS {
        return Err(serialization_error(format!(
            "Bytecode fonksiyon sınırı aşıyor: {} > {}",
            program.functions.len(),
            MAX_FUNCTIONS
        )));
    }

    for (index, constant) in program.constants.iter().enumerate() {
        if let Constant::Sayi(value) = constant {
            if !value.is_finite() {
                return Err(serialization_error(format!(
                    "Bytecode sabiti {index} sonlu bir sayı değil"
                )));
            }
        }
    }

    let total_instructions =
        program
            .functions
            .iter()
            .try_fold(program.instructions.len(), |total, function| {
                total
                    .checked_add(function.instructions.len())
                    .ok_or_else(|| {
                        serialization_error("Bytecode toplam komut sayısı hesaplanırken taştı")
                    })
            })?;
    if total_instructions > MAX_INSTRUCTIONS {
        return Err(serialization_error(format!(
            "Bytecode toplam komut sınırı aşıyor: {total_instructions} > {MAX_INSTRUCTIONS}"
        )));
    }

    validate_instructions(
        &program.instructions,
        program.constants.len(),
        program.functions.len(),
        "Bytecode ana program",
    )?;
    for (function_index, function) in program.functions.iter().enumerate() {
        if function.instruction_spans.len() != function.instructions.len() {
            return Err(serialization_error(format!(
                "Bytecode fonksiyonu {function_index} konum tablosu uzunluğu uyuşmuyor: {} != {}",
                function.instruction_spans.len(),
                function.instructions.len()
            )));
        }
        if function.params.len() > MAX_CALL_ARGUMENTS {
            return Err(serialization_error(format!(
                "Bytecode fonksiyonu {function_index} parametre sınırını aşıyor: {}",
                function.params.len()
            )));
        }
        let mut unique = std::collections::HashSet::with_capacity(function.params.len());
        for parameter in &function.params {
            if parameter.is_empty() || parameter.len() > MAX_SYMBOL_BYTES {
                return Err(serialization_error(format!(
                    "Bytecode fonksiyonu {function_index} geçersiz parametre adı içeriyor"
                )));
            }
            if !unique.insert(parameter) {
                return Err(serialization_error(format!(
                    "Bytecode fonksiyonu {function_index} yinelenen parametre içeriyor: {parameter}"
                )));
            }
        }
        validate_instructions(
            &function.instructions,
            program.constants.len(),
            program.functions.len(),
            &format!("Bytecode fonksiyonu {function_index}"),
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_program, Constant, OpCode, Program};

    #[test]
    fn yigin_eksilmesi_reddedilir() {
        let program = Program {
            constants: Vec::new(),
            functions: Vec::new(),
            instructions: vec![OpCode::Add],
            instruction_spans: vec![None],
        };
        assert!(validate_program(&program)
            .expect_err("eksik işlenenler reddedilmeli")
            .to_string()
            .contains("yığın eksilmesine"));
    }

    #[test]
    fn farkli_yigin_yukseklikleri_birlestirilemez() {
        let program = Program {
            constants: vec![Constant::Sayi(1.0)],
            functions: Vec::new(),
            instructions: vec![
                OpCode::PushConstant(0),
                OpCode::JumpIfFalse(4),
                OpCode::PushConstant(0),
                OpCode::Jump(5),
                OpCode::Jump(5),
                OpCode::Bos,
            ],
            instruction_spans: vec![None; 6],
        };
        assert!(validate_program(&program)
            .expect_err("uyumsuz kontrol akışı reddedilmeli")
            .to_string()
            .contains("uyumsuz yığın"));
    }

    #[test]
    fn eslesmeyen_hata_blogunu_reddeder() {
        let program = Program {
            constants: Vec::new(),
            functions: Vec::new(),
            instructions: vec![OpCode::TryBlockEnd],
            instruction_spans: vec![None],
        };
        assert!(validate_program(&program)
            .expect_err("eşleşmeyen hata bloğu reddedilmeli")
            .to_string()
            .contains("eşleşmeyen"));
    }
}
