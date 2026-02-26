// SC2131: Escape backslashes in single quotes to avoid confusion
//
// Backslashes in single quotes are LITERAL. They don't escape anything.
// To get a literal backslash, just use it. To escape quotes, exit single quotes.
//
// Examples:
// Bad:
//   echo 'path\\to\\file'      // Has literal \\ (not \\)
//   echo 'can\'t'              // Doesn't work - backslash is literal
//
// Good:
//   echo 'path\to\file'        // Single backslash is literal
//   echo 'can'"'"'t'           // Proper quote escaping
//   echo "can't"               // Use double quotes for contractions
//
// Impact: Confusion about escaping in single quotes

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static DOUBLE_BACKSLASH_SINGLE_QUOTE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: '...\\'...'' patterns with double backslashes
    Regex::new(r"'[^']*\\\\[^']*'").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in DOUBLE_BACKSLASH_SINGLE_QUOTE.find_iter(line) {
            let matched = mat.as_str();

            // Check if it contains \\ (double backslash)
            if matched.contains("\\\\") {
                let start_col = mat.start() + 1;
                let end_col = mat.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2131",
                    Severity::Info,
                    "Backslashes in single quotes are literal. Use single \\ for literal backslash"
                        .to_string(),
                    Span::new(line_num, start_col, line_num, end_col),
                );

                result.add(diagnostic);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2131_double_backslash() {
        let code = r#"echo 'path\\to\\file'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2131_single_backslash_ok() {
        let code = "echo 'path\\to\\file'";
        let result = check(code);
        // Single backslash is literal and correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2131_double_quotes_ok() {
        let code = r#"echo "path\\to\\file""#;
        let result = check(code);
        // Double quotes - backslashes escape
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2131_comment_ok() {
        let code = r#"# echo 'path\\to\\file'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2131_regex_pattern() {
        let code = r#"pattern='[0-9]\\+'"#;
        let result = check(code);
        // Regex pattern with escaped +
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2131_multiline() {
        let code = r#"
path='C:\\Users\\Documents'
echo "$path"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2131_multiple() {
        let code = r#"
a='path\\1'
b='path\\2'
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2131_windows_path() {
        let code = r#"winpath='C:\\Windows\\System32'"#;
        let result = check(code);
        // Windows path with double backslashes
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2131_no_backslash_ok() {
        let code = "echo 'hello world'";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2131_escaped_newline() {
        let code = r#"text='line1\\nline2'"#;
        let result = check(code);
        // \\n doesn't expand in single quotes
        assert_eq!(result.diagnostics.len(), 1);
    }
}
