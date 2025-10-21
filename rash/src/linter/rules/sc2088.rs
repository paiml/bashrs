// SC2088: Tilde does not expand in quotes. Use $HOME or remove quotes
//
// The tilde (~) only expands to home directory when unquoted.
// In quotes, it's treated as a literal tilde character.
//
// Examples:
// Bad:
//   path="~/Documents"           // Literal "~/Documents", not expanded
//   cd "~/bin"                   // Tries to cd to literal "~"
//
// Good:
//   path=~/Documents             // Tilde expands to home
//   path="$HOME/Documents"       // Use $HOME in quotes
//   cd ~/bin                     // Unquoted tilde expands
//
// Impact: Paths not resolved correctly, file not found errors

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static TILDE_IN_QUOTES: Lazy<Regex> = Lazy::new(|| {
    // Match: "~..." or '~...' (tilde followed by / or word chars)
    Regex::new(r#"["']~[/a-zA-Z]"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in TILDE_IN_QUOTES.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2088",
                Severity::Warning,
                "Tilde does not expand in quotes. Use $HOME or remove quotes".to_string(),
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
    fn test_sc2088_tilde_in_double_quotes() {
        let code = r#"path="~/Documents""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2088_tilde_in_single_quotes() {
        let code = r#"cd '~/bin'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2088_unquoted_ok() {
        let code = "path=~/Documents";
        let result = check(code);
        // Unquoted tilde expands correctly
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2088_home_var_ok() {
        let code = r#"path="$HOME/Documents""#;
        let result = check(code);
        // Using $HOME is correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2088_literal_tilde_ok() {
        let code = r#"pattern="file~backup""#;
        let result = check(code);
        // Tilde not at start of path
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2088_comment_ok() {
        let code = r#"# path="~/Documents""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2088_user_tilde() {
        let code = r#"path="~user/files""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2088_in_assignment() {
        let code = r#"CONFIG_DIR="~/.config""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2088_in_command() {
        let code = r#"ls "~/Projects""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2088_multiple() {
        let code = r#"a="~/dir1"; b="~/dir2""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
