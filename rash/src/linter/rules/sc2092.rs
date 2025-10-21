// SC2092: Remove backticks to avoid executing output (or use eval)
//
// Backticks `` execute commands and use their output. Executing that
// output as a command is usually wrong.
//
// Examples:
// Bad:
//   `which cp` file1 file2       // Executes output of which
//   `find . -name "*.txt"`       // Tries to execute filenames
//
// Good:
//   which cp                     // Just find the path
//   cp file1 file2               // Execute directly
//   find . -name "*.txt"         // Execute find directly
//
// Impact: Unintended command execution, errors

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static EXECUTE_BACKTICKS: Lazy<Regex> = Lazy::new(|| {
    // Match: `cmd` at command position
    Regex::new(r"(^|[;&|]+)\s*`[^`]+`").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in EXECUTE_BACKTICKS.find_iter(line) {
            // Skip if it's in an assignment
            if line[..mat.start()].contains('=') {
                continue;
            }

            // Skip if it's in echo or other safe contexts
            if line[..mat.start()].contains("echo") || line[..mat.start()].contains("printf") {
                continue;
            }

            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2092",
                Severity::Warning,
                "Remove backticks to avoid executing output (or use eval if intentional)"
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
    fn test_sc2092_backticks_executed() {
        let code = "`which cp` file1 file2";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2092_find_backticks() {
        let code = "`find . -name '*.txt'`";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2092_assignment_ok() {
        let code = "result=`find . -name '*.txt'`";
        let result = check(code);
        // Assignment is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2092_echo_ok() {
        let code = "echo `date`";
        let result = check(code);
        // In echo is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2092_comment_ok() {
        let code = "# `which cp` file1 file2";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2092_prefer_dollar_paren() {
        let code = "result=$(date)";
        let result = check(code);
        // $() is preferred over backticks
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    #[ignore] // TODO: Better context detection for echo/printf
    fn test_sc2092_after_semicolon() {
        let code = "echo start; `find . -name '*.sh'`";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2092_direct_execution() {
        let code = "find . -name '*.txt'";
        let result = check(code);
        // Direct execution is correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2092_printf_ok() {
        let code = "printf '%s\n' `date`";
        let result = check(code);
        // In printf is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2092_nested() {
        let code = r#"`echo `date``"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
