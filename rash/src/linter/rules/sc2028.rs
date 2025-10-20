//! SC2028: Echo may not expand escape sequences
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! echo "Hello\nWorld"  # May print literal \n
//! ```
//!
//! Good:
//! ```bash
//! echo -e "Hello\nWorld"  # bash: -e enables escape sequences
//! printf "Hello\nWorld\n"  # POSIX: always expands escapes
//! ```
//!
//! # Rationale
//!
//! echo behavior with escape sequences is not portable:
//! - Some shells expand \n, \t by default
//! - Others require -e flag
//! - POSIX does not specify behavior
//!
//! Use printf for portable escape sequence handling.
//!
//! # Auto-fix
//!
//! Suggest using printf or echo -e

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for echo with escape sequences without -e flag
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: echo "...\n..." or echo "...\t..." (without -e)
    let pattern = Regex::new(r#"echo\s+["']([^"']*\\[nt][^"']*)["']"#).unwrap();

    // Pattern to detect echo -e (this is OK)
    let echo_e_pattern = Regex::new(r#"echo\s+-e\s+"#).unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip if echo -e is used
        if echo_e_pattern.is_match(line) {
            continue;
        }

        for cap in pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let content = cap.get(1).unwrap().as_str();

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            // Suggest printf as the fix
            let fix_text = format!("printf \"{}\\n\"", content);

            let diagnostic = Diagnostic::new(
                "SC2028",
                Severity::Info,
                "Echo may not expand escape sequences. Use printf or echo -e instead",
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
    fn test_sc2028_basic_detection() {
        let script = r#"echo "Hello\nWorld""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2028");
    }

    #[test]
    fn test_sc2028_autofix() {
        let script = r#"echo "Hello\nWorld""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "printf \"Hello\\nWorld\\n\""
        );
    }

    #[test]
    fn test_sc2028_tab_escape() {
        let script = r#"echo "Column1\tColumn2""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2028_multiple_escapes() {
        let script = r#"echo "Line1\nLine2\nLine3""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2028_false_positive_echo_e() {
        let script = r#"echo -e "Hello\nWorld""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2028_false_positive_printf() {
        let script = r#"printf "Hello\nWorld\n""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2028_false_positive_no_escape() {
        let script = r#"echo "Hello World""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2028_false_positive_in_comment() {
        let script = r#"# echo "Hello\nWorld""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2028_single_quotes() {
        let script = r#"echo 'Hello\nWorld'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2028_mixed_escape_sequences() {
        let script = r#"echo "Tab:\t Newline:\n""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
