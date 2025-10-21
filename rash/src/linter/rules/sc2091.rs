// SC2091: Remove surrounding $() to avoid executing output (or use eval)
//
// Command substitution $() executes the command and uses its output.
// If you then try to execute that output, it's likely wrong.
//
// Examples:
// Bad:
//   $(which cp) file1 file2      // Executes output of which
//   result=$(find . -name "*.txt")
//   $result                      // Tries to execute filenames
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

static EXECUTE_COMMAND_SUB: Lazy<Regex> = Lazy::new(|| {
    // Match: $(cmd) at command position (start of line or after ; && ||)
    Regex::new(r"(^|[;&|]+)\s*\$\([^)]+\)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in EXECUTE_COMMAND_SUB.find_iter(line) {
            // Skip if it's in an assignment
            if line[..mat.start()].contains('=') {
                continue;
            }

            // Skip if it's in a test
            if line[..mat.start()].contains('[') || line[..mat.start()].contains("if") {
                continue;
            }

            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2091",
                Severity::Warning,
                "Remove surrounding $() to avoid executing output (or use eval if intentional)"
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
    fn test_sc2091_which_cp() {
        let code = "$(which cp) file1 file2";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2091_command_sub_executed() {
        let code = "$(find . -name '*.txt')";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2091_assignment_ok() {
        let code = "result=$(find . -name '*.txt')";
        let result = check(code);
        // Assignment is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2091_in_echo_ok() {
        let code = "echo $(date)";
        let result = check(code);
        // In echo is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2091_in_test_ok() {
        let code = "if $(true); then echo ok; fi";
        let result = check(code);
        // In if statement context
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2091_comment_ok() {
        let code = "# $(which cp) file1 file2";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2091_after_semicolon() {
        let code = "echo start; $(find . -name '*.sh')";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2091_direct_execution() {
        let code = "find . -name '*.txt'";
        let result = check(code);
        // Direct execution is correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2091_eval_comment() {
        let code = "$(grep pattern file)  # Should use eval";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2091_with_pipe() {
        let code = "cat file | $(awk '{print $1}')";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
