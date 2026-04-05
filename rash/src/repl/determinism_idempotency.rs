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

// ===== REPL-012-003: IDEMPOTENCY VERIFICATION =====

/// Verifies script idempotency by running multiple times and comparing results
///
/// Runs a script N times (default: 3) and checks if all runs produce
/// identical output and exit codes.
#[derive(Debug, Clone)]
pub struct IdempotencyVerifier {
    /// Number of verification runs to perform (default: 3, min: 2)
    run_count: usize,
}

impl IdempotencyVerifier {
    /// Create a new idempotency verifier with default settings (3 runs)
    pub fn new() -> Self {
        Self { run_count: 3 }
    }

    /// Create verifier with custom run count (minimum 2)
    pub fn with_run_count(count: usize) -> Self {
        Self {
            run_count: count.max(2),
        }
    }

    /// Verify script idempotency by running multiple times
    pub fn verify(&self, script: &str) -> IdempotencyResult {
        let mut runs = Vec::new();

        // Run script multiple times
        for run_number in 1..=self.run_count {
            let mut output = ReplayVerifier::execute_script(script);
            output.run_number = run_number;
            runs.push(output);
        }

        // Compare all runs against the first run
        let mut differences = Vec::new();
        if let Some(first_run) = runs.first() {
            // Compare each subsequent run with the first run
            for run in runs.iter().skip(1) {
                let run_diffs = ReplayVerifier::find_differences(first_run, run);
                // Merge differences (avoid duplicates)
                for diff in run_diffs {
                    if !differences.iter().any(|d: &OutputDifference| {
                        d.line == diff.line && d.run1 == diff.run1 && d.run2 == diff.run2
                    }) {
                        differences.push(diff);
                    }
                }
            }
        }

        let is_idempotent = differences.is_empty();

        IdempotencyResult {
            is_idempotent,
            run_count: self.run_count,
            runs,
            differences,
        }
    }
}

impl Default for IdempotencyVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of idempotency verification
#[derive(Debug, Clone, PartialEq)]
pub struct IdempotencyResult {
    /// Whether all runs produced identical output
    pub is_idempotent: bool,
    /// Number of runs performed
    pub run_count: usize,
    /// Outputs from each run
    pub runs: Vec<RunOutput>,
    /// Lines that differ between runs (if non-idempotent)
    pub differences: Vec<OutputDifference>,
}

impl IdempotencyResult {
    /// Format verification result for display
    pub fn format_result(&self) -> String {
        let mut output = String::new();

        // Show idempotency status
        if self.is_idempotent {
            output.push_str("✓ Script is idempotent\n");
            output.push_str(&format!("  Verified across {} runs\n", self.run_count));
        } else {
            output.push_str("❌ Script is non-idempotent\n");
            output.push_str(&format!("  Tested {} runs\n", self.run_count));
        }

        // Show run count
        output.push_str(&format!("\nRuns: {}\n", self.runs.len()));

        // Show exit codes
        output.push_str("Exit codes: ");
        for run in &self.runs {
            output.push_str(&format!("{} ", run.exit_code));
        }
        output.push('\n');

        // Show differences if any
        if !self.differences.is_empty() {
            output.push('\n');
            output.push_str(&format_replay_diff(&self.differences));
        }

        output
    }
}

// ===== IDEMPOTENCY TESTS =====

#[cfg(test)]
mod idempotency_tests {
    use super::*;

    // ===== REPL-012-001: IDEMPOTENCY SCANNER TESTS =====

    #[test]
    fn test_REPL_012_001_detect_mkdir_without_p() {
        // ARRANGE: Script with mkdir (no -p)
        let script = "mkdir /tmp/testdir";
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should detect mkdir without -p
        assert_eq!(issues.len(), 1, "Should detect 1 issue");
        assert_eq!(
            issues[0].operation_type,
            NonIdempotentOperation::MkdirWithoutP
        );
        assert_eq!(issues[0].line, 1);
        assert!(issues[0].explanation.contains("mkdir"));
        assert!(issues[0].suggestion.contains("-p"));
    }

    #[test]
    fn test_REPL_012_001_mkdir_with_p_is_idempotent() {
        // ARRANGE: Script with mkdir -p
        let script = "mkdir -p /tmp/testdir";
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should not detect any issues
        assert_eq!(issues.len(), 0, "mkdir -p should be idempotent");
        assert!(checker.is_idempotent());
    }

