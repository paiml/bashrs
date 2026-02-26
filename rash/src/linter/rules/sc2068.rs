//! SC2068: Double quote array expansions to prevent globbing and word splitting.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! for i in $@; do
//!   echo "$i"
//! done
//! ```
//!
//! Good:
//! ```bash
//! for i in "$@"; do
//!   echo "$i"
//! done
//! ```
//!
//! # Rationale
//!
//! Unquoted `$@`, `$*`, and `${array[@]}` expansions are subject to word splitting
//! and globbing. This can cause unexpected behavior when arguments contain spaces,
//! special characters, or glob patterns.
//!
//! # Auto-fix
//!
//! Wrap the expansion in double quotes: `$@` → `"$@"`

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for unquoted array expansions ($@, $*, ${array[@]}, ${array[*]})
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Regex patterns for unquoted array expansions
    // Pattern 1: Unquoted $@ or $*
    let simple_pattern = Regex::new(r"\$[@*]").unwrap();

    // Pattern 2: Unquoted ${array[@]} or ${array[*]}
    let array_pattern = Regex::new(r"\$\{[a-zA-Z_][a-zA-Z0-9_]*\[[@*]\]\}").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1; // 1-indexed

        // Skip comments
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for simple $@ and $*
        for mat in simple_pattern.find_iter(line) {
            let start_col = mat.start() + 1; // 1-indexed
            let end_col = mat.end() + 1;

            // Check if inside quotes
            if is_inside_quotes(line, mat.start()) {
                continue;
            }

            let matched_text = mat.as_str();
            let fix_text = format!(r#""{}""#, matched_text);

            let diagnostic = Diagnostic::new(
                "SC2068",
                Severity::Warning,
                "Double quote to prevent globbing and word splitting on $@/$*",
                Span::new(line_num, start_col, line_num, end_col),
            )
            .with_fix(Fix::new(fix_text));

            result.add(diagnostic);
        }

        // Check for ${array[@]} and ${array[*]}
        for mat in array_pattern.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            // Check if inside quotes
            if is_inside_quotes(line, mat.start()) {
                continue;
            }

            let matched_text = mat.as_str();
            let fix_text = format!(r#""{}""#, matched_text);

            let diagnostic = Diagnostic::new(
                "SC2068",
                Severity::Warning,
                "Double quote to prevent globbing and word splitting on array expansion",
                Span::new(line_num, start_col, line_num, end_col),
            )
            .with_fix(Fix::new(fix_text));

            result.add(diagnostic);
        }
    }

    result
}

/// Check if a position in a line is inside quotes
fn is_inside_quotes(line: &str, pos: usize) -> bool {
    let before = &line[..pos];

    // Simple quote counting approach
    let double_quotes = before.matches('"').count();
    let single_quotes = before.matches('\'').count();

    // If odd number of quotes, we're inside quotes
    double_quotes % 2 == 1 || single_quotes % 2 == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2068_basic_detection() {
        let script = r#"
for i in $@; do
  echo "$i"
done
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2068");
        assert!(result.diagnostics[0].message.contains("Double quote"));
    }

    #[test]
    fn test_sc2068_star_detection() {
        let script = r#"command $*"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2068");
    }

    #[test]
    fn test_sc2068_array_expansion() {
        let script = r#"
for i in ${array[@]}; do
  echo "$i"
done
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2068");
        assert!(result.diagnostics[0].message.contains("array expansion"));
    }

    #[test]
    fn test_sc2068_array_star() {
        let script = r#"command ${myarray[*]}"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2068");
    }

    #[test]
    fn test_sc2068_autofix() {
        let script = r#"for i in $@; do echo "$i"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            r#""$@""#
        );
    }

    #[test]
    fn test_sc2068_false_positive_quoted() {
        // Should NOT flag "$@" (already quoted)
        let script = r#"
for i in "$@"; do
  echo "$i"
done
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2068_false_positive_single_quoted() {
        // Should NOT flag '$@' (inside single quotes - literal)
        let script = r#"echo '$@'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2068_multiple_occurrences() {
        let script = r#"
command1 $@
command2 $*
command3 ${arr[@]}
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 3);
    }

    #[test]
    fn test_sc2068_edge_case_beginning_of_line() {
        let script = r#"$@ is the arguments"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2068_edge_case_end_of_line() {
        let script = r#"command $@"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
