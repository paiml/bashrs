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
use regex::Regex;

static EXIT_OR_RETURN: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: exit or return at command position
    Regex::new(r"^\s*(exit|return)\b").unwrap()
});

/// Check if a line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if a line contains exit or return
fn is_exit_or_return_line(line: &str) -> bool {
    EXIT_OR_RETURN.is_match(line)
}

/// Check if a line is a closing token (}, fi, done, esac, ;;, ;&, ;;&)
/// Issue #123: Also handle case statement terminators
fn is_closing_token(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed == "}"
        || trimmed == "fi"
        || trimmed == "done"
        || trimmed == "esac"
        || trimmed == ";;"
        || trimmed == ";&"
        || trimmed == ";;&"
        || trimmed.ends_with(";;")  // Handle `) ... ;;` on same line
        || trimmed.ends_with(";&")
        || trimmed.ends_with(";;&")
}

/// Extract the keyword (exit or return) from a line
fn get_keyword(line: &str) -> &str {
    if line.contains("exit") {
        "exit"
    } else {
        "return"
    }
}

/// Create a diagnostic for unreachable code
fn create_unreachable_diagnostic(
    keyword: &str,
    exit_line: usize,
    unreachable_line: usize,
    line_content: &str,
) -> Diagnostic {
    let start_col = 1;
    let end_col = line_content.len() + 1;

    Diagnostic::new(
        "SC2117",
        Severity::Warning,
        format!("Unreachable code after '{}' on line {}", keyword, exit_line),
        Span::new(unreachable_line, start_col, unreachable_line, end_col),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;

        if is_comment_line(line) {
            continue;
        }

        if is_exit_or_return_line(line) {
            // Check if there's non-comment code after this line
            let mut has_code_after = false;

            for (j, next_line) in lines[i + 1..].iter().enumerate() {
                let trimmed = next_line.trim();

                // Skip empty lines and comments
                if trimmed.is_empty() || is_comment_line(next_line) {
                    continue;
                }

                // Check if it's a closing brace (end of function/block)
                if is_closing_token(next_line) {
                    break;
                }

                // Found code after exit/return
                has_code_after = true;
                let unreachable_line = i + 1 + j + 1;
                let keyword = get_keyword(line);
                let diagnostic =
                    create_unreachable_diagnostic(keyword, line_num, unreachable_line, next_line);

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

    // ===== Manual Property Tests =====

    #[test]
    fn prop_sc2117_comments_never_diagnosed() {
        // Property: Comment lines should never produce diagnostics
        let test_cases = vec!["# exit 1\n# echo test", "  # return 0\n  # local x=5"];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2117_exit_at_end_never_diagnosed() {
        // Property: exit/return at end of script should not be diagnosed
        let test_cases = vec![
            "echo test\nexit 0",
            "local x=5\nreturn 1",
            "exit 1\n\n", // with empty lines
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2117_code_after_always_diagnosed() {
        // Property: Code after exit/return should always be diagnosed
        let test_cases = vec![
            ("exit 1\necho test", "exit"),
            ("return 0\nlocal x=5", "return"),
        ];

        for (code, keyword) in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1);
            assert!(result.diagnostics[0].message.contains(keyword));
        }
    }

    #[test]
    fn prop_sc2117_closing_tokens_not_unreachable() {
        // Property: Closing tokens (}, fi, done, esac) should not be diagnosed
        let closing_tokens = vec!["}", "fi", "done", "esac"];

        for token in closing_tokens {
            let code = format!("exit 0\n{}", token);
            let result = check(&code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2117_only_first_unreachable_reported() {
        // Property: Only first unreachable line should be reported
        let code = "exit 1\necho line1\necho line2\necho line3";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn prop_sc2117_diagnostic_code_always_sc2117() {
        // Property: All diagnostics must have code "SC2117"
        let code = "exit 1\necho test";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(&diagnostic.code, "SC2117");
        }
    }

    #[test]
    fn prop_sc2117_diagnostic_severity_always_warning() {
        // Property: All diagnostics must be Warning severity
        let code = "return 0\nlocal x=1";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Warning);
        }
    }

    #[test]
    fn prop_sc2117_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Original Unit Tests =====

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

    // Issue #123: Case statement terminators should not be flagged
    #[test]
    fn test_issue_123_exit_before_case_terminator() {
        let code = r#"
case $option in
    a)
        exit 0
        ;;
    b)
        exit 1
        ;;
esac
"#;
        let result = check(code);
        // ;; after exit is a case terminator, not unreachable code
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2117 must NOT flag ;; after exit in case statements"
        );
    }

    #[test]
    fn test_issue_123_return_before_case_terminator() {
        let code = r#"
case $mode in
    debug) return 1 ;;
    prod) return 0 ;;
esac
"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2117 must NOT flag ;; after return"
        );
    }

    #[test]
    fn test_issue_123_fallthrough_terminator() {
        let code = r#"
case $x in
    a)
        exit 1
        ;&
    b)
        exit 0
        ;;
esac
"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2117 must NOT flag ;& after exit"
        );
    }
}
