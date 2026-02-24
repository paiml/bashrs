//! SC1117: Backslash is literal in double quotes for unknown escape sequences
//!
//! In double-quoted strings, only `\\`, `\$`, `\"`, `` \` ``, and `\!` are
//! special escapes. Other sequences like `\a`, `\b`, `\v`, `\f` are literal
//! backslash + letter.

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Characters that are valid escape sequences in double quotes
const VALID_DOUBLE_QUOTE_ESCAPES: &[u8] = b"$\\'\"`!n";

/// Check for non-special backslash escapes inside double-quoted strings
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
        if bytes[i] == b'\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            i += 1;
            continue;
        }

        if in_single_quote {
            i += 1;
            continue;
        }

        if bytes[i] == b'"' {
            in_double_quote = !in_double_quote;
            i += 1;
            continue;
        }

        if in_double_quote && bytes[i] == b'\\' && i + 1 < len {
            let next = bytes[i + 1];
            if next.is_ascii_alphabetic() && !VALID_DOUBLE_QUOTE_ESCAPES.contains(&next) {
                let esc_char = next as char;
                let diagnostic = Diagnostic::new(
                    "SC1117",
                    Severity::Info,
                    format!(
                        "Backslash is literal in \"\\{}\". Escape sequences like \\{} are not recognized in double quotes",
                        esc_char, esc_char
                    ),
                    Span::new(line_num, i + 1, line_num, i + 3),
                );
                result.add(diagnostic);
            }
            i += 2;
            continue;
        }

        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1117_unknown_escape_a() {
        let script = r#"echo "hello\aworld""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1117");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("\\a"));
    }

    #[test]
    fn test_sc1117_unknown_escape_v() {
        let script = r#"echo "hello\vworld""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1117_known_escape_n_not_flagged() {
        let script = r#"echo "hello\nworld""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1117_known_escape_dollar_not_flagged() {
        let script = r#"echo "price is \$5""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1117_known_escape_backslash_not_flagged() {
        let script = r#"echo "path\\dir""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1117_outside_quotes_not_flagged() {
        let script = r"echo hello\aworld";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1117_single_quotes_not_flagged() {
        let script = r"echo 'hello\aworld'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
