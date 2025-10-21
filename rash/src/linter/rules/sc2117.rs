// SC2117: Unreachable code after exit or return
//
// Code after 'exit' or 'return' in a function will never execute.
// This is likely a mistake - either remove the dead code or fix the logic.
//
// Examples:
// Bad:
//   exit 1
//   echo "This never runs"          // Unreachable
//
//   return 0
//   local x=5                         // Unreachable
//
// Good:
//   echo "Running"
//   exit 1                            // exit at end
//
//   local x=5
//   return 0                          // return at end
//
// Impact: Dead code, logic error

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static EXIT_OR_RETURN: Lazy<Regex> = Lazy::new(|| {
    // Match: exit or return at command position
    Regex::new(r"^\s*(exit|return)\b").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check if this line has exit or return
        if EXIT_OR_RETURN.is_match(line) {
            // Check if there's non-comment code after this line
            let mut has_code_after = false;
            for (j, next_line) in lines[i + 1..].iter().enumerate() {
                let trimmed = next_line.trim();

                // Skip empty lines and comments
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }

                // Check if it's a closing brace (end of function/block)
                if trimmed == "}" || trimmed == "fi" || trimmed == "done" || trimmed == "esac" {
                    break;
                }

                // Found code after exit/return
                has_code_after = true;
                let unreachable_line = i + 1 + j + 1;
                let start_col = 1;
                let end_col = next_line.len() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2117",
                    Severity::Warning,
                    format!(
                        "Unreachable code after '{}' on line {}",
                        if line.contains("exit") {
                            "exit"
                        } else {
                            "return"
                        },
                        line_num
                    ),
                    Span::new(unreachable_line, start_col, unreachable_line, end_col),
                );

                result.add(diagnostic);
                break; // Only report first unreachable line
            }

            if has_code_after {
                break; // Only check first exit/return
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2117_exit_with_code() {
        let code = r#"
exit 1
echo "Never runs"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2117_return_with_code() {
        let code = r#"
return 0
local x=5
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2117_exit_at_end_ok() {
        let code = r#"
echo "Running"
exit 0
"#;
        let result = check(code);
        // No code after exit
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2117_return_at_end_ok() {
        let code = r#"
local result=$1
return 0
"#;
        let result = check(code);
        // No code after return
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2117_exit_before_closing_brace_ok() {
        let code = r#"
foo() {
    exit 1
}
"#;
        let result = check(code);
        // Closing } doesn't count as unreachable
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2117_comment_after_ok() {
        let code = r#"
exit 1
# This is just a comment
"#;
        let result = check(code);
        // Comments don't count
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2117_empty_lines_ok() {
        let code = r#"
return 0

"#;
        let result = check(code);
        // Empty lines don't count
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2117_multiple_lines() {
        let code = r#"
exit 1
echo "line1"
echo "line2"
"#;
        let result = check(code);
        // Only reports first unreachable line
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2117_in_function() {
        let code = r#"
foo() {
    return 1
    echo "unreachable"
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2117_after_fi_ok() {
        let code = r#"
if [ $? -eq 0 ]; then
    exit 0
fi
echo "This runs if condition is false"
"#;
        let result = check(code);
        // Code after 'fi' is reachable
        assert_eq!(result.diagnostics.len(), 0);
    }
}
