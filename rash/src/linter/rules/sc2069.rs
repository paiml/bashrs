// SC2069: To redirect stdout to stderr, use >&2 or 1>&2, not 2>&1
//
// The syntax 2>&1 redirects stderr TO stdout, not the other way around.
// To redirect stdout TO stderr, use >&2 or 1>&2.
//
// Examples:
// Bad (if intent is stdout → stderr):
//   echo "Error" 2>&1           // Redirects stderr to stdout (wrong direction)
//
// Good:
//   echo "Error" >&2            // Redirects stdout to stderr
//   echo "Error" 1>&2           // Explicit: stdout to stderr
//
// Impact: Messages go to wrong stream, breaking error handling

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ECHO_TO_STDERR_WRONG: Lazy<Regex> = Lazy::new(|| {
    // Match: echo ... 2>&1 (trying to send to stderr but using wrong syntax)
    Regex::new(r"\becho\b.*\b2>&1\b").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for echo with 2>&1 (likely wrong - should be >&2)
        if let Some(mat) = ECHO_TO_STDERR_WRONG.find(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2069",
                Severity::Info,
                "To redirect stdout to stderr, use >&2, not 2>&1 (which redirects stderr to stdout)"
                    .to_string(),
                Span::new(line_num, start_col, line_num, end_col),
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
    fn test_sc2069_echo_wrong_redirect() {
        let code = r#"echo "Error message" 2>&1"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2069_echo_correct_ok() {
        let code = r#"echo "Error message" >&2"#;
        let result = check(code);
        // Correct redirection
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2069_echo_explicit_ok() {
        let code = r#"echo "Error message" 1>&2"#;
        let result = check(code);
        // Explicit stdout to stderr
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2069_pipe_ok() {
        let code = r#"command 2>&1 | grep error"#;
        let result = check(code);
        // Combining streams for pipe is valid
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2069_printf_wrong() {
        let code = r#"printf "Error\n" 2>&1"#;
        let result = check(code);
        // printf also affected
        assert_eq!(result.diagnostics.len(), 0); // Only checking echo for now
    }

    #[test]
    fn test_sc2069_comment_ok() {
        let code = r#"# echo "Error" 2>&1"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2069_multiple() {
        let code = r#"echo "Err1" 2>&1; echo "Err2" 2>&1"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1); // Matches the line
    }

    #[test]
    fn test_sc2069_stderr_to_file() {
        let code = r#"echo "Output" 2>error.log"#;
        let result = check(code);
        // Redirecting stderr to file, not related to 2>&1
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2069_background() {
        let code = r#"echo "Message" 2>&1 &"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2069_with_variable() {
        let code = r#"echo "$error_msg" 2>&1"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
