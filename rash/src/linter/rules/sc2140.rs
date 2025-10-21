// SC2140: Word is of the form "A"B"C" (B indicated). This is not concatenation.
//
// In shell, adjacent strings don't concatenate like "foo""bar" → "foobar".
// Quotes must be properly closed and reopened.
//
// Examples:
// Bad:
//   echo "Hello "World""           // Not concatenation, syntax error
//   var="foo"bar"baz"              // Malformed string
//   path="/usr""/local"            // Incorrect
//
// Good:
//   echo "Hello World"             // Single quoted string
//   echo "Hello ""World"           // Two separate arguments
//   echo "Hello World"             // Or proper quoting
//   var="foo""bar""baz"            // Or: var="foobarbaz"
//
// Impact: Syntax error or unexpected word splitting

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static MALFORMED_QUOTES: Lazy<Regex> = Lazy::new(|| {
    // Match: "string1"word"string2" where word is unquoted between quoted parts
    // Matches: "foo"bar"baz" (malformed - unquoted word between quotes)
    // Matches: "Hello "World"" (malformed - unquoted word with empty string after)
    // May match: "foo""bar""baz" but we filter this in check() logic
    Regex::new(r#""[^"]*"[a-zA-Z_][a-zA-Z0-9_]*"[^"]*""#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for malformed quote patterns
        for mat in MALFORMED_QUOTES.find_iter(line) {
            let matched = mat.as_str();

            // Skip if this is proper concatenation like ""bar""
            // Pattern: two adjacent quotes at the start mean empty string before the word
            // This indicates: "foo""bar" not "foo"bar"
            if matched.starts_with(r#""""#) {
                // Starts with "" - proper concatenation
                continue;
            }

            // Skip if it looks like proper separate arguments (space between quotes)
            if matched.contains("\" \"") {
                continue;
            }

            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2140",
                Severity::Warning,
                "Word is split between quotes. Use proper quoting or concatenation".to_string(),
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
    fn test_sc2140_malformed_quotes() {
        let code = r#"echo "Hello "World"""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0]
            .message
            .contains("split between quotes"));
    }

    #[test]
    fn test_sc2140_single_string_ok() {
        let code = r#"echo "Hello World""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2140_separate_args_ok() {
        let code = r#"echo "Hello" "World""#;
        let result = check(code);
        // Space between quotes = separate arguments (OK)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2140_comment_ok() {
        let code = r#"# echo "Hello "World"""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2140_variable_assignment() {
        let code = r#"var="foo"bar"baz""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2140_path() {
        let code = r#"path="/usr"local"/bin""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2140_proper_concat_ok() {
        let code = r#"var="foo""bar""baz""#;
        let result = check(code);
        // Adjacent quoted strings without unquoted word in between
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2140_single_quotes_ok() {
        let code = "echo 'Hello World'";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2140_multiple() {
        let code = r#"
echo "Hello "World""
var="foo"bar"baz"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2140_escaped_ok() {
        let code = r#"echo "Hello \"World\"""#;
        let result = check(code);
        // Escaped quotes inside string are OK
        assert_eq!(result.diagnostics.len(), 0);
    }
}
