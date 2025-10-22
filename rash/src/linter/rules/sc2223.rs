// SC2223: Remove 'function' keyword or () for POSIX compatibility
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static FUNCTION_WITH_PARENS: Lazy<Regex> = Lazy::new(|| {
    // Match: function name() or function name ()
    Regex::new(r"\bfunction\s+\w+\s*\(\s*\)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if FUNCTION_WITH_PARENS.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2223",
                Severity::Warning,
                "Use 'function name' or 'name()' but not both for POSIX compatibility".to_string(),
                Span::new(line_num, 1, line_num, line.len() + 1),
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
    fn test_sc2223_function_with_parens() {
        let code = r#"function foo() { echo bar; }"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2223_function_only_ok() {
        let code = r#"function foo { echo bar; }"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2223_parens_only_ok() {
        let code = r#"foo() { echo bar; }"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2223_with_space() {
        let code = r#"function bar () { echo baz; }"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2223_comment_skipped() {
        let code = r#"# function foo() { echo bar; }"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2223_multiline() {
        let code = "function test()\n{\n  echo hello\n}";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2223_multiple_functions() {
        let code = "function a() { echo 1; }\nfunction b() { echo 2; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
    #[test]
    fn test_sc2223_no_code() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2223_normal_command() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2223_underscore_name() {
        let code = "function my_func() { return 0; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
