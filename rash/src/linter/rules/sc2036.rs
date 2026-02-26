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
use regex::Regex;

static BACKTICK_WITH_UNESCAPED_QUOTES: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| {
        // Match: `...unescaped "...'`
        // Look for backticks containing unescaped double or single quotes
        Regex::new(r#"`([^`]*[^\\])?(["'])([^`]*)`"#).unwrap()
    });

/// Check if line should be analyzed (has backticks and quotes)
fn should_check_line(line: &str) -> bool {
    line.contains('`') && (line.contains('"') || line.contains('\''))
}

/// Check if character at position is a quote
fn is_quote(c: char) -> bool {
    c == '"' || c == '\''
}

/// Check if quote at position is escaped
fn is_escaped_quote(chars: &[char], pos: usize) -> bool {
    pos > 0 && chars[pos - 1] == '\\'
}

/// Check if character at position is an unescaped quote
fn is_unescaped_quote(chars: &[char], pos: usize) -> bool {
    is_quote(chars[pos]) && !is_escaped_quote(chars, pos)
}

/// Find unescaped quote position inside backtick expression
fn find_unescaped_quote_in_backticks(chars: &[char], start: usize) -> Option<usize> {
    let mut i = start + 1; // Skip opening backtick

    while i < chars.len() && chars[i] != '`' {
        if is_unescaped_quote(chars, i) {
            return Some(i);
        }
        i += 1;
    }

    None
}

/// Create diagnostic for unescaped quote in backticks
fn create_backtick_quote_diagnostic(
    line_num: usize,
    backtick_start: usize,
    quote_pos: usize,
) -> Diagnostic {
    let start_col = backtick_start + 1;
    let end_col = quote_pos + 2; // +2 to include quote

    Diagnostic::new(
        "SC2036",
        Severity::Warning,
        "Quotes in backticks need escaping. Use $( ) instead or escape with \\\"".to_string(),
        Span::new(line_num, start_col, line_num, end_col),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') || !should_check_line(line) {
            continue;
        }

        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '`' {
                let backtick_start = i;

                // Check for unescaped quotes inside backticks
                if let Some(quote_pos) = find_unescaped_quote_in_backticks(&chars, backtick_start) {
                    let diagnostic =
                        create_backtick_quote_diagnostic(line_num, backtick_start, quote_pos);
                    result.add(diagnostic);

                    // Skip to end of this backtick expression
                    i = quote_pos + 1;
                    while i < chars.len() && chars[i] != '`' {
                        i += 1;
                    }
                }

                if i < chars.len() && chars[i] == '`' {
                    i += 1; // Skip closing backtick
                }
            } else {
                i += 1;
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
