//! Hüma'nın isteğe bağlı HTTP istemci/sunucu adaptörü.

use futures_util::StreamExt;
use huma_runtime::capability::{self, Capability};
use huma_runtime::gc::Gc;
use huma_runtime::interpreter::Yorumlayici;
use huma_runtime::value::Deger;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::{mpsc, oneshot};
use tokio::task::{JoinHandle, LocalSet};

const MAX_SAFE_NUMERIC_ID: u64 = (1_u64 << 53) - 1;
const MAX_BODY_BYTES: usize = 16 * 1_024 * 1_024;
const MAX_HEADERS: usize = 128;
const MAX_PENDING_REQUESTS: usize = 4_096;
const MAX_TARGET_BYTES: usize = 8_192;
const MAX_SERVERS: usize = 256;

struct IncomingRequest {
    id: u64,
    url: String,
    method: String,
    body: String,
    respond_to: oneshot::Sender<Response<Body>>,
}

static SERVERS: Lazy<Mutex<HashMap<u64, mpsc::Sender<IncomingRequest>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static SERVER_RECEIVERS: Lazy<tokio::sync::Mutex<HashMap<u64, mpsc::Receiver<IncomingRequest>>>> =
    Lazy::new(|| tokio::sync::Mutex::new(HashMap::new()));
static SERVER_SHUTDOWNS: Lazy<Mutex<HashMap<u64, oneshot::Sender<()>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static RESPONSES: Lazy<Mutex<HashMap<u64, oneshot::Sender<Response<Body>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

struct TaskExecutor {
    runtime: Option<Runtime>,
    local: LocalSet,
    next_id: u64,
    tasks: HashMap<u64, JoinHandle<Deger>>,
    background: Vec<JoinHandle<()>>,
}

impl TaskExecutor {
    fn new() -> Self {
        Self {
            runtime: Builder::new_current_thread().enable_all().build().ok(),
            local: LocalSet::new(),
            next_id: 1,
            tasks: HashMap::new(),
            background: Vec::new(),
        }
    }

    fn spawn<F>(&mut self, future: F) -> Deger
    where
        F: std::future::Future<Output = Deger> + 'static,
    {
        if self.runtime.is_none() {
            return Deger::Hata("Asenkron çalışma zamanı başlatılamadı".to_string());
        }
        let id = self.next_id;
        let Some(next_id) = self.next_id.checked_add(1) else {
            return Deger::Hata("Görev kimliği alanı tükendi".to_string());
        };
        self.next_id = next_id;
        self.tasks.insert(id, self.local.spawn_local(future));
        Deger::GorevId(id)
    }

    fn spawn_background<F>(&mut self, future: F) -> Result<(), String>
    where
        F: std::future::Future<Output = ()> + 'static,
    {
        if self.runtime.is_none() {
            return Err("Asenkron çalışma zamanı başlatılamadı".to_string());
        }
        self.background.retain(|task| !task.is_finished());
        self.background.push(self.local.spawn_local(future));
        Ok(())
    }

    fn await_task(&mut self, id: u64) -> Deger {
        let Some(runtime) = self.runtime.as_mut() else {
            return Deger::Hata("Asenkron çalışma zamanı kullanılamıyor".to_string());
        };
        match self.tasks.remove(&id) {
            Some(handle) => match runtime.block_on(self.local.run_until(handle)) {
                Ok(value) => value,
                Err(error) => Deger::Hata(format!("Görev hatası: {error}")),
            },
            None => Deger::Hata(format!("Bilinmeyen görev: {id}")),
        }
    }
}

impl Drop for TaskExecutor {
    fn drop(&mut self) {
        for (_, task) in self.tasks.drain() {
            task.abort();
        }
        for task in self.background.drain(..) {
            task.abort();
        }
    }
}

thread_local! {
    static TASKS: std::cell::RefCell<TaskExecutor> = std::cell::RefCell::new(TaskExecutor::new());
}

fn spawn_task<F>(future: F) -> Deger
where
    F: std::future::Future<Output = Deger> + 'static,
{
    TASKS.with(|executor| match executor.try_borrow_mut() {
        Ok(mut executor) => executor.spawn(future),
        Err(_) => Deger::Hata("Asenkron çalışma zamanı kullanımda".to_string()),
    })
}

fn spawn_background<F>(future: F) -> Result<(), String>
where
    F: std::future::Future<Output = ()> + 'static,
{
    TASKS.with(|executor| {
        executor
            .try_borrow_mut()
            .map_err(|_| "Asenkron çalışma zamanı kullanımda".to_string())?
            .spawn_background(future)
    })
}

