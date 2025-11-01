// SC2085: Remove '$' or use 'local _=$((expr))' for local side effects
//
// In local declarations, $((expr)) should not have $ if you want side effects only.
// The $ causes the result to be assigned, which may not be intended.
//
// Examples:
// Bad:
//   local x=$((i++))             // Assigns result to x
//   declare y=$((count *= 2))    // y gets the result
//
// Good:
//   local x; ((i++))             // Side effect, x unset
//   local _=$((i++))             // Explicit throwaway assignment
//   ((i++))                      // Just side effect, no local
//
// Impact: Unintended variable assignments, confusion

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static LOCAL_WITH_ARITHMETIC: Lazy<Regex> = Lazy::new(|| {
    // Match: local var=$((expr)) or declare var=$((expr))
    // Also handles flags: local -r var=$((expr))
    // (?:\s+-[a-zA-Z]+)* matches optional flags like -r, -x, -i
    Regex::new(r"\b(local|declare)(?:\s+-[a-zA-Z]+)*\s+([a-zA-Z_][a-zA-Z0-9_]*)=\$\(\([^)]+\)\)")
        .unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in LOCAL_WITH_ARITHMETIC.captures_iter(line) {
            let var_name = cap.get(2).unwrap().as_str();

            // If assigning to _, it's intentional throwaway
            if var_name == "_" {
                continue;
            }

            let full_match = cap.get(0).unwrap();
            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2085",
                Severity::Info,
                format!(
                    "Use 'local {}; ((...))' if you want side effects only, not the result",
                    var_name
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
    fn test_sc2085_local_with_arithmetic() {
        let code = "local x=$((i++))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2085_declare() {
        let code = "declare y=$((count *= 2))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2085_throwaway_ok() {
        let code = "local _=$((i++))";
        let result = check(code);
        // Using _ is intentional
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2085_separate_ok() {
        let code = "local x; ((i++))";
        let result = check(code);
        // Separate declaration and side effect
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2085_plain_assignment_ok() {
        let code = "x=$((i++))";
        let result = check(code);
        // Not local/declare
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2085_comment_ok() {
        let code = "# local x=$((i++))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2085_local_simple_ok() {
        let code = "local x=$((2 + 3))";
        let result = check(code);
        // Assigning arithmetic result is valid
        assert_eq!(result.diagnostics.len(), 1); // Still warns
    }

    #[test]
    fn test_sc2085_multiple() {
        let code = "local a=$((x++)); declare b=$((y++))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2085_with_flags() {
        let code = "local -r result=$((i++))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2085_function_scope() {
        let code = r#"
function test() {
    local counter=$((i++))
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
