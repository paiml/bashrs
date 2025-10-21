// SC2036: Quotes in backticks need escaping. Use $( ) instead or escape quotes.
//
// Backticks are the old-style command substitution. Inside backticks, quotes need
// to be escaped with backslashes. Unescaped quotes can cause unexpected behavior.
//
// Examples:
// Bad:
//   result=`echo "hello world"`    // Quotes not escaped
//   out=`grep "pattern" file`      // Can fail
//
// Good:
//   result=`echo \"hello world\"`  // Escaped quotes
//   result=$(echo "hello world")   // Modern syntax (preferred)
//
// Note: $( ) is preferred over backticks because it handles nesting better
// and doesn't require escaping.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static BACKTICK_WITH_UNESCAPED_QUOTES: Lazy<Regex> = Lazy::new(|| {
    // Match: `...unescaped "...'`
    // Look for backticks containing unescaped double or single quotes
    Regex::new(r#"`([^`]*[^\\])?(["'])([^`]*)`"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for backticks with unescaped quotes
        // Simple heuristic: if line contains backticks and quotes, check if quotes are escaped
        if line.contains('`') && (line.contains('"') || line.contains('\'')) {
            // Find all backtick pairs
            let chars: Vec<char> = line.chars().collect();
            let mut i = 0;

            while i < chars.len() {
                if chars[i] == '`' {
                    // Find closing backtick
                    let start = i;
                    i += 1;

                    while i < chars.len() && chars[i] != '`' {
                        // Check for unescaped quotes
                        if (chars[i] == '"' || chars[i] == '\'') && (i == 0 || chars[i - 1] != '\\')
                        {
                            // Found unescaped quote in backtick
                            let start_col = start + 1;
                            let end_col = i + 2; // +2 to include quote

                            let diagnostic = Diagnostic::new(
                                "SC2036",
                                Severity::Warning,
                                "Quotes in backticks need escaping. Use $( ) instead or escape with \\\"".to_string(),
                                Span::new(line_num, start_col, line_num, end_col),
                            );

                            result.add(diagnostic);
                            break; // One diagnostic per backtick expression
                        }
                        i += 1;
                    }

                    if i < chars.len() && chars[i] == '`' {
                        i += 1; // Skip closing backtick
                    }
                } else {
                    i += 1;
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2036_unescaped_double_quotes() {
        let code = r#"result=`echo "hello"`"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2036");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("escaping"));
    }

    #[test]
    fn test_sc2036_unescaped_single_quotes() {
        let code = r#"result=`echo 'hello'`"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2036_escaped_quotes_ok() {
        let code = r#"result=`echo \"hello\"`"#;
        let result = check(code);
        // Escaped quotes are OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2036_modern_syntax_ok() {
        let code = r#"result=$(echo "hello")"#;
        let result = check(code);
        // $( ) syntax doesn't need escaping
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2036_no_quotes_ok() {
        let code = r#"result=`echo hello`"#;
        let result = check(code);
        // No quotes, no problem
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2036_grep_with_quotes() {
        let code = r#"out=`grep "pattern" file`"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2036_comment_ok() {
        let code = r#"# result=`echo "hello"`"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2036_multiple_backticks() {
        let code = r#"
a=`echo "x"`
b=`echo "y"`
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2036_empty_backticks_ok() {
        let code = r#"result=``"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2036_quotes_outside_backticks_ok() {
        let code = r#"echo "hello" `date`"#;
        let result = check(code);
        // Quotes outside backticks are OK
        assert_eq!(result.diagnostics.len(), 0);
    }
}
