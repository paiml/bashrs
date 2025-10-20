//! SC2066: Quote variables in [[ ... ]] to prevent globbing and word splitting.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! if [[ $var == *.txt ]]; then
//!   echo "Match!"
//! fi
//! ```
//!
//! Good:
//! ```bash
//! if [[ "$var" == *.txt ]]; then
//!   echo "Match!"
//! fi
//! ```
//!
//! # Rationale
//!
//! In `[[ ... ]]` conditionals, unquoted variables can be subject to glob expansion
//! and word splitting. While `[[` is safer than `[`, it's still best practice to
//! quote variables to be explicit about intent and prevent unexpected behavior.
//!
//! Note: Pattern matching on the RIGHT side of `==` should NOT be quoted
//! (e.g., `[[ "$var" == *.txt ]]` is correct).
//!
//! # Auto-fix
//!
//! Wrap the variable in double quotes: `$var` → `"$var"`

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for unquoted variables in [[ ... ]] conditionals
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: [[ ... ]] with unquoted variables
    // We look for $VAR or ${VAR} inside [[ ... ]]
    let bracket_pattern = Regex::new(r#"\[\[([^\]]+)\]\]"#).unwrap();
    let var_pattern =
        Regex::new(r#"\$(?:\{([A-Za-z_][A-Za-z0-9_]*)\}|([A-Za-z_][A-Za-z0-9_]*))"#).unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1; // 1-indexed

        // Skip comments
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Find [[ ... ]] blocks
        for bracket_match in bracket_pattern.captures_iter(line) {
            let bracket_content = bracket_match.get(1).unwrap().as_str();
            let bracket_start = bracket_match.get(0).unwrap().start();

            // Find variables within [[ ... ]]
            for var_match in var_pattern.captures_iter(bracket_content) {
                let var_offset = var_match.get(0).unwrap().start();
                let var_end = var_match.get(0).unwrap().end();
                let var_text = var_match.get(0).unwrap().as_str();

                // Calculate absolute position in line
                let abs_start = bracket_start + 2 + var_offset; // +2 for [[
                let abs_end = bracket_start + 2 + var_end;

                // Check if already quoted
                if is_quoted_at_position(line, abs_start) {
                    continue;
                }

                // Check if this is on the right side of == or != (pattern position)
                // In that case, don't flag it
                if is_pattern_position(bracket_content, var_offset) {
                    continue;
                }

                let start_col = abs_start + 1; // 1-indexed
                let end_col = abs_end + 1;
                let fix_text = format!(r#""{}""#, var_text);

                let diagnostic = Diagnostic::new(
                    "SC2066",
                    Severity::Warning,
                    "Quote variable in [[ ... ]] to prevent globbing and word splitting",
                    Span::new(line_num, start_col, line_num, end_col),
                )
                .with_fix(Fix::new(fix_text));

                result.add(diagnostic);
            }
        }
    }

    result
}

/// Check if a variable is already quoted at a position
fn is_quoted_at_position(line: &str, pos: usize) -> bool {
    if pos == 0 || pos >= line.len() {
        return false;
    }

    // Check if there's a quote immediately before
    let before_char = line.chars().nth(pos.saturating_sub(1));
    if matches!(before_char, Some('"') | Some('\'')) {
        return true;
    }

    // Simple quote counting
    let before = &line[..pos];
    let double_quotes = before.matches('"').count();
    let single_quotes = before.matches('\'').count();

    double_quotes % 2 == 1 || single_quotes % 2 == 1
}

/// Check if variable is on the right side of == or != (pattern position)
fn is_pattern_position(content: &str, var_pos: usize) -> bool {
    // Split by && and || to handle multiple conditions
    // Find which condition part this variable is in
    let before = &content[..var_pos];

    // Find the start of the current condition (after last && or ||)
    let condition_start = before
        .rfind("&&")
        .or_else(|| before.rfind("||"))
        .map(|pos| pos + 2)
        .unwrap_or(0);

    let current_condition = &content[condition_start..var_pos];

    // Look for the last comparison operator in the current condition
    if let Some(eq_pos) = current_condition.rfind("==") {
        // Variable is after ==, so it's in pattern position
        return var_pos - condition_start > eq_pos + 2;
    }

    if let Some(neq_pos) = current_condition.rfind("!=") {
        // Variable is after !=, so it's in pattern position
        return var_pos - condition_start > neq_pos + 2;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2066_basic_detection() {
        let script = r#"if [[ $var == value ]]; then echo "yes"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2066");
        assert!(result.diagnostics[0].message.contains("Quote variable"));
    }

    #[test]
    fn test_sc2066_braced_variable() {
        let script = r#"if [[ ${myvar} == test ]]; then echo "yes"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2066_multiple_variables() {
        let script = r#"if [[ $var1 == $var2 ]]; then echo "yes"; fi"#;
        let result = check(script);
        // Only $var1 should be flagged ($var2 is pattern position)
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2066_autofix() {
        let script = r#"if [[ $var == value ]]; then echo "yes"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            r#""$var""#
        );
    }

    #[test]
    fn test_sc2066_false_positive_quoted() {
        // Should NOT flag "$var" (already quoted)
        let script = r#"if [[ "$var" == value ]]; then echo "yes"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2066_false_positive_pattern() {
        // Should NOT flag pattern on right side of ==
        let script = r#"if [[ "test" == $pattern ]]; then echo "yes"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2066_glob_pattern() {
        // Variable on left should be quoted, glob pattern on right should not
        let script = r#"if [[ $var == *.txt ]]; then echo "yes"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2066_not_equal() {
        let script = r#"if [[ $var != value ]]; then echo "yes"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2066_and_condition() {
        let script = r#"if [[ $var1 == test && $var2 == test ]]; then echo "yes"; fi"#;
        let result = check(script);
        // Both $var1 and $var2 are on left side, should both be flagged
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2066_no_false_positive_single_bracket() {
        // Should NOT flag [ ... ] (that's handled by other rules)
        let script = r#"if [ $var == value ]; then echo "yes"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0); // SC2066 only looks for [[ ]]
    }
}
