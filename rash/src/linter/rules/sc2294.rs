// SC2294: Use arithmetic expansion ((...)) for simple assignments
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static LET_SIMPLE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\blet\s+[a-zA-Z_][a-zA-Z0-9_]*=[a-zA-Z0-9_+\-*/]+\b").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if LET_SIMPLE.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2294",
                Severity::Info,
                "Use ((...)) instead of let for simple arithmetic assignments".to_string(),
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
    fn test_sc2294_let_assign() {
        let code = "let x=5";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2294_double_paren_ok() {
        let code = "(( x = 5 ))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2294_let_arithmetic() {
        let code = "let count=count+1";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2294_comment() {
        let code = "# let x=5";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2294_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2294_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2294_dollar_arithmetic_ok() {
        let code = "x=$(( y + 1 ))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2294_expr_ok() {
        let code = "x=$(expr 1 + 1)";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2294_string_let_ok() {
        let code = r#"let "x = y + 1""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2294_increment() {
        let code = "let i=i+1";
        assert_eq!(check(code).diagnostics.len(), 1);
    }
}
