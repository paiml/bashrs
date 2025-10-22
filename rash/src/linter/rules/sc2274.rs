// SC2274: Prefer [[ && ]] over separate test commands
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static SEPARATE_TESTS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\]\s*&&\s*\[").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Count all occurrences of ] && [
        for _ in SEPARATE_TESTS.find_iter(line) {
            let diagnostic = Diagnostic::new(
                "SC2274",
                Severity::Info,
                "Prefer [[ condition && condition ]] over separate [ ] tests".to_string(),
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
    fn test_sc2274_separate_tests() {
        let code = "[ -f file ] && [ -r file ]";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2274_combined_ok() {
        let code = "[[ -f file && -r file ]]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2274_comment() {
        let code = "# [ test ] && [ test ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2274_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2274_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2274_single_test_ok() {
        let code = "[ -f file ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2274_command_and_test_ok() {
        let code = "command && [ -f file ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2274_test_and_command_ok() {
        let code = "[ -f file ] && command";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2274_or_tests() {
        let code = "[ -f file ] || [ -d dir ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2274_three_tests() {
        let code = "[ -f f1 ] && [ -f f2 ] && [ -f f3 ]";
        assert_eq!(check(code).diagnostics.len(), 2);
    }
}
