use crate::{
    read_frame, write_frame, HmiError, HmiValue, ProtocolVersion, Request, RequestOperation,
    Response, ResponsePayload,
};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const MAX_REQUEST_ID: u64 = (1_u64 << 53) - 1;

/// Tek bir HMI modülünü ayrı işletim sistemi sürecinde çalıştıran istemci.
///
/// İstemci aynı anda tek istek yürütür. Çerçeve okuyucusu ayrı bir iş
/// parçacığındadır; süre aşımında çocuk süreç sonlandırılarak dil sürecinin
/// sonsuza kadar beklemesi önlenir.
pub struct ProcessClient {
    child: Child,
    stdin: Option<ChildStdin>,
    responses: Receiver<Result<Response, HmiError>>,
    reader: Option<JoinHandle<()>>,
    protocol: ProtocolVersion,
    timeout: Duration,
    next_request_id: u64,
    stopped: bool,
}

impl ProcessClient {
    pub fn spawn(
        executable: &Path,
        module: &str,
        protocol: ProtocolVersion,
        timeout: Duration,
    ) -> Result<Self, HmiError> {
        if timeout.is_zero() {
            return Err(HmiError::InvalidContract(
                "HMI zaman aşımı sıfır olamaz".to_string(),
            ));
        }
        let metadata = std::fs::symlink_metadata(executable)?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(HmiError::InvalidContract(format!(
                "HMI yürütülebiliri normal ve sembolik bağ olmayan bir dosya olmalıdır: {}",
                executable.display()
            )));
        }

        let mut command = Command::new(executable);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .env_clear()
            .env(
                "HUMA_HMI_PROTOCOL",
                format!("{}.{}", protocol.major, protocol.minor),
            );
        if let Some(parent) = executable.parent() {
            command.current_dir(parent);
        }
        let mut child = command.spawn()?;
        let stdin = match child.stdin.take() {
            Some(stdin) => stdin,
            None => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(HmiError::ProtocolViolation(
                    "çocuk sürecin stdin kanalı açılamadı".to_string(),
                ));
            }
        };
        let mut stdout = match child.stdout.take() {
            Some(stdout) => stdout,
            None => {
                drop(stdin);
                let _ = child.kill();
                let _ = child.wait();
                return Err(HmiError::ProtocolViolation(
                    "çocuk sürecin stdout kanalı açılamadı".to_string(),
                ));
            }
        };
        let (response_sender, responses) = mpsc::channel();
        let reader = match thread::Builder::new()
            .name("huma-hmi-reader".to_string())
            .spawn(move || loop {
                let result = read_frame::<Response>(&mut stdout);
                let stop = result.is_err();
                if response_sender.send(result).is_err() || stop {
                    break;
                }
            }) {
            Ok(reader) => reader,
            Err(error) => {
                drop(stdin);
                let _ = child.kill();
                let _ = child.wait();
                return Err(HmiError::ReaderSpawn(error));
            }
        };

        let mut client = Self {
            child,
            stdin: Some(stdin),
            responses,
            reader: Some(reader),
            protocol,
            timeout,
            next_request_id: 1,
            stopped: false,
        };
        let initialization = RequestOperation::Initialize {
            module: module.to_string(),
            host_version: protocol,
        };
        match client.exchange(initialization) {
            Ok(HmiValue::Empty) => Ok(client),
            Ok(_) => {
                client.terminate();
                Err(HmiError::ProtocolViolation(
                    "başlatma yanıtı empty olmalıdır".to_string(),
                ))
            }
            Err(error) => {
                client.terminate();
                Err(error)
            }
        }
    }

    pub fn call(
        &mut self,
        function: impl Into<String>,
        arguments: Vec<HmiValue>,
    ) -> Result<HmiValue, HmiError> {
        self.exchange(RequestOperation::Call {
            function: function.into(),
            arguments,
        })
    }

    pub fn set_timeout(&mut self, timeout: Duration) -> Result<(), HmiError> {
        if timeout.is_zero() {
            return Err(HmiError::InvalidContract(
                "HMI zaman aşımı sıfır olamaz".to_string(),
            ));
        }
        self.timeout = timeout;
        Ok(())
    }

    pub fn shutdown(mut self) -> Result<(), HmiError> {
        let result = match self.exchange(RequestOperation::Shutdown) {
            Ok(HmiValue::Empty) => Ok(()),
            Ok(_) => Err(HmiError::ProtocolViolation(
                "kapatma yanıtı empty olmalıdır".to_string(),
            )),
            Err(error) => Err(error),
        };
        self.terminate();
        result
    }

    fn exchange(&mut self, operation: RequestOperation) -> Result<HmiValue, HmiError> {
        if self.stopped {
            return Err(HmiError::ProtocolViolation(
                "HMI süreci artık çalışmıyor".to_string(),
            ));
        }
        let request_id = self.next_request_id;
        self.next_request_id = self
            .next_request_id
            .checked_add(1)
            .filter(|next| *next <= MAX_REQUEST_ID)
            .ok_or_else(|| HmiError::ProtocolViolation("istek kimliği tükendi".to_string()))?;
        let request = Request {
            protocol: self.protocol,
            request_id,
            operation,
        };
        request.validate()?;
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| HmiError::ProtocolViolation("HMI stdin kanalı kapalı".to_string()))?;
        if let Err(error) = write_frame(stdin, &request) {
            self.terminate();
            return Err(error);
        }

        let response = match self.responses.recv_timeout(self.timeout) {
            Ok(Ok(response)) => response,
            Ok(Err(error)) => {
                self.terminate();
                return Err(error);
            }
            Err(RecvTimeoutError::Disconnected) => {
                self.terminate();
                return Err(HmiError::ProtocolViolation(
                    "HMI yanıt kanalı kapandı".to_string(),
                ));
            }
            Err(RecvTimeoutError::Timeout) => {
                self.terminate();
                return Err(HmiError::Timeout(self.timeout));
            }
        };
        if let Err(error) = response.validate() {
            self.terminate();
            return Err(error);
        }
        if let Err(error) = self.protocol.negotiate(response.protocol) {
            self.terminate();
            return Err(error);
        }
        if response.request_id != request_id {
            self.terminate();
            return Err(HmiError::ProtocolViolation(format!(
                "yanıt kimliği uyuşmuyor: beklenen {request_id}, gelen {}",
                response.request_id
            )));
        }
        match response.response {
            ResponsePayload::Success(value) => Ok(value),
            ResponsePayload::Failure(error) => Err(HmiError::RemoteFailure {
                code: error.code,
                message: error.message,
                retryable: error.retryable,
            }),
        }
    }

    fn terminate(&mut self) {
        if self.stopped {
            return;
        }
        self.stopped = true;
        self.stdin.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

impl Drop for ProcessClient {
    fn drop(&mut self) {
        self.terminate();
    }
}
