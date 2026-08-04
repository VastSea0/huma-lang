//! Hüma Host Module Interface (HMI).
//!
//! HMI, güvenilmeyen native/haricî kütüphaneleri dil sürecinden ayırmak için
//! sürümlü, boyut-sınırlı ve uygulamadan bağımsız mesaj sözleşmesidir.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Component, Path};
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;

mod process;

pub use process::ProcessClient;

pub const MAX_FRAME_BYTES: usize = 16 * 1_024 * 1_024;
pub const MAX_COLLECTION_ITEMS: usize = 1_000_000;
pub const MAX_VALUE_DEPTH: usize = 128;
pub const TRANSPORT_STDIO_JSON_V1: &str = "stdio-json-v1";
pub const INTERFACE_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
}

impl ProtocolVersion {
    pub const V1_0: Self = Self { major: 1, minor: 0 };

    pub fn negotiate(self, peer: Self) -> Result<Self, HmiError> {
        if self.major != peer.major {
            return Err(HmiError::IncompatibleVersion {
                host: self,
                module: peer,
            });
        }
        Ok(Self {
            major: self.major,
            minor: self.minor.min(peer.minor),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum HmiValue {
    Number(f64),
    Boolean(bool),
    Text(String),
    Bytes(Vec<u8>),
    List(Vec<HmiValue>),
    Map(BTreeMap<String, HmiValue>),
    Empty,
}

impl HmiValue {
    pub fn validate(&self) -> Result<(), HmiError> {
        fn visit(value: &HmiValue, depth: usize, items: &mut usize) -> Result<(), HmiError> {
            if depth > MAX_VALUE_DEPTH {
                return Err(HmiError::InvalidValue(
                    "değer iç içelik sınırını aşıyor".to_string(),
                ));
            }
            *items = items
                .checked_add(1)
                .ok_or_else(|| HmiError::InvalidValue("öğe sayısı hesabı taştı".to_string()))?;
            if *items > MAX_COLLECTION_ITEMS {
                return Err(HmiError::InvalidValue(format!(
                    "değer {MAX_COLLECTION_ITEMS} öğe sınırını aşıyor"
                )));
            }
            match value {
                HmiValue::Number(number) if !number.is_finite() => Err(HmiError::InvalidValue(
                    "sayısal değer sonlu olmalıdır".to_string(),
                )),
                HmiValue::List(values) => {
                    for value in values {
                        visit(value, depth + 1, items)?;
                    }
                    Ok(())
                }
                HmiValue::Map(values) => {
                    for (key, value) in values {
                        validate_identifier(key, "harita anahtarı")?;
                        visit(value, depth + 1, items)?;
                    }
                    Ok(())
                }
                _ => Ok(()),
            }
        }
        visit(self, 0, &mut 0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageContract {
    pub protocol: ProtocolVersion,
    pub transport: String,
    pub executable: String,
    pub interface: InterfaceContract,
}

impl PackageContract {
    pub fn validate(&self) -> Result<(), HmiError> {
        if self.protocol.major == 0 {
            return Err(HmiError::InvalidContract(
                "HMI ana sürümü sıfır olamaz".to_string(),
            ));
        }
        if self.transport != TRANSPORT_STDIO_JSON_V1 {
            return Err(HmiError::InvalidContract(format!(
                "desteklenmeyen HMI taşıması: {}",
                self.transport
            )));
        }
        validate_identifier(&self.executable, "HMI çalıştırılabilir yolu")?;
        let executable = Path::new(&self.executable);
        if executable.is_absolute()
            || executable
                .components()
                .any(|part| !matches!(part, Component::Normal(_)))
        {
            return Err(HmiError::InvalidContract(
                "HMI çalıştırılabilir yolu paket köküne göre güvenli ve göreli olmalıdır"
                    .to_string(),
            ));
        }
        self.interface.validate()?;
        Ok(())
    }
}

/// HMI sınırından geçebilen, kararlı ve dil-bağımsız değer türleri.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueType {
    Number,
    Boolean,
    Text,
    Bytes,
    List,
    Map,
    Empty,
    Dynamic,
}

/// Bir dış çağrının gözlemlenebilir etkileri. Yeni etki eklemek kırıcıdır.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Effect {
    FileRead,
    FileWrite,
    NetworkClient,
    NetworkServer,
    Process,
    Database,
    Gui,
    Clock,
    Random,
    Native,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterContract {
    pub name: String,
    pub value_type: ValueType,
    #[serde(default)]
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorContract {
    pub code: String,
    #[serde(default)]
    pub retryable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionContract {
    pub name: String,
    #[serde(default)]
    pub parameters: Vec<ParameterContract>,
    pub return_type: ValueType,
    #[serde(default)]
    pub effects: Vec<Effect>,
    #[serde(default)]
    pub errors: Vec<ErrorContract>,
}

/// Paketle yayımlanan makinece okunabilir imza, etki ve hata kataloğu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterfaceContract {
    pub schema_version: u16,
    pub module: String,
    pub huma_version_requirement: String,
    #[serde(default)]
    pub functions: Vec<FunctionContract>,
}

impl InterfaceContract {
    pub fn validate(&self) -> Result<(), HmiError> {
        if self.schema_version != INTERFACE_SCHEMA_VERSION {
            return Err(HmiError::InvalidContract(format!(
                "desteklenmeyen arayüz şema sürümü: {}; beklenen: {INTERFACE_SCHEMA_VERSION}",
                self.schema_version
            )));
        }
        validate_identifier(&self.module, "modül adı")?;
        validate_identifier(&self.huma_version_requirement, "Hüma sürüm gereksinimi")?;
        if self.functions.len() > MAX_COLLECTION_ITEMS {
            return Err(HmiError::InvalidContract(
                "dışa aktarılan fonksiyon sayısı sınırı aşıyor".to_string(),
            ));
        }
        let mut function_names = std::collections::BTreeSet::new();
        for function in &self.functions {
            validate_identifier(&function.name, "fonksiyon adı")?;
            if !function_names.insert(function.name.as_str()) {
                return Err(HmiError::InvalidContract(format!(
                    "yinelenen fonksiyon: {}",
                    function.name
                )));
            }
            let mut optional_seen = false;
            let mut parameter_names = std::collections::BTreeSet::new();
            for parameter in &function.parameters {
                validate_identifier(&parameter.name, "parametre adı")?;
                if !parameter_names.insert(parameter.name.as_str()) {
                    return Err(HmiError::InvalidContract(format!(
                        "{} fonksiyonunda yinelenen parametre: {}",
                        function.name, parameter.name
                    )));
                }
                if optional_seen && !parameter.optional {
                    return Err(HmiError::InvalidContract(format!(
                        "{} fonksiyonunda zorunlu parametre isteğe bağlı parametreden sonra gelemez",
                        function.name
                    )));
                }
                optional_seen |= parameter.optional;
            }
            let mut effects = std::collections::BTreeSet::new();
            for effect in &function.effects {
                if !effects.insert(*effect) {
                    return Err(HmiError::InvalidContract(format!(
                        "{} fonksiyonunda yinelenen etki",
                        function.name
                    )));
                }
            }
            let mut error_codes = std::collections::BTreeSet::new();
            for error in &function.errors {
                validate_error_code(&error.code)?;
                if !error_codes.insert(error.code.as_str()) {
                    return Err(HmiError::InvalidContract(format!(
                        "{} fonksiyonunda yinelenen hata kodu: {}",
                        function.name, error.code
                    )));
                }
            }
        }
        Ok(())
    }

    /// Yeni sözleşmenin eski istemciler için geriye uyumlu olup olmadığını denetler.
    pub fn check_backward_compatible_with(&self, previous: &Self) -> Result<(), HmiError> {
        self.validate()?;
        previous.validate()?;
        if self.module != previous.module {
            return Err(HmiError::IncompatibleInterface(
                "modül adı değiştirilemez".to_string(),
            ));
        }
        let current = self
            .functions
            .iter()
            .map(|function| (function.name.as_str(), function))
            .collect::<BTreeMap<_, _>>();
        for old in &previous.functions {
            let new = current.get(old.name.as_str()).ok_or_else(|| {
                HmiError::IncompatibleInterface(format!(
                    "dışa aktarılan fonksiyon kaldırıldı: {}",
                    old.name
                ))
            })?;
            if new.return_type != old.return_type {
                return Err(HmiError::IncompatibleInterface(format!(
                    "{} dönüş türünü değiştirdi",
                    old.name
                )));
            }
            if new.parameters.len() < old.parameters.len() {
                return Err(HmiError::IncompatibleInterface(format!(
                    "{} parametre kaldırdı",
                    old.name
                )));
            }
            for (index, old_parameter) in old.parameters.iter().enumerate() {
                if new.parameters[index] != *old_parameter {
                    return Err(HmiError::IncompatibleInterface(format!(
                        "{} fonksiyonunun {}. parametresi değişti",
                        old.name,
                        index + 1
                    )));
                }
            }
            if new.parameters[old.parameters.len()..]
                .iter()
                .any(|parameter| !parameter.optional)
            {
                return Err(HmiError::IncompatibleInterface(format!(
                    "{} zorunlu parametre ekledi",
                    old.name
                )));
            }
            let old_effects = old
                .effects
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>();
            if new
                .effects
                .iter()
                .any(|effect| !old_effects.contains(effect))
            {
                return Err(HmiError::IncompatibleInterface(format!(
                    "{} yeni bir etki gerektiriyor",
                    old.name
                )));
            }
            let old_errors = old
                .errors
                .iter()
                .map(|error| (error.code.as_str(), error.retryable))
                .collect::<BTreeMap<_, _>>();
            for error in &new.errors {
                match old_errors.get(error.code.as_str()) {
                    Some(retryable) if *retryable == error.retryable => {}
                    Some(_) => {
                        return Err(HmiError::IncompatibleInterface(format!(
                            "{} hata davranışını değiştirdi: {}",
                            old.name, error.code
                        )));
                    }
                    None => {
                        return Err(HmiError::IncompatibleInterface(format!(
                            "{} yeni hata ekledi: {}",
                            old.name, error.code
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "operation", content = "payload", rename_all = "snake_case")]
pub enum RequestOperation {
    Initialize {
        module: String,
        host_version: ProtocolVersion,
    },
    Call {
        function: String,
        arguments: Vec<HmiValue>,
    },
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub protocol: ProtocolVersion,
    pub request_id: u64,
    pub operation: RequestOperation,
}

impl Request {
    pub fn validate(&self) -> Result<(), HmiError> {
        if self.protocol.major == 0 {
            return Err(HmiError::InvalidMessage(
                "istek protokol ana sürümü sıfır olamaz".to_string(),
            ));
        }
        if self.request_id == 0 || self.request_id > (1_u64 << 53) - 1 {
            return Err(HmiError::InvalidMessage(
                "istek kimliği 1..2^53-1 aralığında olmalıdır".to_string(),
            ));
        }
        match &self.operation {
            RequestOperation::Initialize {
                module,
                host_version,
            } => {
                if host_version.major == 0 {
                    return Err(HmiError::InvalidMessage(
                        "ana makine protokol ana sürümü sıfır olamaz".to_string(),
                    ));
                }
                validate_identifier(module, "modül adı")
            }
            RequestOperation::Call {
                function,
                arguments,
            } => {
                validate_identifier(function, "fonksiyon adı")?;
                if arguments.len() > MAX_COLLECTION_ITEMS {
                    return Err(HmiError::InvalidMessage(
                        "argüman sayısı sınırı aşıyor".to_string(),
                    ));
                }
                for argument in arguments {
                    argument.validate()?;
                }
                Ok(())
            }
            RequestOperation::Shutdown => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", content = "payload", rename_all = "snake_case")]
pub enum ResponsePayload {
    Success(HmiValue),
    Failure(RemoteError),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub protocol: ProtocolVersion,
    pub request_id: u64,
    pub response: ResponsePayload,
}

impl Response {
    pub fn validate(&self) -> Result<(), HmiError> {
        if self.protocol.major == 0 {
            return Err(HmiError::InvalidMessage(
                "yanıt protokol ana sürümü sıfır olamaz".to_string(),
            ));
        }
        if self.request_id == 0 || self.request_id > (1_u64 << 53) - 1 {
            return Err(HmiError::InvalidMessage(
                "yanıt kimliği 1..2^53-1 aralığında olmalıdır".to_string(),
            ));
        }
        match &self.response {
            ResponsePayload::Success(value) => value.validate(),
            ResponsePayload::Failure(error) => {
                validate_error_code(&error.code)?;
                if error.message.len() > MAX_FRAME_BYTES || error.message.contains('\0') {
                    return Err(HmiError::InvalidMessage(
                        "uzak hata iletisi geçersiz".to_string(),
                    ));
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum HmiError {
    #[error("HMI-E-VERSION: uyumsuz HMI sürümleri (ana makine {host:?}, modül {module:?})")]
    IncompatibleVersion {
        host: ProtocolVersion,
        module: ProtocolVersion,
    },
    #[error("HMI-E-FRAME: çerçeve {0} bayt sınırını aşıyor")]
    FrameTooLarge(usize),
    #[error("HMI-E-EOF: HMI akışı beklenmedik biçimde kapandı")]
    UnexpectedEof,
    #[error("HMI-E-IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("HMI-E-JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("HMI-E-VALUE: {0}")]
    InvalidValue(String),
    #[error("HMI-E-MESSAGE: {0}")]
    InvalidMessage(String),
    #[error("HMI-E-CONTRACT: {0}")]
    InvalidContract(String),
    #[error("HMI-E-COMPAT: {0}")]
    IncompatibleInterface(String),
    #[error("HMI-E-PROTOCOL: {0}")]
    ProtocolViolation(String),
    #[error("HMI-E-TIMEOUT: modül {0:?} içinde yanıt vermedi")]
    Timeout(std::time::Duration),
    #[error("HMI-E-REMOTE: {code}: {message} (yeniden denenebilir: {retryable})")]
    RemoteFailure {
        code: String,
        message: String,
        retryable: bool,
    },
    #[error("HMI-E-THREAD: HMI okuyucusu başlatılamadı: {0}")]
    ReaderSpawn(#[source] std::io::Error),
}

pub fn write_frame<T: Serialize>(writer: &mut impl Write, value: &T) -> Result<(), HmiError> {
    let payload = serde_json::to_vec(value)?;
    if payload.len() > MAX_FRAME_BYTES {
        return Err(HmiError::FrameTooLarge(payload.len()));
    }
    let length =
        u32::try_from(payload.len()).map_err(|_| HmiError::FrameTooLarge(payload.len()))?;
    writer.write_all(&length.to_be_bytes())?;
    writer.write_all(&payload)?;
    writer.flush()?;
    Ok(())
}

pub fn read_frame<T: for<'de> Deserialize<'de>>(reader: &mut impl Read) -> Result<T, HmiError> {
    let mut length = [0_u8; 4];
    read_exact_or_eof(reader, &mut length)?;
    let length = u32::from_be_bytes(length) as usize;
    if length > MAX_FRAME_BYTES {
        return Err(HmiError::FrameTooLarge(length));
    }
    let mut payload = vec![0_u8; length];
    read_exact_or_eof(reader, &mut payload)?;
    Ok(serde_json::from_slice(&payload)?)
}

fn read_exact_or_eof(reader: &mut impl Read, buffer: &mut [u8]) -> Result<(), HmiError> {
    reader.read_exact(buffer).map_err(|error| {
        if error.kind() == std::io::ErrorKind::UnexpectedEof {
            HmiError::UnexpectedEof
        } else {
            HmiError::Io(error)
        }
    })
}

fn validate_identifier(value: &str, field: &str) -> Result<(), HmiError> {
    if value.is_empty() || value.len() > 4_096 {
        return Err(HmiError::InvalidContract(format!(
            "{field} boş olamaz ve 4096 baytı aşamaz"
        )));
    }
    if value.contains('\0') || value.chars().any(char::is_control) {
        return Err(HmiError::InvalidContract(format!(
            "{field} denetim karakteri içeremez"
        )));
    }
    if value.nfc().collect::<String>() != value {
        return Err(HmiError::InvalidContract(format!(
            "{field} NFC biçiminde olmalıdır"
        )));
    }
    Ok(())
}

fn validate_error_code(code: &str) -> Result<(), HmiError> {
    if code.is_empty()
        || code.len() > 128
        || !code
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(HmiError::InvalidContract(format!(
            "geçersiz hata kodu: {code}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ayni_ana_surum_en_dusuk_minor_surumde_uzlasir() {
        let host = ProtocolVersion { major: 1, minor: 4 };
        let module = ProtocolVersion { major: 1, minor: 2 };
        assert_eq!(
            host.negotiate(module).unwrap(),
            ProtocolVersion { major: 1, minor: 2 }
        );
        assert!(host
            .negotiate(ProtocolVersion { major: 2, minor: 0 })
            .is_err());
    }

    #[test]
    fn cerceve_uzunluk_on_ekiyle_kayipsiz_doner() {
        let request = Request {
            protocol: ProtocolVersion::V1_0,
            request_id: 7,
            operation: RequestOperation::Call {
                function: "topla".to_string(),
                arguments: vec![HmiValue::Number(1.0), HmiValue::Number(2.0)],
            },
        };
        let mut bytes = Vec::new();
        write_frame(&mut bytes, &request).unwrap();
        let decoded: Request = read_frame(&mut bytes.as_slice()).unwrap();
        assert_eq!(decoded, request);
        decoded.validate().unwrap();
    }

    #[test]
    fn asiri_buyuk_ve_eksik_cerceve_reddedilir() {
        let oversized_prefix = ((MAX_FRAME_BYTES as u32) + 1).to_be_bytes();
        let mut oversized = oversized_prefix.as_slice();
        assert!(matches!(
            read_frame::<Request>(&mut oversized),
            Err(HmiError::FrameTooLarge(_))
        ));
        let mut truncated = [0, 0, 0, 4, b'{'].as_slice();
        assert!(matches!(
            read_frame::<Request>(&mut truncated),
            Err(HmiError::UnexpectedEof)
        ));
    }

    #[test]
    fn sonlu_olmayan_sayi_hmi_sinirindan_gecmez() {
        assert!(HmiValue::Number(f64::NAN).validate().is_err());
    }

    fn test_interface() -> InterfaceContract {
        InterfaceContract {
            schema_version: INTERFACE_SCHEMA_VERSION,
            module: "örnek".to_string(),
            huma_version_requirement: ">=0.6,<1.0".to_string(),
            functions: vec![FunctionContract {
                name: "topla".to_string(),
                parameters: vec![ParameterContract {
                    name: "sol".to_string(),
                    value_type: ValueType::Number,
                    optional: false,
                }],
                return_type: ValueType::Number,
                effects: Vec::new(),
                errors: vec![ErrorContract {
                    code: "ORNEK-ARALIK-1".to_string(),
                    retryable: false,
                }],
            }],
        }
    }

    #[test]
    fn arayuz_yalniz_istege_bagli_parametreyle_genisleyebilir() {
        let previous = test_interface();
        let mut current = previous.clone();
        current.functions[0].parameters.push(ParameterContract {
            name: "sag".to_string(),
            value_type: ValueType::Number,
            optional: true,
        });
        current
            .check_backward_compatible_with(&previous)
            .expect("isteğe bağlı son parametre uyumlu olmalı");

        current.functions[0].effects.push(Effect::NetworkClient);
        assert!(matches!(
            current.check_backward_compatible_with(&previous),
            Err(HmiError::IncompatibleInterface(_))
        ));
    }
}
