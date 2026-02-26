// SC2037: To assign command output to variable, use var=$(cmd), not cmd > $var
//
// Redirecting to a variable like `cmd > $var` doesn't assign output to the variable.
// It redirects to a FILE whose name is the value of $var. To capture output, use
// command substitution: var=$(cmd)
//
// Examples:
// Bad:
//   echo "result" > $VAR    // Redirects to file named by $VAR
//   cmd > $output           // Creates file named by $output
//   date > $timestamp       // Writes to file, doesn't assign
//
// Good:
//   VAR=$(echo "result")    // Assigns output to VAR
//   output=$(cmd)           // Captures command output
//   timestamp=$(date)       // Assigns date output
//
// Note: This is a common mistake for beginners.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static REDIRECT_TO_VAR: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: command > $VAR or command >> $VAR
    Regex::new(r"(>>?)\s*\$([a-zA-Z_][a-zA-Z0-9_]*)").unwrap()
});

/// Check if a line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if position is inside quotes
fn is_inside_quotes(line: &str, pos: usize) -> bool {
    let before = &line[..pos];
    let quote_count = before.matches('"').count() + before.matches('\'').count();
    quote_count % 2 == 1
}

/// Check if redirect is stderr or combined (&>, 2>)
fn is_stderr_or_combined_redirect(line: &str, pos: usize) -> bool {
    if pos > 0 {
        let prev_char = line.chars().nth(pos - 1);
        prev_char == Some('2') || prev_char == Some('&')
    } else {
        false
    }
}

/// Check if variable name is a common log file pattern
fn is_log_file_pattern(var_name: &str) -> bool {
    var_name == "LOGFILE"
        || var_name == "LOG"
        || var_name.ends_with("_FILE")
        || var_name.ends_with("_LOG")
}

/// Create diagnostic for redirect to variable
fn create_redirect_diagnostic(
    var_name: &str,
    redirect_op: &str,
    line_num: usize,
    start_col: usize,
    end_col: usize,
) -> Diagnostic {
    Diagnostic::new(
        "SC2037",
        Severity::Warning,
        format!(
            "To assign command output, use {}=$(cmd), not cmd {} ${}",
            var_name.to_lowercase(),
            redirect_op,
            var_name
        ),
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

        // Look for redirects to variables
        for cap in REDIRECT_TO_VAR.captures_iter(line) {
            let redirect_op = cap.get(1).unwrap().as_str();
            let var_name = cap.get(2).unwrap().as_str();
            let full_match = cap.get(0).unwrap().as_str();
            let pos = line.find(full_match).unwrap_or(0);

            // Skip if inside quotes
            if is_inside_quotes(line, pos) {
                continue;
            }

            // Skip stderr/combined redirects (2>, &>)
            if is_stderr_or_combined_redirect(line, pos) {
                continue;
            }

            // Skip common log file patterns
            if is_log_file_pattern(var_name) {
                continue;
            }

            let start_col = pos + 1;
            let end_col = start_col + full_match.len();
            let diagnostic =
                create_redirect_diagnostic(var_name, redirect_op, line_num, start_col, end_col);

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
    fn prop_sc2037_comments_never_diagnosed() {
        // Property: Comment lines should never produce diagnostics
        let test_cases = vec![
            "# echo result > $VAR",
            "  # cmd >> $output",
            "\t# date > $timestamp",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2037_quoted_never_diagnosed() {
        // Property: Redirects inside quotes should not be diagnosed
        let test_cases = vec![
            r#"echo "cmd > $VAR""#,
            r#"echo 'cmd >> $output'"#,
            r#"printf "redirect > $file""#,
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2037_redirect_to_var_always_diagnosed() {
        // Property: Unquoted redirects to variables should always be diagnosed
        let test_cases = vec![
            ("echo result > $VAR", ">"),
            ("cmd >> $output", ">>"),
            ("date > $timestamp", ">"),
        ];

        for (code, redirect_op) in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
            assert!(result.diagnostics[0].message.contains(redirect_op));
        }
    }

    #[test]
    fn prop_sc2037_log_patterns_never_diagnosed() {
        // Property: Common log file patterns should not be diagnosed
        let log_vars = vec!["LOGFILE", "LOG", "ERROR_FILE", "OUTPUT_LOG"];

        for var in log_vars {
            let code = format!("echo test >> ${}", var);
            let result = check(&code);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "Log pattern {} should be OK",
                var
            );
        }
    }

    #[test]
    fn prop_sc2037_stderr_redirects_never_diagnosed() {
        // Property: Stderr/combined redirects should not be diagnosed
        let test_cases = vec!["cmd 2> $ERROR", "cmd &> $COMBINED"];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0, "Should allow: {}", code);
        }
    }

    #[test]
    fn prop_sc2037_command_subst_never_diagnosed() {
        // Property: Command substitution should never be diagnosed
        let test_cases = vec!["VAR=$(echo result)", "output=$(cmd)", "timestamp=$(date)"];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2037_diagnostic_code_always_sc2037() {
        // Property: All diagnostics must have code "SC2037"
        let code = "echo a > $VAR1\necho b > $VAR2";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(&diagnostic.code, "SC2037");
        }
    }

    #[test]
    fn prop_sc2037_diagnostic_severity_always_warning() {
        // Property: All diagnostics must be Warning severity
        let code = "echo test > $OUTPUT";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Warning);
        }
    }

    #[test]
    fn prop_sc2037_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Original Unit Tests =====

    #[test]
    fn test_sc2037_redirect_to_var() {
        let code = r#"echo "result" > $VAR"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2037");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("$(cmd)"));
    }

    #[test]
    fn test_sc2037_append_to_var() {
        let code = r#"cmd >> $output"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2037_date_to_var() {
        let code = r#"date > $timestamp"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2037_command_subst_ok() {
        let code = r#"VAR=$(echo "result")"#;
        let result = check(code);
        // Proper command substitution, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2037_redirect_to_file_ok() {
        let code = r#"echo "result" > output.txt"#;
        let result = check(code);
        // Redirect to literal file, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2037_logfile_var_ok() {
        let code = r#"echo "log entry" >> $LOGFILE"#;
        let result = check(code);
        // LOGFILE is common pattern for actual file, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2037_stderr_redirect_ok() {
        let code = r#"cmd 2> $ERROR"#;
        let result = check(code);
        // Stderr redirect (2>) is different, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2037_in_quotes_ok() {
        let code = r#"echo "cmd > $VAR""#;
        let result = check(code);
        // Inside quotes, not actual redirect
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2037_comment_ok() {
        let code = r#"# echo "result" > $VAR"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2037_multiple_redirects() {
        let code = r#"
echo "a" > $VAR1
echo "b" > $VAR2
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2037_combined_redirect_ok() {
        let code = r#"cmd &> $COMBINED_LOG"#;
        let result = check(code);
        // Combined redirect with _LOG suffix, OK
        assert_eq!(result.diagnostics.len(), 0);
    }
}
