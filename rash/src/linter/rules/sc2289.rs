// SC2289: Prefer ${#var} over expr length
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static EXPR_LENGTH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"expr\s+length\s+\$").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if EXPR_LENGTH.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2289",
                Severity::Info,
                "Use ${#var} instead of expr length for string length".to_string(),
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
    fn test_sc2289_expr_length() {
        let code = "len=$(expr length $str)";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2289_param_expansion_ok() {
        let code = "len=${#str}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2289_comment() {
        let code = "# expr length $var";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2289_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2289_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2289_expr_arithmetic_ok() {
        let code = "expr 1 + 1";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2289_expr_substr_ok() {
        let code = "expr substr $str 1 5";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2289_wc_ok() {
        let code = "wc -c <<< $str";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2289_braced_var() {
        let code = "expr length ${var}";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2289_awk_ok() {
        let code = r#"awk '{print length}' <<< $str"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
