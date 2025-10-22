// SC2284: Use ${var:+val} for conditional assignment
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static CONDITIONAL_ASSIGN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\[\s+-n\s+\$\{?[a-zA-Z_][a-zA-Z0-9_]*\}?\s+\]\s*&&\s*[a-zA-Z_]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if CONDITIONAL_ASSIGN.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2284",
                Severity::Info,
                "Consider using ${var:+value} for conditional value assignment".to_string(),
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
    fn test_sc2284_conditional_pattern() {
        let code = "[ -n $var ] && other=value";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2284_expansion_ok() {
        let code = "other=${var:+value}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2284_comment() {
        let code = "# [ -n $var ] && x=y";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2284_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2284_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2284_plain_assignment_ok() {
        let code = "var=value";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2284_test_without_assign_ok() {
        let code = "[ -n $var ] && echo set";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2284_braced_var() {
        let code = "[ -n ${config} ] && output=yes";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2284_or_test_ok() {
        let code = "[ -n $var ] || x=y";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2284_double_bracket_ok() {
        let code = "[[ -n $var ]] && x=y";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
