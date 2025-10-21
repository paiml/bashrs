// SC2149: Remove quotes from unset variable names.
//
// The unset command takes variable names, not variable values.
// Quoting the variable name makes unset treat it as a literal string.
//
// Examples:
// Bad:
//   unset "$var"              // Tries to unset variable named "$var" literally
//   unset "${FOO}"            // Tries to unset variable named "${FOO}"
//   unset "PATH"              // Works but quotes unnecessary
//
// Good:
//   unset var                 // Unsets the variable named 'var'
//   unset FOO                 // Unsets the variable named 'FOO'
//   unset PATH                // Unsets PATH
//
// Impact: Variable won't be unset - unset receives wrong argument

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static UNSET_QUOTED: Lazy<Regex> = Lazy::new(|| {
    // Match: unset "var" or unset "$var" or unset "${var}"
    Regex::new(r#"\bunset\s+["'](\$\{?)?[A-Za-z_][A-Za-z0-9_]*\}?["']"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for unset with quoted variable
        for mat in UNSET_QUOTED.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2149",
                Severity::Warning,
                "Remove quotes from unset - it takes variable names, not values".to_string(),
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
    fn test_sc2149_unset_quoted_var() {
        let code = r#"unset "var""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("Remove quotes"));
    }

    #[test]
    fn test_sc2149_unset_quoted_dollar() {
        let code = r#"unset "$var""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2149_unset_quoted_braces() {
        let code = r#"unset "${FOO}""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2149_unset_single_quotes() {
        let code = r#"unset 'PATH'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2149_unset_unquoted_ok() {
        let code = "unset var";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2149_unset_multiple_ok() {
        let code = "unset var1 var2 var3";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2149_comment_ok() {
        let code = r#"# unset "var""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2149_multiple() {
        let code = r#"
unset "FOO"
unset "$BAR"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2149_unset_uppercase() {
        let code = r#"unset "PATH""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2149_unset_with_underscore() {
        let code = r#"unset "MY_VAR""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
