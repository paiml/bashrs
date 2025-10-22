// SC2279: Avoid ambiguous redirects
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static AMBIGUOUS_REDIRECT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r">\s+&\s*[0-9]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if AMBIGUOUS_REDIRECT.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2279",
                Severity::Warning,
                "Ambiguous redirect - use >& for duplicating file descriptors".to_string(),
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
    fn test_sc2279_ambiguous_redirect() {
        let code = "cmd > &1";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2279_proper_redirect_ok() {
        let code = "cmd >&1";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2279_stderr_redirect_ok() {
        let code = "cmd 2>&1";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2279_comment() {
        let code = "# cmd > &1";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2279_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2279_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2279_file_redirect_ok() {
        let code = "cmd > file.txt";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2279_append_redirect_ok() {
        let code = "cmd >> file.txt";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2279_input_redirect_ok() {
        let code = "cmd < input.txt";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2279_proper_fd_ok() {
        let code = "cmd 3>&2";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
