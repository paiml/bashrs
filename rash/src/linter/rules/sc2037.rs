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
use once_cell::sync::Lazy;
use regex::Regex;

static REDIRECT_TO_VAR: Lazy<Regex> = Lazy::new(|| {
    // Match: command > $VAR or command >> $VAR
    Regex::new(r"(>>?)\s*\$([a-zA-Z_][a-zA-Z0-9_]*)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for redirects to variables
        for cap in REDIRECT_TO_VAR.captures_iter(line) {
            let redirect_op = cap.get(1).unwrap().as_str();
            let var_name = cap.get(2).unwrap().as_str();

            // Skip if inside quotes
            let full_match = cap.get(0).unwrap().as_str();
            let pos = line.find(full_match).unwrap_or(0);
            let before = &line[..pos];
            let quote_count = before.matches('"').count() + before.matches('\'').count();
            if quote_count % 2 == 1 {
                continue;
            }

            // Skip stderr/combined redirects (2>, &>)
            if pos > 0 {
                let prev_char = line.chars().nth(pos - 1);
                if prev_char == Some('2') || prev_char == Some('&') {
                    continue;
                }
            }

            // Skip common log file patterns like $LOGFILE
            if var_name == "LOGFILE"
                || var_name == "LOG"
                || var_name.ends_with("_FILE")
                || var_name.ends_with("_LOG")
            {
                continue;
            }

            let start_col = pos + 1;
            let end_col = start_col + full_match.len();

            let diagnostic = Diagnostic::new(
                "SC2037",
                Severity::Warning,
                format!(
                    "To assign command output, use {}=$(cmd), not cmd {} ${}",
                    var_name.to_lowercase(),
                    redirect_op,
                    var_name
                ),
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
