// SC2234: Remove spaces after redirect operators
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static SPACED_REDIRECT: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: > space filename or 2> space filename
    Regex::new(r"\d?>>\s+\S+").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Find all matches on the line
        for _mat in SPACED_REDIRECT.find_iter(line) {
            let diagnostic = Diagnostic::new(
                "SC2234",
                Severity::Info,
                "Spaces after redirect operators (>) are optional but unusual. Consider removing for consistency"
                    .to_string(),
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
    fn test_sc2234_spaced_redirect() {
        let code = "echo test >>  file";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2234_no_space_ok() {
        let code = "echo test >>file";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2234_stderr_redirect() {
        let code = "cmd 2>>  errors.log";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2234_stdout_redirect() {
        let code = "echo data >>  output.txt";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2234_comment_skipped() {
        let code = "# echo test >>  file";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2234_single_space() {
        let code = "echo test >> file";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2234_no_redirect() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2234_no_code() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2234_multiple_redirects() {
        let code = "cmd >>  out 2>>  err";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
    #[test]
    fn test_sc2234_fd_redirect() {
        let code = "cmd 3>>  custom.log";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
