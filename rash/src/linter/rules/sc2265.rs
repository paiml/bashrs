// SC2265: Use arithmetic expansion instead of expr
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static EXPR_COMMAND: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bexpr\s+").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if EXPR_COMMAND.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2265",
                Severity::Info,
                "Use $(( expr )) instead of expr for arithmetic operations".to_string(),
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
    fn test_sc2265_expr() {
        assert_eq!(check("result=$(expr 1 + 2)").diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2265_arithmetic_ok() {
        assert_eq!(check("result=$((1 + 2))").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2265_expr_standalone() {
        assert_eq!(check("expr 5 * 3").diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2265_comment() {
        assert_eq!(check("# expr 1 + 2").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2265_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2265_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2265_expr_substr() {
        assert_eq!(check("expr substr $str 1 5").diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2265_expr_length() {
        assert_eq!(check("expr length $var").diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2265_double_paren() {
        assert_eq!(check("(( x = y + 1 ))").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2265_let() {
        assert_eq!(check("let x=y+1").diagnostics.len(), 0);
    }
}
