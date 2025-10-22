// SC2230: which is non-standard, use command -v instead
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static WHICH_COMMAND: Lazy<Regex> = Lazy::new(|| {
    // Match: which command_name
    Regex::new(r"\bwhich\s+").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }
        // Skip echo commands (string literals)
        if line.trim_start().starts_with("echo ") {
            continue;
        }

        if WHICH_COMMAND.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2230",
                Severity::Info,
                "which is non-standard. Use 'command -v' for POSIX compatibility".to_string(),
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
    fn test_sc2230_which_command() {
        let code = "which bash";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2230_command_v_ok() {
        let code = "command -v bash";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2230_which_in_test() {
        let code = "if which docker > /dev/null; then";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2230_which_assignment() {
        let code = "path=$(which gcc)";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2230_comment_skipped() {
        let code = "# which python";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2230_multiple_which() {
        let code = "which node\nwhich npm";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
    #[test]
    fn test_sc2230_type_ok() {
        let code = "type bash";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2230_no_code() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2230_normal_command() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2230_string_literal() {
        let code = r#"echo "which is non-standard""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // "which" in string
    }
}
