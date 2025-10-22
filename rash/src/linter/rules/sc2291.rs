// SC2291: Prefer [[ ! -v var ]] to check if variable is unset
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static UNSET_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:^|[^\[])\[\s+-z\s+"\$\{[a-zA-Z_][a-zA-Z0-9_]*\+x\}""#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if UNSET_PATTERN.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2291",
                Severity::Info,
                "Use [[ ! -v var ]] to check if variable is unset (cleaner syntax)".to_string(),
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
    fn test_sc2291_unset_pattern() {
        let code = r#"[ -z "${var+x}" ]"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2291_not_v_ok() {
        let code = "[[ ! -v var ]]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2291_comment() {
        let code = r#"# [ -z "${var+x}" ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2291_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2291_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2291_simple_z_ok() {
        let code = r#"[ -z "$var" ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2291_n_test_ok() {
        let code = r#"[ -n "$var" ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2291_config_var() {
        let code = r#"[ -z "${CONFIG+x}" ]"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2291_double_bracket() {
        let code = r#"[[ -z "${var+x}" ]]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2291_v_flag() {
        let code = "[[ -v CONFIG ]]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