pub fn gorev_bekle(id: u64) -> Deger {
    TASKS.with(|executor| match executor.try_borrow_mut() {
        Ok(mut executor) => executor.await_task(id),
        Err(_) => Deger::Hata("Görev beklenemedi: asenkron çalışma zamanı kullanımda".to_string()),
    })
}

fn next_id() -> Result<u64, String> {
    NEXT_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            (current < MAX_SAFE_NUMERIC_ID).then_some(current + 1)
        })
        .map_err(|_| "HTTP kimlik alanı tükendi".to_string())
}

fn numeric_id(value: f64, operation: &str) -> Result<u64, String> {
    if !value.is_finite()
        || value < 0.0
        || value.fract() != 0.0
        || value > MAX_SAFE_NUMERIC_ID as f64
    {
        return Err(format!(
            "{operation}: kimlik güvenli aralıkta negatif olmayan tamsayı olmalıdır"
        ));
    }
    Ok(value as u64)
}

fn capability_error(capability: Capability, operation: &str) -> Option<Deger> {
    capability::require(capability, operation)
        .err()
        .map(Deger::Hata)
}

/// Ağ yerleşiklerini verilen ana makine küresel tablosuna ekler.
pub fn kayit_et(globals: &mut HashMap<String, Deger>) {
    globals.insert(
        "dahili_istek".to_string(),
        Deger::DahiliFonksiyon(http_request),
    );
    globals.insert(
        "dahili_sunucu_baslat".to_string(),
        Deger::DahiliFonksiyon(server_start),
    );
    globals.insert(
        "dahili_sunucu_bekle".to_string(),
        Deger::DahiliFonksiyon(server_wait),
    );
    globals.insert(
        "dahili_sunucu_kapat".to_string(),
        Deger::DahiliFonksiyon(server_close),
    );
    globals.insert(
        "dahili_sunucu_yanitla".to_string(),
        Deger::DahiliFonksiyon(server_respond),
    );
    if let Some(function) = globals.get("dahili_sunucu_yanitla").cloned() {
        globals.insert("dahili_sunucu_yanıtla".to_string(), function);
    }
}

/// Ağ yerleşikleriyle birlikte yorumlayıcının görev bekletme sınırını kurar.
pub fn yorumlayiciyi_yapilandir(interpreter: &mut Yorumlayici) {
    kayit_et(&mut interpreter.global_degiskenler);
    interpreter.task_awaiter_ayarla(gorev_bekle);
}

struct NetworkTaskHost;

impl huma_vm::TaskHost for NetworkTaskHost {
    fn await_task(&self, id: u64) -> Deger {
        gorev_bekle(id)
    }
}

pub fn vm_task_host() -> Rc<dyn huma_vm::TaskHost> {
    Rc::new(NetworkTaskHost)
}

