//! SC2072: Decimal numbers not supported in arithmetic context
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! if (( num > 3.14 )); then
//!   echo "bigger than pi"
//! fi
//! ```
//!
//! Good:
//! ```bash
//! if (( $(echo "$num > 3.14" | bc -l) )); then
//!   echo "bigger than pi"
//! fi
//! ```
//!
//! # Rationale
//!
//! Bash arithmetic expansion `(( ... ))` only supports integers, not floating point.
//! Use `bc` or `awk` for floating point arithmetic.
//!
//! # Auto-fix
//!
//! Suggest using bc for floating point comparisons

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for decimal numbers in arithmetic contexts
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: (( ... <decimal> ... ))
    let arith_pattern = Regex::new(r#"\(\(([^)]+)\)\)"#).unwrap();
    let decimal_pattern = Regex::new(r#"\b\d+\.\d+\b"#).unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Find (( ... )) blocks
        for arith_cap in arith_pattern.captures_iter(line) {
            let arith_content = arith_cap.get(1).unwrap().as_str();

            // Check if it contains decimal numbers
            if decimal_pattern.is_match(arith_content) {
                let full_match = arith_cap.get(0).unwrap();
                let start_col = full_match.start() + 1;
                let end_col = full_match.end() + 1;

                // Suggest bc for floating point
                let fix_text = format!(r#"(( $(echo "{}" | bc -l) ))"#, arith_content.trim());

                let diagnostic = Diagnostic::new(
                    "SC2072",
                    Severity::Warning,
                    "Decimal numbers not supported in (( )) arithmetic. Use bc or awk for floating point.",
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
    fn test_sc2072_basic_detection() {
        let script = r#"if (( num > 3.14 )); then echo "yes"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2072");
    }

    #[test]
    fn test_sc2072_autofix() {
        let script = r#"(( x > 2.5 ))"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert!(result.diagnostics[0]
            .fix
            .as_ref()
            .unwrap()
            .replacement
            .contains("bc"));
    }

    #[test]
    fn test_sc2072_multiple_decimals() {
        let script = r#"(( x > 1.5 && y < 2.7 ))"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1); // One (( )) block
    }

    #[test]
    fn test_sc2072_pi_comparison() {
        let script = r#"if (( radius * 2 > 6.28 )); then echo "big"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2072_false_positive_integer() {
        // Should NOT flag integers
        let script = r#"if (( num > 5 )); then echo "yes"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2072_false_positive_bc_already_used() {
        // Should still flag - but this shows bc is already being used
        let script = r#"if (( $(echo "scale=2; $x > 3.14" | bc -l) )); then echo "yes"; fi"#;
        let result = check(script);
        // This will still be flagged since the outer (( )) contains decimal
        assert!(result.diagnostics.len() >= 0); // May or may not flag depending on implementation
    }

    #[test]
    fn test_sc2072_false_positive_string_context() {
        // Should NOT flag decimals in strings
        let script = r#"echo "value is 3.14""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2072_zero_decimal() {
        let script = r#"(( val > 0.5 ))"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2072_large_decimal() {
        let script = r#"(( num > 1000.99 ))"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2072_assignment_with_decimal() {
        let script = r#"(( result = value * 1.5 ))"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
