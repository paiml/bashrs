//! PERF004: find -exec with \\; forks per file
//!
//! **Rule**: Detect `find -exec cmd {} \;` which forks a process per matched file
//!
//! **Why this matters**:
//! `find -exec cmd {} \;` executes `cmd` once per file found, forking a new
//! process each time. Using `find -exec cmd {} +` batches files into fewer
//! invocations, dramatically improving performance for large file sets.
//!
//! **Auto-fix**: Safe - suggest replacing `\;` with `+`
//!
//! ## Examples
//!
//! Bad (forks per file):
//! ```bash
//! find . -name "*.log" -exec rm {} \;
//! ```
//!
//! Good (batches files):
//! ```bash
//! find . -name "*.log" -exec rm {} +
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for find -exec with \; instead of +
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Match find ... -exec ... {} \;  or  find ... -exec ... {} ';'
    let pattern = Regex::new(r"\bfind\b.*\-exec\b.*\{\}\s*(\\;|';')").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        if let Some(m) = pattern.find(line) {
            let start_col = m.start() + 1;
            let end_col = m.end() + 1;

            let diagnostic = Diagnostic::new(
                "PERF004",
                Severity::Warning,
                "find -exec with \\; forks a process per file. Use + to batch: -exec cmd {} +",
                Span::new(line_num + 1, start_col, line_num + 1, end_col),
            )
            .with_fix(Fix::new("Replace \\; with + to batch file arguments"));

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf004_detects_exec_semicolon() {
        let script = r#"find . -name "*.log" -exec rm {} \;"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "PERF004");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_perf004_no_flag_exec_plus() {
        let script = r#"find . -name "*.log" -exec rm {} +"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_perf004_provides_fix() {
        let script = r#"find . -name "*.log" -exec rm {} \;"#;
        let result = check(script);
        assert!(result.diagnostics[0].fix.is_some());
    }

    #[test]
    fn test_perf004_no_false_positive_comment() {
        let script = r#"# find . -name "*.log" -exec rm {} \;"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_perf004_detects_with_complex_command() {
        let script = r#"find /var/log -type f -name "*.gz" -exec gzip -d {} \;"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