    #[test]
    fn test_REPL_012_001_detect_rm_without_f() {
        // ARRANGE: Script with rm (no -f)
        let script = "rm /tmp/testfile";
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should detect rm without -f
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].operation_type, NonIdempotentOperation::RmWithoutF);
        assert!(issues[0].explanation.contains("rm"));
        assert!(issues[0].suggestion.contains("-f"));
    }

    #[test]
    fn test_REPL_012_001_rm_with_f_is_idempotent() {
        // ARRANGE: Script with rm -f
        let script = "rm -f /tmp/testfile";
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should not detect any issues
        assert_eq!(issues.len(), 0, "rm -f should be idempotent");
    }

    #[test]
    fn test_REPL_012_001_detect_ln_without_f() {
        // ARRANGE: Script with ln -s (no -f)
        let script = "ln -s /source /target";
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should detect ln -s without -f
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].operation_type, NonIdempotentOperation::LnWithoutF);
        assert!(issues[0].explanation.contains("ln"));
        assert!(issues[0].suggestion.contains("-f"));
    }

    #[test]
    fn test_REPL_012_001_ln_with_sf_is_idempotent() {
        // ARRANGE: Script with ln -sf
        let script = "ln -sf /source /target";
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should not detect any issues
        assert_eq!(issues.len(), 0, "ln -sf should be idempotent");
    }

    #[test]
    fn test_REPL_012_001_multiple_issues() {
        // ARRANGE: Script with multiple non-idempotent operations
        let script = r#"
mkdir /tmp/dir1
rm /tmp/file1
ln -s /source /target
mkdir -p /tmp/dir2
        "#;
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should detect 3 issues (line 5 is idempotent)
        assert_eq!(issues.len(), 3);
        assert_eq!(
            checker.count_by_operation(NonIdempotentOperation::MkdirWithoutP),
            1
        );
        assert_eq!(
            checker.count_by_operation(NonIdempotentOperation::RmWithoutF),
            1
        );
        assert_eq!(
            checker.count_by_operation(NonIdempotentOperation::LnWithoutF),
            1
        );
    }

    #[test]
    fn test_REPL_012_001_fully_idempotent_script() {
        // ARRANGE: Fully idempotent script
        let script = r#"
#!/bin/sh
mkdir -p /app/releases
rm -f /app/current
ln -sf /app/releases/v1.0.0 /app/current
        "#;
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should detect no issues
        assert_eq!(issues.len(), 0);
        assert!(checker.is_idempotent());
    }

    #[test]
    fn test_REPL_012_001_rescan_clears_previous() {
        // ARRANGE: Checker with previous detections
        let mut checker = IdempotencyChecker::new();
        checker.scan("mkdir /tmp/test1");
        assert_eq!(checker.scan("mkdir /tmp/test1").len(), 1);

        // ACT: Scan new script
        let issues = checker.scan("mkdir -p /tmp/test2");

        // ASSERT: Should clear previous detections
        assert_eq!(issues.len(), 0, "Rescan should clear previous detections");
        assert!(checker.is_idempotent());
    }

    #[test]
    fn test_REPL_012_001_line_numbers_correct() {
        // ARRANGE: Multi-line script
        let script = r#"
# Line 1: comment
mkdir /tmp/dir  # Line 2: issue
# Line 3: comment
rm /tmp/file    # Line 4: issue
        "#;
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Line numbers should be correct
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].line, 3); // Line 3 in 1-indexed
        assert_eq!(issues[1].line, 5); // Line 5 in 1-indexed
    }
}

