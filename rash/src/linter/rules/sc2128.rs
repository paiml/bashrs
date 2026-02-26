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
use std::collections::HashSet;

/// Regex to detect array declarations: var=(...)
static ARRAY_DECL: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"([A-Za-z_][A-Za-z0-9_]*)\s*=\s*\(").unwrap());

/// Regex to find variable references
static VAR_REF: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?").unwrap());

/// Check if a line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if variable already has an index (like $var[0])
fn has_index(line: &str, end_pos: usize) -> bool {
    end_pos < line.len() && line.chars().nth(end_pos) == Some('[')
}

/// Check if variable already uses [@] or [*]
fn has_array_expansion(text: &str) -> bool {
    text.contains("[@]") || text.contains("[*]")
}

/// Extract array variable names from declarations on a line
fn extract_array_declarations(line: &str) -> Vec<String> {
    ARRAY_DECL
        .captures_iter(line)
        .map(|cap| cap.get(1).unwrap().as_str().to_string())
        .collect()
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
///
/// This implementation tracks actual array declarations rather than using
/// heuristics (like variable names ending in 's'), which caused false positives
/// for scalar variables like `cpu_tps`, `status`, etc.
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let mut known_arrays: HashSet<String> = HashSet::new();

    // First pass: collect all array declarations
    for line in source.lines() {
        if is_comment_line(line) {
            continue;
        }
        known_arrays.extend(extract_array_declarations(line));
    }

    // Second pass: check for unindexed array references
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if is_comment_line(line) {
            continue;
        }

        for cap in VAR_REF.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let var_name = cap.get(1).unwrap().as_str();

            // Only flag variables that are ACTUALLY declared as arrays
            if !known_arrays.contains(var_name) {
                continue;
            }

            // Skip if already has index or array expansion
            if has_index(line, full_match.end()) || has_array_expansion(full_match.as_str()) {
                continue;
            }

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;
            let diagnostic = create_array_diagnostic(var_name, line_num, start_col, end_col);
            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Issue #132: False Positive Tests =====

    #[test]
    fn test_sc2128_issue_132_scalar_ending_in_s_not_flagged() {
        // Issue #132: Variables like cpu_tps, gpu_tps are scalars, not arrays
        // The old heuristic `ends_with("s")` caused false positives
        let script = r#"
cpu_tps=$(measure_throughput "echo '200.5 tok/s'")
echo "$cpu_tps"
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Scalar variable ending in 's' should NOT be flagged as array"
        );
    }

    #[test]
    fn test_sc2128_issue_132_status_variable_not_flagged() {
        let script = r#"
status="success"
echo "$status"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2128_issue_132_formats_scalar_not_flagged() {
        // Variable named 'formats' but assigned as scalar
        let script = r#"
formats="json"
echo "$formats"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2128_issue_132_formats_array_is_flagged() {
        // Variable named 'formats' AND declared as array - SHOULD be flagged
        let script = r#"
formats=("json" "xml")
echo "$formats"
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Array variable should be flagged when used without index"
        );
    }

    // ===== Property Tests =====

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
    fn prop_sc2128_declared_arrays_diagnosed() {
        // Property: Variables declared as arrays should be diagnosed when used without index
        let test_cases = vec![
            ("args=(a b)\necho $args", "args"),
            ("files=(*.txt)\ncat $files", "files"),
            ("items=(x y)\nprintf $items", "items"),
            ("myarray=(1 2)\necho $myarray", "myarray"),
            ("datalist=(a b)\necho $datalist", "datalist"),
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
    fn prop_sc2128_non_arrays_not_diagnosed() {
        // Property: Variables NOT declared as arrays should NEVER be diagnosed
        let test_cases = vec![
            "args=\"hello\"\necho $args",     // scalar ending in 's'
            "files=\"test.txt\"\ncat $files", // scalar ending in 's'
            "status=0\necho $status",         // scalar ending in 's'
            "tps=100\necho $tps",             // scalar ending in 's'
            "mylist=\"item\"\necho $mylist",  // contains 'list' but scalar
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "Should NOT diagnose scalar: {}",
                code
            );
        }
    }

    #[test]
    fn prop_sc2128_declarations_skipped() {
        // Property: Array declarations themselves should not be diagnosed
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
