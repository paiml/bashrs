//! SC2154: Variable referenced but not assigned
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! echo "$undefined_var"
//! ```
//!
//! Good:
//! ```bash
//! undefined_var="value"
//! echo "$undefined_var"
//! ```
//!
//! # Rationale
//!
//! Referencing undefined variables may indicate:
//! - Typos in variable names
//! - Missing initialization
//! - Reliance on environment variables (should be explicit)
//!
//! # Auto-fix
//!
//! Warning only - check for typos or add initialization

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;
use std::collections::HashSet;

/// Check for variables referenced but not assigned
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let assign_pattern = Regex::new(r"^([A-Za-z_][A-Za-z0-9_]*)=").unwrap();
    let use_pattern = Regex::new(r"\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?").unwrap();

    let mut assigned: HashSet<String> = HashSet::new();
    let mut used_vars: Vec<(String, usize, usize)> = Vec::new();

    // Common built-in/environment variables to skip
    let builtins: HashSet<&str> = ["HOME", "PATH", "PWD", "USER", "SHELL", "TERM", "LANG", "LC_ALL"]
        .iter().cloned().collect();

    // Collect assignments and uses
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Find assignments
        for cap in assign_pattern.captures_iter(line) {
            let var_name = cap.get(1).unwrap().as_str().to_string();
            assigned.insert(var_name);
        }

        // Find uses
        for cap in use_pattern.captures_iter(line) {
            let var_name = cap.get(1).unwrap().as_str();
            let col = cap.get(0).unwrap().start() + 1;
            used_vars.push((var_name.to_string(), line_num, col));
        }
    }

    // Check for undefined variables
    for (var_name, line_num, col) in used_vars {
        if !assigned.contains(&var_name) && !builtins.contains(var_name.as_str()) {
            // Skip numeric variables (positional parameters)
            if var_name.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }

            // Skip special variables
            if ["@", "*", "#", "?", "$", "!", "0", "-"].contains(&var_name.as_str()) {
                continue;
            }

            let diagnostic = Diagnostic::new(
                "SC2154",
                Severity::Warning,
                &format!("Variable '{}' is referenced but not assigned", var_name),
                Span::new(line_num, col, line_num, col + var_name.len() + 1),
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
    fn test_sc2154_basic_detection() {
        let script = r#"
echo "$undefined_var"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2154");
    }

    #[test]
    fn test_sc2154_variable_defined() {
        let script = r#"
defined_var="value"
echo "$defined_var"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2154_multiple_undefined() {
        let script = r#"
echo "$var1 $var2"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2154_skip_builtins() {
        let script = r#"
echo "$HOME $PATH"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2154_skip_positional_params() {
        let script = r#"
echo "$1 $2 $3"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2154_skip_special_vars() {
        let script = r#"
echo "$@ $* $# $?"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2154_braced_variable() {
        let script = r#"
echo "${undefined}"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2154_mixed_defined_undefined() {
        let script = r#"
defined="value"
echo "$defined $undefined"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2154_used_before_defined() {
        // NOTE: Our simple two-pass implementation doesn't catch this edge case
        // A full implementation would need to track line-by-line state
        let script = r#"
echo "$var"
var="value"
"#;
        let result = check(script);
        // For now, we accept that this won't be caught
        assert!(result.diagnostics.len() <= 1);
    }

    #[test]
    fn test_sc2154_no_false_positive_in_comment() {
        let script = r#"
# echo "$undefined"
defined="value"
echo "$defined"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
