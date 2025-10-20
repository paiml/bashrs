//! SC2145: Argument mixes string and array. Use * or separate arguments.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! args=(a b c)
//! echo "Arguments: $args[@]"  # Incorrect syntax
//! ```
//!
//! Good:
//! ```bash
//! args=(a b c)
//! echo "Arguments: ${args[*]}"  # Correct
//! echo "Arguments: ${args[@]}"  # Also correct
//! ```
//!
//! # Rationale
//!
//! Mixing string and array without braces:
//! - `$array[@]` is incorrect syntax
//! - Should be `${array[@]}` or `${array[*]}`
//! - `[@]` treated as literal text without braces
//!
//! Always use braces for array expansion with indices.
//!
//! # Auto-fix
//!
//! Add braces around array reference

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for array reference without braces
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: $array[@] or $array[*] (without braces)
    let pattern = Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)\[[@*]\]").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let var_name = cap.get(1).unwrap().as_str();

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            // Determine if [@] or [*]
            let index = if full_match.as_str().contains("[@]") {
                "[@]"
            } else {
                "[*]"
            };

            let fix_text = format!("${{{}{}}} ", var_name, index);

            let diagnostic = Diagnostic::new(
                "SC2145",
                Severity::Warning,
                "Argument mixes string and array. Use * or separate arguments.",
                Span::new(line_num, start_col, line_num, end_col),
            )
            .with_fix(Fix::new(fix_text.trim()));

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2145_basic_detection() {
        let script = "args=(a b c)\necho \"Arguments: $args[@]\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2145");
    }

    #[test]
    fn test_sc2145_autofix_at() {
        let script = "files=(*.txt)\ncat $files[@]";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "${files[@]}"
        );
    }

    #[test]
    fn test_sc2145_autofix_star() {
        let script = "items=(x y z)\necho \"$items[*]\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "${items[*]}"
        );
    }

    #[test]
    fn test_sc2145_in_string() {
        let script = "array=(1 2 3)\nprintf \"Values: $array[@]\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2145_false_positive_with_braces() {
        let script = "args=(a b c)\necho \"${args[@]}\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2145_false_positive_braces_star() {
        let script = "files=(*.txt)\necho \"${files[*]}\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2145_false_positive_in_comment() {
        let script = "# echo \"$args[@]\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2145_multiple_occurrences() {
        let script = "args=(a b)\nfiles=(*.txt)\necho \"$args[@] $files[*]\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2145_in_command_substitution() {
        let script = "result=$(process $items[@])";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2145_with_underscore() {
        let script = "my_array=(x y)\necho $my_array[@]";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
