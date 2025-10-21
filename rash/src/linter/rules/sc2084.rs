// SC2084: Remove '$' or use '_=$((expr))' to avoid executing output
//
// Using $((expr)) in a command position will execute the result as a command.
// If you want the side effect of the arithmetic, assign to a variable.
//
// Examples:
// Bad:
//   $((i++))                     // Tries to execute result as command
//   $((count *= 2))              // Side effect but executes output
//
// Good:
//   : $((i++))                   // : discards output
//   _=$((i++))                   // Assign to throwaway variable
//   ((i++))                      // Use (( )) for side effects
//
// Impact: Unexpected command execution, errors

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ARITHMETIC_AS_COMMAND: Lazy<Regex> = Lazy::new(|| {
    // Match: $((expr)) at start of line or after ; or && or ||
    Regex::new(r"(^|[;&|]+)\s*\$\(\([^)]+\)\)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check if arithmetic expansion is used as a command
        for mat in ARITHMETIC_AS_COMMAND.find_iter(line) {
            let matched = mat.as_str();

            // Skip if it's in an assignment context
            if line.contains('=') && line.find('=').unwrap() < mat.start() {
                continue;
            }

            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2084",
                Severity::Warning,
                "Remove '$' or use '_=$((expr))' to avoid executing output as a command"
                    .to_string(),
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
    fn test_sc2084_arithmetic_as_command() {
        let code = "$((i++))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2084_side_effect() {
        let code = "$((count *= 2))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2084_after_semicolon() {
        let code = "echo start; $((x++))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2084_assignment_ok() {
        let code = "result=$((i++))";
        let result = check(code);
        // Assignment is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2084_double_paren_ok() {
        let code = "((i++))";
        let result = check(code);
        // (( )) without $ is correct for side effects
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2084_colon_ok() {
        let code = ": $((i++))";
        let result = check(code);
        // Using : to discard output
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2084_throwaway_var_ok() {
        let code = "_=$((i++))";
        let result = check(code);
        // Assigning to _ is correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2084_comment_ok() {
        let code = "# $((i++))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2084_in_echo() {
        let code = "echo $((i++))";
        let result = check(code);
        // In echo argument is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2084_in_test() {
        let code = "[ $((count++)) -gt 5 ]";
        let result = check(code);
        // In test is OK
        assert_eq!(result.diagnostics.len(), 0);
    }
}
