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

use crate::linter::command_keyword::command_keyword_hits;
use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // GH-438: only `exit` as the NAME of a simple command counts, not prose or arguments.
    for hit in command_keyword_hits(source, "exit") {
        let Some((arg, arg_end)) = hit.first_arg else {
            continue;
        };
        let Ok(num) = arg.parse::<i32>() else {
            continue;
        };
        if (0..=255).contains(&num) {
            continue;
        }
        result.add(Diagnostic::new(
            "SC2152",
            Severity::Error,
            format!("Exit code must be 0-255. {} will be truncated", num),
            Span::new(hit.line, hit.col, hit.line, arg_end),
        ));
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
}

#[cfg(test)]
mod tests_gh438 {
    use super::*;

    #[test]
    fn test_gh438_sc2152_echo_exit_is_prose() {
        assert_eq!(check("echo exit 999").diagnostics.len(), 0);
        assert_eq!(check("\t@# exit 999").diagnostics.len(), 0);
    }

    #[test]
    fn test_gh438_sc2152_real_exit_still_flagged() {
        assert_eq!(check("true && exit 999").diagnostics.len(), 1);
        assert_eq!(check("\t-exit 256").diagnostics.len(), 1);
    }
}
