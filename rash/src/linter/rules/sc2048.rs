//! SC2048: Use "$@" (with quotes) to prevent word splitting on expansions.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! command $*
//! myfunction $*
//! ```
//!
//! Good:
//! ```bash
//! command "$@"
//! myfunction "$@"
//! ```
//!
//! # Rationale
//!
//! `$*` expands to all positional parameters as a single word (joined by IFS),
//! while `"$@"` expands to separate quoted arguments. Using unquoted `$*` causes:
//! - Word splitting on IFS characters
//! - Glob expansion
//! - Loss of original argument boundaries
//!
//! `"$@"` preserves the original arguments exactly, which is almost always what you want.
//!
//! # Auto-fix
//!
//! Replace `$*` with `"$@"` in command contexts

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for unquoted $* that should be "$@"
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: unquoted $*
    let pattern = Regex::new(r"\$\*").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1; // 1-indexed

        // Skip comments
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Find all occurrences of $*
        for mat in pattern.find_iter(line) {
            let start_col = mat.start() + 1; // 1-indexed
            let end_col = mat.end() + 1;

            // Check if inside quotes
            if is_inside_quotes(line, mat.start()) {
                continue;
            }

            // Auto-fix: replace $* with "$@"
            let fix_text = r#""$@""#.to_string();

            let diagnostic = Diagnostic::new(
                "SC2048",
                Severity::Warning,
                r#"Use "$@" (with quotes) to prevent word splitting and preserve arguments"#,
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
    fn test_sc2048_basic_detection() {
        let script = r#"command $*"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2048");
        assert!(result.diagnostics[0].message.contains(r#""$@""#));
    }

    #[test]
    fn test_sc2048_function_call() {
        let script = r#"myfunction $*"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2048");
    }

    #[test]
    fn test_sc2048_for_loop() {
        let script = r#"for arg in $*; do echo "$arg"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2048_autofix() {
        let script = r#"command $*"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            r#""$@""#
        );
    }

    #[test]
    fn test_sc2048_false_positive_quoted() {
        // Should NOT flag "$*" (already quoted, though not recommended)
        let script = r#"command "$*""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2048_false_positive_single_quoted() {
        // Should NOT flag '$*' (inside single quotes - literal)
        let script = r#"echo '$*'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2048_multiple_occurrences() {
        let script = r#"
command1 $*
command2 $*
command3 $*
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 3);
    }

    #[test]
    fn test_sc2048_edge_case_beginning_of_line() {
        let script = r#"$* are all arguments"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2048_edge_case_middle() {
        let script = r#"echo "Args:" $* "end""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2048_no_false_positive_dollar_at() {
        // Should NOT flag $@ (that's covered by SC2068)
        let script = r#"command $@"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0); // SC2048 only looks for $*
    }
}
