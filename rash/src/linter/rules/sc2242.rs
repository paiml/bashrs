// SC2242: Can only exit from loops or functions, not case
use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check if line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if line starts a case statement
fn is_case_start(line: &str) -> bool {
    line.trim_start().starts_with("case ")
}

/// Check if line starts a loop
fn is_loop_start(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("for ")
        || trimmed.starts_with("while ")
        || trimmed.starts_with("until ")
}

/// Check if line starts a function
fn is_function_start(line: &str) -> bool {
    line.trim_start().contains("() {") || line.trim_start().starts_with("function ")
}

/// Check if line ends a case statement
fn is_case_end(line: &str) -> bool {
    line.trim_start() == "esac"
}

/// Check if line ends a loop
fn is_loop_end(line: &str) -> bool {
    line.trim_start() == "done"
}

/// Check if line ends a function
fn is_function_end(line: &str) -> bool {
    line.trim_start() == "}"
}

/// Check if line contains break or continue
fn has_break_or_continue(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.contains("break") || trimmed.contains("continue")
}

/// Build diagnostic for invalid break/continue in case
fn build_diagnostic(line_num: usize, line_len: usize) -> Diagnostic {
    Diagnostic::new(
        "SC2242",
        Severity::Error,
        "Can only break/continue from loops. Use 'exit' to exit case or function".to_string(),
        Span::new(line_num, 1, line_num, line_len + 1),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let mut in_case = false;
    let mut in_loop = false;
    let mut in_function = false;

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if is_comment_line(line) {
            continue;
        }

        // Track context
        if is_case_start(line) {
            in_case = true;
        }
        if is_loop_start(line) {
            in_loop = true;
        }
        if is_function_start(line) {
            in_function = true;
        }
        if is_case_end(line) {
            in_case = false;
        }
        if is_loop_end(line) {
            in_loop = false;
        }
        if is_function_end(line) {
            in_function = false;
        }

        // Check for break/continue in case (when not in loop or function)
        if in_case && !in_loop && !in_function && has_break_or_continue(line) {
            let diagnostic = build_diagnostic(line_num, line.len());
            result.add(diagnostic);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2242_break_in_case() {
        let code = "case $x in\n  a) break;;\nesac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2242_break_in_loop_ok() {
        let code = "while true; do\n  break\ndone";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2242_continue_in_case() {
        let code = "case $x in\n  a) continue;;\nesac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2242_exit_in_case_ok() {
        let code = "case $x in\n  a) exit 1;;\nesac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2242_comment_skipped() {
        let code = "# case x in a) break;; esac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2242_case_in_loop() {
        let code = "for x in *; do\n  case $x in\n    *.txt) break;;\n  esac\ndone";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // In loop, so break is valid
    }
    #[test]
    fn test_sc2242_no_code() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2242_normal_command() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2242_return_in_function_ok() {
        let code = "foo() {\n  case $1 in\n    a) return 1;;\n  esac\n}";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2242_simple_case() {
        let code = "case $var in\n  x) echo ok;;\nesac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // No break/continue
    }
}
