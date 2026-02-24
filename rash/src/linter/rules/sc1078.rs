//! SC1078: Did you forget to close this double-quoted string?
//!
//! Detects lines with an odd number of unescaped double quotes,
//! suggesting an unclosed string.

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for unclosed double-quoted strings
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with('#') {
            continue;
        }

        // Skip heredoc markers and lines that are part of heredocs
        if trimmed.starts_with("<<") {
            continue;
        }

        let unescaped_quote_count = count_unescaped_double_quotes(line);

        #[allow(clippy::manual_is_multiple_of)]
        if unescaped_quote_count % 2 != 0 {
            // Find the position of the last unescaped quote
            let col = find_last_unescaped_double_quote(line);
            let diagnostic = Diagnostic::new(
                "SC1078",
                Severity::Error,
                "Did you forget to close this double-quoted string?",
                Span::new(line_num, col + 1, line_num, col + 2),
            );
            result.add(diagnostic);
        }
    }

    result
}

fn count_unescaped_double_quotes(line: &str) -> usize {
    let bytes = line.as_bytes();
    let mut count = 0;
    let mut in_single_quote = false;

    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\'' && !in_single_quote {
            in_single_quote = true;
            i += 1;
            continue;
        }
        if bytes[i] == b'\'' && in_single_quote {
            in_single_quote = false;
            i += 1;
            continue;
        }
        if in_single_quote {
            i += 1;
            continue;
        }

        if bytes[i] == b'"' {
            // Check if escaped
            if i > 0 && bytes[i - 1] == b'\\' {
                // Check for double-escape (\\")
                if i > 1 && bytes[i - 2] == b'\\' {
                    count += 1; // \\" means the backslash is escaped, quote is real
                }
                // Otherwise it's escaped, skip
            } else {
                count += 1;
            }
        }
        i += 1;
    }

    count
}

fn find_last_unescaped_double_quote(line: &str) -> usize {
    let bytes = line.as_bytes();
    let mut last_pos = 0;
    let mut in_single_quote = false;

    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\'' && !in_single_quote {
            in_single_quote = true;
            i += 1;
            continue;
        }
        if bytes[i] == b'\'' && in_single_quote {
            in_single_quote = false;
            i += 1;
            continue;
        }
        if in_single_quote {
            i += 1;
            continue;
        }

        if bytes[i] == b'"' && (i == 0 || bytes[i - 1] != b'\\') {
            last_pos = i;
        }
        i += 1;
    }

    last_pos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1078_unclosed_double_quote() {
        let script = "echo \"hello world";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1078");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1078_closed_double_quote() {
        let script = "echo \"hello world\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1078_escaped_quote_not_flagged() {
        let script = r#"echo "hello \" world""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1078_comment_skipped() {
        let script = "# echo \"unclosed";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1078_single_quote_inside_not_counted() {
        // Single-quoted section containing " should not affect count
        let script = "echo 'he said \"hi\"'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
