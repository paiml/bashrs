// SC2155: Declare and assign separately to avoid masking return values
//
// When using `local var=$(command)`, the return value of command is masked
// by the return value of `local` (which is always 0).
//
// Examples:
// Bad:
//   local result=$(failing_command)
//   # $? is 0 (from local), not the command's exit code
//
// Good:
//   local result
//   result=$(failing_command)
//   # $? is the command's actual exit code

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static LOCAL_WITH_COMMAND_SUBST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(local|declare|readonly|export)\s+([A-Za-z_][A-Za-z0-9_]*)=\$\(").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in LOCAL_WITH_COMMAND_SUBST.captures_iter(line) {
            if let Some(keyword) = cap.get(1) {
                let start_col = keyword.start() + 1;
                let end_col = keyword.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2155",
                    Severity::Warning,
                    format!(
                        "Declare and assign separately to avoid masking return values ({})",
                        keyword.as_str()
                    ),
                    Span::new(line_num, start_col, line_num, end_col),
                );

                result.add(diagnostic);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2155_local_with_command_subst() {
        let code = r#"local result=$(command)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2155");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sc2155_declare_with_command_subst() {
        let code = r#"declare var=$(some_command)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2155_readonly_with_command_subst() {
        let code = r#"readonly CONST=$(get_value)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2155_export_with_command_subst() {
        let code = r#"export PATH=$(get_path)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2155_separate_ok() {
        let code = r#"
local result
result=$(command)
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2155_simple_assignment_ok() {
        let code = r#"local var="value""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2155_variable_expansion_ok() {
        let code = r#"local var="$other""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2155_multiple_on_line() {
        let code = r#"local a=$(cmd1) b=$(cmd2)"#;
        let result = check(code);
        // Should detect at least the first one
        assert!(result.diagnostics.len() >= 1);
    }

    #[test]
    fn test_sc2155_function_context() {
        let code = r#"
function test() {
    local result=$(failing_command)
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2155_no_command_subst() {
        let code = r#"local var=value"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
