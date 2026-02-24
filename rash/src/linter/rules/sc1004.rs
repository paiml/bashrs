//! SC1004: Backslash-linefeed is not valid in single quotes
//!
//! In single-quoted strings, a backslash at the end of the line does NOT
//! create a line continuation. It is literal `\`.

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for backslash at end of single-quoted strings
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();

    let mut in_single_quote = false;

    for (line_num, line) in lines.iter().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();

        if !in_single_quote && trimmed.starts_with('#') {
            continue;
        }

        for (i, ch) in line.chars().enumerate() {
            if ch == '\'' && !in_single_quote {
                in_single_quote = true;
            } else if ch == '\'' && in_single_quote {
                in_single_quote = false;
            }

            // Check if we're in a single-quoted region and this is a backslash
            // at the end of the line
            if in_single_quote && ch == '\\' && i == line.len() - 1 {
                let diagnostic = Diagnostic::new(
                    "SC1004",
                    Severity::Warning,
                    "This backslash+linefeed is literal in single quotes. It will not be a line continuation",
                    Span::new(line_num, i + 1, line_num, i + 2),
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
    fn test_sc1004_backslash_in_single_quote() {
        let script = "echo 'hello\\\nworld'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1004");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sc1004_no_backslash() {
        let script = "echo 'hello world'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1004_backslash_in_double_quote() {
        // Backslash continuation IS valid in double quotes - no warning
        let script = "echo \"hello\\\nworld\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1004_backslash_not_at_end() {
        let script = "echo 'hello\\tworld'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
