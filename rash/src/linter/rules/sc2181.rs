//! SC2181: Check exit code directly with e.g. 'if mycmd;', not indirectly with $?
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! command
//! if [ $? -eq 0 ]; then
//!   echo "Success"
//! fi
//! ```
//!
//! Good:
//! ```bash
//! if command; then
//!   echo "Success"
//! fi
//! ```
//!
//! # Rationale
//!
//! Checking $? is:
//! - Less readable
//! - More error-prone (other commands can change $?)
//! - Not idiomatic shell
//!
//! Directly check command exit status in if/while conditions.
//!
//! # Auto-fix
//!
//! Warning only - refactor to use command directly in condition

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Check for indirect $? comparisons
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: if [ $? -eq 0 ] or [ $? -ne 0 ]
    let pattern = Regex::new(r"(?:if|while)\s+\[\s*\$\?\s*(?:-eq|-ne)\s*0\s*\]").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2181",
                Severity::Info,
                "Check exit code directly with e.g. 'if mycmd;', not indirectly with $?",
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2181_basic_detection() {
        let script = "if [ $? -eq 0 ]; then echo ok; fi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2181");
    }

    #[test]
    fn test_sc2181_not_equal() {
        let script = "if [ $? -ne 0 ]; then echo fail; fi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2181_while_loop() {
        let script = "while [ $? -eq 0 ]; do process; done";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2181_with_spaces() {
        let script = "if [  $?  -eq  0  ]; then ok; fi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2181_false_positive_direct_check() {
        let script = "if command; then echo ok; fi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2181_false_positive_in_comment() {
        let script = "# if [ $? -eq 0 ]; then";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2181_multiline() {
        let script = "command\nif [ $? -eq 0 ]; then\n  echo success\nfi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2181_in_function() {
        let script = "check() { if [ $? -eq 0 ]; then return 0; fi; }";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2181_nested() {
        let script = "if [ $? -ne 0 ]; then\n  if [ $? -eq 0 ]; then ok; fi\nfi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2181_false_positive_negation() {
        let script = "if ! command; then echo fail; fi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
