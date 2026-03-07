//! WASM bindings for bashrs shell safety linter (SSC v11 Phase 5).
//!
//! Exposes the bashrs linter and classifier as JavaScript-callable functions
//! for the shell-safety.html interactive app.
//!
//! # Architecture
//!
//! ```text
//! JavaScript (shell-safety.html)
//!   |
//!   v
//! bashrs-wasm (this crate)
//!   ├── lint_shell_wasm(source)       -> JSON findings
//!   ├── lint_makefile_wasm(source)    -> JSON findings
//!   ├── lint_dockerfile_wasm(source)  -> JSON findings
//!   ├── classify_shell_wasm(source)   -> JSON classification
//!   ├── explain_shell_wasm(source)    -> JSON explanation
//!   └── bashrs_version()              -> version string
//!   |
//!   v
//! bashrs (rash crate, minimal features)
//!   └── linter::{lint_shell, lint_makefile, lint_dockerfile_with_profile}
//! ```

use wasm_bindgen::prelude::*;

#[cfg(feature = "codebert")]
mod wasm_encoder;

#[cfg(feature = "codebert")]
use std::cell::RefCell;

#[cfg(feature = "codebert")]
thread_local! {
    /// Cached encoder model (loaded once, reused for all classifications).
    static ENCODER: RefCell<Option<wasm_encoder::WasmEncoder>> = const { RefCell::new(None) };
    /// Cached MLP probe weights.
    static PROBE: RefCell<Option<wasm_encoder::MlpProbe>> = const { RefCell::new(None) };
}

/// Lint a shell script and return findings as JSON.
///
/// Returns a JSON string with the structure:
/// ```json
/// {
///   "diagnostics": [
///     { "code": "SEC001", "severity": "warning", "message": "...", "line": 1 }
///   ],
///   "count": 3
/// }
/// ```
#[wasm_bindgen]
pub fn lint_shell_wasm(source: &str) -> String {
    let result = bashrs::linter::lint_shell(source);
    let findings: Vec<Finding> = result
        .diagnostics
        .iter()
        .map(|d| Finding {
            code: d.code.clone(),
            severity: format!("{:?}", d.severity),
            message: d.message.clone(),
            line: d.span.start_line,
        })
        .collect();

    let output = LintOutput {
        diagnostics: findings,
        count: result.diagnostics.len(),
    };

    serde_json::to_string(&output).unwrap_or_else(|_| "{}".to_string())
}

/// Lint a Makefile and return findings as JSON.
#[wasm_bindgen]
pub fn lint_makefile_wasm(source: &str) -> String {
    let result = bashrs::linter::lint_makefile(source);
    let findings: Vec<Finding> = result
        .diagnostics
        .iter()
        .map(|d| Finding {
            code: d.code.clone(),
            severity: format!("{:?}", d.severity),
            message: d.message.clone(),
            line: d.span.start_line,
        })
        .collect();

    let output = LintOutput {
        diagnostics: findings,
        count: result.diagnostics.len(),
    };

    serde_json::to_string(&output).unwrap_or_else(|_| "{}".to_string())
}

/// Lint a Dockerfile and return findings as JSON.
#[wasm_bindgen]
pub fn lint_dockerfile_wasm(source: &str) -> String {
    let result =
        bashrs::linter::lint_dockerfile_with_profile(source, bashrs::linter::LintProfile::Standard);
    let findings: Vec<Finding> = result
        .diagnostics
        .iter()
        .map(|d| Finding {
            code: d.code.clone(),
            severity: format!("{:?}", d.severity),
            message: d.message.clone(),
            line: d.span.start_line,
        })
        .collect();

    let output = LintOutput {
        diagnostics: findings,
        count: result.diagnostics.len(),
    };

    serde_json::to_string(&output).unwrap_or_else(|_| "{}".to_string())
}

/// Classify a shell script's safety level (rule-based).
///
/// Returns JSON:
/// ```json
/// {
///   "label": "safe",
///   "confidence": 95.0,
///   "has_security": false,
///   "has_determinism": false,
///   "has_idempotency": false,
///   "finding_count": 0
/// }
/// ```
#[wasm_bindgen]
pub fn classify_shell_wasm(source: &str) -> String {
    let result = bashrs::linter::lint_shell(source);

    let has_security = result.diagnostics.iter().any(|d| d.code.starts_with("SEC"));
    let has_determinism = result.diagnostics.iter().any(|d| d.code.starts_with("DET"));
    let has_idempotency = result
        .diagnostics
        .iter()
        .any(|d| d.code.starts_with("IDEM"));

    let label = if has_security || has_determinism || has_idempotency {
        "unsafe"
    } else {
        "safe"
    };

    let confidence = if result.diagnostics.is_empty() {
        95.0
    } else {
        let severity_score: f64 = result
            .diagnostics
            .iter()
            .map(|d| {
                if d.code.starts_with("SEC") {
                    3.0
                } else if d.code.starts_with("DET") {
                    2.0
                } else if d.code.starts_with("IDEM") {
                    1.5
                } else {
                    1.0
                }
            })
            .sum();
        (95.0 - severity_score * 5.0).max(10.0)
    };

    let output = ClassifyOutput {
        label: label.to_string(),
        confidence,
        has_security,
        has_determinism,
        has_idempotency,
        finding_count: result.diagnostics.len(),
    };

    serde_json::to_string(&output).unwrap_or_else(|_| "{}".to_string())
}

