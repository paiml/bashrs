// SC2198: Arrays don't work as scalars in comparisons. Use ${array[0]} or ${array[@]}
//
// When testing arrays with [ ], only the first element is checked. This is rarely
// the intended behavior. Use [[ ]] with ${array[@]} or explicitly access elements.
//
// Examples:
// Bad:
//   [ -n $array ]              # Only checks first element
//   [ $items = "test" ]        # Only compares first element
//
// Good:
//   [ -n "${array[0]}" ]       # Explicitly check first element
//   [[ -n ${array[@]} ]]       # Check if array has any elements
//   [ ${#array[@]} -gt 0 ]     # Check array length

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ARRAY_IN_TEST: Lazy<Regex> = Lazy::new(|| {
    // Match: $var, ${var}, ${var[...]}, etc.
    // Capture the variable name and optionally any subscript
    Regex::new(r"\$\{?([a-z_][a-z0-9_]*)(\[[^\]]*\])?\}?").unwrap()
});

static SINGLE_BRACKET: Lazy<Regex> = Lazy::new(|| {
    // Match [ ... ] (we'll manually skip [[ ... ]] in check logic)
    Regex::new(r"\[([^\]]+)\]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip double brackets
        if line.contains("[[") {
            continue;
        }

        // Only check lines with single [ ]
        if !line.contains('[') || line.contains("[[") {
            continue;
        }

        // Extract [ ] blocks
        for bracket_match in SINGLE_BRACKET.find_iter(line) {
            let bracket_text = bracket_match.as_str();

            // Skip [[ ]] (only want single brackets)
            if bracket_text.starts_with("[[") {
                continue;
            }

            // Look for array-like variable names
            for cap in ARRAY_IN_TEST.captures_iter(bracket_text) {
                let full_match = cap.get(0).unwrap().as_str();
                let var_name = cap.get(1).unwrap().as_str();
                let subscript = cap.get(2).map(|m| m.as_str());

                // Skip if it has array subscripts ([@], [*], [0], etc.) or is ${#array[@]} (length check)
                if subscript.is_some() || bracket_text.contains("#") {
                    continue;
                }

                // Heuristic: plural names or common array indicators
                if var_name.ends_with('s')
                    || var_name.contains("array")
                    || var_name.contains("list")
                    || var_name.contains("items")
                {
                    let start_col = line.find(bracket_text).unwrap_or(0) + 1;
                    let end_col = start_col + bracket_text.len();

                    let diagnostic = Diagnostic::new(
                        "SC2198",
                        Severity::Warning,
                        format!(
                            "Arrays don't work as scalars in [ ]. Use [ -n \"${{{}[0]}}\" ] for first element or [[ ]] with ${{{}[@]}}",
                            var_name, var_name
                        ),
                        Span::new(line_num, start_col, line_num, end_col),
                    );

                    result.add(diagnostic);
                    break; // Only warn once per [ ] block
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
    fn test_sc2198_array_in_test() {
        let code = r#"[ -n $items ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2198");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("Arrays"));
    }

    #[test]
    fn test_sc2198_array_comparison() {
        let code = r#"[ $files = "test" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2198_array_empty_check() {
        let code = r#"[ -z $paths ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2198_with_index_ok() {
        let code = r#"[ -n "${items[0]}" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2198_with_at_ok() {
        let code = r#"[ -n "${items[@]}" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2198_double_bracket_ok() {
        let code = r#"[[ -n $items ]]"#;
        let result = check(code);
        // Double brackets handle arrays better
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2198_singular_var_ok() {
        let code = r#"[ -n $item ]"#;
        let result = check(code);
        // Singular name, likely not an array
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2198_array_length_ok() {
        let code = r#"[ ${#items[@]} -gt 0 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2198_multiple_arrays() {
        let code = r#"
[ -n $files ] && [ -n $paths ]
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2198_array_name_patterns() {
        let code = r#"
[ $my_array ]
[ $data_list ]
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
