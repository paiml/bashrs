//! SC2034: Variable assigned but never used
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! unused_var="value"
//! echo "Hello"
//! ```
//!
//! Good:
//! ```bash
//! used_var="value"
//! echo "$used_var"
//! ```
//!
//! # Rationale
//!
//! Variables that are assigned but never used may indicate:
//! - Dead code
//! - Typos in variable names
//! - Forgotten cleanup
//!
//! # Auto-fix
//!
//! Warning only - may be intentional (exported vars, etc.)

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;
use std::collections::{HashMap, HashSet};

/// Check for variables assigned but never used
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Collect assigned variables
    let assign_pattern = Regex::new(r"^([A-Za-z_][A-Za-z0-9_]*)=").unwrap();
    // Collect used variables
    let use_pattern = Regex::new(r"\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?").unwrap();

    let mut assigned: HashMap<String, usize> = HashMap::new();
    let mut used: HashSet<String> = HashSet::new();

    // First pass: collect assignments and uses
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Find assignments
        for cap in assign_pattern.captures_iter(line) {
            let var_name = cap.get(1).unwrap().as_str().to_string();
            assigned.insert(var_name, line_num);
        }

        // Find uses
        for cap in use_pattern.captures_iter(line) {
            let var_name = cap.get(1).unwrap().as_str().to_string();
            used.insert(var_name);
        }
    }

    // Second pass: find unused variables
    for (var_name, line_num) in assigned.iter() {
        if !used.contains(var_name) {
            // Skip common patterns that are intentionally unused
            if var_name.starts_with('_') || var_name.to_uppercase() == *var_name {
                continue; // Skip _ prefixed and ALL_CAPS (often exported)
            }

            let diagnostic = Diagnostic::new(
                "SC2034",
                Severity::Info,
                format!("Variable '{}' is assigned but never used", var_name),
                Span::new(*line_num, 1, *line_num, var_name.len() + 1),
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
    fn test_sc2034_basic_detection() {
        let script = r#"
unused_var="value"
echo "Hello"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2034");
    }

    #[test]
    fn test_sc2034_variable_used() {
        let script = r#"
used_var="value"
echo "$used_var"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2034_multiple_unused() {
        let script = r#"
unused1="value1"
unused2="value2"
echo "Hello"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2034_skip_underscore_prefix() {
        let script = r#"
_private="value"
echo "Hello"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0); // _ prefix skipped
    }

    #[test]
    fn test_sc2034_skip_all_caps() {
        let script = r#"
EXPORTED_VAR="value"
echo "Hello"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0); // ALL_CAPS skipped
    }

    #[test]
    fn test_sc2034_braced_usage() {
        let script = r#"
used_var="value"
echo "${used_var}"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2034_used_in_condition() {
        let script = r#"
check_var="test"
if [ "$check_var" = "test" ]; then
    echo "yes"
fi
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2034_mixed_used_and_unused() {
        let script = r#"
used="value1"
unused="value2"
echo "$used"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2034_used_in_assignment() {
        let script = r#"
var1="value"
var2="$var1"
echo "$var2"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2034_no_false_positive_in_comment() {
        let script = r#"
# unused_var="value"
echo "Hello"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
