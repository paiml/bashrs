//! SC2071: Use arithmetic comparison, not string comparison
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! if [ "$num" > 5 ]; then
//!   echo "greater"
//! fi
//! ```
//!
//! Good:
//! ```bash
//! if [ "$num" -gt 5 ]; then
//!   echo "greater"
//! fi
//! ```
//!
//! # Rationale
//!
//! Using `>` or `<` in `[ ... ]` performs lexicographic (string) comparison,
//! not numeric comparison. Use `-gt`, `-lt`, `-ge`, `-le`, `-eq`, `-ne` for numbers.
//!
//! # Auto-fix
//!
//! Replace `>` with `-gt`, `<` with `-lt`

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for string comparison operators used on numbers
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: [ ... > ... ] or [ ... < ... ] (NOT [[ ... ]])
    // Check for [[ first to skip those lines
    let gt_pattern = Regex::new(r#"\[\s+"?\$[A-Za-z_][A-Za-z0-9_]*"?\s+>\s+[0-9]+"#).unwrap();
    let lt_pattern = Regex::new(r#"\[\s+"?\$[A-Za-z_][A-Za-z0-9_]*"?\s+<\s+[0-9]+"#).unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip lines with [[ (double bracket)
        if line.contains("[[") {
            continue;
        }

        // Skip lines with (( (arithmetic)
        if line.contains("((") {
            continue;
        }

        // Check for > operator
        for mat in gt_pattern.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;
            let fix_text = mat.as_str().replace(">", "-gt");

            let diagnostic = Diagnostic::new(
                "SC2071",
                Severity::Warning,
                "Use -gt for numeric comparison (> is lexicographic)",
                Span::new(line_num, start_col, line_num, end_col),
            )
            .with_fix(Fix::new(fix_text));

            result.add(diagnostic);
        }

        // Check for < operator
        for mat in lt_pattern.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;
            let fix_text = mat.as_str().replace("<", "-lt");

            let diagnostic = Diagnostic::new(
                "SC2071",
                Severity::Warning,
                "Use -lt for numeric comparison (< is lexicographic)",
                Span::new(line_num, start_col, line_num, end_col),
            )
            .with_fix(Fix::new(fix_text));

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2071_greater_than() {
        let script = r#"if [ "$num" > 5 ]; then echo "big"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2071");
    }

    #[test]
    fn test_sc2071_less_than() {
        let script = r#"if [ "$num" < 5 ]; then echo "small"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2071_autofix_gt() {
        let script = r#"[ "$num" > 5 ]"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert!(result.diagnostics[0]
            .fix
            .as_ref()
            .unwrap()
            .replacement
            .contains("-gt"));
    }

    #[test]
    fn test_sc2071_autofix_lt() {
        let script = r#"[ "$num" < 5 ]"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert!(result.diagnostics[0]
            .fix
            .as_ref()
            .unwrap()
            .replacement
            .contains("-lt"));
    }

    #[test]
    fn test_sc2071_false_positive_correct_usage() {
        let script = r#"if [ "$num" -gt 5 ]; then echo "big"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2071_false_positive_double_bracket() {
        let script = r#"if [[ "$num" > 5 ]]; then echo "big"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2071_false_positive_arithmetic() {
        let script = r#"if (( num > 5 )); then echo "big"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2071_multiple_comparisons() {
        let script = r#"
[ "$a" > 10 ]
[ "$b" < 20 ]
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2071_quoted_variable() {
        let script = r#"[ "$num" > 100 ]"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2071_unquoted_variable() {
        let script = r#"[ $num > 10 ]"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
