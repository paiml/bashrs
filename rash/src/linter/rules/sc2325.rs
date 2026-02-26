// SC2325: Use $var instead of ${var} in arithmetic contexts
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static BRACED_IN_ARITH: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"(?:\$\(\(|\(\()\s*\$\{[a-zA-Z_][a-zA-Z0-9_]*\}").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if BRACED_IN_ARITH.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2325",
                Severity::Info,
                "In arithmetic contexts, ${var} can be simplified to $var or just var".to_string(),
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
    fn test_sc2325_braced_in_arithmetic() {
        let code = "result=$(( ${var} + 1 ))";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2325_unbraced_ok() {
        let code = "result=$(( $var + 1 ))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2325_no_dollar_ok() {
        let code = "result=$(( var + 1 ))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2325_comment() {
        let code = "# result=$(( ${x} + 1 ))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2325_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2325_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2325_double_paren() {
        let code = "(( ${count}++ ))";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2325_outside_arithmetic_ok() {
        let code = "echo ${var}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2325_multiple_vars() {
        let code = "result=$(( ${x} + ${y} ))";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2325_complex_expression() {
        let code = r#"
a=$(( ${b} * 2 ))
c=$(( ${d} / 3 ))
"#;
        assert_eq!(check(code).diagnostics.len(), 2);
    }
}
