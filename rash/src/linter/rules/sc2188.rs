// SC2188: Redirection without command
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static LONE_REDIRECT: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*[<>]").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if LONE_REDIRECT.is_match(line) && !line.contains("<<") {
            let diagnostic = Diagnostic::new(
                "SC2188",
                Severity::Error,
                "Redirection without command".to_string(),
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
    fn test_sc2188_lone_redirect() {
        let code = "> output.txt";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2188_normal_ok() {
        let code = "echo test > output.txt";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
