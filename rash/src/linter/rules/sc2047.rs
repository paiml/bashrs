// SC2047: Quote this to prevent word splitting, or use [[ ]] for regex.
//
// Unquoted variables in test conditions can cause unexpected word splitting.
// This leads to syntax errors or incorrect test results.
//
// Examples:
// Bad:
//   [ -z $var ]              // If var is empty, becomes [ -z ], syntax error
//   [ $count -gt 5 ]         // If count="1 2", becomes [ 1 2 -gt 5 ], error
//   test $status = "ok"      // If status="not ok", word splits
//
// Good:
//   [ -z "$var" ]            // Properly quoted
//   [ "$count" -gt 5 ]       // Safe from word splitting
//   test "$status" = "ok"    // Correct
//   [[ -z $var ]]            // [[ ]] doesn't word split (bash/ksh)
//
// Note: Always quote variables in [ ] tests. Or use [[ ]] which is safer.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static TEST_COMMAND: Lazy<Regex> = Lazy::new(|| {
    // Match [ ... ] or test ...
    Regex::new(r"\[\s+[^\]]+\]|test\s+.*").unwrap()
});

static VARIABLE_REF: Lazy<Regex> = Lazy::new(|| {
    // Match $var or ${var}
    Regex::new(r"\$\{?([a-zA-Z_][a-zA-Z0-9_]*)\}?").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip [[ ]] tests (they don't word split)
        if line.contains("[[") {
            continue;
        }

        // Find all [ ... ] or test commands in the line
        for test_match in TEST_COMMAND.find_iter(line) {
            let test_str = test_match.as_str();
            let test_start = test_match.start();

            // Find all variable references within this test command
            for var_match in VARIABLE_REF.find_iter(test_str) {
                let var_str = var_match.as_str();
                let var_start_in_test = var_match.start();
                let var_start_in_line = test_start + var_start_in_test;

                // Check if the variable is quoted
                let before_var = &line[..var_start_in_line];
                let after_var_pos = var_start_in_line + var_str.len();

                // Check for double quotes surrounding the variable
                if before_var.ends_with('"') && after_var_pos < line.len() {
                    let after = &line[after_var_pos..];
                    if after.starts_with('"') {
                        continue; // Already quoted
                    }
                }

                let start_col = var_start_in_line + 1;
                let end_col = start_col + var_str.len();

                let diagnostic = Diagnostic::new(
                    "SC2047",
                    Severity::Warning,
                    format!(
                        "Quote {} to prevent word splitting, or use [[..]] instead of [..]",
                        var_str
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
    fn test_sc2047_unquoted_var_in_test() {
        let code = r#"[ -z $var ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2047");
        assert!(result.diagnostics[0].message.contains("Quote"));
    }

    #[test]
    fn test_sc2047_unquoted_var_with_gt() {
        let code = r#"[ $count -gt 5 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2047_test_command() {
        let code = r#"test $status = "ok""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2047_quoted_var_ok() {
        let code = r#"[ -z "$var" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2047_double_bracket_ok() {
        let code = r#"[[ -z $var ]]"#;
        let result = check(code);
        // [[ ]] doesn't word split, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2047_quoted_count_ok() {
        let code = r#"[ "$count" -gt 5 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2047_test_quoted_ok() {
        let code = r#"test "$status" = "ok""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2047_comment_ok() {
        let code = r#"# [ -z $var ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2047_multiple_unquoted() {
        let code = r#"[ $a = $b ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2047_braced_var() {
        let code = r#"[ -n ${var} ]"#;
        let result = check(code);
        // Braced but not quoted
        assert_eq!(result.diagnostics.len(), 1);
    }
}
