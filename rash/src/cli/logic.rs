// CLI Logic Module - Extracted for testability
//
// This module contains pure functions that return structured results
// instead of printing directly. The commands.rs file acts as a thin shim
// that calls these functions and handles I/O.
//
// Architecture:
// - logic.rs: Hub that re-exports from focused submodules
// - logic_lint.rs: Lint processing, filtering, and rule handling
// - logic_dockerfile.rs: Dockerfile purification, analysis, build helpers
// - logic_format.rs: Formatting, display, and report generation
// - logic_shell.rs: Shell detection, normalization, file type detection
// - logic_validate.rs: Validation, verification, and checking
// - commands.rs: Thin I/O shim (reads files, calls logic, prints output)
//
// This separation enables:
// - Unit testing of all business logic
// - High test coverage (95%+ target)
// - Clear separation of concerns

// =============================================================================
// SUBMODULES
// =============================================================================

#[path = "logic_lint.rs"]
mod lint;
#[path = "logic_dockerfile.rs"]
mod dockerfile;
#[path = "logic_format.rs"]
mod format;
#[path = "logic_shell.rs"]
mod shell;
#[path = "logic_validate.rs"]
mod validate;

// =============================================================================
// RE-EXPORTS - All public items from submodules
// =============================================================================

// From logic_lint.rs
pub use lint::{
    build_ignored_rules, convert_lint_profile, determine_min_severity, diagnostic_matches_rules,
    filter_diagnostics, filter_diagnostics_by_rules, parse_rule_codes, parse_rule_filter,
    process_lint, process_purify_bash, severity_icon, FileType, LintDiagnostic, LintOptions,
    LintProcessResult, PurificationStats, PurifyProcessResult, Transformation,
};

// From logic_dockerfile.rs
pub use dockerfile::{
    add_no_install_recommends, add_package_manager_cleanup, convert_add_to_copy_if_local,
    dockerfile_find_cmd_line, dockerfile_has_user_directive, dockerfile_is_scratch,
    estimate_build_time_seconds, find_devcontainer_json, format_build_time,
    format_build_time_estimate, format_size_comparison, layer_has_slow_operation,
    parse_size_limit, parse_size_string, pin_base_image_version, purify_dockerfile_source,
    size_exceeds_limit, size_percentage_of_limit,
};

// From logic_format.rs
pub use format::{
    calculate_percentage, coverage_class, coverage_status, format_bytes_human,
    format_duration_human, format_purify_report_human, format_purify_report_json,
    format_purify_report_markdown, format_score_human, format_timestamp, generate_diff_lines,
    grade_interpretation, grade_symbol, hex_encode, score_status, test_pass_rate,
    test_result_status, truncate_str,
};

// From logic_shell.rs
pub use shell::{
    count_duplicate_path_entries, detect_platform, is_dockerfile, is_makefile, is_shell_script_file,
    is_stdio_path, normalize_shell_script, parse_shell_dialect, should_output_to_stdout,
};

// From logic_validate.rs
pub use validate::{
    extract_exit_code, process_check, validate_gate_tier, validate_proof_data, verify_scripts,
    CheckResult, GateResult, VerifyResult,
};

// =============================================================================
// TYPES THAT REMAIN IN THE HUB
// (Data-only types with no complex logic or cross-module dependencies)
// =============================================================================

/// Result of test processing
#[derive(Debug, Clone)]
pub struct TestProcessResult {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total: usize,
    pub test_results: Vec<TestCaseResult>,
    pub duration_ms: u64,
}

/// Result of a single test case
#[derive(Debug, Clone)]
pub struct TestCaseResult {
    pub name: String,
    pub passed: bool,
    pub message: Option<String>,
    pub duration_ms: u64,
}

impl TestProcessResult {
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            (self.passed as f64 / self.total as f64) * 100.0
        }
    }

    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

// =============================================================================
// SCORE COMMAND LOGIC
// =============================================================================

