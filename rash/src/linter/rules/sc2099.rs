// SC2099: Use $(...) instead of deprecated backtick command substitution
//
// Backticks `` are deprecated in favor of $(...) for command substitution.
// $(...) is more readable, easier to nest, and recommended by POSIX.
//
// Examples:
// Bad:
//   result=`date`                // Deprecated backticks
//   output=`echo \`date\``       // Hard to read nesting
//
// Good:
//   result=$(date)               // Modern syntax
//   output=$(echo $(date))       // Easy nesting
//
// Impact: Deprecated syntax, harder to maintain

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static BACKTICK_SUBSTITUTION: Lazy<Regex> = Lazy::new(|| {
    // Match: `command` in any context
    Regex::new(r"`[^`]+`").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in BACKTICK_SUBSTITUTION.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2099",
                Severity::Info,
                "Use $(...) instead of deprecated backtick command substitution".to_string(),
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
    fn test_sc2099_backticks() {
        let code = "result=`date`";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2099_in_string() {
        let code = r#"msg="Today is `date`""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2099_dollar_paren_ok() {
        let code = "result=$(date)";
        let result = check(code);
        // $() is preferred
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    #[ignore] // TODO: Better nested backtick detection
    fn test_sc2099_nested_backticks() {
        let code = r#"output=`echo `date``"#;
        let result = check(code);
        // Nested backticks (hard to read)
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2099_comment_ok() {
        let code = "# result=`date`";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2099_multiple() {
        let code = "a=`cmd1`; b=`cmd2`";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2099_in_command() {
        let code = "echo `hostname`";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2099_with_pipe() {
        let code = "cat file | grep `whoami`";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2099_in_test() {
        let code = "[ `id -u` -eq 0 ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2099_multiline() {
        let code = r#"
result=`date`
echo "Result: $result"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
