//! SC2178: Variable was used as an array but is now assigned a string
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! array=(a b c)
//! array="single value"  # Overwrites array with string
//! echo "${array[@]}"    # Only prints "single value"
//! ```
//!
//! Good:
//! ```bash
//! array=(a b c)
//! array[0]="first"      # Update specific element
//! # OR
//! array=("single value") # Keep as array
//! ```
//!
//! # Rationale
//!
//! Assigning a string to an array variable:
//! - Converts array to string
//! - Loses all elements except the first
//! - Usually a bug
//!
//! Use array syntax for assignment or update individual elements.
//!
//! # Auto-fix
//!
//! Warning only - context needed to determine correct fix

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Check for string assignment to array variable
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Track array declarations
    let mut array_vars = std::collections::HashSet::new();

    // Pattern for array declaration: var=(...)
    let array_decl_pattern = Regex::new(r"([A-Za-z_][A-Za-z0-9_]*)=\(").unwrap();

    // Pattern for string assignment: var="..." or var=...
    let string_assign_pattern =
        Regex::new(r#"([A-Za-z_][A-Za-z0-9_]*)=(?:"[^"]*"|'[^']*'|[^\s;]+)"#).unwrap();

    // First pass: identify array variables
    for line in source.lines() {
        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in array_decl_pattern.captures_iter(line) {
            let var_name = cap.get(1).unwrap().as_str();
            array_vars.insert(var_name.to_string());
        }
    }

    // Second pass: find string assignments to array variables
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in string_assign_pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let var_name = cap.get(1).unwrap().as_str();

            // Check if this variable was declared as an array
            if array_vars.contains(var_name) {
                // Skip if this is array syntax (has parentheses or brackets)
                if line.contains("=(") || line.contains("[") {
                    continue;
                }

                let start_col = full_match.start() + 1;
                let end_col = full_match.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2178",
                    Severity::Warning,
                    "Variable was used as an array but is now assigned a string",
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
    fn test_sc2178_basic_detection() {
        let script = "array=(a b c)\narray=\"string\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2178");
    }

    #[test]
    fn test_sc2178_with_quotes() {
        let script = "files=(*.txt)\nfiles='single file'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2178_no_quotes() {
        let script = "items=(x y z)\nitems=single";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2178_multiple_vars() {
        let script = "a=(1 2)\nb=(3 4)\na=\"str1\"\nb=\"str2\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2178_false_positive_indexed_assign() {
        let script = "array=(a b c)\narray[0]=\"updated\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2178_false_positive_array_reassign() {
        let script = "array=(a b c)\narray=(x y z)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2178_false_positive_in_comment() {
        let script = "array=(a b c)\n# array=\"string\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2178_multiline() {
        let script = "array=(a b c)\necho test\narray=\"string\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2178_in_function() {
        let script = "func() {\n  local arr=(1 2 3)\n  arr=\"value\"\n}";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2178_underscore_var() {
        let script = "my_array=(a b)\nmy_array=\"text\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
