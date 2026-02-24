//! SC1135: Use `\$` in double quotes rather than breaking the quote
//!
//! Detects patterns like `"foo"'$bar'"baz"` where quotes are broken
//! unnecessarily to avoid `$` expansion. Use `\$` inside double quotes instead.

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for unnecessary quote-breaking to avoid $ expansion
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

    // Look for pattern: "'$ or "'\$ suggesting quote break for dollar sign
    // Specifically: closing double-quote, opening single-quote, dollar sign
    let mut i = 0;
    while i + 2 < len {
        if bytes[i] == b'"' && bytes[i + 1] == b'\'' && bytes[i + 2] == b'$' {
            let diagnostic = Diagnostic::new(
                "SC1135",
                Severity::Info,
                "Use \\$ in double quotes rather than breaking out of the quote",
                Span::new(line_num, i + 1, line_num, i + 4),
            );
            result.add(diagnostic);
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1135_quote_break_for_dollar() {
        let script = r#"echo "price is "'$5'"per item""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1135");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
    }

    #[test]
    fn test_sc1135_no_quote_break() {
        let script = r#"echo "price is \$5 per item""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1135_normal_quotes() {
        let script = r#"echo "hello" 'world'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1135_comment_skipped() {
        let script = r#"# echo "foo"'$bar'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
