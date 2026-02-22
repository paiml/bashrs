// Format & Coverage Logic Module - Extracted for testability
//
// This module contains pure functions for format checking, config loading,
// and coverage data formatting. No I/O or printing.
//
// Architecture:
// - format_logic.rs: Pure format/coverage computation (no I/O)
// - commands.rs: Thin I/O shim (reads files, calls logic, prints output)

use std::collections::BTreeMap;
use std::path::Path;

// =============================================================================
// FORMAT COMMAND LOGIC
// =============================================================================

/// Result of checking whether a file needs formatting
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatCheckStatus {
    /// File is already properly formatted
    Formatted,
    /// File needs formatting (has differences)
    NeedsFormatting,
}

impl FormatCheckStatus {
    pub fn is_formatted(&self) -> bool {
        matches!(self, Self::Formatted)
    }
}

/// Check if source and formatted content are equivalent.
/// Pure function: compares trimmed content and returns a status.
pub fn check_format_status(source: &str, formatted: &str) -> FormatCheckStatus {
    if source.trim() == formatted.trim() {
        FormatCheckStatus::Formatted
    } else {
        FormatCheckStatus::NeedsFormatting
    }
}

/// Determine the appropriate config file path for a given input file.
/// Returns the first existing config path, or None if no config file is found.
///
/// Search order:
/// 1. `.bashrs-fmt.toml` in the same directory as the input file
/// 2. `.bashrs-fmt.toml` in the current directory
pub fn find_format_config(input_path: &Path) -> Option<std::path::PathBuf> {
    // Check parent directory of input file
    if let Some(parent) = input_path.parent() {
        let script_dir_config = parent.join(".bashrs-fmt.toml");
        if script_dir_config.exists() {
            return Some(script_dir_config);
        }
    }

    // Check current directory
    let cwd_config = std::path::PathBuf::from(".bashrs-fmt.toml");
    if cwd_config.exists() {
        return Some(cwd_config);
    }

    None
}

/// Describe what would happen during a dry-run of formatting.
/// Returns a description string without performing any I/O.
pub fn describe_dry_run(source: &str, formatted: &str) -> &'static str {
    if source.trim() != formatted.trim() {
        "Changes detected"
    } else {
        "No changes needed"
    }
}

// =============================================================================
// COVERAGE DATA STRUCTURES & LOGIC
// =============================================================================

/// Summary of coverage data, suitable for rendering in any format.
#[derive(Debug, Clone)]
pub struct CoverageSummary {
    pub total_lines: usize,
    pub covered_lines: usize,
    pub line_percent: f64,
    pub total_functions: usize,
    pub covered_functions: usize,
    pub function_percent: f64,
    pub uncovered_line_numbers: Vec<usize>,
    pub uncovered_function_names: Vec<String>,
}

impl CoverageSummary {
    /// Build a summary from coverage report data.
    pub fn from_report(
        total_lines: usize,
        covered_lines_count: usize,
        line_percent: f64,
        all_functions: &[String],
        covered_functions: &[String],
        function_percent: f64,
        uncovered_lines: Vec<usize>,
        uncovered_functions: Vec<String>,
    ) -> Self {
        Self {
            total_lines,
            covered_lines: covered_lines_count,
            line_percent,
            total_functions: all_functions.len(),
            covered_functions: covered_functions.len(),
            function_percent,
            uncovered_line_numbers: uncovered_lines,
            uncovered_function_names: uncovered_functions,
        }
    }

    /// Get status message for terminal display
    pub fn terminal_status(&self) -> &'static str {
        if self.total_lines == 0 {
            "No executable code found"
        } else if self.covered_lines == 0 {
            "No tests found - 0% coverage"
        } else if self.line_percent >= 80.0 {
            "Good coverage!"
        } else if self.line_percent >= 50.0 {
            "Moderate coverage - consider adding more tests"
        } else {
            "Low coverage - more tests needed"
        }
    }
}

/// CSS class name for coverage percentage (used in HTML reports).
pub fn coverage_css_class(percent: f64) -> &'static str {
    if percent >= 80.0 {
        "good"
    } else if percent >= 50.0 {
        "medium"
    } else {
        "poor"
    }
}

