// SC2121: To assign a variable, use just 'var=value', no $ or $ {}
//
// In assignments, don't use $ on the left side. $var refers to the VALUE, not the variable itself.
// Common mistake: trying to assign to a dereferenced variable.
//
// Examples:
// Bad:
//   $var=value              // Tries to execute "valueOfVar=value"
//   ${var}=value            // Same issue
//
// Good:
//   var=value               // Correct assignment
//   eval "$var=value"       // If you need dynamic variable names
//
// Impact: Assignment doesn't work, creates command not found error

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static DOLLAR_ASSIGNMENT: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: $var=value or ${var}=value at start of line
    Regex::new(r"^\s*\$(\{[a-zA-Z_][a-zA-Z0-9_]*\}|[a-zA-Z_][a-zA-Z0-9_]*)\s*=").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in DOLLAR_ASSIGNMENT.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2121",
                Severity::Error,
                "To assign a variable, use just 'var=value', no $".to_string(),
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
    fn test_sc2121_dollar_var_assignment() {
        let code = "$var=value";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2121_braced_var_assignment() {
        let code = "${var}=value";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2121_correct_assignment_ok() {
        let code = "var=value";
        let result = check(code);
        // Correct assignment
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2121_expansion_ok() {
        let code = r#"echo "$var=value""#;
        let result = check(code);
        // Inside string/command, not assignment
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2121_comment_ok() {
        let code = "# $var=value";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2121_indented() {
        let code = "    $myvar=test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2121_with_spaces() {
        let code = "$var = value";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2121_multiple() {
        let code = r#"
$var1=value1
$var2=value2
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2121_underscore() {
        let code = "$_private=secret";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2121_eval_ok() {
        let code = r#"eval "$var=value""#;
        let result = check(code);
        // eval with $ is intentional (dynamic assignment)
        assert_eq!(result.diagnostics.len(), 0);
    }
}
