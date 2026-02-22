//! Pure logic functions for config file (bashrc/zshrc/profile) operations.
//!
//! This module contains functions extracted from `commands.rs` that have no
//! filesystem side-effects. They operate purely on data and return `String`
//! or simple values, making them straightforward to unit-test in isolation.

use crate::config::{ConfigAnalysis, ConfigIssue, PathEntry, Severity};
use std::path::Path;

// ---------------------------------------------------------------------------
// Output routing helpers
// ---------------------------------------------------------------------------

/// Returns `true` when the output path represents stdout (i.e. `"-"`).
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use bashrs::cli::config_logic::should_output_to_stdout;
///
/// assert!(should_output_to_stdout(Path::new("-")));
/// assert!(!should_output_to_stdout(Path::new("output.sh")));
/// ```
pub(crate) fn should_output_to_stdout(output_path: &Path) -> bool {
    output_path.to_str() == Some("-")
}

// ---------------------------------------------------------------------------
// Analysis helpers
// ---------------------------------------------------------------------------

/// Count the number of duplicate PATH entries in an analysis result.
///
/// # Examples
///
/// ```
/// use bashrs::config::{ConfigAnalysis, PathEntry, ConfigType, ConfigIssue, PerformanceIssue};
/// use std::path::PathBuf;
/// use bashrs::cli::config_logic::count_duplicate_path_entries;
///
/// let analysis = ConfigAnalysis {
///     file_path: PathBuf::from(".bashrc"),
///     config_type: ConfigType::Bashrc,
///     line_count: 10,
///     complexity_score: 1,
///     issues: vec![],
///     path_entries: vec![
///         PathEntry { line: 1, path: "/usr/bin".to_string(), is_duplicate: false },
///         PathEntry { line: 2, path: "/usr/bin".to_string(), is_duplicate: true },
///     ],
///     performance_issues: vec![],
/// };
/// assert_eq!(count_duplicate_path_entries(&analysis), 1);
/// ```
pub(crate) fn count_duplicate_path_entries(analysis: &ConfigAnalysis) -> usize {
    analysis
        .path_entries
        .iter()
        .filter(|e| e.is_duplicate)
        .count()
}

// ---------------------------------------------------------------------------
// Severity formatting helpers
// ---------------------------------------------------------------------------

/// Return a human-readable severity label string (lowercase).
///
/// Used by both lint human and JSON formatters.
pub(crate) fn severity_to_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}

/// Return a single-character marker symbol for a severity level.
///
/// - Error   → `"✗"`
/// - Warning → `"⚠"`
/// - Info    → `"ℹ"`
pub(crate) fn severity_to_marker(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "\u{2717}",   // ✗
        Severity::Warning => "\u{26A0}", // ⚠
        Severity::Info => "\u{2139}",   // ℹ
    }
}

// ---------------------------------------------------------------------------
// Analyze human formatter
// ---------------------------------------------------------------------------

/// Format a single PATH entry line for human-readable output.
///
/// Duplicate entries are prefixed with `"  ✗"`, unique ones with `"  ✓"`.
pub(crate) fn format_path_entry(entry: &PathEntry) -> String {
    let marker = if entry.is_duplicate { "  \u{2717}" } else { "  \u{2713}" };
    format!("{}  Line {}: {}", marker, entry.line, entry.path)
}

/// Format the issues section of a human-readable config analysis.
///
/// Returns an empty string when `issues` is empty (caller prints the
/// "No issues found" message).
pub(crate) fn format_config_analyze_human_issues(issues: &[ConfigIssue]) -> String {
    if issues.is_empty() {
        return "\u{2713} No issues found\n".to_string();
    }

    let mut out = format!("Issues Found: {}\n", issues.len());
    for issue in issues {
        let marker = severity_to_marker(issue.severity);
        out.push_str(&format!(
            "  {} [{}] Line {}: {}\n",
            marker, issue.rule_id, issue.line, issue.message
        ));
        if let Some(suggestion) = &issue.suggestion {
            out.push_str(&format!("    \u{2192} {}\n", suggestion));
        }
    }
    out
}

/// Format a full human-readable config analysis report as a `String`.
///
/// This is the pure-logic counterpart to the former `config_analyze_human`
/// function in `commands.rs`. The caller is responsible for printing the
/// returned string.
pub(crate) fn format_config_analyze_human(
    input_name: &str,
    analysis: &ConfigAnalysis,
) -> String {
    let separator = "=".repeat(input_name.len());
    let mut out = format!("Analysis: {}\n", input_name);
    out.push_str(&format!("========={}=\n", separator));
    out.push('\n');
    out.push_str("Statistics:\n");
    out.push_str(&format!("  - Lines: {}\n", analysis.line_count));
    out.push_str(&format!(
        "  - Complexity score: {}/10\n",
        analysis.complexity_score
    ));
    out.push_str(&format!(
        "  - Config type: {:?}\n",
        analysis.config_type
    ));
    out.push('\n');

    if !analysis.path_entries.is_empty() {
        out.push_str(&format!(
            "PATH Entries ({}):\n",
            analysis.path_entries.len()
        ));
        for entry in &analysis.path_entries {
            out.push_str(&format_path_entry(entry));
            out.push('\n');
        }
        out.push('\n');
    }

    if !analysis.performance_issues.is_empty() {
        out.push_str(&format!(
            "Performance Issues ({}):\n",
            analysis.performance_issues.len()
        ));
        for issue in &analysis.performance_issues {
            out.push_str(&format!(
                "  - Line {}: {} (~{}ms)\n",
                issue.line, issue.command, issue.estimated_cost_ms
            ));
            out.push_str(&format!("    Suggestion: {}\n", issue.suggestion));
        }
        out.push('\n');
    }

    out.push_str(&format_config_analyze_human_issues(&analysis.issues));
    out
}

