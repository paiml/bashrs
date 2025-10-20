//! SC2191: The = here is literal. To assign by index, use ( [index]=value ) with no spaces. To keep as literal, quote it.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! array=([0]=value)  # Space between = and ( causes literal =
//! ```
//!
//! Good:
//! ```bash
//! array=([0]=value)   # No space - correct
//! array="([0]=value)" # Quoted - literal
//! ```
//!
//! # Rationale
//!
//! Space between = and ( in array assignment:
//! - Bash treats ( as a command, not array syntax
//! - Assignment fails or behaves unexpectedly
//! - Common syntax error
//!
//! Remove space or quote the entire value.
//!
//! # Auto-fix
//!
//! Remove space between = and (

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for space between = and ( in array assignment
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: var= (value) with space between = and (
    let pattern = Regex::new(r"([A-Za-z_][A-Za-z0-9_]*)=\s+\(").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip if this is a quoted string
        if line.contains("\"=(") || line.contains("'=(") {
            continue;
        }

        for cap in pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let var_name = cap.get(1).unwrap().as_str();

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let fix_text = format!("{}=(", var_name);

            let diagnostic = Diagnostic::new(
                "SC2191",
                Severity::Warning,
                "The = here is literal. To assign by index, use ( [index]=value ) with no spaces. To keep as literal, quote it.",
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
    fn test_sc2191_basic_detection() {
        let script = "array= (value1 value2)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2191");
    }

    #[test]
    fn test_sc2191_autofix() {
        let script = "files= (*.txt)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "files=("
        );
    }

    #[test]
    fn test_sc2191_multiple_spaces() {
        let script = "items=  (a b c)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2191_with_indexed() {
        let script = "array= ([0]=first [1]=second)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2191_false_positive_no_space() {
        let script = "array=(value1 value2)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2191_false_positive_quoted() {
        let script = "var=\"=(value)\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2191_false_positive_in_comment() {
        let script = "# array= (value)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2191_multiline() {
        let script = "echo test\narray= (a b)\necho done";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2191_in_function() {
        let script = "func() {\n  local arr= (x y z)\n}";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2191_underscore_var() {
        let script = "my_array= (value)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
