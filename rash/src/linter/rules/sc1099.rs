//! SC1099: Missing space before `#` comment
//!
//! Detects `cmd#comment` where `#` is not preceded by a space and could be
//! misinterpreted. In shell, `#` starts a comment only when preceded by
//! whitespace or at the start of a line.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! echo hello#world
//! x=1#set x
//! ```
//!
//! Good:
//! ```bash
//! echo hello #world
//! x=1 #set x
//! echo "$#"  # parameter count
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim();

        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        let bytes = line.as_bytes();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut i = 0;

        while i < bytes.len() {
            let ch = bytes[i];

            // Track quoting state
            if ch == b'\'' && !in_double_quote {
                in_single_quote = !in_single_quote;
                i += 1;
                continue;
            }
            if ch == b'"' && !in_single_quote {
                in_double_quote = !in_double_quote;
                i += 1;
                continue;
            }

            // Skip escaped characters
            if ch == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }

            // Only check for # outside quotes
            if ch == b'#' && !in_single_quote && !in_double_quote && i > 0 {
                let prev = bytes[i - 1];

                // Skip $# (parameter count)
                if prev == b'$' {
                    i += 1;
                    continue;
                }

                // Skip ${# (string length prefix)
                if prev == b'{' && i >= 2 && bytes[i - 2] == b'$' {
                    i += 1;
                    continue;
                }

                // Skip if # is already preceded by whitespace (it's a proper comment)
                if prev == b' ' || prev == b'\t' {
                    // This is a proper comment start, stop scanning
                    break;
                }

                // Skip #! (shebang-like patterns)
                if i + 1 < bytes.len() && bytes[i + 1] == b'!' {
                    i += 1;
                    continue;
                }

                // This is a # not preceded by space and not in quotes
                let col = i + 1;
                result.add(Diagnostic::new(
                    "SC1099",
                    Severity::Warning,
                    "Add a space before # to make it a comment, or quote it for literal #",
                    Span::new(line_num, col, line_num, col + 1),
                ));
                break; // Only flag first occurrence per line
            }

            i += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1099_no_space_before_hash() {
        let result = check("echo hello#world");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1099");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sc1099_proper_comment_ok() {
        let result = check("echo hello # this is a comment");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1099_dollar_hash_ok() {
        let result = check("echo $#");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1099_string_length_ok() {
        let result = check("echo ${#var}");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1099_in_single_quotes_ok() {
        let result = check("echo 'hello#world'");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1099_in_double_quotes_ok() {
        let result = check(r#"echo "hello#world""#);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1099_line_starting_with_hash() {
        let result = check("# this is a comment");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1099_assignment_with_comment() {
        let result = check("x=1#comment");
        assert_eq!(result.diagnostics.len(), 1);
    }
}
