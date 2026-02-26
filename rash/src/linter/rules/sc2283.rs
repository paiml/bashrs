// SC2283: Remove spaces after ! in [ ! -f ... ]
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static NEGATION_SPACE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\[\s+!\s+\s+-").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if NEGATION_SPACE.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2283",
                Severity::Warning,
                "Remove extra spaces after ! in test expressions".to_string(),
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
    fn test_sc2283_extra_space() {
        let code = "[ !  -f file ]";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2283_correct_ok() {
        let code = "[ ! -f file ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2283_double_bracket_ok() {
        let code = "[[ ! -f file ]]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2283_comment() {
        let code = "# [ !  -f file ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2283_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2283_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2283_no_negation_ok() {
        let code = "[ -f file ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2283_string_test() {
        let code = "[ !  -z $var ]";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2283_numeric_test() {
        let code = "[ !  -eq 5 ]";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2283_proper_spacing() {
        let code = "[ ! -d /path ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