fn http_request(args: Vec<Deger>) -> Deger {
    if !(2..=4).contains(&args.len()) {
        return Deger::Hata(format!(
            "dahili_istek: 2 ile 4 arasında argüman bekleniyordu; {} geldi",
            args.len()
        ));
    }
    if let Some(error) = capability_error(Capability::NetworkClient, "dahili_istek") {
        return error;
    }
    let (Deger::Metin(method_text), Deger::Metin(url_text)) = (&args[0], &args[1]) else {
        return Deger::Hata("dahili_istek: yöntem ve URL metin olmalıdır".to_string());
    };
    let method = match reqwest::Method::from_bytes(method_text.as_bytes()) {
        Ok(method) => method,
        Err(error) => return Deger::Hata(format!("dahili_istek: geçersiz HTTP yöntemi: {error}")),
    };
    let url = match reqwest::Url::parse(url_text) {
        Ok(url) if matches!(url.scheme(), "http" | "https") => url,
        Ok(_) => {
            return Deger::Hata(
                "dahili_istek: yalnızca http ve https URL'leri desteklenir".to_string(),
            )
        }
        Err(error) => return Deger::Hata(format!("dahili_istek: geçersiz URL: {error}")),
    };
    let body = match args.get(2) {
        None | Some(Deger::Bos) => None,
        Some(Deger::Metin(text)) => Some(text.as_bytes().to_vec()),
        Some(Deger::Bayt(bytes)) => Some(bytes.clone()),
        Some(other) => {
            return Deger::Hata(format!(
                "dahili_istek: gövde metin, bayt veya boş olmalıdır; {other} geldi"
            ))
        }
    };
    if body
        .as_ref()
        .is_some_and(|body| body.len() > MAX_BODY_BYTES)
    {
        return Deger::Hata(format!(
            "dahili_istek: istek gövdesi {MAX_BODY_BYTES} bayt sınırını aşıyor"
        ));
    }
    let headers = match parse_request_headers(args.get(3)) {
        Ok(headers) => headers,
        Err(error) => return Deger::Hata(error),
    };

    spawn_task(async move {
        let client = match reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
        {
            Ok(client) => client,
            Err(error) => {
                return Deger::Hata(format!(
                    "dahili_istek: HTTP istemcisi oluşturulamadı: {error}"
                ))
            }
        };
        let mut request = client.request(method, url);
        for (name, value) in headers {
            request = request.header(name, value);
        }
        if let Some(body) = body {
            request = request.body(body);
        }
        let response = match request.send().await {
            Ok(response) => response,
            Err(error) => return Deger::Hata(format!("dahili_istek: {error}")),
        };
        if response
            .content_length()
            .is_some_and(|length| length > MAX_BODY_BYTES as u64)
        {
            return Deger::Hata(format!(
                "dahili_istek: yanıt {MAX_BODY_BYTES} bayt sınırını aşıyor"
            ));
        }
        let status = response.status().as_u16() as f64;
        let mut bytes = Vec::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = match chunk {
                Ok(chunk) => chunk,
                Err(error) => {
                    return Deger::Hata(format!("dahili_istek: yanıt gövdesi okunamadı: {error}"))
                }
            };
            if bytes
                .len()
                .checked_add(chunk.len())
                .is_none_or(|length| length > MAX_BODY_BYTES)
            {
                return Deger::Hata(format!(
                    "dahili_istek: yanıt {MAX_BODY_BYTES} bayt sınırını aşıyor"
                ));
            }
            bytes.extend_from_slice(&chunk);
        }
        Deger::Nesne {
            sinif_adi: "İstekCevabı".to_string(),
            alanlar: Gc::new(HashMap::from([
                ("durum".to_string(), Deger::Sayi(status)),
                (
                    "içerik".to_string(),
                    Deger::Metin(String::from_utf8_lossy(&bytes).into_owned()),
                ),
            ])),
            module_kimligi: None,
        }
    })
}

fn parse_request_headers(
    value: Option<&Deger>,
) -> Result<Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)>, String> {
    let fields =
        match value {
            None | Some(Deger::Bos) => return Ok(Vec::new()),
            Some(Deger::Nesne { alanlar, .. }) | Some(Deger::Sozluk(alanlar)) => alanlar
                .try_borrow()
                .map_err(|_| "dahili_istek: başlık nesnesi kullanımda".to_string())?,
            Some(other) => {
                return Err(format!(
                    "dahili_istek: başlıklar nesne, sözlük veya boş olmalıdır; {other} geldi"
                ))
            }
        };
    if fields.len() > MAX_HEADERS {
        return Err(format!(
            "dahili_istek: en fazla {MAX_HEADERS} başlık desteklenir"
        ));
    }
    fields
        .iter()
        .map(|(key, value)| {
            let Deger::Metin(value) = value else {
                return Err(format!(
                    "dahili_istek: '{key}' başlığının değeri metin olmalıdır"
                ));
            };
            let name = reqwest::header::HeaderName::from_bytes(key.as_bytes())
                .map_err(|error| format!("dahili_istek: geçersiz başlık adı '{key}': {error}"))?;
            let value = reqwest::header::HeaderValue::from_str(value).map_err(|error| {
                format!("dahili_istek: '{key}' başlık değeri geçersiz: {error}")
            })?;
            Ok((name, value))
        })
        .collect()
}

