// SC2152: Can only exit with status 0-255. Other values are truncated.
//
// Like return, exit codes are limited to 0-255. Values outside this range
// will be truncated modulo 256.
//
// Examples:
// Bad:
//   exit 256                // Exits with 0 (256 % 256)
//   exit 1000               // Exits with 232 (1000 % 256)
//   exit -1                 // Exits with 255 (-1 % 256)
//
// Good:
//   exit 0                  // Success
//   exit 1                  // Generic failure
//   exit 2                  // Specific error code
//
// Impact: Exit code truncated, unexpected behavior

use crate::linter::shell_words::{has_command_word, logical_lines};
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static EXIT_OUT_OF_RANGE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\bexit\s+(-[0-9]+|[0-9]{3,})").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in logical_lines(source) {
        let line = line.as_str();
        if !has_command_word(line, "exit") {
            continue;
        }

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in EXIT_OUT_OF_RANGE.find_iter(line) {
            let matched = mat.as_str();

            if let Some(num_str) = matched.strip_prefix("exit").map(|s| s.trim()) {
                if let Ok(num) = num_str.parse::<i32>() {
                    if !(0..=255).contains(&num) {
                        let start_col = mat.start() + 1;
                        let end_col = mat.end() + 1;

                        let diagnostic = Diagnostic::new(
                            "SC2152",
                            Severity::Error,
                            format!("Exit code must be 0-255. {} will be truncated", num),
                            Span::new(line_num, start_col, line_num, end_col),
                        );

                        result.add(diagnostic);
                    }
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2152_exit_256() {
        let code = "exit 256";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2152_exit_1000() {
        let code = "exit 1000";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2152_exit_negative() {
        let code = "exit -1";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2152_exit_0_ok() {
        let code = "exit 0";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2152_exit_255_ok() {
        let code = "exit 255";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2152_exit_1_ok() {
        let code = "exit 1";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2152_comment_ok() {
        let code = "# exit 1000";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2152_multiple() {
        let code = "exit 256\nexit 1000";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2152_exit_100_ok() {
        let code = "exit 100";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2152_exit_no_arg_ok() {
        let code = "exit";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2152_gh438_prose_not_flagged() {
        for code in [
            "echo exit 256",
            "\t@# exit 999 is prose",
            "\t: exit 300 here",
        ] {
            assert_eq!(check(code).diagnostics.len(), 0, "false positive: {code:?}");
        }
    }

    #[test]
    fn test_sc2152_gh438_command_position_flagged() {
        for code in ["exit 256", "\t@exit 256", "true && exit 256"] {
            assert_eq!(check(code).diagnostics.len(), 1, "missed: {code:?}");
        }
    }
}
