//! SC2162: read without -r will mangle backslashes
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! read line
//! ```
//!
//! Good:
//! ```bash
//! read -r line
//! ```
//!
//! # Rationale
//!
//! Without -r flag, read treats backslashes as escape characters:
//! - `\n` becomes `n` (not newline)
//! - `\\` becomes `\`
//! - Unexpected behavior for most use cases
//!
//! Always use `read -r` unless you specifically need backslash processing.
//!
//! # Auto-fix
//!
//! Add -r flag to read command

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for read without -r flag
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: read (without -r) followed by variable name
    let pattern = Regex::new(r"\bread\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip if line contains "read -r"
        if line.contains("read -r") {
            continue;
        }

        for cap in pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let var_name = cap.get(1).unwrap().as_str();

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let fix_text = format!("read -r {}", var_name);

            let diagnostic = Diagnostic::new(
                "SC2162",
                Severity::Info,
                "read without -r will mangle backslashes",
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
    fn test_sc2162_basic_detection() {
        let script = "read line";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2162");
    }

    #[test]
    fn test_sc2162_autofix() {
        let script = "read line";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "read -r line"
        );
    }

    #[test]
    fn test_sc2162_in_while_loop() {
        let script = "while read line; do echo \"$line\"; done";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2162_multiple_variables() {
        let script = "read var1 var2";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1); // Only matches first var
    }

    #[test]
    fn test_sc2162_false_positive_with_r_flag() {
        let script = "read -r line";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2162_false_positive_in_comment() {
        let script = "# read line";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2162_with_prompt() {
        let script = "read input";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2162_underscore_variable() {
        let script = "read user_input";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2162_in_function() {
        let script = "get_input() { read value; }";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2162_with_ifs() {
        let script = "IFS= read line";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
