// SC2042: Use printf instead of echo with backslash escapes.
//
// The behavior of `echo` with backslash escapes is non-portable:
// - Some shells interpret \n, \t by default (bash, zsh)
// - Others require echo -e (dash, sh)
// - POSIX doesn't specify the behavior
//
// Examples:
// Bad:
//   echo "line1\nline2"     // May not work (depends on shell)
//   echo "tab\there"        // Unpredictable
//   echo "Path:\t$HOME"     // Might print literal \t
//
// Good:
//   printf "line1\nline2\n" // Always works
//   printf "tab\there\n"    // Portable
//   printf "Path:\t%s\n" "$HOME"  // Correct escaping
//
// Note: printf is POSIX-standard and handles escapes consistently.
// Always use printf for formatted output with escape sequences.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ECHO_WITH_ESCAPES: Lazy<Regex> = Lazy::new(|| {
    // Match: echo with backslash escapes like \n, \t, \r, \\, etc.
    Regex::new(r#"\becho\s+[^|;&\n]*\\[ntr\\]"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for echo with backslash escapes
        if let Some(mat) = ECHO_WITH_ESCAPES.find(line) {
            let pos = mat.start();

            // Skip echo -e (user is aware of escapes)
            if line.contains("echo -e") {
                continue;
            }

            // Check if the escape sequence is inside single quotes
            // Count single quotes from start to the position of the escape
            // Find where the escape actually is (after echo command)
            let escape_pos = mat.as_str().rfind('\\').map(|p| pos + p).unwrap_or(pos);
            let before_escape = &line[..escape_pos];
            let single_quote_count = before_escape.matches('\'').count();

            if single_quote_count % 2 == 1 {
                continue; // Escape is inside single quotes
            }

            let start_col = pos + 1;
            let end_col = start_col + mat.as_str().len();

            let diagnostic = Diagnostic::new(
                "SC2042",
                Severity::Warning,
                "Use printf instead of echo with backslash escapes. Echo behavior is non-portable."
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
    fn test_sc2042_echo_newline() {
        let code = r#"echo "line1\nline2""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2042");
        assert!(result.diagnostics[0].message.contains("printf"));
    }

    #[test]
    fn test_sc2042_echo_tab() {
        let code = r#"echo "name\tvalue""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2042_echo_carriage_return() {
        let code = r#"echo "Progress\r100%""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2042_echo_backslash() {
        let code = r#"echo "Path: C:\\Windows\\System32""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2042_printf_ok() {
        let code = r#"printf "line1\nline2\n""#;
        let result = check(code);
        // printf is correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2042_echo_e_ok() {
        let code = r#"echo -e "line1\nline2""#;
        let result = check(code);
        // echo -e explicitly enables escapes
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2042_echo_plain_ok() {
        let code = r#"echo "hello world""#;
        let result = check(code);
        // No backslash escapes, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2042_in_single_quotes_ok() {
        let code = r#"echo 'line1\nline2'"#;
        let result = check(code);
        // Single quotes, backslash is literal
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2042_comment_ok() {
        let code = r#"# echo "line1\nline2""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2042_multiple_escapes() {
        let code = r#"echo "Name:\t$USER\nHome:\t$HOME""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2042_with_variables() {
        let code = r#"echo "Result:\n$output""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
