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

/// Check if a line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if text starts with command separator (;, |, &)
fn starts_with_separator(text: &str) -> bool {
    text.starts_with(';') || text.starts_with('|') || text.starts_with('&')
}

/// Check if backticks are in an assignment
fn is_in_assignment(prefix: &str) -> bool {
    prefix.contains('=') && !prefix.ends_with(';') && !prefix.ends_with('|')
}

/// Check if backticks are in safe context (echo/printf)
fn is_in_safe_context(prefix: &str) -> bool {
    prefix.contains("echo") || prefix.contains("printf")
}

/// Create diagnostic for executed backticks
fn create_backtick_diagnostic(line_num: usize, start_col: usize, end_col: usize) -> Diagnostic {
    Diagnostic::new(
        "SC2092",
        Severity::Warning,
        "Remove backticks to avoid executing output (or use eval if intentional)".to_string(),
        Span::new(line_num, start_col, line_num, end_col),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if is_comment_line(line) {
            continue;
        }

        for mat in EXECUTE_BACKTICKS.find_iter(line) {
            let matched_text = &line[mat.start()..mat.end()];

            // If it starts with a separator, it's a new command - should be flagged
            if !starts_with_separator(matched_text) {
                // Check the immediate context before the backticks
                let prefix = &line[..mat.start()];

                // Skip if in assignment or safe context
                if is_in_assignment(prefix) || is_in_safe_context(prefix) {
                    continue;
                }
            }

            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;
            let diagnostic = create_backtick_diagnostic(line_num, start_col, end_col);

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Manual Property Tests =====

    #[test]
    fn prop_sc2092_comments_never_diagnosed() {
        // Property: Comment lines should never produce diagnostics
        let test_cases = vec![
            "# `which cp` file",
            "  # `find . -name '*.sh'`",
            "\t# `date`",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2092_assignments_never_diagnosed() {
        // Property: Backticks in assignments should not be diagnosed
        let test_cases = vec!["result=`find .`", "output=`which cp`", "timestamp=`date`"];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2092_echo_printf_never_diagnosed() {
        // Property: Backticks in echo/printf should not be diagnosed
        let test_cases = vec!["echo `date`", "printf '%s' `date`", "echo test `which cp`"];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2092_executed_backticks_always_diagnosed() {
        // Property: Backticks at command position should be diagnosed
        let test_cases = vec!["`which cp` file", "`find . -name '*.sh'`", "`date`"];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
        }
    }

    #[test]
    fn prop_sc2092_after_separator_always_diagnosed() {
        // Property: Backticks after command separator should be diagnosed
        let test_cases = vec!["echo start; `find .`", "cmd | `which cp`", "cmd & `date`"];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
        }
    }

    #[test]
    fn prop_sc2092_dollar_paren_never_diagnosed() {
        // Property: $() command substitution should never be diagnosed
        let test_cases = vec!["result=$(date)", "$(which cp)", "echo $(date)"];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2092_diagnostic_code_always_sc2092() {
        // Property: All diagnostics must have code "SC2092"
        let code = "`which cp` file1\n`find .`";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(&diagnostic.code, "SC2092");
        }
    }

    #[test]
    fn prop_sc2092_diagnostic_severity_always_warning() {
        // Property: All diagnostics must be Warning severity
        let code = "`which cp` file";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Warning);
        }
    }

    #[test]
    fn prop_sc2092_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Original Unit Tests =====

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
    fn test_sc2092_after_semicolon() {
        let code = "echo start; `find . -name '*.sh'`";
        let result = check(code);
        // Backticks after semicolon are a new command - should be flagged
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
