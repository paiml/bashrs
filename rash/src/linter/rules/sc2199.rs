// SC2199: Arrays implicitly concatenate in [[ ]]. Use ${array[@]} or ${array[*]} to access elements
//
// When an array is used bare (without [@] or [*]) in [[ ]] conditionals, it only
// uses the first element, not all elements. This is rarely what's intended.
//
// Examples:
// Bad:
//   [[ $array = "value" ]]     # Only checks first element
//   [[ -n $array ]]             # Only checks if first element is non-empty
//
// Good:
//   [[ ${array[0]} = "value" ]]  # Explicitly check first element
//   [[ ${array[@]} ]]            # Check if array has elements
//   [[ -n ${array[*]} ]]         # Check if any element is non-empty
//
// Also Bad:
//   [[ ${array} = "value" ]]    # Bare array reference

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ARRAY_IN_CONDITIONAL: Lazy<Regex> = Lazy::new(|| {
    // Match: $var, ${var}, ${var[...]}, etc.
    // Capture the variable name and optionally any subscript
    Regex::new(r"\$\{?([a-z_][a-z0-9_]*)(\[[^\]]*\])?\}?").unwrap()
});

static DOUBLE_BRACKET: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[\[.*?\]\]").unwrap());

/// Check if a line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if line contains double bracket [[ ]]
fn has_double_bracket(line: &str) -> bool {
    line.contains("[[")
}

/// Check if variable has a subscript
fn has_subscript(subscript: Option<&str>) -> bool {
    subscript.is_some()
}

/// Check if variable name matches array heuristics
fn matches_array_heuristics(var_name: &str) -> bool {
    var_name.ends_with('s')
        || var_name.contains("array")
        || var_name.contains("list")
        || var_name.contains("items")
}

/// Create diagnostic for bare array in conditional
fn create_array_diagnostic(
    var_name: &str,
    line_num: usize,
    start_col: usize,
    end_col: usize,
) -> Diagnostic {
    Diagnostic::new(
        "SC2199",
        Severity::Warning,
        format!(
            "Arrays implicitly concatenate in [[ ]]. Use ${{{}[@]}} to check all elements or ${{{}[0]}} for first element",
            var_name, var_name
        ),
        Span::new(line_num, start_col, line_num, end_col),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if is_comment_line(line) || !has_double_bracket(line) {
            continue;
        }

        // Extract [[ ]] blocks
        for bracket_match in DOUBLE_BRACKET.find_iter(line) {
            let bracket_text = bracket_match.as_str();

            // Look for variable references that might be arrays
            for cap in ARRAY_IN_CONDITIONAL.captures_iter(bracket_text) {
                let var_name = cap.get(1).unwrap().as_str();
                let subscript = cap.get(2).map(|m| m.as_str());

                // Skip if it has array subscripts
                if has_subscript(subscript) {
                    continue;
                }

                // Check if matches array heuristics
                if matches_array_heuristics(var_name) {
                    let start_col = line.find(bracket_text).unwrap_or(0) + 1;
                    let end_col = start_col + bracket_text.len();
                    let diagnostic =
                        create_array_diagnostic(var_name, line_num, start_col, end_col);

                    result.add(diagnostic);
                    break; // Only warn once per [[ ]] block
                }
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
    fn prop_sc2199_comments_never_diagnosed() {
        // Property: Comment lines should never produce diagnostics
        let test_cases = vec![
            "# [[ $items = \"test\" ]]",
            "  # [[ ${arrays} ]]",
            "\t# [[ -n $files ]]",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2199_single_bracket_never_diagnosed() {
        // Property: Single bracket [ ] should not be diagnosed
        let test_cases = vec!["[ $items = \"test\" ]", "[ -n $files ]", "[ ${arrays} ]"];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2199_subscripted_arrays_never_diagnosed() {
        // Property: Arrays with subscripts should not be diagnosed
        let test_cases = vec![
            "[[ ${items[@]} ]]",
            "[[ ${files[*]} ]]",
            "[[ ${items[0]} = \"first\" ]]",
            "[[ ${arrays[1]} ]]",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2199_bare_arrays_always_diagnosed() {
        // Property: Bare arrays in [[ ]] should always be diagnosed
        let test_cases = vec![
            ("[[ $items = \"test\" ]]", "items"),
            ("[[ ${files} ]]", "files"),
            ("[[ -n $arrays ]]", "arrays"),
            ("[[ $my_array ]]", "my_array"),
            ("[[ $data_list ]]", "data_list"),
        ];

        for (code, var_name) in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
            assert!(result.diagnostics[0].message.contains(var_name));
        }
    }

    #[test]
    fn prop_sc2199_singular_names_never_diagnosed() {
        // Property: Singular variable names should not be diagnosed
        let test_cases = vec!["[[ $item = \"test\" ]]", "[[ $file ]]", "[[ -n $path ]]"];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2199_one_diagnostic_per_bracket() {
        // Property: Only one diagnostic per [[ ]] block
        let code = "[[ $items && $files ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn prop_sc2199_diagnostic_code_always_sc2199() {
        // Property: All diagnostics must have code "SC2199"
        let code = "[[ $items ]] && [[ $files ]]";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(&diagnostic.code, "SC2199");
        }
    }

    #[test]
    fn prop_sc2199_diagnostic_severity_always_warning() {
        // Property: All diagnostics must be Warning severity
        let code = "[[ $items = \"test\" ]]";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Warning);
        }
    }

    #[test]
    fn prop_sc2199_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Original Unit Tests =====

    #[test]
    fn test_sc2199_array_in_conditional() {
        let code = r#"[[ $items = "test" ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2199");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("[@]"));
    }

    #[test]
    fn test_sc2199_array_braces() {
        let code = r#"[[ ${arrays} ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2199_array_check_empty() {
        let code = r#"[[ -n $files ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2199_with_at_ok() {
        let code = r#"[[ ${items[@]} ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2199_with_star_ok() {
        let code = r#"[[ ${items[*]} ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2199_with_index_ok() {
        let code = r#"[[ ${items[0]} = "first" ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2199_singular_var_ok() {
        let code = r#"[[ $item = "test" ]]"#;
        let result = check(code);
        // Singular variable name, probably not an array
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2199_array_name_patterns() {
        let code = r#"
[[ $my_array ]]
[[ $data_list ]]
[[ $all_items ]]
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 3);
    }

    #[test]
    fn test_sc2199_regular_test_ok() {
        let code = r#"[ $items = "test" ]"#;
        let result = check(code);
        // Single bracket, not [[ ]]
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2199_multiple_conditionals() {
        let code = r#"[[ $paths ]] && [[ $files ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
