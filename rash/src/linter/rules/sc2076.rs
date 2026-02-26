//! SC2076: Don't quote right-hand side of =~ (regex matching)
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! if [[ "$var" =~ "^[0-9]+$" ]]; then
//!   echo "Numeric"
//! fi
//! ```
//!
//! Good:
//! ```bash
//! if [[ "$var" =~ ^[0-9]+$ ]]; then
//!   echo "Numeric"
//! fi
//! ```
//!
//! # Rationale
//!
//! In `[[ ... =~ ... ]]` expressions, quoting the right-hand side (regex pattern)
//! causes it to be treated as a literal string match instead of a regex pattern.
//! This breaks regex functionality.
//!
//! # Auto-fix
//!
//! Remove quotes from the regex pattern: `"^[0-9]+$"` → `^[0-9]+$`

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for quoted regex patterns in =~ comparisons
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: [[ ... =~ "..." ]]
    // We need to match the =~ operator followed by a quoted string
    // Use non-greedy matching to properly handle ]]
    let bracket_pattern = Regex::new(r"\[\[(.*?)\]\]").unwrap();
    let regex_match_pattern = Regex::new(r#"=~\s+"([^"]+)""#).unwrap();

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

            // Find =~ "..." patterns within [[ ... ]]
            for regex_cap in regex_match_pattern.captures_iter(bracket_content) {
                let full_match = regex_cap.get(0).unwrap();
                let quoted_regex = regex_cap.get(1).unwrap().as_str();

                let match_offset = full_match.start();
                let match_end = full_match.end();

                // Calculate absolute position in line
                let abs_start = bracket_start + 2 + match_offset; // +2 for [[
                let abs_end = bracket_start + 2 + match_end;

                let start_col = abs_start + 1; // 1-indexed
                let end_col = abs_end + 1;

                // Auto-fix: remove quotes from regex
                let fix_text = format!("=~ {}", quoted_regex);

                let diagnostic = Diagnostic::new(
                    "SC2076",
                    Severity::Warning,
                    "Don't quote right-hand side of =~ (regex will be treated as literal string)",
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
    fn test_sc2076_basic_detection() {
        let script = r#"if [[ "$var" =~ "^[0-9]+$" ]]; then echo "numeric"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2076");
        assert!(result.diagnostics[0].message.contains("Don't quote"));
    }

    #[test]
    fn test_sc2076_autofix() {
        let script = r#"if [[ "$var" =~ "^[0-9]+$" ]]; then echo "numeric"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            r#"=~ ^[0-9]+$"#
        );
    }

    #[test]
    fn test_sc2076_email_regex() {
        // Properly quote the regex pattern with escaped quotes
        let script = r#"[[ "$email" =~ "^[a-z]+@[a-z]+\.[a-z]+$" ]]"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2076_word_boundary() {
        let script = r#"[[ "$text" =~ "word" ]]"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2076_false_positive_unquoted() {
        // Should NOT flag unquoted regex (correct usage)
        let script = r#"if [[ "$var" =~ ^[0-9]+$ ]]; then echo "numeric"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2076_false_positive_single_bracket() {
        // Should NOT flag [ ... ] (not [[ ... ]])
        let script = r#"if [ "$var" = "test" ]; then echo "yes"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2076_false_positive_no_regex_operator() {
        // Should NOT flag == operator (not =~)
        let script = r#"if [[ "$var" == "pattern" ]]; then echo "yes"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2076_multiple_occurrences() {
        let script = r#"
if [[ "$var1" =~ "^[0-9]+$" ]]; then
    echo "numeric"
fi
if [[ "$var2" =~ "[a-z]+" ]]; then
    echo "lowercase"
fi
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2076_complex_regex() {
        let script = r#"[[ "$path" =~ "^/[a-zA-Z0-9/_-]+$" ]]"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2076_with_spaces() {
        let script = r#"[[ "$var" =~  "pattern" ]]"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