fn server_start(args: Vec<Deger>) -> Deger {
    if let Some(error) = capability_error(Capability::NetworkServer, "dahili_sunucu_baslat") {
        return error;
    }
    let port = match args.as_slice() {
        [Deger::Sayi(port)]
            if port.is_finite()
                && port.fract() == 0.0
                && (1.0..=u16::MAX as f64).contains(port) =>
        {
            *port as u16
        }
        [other] => {
            return Deger::Hata(format!(
                "dahili_sunucu_baslat: 1..65535 arasında tamsayı port bekleniyordu; {other} geldi"
            ))
        }
        _ => {
            return Deger::Hata(format!(
                "dahili_sunucu_baslat: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            ))
        }
    };
    let server_id = match next_id() {
        Ok(id) => id,
        Err(error) => return Deger::Hata(format!("dahili_sunucu_baslat: {error}")),
    };
    let address = ([0, 0, 0, 0], port).into();
    let server = match Server::try_bind(&address) {
        Ok(server) => server,
        Err(error) => {
            return Deger::Hata(format!(
                "dahili_sunucu_baslat: {port} portu dinlenemedi: {error}"
            ))
        }
    };
    let (sender, receiver) = mpsc::channel(1_024);
    let (shutdown_sender, shutdown_receiver) = oneshot::channel();
    let mut servers = match SERVERS.lock() {
        Ok(servers) => servers,
        Err(_) => {
            return Deger::Hata("dahili_sunucu_baslat: sunucu kayıt kilidi bozuldu".to_string())
        }
    };
    if servers.len() >= MAX_SERVERS {
        return Deger::Hata(format!(
            "dahili_sunucu_baslat: aynı anda en fazla {MAX_SERVERS} sunucu çalışabilir"
        ));
    }
    let mut shutdowns = match SERVER_SHUTDOWNS.lock() {
        Ok(shutdowns) => shutdowns,
        Err(_) => {
            return Deger::Hata("dahili_sunucu_baslat: kapatma kayıt kilidi bozuldu".to_string())
        }
    };
    servers.insert(server_id, sender);
    shutdowns.insert(server_id, shutdown_sender);
    drop(shutdowns);
    drop(servers);
    SERVER_RECEIVERS.blocking_lock().insert(server_id, receiver);

    let service = make_service_fn(move |_| {
        let server_id = server_id;
        async move {
            Ok::<_, hyper::Error>(service_fn(move |request| {
                handle_incoming_request(server_id, request)
            }))
        }
    });
    if let Err(error) = spawn_background(async move {
        let _ = server
            .serve(service)
            .with_graceful_shutdown(async move {
                let _ = shutdown_receiver.await;
            })
            .await;
        if let Ok(mut servers) = SERVERS.lock() {
            servers.remove(&server_id);
        }
        SERVER_RECEIVERS.lock().await.remove(&server_id);
        if let Ok(mut shutdowns) = SERVER_SHUTDOWNS.lock() {
            shutdowns.remove(&server_id);
        }
    }) {
        if let Ok(mut servers) = SERVERS.lock() {
            servers.remove(&server_id);
        }
        SERVER_RECEIVERS.blocking_lock().remove(&server_id);
        if let Ok(mut shutdowns) = SERVER_SHUTDOWNS.lock() {
            shutdowns.remove(&server_id);
        }
        return Deger::Hata(format!("dahili_sunucu_baslat: {error}"));
    }
    Deger::Sayi(server_id as f64)
}

fn server_close(args: Vec<Deger>) -> Deger {
    if let Some(error) = capability_error(Capability::NetworkServer, "dahili_sunucu_kapat") {
        return error;
    }
    let server_id = match args.as_slice() {
        [Deger::Sayi(id)] => match numeric_id(*id, "dahili_sunucu_kapat") {
            Ok(id) => id,
            Err(error) => return Deger::Hata(error),
        },
        [other] => {
            return Deger::Hata(format!(
                "dahili_sunucu_kapat: geçerli sunucu kimliği bekleniyordu; {other} geldi"
            ))
        }
        _ => {
            return Deger::Hata(format!(
                "dahili_sunucu_kapat: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            ))
        }
    };
    let shutdown = match SERVER_SHUTDOWNS.lock() {
        Ok(mut shutdowns) => shutdowns.remove(&server_id),
        Err(_) => {
            return Deger::Hata("dahili_sunucu_kapat: kapatma kayıt kilidi bozuldu".to_string())
        }
    };
    let Some(shutdown) = shutdown else {
        return Deger::Hata(format!(
            "dahili_sunucu_kapat: {server_id} kimlikli sunucu bulunamadı"
        ));
    };
    if let Ok(mut servers) = SERVERS.lock() {
        servers.remove(&server_id);
    }
    SERVER_RECEIVERS.blocking_lock().remove(&server_id);
    let _ = shutdown.send(());
    Deger::Sayi(1.0)
}

async fn handle_incoming_request(
    server_id: u64,
    request: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    let url = request.uri().to_string();
    if url.len() > MAX_TARGET_BYTES {
        return Ok(simple_response(414, "istek hedefi çok uzun"));
    }
    let method = request.method().to_string();
    let mut body = request.into_body();
    let mut bytes = Vec::new();
    while let Some(chunk) = body.next().await {
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(_) => return Ok(simple_response(400, "istek gövdesi okunamadı")),
        };
        if bytes
            .len()
            .checked_add(chunk.len())
            .is_none_or(|length| length > MAX_BODY_BYTES)
        {
            return Ok(simple_response(413, "istek gövdesi çok büyük"));
        }
        bytes.extend_from_slice(&chunk);
    }
    let body = match String::from_utf8(bytes) {
        Ok(body) => body,
        Err(_) => return Ok(simple_response(415, "istek gövdesi geçerli UTF-8 değil")),
    };
    let request_id = match next_id() {
        Ok(id) => id,
        Err(error) => return Ok(simple_response(503, &error)),
    };
    let (respond_to, response) = oneshot::channel();
    let sender = SERVERS
        .lock()
        .ok()
        .and_then(|servers| servers.get(&server_id).cloned());
    let Some(sender) = sender else {
        return Ok(simple_response(503, "sunucu kuyruğu kullanılamıyor"));
    };
    if sender
        .send(IncomingRequest {
            id: request_id,
            url,
            method,
            body,
            respond_to,
        })
        .await
        .is_err()
    {
        return Ok(simple_response(503, "sunucu kuyruğu kullanılamıyor"));
    }
    match tokio::time::timeout(Duration::from_secs(30), response).await {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(_)) => Ok(simple_response(500, "yanıtlayıcı kapandı")),
        Err(_) => {
            if let Ok(mut responses) = RESPONSES.lock() {
                responses.remove(&request_id);
            }
            Ok(simple_response(504, "yanıt zaman aşımına uğradı"))
        }
    }
}

fn simple_response(status: u16, message: &str) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(Body::from(message.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from(message.to_string())))
}

fn server_wait(args: Vec<Deger>) -> Deger {
    if let Some(error) = capability_error(Capability::NetworkServer, "dahili_sunucu_bekle") {
        return error;
    }
    let server_id = match args.as_slice() {
        [Deger::Sayi(id)] => match numeric_id(*id, "dahili_sunucu_bekle") {
            Ok(id) => id,
            Err(error) => return Deger::Hata(error),
        },
        [other] => {
            return Deger::Hata(format!(
                "dahili_sunucu_bekle: geçerli sunucu kimliği bekleniyordu; {other} geldi"
            ))
        }
        _ => {
            return Deger::Hata(format!(
                "dahili_sunucu_bekle: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            ))
        }
    };
    spawn_task(async move {
        let mut receivers = SERVER_RECEIVERS.lock().await;
        let receiver = match receivers.get_mut(&server_id) {
            Some(receiver) => receiver,
            None => {
                return Deger::Hata(format!(
                    "dahili_sunucu_bekle: bilinmeyen sunucu kimliği: {server_id}"
                ))
            }
        };
        let incoming = match receiver.recv().await {
            Some(incoming) => incoming,
            None => {
                return Deger::Hata("dahili_sunucu_bekle: sunucu istek kanalı kapandı".to_string())
            }
        };
        match RESPONSES.lock() {
            Ok(mut responses) if responses.len() < MAX_PENDING_REQUESTS => {
                responses.insert(incoming.id, incoming.respond_to);
            }
            Ok(_) => {
                return Deger::Hata(format!(
                    "dahili_sunucu_bekle: en fazla {MAX_PENDING_REQUESTS} yanıt bekleyebilir"
                ))
            }
            Err(_) => return Deger::Hata("dahili_sunucu_bekle: yanıt kilidi bozuldu".to_string()),
        }
        Deger::Nesne {
            sinif_adi: "İstek".to_string(),
            alanlar: Gc::new(HashMap::from([
                ("id".to_string(), Deger::Sayi(incoming.id as f64)),
                ("url".to_string(), Deger::Metin(incoming.url)),
                ("metot".to_string(), Deger::Metin(incoming.method)),
                ("gövde".to_string(), Deger::Metin(incoming.body)),
            ])),
            module_kimligi: None,
        }
    })
}

fn server_respond(args: Vec<Deger>) -> Deger {
    if let Some(error) = capability_error(Capability::NetworkServer, "dahili_sunucu_yanitla") {
        return error;
    }
    if !(2..=5).contains(&args.len()) {
        return Deger::Hata(format!(
            "dahili_sunucu_yanitla: 2 ile 5 arasında argüman bekleniyordu; {} geldi",
            args.len()
        ));
    }
    let request_id = match &args[0] {
        Deger::Sayi(id) => match numeric_id(*id, "dahili_sunucu_yanitla") {
            Ok(id) => id,
            Err(error) => return Deger::Hata(error),
        },
        other => {
            return Deger::Hata(format!(
                "dahili_sunucu_yanitla: geçerli istek kimliği bekleniyordu; {other} geldi"
            ))
        }
    };
    let data = match &args[1] {
        Deger::Metin(text) => text.as_bytes().to_vec(),
        Deger::Bayt(bytes) => bytes.clone(),
        other => {
            return Deger::Hata(format!(
                "dahili_sunucu_yanitla: içerik metin veya bayt olmalıdır; {other} geldi"
            ))
        }
    };
    if data.len() > MAX_BODY_BYTES {
        return Deger::Hata(format!(
            "dahili_sunucu_yanitla: içerik {MAX_BODY_BYTES} bayt sınırını aşıyor"
        ));
    }
    let status = match args.get(2) {
        Some(Deger::Sayi(status))
            if status.is_finite()
                && status.fract() == 0.0
                && (100.0..=599.0).contains(status) =>
        {
            *status as u16
        }
        Some(other) => {
            return Deger::Hata(format!(
                "dahili_sunucu_yanitla: HTTP durum kodu 100..599 arasında tamsayı olmalıdır; {other} geldi"
            ))
        }
        None => 200,
    };
    let content_type = match args.get(3) {
        Some(Deger::Metin(value)) => value.as_str(),
        Some(other) => {
            return Deger::Hata(format!(
                "dahili_sunucu_yanitla: içerik türü metin olmalıdır; {other} geldi"
            ))
        }
        None => "text/html; charset=utf-8",
    };
    let mut builder = Response::builder()
        .status(status)
        .header("content-type", content_type);
    if let Some(headers) = args.get(4) {
        let fields = match headers {
            Deger::Nesne { alanlar, .. } | Deger::Sozluk(alanlar) => match alanlar.try_borrow() {
                Ok(fields) => fields,
                Err(_) => {
                    return Deger::Hata("dahili_sunucu_yanitla: başlıklar kullanımda".to_string())
                }
            },
            other => {
                return Deger::Hata(format!(
                    "dahili_sunucu_yanitla: başlıklar nesne veya sözlük olmalıdır; {other} geldi"
                ))
            }
        };
        if fields.len() > MAX_HEADERS {
            return Deger::Hata(format!(
                "dahili_sunucu_yanitla: en fazla {MAX_HEADERS} başlık desteklenir"
            ));
        }
        for (key, value) in fields.iter() {
            let Deger::Metin(value) = value else {
                return Deger::Hata(format!(
                    "dahili_sunucu_yanitla: '{key}' başlık değeri metin olmalıdır"
                ));
            };
            builder = builder.header(key.as_str(), value.as_str());
        }
    }
    let response = match builder.body(Body::from(data)) {
        Ok(response) => response,
        Err(error) => {
            return Deger::Hata(format!(
                "dahili_sunucu_yanitla: HTTP yanıtı oluşturulamadı: {error}"
            ))
        }
    };
    let sender = match RESPONSES.lock() {
        Ok(mut responses) => responses.remove(&request_id),
        Err(_) => return Deger::Hata("dahili_sunucu_yanitla: yanıt kilidi bozuldu".to_string()),
    };
    let Some(sender) = sender else {
        return Deger::Hata(format!(
            "dahili_sunucu_yanitla: bilinmeyen veya yanıtlanmış istek kimliği: {request_id}"
        ));
    };
    if sender.send(response).is_err() {
        return Deger::Hata("dahili_sunucu_yanitla: istemci bağlantısı kapandı".to_string());
    }
    Deger::Sayi(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kimlik_ve_url_girdileri_sessizce_yutulmaz() {
        assert!(numeric_id(f64::NAN, "test").is_err());
        let value = http_request(vec![
            Deger::Metin("GET".to_string()),
            Deger::Metin("file:///etc/passwd".to_string()),
        ]);
        assert!(matches!(value, Deger::Hata(_)));
        assert!(matches!(
            server_close(vec![Deger::Sayi(f64::NAN)]),
            Deger::Hata(_)
        ));
    }
}
