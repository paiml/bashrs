// SC2242: Can only exit from loops or functions, not case
use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let mut in_case = false;
    let mut in_loop = false;
    let mut in_function = false;

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with('#') {
            continue;
        }

        // Track context
        if trimmed.starts_with("case ") {
            in_case = true;
        }
        if trimmed.starts_with("for ")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("until ")
        {
            in_loop = true;
        }
        if trimmed.contains("() {") || trimmed.starts_with("function ") {
            in_function = true;
        }
        if trimmed == "esac" {
            in_case = false;
        }
        if trimmed == "done" {
            in_loop = false;
        }
        if trimmed == "}" {
            in_function = false;
        }

        // Check for break/continue in case (when not in loop)
        if in_case && !in_loop && !in_function {
            if trimmed.contains("break") || trimmed.contains("continue") {
                let diagnostic = Diagnostic::new(
                    "SC2242",
                    Severity::Error,
                    "Can only break/continue from loops. Use 'exit' to exit case or function"
                        .to_string(),
                    Span::new(line_num, 1, line_num, line.len() + 1),
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
