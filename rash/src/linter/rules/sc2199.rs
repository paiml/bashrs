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

static DOUBLE_BRACKET: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\[\[.*?\]\]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Only check lines with [[ ]]
        if !line.contains("[[") {
            continue;
        }

        // Extract [[ ]] blocks
        for bracket_match in DOUBLE_BRACKET.find_iter(line) {
            let bracket_text = bracket_match.as_str();

            // Look for variable references that might be arrays
            for cap in ARRAY_IN_CONDITIONAL.captures_iter(bracket_text) {
                let full_match = cap.get(0).unwrap().as_str();
                let var_name = cap.get(1).unwrap().as_str();
                let subscript = cap.get(2).map(|m| m.as_str());

                // Skip if it has array subscripts ([@], [*], [0], etc.)
                if subscript.is_some() {
                    continue;
                }

                // Heuristic: lowercase variables starting with common array names or plurals
                // are likely arrays (arrays, items, files, paths, etc.)
                if var_name.ends_with('s') ||
                   var_name.contains("array") ||
                   var_name.contains("list") ||
                   var_name.contains("items") {

                    let start_col = line.find(bracket_text).unwrap_or(0) + 1;
                    let end_col = start_col + bracket_text.len();

                    let diagnostic = Diagnostic::new(
                        "SC2199",
                        Severity::Warning,
                        format!(
                            "Arrays implicitly concatenate in [[ ]]. Use ${{{}[@]}} to check all elements or ${{{}[0]}} for first element",
                            var_name, var_name
                        ),
                        Span::new(line_num, start_col, line_num, end_col),
                    );

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
