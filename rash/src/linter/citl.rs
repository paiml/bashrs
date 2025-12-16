//! CITL (Compiler-In-The-Loop) export for OIP integration
//!
//! Issue #83: Export lint diagnostics in a format compatible with
//! organizational-intelligence-plugin for ML-based defect classification.
//!
//! # Format
//!
//! The CITL format provides structured diagnostic information suitable for:
//! - Training ML models for defect classification
//! - Ground-truth labeling for compiler diagnostics
//! - Integration with OIP (organizational-intelligence-plugin)
//!
//! # Examples
//!
//! ```
//! use bashrs::linter::citl::{CitlExport, CitlDiagnostic};
//! use bashrs::linter::{LintResult, Diagnostic, Severity, Span};
//!
//! let result = LintResult {
//!     diagnostics: vec![
//!         Diagnostic {
//!             code: "SEC010".to_string(),
//!             severity: Severity::Error,
//!             message: "Security issue".to_string(),
//!             span: Span::new(1, 1, 1, 10),
//!             fix: None,
//!         }
//!     ],
//! };
//!
//! let export = CitlExport::from_lint_result("script.sh", &result);
//! let json = export.to_json().expect("valid json");
//! ```

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use super::{Diagnostic, FixSafetyLevel, LintResult, Severity};

/// CITL export container for lint diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitlExport {
    /// Version of the CITL format
    pub version: String,
    /// Source file that was analyzed
    pub source_file: String,
    /// Timestamp of the export (Unix epoch seconds)
    pub timestamp: i64,
    /// Tool that produced the diagnostics
    pub tool: String,
    /// Tool version
    pub tool_version: String,
    /// List of diagnostics
    pub diagnostics: Vec<CitlDiagnostic>,
    /// Summary statistics
    pub summary: CitlSummary,
}

/// Individual diagnostic in CITL format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitlDiagnostic {
    /// Error/warning code (e.g., "SEC010", "SC2086")
    pub error_code: String,
    /// Clippy-style lint name if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clippy_lint: Option<String>,
    /// Severity level: "error", "warning", "info"
    pub level: String,
    /// Human-readable message
    pub message: String,
    /// OIP category for ML classification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oip_category: Option<String>,
    /// Confidence score for classification (0.0-1.0)
    pub confidence: f32,
    /// Source location information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<CitlSpan>,
    /// Suggested fix if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<CitlSuggestion>,
}

/// Source location span in CITL format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitlSpan {
    /// Starting line number (1-indexed)
    pub start_line: usize,
    /// Starting column (1-indexed)
    pub start_col: usize,
    /// Ending line number (1-indexed)
    pub end_line: usize,
    /// Ending column (1-indexed)
    pub end_col: usize,
}

/// Suggested fix in CITL format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitlSuggestion {
    /// Replacement text
    pub replacement: String,
    /// Description of the fix
    pub description: String,
    /// Whether the fix is safe to apply automatically
    pub is_safe: bool,
}

/// Summary statistics for CITL export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitlSummary {
    /// Total number of diagnostics
    pub total: usize,
    /// Number of errors
    pub errors: usize,
    /// Number of warnings
    pub warnings: usize,
    /// Number of info messages
    pub info: usize,
}

impl CitlExport {
    /// Create a CITL export from lint results
    pub fn from_lint_result(source_file: &str, result: &LintResult) -> Self {
        let diagnostics: Vec<CitlDiagnostic> = result
            .diagnostics
            .iter()
            .map(CitlDiagnostic::from_diagnostic)
            .collect();

        let summary = CitlSummary {
            total: diagnostics.len(),
            errors: diagnostics.iter().filter(|d| d.level == "error").count(),
            warnings: diagnostics.iter().filter(|d| d.level == "warning").count(),
            info: diagnostics.iter().filter(|d| d.level == "info").count(),
        };

        Self {
            version: "1.0.0".to_string(),
            source_file: source_file.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            tool: "bashrs".to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            diagnostics,
            summary,
        }
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Write to file
    pub fn write_to_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let json = self.to_json()?;
        fs::write(path, json)?;
        Ok(())
    }
}

