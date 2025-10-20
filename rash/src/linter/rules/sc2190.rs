//! SC2190: Elements in associative arrays need index, e.g. array=([key]=value)
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! declare -A assoc
//! assoc=(value1 value2)  # Wrong: no keys
//! ```
//!
//! Good:
//! ```bash
//! declare -A assoc
//! assoc=([key1]=value1 [key2]=value2)
//! ```
//!
//! # Rationale
//!
//! Associative arrays require key-value pairs:
//! - Regular array syntax doesn't work
//! - Must use [key]=value syntax
//! - Without keys, assignment fails
//!
//! Always use [key]=value for associative arrays.
//!
//! # Auto-fix
//!
//! Warning only - need keys from user

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Check for associative array without keys
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Track associative array declarations
    let mut assoc_arrays = std::collections::HashSet::new();

    // Pattern for associative array declaration: declare -A var
    let assoc_decl_pattern = Regex::new(r"declare\s+-A\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap();

    // Pattern for array assignment: var=(value1 value2) without [key]=
    let array_assign_pattern = Regex::new(r"([A-Za-z_][A-Za-z0-9_]*)=\(([^)]+)\)").unwrap();

    // First pass: identify associative arrays
    for line in source.lines() {
        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in assoc_decl_pattern.captures_iter(line) {
            let var_name = cap.get(1).unwrap().as_str();
            assoc_arrays.insert(var_name.to_string());
        }
    }

    // Second pass: find assignments without keys
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in array_assign_pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let var_name = cap.get(1).unwrap().as_str();
            let content = cap.get(2).unwrap().as_str();

            // Check if this is an associative array
            if assoc_arrays.contains(var_name) {
                // Check if content has [key]=value syntax
                if !content.contains("[") || !content.contains("]=") {
                    let start_col = full_match.start() + 1;
                    let end_col = full_match.end() + 1;

                    let diagnostic = Diagnostic::new(
                        "SC2190",
                        Severity::Error,
                        "Elements in associative arrays need index, e.g. array=([key]=value)",
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
    fn test_sc2190_basic_detection() {
        let script = "declare -A assoc\nassoc=(value1 value2)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2190");
    }

    #[test]
    fn test_sc2190_multiple_values() {
        let script = "declare -A map\nmap=(a b c d)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2190_single_value() {
        let script = "declare -A dict\ndict=(value)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2190_false_positive_with_keys() {
        let script = "declare -A assoc\nassoc=([key1]=value1 [key2]=value2)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2190_false_positive_single_key() {
        let script = "declare -A map\nmap=([foo]=bar)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2190_false_positive_regular_array() {
        let script = "declare -a array\narray=(a b c)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2190_false_positive_in_comment() {
        let script = "declare -A assoc\n# assoc=(value)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2190_multiline() {
        let script = "declare -A dict\necho test\ndict=(val1 val2)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2190_in_function() {
        let script = "func() {\n  declare -A local_map\n  local_map=(x y)\n}";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2190_multiple_assoc_arrays() {
        let script = "declare -A a1\ndeclare -A a2\na1=(v1)\na2=(v2)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
