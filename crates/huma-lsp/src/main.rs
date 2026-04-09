use anyhow::Result;
use huma_core::{HumaError};
use huma_core::lexer::Lexer;
use huma_core::parser::Parser;
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
    fn uri_to_path(uri: &Url) -> Option<PathBuf> {
        uri.to_file_path().ok()
    }

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
        let col = pos.character as usize;
        let bytes = line.as_bytes();
        if col > bytes.len() {
            return None;
        }

        let is_word = |b: u8| {
            (b as char).is_alphanumeric() || b == b'_' || b >= 0x80
        };

        let mut start = col;
        while start > 0 && is_word(bytes[start - 1]) {
            start -= 1;
        }
        let mut end = col;
        while end < bytes.len() && is_word(bytes[end]) {
            end += 1;
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
                            start: Position { line: l, character: c },
                            end: Position { line: l, character: c + 1 },
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
        // Very simple heuristic: find "<name> fonksiyon olsun" in builtin lib files.
        let re = Regex::new(r#"(?m)^\s*([A-Za-z_ÇĞİÖŞÜçğıöşü][\wÇĞİÖŞÜçğıöşü]*)\s+fonksiyon\s+olsun"#).ok();
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
            .or_else(|| params.workspace_folders.and_then(|mut w| w.pop()).and_then(|f| f.uri.to_file_path().ok()));
        if let Some(r) = root {
            *self.root.write().await = Some(r);
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                definition_provider: Some(OneOf::Left(true)),
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
        self.docs.lock().await.insert(uri.clone(), Document { text: text.clone() });

        let diags = Self::diagnostics_for(&text);
        self.client.publish_diagnostics(uri, diags, None).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        if let Some(change) = params.content_changes.into_iter().last() {
            let text = change.text;
            self.docs.lock().await.insert(uri.clone(), Document { text: text.clone() });
            let diags = Self::diagnostics_for(&text);
            self.client.publish_diagnostics(uri, diags, None).await;
        }
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> LspResult<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let docs = self.docs.lock().await;
        let Some(doc) = docs.get(&uri) else { return Ok(None); };
        let Some(word) = Self::word_at_position(&doc.text, pos) else { return Ok(None); };

        // 1) Search opened documents first
        let def_re = Regex::new(&format!(r#"(?m)^\s*{}\s+fonksiyon\b"#, regex::escape(&word))).ok();
        if let Some(def_re) = def_re {
            for (u, d) in docs.iter() {
                for (idx, line) in d.text.lines().enumerate() {
                    if def_re.is_match(line) {
                        let loc = Location {
                            uri: u.clone(),
                            range: Range {
                                start: Position { line: idx as u32, character: 0 },
                                end: Position { line: idx as u32, character: line.len() as u32 },
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
        let Some(root) = root_guard.as_ref() else { return Ok(None); };
        let files = Self::scan_hb_files(&root);
        let def_re = Regex::new(&format!(r#"(?m)^\s*{}\s+fonksiyon\b"#, regex::escape(&word))).ok();
        if let Some(def_re) = def_re {
            for p in files {
                if let Ok(text) = std::fs::read_to_string(&p) {
                    for (idx, line) in text.lines().enumerate() {
                        if def_re.is_match(line) {
                            if let Ok(u) = Url::from_file_path(&p) {
                                let loc = Location {
                                    uri: u,
                                    range: Range {
                                        start: Position { line: idx as u32, character: 0 },
                                        end: Position { line: idx as u32, character: line.len() as u32 },
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
        let names = Self::collect_builtin_function_names();
        let items = names
            .into_iter()
            .map(|n| CompletionItem {
                label: n.clone(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("stdlib".to_string()),
                insert_text: Some(n),
                ..Default::default()
            })
            .collect::<Vec<_>>();

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

