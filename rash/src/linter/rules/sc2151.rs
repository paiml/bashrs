// SC2151: Only one integer 0-255 can be returned. Use stdout for other data.
//
// Shell functions and scripts can only return exit codes 0-255.
// Attempting to return other values will be truncated modulo 256.
//
// Examples:
// Bad:
//   return 256              // Returns 0 (256 % 256)
//   return 1000             // Returns 232 (1000 % 256)
//   return -1               // Returns 255 (-1 % 256)
//
// Good:
//   return 0                // Success
//   return 1                // Failure
//   echo "1000"             // Use stdout for data
//   return 0
//
// Impact: Return value truncated, unexpected behavior

use crate::linter::command_keyword::command_keyword_hits;
use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // GH-438: only `return` as the NAME of a simple command counts, not prose or arguments.
    for hit in command_keyword_hits(source, "return") {
        let Some((arg, end_line, arg_end)) = hit.first_arg else {
            continue;
        };
        let Ok(num) = arg.parse::<i32>() else {
            continue;
        };
        if (0..=255).contains(&num) {
            continue;
        }
        result.add(Diagnostic::new(
            "SC2151",
            Severity::Error,
            format!(
                "Only values 0-255 can be returned. {} will be truncated",
                num
            ),
            Span::new(hit.line, hit.col, end_line, arg_end),
        ));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2151_return_256() {
        let code = "return 256";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("truncated"));
    }

    #[test]
    fn test_sc2151_return_1000() {
        let code = "return 1000";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2151_return_negative() {
        let code = "return -1";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2151_return_0_ok() {
        let code = "return 0";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2151_return_255_ok() {
        let code = "return 255";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2151_return_1_ok() {
        let code = "return 1";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2151_comment_ok() {
        let code = "# return 1000";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2151_multiple() {
        let code = r#"
return 256
return 1000
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2151_return_100_ok() {
        let code = "return 100";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2151_return_in_function() {
        let code = r#"
foo() {
  return 500
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}

#[cfg(test)]
mod tests_gh438 {
    use super::*;

    #[test]
    fn test_gh438_sc2151_echo_return_is_prose() {
        assert_eq!(check("echo return 256").diagnostics.len(), 0);
        assert_eq!(check("\t@# return 256").diagnostics.len(), 0);
    }

    #[test]
    fn test_gh438_sc2151_real_return_still_flagged() {
        assert_eq!(check("f() { return 256; }").diagnostics.len(), 1);
        assert_eq!(check("\t@return 1000").diagnostics.len(), 1);
    }
}
