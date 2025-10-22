// SC2292: Prefer ${var:0:1} over expr substr for single character
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static EXPR_SUBSTR_ONE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"expr\s+substr\s+\$[a-zA-Z_][a-zA-Z0-9_]*\s+\d+\s+1\b").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if EXPR_SUBSTR_ONE.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2292",
                Severity::Info,
                "Use ${var:pos:1} instead of expr substr for extracting single characters"
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
    fn test_sc2292_expr_substr_one() {
        let code = "char=$(expr substr $str 1 1)";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2292_param_expansion_ok() {
        let code = "char=${str:0:1}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2292_comment() {
        let code = "# expr substr $str 1 1";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2292_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2292_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2292_multi_char_ok() {
        let code = "expr substr $str 1 5";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2292_different_pos() {
        let code = "expr substr $str 3 1";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2292_cut_ok() {
        let code = "cut -c1 <<< $str";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2292_expansion_range() {
        let code = "part=${str:5:10}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2292_expr_length_ok() {
        let code = "expr length $str";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
