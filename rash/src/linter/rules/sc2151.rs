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

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static RETURN_OUT_OF_RANGE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: return <number> where number > 255 or < 0
    Regex::new(r"\breturn\s+(-[0-9]+|[0-9]{3,})").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for return with out-of-range values
        for mat in RETURN_OUT_OF_RANGE.find_iter(line) {
            let matched = mat.as_str();

            // Extract the number
            if let Some(num_str) = matched.strip_prefix("return").map(|s| s.trim()) {
                if let Ok(num) = num_str.parse::<i32>() {
                    // Check if out of range 0-255
                    if !(0..=255).contains(&num) {
                        let start_col = mat.start() + 1;
                        let end_col = mat.end() + 1;

                        let diagnostic = Diagnostic::new(
                            "SC2151",
                            Severity::Error,
                            format!(
                                "Only values 0-255 can be returned. {} will be truncated",
                                num
                            ),
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
