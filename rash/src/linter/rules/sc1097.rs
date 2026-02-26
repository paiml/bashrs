// SC1097: Unexpected `==` in `[ ]`. Use a single `=` for string comparison.
//
// In POSIX `[ ]` (test), only single `=` is supported for string comparison.
// `==` works in bash's `[ ]` and `[[ ]]`, but is not POSIX-compliant.
//
// Examples:
// Bad:
//   [ "$x" == "hello" ]
//   [ "$a" == "$b" ]
//
// Good:
//   [ "$x" = "hello" ]
//   [[ "$x" == "hello" ]]   # == is fine in [[ ]]

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Matches `[ ... == ... ]` inside a single-bracket test
static SINGLE_BRACKET_DOUBLE_EQ: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\[\s+.*\s+==\s+.*\s+\]").unwrap());

/// Matches `[[ ` double-bracket open (to exclude)
static DOUBLE_BRACKET_OPEN: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\[\[").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with('#') {
            continue;
        }

        // Skip lines with [[ ]] - == is valid there
        if DOUBLE_BRACKET_OPEN.is_match(line) {
            continue;
        }

        for mat in SINGLE_BRACKET_DOUBLE_EQ.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC1097",
                Severity::Warning,
                "Unexpected `==` in `[ ]`. Use a single `=` for POSIX string comparison."
                    .to_string(),
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
    fn test_sc1097_double_eq_in_single_bracket() {
        let code = r#"[ "$x" == "hello" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1097");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sc1097_double_eq_variables() {
        let code = r#"[ "$a" == "$b" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1097_single_eq_ok() {
        let code = r#"[ "$x" = "hello" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1097_double_bracket_ok() {
        let code = r#"[[ "$x" == "hello" ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1097_comment_ok() {
        let code = r#"# [ "$x" == "hello" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1097_in_if_statement() {
        let code = r#"if [ "$status" == "ok" ]; then echo good; fi"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
