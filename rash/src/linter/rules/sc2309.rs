// SC2309: Arithmetic $((...)) expansion doesn't need $ on variables
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static DOLLAR_IN_ARITH: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\$\(\(\s*\$[a-zA-Z_][a-zA-Z0-9_]*").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if DOLLAR_IN_ARITH.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2309",
                Severity::Info,
                "Remove $ on variables inside arithmetic expansion $((...)): $((count)) instead of $(($count))".to_string(),
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
    fn test_sc2309_dollar_in_arith() {
        let code = r#"result=$(( $count + 1 ))"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2309_no_dollar_ok() {
        let code = r#"result=$(( count + 1 ))"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2309_comment() {
        let code = r#"# result=$(( $count + 1 ))"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2309_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2309_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2309_literal_ok() {
        let code = r#"result=$(( 5 + 3 ))"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2309_multiple_vars() {
        let code = r#"result=$(( $x + $y ))"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2309_double_paren_ok() {
        let code = r#"(( count++ ))"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2309_complex_expr() {
        let code = r#"result=$(( $a * $b / $c ))"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2309_multiple_lines() {
        let code = r#"
x=$(( $a + 1 ))
y=$(( $b + 2 ))
"#;
        assert_eq!(check(code).diagnostics.len(), 2);
    }
}
