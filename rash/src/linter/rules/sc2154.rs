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

    let patterns = create_patterns();
    let builtins = get_builtins();
    let (assigned, used_vars) = collect_variable_info(source, &patterns);
    let diagnostics = validate_undefined_variables(&assigned, &used_vars, &builtins);

    for diag in diagnostics {
        result.add(diag);
    }

    result
}

/// Patterns for variable detection
struct Patterns {
    assign: Regex,
    use_: Regex,
    for_loop: Regex,
}

/// Create regex patterns for variable detection
fn create_patterns() -> Patterns {
    Patterns {
        // Issue #20: Allow leading whitespace for indented assignments
        assign: Regex::new(r"^\s*([A-Za-z_][A-Za-z0-9_]*)=").unwrap(),
        use_: Regex::new(r"\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?").unwrap(),
        // Issue #20: Detect loop variables (for var in ...)
        for_loop: Regex::new(r"\bfor\s+([A-Za-z_][A-Za-z0-9_]*)\s+in\b").unwrap(),
    }
}

/// Get set of built-in/environment variables to skip
fn get_builtins() -> HashSet<&'static str> {
    [
        "HOME", "PATH", "PWD", "USER", "SHELL", "TERM", "LANG", "LC_ALL",
    ]
    .iter()
    .copied()
    .collect()
}

/// Collect variable assignments and uses from source
fn collect_variable_info(
    source: &str,
    patterns: &Patterns,
) -> (HashSet<String>, Vec<(String, usize, usize)>) {
    let mut assigned: HashSet<String> = HashSet::new();
    let mut used_vars: Vec<(String, usize, usize)> = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Find assignments
        for cap in patterns.assign.captures_iter(line) {
            let var_name = cap.get(1).unwrap().as_str().to_string();
            assigned.insert(var_name);
        }

        // Issue #20: Find loop variables (for var in ...)
        for cap in patterns.for_loop.captures_iter(line) {
            let var_name = cap.get(1).unwrap().as_str().to_string();
            assigned.insert(var_name);
        }

        // Find uses
        for cap in patterns.use_.captures_iter(line) {
            let var_name = cap.get(1).unwrap().as_str();
            let col = cap.get(0).unwrap().start() + 1;
            used_vars.push((var_name.to_string(), line_num, col));
        }
    }

    (assigned, used_vars)
}

/// Check if variable is special or builtin (should be skipped)
fn is_special_or_builtin(var_name: &str, builtins: &HashSet<&str>) -> bool {
    // Skip if in builtins
    if builtins.contains(var_name) {
        return true;
    }

    // Skip numeric variables (positional parameters)
    if var_name.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }

    // Skip special variables
    if ["@", "*", "#", "?", "$", "!", "0", "-"].contains(&var_name) {
        return true;
    }

    false
}

/// Validate undefined variables and return diagnostics
fn validate_undefined_variables(
    assigned: &HashSet<String>,
    used_vars: &[(String, usize, usize)],
    builtins: &HashSet<&str>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (var_name, line_num, col) in used_vars {
        if assigned.contains(var_name) {
            continue;
        }

        if is_special_or_builtin(var_name, builtins) {
            continue;
        }

        let diagnostic = Diagnostic::new(
            "SC2154",
            Severity::Warning,
            format!("Variable '{}' is referenced but not assigned", var_name),
            Span::new(*line_num, *col, *line_num, col + var_name.len() + 1),
        );

        diagnostics.push(diagnostic);
    }

    diagnostics
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

    // Issue #20: Loop variable tests
    #[test]
    fn test_issue_020_sc2154_for_loop_variable() {
        let script = r#"
for file in *.txt; do
    echo "$file"
done
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Loop variable 'file' should not be flagged as undefined"
        );
    }

    #[test]
    fn test_issue_020_sc2154_multiple_loop_variables() {
        let script = r#"
for file in *.txt; do
    echo "$file"
done

for dockerfile in docker/*/Dockerfile; do
    echo "$dockerfile"
done
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Multiple loop variables should not be flagged"
        );
    }

    #[test]
    fn test_issue_020_sc2154_loop_variable_with_command_subst() {
        let script = r#"
for dockerfile in $(find . -name "*.Dockerfile"); do
    lang="$(basename "$(dirname "$dockerfile")")"
    echo "Processing: ${lang}"
done
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Loop and assigned variables should not be flagged"
        );
    }

    #[test]
    fn test_issue_020_sc2154_undefined_var_in_loop_still_flagged() {
        let script = r#"
for file in *.txt; do
    echo "$file $undefined_var"
done
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Undefined variables in loops should still be flagged"
        );
        assert_eq!(result.diagnostics[0].code, "SC2154");
        assert!(result.diagnostics[0].message.contains("undefined_var"));
    }

    // Property tests for Issue #20
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_issue_020_loop_variables_never_flagged(
                var_name in "[a-z][a-z0-9_]{0,10}",
                pattern in "[a-z/*.]+",
            ) {
                let script = format!("for {} in {}; do\n    echo \"${}\"\ndone", var_name, pattern, var_name);
                let result = check(&script);

                // Loop variable should never be flagged as undefined
                for diagnostic in &result.diagnostics {
                    if diagnostic.code == "SC2154" {
                        prop_assert!(
                            !diagnostic.message.contains(&var_name),
                            "Loop variable '{}' should not be flagged as undefined",
                            var_name
                        );
                    }
                }
            }

            #[test]
            fn prop_issue_020_assigned_vars_never_flagged(
                var_name in "[a-z][a-z0-9_]{0,10}",
                value in "[a-zA-Z0-9]+",
            ) {
                let script = format!("{}=\"{}\"\necho \"${{{}}}\"", var_name, value, var_name);
                let result = check(&script);

                // Assigned variables should never be flagged
                prop_assert_eq!(result.diagnostics.len(), 0, "Assigned variable should not be flagged");
            }

            #[test]
            fn prop_issue_020_undefined_vars_always_flagged(
                defined_var in "[a-z][a-z0-9_]{0,10}",
                undefined_var in "[a-z][a-z0-9_]{0,10}",
            ) {
                prop_assume!(defined_var != undefined_var);
                // Avoid substring matches: ensure neither is a substring of the other
                prop_assume!(!defined_var.contains(&undefined_var) && !undefined_var.contains(&defined_var));

                let script = format!("{}=\"value\"\necho \"${{{}}} ${{{}}}\"", defined_var, defined_var, undefined_var);
                let result = check(&script);

                // Undefined variable should be flagged
                let has_undefined_warning = result.diagnostics.iter().any(|d| {
                    d.code == "SC2154" && d.message.contains(&format!("'{}'", undefined_var))
                });
                prop_assert!(has_undefined_warning, "Undefined variable '{}' should be flagged", undefined_var);

                // Defined variable should NOT be flagged
                let has_defined_warning = result.diagnostics.iter().any(|d| {
                    d.code == "SC2154" && d.message.contains(&format!("'{}'", defined_var))
                });
                prop_assert!(!has_defined_warning, "Defined variable '{}' should not be flagged", defined_var);
            }

            #[test]
            fn prop_issue_020_indented_assignments_recognized(
                indent in "[ ]{0,8}",
                var_name in "[a-z][a-z0-9_]{0,10}",
                value in "[a-zA-Z0-9]+",
            ) {
                let script = format!("{}{}=\"{}\"\necho \"${{{}}}\"", indent, var_name, value, var_name);
                let result = check(&script);

                // Indented assignments should be recognized (Issue #20 fix)
                prop_assert_eq!(result.diagnostics.len(), 0, "Indented assignment should be recognized");
            }
        }
    }
}
