//! Combined safety check command (SSC v11 Section 8.2).
//!
//! Combines linter findings + rule-based classification in a single pass.
//! When CodeBERT is available, adds ML confidence score.
//!
//! ```text
//! bashrs safety-check script.sh
//!     ├── bashrs lint (<1ms) ──> rule findings
//!     ├── rule-based classify ──> label + confidence
//!     v
//!     Output: {label, confidence, findings[]}
//! ```

use crate::cli::args::ClassifyFormat;
use crate::corpus::dataset::{derive_safety_label, SAFETY_LABELS};
use crate::linter::{lint_dockerfile_with_profile, lint_makefile, lint_shell, LintProfile};
use crate::models::{Error, Result};
use serde::Serialize;
use std::path::Path;

/// Combined safety check result (S8.2).
#[derive(Debug, Serialize)]
pub(crate) struct SafetyCheckResult {
    /// Safety label (safe/unsafe for binary; 5-class for detailed)
    pub label: String,
    /// Binary label index (0=safe, 1=unsafe)
    pub binary_label: u8,
    /// Rule-based confidence (0.0-1.0)
    pub confidence: f64,
    /// Source of confidence: "rule-based" or "codebert" (future)
    pub classifier: String,
    /// Detected script format
    pub format: String,
    /// Lint findings
    pub findings: Vec<Finding>,
    /// Summary counts
    pub summary: CheckSummary,
}

/// A single lint finding.
#[derive(Debug, Serialize)]
pub(crate) struct Finding {
    /// Rule code (e.g., "SEC001", "DET003")
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Line number (1-based)
    pub line: usize,
    /// Severity: "error", "warning", "info"
    pub severity: String,
}

/// Summary of check results.
#[derive(Debug, Serialize)]
pub(crate) struct CheckSummary {
    pub total_findings: usize,
    pub security_findings: usize,
    pub determinism_findings: usize,
    pub idempotency_findings: usize,
    pub other_findings: usize,
}

/// Detect format from file path.
#[allow(clippy::case_sensitive_file_extension_comparisons)] // Already lowercased
fn detect_format(path: &Path) -> ClassifyFormat {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "sh" | "bash" | "zsh" | "ksh" | "dash" => ClassifyFormat::Bash,
        _ => {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();
            if name == "makefile" || name == "gnumakefile" || name.ends_with(".mk") {
                ClassifyFormat::Makefile
            } else if name == "dockerfile"
                || name.starts_with("dockerfile.")
                || name.ends_with(".dockerfile")
            {
                ClassifyFormat::Dockerfile
            } else {
                ClassifyFormat::Bash
            }
        }
    }
}

/// Run the combined safety check on a file.
pub(crate) fn safety_check_command(
    input: &Path,
    json: bool,
    forced_format: Option<&ClassifyFormat>,
) -> Result<()> {
    let source = std::fs::read_to_string(input)
        .map_err(|e| Error::Validation(format!("Cannot read {}: {e}", input.display())))?;

    let fmt = forced_format
        .cloned()
        .unwrap_or_else(|| detect_format(input));

    let result = run_safety_check(&source, &fmt);

    if json {
        let json_str = serde_json::to_string_pretty(&result)
            .map_err(|e| Error::Validation(format!("JSON serialization failed: {e}")))?;
        println!("{json_str}");
    } else {
        print_safety_check(&result);
    }

    Ok(())
}

/// Run combined lint + classify on source text.
fn run_safety_check(source: &str, fmt: &ClassifyFormat) -> SafetyCheckResult {
    // Get lint diagnostics
    let diagnostics = match fmt {
        ClassifyFormat::Bash => lint_shell(source).diagnostics,
        ClassifyFormat::Makefile => lint_makefile(source).diagnostics,
        ClassifyFormat::Dockerfile => {
            lint_dockerfile_with_profile(source, LintProfile::Standard).diagnostics
        }
    };

    // Categorize findings
    let sec_count = diagnostics
        .iter()
        .filter(|d| {
            d.code.starts_with("SEC")
                || d.code == "DOCKER001"
                || d.code == "DOCKER006"
                || d.code == "MAKE003"
        })
        .count();
    let det_count = diagnostics
        .iter()
        .filter(|d| d.code.starts_with("DET") || d.code == "DOCKER002" || d.code == "MAKE001")
        .count();
    let idem_count = diagnostics
        .iter()
        .filter(|d| d.code.starts_with("IDEM") || d.code == "MAKE002")
        .count();
    let other_count = diagnostics.len() - sec_count - det_count - idem_count;

    // Build findings list
    let findings: Vec<Finding> = diagnostics
        .iter()
        .map(|d| {
            let severity = match d.severity {
                crate::linter::Severity::Error => "error",
                crate::linter::Severity::Warning | crate::linter::Severity::Risk => "warning",
                _ => "info",
            };
            Finding {
                code: d.code.clone(),
                message: d.message.clone(),
                line: d.span.start_line,
                severity: severity.to_string(),
            }
        })
        .collect();

    // Derive safety label
    let has_security = sec_count > 0;
    let has_determinism = det_count > 0;
    let lint_clean = !has_security;
    let deterministic = !has_determinism;
    let safety_index = derive_safety_label(source, true, lint_clean, deterministic);

    // Binary label: safe(0) or unsafe(1)
    let binary_label = u8::from(safety_index != 0);
    let label = if binary_label == 0 {
        "safe".to_string()
    } else {
        format!("unsafe ({})", SAFETY_LABELS[safety_index as usize])
    };

    // Confidence from signal strength
    let confidence =
        compute_check_confidence(sec_count, det_count, diagnostics.len(), safety_index);

    SafetyCheckResult {
        label,
        binary_label,
        confidence,
        classifier: "rule-based".to_string(),
        format: match fmt {
            ClassifyFormat::Bash => "bash",
            ClassifyFormat::Makefile => "makefile",
            ClassifyFormat::Dockerfile => "dockerfile",
        }
        .to_string(),
        summary: CheckSummary {
            total_findings: findings.len(),
            security_findings: sec_count,
            determinism_findings: det_count,
            idempotency_findings: idem_count,
            other_findings: other_count,
        },
        findings,
    }
}

