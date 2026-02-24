//! SC1079: This is actually an end quote, but it looks suspicious
//!
//! Detects closing double quote immediately followed by an alphanumeric
//! character, like `"foo"bar`, which is likely a quoting mistake.

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for suspicious end quotes followed by alphanumeric characters
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with('#') {
            continue;
        }

        check_line(line, line_num, &mut result);
    }

    result
}

fn check_line(line: &str, line_num: usize, result: &mut LintResult) {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut in_double_quote = false;
    let mut in_single_quote = false;

    let mut i = 0;
    while i < len {
        // Track single quotes (outside double quotes)
        if bytes[i] == b'\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            i += 1;
            continue;
        }

        if in_single_quote {
            i += 1;
            continue;
        }

        // Handle escaped characters inside double quotes
        if in_double_quote && bytes[i] == b'\\' && i + 1 < len {
            i += 2;
            continue;
        }

        if bytes[i] == b'"' {
            if in_double_quote {
                // Closing quote - check what follows
                if i + 1 < len && bytes[i + 1].is_ascii_alphanumeric() {
                    // Skip known patterns like "$var"s (pluralizing) which is intentional
                    // but flag other cases
                    let diagnostic = Diagnostic::new(
                        "SC1079",
                        Severity::Warning,
                        "This is actually an end quote, but it looks suspicious. Did you mean to concatenate?",
                        Span::new(line_num, i + 1, line_num, i + 3),
                    );
                    result.add(diagnostic);
                }
                in_double_quote = false;
            } else {
                in_double_quote = true;
            }
        }

        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1079_end_quote_followed_by_alpha() {
        let script = r#"echo "hello"world"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1079");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sc1079_normal_quote() {
        let script = r#"echo "hello" world"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1079_end_quote_followed_by_digit() {
        let script = r#"echo "test"123"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1079_comment_skipped() {
        let script = r#"# "hello"world"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1079_end_quote_followed_by_space() {
        let script = r#"echo "hello" "world""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
