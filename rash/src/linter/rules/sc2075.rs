// SC2075: Escaping quotes in quotes won't work. Use \\' or \"'\"
//
// In single quotes, nothing can be escaped - not even backslashes.
// To include a single quote in a single-quoted string, you must end the string,
// add an escaped quote, and start a new string.
//
// Examples:
// Incorrect:
//   echo 'can\'t'              // Won't work - backslash is literal
//   msg='it\'s wrong'          // Syntax error
//
// Correct:
//   echo 'can'\''t'            // End string, escaped quote, new string
//   echo "can't"               // Use double quotes instead
//   msg='it'"'"'s fixed'       // End, quote in double quotes, continue
//
// Impact: Syntax errors, incorrect string values

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ESCAPED_QUOTE_IN_SINGLE: Lazy<Regex> = Lazy::new(|| {
    // Match: 'string\'more' (escaped quote inside single quotes)
    Regex::new(r"'[^']*\\'[^']*'").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in ESCAPED_QUOTE_IN_SINGLE.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2075",
                Severity::Error,
                "Escaping a single quote in single quotes won't work. Use '\"'\"' or double quotes"
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
    fn test_sc2075_escaped_quote() {
        let code = r#"echo 'can\'t'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2075_its() {
        let code = r#"msg='it\'s broken'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2075_double_quotes_ok() {
        let code = r#"echo "can't""#;
        let result = check(code);
        // Double quotes allow single quotes
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2075_concatenation_ok() {
        let code = r#"echo 'can'"'"'t'"#;
        let result = check(code);
        // Correct workaround (ends string, quotes, starts new)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2075_plain_string_ok() {
        let code = r#"echo 'hello world'"#;
        let result = check(code);
        // No escaped quotes
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2075_comment_ok() {
        let code = r#"# echo 'can\'t'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2075_double_escape() {
        let code = r#"path='C:\\Users\\file'"#;
        let result = check(code);
        // Backslashes but no quote
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2075_multiple() {
        let code = r#"echo 'don\'t' 'won\'t'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2075_in_command_sub() {
        let code = r#"result=$(echo 'can\'t')"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2075_escaped_backslash() {
        let code = r#"path='some\\path'"#;
        let result = check(code);
        // Backslash not escaping a quote
        assert_eq!(result.diagnostics.len(), 0);
    }
}
