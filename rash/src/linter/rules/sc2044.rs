//! SC2044: Use find with -print0 and read -d '' for safe iteration
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! for f in $(find . -name "*.txt"); do
//!   echo "$f"
//! done
//! ```
//!
//! Good:
//! ```bash
//! while IFS= read -r -d '' f; do
//!   echo "$f"
//! done < <(find . -name "*.txt" -print0)
//! ```
//!
//! # Rationale
//!
//! Using `$(find ...)` for iteration breaks with filenames containing:
//! - Spaces
//! - Newlines
//! - Special characters
//!
//! Use `-print0` with `read -d ''` for null-delimited safe iteration.
//!
//! # Auto-fix
//!
//! Suggest using while read with -print0

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Check for find in for loops without -print0
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: for ... in $(find ...)
    let pattern = Regex::new(r"for\s+(\w+)\s+in\s+\$\(find\s+([^)]+)\)").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let var_name = cap.get(1).unwrap().as_str();
            let find_args = cap.get(2).unwrap().as_str();

            // Skip if already using -print0
            if find_args.contains("-print0") {
                continue;
            }

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2044",
                Severity::Warning,
                format!(
                    "Use 'while IFS= read -r -d '' {}; do ... done < <(find {} -print0)' for safe iteration",
                    var_name, find_args
                ),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2044_basic_detection() {
        let script = r#"for f in $(find . -name "*.txt"); do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2044");
    }

    #[test]
    fn test_sc2044_with_type() {
        let script = r#"for f in $(find /tmp -type f); do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2044_complex_find() {
        let script = r#"for f in $(find . -name "*.log" -mtime +7); do rm "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2044_false_positive_with_print0() {
        let script = r#"for f in $(find . -print0); do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2044_false_positive_glob() {
        let script = r#"for f in *.txt; do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2044_false_positive_ls() {
        let script = r#"for f in $(ls); do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0); // This is SC2045
    }

    #[test]
    fn test_sc2044_multiline() {
        let script = r#"
for file in $(find /var/log -name "*.log"); do
    echo "$file"
done
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2044_variable_name() {
        let script = r#"for path in $(find . -type d); do echo "$path"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2044_find_with_exec() {
        let script = r#"for f in $(find . -exec echo {} \;); do process "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2044_nested_find() {
        let script = r#"for f in $(find /home/user -name "*.sh"); do bash "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
