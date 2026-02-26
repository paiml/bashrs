// SC2231: Quote variables in case patterns to avoid glob expansion
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static UNQUOTED_VAR_IN_CASE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: case pattern with unquoted variable ($var or ${var})
    Regex::new(r"case\s+\$[{]?\w+[}]?\s+in").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if UNQUOTED_VAR_IN_CASE.is_match(line) && !line.contains("\"$") {
            let diagnostic = Diagnostic::new(
                "SC2231",
                Severity::Warning,
                "Quote variables in case expressions to prevent glob expansion".to_string(),
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
    fn test_sc2231_unquoted_var() {
        let code = "case $var in";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2231_quoted_var_ok() {
        let code = r#"case "$var" in"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2231_complete_case() {
        let code = "case $val in\n  pattern) echo test;;\nesac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2231_braced_var() {
        let code = "case ${var} in";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2231_comment_skipped() {
        let code = "# case $var in";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2231_literal_ok() {
        let code = "case value in";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2231_command_substitution() {
        let code = "case $(cmd) in";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Not a variable
    }
    #[test]
    fn test_sc2231_no_code() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2231_normal_command() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2231_multiple_cases() {
        let code = "case $a in\n  x) :;;\nesac\ncase $b in\n  y) :;;\nesac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
