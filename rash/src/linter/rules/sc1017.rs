// SC1017: Literal carriage return found in source.
//
// Shell scripts should use Unix line endings (LF only).
// Literal carriage return characters (\r) indicate Windows line endings
// or copy-paste from Windows sources, which can cause subtle parsing issues.
//
// Examples:
// Bad:
//   echo "hello"\r\n    (file has Windows CRLF line endings)
//
// Good:
//   echo "hello"\n      (file has Unix LF line endings)
//
// Fix: Run script through tr -d '\r' or dos2unix

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Split on \n to preserve \r characters (str::lines() strips trailing \r)
    for (idx, line) in source.split('\n').enumerate() {
        let line_num = idx + 1;

        if line.contains('\r') {
            let diagnostic = Diagnostic::new(
                "SC1017",
                Severity::Warning,
                "Literal carriage return. Run script through tr -d '\\r'",
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
    fn test_sc1017_detects_carriage_return() {
        let script = "echo hello\r";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1017");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("carriage return"));
    }

    #[test]
    fn test_sc1017_no_carriage_return() {
        let script = "#!/bin/sh\necho hello\necho world\n";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1017_multiple_lines_with_cr() {
        // CRLF line endings: split('\n') preserves \r at end of each line
        let script = "echo a\r\necho b\r\necho c";
        let result = check(script);
        // "echo a\r" and "echo b\r" have \r, "echo c" does not
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc1017_cr_in_middle_of_line() {
        let script = "echo hel\rlo";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1017_empty_source() {
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1017_comment_with_cr() {
        // Even comments with \r are flagged -- it's a file encoding issue
        let script = "# comment\r";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
