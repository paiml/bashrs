// SC2298: Useless use of cat before pipe
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static USELESS_CAT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bcat\s+[^\s|-][^\s|]*\s*\|").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if USELESS_CAT.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2298",
                Severity::Info,
                "Use < file instead of cat file | for better performance".to_string(),
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
    fn test_sc2298_cat_pipe() {
        let code = "cat file | grep pattern";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2298_redirect_ok() {
        let code = "grep pattern < file";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2298_cat_multiple_ok() {
        let code = "cat file1 file2 | grep";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2298_comment() {
        let code = "# cat file | grep";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2298_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2298_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2298_cat_no_pipe_ok() {
        let code = "cat file";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2298_pipe_without_cat_ok() {
        let code = "echo test | grep t";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2298_cat_dash_ok() {
        let code = "cat - | grep pattern";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2298_with_path() {
        let code = "cat /path/to/file | sort";
        assert_eq!(check(code).diagnostics.len(), 1);
    }
}