#[cfg(test)]
mod idempotency_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_REPL_012_001_mkdir_without_p_always_detected(
            path in "[a-z0-9/]{1,50}"
        ) {
            let script = format!("mkdir {}", path);
            let mut checker = IdempotencyChecker::new();
            let issues = checker.scan(&script);

            // mkdir without -p should always be detected
            prop_assert_eq!(issues.len(), 1);
            prop_assert_eq!(issues[0].operation_type, NonIdempotentOperation::MkdirWithoutP);
        }

        #[test]
        fn prop_REPL_012_001_mkdir_with_p_never_detected(
            path in "[a-z0-9/]{1,50}"
        ) {
            let script = format!("mkdir -p {}", path);
            let mut checker = IdempotencyChecker::new();
            let issues = checker.scan(&script);

            // mkdir -p should never be detected as non-idempotent
            prop_assert_eq!(issues.len(), 0);
            prop_assert!(checker.is_idempotent());
        }

        #[test]
        fn prop_REPL_012_001_scan_never_panics(
            script in ".*{0,1000}"
        ) {
            let mut checker = IdempotencyChecker::new();
            // Should never panic on any input
            let _ = checker.scan(&script);
        }

        #[test]
        fn prop_REPL_012_001_rescan_always_clears(
            script1 in "mkdir [a-z]{1,20}",
            script2 in "mkdir -p [a-z]{1,20}"
        ) {
            let mut checker = IdempotencyChecker::new();

            // First scan should find issue
            let issues1 = checker.scan(&script1);
            prop_assert_eq!(issues1.len(), 1);

            // Second scan should clear and find no issues
            let issues2 = checker.scan(&script2);
            prop_assert_eq!(issues2.len(), 0);
            prop_assert!(checker.is_idempotent());
        }
    }
}

#[cfg(test)]
mod idempotency_report_tests {
    use super::*;

    // ===== REPL-012-002: REPORT FORMATTING TESTS =====

    #[test]
    fn test_REPL_012_002_format_single_mkdir_issue() {
        // ARRANGE: One mkdir issue
        let issues = vec![IdempotencyIssue {
            line: 10,
            operation_type: NonIdempotentOperation::MkdirWithoutP,
            code: "mkdir /tmp/test".to_string(),
            explanation: "mkdir without -p fails if directory already exists".to_string(),
            suggestion: "Add -p flag: mkdir -p".to_string(),
        }];

        // ACT: Format report
        let formatted = format_idempotency_report(&issues);

        // ASSERT: Should show line, code, problem, and fix
        assert!(formatted.contains("Line 10:"));
        assert!(formatted.contains("mkdir /tmp/test"));
        assert!(formatted.contains("mkdir without -p"));
        assert!(formatted.contains("Add -p flag"));
        assert!(formatted.contains("⚠️") || formatted.contains("warning"));
    }

    #[test]
    fn test_REPL_012_002_format_single_rm_issue() {
        // ARRANGE: One rm issue
        let issues = vec![IdempotencyIssue {
            line: 15,
            operation_type: NonIdempotentOperation::RmWithoutF,
            code: "rm /tmp/file.txt".to_string(),
            explanation: "rm without -f fails if file doesn't exist".to_string(),
            suggestion: "Add -f flag: rm -f".to_string(),
        }];

        // ACT: Format report
        let formatted = format_idempotency_report(&issues);

        // ASSERT: Should show rm-specific details
        assert!(formatted.contains("Line 15:"));
        assert!(formatted.contains("rm /tmp/file.txt"));
        assert!(formatted.contains("rm without -f"));
        assert!(formatted.contains("Add -f flag"));
    }

    #[test]
    fn test_REPL_012_002_format_multiple_issues() {
        // ARRANGE: Multiple issues
        let issues = vec![
            IdempotencyIssue {
                line: 5,
                operation_type: NonIdempotentOperation::MkdirWithoutP,
                code: "mkdir foo".to_string(),
                explanation: "mkdir without -p fails if directory already exists".to_string(),
                suggestion: "Add -p flag: mkdir -p".to_string(),
            },
            IdempotencyIssue {
                line: 10,
                operation_type: NonIdempotentOperation::RmWithoutF,
                code: "rm bar".to_string(),
                explanation: "rm without -f fails if file doesn't exist".to_string(),
                suggestion: "Add -f flag: rm -f".to_string(),
            },
        ];

        // ACT: Format report
        let formatted = format_idempotency_report(&issues);

        // ASSERT: Should show all issues
        assert!(formatted.contains("Line 5:"));
        assert!(formatted.contains("Line 10:"));
        assert!(formatted.contains("Found 2 issue(s)") || formatted.contains("2 issue"));
    }

    #[test]
    fn test_REPL_012_002_format_no_issues() {
        // ARRANGE: Empty issues
        let issues = vec![];

        // ACT: Format report
        let formatted = format_idempotency_report(&issues);

        // ASSERT: Should show success message
        assert!(formatted.contains("✓") || formatted.contains("success"));
        assert!(formatted.contains("idempotent"));
        assert!(formatted.contains("safe") || formatted.contains("re-run"));
    }