/// Result of score processing
#[derive(Debug, Clone)]
pub struct ScoreProcessResult {
    pub overall_score: f64,
    pub grade: Grade,
    pub dimensions: Vec<ScoreDimension>,
}

/// A dimension of the quality score
#[derive(Debug, Clone)]
pub struct ScoreDimension {
    pub name: String,
    pub score: f64,
    pub max_score: f64,
    pub status: &'static str,
}

// =============================================================================
// AUDIT COMMAND LOGIC
// =============================================================================

/// Result of audit processing
#[derive(Debug, Clone)]
pub struct AuditProcessResult {
    pub parse_success: bool,
    pub parse_error: Option<String>,
    pub lint_errors: usize,
    pub lint_warnings: usize,
    pub test_passed: usize,
    pub test_failed: usize,
    pub test_total: usize,
    pub score: Option<ScoreProcessResult>,
    pub overall_pass: bool,
    pub failure_reason: Option<String>,
}

impl AuditProcessResult {
    pub fn passed(&self) -> bool {
        self.overall_pass
    }
}

// =============================================================================
// COVERAGE COMMAND LOGIC
// =============================================================================

/// Result of coverage processing
#[derive(Debug, Clone)]
pub struct CoverageProcessResult {
    pub line_coverage: f64,
    pub function_coverage: f64,
    pub total_lines: usize,
    pub covered_lines: usize,
    pub total_functions: usize,
    pub covered_functions: usize,
    pub uncovered_lines: Vec<usize>,
    pub uncovered_functions: Vec<String>,
}

impl CoverageProcessResult {
    pub fn meets_threshold(&self, min_percent: f64) -> bool {
        self.line_coverage >= min_percent
    }
}

// =============================================================================
// FORMAT COMMAND LOGIC
// =============================================================================

/// Result of format processing
#[derive(Debug, Clone)]
pub struct FormatProcessResult {
    pub original: String,
    pub formatted: String,
    pub changed: bool,
    pub diff_lines: Vec<(usize, String, String)>,
}

/// Result of format check
#[derive(Debug, Clone)]
pub struct FormatCheckResult {
    pub files_checked: usize,
    pub files_formatted: usize,
    pub files_unchanged: usize,
}

impl FormatCheckResult {
    pub fn all_formatted(&self) -> bool {
        self.files_formatted == 0
    }
}

// =============================================================================
// GRADE LOGIC
// =============================================================================

/// Grade calculation based on score
/// Note: Higher grades (A) are "better" than lower grades (F)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    A,
    B,
    C,
    D,
    F,
}

impl PartialOrd for Grade {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Grade {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // A is the best (highest), F is the worst (lowest)
        // So A > B > C > D > F
        let self_val = match self {
            Grade::A => 4,
            Grade::B => 3,
            Grade::C => 2,
            Grade::D => 1,
            Grade::F => 0,
        };
        let other_val = match other {
            Grade::A => 4,
            Grade::B => 3,
            Grade::C => 2,
            Grade::D => 1,
            Grade::F => 0,
        };
        self_val.cmp(&other_val)
    }
}

impl Grade {
    pub fn from_score(score: f64) -> Self {
        if score >= 90.0 {
            Grade::A
        } else if score >= 80.0 {
            Grade::B
        } else if score >= 70.0 {
            Grade::C
        } else if score >= 60.0 {
            Grade::D
        } else {
            Grade::F
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Grade::A => "A",
            Grade::B => "B",
            Grade::C => "C",
            Grade::D => "D",
            Grade::F => "F",
        }
    }

