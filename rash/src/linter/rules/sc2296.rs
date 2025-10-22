// SC2296: Parameter expansions can't be nested
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static NESTED_EXPANSION: Lazy<Regex> = Lazy::new(|| Regex::new(r"\$\{[^}]*\$\{").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if NESTED_EXPANSION.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2296",
                Severity::Error,
                "Parameter expansions can't be nested. Use separate expansions.".to_string(),
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
    fn test_sc2296_nested_expansion() {
        let code = "${var:-${default}}";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2296_separate_ok() {
        let code = "default=${DEFAULT}\nvar=${var:-$default}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2296_simple_expansion_ok() {
        let code = "${var:-default}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2296_comment() {
        let code = "# ${var:-${def}}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2296_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2296_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2296_multiple_vars_ok() {
        let code = "${var1} ${var2}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2296_command_subst_ok() {
        let code = "${var:-$(command)}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2296_nested_in_assign() {
        let code = "x=${a:-${b}}";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2296_arithmetic_ok() {
        let code = "$((x + y))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
