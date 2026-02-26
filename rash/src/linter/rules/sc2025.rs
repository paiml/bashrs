// SC2025: Make sure all escape sequences are enclosed in quotes
//
// Escape sequences like \n, \t need to be in quotes to be processed correctly.
// Without quotes, the backslash may be interpreted literally or stripped.
//
// Examples:
// Bad:
//   echo Hello\nWorld                // Backslash may be literal
//   printf Format:\tValue            // \t not processed
//   msg=Line1\nLine2                 // Backslash literal in assignment
//
// Good:
//   echo "Hello\nWorld"              // Quoted, processed by echo -e
//   printf "Format:\tValue"          // Quoted, \t works
//   msg="Line1\nLine2"               // Quoted, stored correctly
//
// Note: Some commands like printf process escape sequences, but they still
// need to be quoted to prevent shell interpretation.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static UNQUOTED_ESCAPE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: \n, \t, \r, etc. outside of quotes
    // Look for backslash followed by common escape char
    Regex::new(r"\\[ntr]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Find all escape sequences
        for m in UNQUOTED_ESCAPE.find_iter(line) {
            let pos = m.start();

            // Check if this is inside quotes
            let before = &line[..pos];
            let double_quote_count = before.matches('"').count();
            let single_quote_count = before.matches('\'').count();

            // If odd number of quotes, we're inside a quoted string
            if double_quote_count % 2 == 1 || single_quote_count % 2 == 1 {
                continue;
            }

            let start_col = pos + 1;
            let end_col = m.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2025",
                Severity::Info,
                "Make sure all escape sequences are enclosed in quotes".to_string(),
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
    fn test_sc2025_unquoted_newline() {
        let code = r#"echo Hello\nWorld"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2025");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("quotes"));
    }

    #[test]
    fn test_sc2025_unquoted_tab() {
        let code = r#"printf Format:\tValue"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2025_unquoted_carriage_return() {
        let code = r#"msg=Line1\rLine2"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2025_quoted_ok() {
        let code = r#"echo "Hello\nWorld""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2025_single_quoted_ok() {
        let code = r#"echo 'Hello\nWorld'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2025_printf_quoted_ok() {
        let code = r#"printf "Format:\tValue\n""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2025_assignment_quoted_ok() {
        let code = r#"msg="Line1\nLine2""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2025_multiple_issues() {
        let code = r#"
echo Start\n
printf Tab:\t
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2025_no_escape_ok() {
        let code = r#"echo Hello World"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2025_backslash_other_char_ok() {
        let code = r#"path=/usr\/local"#;
        let result = check(code);
        // \/ is not an escape sequence we check
        assert_eq!(result.diagnostics.len(), 0);
    }
}
