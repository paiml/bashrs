// SC1018: Unicode non-breaking space (U+00A0) used instead of regular space.
//
// Non-breaking spaces look identical to regular spaces but are not valid
// whitespace in shell syntax. This typically happens when copying scripts
// from web pages, PDFs, or word processors.
//
// Examples:
// Bad:
//   echo\u{00a0}"hello"   (non-breaking space between echo and "hello")
//
// Good:
//   echo "hello"          (regular space)
//
// Fix: Delete the non-breaking space and retype as a regular space

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.contains('\u{00a0}') {
            let diagnostic = Diagnostic::new(
                "SC1018",
                Severity::Error,
                "This is a unicode non-breaking space. Delete and retype it",
                Span::new(line_num, 1, line_num, line.len() + 1),
            );
            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1018_detects_nbsp() {
        let script = "echo\u{00a0}hello";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1018");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
        assert!(result.diagnostics[0].message.contains("non-breaking space"));
    }

    #[test]
    fn test_sc1018_no_nbsp() {
        let script = "#!/bin/sh\necho hello\necho world\n";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1018_multiple_nbsp() {
        let script = "echo\u{00a0}hello\nif\u{00a0}true; then\n  echo ok\nfi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc1018_nbsp_in_string() {
        // Even inside a string, NBSP is suspicious and should be flagged
        let script = "echo \"hello\u{00a0}world\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1018_regular_spaces_ok() {
        let script = "echo   hello   world";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1018_empty_source() {
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }
}
