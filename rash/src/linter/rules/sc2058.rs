// SC2058: Unknown unary operator in test expression
//
// Test commands support specific unary operators for file tests and string tests.
// Using an invalid unary operator causes syntax errors or unexpected behavior.
//
// Examples:
// Bad:
//   [ -q file ]              // -q is not a valid test operator
//   [ -m file ]              // -m is not a valid test operator
//   test -j file             // -j is not a valid test operator
//
// Good:
//   [ -f file ]              // File exists and is a regular file
//   [ -d dir ]               // Directory exists
//   [ -z "$var" ]            // String is empty
//   [ -n "$var" ]            // String is non-empty
//   [ -e file ]              // File exists
//   test -r file             // File is readable
//
// Valid unary operators:
//   File: -e, -f, -d, -r, -w, -x, -s, -h, -L, -p, -b, -c, -t, -S, -g, -u, -k, -O, -G, -N, -a
//   String: -z, -n

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

/// Valid unary test operators in POSIX and bash test expressions.
const VALID_UNARY_OPS: &[&str] = &[
    "e", "f", "d", "r", "w", "x", "s", "z", "n", "h", "L", "p", "b", "c", "t", "S", "g", "u",
    "k", "O", "G", "N", "a",
];

static BRACKET_UNARY: Lazy<Regex> = Lazy::new(|| {
    // Match [ -X ... ] where X is one or more letters
    Regex::new(r"\[\s+-([a-zA-Z]+)\s+").expect("SC2058 bracket regex must compile")
});

static TEST_UNARY: Lazy<Regex> = Lazy::new(|| {
    // Match test -X ... (the test builtin form)
    Regex::new(r"\btest\s+-([a-zA-Z]+)\s+").expect("SC2058 test regex must compile")
});

fn is_valid_unary_op(op: &str) -> bool {
    VALID_UNARY_OPS.contains(&op)
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check [ -X ... ] form
        for cap in BRACKET_UNARY.captures_iter(line) {
            let operator = cap.get(1).expect("SC2058 capture group 1 must exist").as_str();
            if !is_valid_unary_op(operator) {
                let full_match = cap.get(0).expect("SC2058 capture group 0 must exist").as_str();
                let pos = line.find(full_match).unwrap_or(0);
                let start_col = pos + 1;
                let end_col = start_col + full_match.len();

                result.add(Diagnostic::new(
                    "SC2058",
                    Severity::Error,
                    format!(
                        "Unknown unary operator '-{}' in test expression. Use a valid operator like -f, -d, -e, -z, -n, etc.",
                        operator
                    ),
                    Span::new(line_num, start_col, line_num, end_col),
                ));
            }
        }

        // Check test -X ... form
        for cap in TEST_UNARY.captures_iter(line) {
            let operator = cap.get(1).expect("SC2058 capture group 1 must exist").as_str();
            if !is_valid_unary_op(operator) {
                let full_match = cap.get(0).expect("SC2058 capture group 0 must exist").as_str();
                let pos = line.find(full_match).unwrap_or(0);
                let start_col = pos + 1;
                let end_col = start_col + full_match.len();

                result.add(Diagnostic::new(
                    "SC2058",
                    Severity::Error,
                    format!(
                        "Unknown unary operator '-{}' in test expression. Use a valid operator like -f, -d, -e, -z, -n, etc.",
                        operator
                    ),
                    Span::new(line_num, start_col, line_num, end_col),
                ));
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn test_sc2058_unknown_operator_q() {
        let code = "[ -q file ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2058");
        assert!(result.diagnostics[0].message.contains("-q"));
    }

    #[test]
    fn test_sc2058_unknown_operator_m() {
        let code = "[ -m file ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("-m"));
    }

    #[test]
    fn test_sc2058_test_builtin_unknown() {
        let code = "test -q file";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2058");
        assert!(result.diagnostics[0].message.contains("-q"));
    }

    #[test]
    fn test_sc2058_valid_f() {
        let code = "[ -f file ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2058_valid_d() {
        let code = "[ -d dir ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2058_valid_z() {
        let code = r#"[ -z "$var" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2058_valid_n() {
        let code = r#"[ -n "$var" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2058_valid_e() {
        let code = "[ -e /tmp/file ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2058_valid_r_w_x() {
        let code = "[ -r file ] && [ -w file ] && [ -x file ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2058_comment_ignored() {
        let code = "# [ -q file ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2058_test_builtin_valid() {
        let code = "test -f file";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2058_multiple_unknown() {
        let code = "[ -q file ] && [ -m dir ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2058_valid_capital_l() {
        let code = "[ -L /path/to/link ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2058_valid_capital_s() {
        let code = "[ -S /path/to/socket ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