    pub fn meets_minimum(&self, min: &str) -> bool {
        let min_grade = match min.to_uppercase().as_str() {
            "A" => Grade::A,
            "B" => Grade::B,
            "C" => Grade::C,
            "D" => Grade::D,
            _ => Grade::F,
        };
        *self >= min_grade
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== TEST PROCESS RESULT TESTS =====

    #[test]
    fn test_test_process_result_success_rate() {
        let result = TestProcessResult {
            passed: 8,
            failed: 2,
            skipped: 0,
            total: 10,
            test_results: vec![],
            duration_ms: 100,
        };

        assert_eq!(result.success_rate(), 80.0);
        assert!(!result.all_passed());
    }

    #[test]
    fn test_test_process_result_all_passed() {
        let result = TestProcessResult {
            passed: 10,
            failed: 0,
            skipped: 0,
            total: 10,
            test_results: vec![],
            duration_ms: 100,
        };

        assert_eq!(result.success_rate(), 100.0);
        assert!(result.all_passed());
    }

    #[test]
    fn test_test_process_result_empty() {
        let result = TestProcessResult {
            passed: 0,
            failed: 0,
            skipped: 0,
            total: 0,
            test_results: vec![],
            duration_ms: 0,
        };

        assert_eq!(result.success_rate(), 100.0);
        assert!(result.all_passed());
    }

    // ===== COVERAGE PROCESS RESULT TESTS =====

    #[test]
    fn test_coverage_process_result_threshold() {
        let result = CoverageProcessResult {
            line_coverage: 85.0,
            function_coverage: 90.0,
            total_lines: 100,
            covered_lines: 85,
            total_functions: 10,
            covered_functions: 9,
            uncovered_lines: vec![1, 2, 3],
            uncovered_functions: vec!["foo".to_string()],
        };

        assert!(result.meets_threshold(80.0));
        assert!(result.meets_threshold(85.0));
        assert!(!result.meets_threshold(90.0));
    }

    // ===== AUDIT PROCESS RESULT TESTS =====

    #[test]
    fn test_audit_process_result_passed() {
        let result = AuditProcessResult {
            parse_success: true,
            parse_error: None,
            lint_errors: 0,
            lint_warnings: 2,
            test_passed: 10,
            test_failed: 0,
            test_total: 10,
            score: None,
            overall_pass: true,
            failure_reason: None,
        };

        assert!(result.passed());
    }

    #[test]
    fn test_audit_process_result_failed() {
        let result = AuditProcessResult {
            parse_success: true,
            parse_error: None,
            lint_errors: 5,
            lint_warnings: 2,
            test_passed: 8,
            test_failed: 2,
            test_total: 10,
            score: None,
            overall_pass: false,
            failure_reason: Some("Lint errors found".to_string()),
        };

        assert!(!result.passed());
    }

    // ===== GRADE TESTS =====

    #[test]
    fn test_grade_from_score() {
        assert_eq!(Grade::from_score(95.0), Grade::A);
        assert_eq!(Grade::from_score(90.0), Grade::A);
        assert_eq!(Grade::from_score(85.0), Grade::B);
        assert_eq!(Grade::from_score(80.0), Grade::B);
        assert_eq!(Grade::from_score(75.0), Grade::C);
        assert_eq!(Grade::from_score(65.0), Grade::D);
        assert_eq!(Grade::from_score(55.0), Grade::F);
    }

    #[test]
    fn test_grade_meets_minimum() {
        assert!(Grade::A.meets_minimum("A"));
        assert!(Grade::A.meets_minimum("B"));
        assert!(Grade::A.meets_minimum("C"));
        assert!(!Grade::B.meets_minimum("A"));
        assert!(Grade::B.meets_minimum("B"));
        assert!(Grade::C.meets_minimum("D"));
        assert!(!Grade::D.meets_minimum("C"));
    }

    // ===== FORMAT CHECK RESULT TESTS =====

    #[test]
    fn test_format_check_result_all_formatted() {
        let result = FormatCheckResult {
            files_checked: 5,
            files_formatted: 0,
            files_unchanged: 5,
        };
        assert!(result.all_formatted());
    }

    #[test]
    fn test_format_check_result_needs_formatting() {
        let result = FormatCheckResult {
            files_checked: 5,
            files_formatted: 2,
            files_unchanged: 3,
        };
        assert!(!result.all_formatted());
    }
}
