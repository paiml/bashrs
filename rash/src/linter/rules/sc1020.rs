//! SC1020: Missing space before closing `]`
//!
//! Detects `[ expr]` without a space before the closing `]` in test
//! commands. The `]` must be a separate argument to `[`.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! [ -f file.txt]
//! [ "$x" = "y"]
//! ```
//!
//! Good:
//! ```bash
//! [ -f file.txt ]
//! [ "$x" = "y" ]
//! ```

use crate::linter::rules::posix_bracket;
use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim();

        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        for diag in line_diagnostics(line, line_num) {
            result.add(diag);
        }
    }

    result
}

/// One finding per `[ … ]` test whose closing bracket has no blank before it.
///
/// GH-226: this used to scan for any `[`, so `[[`, glob character classes,
/// array subscripts and case patterns all looked like tests. `posix_bracket`
/// applies the POSIX word rules instead.
fn line_diagnostics(line: &str, line_num: usize) -> Vec<Diagnostic> {
    let bytes = line.as_bytes();
    posix_bracket::openers(line)
        .into_iter()
        .filter_map(|open| posix_bracket::close_of(line, open).map(|close| (open, close)))
        .filter(|&(open, close)| missing_space_before_close(bytes, open, close))
        .map(|(_, close)| {
            let col = close + 1;
            Diagnostic::new(
                "SC1020",
                Severity::Error,
                "Missing space before closing ] in test expression",
                Span::new(line_num, col, line_num, col + 1),
            )
        })
        .collect()
}

/// True when `]` is preceded by a non-blank and the test is not empty.
fn missing_space_before_close(bytes: &[u8], open: usize, close: usize) -> bool {
    let Some(prev) = close.checked_sub(1).map(|p| bytes[p]) else {
        return false;
    };
    if prev.is_ascii_whitespace() {
        return false;
    }
    // `[ ]` has nothing to complain about; the defect needs actual content.
    bytes
        .get(open + 1..close)
        .is_some_and(|inner| inner.iter().any(|b| !b.is_ascii_whitespace()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1020_missing_space_before_close() {
        let result = check("[ -f file.txt]");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1020");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1020_proper_spacing_ok() {
        let result = check("[ -f file.txt ]");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1020_string_comparison() {
        let result = check(r#"[ "$x" = "y"]"#);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1020_array_subscript_not_flagged() {
        let result = check("echo ${arr[0]}");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1020_double_bracket_not_matched() {
        // Double brackets have different parsing rules
        let result = check("[[ -f file.txt ]]");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1020_comment_not_flagged() {
        let result = check("# [ -f file.txt]");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1020_if_context() {
        let result = check("if [ -f file.txt]; then");
        assert_eq!(result.diagnostics.len(), 1);
    }
}
