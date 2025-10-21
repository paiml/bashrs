// SC2153: Possible misspelling: VAR is not assigned, but var is
//
// Detects cases where an uppercase variable is referenced but never assigned,
// while a similar lowercase variable exists. This often indicates a typo.
//
// Examples:
// Bad:
//   var="value"
//   echo "$VAR"  # Typo: should be $var
//
// Good:
//   var="value"
//   echo "$var"

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

static VAR_ASSIGNMENT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"([A-Za-z_][A-Za-z0-9_]*)=").unwrap());

static VAR_REFERENCE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Track all assigned variables
    let mut assigned_vars = HashSet::new();

    for line in source.lines() {
        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in VAR_ASSIGNMENT.captures_iter(line) {
            if let Some(var) = cap.get(1) {
                assigned_vars.insert(var.as_str().to_string());
            }
        }
    }

    // Check for references to unassigned uppercase variables
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in VAR_REFERENCE.captures_iter(line) {
            if let Some(var) = cap.get(1) {
                let var_name = var.as_str();

                // Skip if already assigned
                if assigned_vars.contains(var_name) {
                    continue;
                }

                // Skip common environment variables
                let env_vars = [
                    "PATH", "HOME", "USER", "SHELL", "PWD", "OLDPWD", "LANG", "LC_ALL", "TERM",
                    "EDITOR", "PAGER",
                ];
                if env_vars.contains(&var_name) {
                    continue;
                }

                // Check if lowercase version exists
                let lowercase = var_name.to_lowercase();
                if var_name.chars().any(|c| c.is_uppercase()) && assigned_vars.contains(&lowercase)
                {
                    let start_col = var.start() + 1;
                    let end_col = var.end() + 1;

                    let diagnostic = Diagnostic::new(
                        "SC2153",
                        Severity::Warning,
                        format!(
                            "Possible misspelling: {} is not assigned, but {} is",
                            var_name, lowercase
                        ),
                        Span::new(line_num, start_col, line_num, end_col),
                    );

                    result.add(diagnostic);
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
    fn test_sc2153_basic_misspelling() {
        let code = r#"
var="value"
echo "$VAR"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2153");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("VAR"));
        assert!(result.diagnostics[0].message.contains("var"));
    }

    #[test]
    fn test_sc2153_correct_casing_ok() {
        let code = r#"
var="value"
echo "$var"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2153_uppercase_assigned_ok() {
        let code = r#"
VAR="value"
echo "$VAR"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2153_env_var_ok() {
        let code = r#"
echo "$PATH"
echo "$HOME"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2153_multiple_vars() {
        let code = r#"
file="test.txt"
name="example"
echo "$FILE"
echo "$NAME"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2153_mixed_case() {
        let code = r#"
myVar="value"
echo "$MYVAR"
"#;
        let result = check(code);
        // Note: Simple lowercase comparison won't catch camelCase vs UPPERCASE
        // This is a known limitation - only exact case-insensitive matches work
        // "MYVAR".to_lowercase() = "myvar" != "myVar"
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2153_no_lowercase_ok() {
        let code = r#"
echo "$UNDEFINED"
"#;
        let result = check(code);
        // No lowercase version exists, so no warning
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2153_braces() {
        let code = r#"
var="value"
echo "${VAR}"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2153_both_assigned_ok() {
        let code = r#"
var="lower"
VAR="upper"
echo "$var"
echo "$VAR"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2153_function_local() {
        let code = r#"
function test() {
    local var="value"
    echo "$VAR"
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
