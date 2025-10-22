// SC2323: Arithmetic equality uses = not ==
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ARITH_DOUBLE_EQUALS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\(\(\s*[^)]*==").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if ARITH_DOUBLE_EQUALS.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2323",
                Severity::Info,
                "In arithmetic contexts, use = for assignment and = for comparison (== also works)"
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
    fn test_sc2323_double_equals() {
        let code = "(( x == 5 ))";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2323_single_equals_ok() {
        let code = "(( x = 5 ))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2323_bracket_comparison_ok() {
        let code = "[[ $x == 5 ]]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2323_comment() {
        let code = "# (( x == 5 ))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2323_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2323_normal() {
        let code = "echo test";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2323_arithmetic_expansion() {
        let code = "result=$(( x == 5 ))";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2323_greater_than_ok() {
        let code = "(( x > 5 ))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2323_not_equal_ok() {
        let code = "(( x != 5 ))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2323_multiple() {
        let code = r#"
(( a == 1 ))
(( b == 2 ))
"#;
        assert_eq!(check(code).diagnostics.len(), 2);
    }
}
