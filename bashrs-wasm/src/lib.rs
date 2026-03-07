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
//!   ├── lint_shell_wasm(source) -> JSON findings
//!   ├── classify_shell_wasm(source) -> JSON classification
//!   └── explain_shell_wasm(source) -> JSON explanation
//!   |
//!   v
//! bashrs (rash crate, no-default-features)
//!   ├── linter::lint_shell()
//!   ├── linter::lint_makefile()
//!   └── linter::lint_dockerfile()
//! ```

use wasm_bindgen::prelude::*;

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
}
