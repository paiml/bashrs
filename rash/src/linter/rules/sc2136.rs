// SC2136: Unexpected 'do' in 'if' statement
//
// The 'do' keyword is for loops (for, while, until), not 'if' statements.
// 'if' statements use 'then', not 'do'.
//
// Examples:
// Bad:
//   if [ -f file ]; do echo "exists"; done     // 'do' with if (should be 'then')
//   if true; do command; fi                    // Wrong keyword
//
// Good:
//   if [ -f file ]; then echo "exists"; fi     // 'then' for if statements
//   while [ -f file ]; do echo "exists"; done  // 'do' for while loops
//   for i in 1 2 3; do echo $i; done           // 'do' for for loops
//
// Impact: Syntax error, script will fail

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static IF_DO: Lazy<Regex> = Lazy::new(|| {
    // Match: if ... do (should be then)
    Regex::new(r"\bif\b[^\n]*;\s*do\b").unwrap()
});

static ELIF_DO: Lazy<Regex> = Lazy::new(|| {
    // Match: elif ... do (should be then)
    Regex::new(r"\belif\b[^\n]*;\s*do\b").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for if ... do
        if IF_DO.is_match(line) {
            if let Some(mat) = IF_DO.find(line) {
                let start_col = mat.start() + 1;
                let end_col = mat.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2136",
                    Severity::Error,
                    "'if' statements use 'then', not 'do'. Change 'do' to 'then'".to_string(),
                    Span::new(line_num, start_col, line_num, end_col),
                );

                result.add(diagnostic);
            }
        }

        // Check for elif ... do
        if ELIF_DO.is_match(line) {
            if let Some(mat) = ELIF_DO.find(line) {
                let start_col = mat.start() + 1;
                let end_col = mat.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2136",
                    Severity::Error,
                    "'elif' uses 'then', not 'do'. Change 'do' to 'then'".to_string(),
                    Span::new(line_num, start_col, line_num, end_col),
                );

                result.add(diagnostic);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2136_if_do() {
        let code = "if [ -f file ]; do echo \"exists\"; done";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("'then'"));
    }

    #[test]
    fn test_sc2136_if_then_ok() {
        let code = "if [ -f file ]; then echo \"exists\"; fi";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2136_while_do_ok() {
        let code = "while [ -f file ]; do echo \"exists\"; done";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2136_for_do_ok() {
        let code = "for i in 1 2 3; do echo $i; done";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2136_elif_do() {
        let code = r#"
if [ -f file1 ]; then
    echo "1"
elif [ -f file2 ]; do
    echo "2"
fi
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("'then'"));
    }

    #[test]
    fn test_sc2136_elif_then_ok() {
        let code = r#"
if [ -f file1 ]; then
    echo "1"
elif [ -f file2 ]; then
    echo "2"
fi
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2136_comment_ok() {
        let code = "# if [ -f file ]; do echo \"test\"";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2136_case_do_ok() {
        let code = r#"
case $var in
    pattern) do_something ;;
esac
"#;
        let result = check(code);
        // do_something is a function name, not keyword
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2136_nested_if_do() {
        let code = r#"
if [ -f outer ]; then
    if [ -f inner ]; do
        echo "nested"
    fi
fi
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2136_multiple_errors() {
        let code = r#"
if [ -f file1 ]; do echo "1"; fi
if [ -f file2 ]; do echo "2"; fi
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
