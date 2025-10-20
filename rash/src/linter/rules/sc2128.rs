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

/// Check for array reference without index
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: $array or "$array" (without [@] or [*] or [n])
    let pattern = Regex::new(r#"\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?"#).unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip if line contains array declaration
        if line.contains("=(") {
            continue;
        }

        for cap in pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let var_name = cap.get(1).unwrap().as_str();

            // Skip if already has index
            let end_pos = full_match.end();
            if end_pos < line.len() && line.chars().nth(end_pos) == Some('[') {
                continue;
            }

            // Skip if already uses braces with [@] or [*]
            if full_match.as_str().contains("[@]") || full_match.as_str().contains("[*]") {
                continue;
            }

            // Heuristic: Common array variable names
            if var_name.ends_with("s") || var_name.contains("array") || var_name.contains("list") {
                let start_col = full_match.start() + 1;
                let end_col = full_match.end() + 1;

                let fix_text = format!("${{{}[@]}}", var_name);

                let diagnostic = Diagnostic::new(
                    "SC2128",
                    Severity::Warning,
                    "Expanding an array without an index only gives the first element",
                    Span::new(line_num, start_col, line_num, end_col),
                )
                .with_fix(Fix::new(fix_text));

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
