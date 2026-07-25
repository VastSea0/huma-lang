use anyhow::Result;
use huma_core::lexer::Lexer;
use huma_core::parser::Parser;
use huma_core::HumaError;
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use walkdir::WalkDir;

#[derive(Debug, Default)]
struct Document {
    text: String,
}

#[derive(Debug)]
struct Backend {
    client: Client,
    root: tokio::sync::RwLock<Option<PathBuf>>,
    docs: tokio::sync::Mutex<HashMap<Url, Document>>,
}

impl Backend {
    fn scan_hb_files(root: &Path) -> Vec<PathBuf> {
        WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.into_path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("hb"))
            .collect()
    }

    fn word_at_position(text: &str, pos: Position) -> Option<String> {
        let line = text.lines().nth(pos.line as usize)?;
        let target_utf16 = pos.character as usize;
        let mut utf16_offset = 0;
        let mut byte_offset = None;
        for (index, ch) in line.char_indices() {
            if utf16_offset == target_utf16 {
                byte_offset = Some(index);
                break;
            }
            let next_offset = utf16_offset + ch.len_utf16();
            if target_utf16 < next_offset {
                return None;
            }
            utf16_offset = next_offset;
        }
        let byte_offset =
            byte_offset.or_else(|| (utf16_offset == target_utf16).then_some(line.len()))?;

        let is_word = |ch: char| ch.is_alphanumeric() || ch == '_';

        let mut start = byte_offset;
        while let Some((index, ch)) = line[..start].char_indices().next_back() {
            if !is_word(ch) {
                break;
            }
            start = index;
        }

        let mut end = byte_offset;
        for (relative_index, ch) in line[byte_offset..].char_indices() {
            if !is_word(ch) {
                break;
            }
            end = byte_offset + relative_index + ch.len_utf8();
        }
        if start == end {
            return None;
        }
        Some(line[start..end].to_string())
    }

    fn diagnostics_for(text: &str) -> Vec<Diagnostic> {
        let lexer = Lexer::new(text);
        let mut parser = Parser::new(lexer);
        let (_program, errors) = parser.parse_program_with_diagnostics();

        errors
            .into_iter()
            .filter_map(|e| match e {
                HumaError::SyntaxError { line, col, message } => {
                    let l = line.saturating_sub(1) as u32;
                    let c = col.saturating_sub(1) as u32;
                    Some(Diagnostic {
                        range: Range {
                            start: Position {
                                line: l,
                                character: c,
                            },
                            end: Position {
                                line: l,
                                character: c + 1,
                            },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: None,
                        code_description: None,
                        source: Some("huma".to_string()),
                        message,
                        related_information: None,
                        tags: None,
                        data: None,
                    })
                }
                _ => None,
            })
            .collect()
    }

    fn collect_builtin_function_names() -> Vec<String> {
        let re =
            Regex::new(r#"(?m)^\s*([A-Za-z_ÇĞİÖŞÜçğıöşü][\wÇĞİÖŞÜçğıöşü]*)\s+fonksiyon\s+olsun"#)
                .ok();
        let mut out = Vec::new();
        if let Some(re) = re {
            for (_name, content) in huma_core::builtin_files::get_lib_files() {
                for cap in re.captures_iter(content) {
                    if let Some(m) = cap.get(1) {
                        out.push(m.as_str().to_string());
                    }
                }
            }
        }
        out.sort();
        out.dedup();
        out
    }

    /// Returns a Turkish documentation string for known Hüma keywords.
    fn keyword_hover(word: &str) -> Option<String> {
        let doc = match word {
            "yazdır"    => "**yazdır** — Bir değeri ekrana yazar.\n\n```hüma\n\"Merhaba\"'yı yazdır\n```",
            "olsun"     => "**olsun** — Değişken tanımlar veya değer atar.\n\n```hüma\nisim = \"Hüma\" olsun\n```",
            "fonksiyon" => "**fonksiyon** — Yeni bir fonksiyon tanımlar.\n\n```hüma\nselamla fonksiyon olsun isim alsın {\n    \"Merhaba \" + isim'i yazdır\n}\n```",
            "ise"       => "**ise** — Koşul ifadesi (if).\n\n```hüma\nx > 0 ise { \"Pozitif\"'i yazdır }\n```",
            "yoksa"     => "**yoksa** — Koşul karşılanmadığında çalışır (else).",
            "döndür"    => "**döndür** — Fonksiyondan değer döndürür.\n\n```hüma\nsonuç'u döndür\n```",
            "yükle"     => "**yükle** — Bir kütüphane veya modülü yükler (postfix).\n\n```hüma\n\"matematik.hb\"'yi yükle\n```",
            "ile"       => "**ile** — Fonksiyon argümanlarını bağlar (with).\n\n```hüma\n10 ile 20'yi topla\n```",
            "çağır"     => "**çağır** — Bir fonksiyonu doğrudan çağırır.\n\n```hüma\nçağır hesapla(5)\n```",
            "bekle"     => "**bekle** — Asenkron işlem sonucunu bekler (await).",
            "kadar"     => "**kadar** — Aralık döngüsü üst sınırını belirtir.\n\n```hüma\ni = 0'dan 10'a kadar { ... }\n```",
            "dene"      => "**dene** — Hata yakalama bloğu başlatır (try).",
            "yakala"    => "**yakala** — Dene bloğundaki hatayı yakalar (catch).",
            "sınıf"     => "**sınıf** — Yeni bir sınıf tanımlar.\n\n```hüma\nKişi sınıf olsun { ... }\n```",
            "liste"     => "**liste** — Boş bir liste oluşturur.\n\n```hüma\nsayılar liste olsun\n```",
            "ekle"      => "**ekle** — Listeye eleman ekler.\n\n```hüma\nsayılar'a 5'i ekle\n```",
            "kendisi"   => "**kendisi** — Sınıf içinde mevcut örneğe erişir (self/this).",
            "doğru"     => "**doğru** — Boolean true değeri.",
            "yanlış"    => "**yanlış** — Boolean false değeri.",
            "ve"        => "**ve** — Mantıksal VE (AND) operatörü.",
            "veya"      => "**veya** — Mantıksal VEYA (OR) operatörü.",
            "değil"     => "**değil** — Mantıksal DEĞİL (NOT) operatörü.",
            _ => return None,
        };
        Some(doc.to_string())
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> LspResult<InitializeResult> {
        self.client
            .log_message(MessageType::INFO, "Hüma LSP başlatıldı.")
            .await;

        let root = params
            .root_uri
            .and_then(|u| u.to_file_path().ok())
            .or_else(|| {
                params
                    .workspace_folders
                    .and_then(|mut w| w.pop())
                    .and_then(|f| f.uri.to_file_path().ok())
            });
        if let Some(r) = root {
            *self.root.write().await = Some(r);
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                definition_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), "'".to_string()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "huma-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Hüma LSP hazır.")
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text;
        self.docs
            .lock()
            .await
            .insert(uri.clone(), Document { text: text.clone() });

        let diags = Self::diagnostics_for(&text);
        self.client.publish_diagnostics(uri, diags, None).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        if let Some(change) = params.content_changes.into_iter().last() {
            let text = change.text;
            self.docs
                .lock()
                .await
                .insert(uri.clone(), Document { text: text.clone() });
            let diags = Self::diagnostics_for(&text);
            self.client.publish_diagnostics(uri, diags, None).await;
        }
    }

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let docs = self.docs.lock().await;
        let Some(doc) = docs.get(&uri) else {
            return Ok(None);
        };
        let Some(word) = Self::word_at_position(&doc.text, pos) else {
            return Ok(None);
        };
        drop(docs);

        // 1) Check built-in keyword docs
        if let Some(doc_str) = Self::keyword_hover(&word) {
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: doc_str,
                }),
                range: None,
            }));
        }

        // 2) Look for function definition in workspace
        let root_guard = self.root.read().await;
        if let Some(root) = root_guard.as_ref() {
            let files = Self::scan_hb_files(root);
            let def_re = Regex::new(&format!(
                r#"(?m)^\s*{}\s+fonksiyon\s+olsun(?:\s+(\w[\wÇĞİÖŞÜçğıöşü,\s]*)\s+alsın)?"#,
                regex::escape(&word)
            ))
            .ok();
            if let Some(def_re) = def_re {
                for p in files {
                    if let Ok(text) = std::fs::read_to_string(&p) {
                        if let Some(cap) = def_re.captures_iter(&text).next() {
                            let params_str = cap
                                .get(1)
                                .map(|m| m.as_str().to_string())
                                .unwrap_or_default();
                            let hover_text = if params_str.is_empty() {
                                format!("**{}** *(fonksiyon)*", word)
                            } else {
                                format!(
                                    "**{}** *(fonksiyon)*\n\nParametreler: `{}`",
                                    word, params_str
                                )
                            };
                            return Ok(Some(Hover {
                                contents: HoverContents::Markup(MarkupContent {
                                    kind: MarkupKind::Markdown,
                                    value: hover_text,
                                }),
                                range: None,
                            }));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> LspResult<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let docs = self.docs.lock().await;
        let Some(doc) = docs.get(&uri) else {
            return Ok(None);
        };
        let Some(word) = Self::word_at_position(&doc.text, pos) else {
            return Ok(None);
        };

        // 1) Search opened documents first
        let def_re = Regex::new(&format!(
            r#"(?m)^\s*{}\s+fonksiyon\b"#,
            regex::escape(&word)
        ))
        .ok();
        if let Some(def_re) = def_re {
            for (u, d) in docs.iter() {
                for (idx, line) in d.text.lines().enumerate() {
                    if def_re.is_match(line) {
                        let loc = Location {
                            uri: u.clone(),
                            range: Range {
                                start: Position {
                                    line: idx as u32,
                                    character: 0,
                                },
                                end: Position {
                                    line: idx as u32,
                                    character: line.len() as u32,
                                },
                            },
                        };
                        return Ok(Some(GotoDefinitionResponse::Scalar(loc)));
                    }
                }
            }
        }
        drop(docs);

        // 2) Search workspace files
        let root_guard = self.root.read().await;
        let Some(root) = root_guard.as_ref() else {
            return Ok(None);
        };
        let files = Self::scan_hb_files(root);
        let def_re = Regex::new(&format!(
            r#"(?m)^\s*{}\s+fonksiyon\b"#,
            regex::escape(&word)
        ))
        .ok();
        if let Some(def_re) = def_re {
            for p in files {
                if let Ok(text) = std::fs::read_to_string(&p) {
                    for (idx, line) in text.lines().enumerate() {
                        if def_re.is_match(line) {
                            if let Ok(u) = Url::from_file_path(&p) {
                                let loc = Location {
                                    uri: u,
                                    range: Range {
                                        start: Position {
                                            line: idx as u32,
                                            character: 0,
                                        },
                                        end: Position {
                                            line: idx as u32,
                                            character: line.len() as u32,
                                        },
                                    },
                                };
                                return Ok(Some(GotoDefinitionResponse::Scalar(loc)));
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    async fn completion(&self, _params: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        let mut items = Vec::new();

        // 1) Keywords with documentation
        let keywords: Vec<(&str, &str)> = vec![
            ("yazdır", "Bir değeri ekrana yazar"),
            ("olsun", "Değişken tanımlar veya değer atar"),
            ("alsın", "Fonksiyon parametrelerini tanımlar"),
            ("fonksiyon", "Yeni bir fonksiyon tanımlar"),
            ("sınıf", "Yeni bir sınıf tanımlar"),
            ("ise", "Koşul ifadesi (if)"),
            ("yoksa", "Koşul karşılanmadığında (else)"),
            ("olduğu", "'olduğu sürece' döngüsünün parçası"),
            ("sürece", "'olduğu sürece' döngüsünün parçası"),
            ("döndür", "Fonksiyondan değer döndürür"),
            ("ve", "Mantıksal VE (AND)"),
            ("veya", "Mantıksal VEYA (OR)"),
            ("değil", "Mantıksal DEĞİL (NOT)"),
            ("yükle", "Modül yükler: \"lib.hb\"'yi yükle"),
            ("liste", "Boş liste oluşturur"),
            ("ekle", "Listeye eleman ekler"),
            ("çıkar", "Listeden eleman çıkarır"),
            ("uzunluğu", "Boyutu döndürür"),
            ("kendisi", "Sınıf içi öz-referans (self)"),
            ("doğru", "Boolean true"),
            ("yanlış", "Boolean false"),
            ("dene", "Hata yakalama bloğu (try)"),
            ("yakala", "Hata yakalar (catch)"),
            ("var", "ile var: Boyunca gezinir"),
            ("kadar", "Aralık döngüsü üst sınırı"),
            ("mi", "Soru eki"),
            ("ile", "Fonksiyon argümanını bağlar (with)"),
            ("bekle", "Asenkron bekler (await)"),
            ("çağır", "Fonksiyon çağırır"),
        ];

        for (kw, desc) in keywords {
            items.push(CompletionItem {
                label: kw.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(desc.to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**{}** — Hüma anahtar kelimesi\n\n{}", kw, desc),
                })),
                ..Default::default()
            });
        }

        // 2) Built-in Functions
        let names = Self::collect_builtin_function_names();
        for n in names {
            items.push(CompletionItem {
                label: n.clone(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Standart Kütüphane".to_string()),
                insert_text: Some(n),
                ..Default::default()
            });
        }

        Ok(Some(CompletionResponse::Array(items)))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        root: tokio::sync::RwLock::new(None),
        docs: tokio::sync::Mutex::new(HashMap::new()),
    });

    Server::new(stdin, stdout, socket).serve(service).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Backend;
    use tower_lsp::lsp_types::Position;

    #[test]
    fn turkce_sozcugu_utf16_konumundan_bulur() {
        let text = "öğrenci_sayısı'nı yazdır";
        let word = Backend::word_at_position(
            text,
            Position {
                line: 0,
                character: 5,
            },
        );
        assert_eq!(word.as_deref(), Some("öğrenci_sayısı"));
    }

    #[test]
    fn emoji_sonrasindaki_utf16_konumunu_dogru_cevirir() {
        let text = "🙂 değer'i yazdır";
        let word = Backend::word_at_position(
            text,
            Position {
                line: 0,
                character: 5,
            },
        );
        assert_eq!(word.as_deref(), Some("değer"));
    }

    #[test]
    fn ayrac_uzerinde_sozcuk_dondurmez() {
        let word = Backend::word_at_position(
            "ad = 1 olsun",
            Position {
                line: 0,
                character: 3,
            },
        );
        assert!(word.is_none());
    }
}
