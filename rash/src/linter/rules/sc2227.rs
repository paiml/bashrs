// SC2227: Redirection before pipe applies to first command in pipeline
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static REDIRECT_BEFORE_PIPE: Lazy<Regex> = Lazy::new(|| {
    // Match: command > file | other_command (but not >>)
    Regex::new(r"[^|>]+>\s*\S+\s*\|").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if REDIRECT_BEFORE_PIPE.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2227",
                Severity::Info,
                "Redirection before pipe applies to first command only. Reorder if this is unexpected"
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
    fn test_sc2227_redirect_before_pipe() {
        let code = r#"echo test > file | grep pattern"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2227_redirect_after_pipe_ok() {
        let code = r#"echo test | grep pattern > file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2227_no_redirect_ok() {
        let code = r#"echo test | grep pattern"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2227_multiple_redirects() {
        let code = r#"cmd1 > file1 | cmd2 > file2"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1); // Only first redirect before pipe
    }
    #[test]
    fn test_sc2227_comment_skipped() {
        let code = r#"# echo test > file | grep"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2227_append_redirect() {
        let code = r#"echo data >> log | process"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // >> not matched
    }
    #[test]
    fn test_sc2227_stderr_redirect() {
        let code = r#"cmd 2> errors | filter"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2227_no_code() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2227_simple_command() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2227_grouped_commands() {
        let code = r#"{ cmd1; cmd2; } > file | process"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
