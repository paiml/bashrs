// SC1100: Unicode dash used instead of minus/hyphen.
//
// Unicode dashes (en dash, em dash, minus sign) look similar to the ASCII
// hyphen-minus (-) but are not valid shell operators. This typically happens
// when copying commands from formatted documents, web pages, or word processors.
//
// Examples:
// Bad:
//   grep \u{2013}i pattern file    (en dash instead of -)
//   test \u{2014}f /etc/passwd     (em dash instead of -)
//   echo $((5 \u{2212} 3))        (unicode minus instead of -)
//
// Good:
//   grep -i pattern file
//   test -f /etc/passwd
//   echo $((5 - 3))
//
// Fix: Delete the unicode dash and retype as a regular minus sign (-)

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Unicode characters that look like minus but aren't
const UNICODE_DASHES: [char; 3] = [
    '\u{2013}', // EN DASH
    '\u{2014}', // EM DASH
    '\u{2212}', // MINUS SIGN
];

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        // Skip comment lines
        if line.trim_start().starts_with('#') {
            continue;
        }

        if UNICODE_DASHES.iter().any(|&dash| line.contains(dash)) {
            let diagnostic = Diagnostic::new(
                "SC1100",
                Severity::Error,
                "This is a unicode dash. Delete and retype as a regular minus sign",
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
    fn test_sc1100_detects_en_dash() {
        let script = "grep \u{2013}i pattern file";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1100");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
        assert!(result.diagnostics[0].message.contains("unicode dash"));
    }

    #[test]
    fn test_sc1100_detects_em_dash() {
        let script = "test \u{2014}f /etc/passwd";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1100_detects_unicode_minus() {
        let script = "echo $((5 \u{2212} 3))";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1100_no_unicode_dashes() {
        let script = "#!/bin/sh\ngrep -i pattern file\ntest -f /etc/passwd\necho $((5 - 3))";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1100_skips_comments() {
        let script = "# This has an en dash \u{2013} in a comment";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1100_multiple_dashes_on_one_line() {
        // Only one diagnostic per line even with multiple dashes
        let script = "cmd \u{2013}a \u{2014}b";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1100_empty_source() {
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1100_regular_hyphens_ok() {
        let script = "cmd -a -b --long-option -- arg";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
