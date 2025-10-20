//! SC2043: Use direct command instead of for loop with single element
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! for i in "$var"; do
//!   echo "$i"
//! done
//! ```
//!
//! Good:
//! ```bash
//! echo "$var"
//! ```
//!
//! # Rationale
//!
//! A for loop with a single quoted variable is unnecessary complexity.
//! Just use the variable directly.
//!
//! # Auto-fix
//!
//! Suggest simplifying to direct command

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Check for useless for loops with single element
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: for var in "$single_var"
    let pattern = Regex::new(r#"for\s+\w+\s+in\s+"(\$[A-Za-z_][A-Za-z0-9_]*)"\s*;"#).unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2043",
                Severity::Info,
                "This for loop will only run once. Use direct command instead.",
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
    fn test_sc2043_basic_detection() {
        let script = r#"for i in "$var"; do echo "$i"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2043");
    }

    #[test]
    fn test_sc2043_single_variable() {
        let script = r#"for f in "$file"; do cat "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2043_false_positive_multiple_vars() {
        let script = r#"for f in "$var1" "$var2"; do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2043_false_positive_unquoted() {
        let script = r#"for f in $files; do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0); // Unquoted might expand to multiple
    }

    #[test]
    fn test_sc2043_false_positive_array() {
        let script = r#"for f in "${array[@]}"; do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2043_false_positive_glob() {
        let script = r#"for f in *.txt; do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2043_multiline() {
        let script = r#"
for item in "$single_item"; do
    echo "$item"
done
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2043_different_variable_names() {
        let script = r#"for x in "$value"; do process "$x"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2043_false_positive_command_subst() {
        let script = r#"for f in "$(ls)"; do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0); // Command substitution might return multiple
    }

    #[test]
    fn test_sc2043_nested_var() {
        let script = r#"for i in "$myvar"; do echo "$i"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
