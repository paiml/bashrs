// SC2161: Use 'cd ... || exit' to handle cd failures.
//
// cd can fail if directory doesn't exist or lacks permissions.
// Always check for errors or the script continues in wrong directory.
//
// Examples:
// Bad:
//   cd "$dir"                    // Ignores failure
//   cd /nonexistent; rm -rf *    // Dangerous if cd fails!
//
// Good:
//   cd "$dir" || exit            // Exit on failure
//   cd "$dir" || return 1        // Return error in function
//   if ! cd "$dir"; then         // Explicit check
//     echo "Failed"
//     exit 1
//   fi
//
// Impact: Script continues in wrong directory, data loss risk

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static CD_WITHOUT_CHECK: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*cd\s+[^|;&]+$").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for cd without error handling
        if CD_WITHOUT_CHECK.is_match(line) {
            // Make sure it's actually a cd command
            if line.trim().starts_with("cd ") {
                let start_col = 1;
                let end_col = line.len() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2161",
                    Severity::Warning,
                    "Use 'cd ... || exit' or check for errors".to_string(),
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
    fn test_sc2161_cd_without_check() {
        let code = r#"cd "$dir""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2161_cd_or_exit_ok() {
        let code = r#"cd "$dir" || exit"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2161_cd_or_return_ok() {
        let code = r#"cd "$dir" || return 1"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2161_cd_and_ok() {
        let code = r#"cd "$dir" && ls"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2161_if_cd_ok() {
        let code = r#"if cd "$dir"; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2161_comment_ok() {
        let code = r#"# cd /tmp"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2161_cd_home() {
        let code = "cd ~";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2161_cd_semicolon() {
        let code = "cd /tmp; ls";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2161_multiple() {
        let code = "cd /tmp\ncd /var";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2161_cd_pipe() {
        let code = "cd /tmp | tee log";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