impl CitlDiagnostic {
    /// Create from a bashrs Diagnostic
    fn from_diagnostic(diag: &Diagnostic) -> Self {
        // Map bashrs severity to CITL level
        let level = match diag.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info | Severity::Note => "info",
            Severity::Perf => "performance",
            Severity::Risk => "risk",
        }
        .to_string();

        // Determine OIP category based on error code prefix
        let oip_category = Self::classify_oip_category(&diag.code);

        // Create span info
        let span = Some(CitlSpan {
            start_line: diag.span.start_line,
            start_col: diag.span.start_col,
            end_line: diag.span.end_line,
            end_col: diag.span.end_col,
        });

        // Create suggestion if fix is available
        let suggestion = diag.fix.as_ref().map(|fix| CitlSuggestion {
            replacement: fix.replacement.clone(),
            description: if fix.assumptions.is_empty() {
                "Auto-fix available".to_string()
            } else {
                format!("Fix with assumptions: {}", fix.assumptions.join(", "))
            },
            is_safe: matches!(fix.safety_level, FixSafetyLevel::Safe),
        });

        Self {
            error_code: diag.code.clone(),
            clippy_lint: Self::map_to_clippy_lint(&diag.code),
            level,
            message: diag.message.clone(),
            oip_category,
            confidence: Self::compute_confidence(&diag.code),
            span,
            suggestion,
        }
    }

    /// Classify diagnostic into OIP category
    fn classify_oip_category(code: &str) -> Option<String> {
        let code_upper = code.to_uppercase();
        if code_upper.starts_with("SEC") {
            Some("security".to_string())
        } else if code_upper.starts_with("DET") {
            Some("determinism".to_string())
        } else if code_upper.starts_with("IDEM") {
            Some("idempotency".to_string())
        } else if code_upper.starts_with("SC2") {
            // ShellCheck rules - classify by number range
            if let Some(num_str) = code_upper.strip_prefix("SC") {
                if let Ok(num) = num_str.parse::<u32>() {
                    return Some(classify_shellcheck_category(num));
                }
            }
            Some("shellcheck".to_string())
        } else if code_upper.starts_with("MAKE") {
            Some("makefile".to_string())
        } else if code_upper.starts_with("DOCKER") {
            Some("dockerfile".to_string())
        } else if code_upper.starts_with("CONFIG") {
            Some("config".to_string())
        } else {
            None
        }
    }

    /// Map bashrs code to equivalent clippy lint (if applicable)
    fn map_to_clippy_lint(code: &str) -> Option<String> {
        // Only SEC rules have rough clippy equivalents
        match code.to_uppercase().as_str() {
            "SEC001" => Some("clippy::unwrap_used".to_string()),
            "SEC002" => Some("clippy::expect_used".to_string()),
            "SEC010" => Some("clippy::hardcoded_path".to_string()),
            _ => None,
        }
    }

    /// Compute confidence score based on rule type
    fn compute_confidence(code: &str) -> f32 {
        let code_upper = code.to_uppercase();
        // Higher confidence for well-defined security and determinism rules
        if code_upper.starts_with("SEC") {
            0.95
        } else if code_upper.starts_with("DET") || code_upper.starts_with("IDEM") {
            // Determinism and idempotency rules have same confidence
            0.90
        } else if code_upper.starts_with("SC2") {
            0.85 // ShellCheck rules are well-established
        } else {
            0.75 // Default confidence
        }
    }
}