/// Explain a shell script's safety issues in human-readable format.
///
/// Returns JSON:
/// ```json
/// {
///   "summary": "Found 2 issues: 1 security, 1 determinism",
///   "issues": [
///     { "code": "SEC001", "severity": "Error", "explanation": "...", "fix": "...", "line": 1 }
///   ],
///   "recommendation": "unsafe — review security issues before deployment"
/// }
/// ```
#[wasm_bindgen]
pub fn explain_shell_wasm(source: &str) -> String {
    let result = bashrs::linter::lint_shell(source);

    let issues: Vec<ExplainIssue> = result
        .diagnostics
        .iter()
        .map(|d| ExplainIssue {
            code: d.code.clone(),
            severity: format!("{:?}", d.severity),
            explanation: d.message.clone(),
            fix: d.fix.as_ref().map(|f| f.replacement.clone()),
            line: d.span.start_line,
        })
        .collect();

    let sec_count = result
        .diagnostics
        .iter()
        .filter(|d| d.code.starts_with("SEC"))
        .count();
    let det_count = result
        .diagnostics
        .iter()
        .filter(|d| d.code.starts_with("DET"))
        .count();
    let idem_count = result
        .diagnostics
        .iter()
        .filter(|d| d.code.starts_with("IDEM"))
        .count();

    let summary = if issues.is_empty() {
        "No issues found — script appears safe".to_string()
    } else {
        let mut parts = Vec::new();
        if sec_count > 0 {
            parts.push(format!("{sec_count} security"));
        }
        if det_count > 0 {
            parts.push(format!("{det_count} determinism"));
        }
        if idem_count > 0 {
            parts.push(format!("{idem_count} idempotency"));
        }
        let other = issues.len() - sec_count - det_count - idem_count;
        if other > 0 {
            parts.push(format!("{other} other"));
        }
        format!("Found {} issues: {}", issues.len(), parts.join(", "))
    };

    let recommendation = if sec_count > 0 {
        "unsafe — review security issues before deployment"
    } else if det_count > 0 || idem_count > 0 {
        "unsafe — non-deterministic or non-idempotent operations detected"
    } else if issues.is_empty() {
        "safe — no issues detected"
    } else {
        "review — minor issues detected"
    };

    let output = ExplainOutput {
        summary,
        issues,
        recommendation: recommendation.to_string(),
    };

    serde_json::to_string(&output).unwrap_or_else(|_| "{}".to_string())
}

/// Load CodeBERT int8 model weights into WASM memory.
///
/// Call this once with the contents of `model_int8.safetensors`.
/// The model is cached in thread-local storage for subsequent classifications.
///
/// Returns empty string on success, error message on failure.
#[cfg(feature = "codebert")]
#[wasm_bindgen]
pub fn load_codebert_model(data: &[u8]) -> String {
    match wasm_encoder::WasmEncoder::from_safetensors_bytes(data) {
        Ok(encoder) => {
            ENCODER.with(|cell| {
                *cell.borrow_mut() = Some(encoder);
            });
            String::new()
        }
        Err(e) => e,
    }
}

/// Load MLP probe weights for CodeBERT classification.
///
/// Call this once with the contents of `probe.json`.
/// Returns empty string on success, error message on failure.
#[cfg(feature = "codebert")]
#[wasm_bindgen]
pub fn load_codebert_probe(data: &[u8]) -> String {
    match wasm_encoder::MlpProbe::from_json(data) {
        Ok(probe) => {
            PROBE.with(|cell| {
                *cell.borrow_mut() = Some(probe);
            });
            String::new()
        }
        Err(e) => e,
    }
}

