// SC2147: Literal tilde in PATH doesn't expand. Use $HOME or unquoted ~.
//
// Tilde (~) only expands at the beginning of a word and when unquoted.
// Inside quotes, it's treated literally as "~" character.
//
// Examples:
// Bad:
//   PATH="~/bin:$PATH"            // Literal "~/bin", not expanded
//   export PATH="~/.local/bin:$PATH"
//   PYTHONPATH="~/lib:$PYTHONPATH"
//
// Good:
//   PATH="$HOME/bin:$PATH"        // Use $HOME instead
//   PATH=~/bin:$PATH               // Unquoted tilde (but risks word splitting)
//   PATH="${HOME}/bin:${PATH}"    // Safest option
//
// Impact: Path won't work - directory "~" literally doesn't exist

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static TILDE_IN_QUOTES: Lazy<Regex> = Lazy::new(|| {
    // Match: PATH="~/..." or similar environment variables with quoted tilde
    // Common path variables: PATH, LD_LIBRARY_PATH, PYTHONPATH, CLASSPATH, etc.
    // Use * (zero or more) to match just "PATH" and also longer names like "PYTHONPATH"
    Regex::new(r#"\b[A-Z_]*PATH="[^"]*~/[^"]*""#).unwrap()
});

static TILDE_IN_ASSIGNMENT: Lazy<Regex> = Lazy::new(|| {
    // Match: any variable="~/..." assignment
    Regex::new(r#"\b\w+="~/[^"]*""#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for tilde in quoted PATH-like variables
        for mat in TILDE_IN_QUOTES.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2147",
                Severity::Warning,
                "Literal tilde in PATH doesn't expand. Use $HOME or unquoted ~".to_string(),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }

        // Check for tilde in any quoted variable assignment
        // But skip if already caught by TILDE_IN_QUOTES
        if !TILDE_IN_QUOTES.is_match(line) {
            for mat in TILDE_IN_ASSIGNMENT.find_iter(line) {
                let start_col = mat.start() + 1;
                let end_col = mat.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2147",
                    Severity::Warning,
                    "Literal tilde in quotes doesn't expand. Use $HOME instead".to_string(),
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
    fn test_sc2147_tilde_in_path() {
        let code = r#"PATH="~/bin:$PATH""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("expand"));
    }

    #[test]
    fn test_sc2147_tilde_in_pythonpath() {
        let code = r#"PYTHONPATH="~/lib:$PYTHONPATH""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2147_tilde_in_ld_library_path() {
        let code = r#"LD_LIBRARY_PATH="~/local/lib:$LD_LIBRARY_PATH""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2147_home_in_path_ok() {
        let code = r#"PATH="$HOME/bin:$PATH""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2147_unquoted_tilde_ok() {
        let code = r#"PATH=~/bin:$PATH"#;
        let result = check(code);
        // Unquoted tilde expands (though it risks word splitting)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2147_comment_ok() {
        let code = r#"# PATH="~/bin:$PATH""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2147_tilde_in_regular_var() {
        let code = r#"mydir="~/projects""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2147_multiple() {
        let code = r#"
PATH="~/bin:$PATH"
PYTHONPATH="~/lib:$PYTHONPATH"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2147_tilde_in_middle() {
        let code = r#"PATH="/usr/local/bin:~/bin:$PATH""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2147_export_statement() {
        let code = r#"export PATH="~/.local/bin:$PATH""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