/// Generate HTML coverage report content as a string (pure function).
pub fn generate_html_coverage(
    input_display: &str,
    line_percent: f64,
    covered_lines: usize,
    total_lines: usize,
    function_percent: f64,
    covered_functions: usize,
    total_functions: usize,
    uncovered_function_names: &[String],
) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Coverage Report - {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        h1 {{ color: #333; }}
        .summary {{ background: #f5f5f5; padding: 15px; border-radius: 5px; }}
        .coverage {{ font-size: 24px; font-weight: bold; }}
        .good {{ color: #28a745; }}
        .medium {{ color: #ffc107; }}
        .poor {{ color: #dc3545; }}
        table {{ border-collapse: collapse; width: 100%; margin-top: 20px; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
        .covered {{ background-color: #d4edda; }}
        .uncovered {{ background-color: #f8d7da; }}
    </style>
</head>
<body>
    <h1>Coverage Report</h1>
    <h2>{}</h2>
    <div class="summary">
        <p><strong>Line Coverage:</strong>
            <span class="coverage {}">{:.1}%</span>
            ({}/{})</p>
        <p><strong>Function Coverage:</strong>
            <span class="coverage {}">{:.1}%</span>
            ({}/{})</p>
    </div>
    <h3>Uncovered Functions</h3>
    <ul>
        {}
    </ul>
</body>
</html>"#,
        input_display,
        input_display,
        coverage_css_class(line_percent),
        line_percent,
        covered_lines,
        total_lines,
        coverage_css_class(function_percent),
        function_percent,
        covered_functions,
        total_functions,
        uncovered_function_names
            .iter()
            .map(|f| format!("<li>{}</li>", f))
            .collect::<Vec<_>>()
            .join("\n        ")
    )
}

/// Generate LCOV format coverage data as a string (pure function).
pub fn generate_lcov_coverage(
    input_display: &str,
    all_functions: &[String],
    covered_functions: &[String],
    line_coverage: &BTreeMap<usize, bool>,
    total_lines: usize,
    covered_lines_count: usize,
) -> String {
    let mut output = String::new();
    output.push_str("TN:\n");
    output.push_str(&format!("SF:{}\n", input_display));

    // Function coverage
    for func in all_functions {
        let covered = if covered_functions.contains(func) {
            1
        } else {
            0
        };
        output.push_str(&format!("FN:0,{}\n", func));
        output.push_str(&format!("FNDA:{},{}\n", covered, func));
    }
    output.push_str(&format!("FNF:{}\n", all_functions.len()));
    output.push_str(&format!("FNH:{}\n", covered_functions.len()));

    // Line coverage
    for (line_num, &is_covered) in line_coverage {
        let hit = if is_covered { 1 } else { 0 };
        output.push_str(&format!("DA:{},{}\n", line_num, hit));
    }
    output.push_str(&format!("LF:{}\n", total_lines));
    output.push_str(&format!("LH:{}\n", covered_lines_count));

    output.push_str("end_of_record\n");
    output
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use std::collections::BTreeMap;

    // ===== check_format_status + describe_dry_run tests =====

    #[test]
    fn test_check_format_status_cases() {
        // Identical → Formatted
        assert_eq!(check_format_status("echo hello", "echo hello"), FormatCheckStatus::Formatted);
        // Trailing whitespace only difference → Formatted (trim)
        assert_eq!(check_format_status("echo hello  \n", "echo hello\n"), FormatCheckStatus::Formatted);
        // Empty → Formatted
        assert_eq!(check_format_status("", ""), FormatCheckStatus::Formatted);
        // Whitespace-only both sides → Formatted (trim)
        assert_eq!(check_format_status("   \n  ", "  \n "), FormatCheckStatus::Formatted);
        // Different content → NeedsFormatting
        assert_eq!(check_format_status("echo hello", "echo   hello"), FormatCheckStatus::NeedsFormatting);
        // is_formatted helper
        assert!(FormatCheckStatus::Formatted.is_formatted());
        assert!(!FormatCheckStatus::NeedsFormatting.is_formatted());
    }

    #[test]
    fn test_describe_dry_run_cases() {
        assert_eq!(describe_dry_run("abc", "def"), "Changes detected");
        assert_eq!(describe_dry_run("abc", "abc"), "No changes needed");
        // Trailing whitespace treated as no change
        assert_eq!(describe_dry_run("abc  ", "abc"), "No changes needed");
    }

    // ===== CoverageSummary tests =====

    #[test]
    fn test_coverage_summary_terminal_status_all_cases() {
        let make = |total_lines, covered_lines, line_percent| CoverageSummary {
            total_lines,
            covered_lines,
            line_percent,
            total_functions: 0,
            covered_functions: 0,
            function_percent: 0.0,
            uncovered_line_numbers: vec![],
            uncovered_function_names: vec![],
        };
        assert_eq!(make(0, 0, 0.0).terminal_status(), "No executable code found");
        assert_eq!(make(10, 0, 0.0).terminal_status(), "No tests found - 0% coverage");
        assert_eq!(make(100, 85, 85.0).terminal_status(), "Good coverage!");
        assert_eq!(make(100, 60, 60.0).terminal_status(), "Moderate coverage - consider adding more tests");
        assert_eq!(make(100, 30, 30.0).terminal_status(), "Low coverage - more tests needed");
    }

    #[test]
    fn test_coverage_summary_from_report() {
        let all_funcs = vec!["fn1".to_string(), "fn2".to_string(), "fn3".to_string()];
        let covered_funcs = vec!["fn1".to_string(), "fn2".to_string()];
        let summary = CoverageSummary::from_report(
            100,
            80,
            80.0,
            &all_funcs,
            &covered_funcs,
            66.7,
            vec![5, 10, 15],
            vec!["fn3".to_string()],
        );
        assert_eq!(summary.total_lines, 100);
        assert_eq!(summary.covered_lines, 80);
        assert!((summary.line_percent - 80.0).abs() < 0.001);
        assert_eq!(summary.total_functions, 3);
        assert_eq!(summary.covered_functions, 2);
        assert_eq!(summary.uncovered_line_numbers, vec![5, 10, 15]);
        assert_eq!(summary.uncovered_function_names, vec!["fn3"]);
    }

    // ===== coverage_css_class tests =====

    #[test]
    fn test_coverage_css_class_boundaries() {
        assert_eq!(coverage_css_class(80.0), "good");
        assert_eq!(coverage_css_class(100.0), "good");
        assert_eq!(coverage_css_class(50.0), "medium");
        assert_eq!(coverage_css_class(79.9), "medium");
        assert_eq!(coverage_css_class(49.9), "poor");
        assert_eq!(coverage_css_class(0.0), "poor");
    }

    // ===== generate_html_coverage tests =====

    #[test]
    fn test_generate_html_coverage_contains_key_elements() {
        let html = generate_html_coverage(
            "test.sh",
            85.0,
            85,
            100,
            90.0,
            9,
            10,
            &["uncovered_fn".to_string()],
        );
        assert!(html.contains("Coverage Report"));
        assert!(html.contains("test.sh"));
        assert!(html.contains("85.0%"));
        assert!(html.contains("90.0%"));
        assert!(html.contains("85"));
        assert!(html.contains("100"));
        assert!(html.contains("<li>uncovered_fn</li>"));
        assert!(html.contains("class=\"coverage good\""));
    }

    #[test]
    fn test_generate_html_coverage_poor_coverage() {
        let html = generate_html_coverage("low.sh", 30.0, 30, 100, 20.0, 2, 10, &[]);
        assert!(html.contains("class=\"coverage poor\""));
    }

    #[test]
    fn test_generate_html_coverage_no_uncovered_functions() {
        let html = generate_html_coverage("perfect.sh", 100.0, 100, 100, 100.0, 10, 10, &[]);
        assert!(html.contains("<ul>"));
        // No <li> items
        assert!(!html.contains("<li>"));
    }

    // ===== generate_lcov_coverage tests =====

    #[test]
    fn test_generate_lcov_coverage_basic() {
        let mut line_cov = BTreeMap::new();
        line_cov.insert(1, true);
        line_cov.insert(2, false);
        line_cov.insert(3, true);

        let all_funcs = vec!["main".to_string(), "helper".to_string()];
        let covered_funcs = vec!["main".to_string()];

        let lcov = generate_lcov_coverage("test.sh", &all_funcs, &covered_funcs, &line_cov, 3, 2);

        assert!(lcov.contains("TN:"));
        assert!(lcov.contains("SF:test.sh"));
        assert!(lcov.contains("FN:0,main"));
        assert!(lcov.contains("FN:0,helper"));
        assert!(lcov.contains("FNDA:1,main"));
        assert!(lcov.contains("FNDA:0,helper"));
        assert!(lcov.contains("FNF:2"));
        assert!(lcov.contains("FNH:1"));
        assert!(lcov.contains("DA:1,1"));
        assert!(lcov.contains("DA:2,0"));
        assert!(lcov.contains("DA:3,1"));
        assert!(lcov.contains("LF:3"));
        assert!(lcov.contains("LH:2"));
        assert!(lcov.contains("end_of_record"));
    }

    #[test]
    fn test_generate_lcov_coverage_empty() {
        let line_cov = BTreeMap::new();
        let lcov = generate_lcov_coverage("empty.sh", &[], &[], &line_cov, 0, 0);
        assert!(lcov.contains("TN:"));
        assert!(lcov.contains("SF:empty.sh"));
        assert!(lcov.contains("FNF:0"));
        assert!(lcov.contains("FNH:0"));
        assert!(lcov.contains("LF:0"));
        assert!(lcov.contains("LH:0"));
        assert!(lcov.contains("end_of_record"));
    }

    // ===== find_format_config tests =====
    // Note: These are unit tests for the path-building logic.
    // They test with non-existent paths (both return None since no file exists).

    #[test]
    fn test_find_format_config_no_config_exists() {
        let result = find_format_config(Path::new("/nonexistent/dir/script.sh"));
        assert!(result.is_none());
    }
}
