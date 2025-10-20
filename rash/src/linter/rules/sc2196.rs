//! SC2196: egrep is deprecated and non-standard. Use grep -E instead.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! egrep 'pattern' file.txt
//! fgrep 'literal' file.txt
//! ```
//!
//! Good:
//! ```bash
//! grep -E 'pattern' file.txt
//! grep -F 'literal' file.txt
//! ```
//!
//! # Rationale
//!
//! egrep and fgrep are deprecated:
//! - Not in POSIX standard
//! - Removed from newer systems
//! - Use grep with flags instead
//!
//! Use grep -E (extended) or grep -F (fixed strings).
//!
//! # Auto-fix
//!
//! Replace egrep with grep -E, fgrep with grep -F

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for deprecated egrep/fgrep commands
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: egrep or fgrep
    let egrep_pattern = Regex::new(r"\begrep\b").unwrap();
    let fgrep_pattern = Regex::new(r"\bfgrep\b").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for egrep
        for cap in egrep_pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2196",
                Severity::Warning,
                "egrep is deprecated and non-standard. Use grep -E instead.",
                Span::new(line_num, start_col, line_num, end_col),
            )
            .with_fix(Fix::new("grep -E"));

            result.add(diagnostic);
        }

        // Check for fgrep
        for cap in fgrep_pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2196",
                Severity::Warning,
                "fgrep is deprecated and non-standard. Use grep -F instead.",
                Span::new(line_num, start_col, line_num, end_col),
            )
            .with_fix(Fix::new("grep -F"));

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2196_egrep_detection() {
        let script = "egrep 'pattern' file.txt";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2196");
    }

    #[test]
    fn test_sc2196_egrep_autofix() {
        let script = "egrep 'pattern' file.txt";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "grep -E"
        );
    }

    #[test]
    fn test_sc2196_fgrep_detection() {
        let script = "fgrep 'literal' file.txt";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2196_fgrep_autofix() {
        let script = "fgrep 'literal' file.txt";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "grep -F"
        );
    }

    #[test]
    fn test_sc2196_false_positive_grep_e() {
        let script = "grep -E 'pattern' file.txt";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2196_false_positive_in_comment() {
        let script = "# egrep 'pattern' file.txt";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2196_multiple_occurrences() {
        let script = "egrep 'foo' a.txt\nfgrep 'bar' b.txt";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2196_with_pipe() {
        let script = "cat file.txt | egrep 'error'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2196_with_options() {
        let script = "egrep -i 'PATTERN' file.txt";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2196_in_command_substitution() {
        let script = "result=$(egrep 'pattern' file.txt)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