/// Classify ShellCheck rule by number into OIP category
fn classify_shellcheck_category(num: u32) -> String {
    match num {
        // Quoting-related (SC2000-SC2099)
        2000..=2099 => "quoting".to_string(),
        // Variable-related (SC2100-SC2199)
        2100..=2199 => "variables".to_string(),
        // Miscellaneous (SC2200+)
        _ => "shellcheck".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::{Fix, Span};

    #[test]
    fn test_issue_83_citl_export_basic() {
        let result = LintResult {
            diagnostics: vec![Diagnostic {
                code: "SEC010".to_string(),
                severity: Severity::Error,
                message: "Hardcoded path detected".to_string(),
                span: Span {
                    start_line: 1,
                    start_col: 1,
                    end_line: 1,
                    end_col: 10,
                },
                fix: None,
            }],
        };

        let export = CitlExport::from_lint_result("test.sh", &result);

        assert_eq!(export.source_file, "test.sh");
        assert_eq!(export.diagnostics.len(), 1);
        assert_eq!(export.diagnostics[0].error_code, "SEC010");
        assert_eq!(export.diagnostics[0].level, "error");
        assert_eq!(
            export.diagnostics[0].oip_category,
            Some("security".to_string())
        );
        assert!(export.diagnostics[0].confidence > 0.9);
    }

    #[test]
    fn test_issue_83_citl_export_with_fix() {
        let result = LintResult {
            diagnostics: vec![Diagnostic {
                code: "SC2086".to_string(),
                severity: Severity::Warning,
                message: "Double quote to prevent globbing".to_string(),
                span: Span::new(1, 1, 1, 10),
                fix: Some(Fix::new("\"$var\"")),
            }],
        };

        let export = CitlExport::from_lint_result("test.sh", &result);

        assert_eq!(export.diagnostics.len(), 1);
        assert!(export.diagnostics[0].suggestion.is_some());
        let suggestion = export.diagnostics[0].suggestion.as_ref().unwrap();
        assert_eq!(suggestion.replacement, "\"$var\"");
        assert!(suggestion.is_safe);
    }

    #[test]
    fn test_issue_83_citl_summary() {
        let result = LintResult {
            diagnostics: vec![
                Diagnostic {
                    code: "SEC001".to_string(),
                    severity: Severity::Error,
                    message: "Error".to_string(),
                    span: Span::new(1, 1, 1, 10),
                    fix: None,
                },
                Diagnostic {
                    code: "SC2086".to_string(),
                    severity: Severity::Warning,
                    message: "Warning".to_string(),
                    span: Span::new(2, 1, 2, 10),
                    fix: None,
                },
                Diagnostic {
                    code: "SC2148".to_string(),
                    severity: Severity::Info,
                    message: "Info".to_string(),
                    span: Span::new(3, 1, 3, 10),
                    fix: None,
                },
            ],
        };

        let export = CitlExport::from_lint_result("test.sh", &result);

        assert_eq!(export.summary.total, 3);
        assert_eq!(export.summary.errors, 1);
        assert_eq!(export.summary.warnings, 1);
        assert_eq!(export.summary.info, 1);
    }

    #[test]
    fn test_issue_83_citl_to_json() {
        let result = LintResult {
            diagnostics: vec![Diagnostic {
                code: "DET001".to_string(),
                severity: Severity::Error,
                message: "Non-deterministic".to_string(),
                span: Span::new(1, 1, 1, 20),
                fix: None,
            }],
        };

        let export = CitlExport::from_lint_result("test.sh", &result);
        let json = export.to_json().expect("should serialize");

        assert!(json.contains("\"error_code\": \"DET001\""));
        assert!(json.contains("\"oip_category\": \"determinism\""));
        assert!(json.contains("\"tool\": \"bashrs\""));
    }

    #[test]
    fn test_issue_83_oip_category_classification() {
        // Security rules
        assert_eq!(
            CitlDiagnostic::classify_oip_category("SEC001"),
            Some("security".to_string())
        );
        assert_eq!(
            CitlDiagnostic::classify_oip_category("SEC010"),
            Some("security".to_string())
        );

        // Determinism rules
        assert_eq!(
            CitlDiagnostic::classify_oip_category("DET001"),
            Some("determinism".to_string())
        );

        // Idempotency rules
        assert_eq!(
            CitlDiagnostic::classify_oip_category("IDEM001"),
            Some("idempotency".to_string())
        );

        // ShellCheck rules
        assert!(CitlDiagnostic::classify_oip_category("SC2086").is_some());
    }
}
