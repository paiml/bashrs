// SC2186: Useless echo in pipeline
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static ECHO_PIPE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\becho\s+[^|]+\s*\|\s*cat\b").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if ECHO_PIPE.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2186",
                Severity::Info,
                "Useless echo | cat. Just use echo directly".to_string(),
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
    fn test_sc2186_echo_cat() {
        let code = "echo test | cat";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2186_normal_ok() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
