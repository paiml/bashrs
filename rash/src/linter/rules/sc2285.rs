// SC2285: Remove $/${} for arithmetic variables
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ARITHMETIC_VAR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\(\(\s*\$\{?[a-zA-Z_][a-zA-Z0-9_]*\}?\s*[+\-*/]").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if ARITHMETIC_VAR.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2285",
                Severity::Info,
                "Remove $ from variables in arithmetic contexts: use ((var + 1)) not (($var + 1))"
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
    fn test_sc2285_dollar_in_arithmetic() {
        let code = "(( $count + 1 ))";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2285_correct_ok() {
        let code = "(( count + 1 ))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2285_braced() {
        let code = "(( ${num} * 2 ))";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2285_comment() {
        let code = "# (( $x + 1 ))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2285_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2285_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2285_outside_arithmetic_ok() {
        let code = "x=$var";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2285_subtraction() {
        let code = "(( $total - 5 ))";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2285_division() {
        let code = "(( $value / 2 ))";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2285_multiple_vars() {
        let code = "(( x + y ))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
