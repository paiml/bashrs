//! SC1003: Want to escape a single quote? Use `'don'\''t'` or `'don'"'"'t'`
//!
//! Detects broken single-quote patterns like `'don't'` where a word character
//! appears on both sides of what looks like a misplaced single quote.

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for broken single-quote escaping patterns
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

    // Track single-quote regions: find sequences of quotes
    // We look for the pattern: '<word_chars>'<word_chars>'
    // which suggests someone wrote 'don't' instead of 'don'\''t'
    let mut i = 0;
    while i < len {
        if bytes[i] == b'\'' {
            // Found opening quote - scan for the pattern
            if let Some(issue_col) = find_broken_quote_pattern(bytes, i, len) {
                let diagnostic = Diagnostic::new(
                    "SC1003",
                    Severity::Warning,
                    "Want to escape a single quote? Use '\\'' or '\"'\"'",
                    Span::new(line_num, issue_col + 1, line_num, issue_col + 2),
                );
                result.add(diagnostic);
                // Skip past this match to avoid duplicate reports
                i = issue_col + 1;
                continue;
            }
        }
        i += 1;
    }
}

/// Look for pattern: 'word_chars'word_chars' starting at position `start`.
/// Returns the column of the middle (problematic) quote if found.
fn find_broken_quote_pattern(bytes: &[u8], start: usize, len: usize) -> Option<usize> {
    // bytes[start] == b'\''
    // Look for: '<alphanum_chars>'<alphanum_chars>'
    let mut i = start + 1;

    // Scan word chars after opening quote
    let word_start = i;
    while i < len && is_word_char(bytes[i]) {
        i += 1;
    }
    if i == word_start || i >= len {
        return None;
    }

    // Must hit a single quote (the middle one)
    if bytes[i] != b'\'' {
        return None;
    }
    let middle_quote = i;

    // Character before the middle quote must be a word char (already guaranteed)
    // Character after the middle quote must be a word char
    i += 1;
    if i >= len || !is_word_char(bytes[i]) {
        return None;
    }

    // Scan word chars after the middle quote
    while i < len && is_word_char(bytes[i]) {
        i += 1;
    }

    // Must end with a closing single quote
    if i >= len || bytes[i] != b'\'' {
        return None;
    }

    Some(middle_quote)
}

fn is_word_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1003_broken_single_quote() {
        // 'don't' has the pattern: '<word>'<word>'
        let script = "echo 'don't'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1003");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sc1003_correct_escaping() {
        let script = r"echo 'don'\''t'";
        let result = check(script);
        // The '\'' pattern breaks out of single quotes properly
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1003_no_false_positive_normal_quotes() {
        let script = "echo 'hello world'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1003_comment_line_skipped() {
        let script = "# echo 'don't'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1003_wont_pattern() {
        // 'won't' has the pattern: '<word>'<word>'
        let script = "echo 'won't'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