fn compute_check_confidence(
    sec_count: usize,
    det_count: usize,
    total_diagnostics: usize,
    safety_index: u8,
) -> f64 {
    match safety_index {
        4 => f64::min(0.85 + (sec_count as f64 - 1.0).max(0.0) * 0.03, 0.99),
        2 => f64::min(0.85 + (det_count as f64 - 1.0).max(0.0) * 0.03, 0.99),
        3 => 0.85,
        1 => 0.80,
        0 => {
            if total_diagnostics == 0 {
                0.95
            } else {
                0.85
            }
        }
        _ => 0.50,
    }
}

/// Print human-readable safety check output.
fn print_safety_check(result: &SafetyCheckResult) {
    use crate::cli::color::*;

    let label_color = if result.binary_label == 0 { GREEN } else { RED };

    println!(
        "{BOLD}{label_color}{}{RESET} (confidence: {:.0}%, classifier: {})",
        result.label,
        result.confidence * 100.0,
        result.classifier,
    );

    if result.summary.total_findings == 0 {
        println!("  No issues found.");
        return;
    }

    println!(
        "\n  {BOLD}Findings:{RESET} {} total ({} security, {} determinism, {} idempotency, {} other)",
        result.summary.total_findings,
        result.summary.security_findings,
        result.summary.determinism_findings,
        result.summary.idempotency_findings,
        result.summary.other_findings,
    );

    for f in &result.findings {
        let severity_color = match f.severity.as_str() {
            "error" => RED,
            "warning" => YELLOW,
            _ => RESET,
        };
        println!(
            "  {severity_color}{:<8}{RESET} L{:<4} {BOLD}{}{RESET}: {}",
            f.code, f.line, f.severity, f.message,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_check_safe_script() {
        let result = run_safety_check("#!/bin/sh\necho \"hello world\"\n", &ClassifyFormat::Bash);
        assert_eq!(result.binary_label, 0);
        assert_eq!(result.label, "safe");
        assert!(result.confidence > 0.7);
        assert_eq!(result.classifier, "rule-based");
        assert_eq!(result.format, "bash");
    }

    #[test]
    fn test_safety_check_unsafe_eval() {
        let result = run_safety_check("#!/bin/bash\neval \"$user_input\"\n", &ClassifyFormat::Bash);
        assert_eq!(result.binary_label, 1);
        assert!(result.label.contains("unsafe"));
        assert!(result.confidence > 0.8);
        assert!(result.summary.security_findings > 0);
    }

    #[test]
    fn test_safety_check_nondeterministic() {
        let result = run_safety_check("#!/bin/bash\necho $RANDOM\n", &ClassifyFormat::Bash);
        assert_eq!(result.binary_label, 1);
        assert!(result.label.contains("non-deterministic"));
        assert!(result.summary.determinism_findings > 0);
    }

    #[test]
    fn test_safety_check_findings_present() {
        let result = run_safety_check("#!/bin/bash\neval \"$RANDOM\"\n", &ClassifyFormat::Bash);
        assert!(!result.findings.is_empty());
        assert!(result.findings.iter().any(|f| f.code.starts_with("SEC")));
    }

    #[test]
    fn test_safety_check_json_serializable() {
        let result = run_safety_check("#!/bin/sh\necho ok\n", &ClassifyFormat::Bash);
        let json = serde_json::to_string_pretty(&result);
        assert!(json.is_ok());
        let json_str = json.expect("serialization should succeed");
        assert!(json_str.contains("\"label\""));
        assert!(json_str.contains("\"findings\""));
        assert!(json_str.contains("\"classifier\""));
    }

    #[test]
    fn test_safety_check_makefile() {
        let result = run_safety_check(
            ".PHONY: build\nbuild:\n\techo ok\n",
            &ClassifyFormat::Makefile,
        );
        assert_eq!(result.format, "makefile");
    }

    #[test]
    fn test_safety_check_dockerfile() {
        let result = run_safety_check(
            "FROM alpine:3.18\nUSER nobody\nCOPY app /app\n",
            &ClassifyFormat::Dockerfile,
        );
        assert_eq!(result.format, "dockerfile");
    }

    #[test]
    fn test_safety_check_summary_counts() {
        let result = run_safety_check("#!/bin/bash\neval \"$RANDOM\"\n", &ClassifyFormat::Bash);
        assert_eq!(
            result.summary.total_findings,
            result.summary.security_findings
                + result.summary.determinism_findings
                + result.summary.idempotency_findings
                + result.summary.other_findings
        );
    }

    #[test]
    fn test_safety_check_empty_script() {
        let result = run_safety_check("", &ClassifyFormat::Bash);
        assert_eq!(result.binary_label, 0);
        assert_eq!(result.summary.total_findings, 0);
    }
}
