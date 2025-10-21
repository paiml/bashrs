// SC2208: Use [[ ]] or quote to avoid glob/word splitting
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static UNQUOTED_IN_TEST: Lazy<Regex> = Lazy::new(|| {
    // Match [ $var = ... ] or [ ${var} = ... ] without quotes
    Regex::new(r"\[\s+\$[{]?\w+[}]?\s+(=|!=)\s+").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip [[ ]] tests
        if line.contains("[[") {
            continue;
        }

        if UNQUOTED_IN_TEST.is_match(line) {
            // Check if quotes are missing
            if !line.contains("\"$") {
                let diagnostic = Diagnostic::new(
                    "SC2208",
                    Severity::Warning,
                    "Use [[ ]] or quote variables to avoid glob/word splitting in tests"
                        .to_string(),
                    Span::new(line_num, 1, line_num, line.len() + 1),
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
    fn test_sc2208_unquoted_var() {
        let code = r#"[ $var = value ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2208_quoted_ok() {
        let code = r#"[ "$var" = value ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2208_double_bracket_ok() {
        let code = r#"[[ $var = value ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2208_not_equal() {
        let code = r#"[ $x != value ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2208_both_vars_unquoted() {
        let code = r#"[ $a = $b ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2208_braced_var() {
        let code = r#"[ ${var} = test ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2208_if_test() {
        let code = r#"if [ $status = success ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2208_string_literal() {
        let code = r#"[ value = test ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // No variable
    }
    #[test]
    fn test_sc2208_comment_skipped() {
        let code = r#"# [ $var = value ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2208_numeric_test_ok() {
        let code = r#"[ "$count" -eq 5 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
