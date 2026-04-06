//! Idempotency checking for bash scripts
//!
//! Detects non-idempotent operations that may fail on re-run:
//! - mkdir without -p: Fails if directory exists
//! - rm without -f: Fails if file doesn't exist
//! - ln -s without -f: Fails if symlink exists

use super::determinism::{OutputDifference, RunOutput};
use super::determinism_replay::{format_replay_diff, ReplayVerifier};

/// Detects non-idempotent operations in bash scripts
#[derive(Debug, Clone, PartialEq)]
pub struct IdempotencyChecker {
    /// Issues detected in the script
    detections: Vec<IdempotencyIssue>,
}

/// A single non-idempotent operation detection
#[derive(Debug, Clone, PartialEq)]
pub struct IdempotencyIssue {
    /// Line number where operation was found (1-indexed)
    pub line: usize,
    /// Type of non-idempotent operation
    pub operation_type: NonIdempotentOperation,
    /// The actual code that triggered the detection
    pub code: String,
    /// Human-readable explanation
    pub explanation: String,
    /// Suggested fix
    pub suggestion: String,
}

/// Types of non-idempotent operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonIdempotentOperation {
    /// mkdir without -p flag
    MkdirWithoutP,
    /// rm without -f flag
    RmWithoutF,
    /// ln -s without -f flag
    LnWithoutF,
}

impl IdempotencyChecker {
    /// Create a new idempotency checker
    pub fn new() -> Self {
        Self {
            detections: Vec::new(),
        }
    }

    /// Scan bash script for non-idempotent operations
    ///
    /// Returns: List of detected issues
    pub fn scan(&mut self, script: &str) -> Vec<IdempotencyIssue> {
        // Clear previous results
        self.detections.clear();

        for (line_num, line) in script.lines().enumerate() {
            let line_num = line_num + 1; // 1-indexed

            // Detect mkdir without -p
            if line.contains("mkdir ") && !line.contains("mkdir -p") {
                self.detections.push(IdempotencyIssue {
                    line: line_num,
                    operation_type: NonIdempotentOperation::MkdirWithoutP,
                    code: line.to_string(),
                    explanation: "mkdir without -p fails if directory already exists".to_string(),
                    suggestion: "Add -p flag: mkdir -p".to_string(),
                });
            }

            // Detect rm without -f (but not rm -rf)
            if line.contains("rm ") && !line.contains(" -f") && !line.contains(" -rf") {
                self.detections.push(IdempotencyIssue {
                    line: line_num,
                    operation_type: NonIdempotentOperation::RmWithoutF,
                    code: line.to_string(),
                    explanation: "rm without -f fails if file doesn't exist".to_string(),
                    suggestion: "Add -f flag: rm -f".to_string(),
                });
            }

            // Detect ln -s without -f
            if line.contains("ln -s") && !line.contains("ln -sf") {
                self.detections.push(IdempotencyIssue {
                    line: line_num,
                    operation_type: NonIdempotentOperation::LnWithoutF,
                    code: line.to_string(),
                    explanation: "ln -s without -f fails if symlink already exists".to_string(),
                    suggestion: "Add -f flag: ln -sf".to_string(),
                });
            }
        }

        self.detections.clone()
    }

    /// Check if script is idempotent (no issues found)
    pub fn is_idempotent(&self) -> bool {
        self.detections.is_empty()
    }

    /// Get count of issues by operation type
    pub fn count_by_operation(&self, operation: NonIdempotentOperation) -> usize {
        self.detections
            .iter()
            .filter(|issue| issue.operation_type == operation)
            .count()
    }
}

impl Default for IdempotencyChecker {
    fn default() -> Self {
        Self::new()
    }
}

// ===== REPL-012-002: IDEMPOTENCY REPORT FORMATTING =====

/// Format idempotency scan results for display
///
/// Shows a user-friendly report of non-idempotent operations with
/// line numbers, problematic code, explanations, and fix suggestions.
///
/// # Examples
///
/// ```
/// use bashrs::repl::determinism_idempotency::{IdempotencyIssue, NonIdempotentOperation};
/// use bashrs::repl::determinism_idempotency::format_idempotency_report;
///
/// let issues = vec![
///     IdempotencyIssue {
///         line: 5,
///         operation_type: NonIdempotentOperation::MkdirWithoutP,
///         code: "mkdir /tmp/foo".to_string(),
///         explanation: "mkdir without -p fails if directory already exists".to_string(),
///         suggestion: "Add -p flag: mkdir -p".to_string(),
///     },
/// ];
///
/// let formatted = format_idempotency_report(&issues);
/// assert!(formatted.contains("Line 5:"));
/// assert!(formatted.contains("mkdir /tmp/foo"));
/// assert!(formatted.contains("Add -p flag"));
/// ```
pub fn format_idempotency_report(issues: &[IdempotencyIssue]) -> String {
    if issues.is_empty() {
        return String::from("✓ Script is idempotent - safe to re-run");
    }

    let mut output = String::new();
    output.push_str("⚠️  Non-idempotent operations detected!\n\n");
    output.push_str(&format!("Found {} issue(s):\n\n", issues.len()));

    for issue in issues {
        output.push_str(&format!("Line {}:\n", issue.line));
        output.push_str(&format!("  Code:        {}\n", issue.code));
        output.push_str(&format!("  Problem:     {}\n", issue.explanation));
        output.push_str(&format!("  💡 Fix:      {}\n", issue.suggestion));
        output.push('\n');
    }

    output
}

impl IdempotencyChecker {
    /// Format scan results for display
    ///
    /// Shows idempotency status, issue counts, operation breakdown,
    /// and detailed issue reports.
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::IdempotencyChecker;
    ///
    /// let mut checker = IdempotencyChecker::new();
    /// let script = "mkdir /tmp/foo\nrm /tmp/bar";
    /// checker.scan(script);
    ///
    /// let report = checker.format_report();
    /// assert!(report.contains("non-idempotent") || report.contains("idempotent"));
    /// ```
    pub fn format_report(&self) -> String {
        let mut output = String::new();

        // Show idempotency status
        if self.is_idempotent() {
            output.push_str("✓ Script is idempotent\n");
            output.push_str("  Safe to run multiple times\n");
        } else {
            output.push_str("⚠️  Script is non-idempotent\n");
            output.push_str(&format!("  {} issue(s) found\n", self.detections.len()));
        }

        // Show issue breakdown by type
        let mkdir_count = self.count_by_operation(NonIdempotentOperation::MkdirWithoutP);
        let rm_count = self.count_by_operation(NonIdempotentOperation::RmWithoutF);
        let ln_count = self.count_by_operation(NonIdempotentOperation::LnWithoutF);

        if mkdir_count > 0 || rm_count > 0 || ln_count > 0 {
            output.push_str("\nOperation breakdown:\n");
            if mkdir_count > 0 {
                output.push_str(&format!("  mkdir (missing -p): {}\n", mkdir_count));
            }
            if rm_count > 0 {
                output.push_str(&format!("  rm (missing -f):    {}\n", rm_count));
            }
            if ln_count > 0 {
                output.push_str(&format!("  ln (missing -f):    {}\n", ln_count));
            }
        }

        // Show detailed issues
        if !self.detections.is_empty() {
            output.push('\n');
            output.push_str(&format_idempotency_report(&self.detections));
        }

        output
    }
}

include!("determinism_idempotency_idempotencyverifier.rs");
