// SC2322: Arithmetic operations don't accept this argument count
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static ARITH_SYNTAX_ERROR: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\$\(\(\s*[+\-*/]\s*\)\)").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if ARITH_SYNTAX_ERROR.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2322",
                Severity::Error,
                "Arithmetic operation missing operands: $(( + )) is invalid".to_string(),
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
    fn test_sc2322_missing_operands() {
        let code = "result=$(( + ))";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2322_valid_addition_ok() {
        let code = "result=$((5 + 3))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2322_variable_ok() {
        let code = "result=$((x + y))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2322_comment() {
        let code = "# result=$(( + ))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2322_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2322_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2322_missing_subtraction() {
        let code = "result=$(( - ))";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2322_missing_multiplication() {
        let code = "result=$(( * ))";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2322_unary_ok() {
        let code = "result=$((-x))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2322_multiple() {
        let code = r#"
a=$(( + ))
b=$(( / ))
"#;
        assert_eq!(check(code).diagnostics.len(), 2);
    }
}