// ---------------------------------------------------------------------------
// Analyze JSON formatter
// ---------------------------------------------------------------------------

/// Format a config analysis result as a JSON string.
///
/// This is the pure-logic counterpart to the former `config_analyze_json`
/// function in `commands.rs`.
pub(crate) fn format_config_analyze_json(
    input_name: &str,
    analysis: &ConfigAnalysis,
) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("  \"file\": \"{}\",\n", input_name));
    out.push_str(&format!("  \"line_count\": {},\n", analysis.line_count));
    out.push_str(&format!(
        "  \"complexity_score\": {},\n",
        analysis.complexity_score
    ));
    out.push_str(&format!(
        "  \"path_entries\": {},\n",
        analysis.path_entries.len()
    ));
    out.push_str(&format!(
        "  \"performance_issues\": {},\n",
        analysis.performance_issues.len()
    ));
    out.push_str("  \"issues\": [\n");

    let issue_count = analysis.issues.len();
    for (i, issue) in analysis.issues.iter().enumerate() {
        let comma = if i < issue_count - 1 { "," } else { "" };
        out.push_str("    {\n");
        out.push_str(&format!("      \"rule_id\": \"{}\",\n", issue.rule_id));
        out.push_str(&format!("      \"line\": {},\n", issue.line));
        out.push_str(&format!(
            "      \"message\": \"{}\"\n",
            issue.message
        ));
        out.push_str(&format!("    }}{}\n", comma));
    }

    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

// ---------------------------------------------------------------------------
// Lint formatters
// ---------------------------------------------------------------------------

/// Format config lint results as a human-readable string.
///
/// Returns `None` when there are no issues (caller prints "no issues" message).
pub(crate) fn format_config_lint_human(
    input_name: &str,
    analysis: &ConfigAnalysis,
) -> Option<String> {
    if analysis.issues.is_empty() {
        return None;
    }

    let mut out = String::new();
    for issue in &analysis.issues {
        let severity = severity_to_label(issue.severity);
        out.push_str(&format!(
            "{}:{}:{}: {}: {} [{}]\n",
            input_name,
            issue.line,
            issue.column,
            severity,
            issue.message,
            issue.rule_id
        ));
        if let Some(suggestion) = &issue.suggestion {
            out.push_str(&format!("  suggestion: {}\n", suggestion));
        }
    }
    Some(out)
}

/// Format config lint results as a JSON string.
pub(crate) fn format_config_lint_json(
    input_name: &str,
    analysis: &ConfigAnalysis,
) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("  \"file\": \"{}\",\n", input_name));
    out.push_str("  \"issues\": [\n");

    let issue_count = analysis.issues.len();
    for (i, issue) in analysis.issues.iter().enumerate() {
        let comma = if i < issue_count - 1 { "," } else { "" };
        out.push_str("    {\n");
        out.push_str(&format!("      \"rule_id\": \"{}\",\n", issue.rule_id));
        out.push_str(&format!("      \"line\": {},\n", issue.line));
        out.push_str(&format!("      \"column\": {},\n", issue.column));
        out.push_str(&format!(
            "      \"message\": \"{}\"\n",
            issue.message
        ));
        out.push_str(&format!("    }}{}\n", comma));
    }

    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

// ---------------------------------------------------------------------------
// Dry-run / preview helpers
// ---------------------------------------------------------------------------

/// Format the dry-run preview header for a config purify operation.
///
/// Returns the header text (excluding the diff body) as a `String`.
pub(crate) fn format_dry_run_header(input_name: &str, issue_count: usize) -> String {
    let separator = "=".repeat(input_name.len());
    let mut out = format!("Preview of changes to {}:\n", input_name);
    out.push_str(&format!("================================{}=\n", separator));
    out.push('\n');

    if issue_count == 0 {
        out.push_str("\u{2713} No issues found - file is already clean!\n");
    } else {
        out.push_str(&format!("Would fix {} issue(s):\n", issue_count));
    }
    out
}

/// Format a single issue entry for the dry-run preview.
pub(crate) fn format_dry_run_issue(rule_id: &str, message: &str) -> String {
    format!("  - {}: {}\n", rule_id, message)
}

/// Format the diff section header for a dry-run preview.
pub(crate) fn format_diff_header(input_name: &str) -> String {
    let mut out = String::new();
    out.push('\n');
    out.push_str(&format!("--- {} (original)\n", input_name));
    out.push_str(&format!("+++ {} (purified)\n", input_name));
    out.push('\n');
    out
}

/// Format a single diff line pair (original → purified).
pub(crate) fn format_diff_line(line_num: usize, orig: &str, purified: &str) -> String {
    format!("-{}: {}\n+{}: {}\n", line_num, orig, line_num, purified)
}

/// Format the footer suggestion for applying fixes.
pub(crate) fn format_apply_fix_suggestion(input_name: &str) -> String {
    format!("\nApply fixes: bashrs config purify {} --fix\n", input_name)
}

// ---------------------------------------------------------------------------
// Tests — in separate file to keep this module under 500 lines
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "config_logic_tests.rs"]
mod tests;
