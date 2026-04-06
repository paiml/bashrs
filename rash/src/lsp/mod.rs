//! bashrs Language Server Protocol implementation (Sprint 8)
//!
//! Provides real-time lint diagnostics and auto-fix code actions to editors via LSP.
//! Supports VS Code, Neovim, Helix, Zed, and any LSP-compatible editor.
//!
//! ## Features
//!
//! - **Diagnostics**: Real-time lint on open/save/change (all 487+ rules)
//! - **Code Actions**: Quick Fix for diagnostics with auto-fix (Safe + SafeWithAssumptions)
//! - **File Detection**: Automatic shell/Makefile/Dockerfile detection
//!
//! ## Usage
//!
//! ```bash
//! bashrs lsp          # Start LSP server on stdio
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "lsp_tests.rs"]
// FIXME(PMAT-238): mod tests;

/// Per-document state: bashrs diagnostics with fix data.
type DocDiagnostics = HashMap<Url, Vec<crate::linter::Diagnostic>>;

/// bashrs LSP backend.
pub struct BashrsLsp {
    client: Client,
    /// Stored bashrs diagnostics per document (needed for code action fix lookup).
    doc_diagnostics: Arc<RwLock<DocDiagnostics>>,
}

impl BashrsLsp {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            doc_diagnostics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Lint a document and publish diagnostics + store bashrs diagnostics for code actions.
    async fn lint_and_publish(&self, uri: Url, text: &str) {
        let bashrs_diags = lint_document(text, &uri);
        let lsp_diags: Vec<Diagnostic> = bashrs_diags.iter().map(to_lsp_diagnostic).collect();

        // Store bashrs diagnostics for code action lookups
        self.doc_diagnostics
            .write()
            .await
            .insert(uri.clone(), bashrs_diags);

        self.client.publish_diagnostics(uri, lsp_diags, None).await;
    }
}

/// Convert bashrs Severity to LSP DiagnosticSeverity.
fn to_lsp_severity(severity: crate::linter::Severity) -> DiagnosticSeverity {
    match severity {
        crate::linter::Severity::Error => DiagnosticSeverity::ERROR,
        crate::linter::Severity::Warning | crate::linter::Severity::Risk => {
            DiagnosticSeverity::WARNING
        }
        crate::linter::Severity::Perf | crate::linter::Severity::Note => {
            DiagnosticSeverity::INFORMATION
        }
        crate::linter::Severity::Info => DiagnosticSeverity::HINT,
    }
}

/// Convert bashrs Span to LSP Range (0-indexed).
fn to_lsp_range(span: crate::linter::Span) -> Range {
    Range {
        start: Position {
            line: span.start_line.saturating_sub(1) as u32,
            character: span.start_col.saturating_sub(1) as u32,
        },
        end: Position {
            line: span.end_line.saturating_sub(1) as u32,
            character: span.end_col.saturating_sub(1) as u32,
        },
    }
}

/// Convert a bashrs Diagnostic to an LSP Diagnostic.
fn to_lsp_diagnostic(diag: &crate::linter::Diagnostic) -> Diagnostic {
    Diagnostic {
        range: to_lsp_range(diag.span),
        severity: Some(to_lsp_severity(diag.severity)),
        code: Some(NumberOrString::String(diag.code.clone())),
        code_description: None,
        source: Some("bashrs".to_string()),
        message: diag.message.clone(),
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Convert a bashrs Fix into an LSP CodeAction.
fn to_code_action(
    diag: &crate::linter::Diagnostic,
    uri: &Url,
) -> Option<CodeActionOrCommand> {
    let fix = diag.fix.as_ref()?;

    // Only offer code actions for Safe and SafeWithAssumptions fixes
    if fix.replacement.is_empty() {
        return None;
    }

    let safety_label = match fix.safety_level {
        crate::linter::FixSafetyLevel::Safe => "",
        crate::linter::FixSafetyLevel::SafeWithAssumptions => " (with assumptions)",
        crate::linter::FixSafetyLevel::Unsafe => return None, // Never auto-apply unsafe
    };

    let title = format!(
        "Fix {}{}: {}",
        diag.code,
        safety_label,
        truncate(&fix.replacement, 60),
    );

    let range = to_lsp_range(diag.span);
    let edit = TextEdit {
        range,
        new_text: fix.replacement.clone(),
    };

    let mut changes = HashMap::new();
    changes.insert(uri.clone(), vec![edit]);

    let action = CodeAction {
        title,
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![to_lsp_diagnostic(diag)]),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            ..WorkspaceEdit::default()
        }),
        is_preferred: Some(fix.safety_level == crate::linter::FixSafetyLevel::Safe),
        ..CodeAction::default()
    };

    Some(CodeActionOrCommand::CodeAction(action))
}

/// Truncate a string for display.
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

