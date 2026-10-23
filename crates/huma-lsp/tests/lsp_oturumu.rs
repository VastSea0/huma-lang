//! `huma-lsp` ikilisiyle gerçek bir JSON-RPC oturumu: tanılar, tanıma gitme
//! ve hover uçtan uca doğrulanır.

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct Oturum {
    child: Child,
    stdin: Option<ChildStdin>,
    mesajlar: Receiver<Value>,
}

impl Oturum {
    fn baslat(kok: &std::path::Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_huma-lsp"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("huma-lsp başlatılmalı");
        let stdout = child.stdout.take().expect("stdout alınmalı");
        let (gonderici, mesajlar) = mpsc::channel();
        std::thread::spawn(move || {
            let mut okuyucu = BufReader::new(stdout);
            loop {
                let mut uzunluk = None;
                loop {
                    let mut satir = String::new();
                    if okuyucu.read_line(&mut satir).unwrap_or(0) == 0 {
                        return;
                    }
                    let satir = satir.trim_end();
                    if satir.is_empty() {
                        break;
                    }
                    if let Some(deger) = satir.strip_prefix("Content-Length: ") {
                        uzunluk = deger.parse::<usize>().ok();
                    }
                }
                let Some(uzunluk) = uzunluk else { return };
                let mut govde = vec![0; uzunluk];
                if okuyucu.read_exact(&mut govde).is_err() {
                    return;
                }
                let Ok(mesaj) = serde_json::from_slice(&govde) else {
                    return;
                };
                if gonderici.send(mesaj).is_err() {
                    return;
                }
            }
        });
        let stdin = child.stdin.take();
        let mut oturum = Self {
            child,
            stdin,
            mesajlar,
        };
        let kok_uri = format!("file://{}", kok.display());
        oturum.gonder(json!({"jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"processId": null, "rootUri": kok_uri, "capabilities": {}}}));
        oturum.bekle(|m| m["id"] == 1);
        oturum.gonder(json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}));
        oturum
    }

    fn gonder(&mut self, mesaj: Value) {
        let govde = mesaj.to_string();
        let stdin = self.stdin.as_mut().expect("stdin açık olmalı");
        write!(stdin, "Content-Length: {}\r\n\r\n{}", govde.len(), govde)
            .and_then(|()| stdin.flush())
            .expect("LSP'ye yazılmalı");
    }

    fn bekle(&self, kosul: impl Fn(&Value) -> bool) -> Value {
        loop {
            let mesaj = self
                .mesajlar
                .recv_timeout(Duration::from_secs(20))
                .expect("LSP 20 saniye içinde yanıt vermeli");
            if kosul(&mesaj) {
                return mesaj;
            }
        }
    }

    fn ac(&mut self, uri: &str, text: &str) {
        self.gonder(json!({"jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {"textDocument": {"uri": uri, "languageId": "huma", "version": 1, "text": text}}}));
    }

    fn istek(&mut self, id: u64, method: &str, uri: &str, satir: u32, karakter: u32) -> Value {
        self.gonder(json!({"jsonrpc": "2.0", "id": id, "method": method,
            "params": {"textDocument": {"uri": uri}, "position": {"line": satir, "character": karakter}}}));
        self.bekle(|m| m["id"] == id)["result"].clone()
    }
}

impl Drop for Oturum {
    fn drop(&mut self) {
        // Editörler gibi stdin'i kapatır; sunucu EOF'ta sonlanır.
        drop(self.stdin.take());
        let _ = self.child.wait();
    }
}

fn gecici_kok() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("saat geçerli olmalı")
        .as_nanos();
    let kok = std::env::temp_dir().join(format!("huma_lsp_{}_{nonce}", std::process::id()));
    std::fs::create_dir_all(&kok).expect("geçici kök oluşturulmalı");
    kok
}

#[test]
fn tanilar_tanim_ve_hover_belirlenimli_calisir() {
    let kok = gecici_kok();
    std::fs::write(
        kok.join("yardimci.hb"),
        "yardımcı fonksiyon olsun n alsın {\n    n'yi döndür\n}\n",
    )
    .expect("çalışma alanı dosyası yazılmalı");
    let a = format!("file://{}/a.hb", kok.display());
    let b = format!("file://{}/b.hb", kok.display());
    let mut oturum = Oturum::baslat(&kok);

    // Aynı adı tanımlayan başka bir açık belge: geçerli belge önceliklidir.
    oturum.ac(&b, "topla fonksiyon olsun x alsın { x'i döndür }\n");
    let kaynak = "topla fonksiyon olsun a, b alsın {\n    (a + b)'yi döndür\n}\nsonuç = topla(1, yardımcı(2)) olsun\nsonuç'u yazdır\nx = (\n";
    oturum.ac(&a, kaynak);
    let tanilar = oturum.bekle(|m| {
        m["method"] == "textDocument/publishDiagnostics" && m["params"]["uri"] == a.as_str()
    });
    let tanilar = tanilar["params"]["diagnostics"]
        .as_array()
        .expect("tanı listesi olmalı");
    assert!(!tanilar.is_empty(), "sözdizimi hatası raporlanmalı");
    assert_eq!(tanilar[0]["range"]["start"]["line"], 6, "{tanilar:?}");

    for id in [2, 3, 4] {
        let tanim = oturum.istek(id, "textDocument/definition", &a, 3, 10);
        assert_eq!(
            tanim["uri"],
            a.as_str(),
            "her denemede geçerli belge: {tanim}"
        );
        assert_eq!(tanim["range"]["start"], json!({"line": 0, "character": 0}));
        assert_eq!(tanim["range"]["end"], json!({"line": 0, "character": 5}));
    }

    let degisken = oturum.istek(5, "textDocument/definition", &a, 4, 2);
    assert_eq!(degisken["uri"], a.as_str());
    assert_eq!(
        degisken["range"]["start"],
        json!({"line": 3, "character": 0})
    );

    let calisma_alani = oturum.istek(6, "textDocument/definition", &a, 3, 20);
    assert!(
        calisma_alani["uri"]
            .as_str()
            .is_some_and(|uri| uri.ends_with("/yardimci.hb")),
        "{calisma_alani}"
    );

    let hover = oturum.istek(7, "textDocument/hover", &a, 3, 10);
    let hover = hover["contents"]["value"].as_str().unwrap_or_default();
    assert!(hover.contains("**topla** *(fonksiyon)*"), "{hover}");
    assert!(hover.contains("Parametreler: `a, b`"), "{hover}");

    drop(oturum);
    std::fs::remove_dir_all(&kok).expect("geçici kök temizlenmeli");
}
