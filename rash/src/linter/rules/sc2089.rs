// SC2089: Quotes/backslashes in assignment will be treated literally. Use array or separate assignment
//
// When assigning to variables, quotes and backslashes are stored literally
// and don't provide quoting when the variable is used.
//
// Examples:
// Bad:
//   args="-name '*.txt'"         // Quotes stored literally
//   find . $args                 // Expands wrong
//
// Good:
//   args=(-name '*.txt')         // Array
//   find . "${args[@]}"          // Correct expansion
//   find . -name '*.txt'         // Direct usage
//
// Impact: Arguments not parsed correctly, quoting doesn't work

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ASSIGNMENT_WITH_QUOTES: Lazy<Regex> = Lazy::new(|| {
    // Match: var="something 'quoted' or \"quoted\""
    Regex::new(r#"^[a-zA-Z_][a-zA-Z0-9_]*=["'].*["'].*["']"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }

        // Look for assignments with nested quotes
        if let Some(mat) = ASSIGNMENT_WITH_QUOTES.find(trimmed) {
            let start_col = line.find(mat.as_str()).unwrap() + 1;
            let end_col = start_col + mat.as_str().len();

            let diagnostic = Diagnostic::new(
                "SC2089",
                Severity::Info,
                "Quotes/backslashes will be treated literally. Use array or separate assignment"
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
    fn test_sc2089_quotes_in_assignment() {
        let code = r#"args="-name '*.txt'""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2089_double_quotes() {
        let code = r#"cmd="echo \"hello\"""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2089_array_ok() {
        let code = r#"args=(-name '*.txt')"#;
        let result = check(code);
        // Array assignment is correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2089_simple_assignment_ok() {
        let code = r#"name="value""#;
        let result = check(code);
        // Simple assignment without nested quotes
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2089_no_nested_quotes_ok() {
        let code = r#"path="/usr/local/bin""#;
        let result = check(code);
        // No nested quotes
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2089_comment_ok() {
        let code = r#"# args="-name '*.txt'""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2089_mixed_quotes() {
        let code = r#"options='--flag "value"'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2089_command_substitution() {
        let code = r#"result=$(echo "test")"#;
        let result = check(code);
        // Command substitution, not literal quotes
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    #[ignore] // TODO: Handle export with nested quotes
    fn test_sc2089_export() {
        let code = r#"export FLAGS="-Wall '-Werror'""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2089_multiline() {
        let code = r#"
args="-name '*.txt'"
find . $args
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
