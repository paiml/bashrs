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

/// Check if a line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if line has array syntax (parentheses or brackets)
fn has_array_syntax(line: &str) -> bool {
    line.contains("=(") || line.contains("[")
}

/// Create diagnostic for array-to-string assignment
fn create_array_to_string_diagnostic(
    line_num: usize,
    start_col: usize,
    end_col: usize,
) -> Diagnostic {
    Diagnostic::new(
        "SC2178",
        Severity::Warning,
        "Variable was used as an array but is now assigned a string",
        Span::new(line_num, start_col, line_num, end_col),
    )
}

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
        if is_comment_line(line) {
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

        if is_comment_line(line) {
            continue;
        }

        for cap in string_assign_pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let var_name = cap.get(1).unwrap().as_str();

            // Check if this variable was declared as an array
            if array_vars.contains(var_name) {
                // Skip if this is array syntax (has parentheses or brackets)
                if has_array_syntax(line) {
                    continue;
                }

                let start_col = full_match.start() + 1;
                let end_col = full_match.end() + 1;

                let diagnostic = create_array_to_string_diagnostic(line_num, start_col, end_col);

                result.add(diagnostic);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Manual Property Tests =====

    #[test]
    fn prop_sc2178_comments_never_diagnosed() {
        // Property: Comment lines should never produce diagnostics
        let test_cases = vec![
            "array=(a b c)\n# array=\"string\"",
            "files=(*.txt)\n  # files='file'",
            "items=(x y)\n\t# items=value",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2178_array_reassignment_never_diagnosed() {
        // Property: Array-to-array reassignment should never be diagnosed
        let test_cases = vec![
            "array=(a b c)\narray=(x y z)",
            "files=(*.txt)\nfiles=(*.md)",
            "items=(1 2)\nitems=(3 4 5)",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2178_indexed_assignment_never_diagnosed() {
        // Property: Indexed array assignment should never be diagnosed
        let test_cases = vec![
            "array=(a b c)\narray[0]=\"updated\"",
            "files=(*.txt)\nfiles[1]='file.md'",
            "items=(x y)\nitems[2]=z",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2178_non_array_vars_never_diagnosed() {
        // Property: Non-array variables should never be diagnosed
        let test_cases = vec![
            "var=\"string\"\nvar=\"other string\"",
            "name=value\nname=newvalue",
            "file='test.txt'\nfile='other.txt'",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2178_string_to_array_always_diagnosed() {
        // Property: String assignment to array variable should always be diagnosed
        let test_cases = vec![
            "array=(a b c)\narray=\"string\"",
            "files=(*.txt)\nfiles='single file'",
            "items=(x y z)\nitems=single",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
            assert!(result.diagnostics[0].message.contains("array"));
        }
    }

    #[test]
    fn prop_sc2178_multiple_violations_all_diagnosed() {
        // Property: Multiple string assignments to arrays should all be diagnosed
        let code = "a=(1 2)\nb=(3 4)\na=\"str1\"\nb=\"str2\"";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn prop_sc2178_diagnostic_code_always_sc2178() {
        // Property: All diagnostics must have code "SC2178"
        let code = "a=(1 2)\nb=(3 4)\na=\"str1\"\nb=\"str2\"";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(&diagnostic.code, "SC2178");
        }
    }

    #[test]
    fn prop_sc2178_diagnostic_severity_always_warning() {
        // Property: All diagnostics must be Warning severity
        let code = "array=(a b c)\narray=\"string\"";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Warning);
        }
    }

    #[test]
    fn prop_sc2178_works_across_lines() {
        // Property: Detection works regardless of line distance
        let test_cases = vec![
            "array=(a b)\narray=\"str\"",            // Immediate
            "array=(a b)\necho test\narray=\"str\"", // One line between
            "array=(a b)\n\n\narray=\"str\"",        // Multiple empty lines
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
        }
    }

    #[test]
    fn prop_sc2178_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Original Unit Tests =====

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
