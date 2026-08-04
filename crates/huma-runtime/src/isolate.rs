//! Paylaşımlı değişken heap'i olmadan iş parçacığı tabanlı isolate'lar.

use crate::capability::{self, CapabilitySet};
use crate::gc::Gc;
use crate::interpreter::Yorumlayici;
use crate::lexer::Lexer;
use crate::limits::ExecutionLimits;
use crate::parser::Parser;
use crate::value::Deger;
use huma_syntax::{DiagnosticEnvelope, ErrorCategory};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use thiserror::Error;

const MAX_MESSAGE_ITEMS: usize = 1_000_000;
const MAX_MESSAGE_DEPTH: usize = 128;

/// Isolate sınırından geçebilen, heap paylaşmayan değerler.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum IsolateValue {
    Number(f64),
    Text(String),
    Bytes(Vec<u8>),
    List(Vec<IsolateValue>),
    Map(BTreeMap<String, IsolateValue>),
    Empty,
}

#[derive(Debug, Clone)]
pub struct IsolateConfig {
    pub capabilities: CapabilitySet,
    pub limits: ExecutionLimits,
    pub response_timeout: Duration,
}

impl Default for IsolateConfig {
    fn default() -> Self {
        Self {
            capabilities: CapabilitySet::deny_all(),
            limits: ExecutionLimits::default(),
            response_timeout: Duration::from_secs(30),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct IsolateRequest {
    pub source: String,
    pub bindings: BTreeMap<String, IsolateValue>,
    pub exports: Vec<String>,
}

impl IsolateRequest {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolateResponse {
    pub output: String,
    pub exports: BTreeMap<String, IsolateValue>,
}

#[derive(Debug, Error)]
pub enum IsolateError {
    #[error("Isolate iş parçacığı başlatılamadı: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("Isolate başlatılamadı: {0}")]
    Initialization(String),
    #[error("Isolate iletişim kanalı kapandı")]
    ChannelClosed,
    #[error("Isolate {0:?} içinde yanıt vermedi")]
    Timeout(Duration),
    #[error("Isolate çalıştırma hatası: {0:?}")]
    Execution(DiagnosticEnvelope),
}

enum Command {
    Execute {
        request: IsolateRequest,
        respond_to: Sender<Result<IsolateResponse, DiagnosticEnvelope>>,
    },
    Shutdown,
}

/// Bir Hüma yorumlayıcısını kendine ait iş parçacığı ve heap'iyle tutar.
pub struct Isolate {
    sender: Sender<Command>,
    worker: Option<JoinHandle<()>>,
    response_timeout: Duration,
    timed_out: AtomicBool,
}

impl Isolate {
    pub fn spawn(config: IsolateConfig) -> Result<Self, IsolateError> {
        config
            .limits
            .validate()
            .map_err(IsolateError::Initialization)?;
        if config.response_timeout.is_zero() {
            return Err(IsolateError::Initialization(
                "Isolate yanıt zaman aşımı sıfır olamaz".to_string(),
            ));
        }

        let (sender, receiver) = mpsc::channel();
        let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
        let capabilities = config.capabilities.clone();
        let limits = config.limits;
        let worker = thread::Builder::new()
            .name("huma-isolate".to_string())
            .spawn(move || worker_loop(receiver, ready_sender, capabilities, limits))
            .map_err(IsolateError::Spawn)?;

        match ready_receiver.recv() {
            Ok(Ok(())) => Ok(Self {
                sender,
                worker: Some(worker),
                response_timeout: config.response_timeout,
                timed_out: AtomicBool::new(false),
            }),
            Ok(Err(error)) => {
                let _ = worker.join();
                Err(IsolateError::Initialization(error))
            }
            Err(_) => {
                let _ = worker.join();
                Err(IsolateError::ChannelClosed)
            }
        }
    }

    pub fn execute(&self, request: IsolateRequest) -> Result<IsolateResponse, IsolateError> {
        let (respond_to, response) = mpsc::channel();
        self.sender
            .send(Command::Execute {
                request,
                respond_to,
            })
            .map_err(|_| IsolateError::ChannelClosed)?;
        match response.recv_timeout(self.response_timeout) {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(diagnostic)) => Err(IsolateError::Execution(diagnostic)),
            Err(RecvTimeoutError::Disconnected) => Err(IsolateError::ChannelClosed),
            Err(RecvTimeoutError::Timeout) => {
                self.timed_out.store(true, Ordering::Release);
                Err(IsolateError::Timeout(self.response_timeout))
            }
        }
    }
}

impl Drop for Isolate {
    fn drop(&mut self) {
        let _ = self.sender.send(Command::Shutdown);
        if !self.timed_out.load(Ordering::Acquire) {
            if let Some(worker) = self.worker.take() {
                let _ = worker.join();
            }
        }
    }
}

fn worker_loop(
    receiver: Receiver<Command>,
    ready: mpsc::SyncSender<Result<(), String>>,
    capabilities: CapabilitySet,
    limits: ExecutionLimits,
) {
    let _capability_guard = match capability::install(capabilities) {
        Ok(guard) => guard,
        Err(error) => {
            let _ = ready.send(Err(error));
            return;
        }
    };
    let output = Rc::new(RefCell::new(String::new()));
    let mut interpreter = match Yorumlayici::new()
        .with_output_buffer(Rc::clone(&output))
        .with_limits(limits)
    {
        Ok(interpreter) => interpreter,
        Err(error) => {
            let _ = ready.send(Err(error));
            return;
        }
    };
    if ready.send(Ok(())).is_err() {
        return;
    }

    while let Ok(command) = receiver.recv() {
        match command {
            Command::Execute {
                request,
                respond_to,
            } => {
                let result = execute_request(&mut interpreter, &output, request);
                let _ = respond_to.send(result);
            }
            Command::Shutdown => break,
        }
    }
}

fn execute_request(
    interpreter: &mut Yorumlayici,
    output: &Rc<RefCell<String>>,
    request: IsolateRequest,
) -> Result<IsolateResponse, DiagnosticEnvelope> {
    output
        .try_borrow_mut()
        .map_err(|_| isolate_diagnostic("Isolate çıktı tamponu kullanımda"))?
        .clear();

    let mut budget = 0;
    for (name, value) in request.bindings {
        let value = into_runtime(value, 0, &mut budget).map_err(|error| {
            isolate_diagnostic(&format!("'{name}' ileti bağı dönüştürülemedi: {error}"))
        })?;
        interpreter.global_degiskenler.insert(name, value);
    }

    let mut parser = Parser::new(Lexer::new(&request.source));
    let (program, diagnostics) = parser.parse_program_with_diagnostics();
    if let Some(error) = diagnostics.first() {
        return Err(error.diagnostic());
    }
    interpreter
        .yorumla_kontrollu(program)
        .map_err(|error| error.diagnostic())?;

    let mut exports = BTreeMap::new();
    for name in request.exports {
        let value = interpreter.global_degiskenler.get(&name).ok_or_else(|| {
            isolate_diagnostic(&format!("Isolate dışa aktarımı bulunamadı: {name}"))
        })?;
        let mut active = HashSet::new();
        let mut export_budget = 0;
        let value = from_runtime(value, &mut active, 0, &mut export_budget)
            .map_err(|error| isolate_diagnostic(&format!("'{name}' dışa aktarılamadı: {error}")))?;
        exports.insert(name, value);
    }
    let output = output
        .try_borrow()
        .map_err(|_| isolate_diagnostic("Isolate çıktı tamponu kullanımda"))?
        .clone();
    Ok(IsolateResponse { output, exports })
}

fn isolate_diagnostic(message: &str) -> DiagnosticEnvelope {
    DiagnosticEnvelope {
        schema_version: DiagnosticEnvelope::SCHEMA_VERSION,
        code: "HUMA-ISOLATE-0001".to_string(),
        category: ErrorCategory::Runtime,
        message: message.to_string(),
        location: None,
        stack: Default::default(),
        details: Default::default(),
        causes: Default::default(),
    }
}

fn count_item(items: &mut usize) -> Result<(), String> {
    *items = items
        .checked_add(1)
        .ok_or_else(|| "Isolate ileti öğe sayısı taştı".to_string())?;
    if *items > MAX_MESSAGE_ITEMS {
        return Err(format!(
            "Isolate iletisi {MAX_MESSAGE_ITEMS} öğe sınırını aşıyor"
        ));
    }
    Ok(())
}

fn into_runtime(value: IsolateValue, depth: usize, items: &mut usize) -> Result<Deger, String> {
    if depth > MAX_MESSAGE_DEPTH {
        return Err("Isolate ileti iç içelik sınırı aşıldı".to_string());
    }
    count_item(items)?;
    match value {
        IsolateValue::Number(value) if value.is_finite() => Ok(Deger::Sayi(value)),
        IsolateValue::Number(_) => Err("Isolate iletisindeki sayı sonlu olmalıdır".to_string()),
        IsolateValue::Text(value) => Ok(Deger::Metin(value)),
        IsolateValue::Bytes(value) => Ok(Deger::Bayt(value)),
        IsolateValue::List(values) => values
            .into_iter()
            .map(|value| into_runtime(value, depth + 1, items))
            .collect::<Result<Vec<_>, _>>()
            .map(Gc::new)
            .map(Deger::Liste),
        IsolateValue::Map(values) => values
            .into_iter()
            .map(|(key, value)| into_runtime(value, depth + 1, items).map(|value| (key, value)))
            .collect::<Result<_, _>>()
            .map(Gc::new)
            .map(Deger::Sozluk),
        IsolateValue::Empty => Ok(Deger::Bos),
    }
}

fn from_runtime(
    value: &Deger,
    active: &mut HashSet<usize>,
    depth: usize,
    items: &mut usize,
) -> Result<IsolateValue, String> {
    if depth > MAX_MESSAGE_DEPTH {
        return Err("Isolate ileti iç içelik sınırı aşıldı".to_string());
    }
    count_item(items)?;
    match value {
        Deger::Sayi(value) if value.is_finite() => Ok(IsolateValue::Number(*value)),
        Deger::Sayi(_) => Err("Sonlu olmayan sayı isolate sınırından geçemez".to_string()),
        Deger::Metin(value) => Ok(IsolateValue::Text(value.clone())),
        Deger::Bayt(value) => Ok(IsolateValue::Bytes(value.clone())),
        Deger::Bos => Ok(IsolateValue::Empty),
        Deger::Liste(values) => {
            let identity = Gc::as_ptr(values) as usize;
            if !active.insert(identity) {
                return Err("Döngüsel liste isolate sınırından geçemez".to_string());
            }
            let values = values
                .try_borrow()
                .map_err(|_| "Liste kullanımda".to_string())?;
            let result = values
                .iter()
                .map(|value| from_runtime(value, active, depth + 1, items))
                .collect::<Result<Vec<_>, _>>();
            active.remove(&identity);
            result.map(IsolateValue::List)
        }
        Deger::Sozluk(values) => {
            let identity = Gc::as_ptr(values) as usize;
            if !active.insert(identity) {
                return Err("Döngüsel sözlük isolate sınırından geçemez".to_string());
            }
            let values = values
                .try_borrow()
                .map_err(|_| "Sözlük kullanımda".to_string())?;
            let result = values
                .iter()
                .map(|(key, value)| {
                    from_runtime(value, active, depth + 1, items).map(|value| (key.clone(), value))
                })
                .collect::<Result<BTreeMap<_, _>, _>>();
            active.remove(&identity);
            result.map(IsolateValue::Map)
        }
        Deger::Vektor(values) => values
            .try_borrow()
            .map_err(|_| "Vektör kullanımda".to_string())?
            .iter()
            .map(|value| Ok(IsolateValue::Number(*value)))
            .collect::<Result<Vec<_>, String>>()
            .map(IsolateValue::List),
        Deger::Matris {
            satirlar,
            sutunlar,
            veri,
        } => {
            let values = veri
                .try_borrow()
                .map_err(|_| "Matris kullanımda".to_string())?;
            if values.len() != satirlar.saturating_mul(*sutunlar) {
                return Err("Matris boyutu bozuk".to_string());
            }
            let rows = values
                .chunks(*sutunlar)
                .map(|row| {
                    IsolateValue::List(
                        row.iter()
                            .map(|value| IsolateValue::Number(*value))
                            .collect(),
                    )
                })
                .collect();
            Ok(IsolateValue::List(rows))
        }
        Deger::GorevId(_)
        | Deger::Fonksiyon { .. }
        | Deger::BytecodeFonksiyon { .. }
        | Deger::DahiliFonksiyon(_)
        | Deger::BaglamliDahiliFonksiyon(_)
        | Deger::Sinif { .. }
        | Deger::Nesne { .. }
        | Deger::Hata(_)
        | Deger::Harici(_) => {
            Err("Bu çalışma zamanı değeri isolate sınırından geçemez".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolate_durumu_kendi_heapinde_korur() {
        let isolate = Isolate::spawn(IsolateConfig::default()).expect("isolate başlamalı");
        let mut request = IsolateRequest::new("x = 41 olsun\nx'i yazdır");
        request.exports.push("x".to_string());
        let response = isolate.execute(request).expect("kaynak çalışmalı");
        assert_eq!(response.output, "41\n");
        assert_eq!(response.exports.get("x"), Some(&IsolateValue::Number(41.0)));

        let response = isolate
            .execute(IsolateRequest::new("x = x + 1 olsun\nx'i yazdır"))
            .expect("ikinci istek aynı isolate durumunu görmeli");
        assert_eq!(response.output, "42\n");
    }

    #[test]
    fn iki_isolate_degisken_heapini_paylasmaz() {
        let first = Isolate::spawn(IsolateConfig::default()).expect("ilk isolate başlamalı");
        let second = Isolate::spawn(IsolateConfig::default()).expect("ikinci isolate başlamalı");
        first
            .execute(IsolateRequest::new("x = 1 olsun"))
            .expect("ilk atama çalışmalı");
        let error = second
            .execute(IsolateRequest::new("x'i yazdır"))
            .expect_err("ikinci isolate x'i görmemeli");
        let IsolateError::Execution(diagnostic) = error else {
            panic!("yapılandırılmış çalışma hatası bekleniyordu");
        };
        assert_eq!(diagnostic.code, "HUMA-RUNTIME-0001");
    }

    #[test]
    fn isolate_varsayilan_olarak_dis_dunyayi_reddeder() {
        let isolate = Isolate::spawn(IsolateConfig::default()).expect("isolate başlamalı");
        let error = isolate
            .execute(IsolateRequest::new("ortam_değişkeni(\"PATH\")"))
            .expect_err("süreç yeteneği kapalı olmalı");
        let IsolateError::Execution(diagnostic) = error else {
            panic!("yapılandırılmış çalışma hatası bekleniyordu");
        };
        assert!(diagnostic.message.contains("yeteneği verilmedi"));
    }
}
