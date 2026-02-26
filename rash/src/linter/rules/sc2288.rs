// SC2288: Prefer always true/false over [ 1 = 1 ]
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static TAUTOLOGY: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\[\s+(1\s*=\s*1|true\s*=\s*true)\s*\]").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if TAUTOLOGY.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2288",
                Severity::Info,
                "Use 'true' or 'false' directly instead of [ 1 = 1 ]".to_string(),
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
    fn test_sc2288_one_equals_one() {
        let code = "[ 1 = 1 ]";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2288_true_command_ok() {
        let code = "if true; then";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2288_comment() {
        let code = "# [ 1 = 1 ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2288_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2288_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2288_variable_comparison_ok() {
        let code = "[ $x = 1 ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2288_two_equals_two_ok() {
        let code = "[ 2 = 2 ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2288_false_ok() {
        let code = "if false; then";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2288_true_equals_true() {
        let code = "[ true = true ]";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2288_colon_ok() {
        let code = ": # no-op";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
