// SC2287: Prefer [[ -v var ]] to check if variable is set
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ISSET_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"\[\s+-n\s+"\$\{[a-zA-Z_][a-zA-Z0-9_]*\+x\}""#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if ISSET_PATTERN.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2287",
                Severity::Info,
                "Use [[ -v var ]] to check if variable is set (cleaner syntax)".to_string(),
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
    fn test_sc2287_isset_pattern() {
        let code = r#"[ -n "${var+x}" ]"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2287_v_flag_ok() {
        let code = "[[ -v var ]]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2287_comment() {
        let code = r#"# [ -n "${var+x}" ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2287_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2287_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2287_simple_test_ok() {
        let code = r#"[ -n "$var" ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2287_z_test_ok() {
        let code = r#"[ -z "$var" ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2287_alternative_value_ok() {
        let code = r#"${var:+value}"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2287_complex_isset() {
        let code = r#"[ -n "${CONFIG+x}" ]"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2287_double_bracket_v() {
        let code = "[[ -v CONFIG ]]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