/// Classify a shell script using CodeBERT encoder + MLP probe.
///
/// Requires `load_codebert_model` and `load_codebert_probe` to be called first.
///
/// Returns JSON:
/// ```json
/// {
///   "label": "unsafe",
///   "confidence": 0.87,
///   "model": "codebert-int8",
///   "error": null
/// }
/// ```
#[cfg(feature = "codebert")]
#[wasm_bindgen]
pub fn classify_codebert_wasm(source: &str) -> String {
    let result = ENCODER.with(|enc_cell| {
        PROBE.with(|probe_cell| {
            let enc = enc_cell.borrow();
            let probe = probe_cell.borrow();
            match (enc.as_ref(), probe.as_ref()) {
                (Some(encoder), Some(probe)) => {
                    let r = wasm_encoder::classify_with_codebert(encoder, probe, source);
                    let label = if r.label == 0 { "safe" } else { "unsafe" };
                    serde_json::json!({
                        "label": label,
                        "confidence": r.confidence,
                        "model": "codebert-int8",
                        "error": serde_json::Value::Null
                    })
                }
                (None, _) => serde_json::json!({
                    "label": serde_json::Value::Null,
                    "confidence": 0.0,
                    "model": "codebert-int8",
                    "error": "Model not loaded. Call load_codebert_model() first."
                }),
                (_, None) => serde_json::json!({
                    "label": serde_json::Value::Null,
                    "confidence": 0.0,
                    "model": "codebert-int8",
                    "error": "Probe not loaded. Call load_codebert_probe() first."
                }),
            }
        })
    });
    result.to_string()
}

/// Get the bashrs version.
#[wasm_bindgen]
pub fn bashrs_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[derive(serde::Serialize)]
struct Finding {
    code: String,
    severity: String,
    message: String,
    line: usize,
}

#[derive(serde::Serialize)]
struct LintOutput {
    diagnostics: Vec<Finding>,
    count: usize,
}

#[derive(serde::Serialize)]
struct ClassifyOutput {
    label: String,
    confidence: f64,
    has_security: bool,
    has_determinism: bool,
    has_idempotency: bool,
    finding_count: usize,
}

#[derive(serde::Serialize)]
struct ExplainIssue {
    code: String,
    severity: String,
    explanation: String,
    fix: Option<String>,
    line: usize,
}

#[derive(serde::Serialize)]
struct ExplainOutput {
    summary: String,
    issues: Vec<ExplainIssue>,
    recommendation: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lint_shell_wasm_safe_script() {
        let result = lint_shell_wasm("#!/bin/sh\necho hello\n");
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["count"], 0);
    }

    #[test]
    fn test_lint_shell_wasm_unsafe_script() {
        let result = lint_shell_wasm("#!/bin/bash\neval \"$user_input\"\n");
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert!(parsed["count"].as_u64().expect("count") > 0);
        let diags = parsed["diagnostics"].as_array().expect("diagnostics array");
        assert!(diags
            .iter()
            .any(|d| d["code"].as_str().unwrap_or("").starts_with("SEC")));
    }

    #[test]
    fn test_classify_shell_wasm_safe() {
        let result = classify_shell_wasm("#!/bin/sh\necho hello\n");
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["label"], "safe");
        assert!(parsed["confidence"].as_f64().expect("confidence") > 90.0);
    }

    #[test]
    fn test_classify_shell_wasm_unsafe() {
        let result = classify_shell_wasm("#!/bin/bash\neval \"$1\"\n");
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["label"], "unsafe");
        assert!(parsed["has_security"].as_bool().expect("bool"));
    }

    #[test]
    fn test_lint_makefile_wasm() {
        let result = lint_makefile_wasm(".PHONY: build\nbuild:\n\techo ok\n");
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert!(parsed["count"].is_number());
    }

    #[test]
    fn test_lint_dockerfile_wasm() {
        let result = lint_dockerfile_wasm("FROM alpine:latest\nRUN apt-get install curl\n");
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert!(parsed["count"].is_number());
    }

    #[test]
    fn test_bashrs_version() {
        let v = bashrs_version();
        assert!(!v.is_empty());
    }

    #[test]
    fn test_explain_shell_wasm_safe() {
        let result = explain_shell_wasm("#!/bin/sh\necho hello\n");
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert!(parsed["summary"].as_str().unwrap().contains("No issues"));
        assert_eq!(parsed["recommendation"], "safe — no issues detected");
        assert_eq!(parsed["issues"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_explain_shell_wasm_unsafe() {
        let result = explain_shell_wasm("#!/bin/bash\neval \"$1\"\n");
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert!(parsed["summary"].as_str().unwrap().contains("security"));
        assert!(parsed["recommendation"]
            .as_str()
            .unwrap()
            .contains("unsafe"));
        let issues = parsed["issues"].as_array().unwrap();
        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|i| i["code"].as_str().unwrap().starts_with("SEC")));
    }
}