    #[test]
    fn test_REPL_012_002_checker_format_report() {
        // ARRANGE: Checker with detected issues
        let mut checker = IdempotencyChecker::new();
        let script = r#"
mkdir /tmp/foo
rm /tmp/bar
ln -s /tmp/baz /tmp/link
"#;
        checker.scan(script);

        // ACT: Format report
        let formatted = checker.format_report();

        // ASSERT: Should show status and breakdown
        assert!(formatted.contains("non-idempotent") || formatted.contains("issue"));
        assert!(
            formatted.contains("mkdir") || formatted.contains("rm") || formatted.contains("ln")
        );
    }

    #[test]
    fn test_REPL_012_002_checker_format_idempotent() {
        // ARRANGE: Checker with idempotent script
        let mut checker = IdempotencyChecker::new();
        let script = r#"
mkdir -p /tmp/foo
rm -f /tmp/bar
ln -sf /tmp/baz /tmp/link
"#;
        checker.scan(script);

        // ACT: Format report
        let formatted = checker.format_report();

        // ASSERT: Should show success status
        assert!(formatted.contains("✓") || formatted.contains("success"));
        assert!(formatted.contains("idempotent"));
    }

    #[test]
    fn test_REPL_012_002_format_preserves_line_numbers() {
        // ARRANGE: Issues with specific line numbers
        let issues = vec![IdempotencyIssue {
            line: 42,
            operation_type: NonIdempotentOperation::MkdirWithoutP,
            code: "mkdir test".to_string(),
            explanation: "explanation".to_string(),
            suggestion: "suggestion".to_string(),
        }];

        // ACT: Format report
        let formatted = format_idempotency_report(&issues);

        // ASSERT: Line number should be preserved
        assert!(formatted.contains("Line 42:") || formatted.contains("42"));
    }
}

#[cfg(test)]
mod idempotency_report_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_REPL_012_002_format_never_panics(
            num_issues in 0usize..10,
            line in 1usize..100,
        ) {
            // Generate arbitrary issues
            let issues: Vec<IdempotencyIssue> = (0..num_issues)
                .map(|i| IdempotencyIssue {
                    line: line + i,
                    operation_type: NonIdempotentOperation::MkdirWithoutP,
                    code: format!("mkdir /tmp/test{}", i),
                    explanation: "explanation".to_string(),
                    suggestion: "suggestion".to_string(),
                })
                .collect();

            // Format should never panic
            let _ = format_idempotency_report(&issues);
        }

        #[test]
        fn prop_REPL_012_002_empty_always_idempotent(
            _any in 0..100,
        ) {
            let formatted = format_idempotency_report(&[]);

            // Empty issues should always show idempotent
            prop_assert!(
                (formatted.contains("✓") || formatted.contains("success")) && formatted.contains("idempotent"),
                "Empty report should show idempotent: {}",
                formatted
            );
        }

        #[test]
        fn prop_REPL_012_002_count_matches_issues(
            num_issues in 1usize..20,
        ) {
            let issues: Vec<IdempotencyIssue> = (0..num_issues)
                .map(|i| IdempotencyIssue {
                    line: i + 1,
                    operation_type: NonIdempotentOperation::RmWithoutF,
                    code: format!("rm file{}", i),
                    explanation: "explanation".to_string(),
                    suggestion: "suggestion".to_string(),
                })
                .collect();

            let formatted = format_idempotency_report(&issues);

            // Count should match number of issues (flexible matching for different formats)
            let count_str = format!("{}", num_issues);
            prop_assert!(
                formatted.contains(&count_str),
                "Report should show count {}: {}",
                num_issues,
                formatted
            );
        }
    }
}

#[cfg(test)]
mod idempotency_verification_tests {
    use super::*;

    // ===== REPL-012-003: IDEMPOTENCY VERIFICATION TESTS =====

    #[test]
    fn test_REPL_012_003_verifier_new_default() {
        // ARRANGE & ACT: Create default verifier
        let verifier = IdempotencyVerifier::new();

        // ASSERT: Should have 3 runs by default
        assert_eq!(verifier.run_count, 3);
    }

