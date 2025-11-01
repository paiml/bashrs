//! SC2128: Expanding an array without an index only gives the first element
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! array=(a b c)
//! echo "$array"  # Only prints 'a'
//! ```
//!
//! Good:
//! ```bash
//! array=(a b c)
//! echo "${array[@]}"  # Prints all elements
//! echo "${array[0]}"  # Explicitly get first element
//! ```
//!
//! # Rationale
//!
//! Referencing an array without an index:
//! - Only expands the first element
//! - Misleading behavior
//! - Usually a bug
//!
//! Use [@] or [*] to reference all elements, or explicit index.
//!
//! # Auto-fix
//!
//! Suggest adding [@] to expand all elements

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check if a line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if a line contains an array declaration
fn is_array_declaration(line: &str) -> bool {
    line.contains("=(")
}

/// Check if variable already has an index (like $var[0])
fn has_index(line: &str, end_pos: usize) -> bool {
    end_pos < line.len() && line.chars().nth(end_pos) == Some('[')
}

/// Check if variable already uses [@] or [*]
fn has_array_expansion(text: &str) -> bool {
    text.contains("[@]") || text.contains("[*]")
}

/// Check if variable name matches array heuristics
fn matches_array_heuristics(var_name: &str) -> bool {
    var_name.ends_with("s") || var_name.contains("array") || var_name.contains("list")
}

/// Create diagnostic for array without index
fn create_array_diagnostic(
    var_name: &str,
    line_num: usize,
    start_col: usize,
    end_col: usize,
) -> Diagnostic {
    let fix_text = format!("${{{}[@]}}", var_name);

    Diagnostic::new(
        "SC2128",
        Severity::Warning,
        "Expanding an array without an index only gives the first element",
        Span::new(line_num, start_col, line_num, end_col),
    )
    .with_fix(Fix::new(fix_text))
}

/// Check for array reference without index
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: $array or "$array" (without [@] or [*] or [n])
    let pattern = Regex::new(r#"\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?"#).unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if is_comment_line(line) || is_array_declaration(line) {
            continue;
        }

        for cap in pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let var_name = cap.get(1).unwrap().as_str();

            // Skip if already has index or array expansion
            if has_index(line, full_match.end()) || has_array_expansion(full_match.as_str()) {
                continue;
            }

            // Check if matches array heuristics
            if matches_array_heuristics(var_name) {
                let start_col = full_match.start() + 1;
                let end_col = full_match.end() + 1;
                let diagnostic = create_array_diagnostic(var_name, line_num, start_col, end_col);
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
    fn prop_sc2128_comments_never_diagnosed() {
        // Property: Comment lines should never produce diagnostics
        let test_cases = vec![
            "# args=(a b)\n# echo $args",
            "  # files=(*.txt)\n  # cat $files",
            "\t# array=(1 2 3)",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2128_indexed_access_never_diagnosed() {
        // Property: Array access with [@], [*], or [n] should not be diagnosed
        let test_cases = vec![
            "args=(a b c)\necho \"${args[@]}\"",
            "files=(*.txt)\ncat \"${files[*]}\"",
            "items=(x y z)\necho \"${items[0]}\"",
            "array=(1 2)\necho \"${array[1]}\"",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2128_array_heuristics_always_diagnosed() {
        // Property: Variables matching array heuristics should be diagnosed
        let test_cases = vec![
            ("args=(a b)\necho $args", "args"),             // ends with 's'
            ("files=(*.txt)\ncat $files", "files"),         // ends with 's'
            ("items=(x y)\nprintf $items", "items"),        // ends with 's'
            ("myarray=(1 2)\necho $myarray", "myarray"),    // contains 'array'
            ("datalist=(a b)\necho $datalist", "datalist"), // contains 'list'
        ];

        for (code, var_name) in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
            assert!(result.diagnostics[0]
                .fix
                .as_ref()
                .unwrap()
                .replacement
                .contains(var_name));
        }
    }

    #[test]
    fn prop_sc2128_declarations_skipped() {
        // Property: Array declarations should not be diagnosed
        let test_cases = vec![
            "args=(a b c)",
            "files=(*.txt)",
            "items=(x y z)",
            "array=(1 2 3 4)",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2128_all_diagnostics_have_fix() {
        // Property: All SC2128 diagnostics must provide a fix
        let code = "args=(a b)\nfiles=(*.txt)\necho $args $files";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert!(
                diagnostic.fix.is_some(),
                "All SC2128 diagnostics should have a fix"
            );
        }
    }

    #[test]
    fn prop_sc2128_diagnostic_code_always_sc2128() {
        // Property: All diagnostics must have code "SC2128"
        let code = "args=(a b)\nfiles=(*.txt)\necho $args $files";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(&diagnostic.code, "SC2128");
        }
    }

    #[test]
    fn prop_sc2128_diagnostic_severity_always_warning() {
        // Property: All diagnostics must be Warning severity
        let code = "items=(x y z)\necho $items";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Warning);
        }
    }

    #[test]
    fn prop_sc2128_fix_format_correct() {
        // Property: Auto-fix should always suggest ${var[@]}
        let code = "args=(a b c)\necho $args";
        let result = check(code);

        assert_eq!(result.diagnostics.len(), 1);
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, "${args[@]}");
    }

    #[test]
    fn prop_sc2128_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Original Unit Tests =====

    #[test]
    fn test_sc2128_basic_detection() {
        let script = "args=(a b c)\necho \"$args\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2128");
    }

    #[test]
    fn test_sc2128_autofix() {
        let script = "files=(*.txt)\ncat $files";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "${files[@]}"
        );
    }

    #[test]
    fn test_sc2128_with_braces() {
        let script = "items=(x y z)\necho \"${items}\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2128_array_suffix() {
        let script = "array=(1 2 3)\nprintf '%s' \"$array\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2128_false_positive_with_at() {
        let script = "args=(a b c)\necho \"${args[@]}\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2128_false_positive_with_star() {
        let script = "files=(*.txt)\necho \"${files[*]}\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2128_false_positive_with_index() {
        let script = "items=(a b c)\necho \"${items[0]}\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2128_false_positive_in_comment() {
        let script = "# echo \"$args\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2128_multiple_vars() {
        let script = "args=(a b)\nfiles=(*.txt)\necho \"$args $files\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2128_in_loop() {
        let script = "files=(*.txt)\nfor f in $files; do echo $f; done";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1); // Only $files in 'for' line
    }
}
