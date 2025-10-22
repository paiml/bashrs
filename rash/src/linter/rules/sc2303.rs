// SC2303: Arithmetic base only allowed in assignments
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ARITHMETIC_BASE_IN_EXPR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\(\(\s*\d+#").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if ARITHMETIC_BASE_IN_EXPR.is_match(line) && !line.contains("=") {
            let diagnostic = Diagnostic::new(
                "SC2303",
                Severity::Error,
                "Arithmetic base syntax (N#) only allowed in assignments".to_string(),
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
    fn test_sc2303_base_in_expression() {
        let code = "(( 16#FF ))";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2303_base_in_assignment_ok() {
        let code = "(( x = 16#FF ))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2303_normal_arithmetic_ok() {
        let code = "(( x + 5 ))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2303_comment() {
        let code = "# (( 16#FF ))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2303_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2303_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2303_decimal_ok() {
        let code = "(( 255 ))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2303_binary_assign_ok() {
        let code = "(( x = 2#1010 ))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2303_hex_no_assign() {
        let code = "(( 16#A ))";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2303_let_assign_ok() {
        let code = "let x=16#FF";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