    #[test]
    fn test_REPL_012_003_verifier_custom_count() {
        // ARRANGE & ACT: Create verifier with custom count
        let verifier = IdempotencyVerifier::with_run_count(5);

        // ASSERT: Should have 5 runs
        assert_eq!(verifier.run_count, 5);
    }

    #[test]
    fn test_REPL_012_003_verifier_minimum_two_runs() {
        // ARRANGE & ACT: Try to create verifier with 1 run
        let verifier = IdempotencyVerifier::with_run_count(1);

        // ASSERT: Should enforce minimum of 2 runs
        assert!(verifier.run_count >= 2);
    }

    #[test]
    fn test_REPL_012_003_idempotent_script_passes() {
        // ARRANGE: Idempotent script (echo with constant)
        let verifier = IdempotencyVerifier::new();
        let script = "echo 'hello world'";

        // ACT: Verify idempotency
        let result = verifier.verify(script);

        // ASSERT: Should pass as idempotent
        assert!(result.is_idempotent);
        assert_eq!(result.run_count, 3);
        assert!(result.differences.is_empty());
    }

    #[test]
    fn test_REPL_012_003_nonidempotent_script_fails() {
        // ARRANGE: Non-idempotent script (random)
        let verifier = IdempotencyVerifier::new();
        let script = "echo $RANDOM";

        // ACT: Verify idempotency
        let result = verifier.verify(script);

        // ASSERT: Should fail as non-idempotent
        assert!(!result.is_idempotent);
        assert_eq!(result.run_count, 3);
        assert!(!result.differences.is_empty());
    }

    #[test]
    fn test_REPL_012_003_result_format_idempotent() {
        // ARRANGE: Idempotent result
        let result = IdempotencyResult {
            is_idempotent: true,
            run_count: 3,
            runs: vec![
                RunOutput {
                    run_number: 1,
                    stdout: "hello\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
                RunOutput {
                    run_number: 2,
                    stdout: "hello\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
                RunOutput {
                    run_number: 3,
                    stdout: "hello\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            ],
            differences: vec![],
        };

        // ACT: Format result
        let formatted = result.format_result();

        // ASSERT: Should show success
        assert!(formatted.contains("✓") || formatted.contains("success"));
        assert!(formatted.contains("idempotent"));
    }

    #[test]
    fn test_REPL_012_003_result_format_nonidempotent() {
        // ARRANGE: Non-idempotent result
        let result = IdempotencyResult {
            is_idempotent: false,
            run_count: 3,
            runs: vec![
                RunOutput {
                    run_number: 1,
                    stdout: "123\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
                RunOutput {
                    run_number: 2,
                    stdout: "456\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
                RunOutput {
                    run_number: 3,
                    stdout: "789\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            ],
            differences: vec![OutputDifference {
                line: 1,
                run1: "123".to_string(),
                run2: "456".to_string(),
            }],
        };

        // ACT: Format result
        let formatted = result.format_result();

        // ASSERT: Should show failure
        assert!(
            formatted.contains("❌")
                || formatted.contains("fail")
                || formatted.contains("non-idempotent")
        );
    }
}

#[cfg(test)]
mod idempotency_verification_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_REPL_012_003_verifier_enforces_minimum(
            count in 0usize..10,
        ) {
            let verifier = IdempotencyVerifier::with_run_count(count);

            // Should always have at least 2 runs
            prop_assert!(verifier.run_count >= 2);
        }

        #[test]
        fn prop_REPL_012_003_constant_script_idempotent(
            constant in "[a-z]{1,20}",
        ) {
            let verifier = IdempotencyVerifier::new();
            let script = format!("echo '{}'", constant);

            let result = verifier.verify(&script);

            // Constant output should always be idempotent
            prop_assert!(
                result.is_idempotent,
                "Constant script should be idempotent: {}",
                script
            );
            prop_assert_eq!(result.differences.len(), 0);
        }

        #[test]
        fn prop_REPL_012_003_result_runs_match_count(
            run_count in 2usize..10,
        ) {
            let verifier = IdempotencyVerifier::with_run_count(run_count);
            let result = verifier.verify("echo 'test'");

            // Result should have exactly run_count runs
            prop_assert_eq!(result.runs.len(), run_count);
            prop_assert_eq!(result.run_count, run_count);
        }
    }
}