/// Run the linter on source text and return bashrs Diagnostics (with Fix data).
fn lint_document(text: &str, uri: &Url) -> Vec<crate::linter::Diagnostic> {
    let path_str = uri.path();

    let result = if path_str.ends_with("Makefile")
        || path_str.ends_with(".mk")
        || path_str.ends_with("GNUmakefile")
    {
        crate::linter::lint_makefile(text)
    } else if path_str.to_lowercase().contains("dockerfile") {
        crate::linter::lint_dockerfile_with_profile(text, crate::linter::LintProfile::Standard)
    } else {
        crate::linter::lint_shell(text)
    };

    result.diagnostics
}

#[tower_lsp::async_trait]
impl LanguageServer for BashrsLsp {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("bashrs".to_string()),
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                    },
                )),
                code_action_provider: Some(CodeActionProviderCapability::Options(
                    CodeActionOptions {
                        code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
                        resolve_provider: Some(false),
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                    },
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "bashrs-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("bashrs-lsp v{} initialized", env!("CARGO_PKG_VERSION")),
            )
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        self.lint_and_publish(uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().last() {
            self.lint_and_publish(uri, &change.text).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Ok(text) = std::fs::read_to_string(uri.path()) {
            self.lint_and_publish(uri, &text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        self.doc_diagnostics.write().await.remove(&uri);
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = &params.text_document.uri;
        let docs = self.doc_diagnostics.read().await;
        let Some(diags) = docs.get(uri) else {
            return Ok(None);
        };

        let request_range = params.range;
        let actions: Vec<CodeActionOrCommand> = diags
            .iter()
            .filter(|d| {
                // Only offer actions for diagnostics that overlap the requested range
                let diag_range = to_lsp_range(d.span);
                ranges_overlap(&diag_range, &request_range)
            })
            .filter_map(|d| to_code_action(d, uri))
            .collect();

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let docs = self.doc_diagnostics.read().await;
        let Some(diags) = docs.get(uri) else {
            return Ok(None);
        };

        // Find diagnostic at hover position
        let diag = diags.iter().find(|d| {
            let range = to_lsp_range(d.span);
            position_in_range(&pos, &range)
        });

        let Some(diag) = diag else {
            return Ok(None);
        };

        let content = format_hover_content(diag);
        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: Some(to_lsp_range(diag.span)),
        }))
    }
}

/// Check if a position falls within a range.
fn position_in_range(pos: &Position, range: &Range) -> bool {
    if pos.line < range.start.line || pos.line > range.end.line {
        return false;
    }
    if pos.line == range.start.line && pos.character < range.start.character {
        return false;
    }
    if pos.line == range.end.line && pos.character > range.end.character {
        return false;
    }
    true
}

/// Format hover content for a diagnostic as Markdown.
fn format_hover_content(diag: &crate::linter::Diagnostic) -> String {
    use crate::linter::rule_registry::get_rule_metadata;

    let mut content = String::new();

    // Rule header
    content.push_str(&format!("### `{}` — {}\n\n", diag.code, diag.message));

    // Rule metadata from registry
    if let Some(meta) = get_rule_metadata(&diag.code) {
        content.push_str(&format!("**Rule**: {}\n\n", meta.name));
        content.push_str(&format!(
            "**Compatibility**: {}\n\n",
            meta.compatibility.description()
        ));
    }

    // Severity
    let severity_str = match diag.severity {
        crate::linter::Severity::Error => "Error",
        crate::linter::Severity::Warning => "Warning",
        crate::linter::Severity::Risk => "Risk",
        crate::linter::Severity::Perf => "Performance",
        crate::linter::Severity::Note => "Note",
        crate::linter::Severity::Info => "Info",
    };
    content.push_str(&format!("**Severity**: {}\n\n", severity_str));

    // Fix suggestion
    if let Some(ref fix) = diag.fix {
        if !fix.replacement.is_empty() {
            let safety = match fix.safety_level {
                crate::linter::FixSafetyLevel::Safe => "Safe",
                crate::linter::FixSafetyLevel::SafeWithAssumptions => "Safe (with assumptions)",
                crate::linter::FixSafetyLevel::Unsafe => "Unsafe",
            };
            content.push_str(&format!("**Fix** ({}): `{}`\n\n", safety, truncate(&fix.replacement, 80)));
        }
    }

    // Disable hint
    content.push_str(&format!(
        "---\n*Disable: `# shellcheck disable={}`*",
        diag.code
    ));

    content
}

/// Check if two LSP ranges overlap.
fn ranges_overlap(a: &Range, b: &Range) -> bool {
    !(a.end.line < b.start.line
        || (a.end.line == b.start.line && a.end.character < b.start.character)
        || b.end.line < a.start.line
        || (b.end.line == a.start.line && b.end.character < a.start.character))
}

/// Start the LSP server on stdio (default mode for editor integration).
pub async fn run_stdio() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(BashrsLsp::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
