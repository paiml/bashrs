// SC2244: Prefer explicit -n to omitted second operand in test
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static IMPLICIT_STRING_TEST: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: [ "$var" ] or [ $var ] or [ "${var}" ] (implicit non-empty test)
    Regex::new(r#"\[\s+"?\$\{?\w+\}?"?\s+\]"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if IMPLICIT_STRING_TEST.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2244",
                Severity::Info,
                "Prefer explicit -n for string length tests: [ -n \"$var\" ] instead of [ \"$var\" ]"
                    .to_string(),
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
    fn test_sc2244_implicit_test() {
        let code = r#"[ "$var" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2244_explicit_n_ok() {
        let code = r#"[ -n "$var" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2244_unquoted() {
        let code = "[ $var ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2244_comparison_ok() {
        let code = r#"[ "$var" = "value" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Has comparison
    }
    #[test]
    fn test_sc2244_comment_skipped() {
        let code = r#"# [ "$var" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2244_braced_var() {
        let code = r#"[ "${var}" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2244_in_if() {
        let code = r#"if [ "$status" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2244_no_code() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2244_normal_command() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2244_file_test_ok() {
        let code = r#"[ -f "$file" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
