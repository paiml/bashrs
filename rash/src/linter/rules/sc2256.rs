// SC2256: Prefer -n/-z over comparison with empty string
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static EMPTY_STRING_COMPARE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r#"\[\[?\s*"?\$\w+"?\s*(=|!=)\s*""\s*\]\]?"#).unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if EMPTY_STRING_COMPARE.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2256",
                Severity::Info,
                r#"Prefer -n/-z over comparison with empty string: use [ -z "$var" ] instead of [ "$var" = "" ]"#.to_string(),
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
    fn test_sc2256_empty_compare() {
        assert_eq!(check(r#"[ "$var" = "" ]"#).diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2256_not_empty_compare() {
        assert_eq!(check(r#"[ "$var" != "" ]"#).diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2256_use_z_ok() {
        assert_eq!(check(r#"[ -z "$var" ]"#).diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2256_use_n_ok() {
        assert_eq!(check(r#"[ -n "$var" ]"#).diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2256_comment() {
        assert_eq!(check("# comment").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2256_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2256_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2256_double_bracket() {
        assert_eq!(check(r#"[[ "$x" = "" ]]"#).diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2256_value_compare() {
        assert_eq!(check(r#"[ "$a" = "value" ]"#).diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2256_unquoted() {
        assert_eq!(check(r#"[ $var = "" ]"#).diagnostics.len(), 1);
    }
}
