// SC2204: (..) is subshell, not test
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static IF_SUBSHELL: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bif\s+\(").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if IF_SUBSHELL.is_match(line) && !line.contains("$(") && !line.contains("((") {
            let diagnostic = Diagnostic::new(
                "SC2204",
                Severity::Warning,
                "(..) is a subshell. Use [ ] or [[ ]] for test".to_string(),
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
    fn test_sc2204_if_subshell() {
        let code = "if ( test ); then";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2204_if_test_ok() {
        let code = "if [ test ]; then";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
