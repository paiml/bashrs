// SC2297: Redirect before pipe
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static REDIRECT_AFTER_PIPE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\|\s+[^|]+\s+[<>]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if REDIRECT_AFTER_PIPE.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2297",
                Severity::Warning,
                "Redirects should come before pipes in the command".to_string(),
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
    fn test_sc2297_redirect_after_pipe() {
        let code = "cat file | sort > output";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2297_redirect_before_ok() {
        let code = "cat file > output | sort";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2297_simple_pipe_ok() {
        let code = "cat file | sort";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2297_comment() {
        let code = "# cat | sort > out";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2297_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2297_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2297_redirect_only_ok() {
        let code = "cat < input";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2297_input_redirect() {
        let code = "cat file | grep pattern < input";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2297_multiple_pipes_ok() {
        let code = "cat | sort | uniq";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2297_append_redirect() {
        let code = "cat file | sort >> output";
        assert_eq!(check(code).diagnostics.len(), 1);
    }
}
