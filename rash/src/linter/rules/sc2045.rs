//! SC2045: Don't use ls output for iteration
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! for f in $(ls); do
//!   echo "$f"
//! done
//! ```
//!
//! Good:
//! ```bash
//! for f in *; do
//!   echo "$f"
//! done
//! ```
//!
//! # Rationale
//!
//! Using `$(ls)` or `` `ls` `` for iteration breaks with:
//! - Filenames containing spaces
//! - Filenames containing newlines
//! - Special characters
//!
//! Glob patterns (`*`) handle these correctly.
//!
//! # Auto-fix
//!
//! Replace `$(ls)` with `*` glob pattern

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for ls in for loops
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: for ... in $(ls ...) or `ls ...`
    let pattern1 = Regex::new(r"for\s+\w+\s+in\s+\$\(ls([^)]*)\)").unwrap();
    let pattern2 = Regex::new(r"for\s+\w+\s+in\s+`ls([^`]*)`").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check $(ls) pattern
        for cap in pattern1.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let ls_args = cap.get(1).map_or("", |m| m.as_str().trim());

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let fix_text = if ls_args.is_empty() {
                full_match.as_str().replace("$(ls)", "*")
            } else {
                full_match
                    .as_str()
                    .replace(&format!("$(ls {})", ls_args), &format!("{}*", ls_args))
            };

            let diagnostic = Diagnostic::new(
                "SC2045",
                Severity::Warning,
                "Don't use ls in for loops. Use glob patterns (*) instead.",
                Span::new(line_num, start_col, line_num, end_col),
            )
            .with_fix(Fix::new(fix_text));

            result.add(diagnostic);
        }

        // Check `ls` pattern
        for cap in pattern2.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let fix_text = full_match.as_str().replace("`ls`", "*").replace("`ls ", "");

            let diagnostic = Diagnostic::new(
                "SC2045",
                Severity::Warning,
                "Don't use ls in for loops. Use glob patterns (*) instead.",
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
    fn test_sc2045_basic_detection() {
        let script = r#"for f in $(ls); do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2045");
    }

    #[test]
    fn test_sc2045_autofix() {
        let script = r#"for f in $(ls); do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert!(result.diagnostics[0]
            .fix
            .as_ref()
            .unwrap()
            .replacement
            .contains("*"));
    }

    #[test]
    fn test_sc2045_backticks() {
        let script = r#"for f in `ls`; do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2045_with_pattern() {
        let script = r#"for f in $(ls *.txt); do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2045_false_positive_glob() {
        let script = r#"for f in *; do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2045_false_positive_array() {
        let script = r#"for f in "${files[@]}"; do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2045_multiline() {
        let script = r#"
for f in $(ls); do
    echo "$f"
done
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2045_nested_command() {
        let script = r#"for f in $(ls /tmp); do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2045_no_false_positive_find() {
        let script = r#"for f in $(find . -name "*.txt"); do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0); // This is SC2044, not SC2045
    }

    #[test]
    fn test_sc2045_variable_name() {
        let script = r#"for file in $(ls); do echo "$file"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
