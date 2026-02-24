//! SC1012: `\t` is literal in single quotes, not a tab character
//!
//! Inside single quotes, `\t`, `\n`, and `\r` are literal backslash+letter,
//! not escape sequences. Use `$'\t'` or double quotes for actual escapes.

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for escape-like sequences inside single-quoted strings
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
    let mut in_single_quote = false;
    let mut i = 0;

    while i < len {
        if bytes[i] == b'\'' {
            in_single_quote = !in_single_quote;
            i += 1;
            continue;
        }

        if in_single_quote && bytes[i] == b'\\' && i + 1 < len {
            let next = bytes[i + 1];
            if next == b't' || next == b'n' || next == b'r' {
                let esc_char = next as char;
                let diagnostic = Diagnostic::new(
                    "SC1012",
                    Severity::Info,
                    format!(
                        "\\{} is just literal '\\{}' in single quotes. Use $'\\{}' or double quotes for escape sequence",
                        esc_char, esc_char, esc_char
                    ),
                    Span::new(line_num, i + 1, line_num, i + 3),
                );
                result.add(diagnostic);
                i += 2;
                continue;
            }
        }

        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1012_tab_in_single_quotes() {
        let script = r"echo 'hello\tworld'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1012");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("\\t"));
    }

    #[test]
    fn test_sc1012_newline_in_single_quotes() {
        let script = r"echo 'hello\nworld'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("\\n"));
    }

    #[test]
    fn test_sc1012_carriage_return_in_single_quotes() {
        let script = r"echo 'hello\rworld'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("\\r"));
    }

    #[test]
    fn test_sc1012_no_false_positive_double_quotes() {
        let script = r#"echo "hello\tworld""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1012_no_false_positive_no_escape() {
        let script = "echo 'hello world'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1012_multiple_escapes() {
        let script = r"echo 'col1\tcol2\tcol3'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
