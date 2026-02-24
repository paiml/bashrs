// SC1084: Use #! not !# on the shebang
//
// The shebang must start with #! not !#. The reversed form is a common typo
// that will cause the script to fail to execute.
//
// Examples:
// Bad:
//   !#/bin/bash
//   !# /usr/bin/env sh
//
// Good:
//   #!/bin/bash
//   #!/usr/bin/env sh

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let first_line = match source.lines().next() {
        Some(line) => line,
        None => return result,
    };

    let trimmed = first_line.trim_start();

    if trimmed.starts_with("!#") {
        let diagnostic = Diagnostic::new(
            "SC1084",
            Severity::Error,
            "Use #! for the shebang, not !#. The characters are in the wrong order.".to_string(),
            Span::new(1, 1, 1, first_line.len() + 1),
        );
        result.add(diagnostic);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // Detection tests
    #[test]
    fn test_sc1084_reversed_shebang() {
        let code = "!#/bin/bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1084");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1084_reversed_with_space() {
        let code = "!# /usr/bin/env bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1084_reversed_sh() {
        let code = "!#/bin/sh\necho test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    // False-positive avoidance tests
    #[test]
    fn test_sc1084_correct_shebang_ok() {
        let code = "#!/bin/bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1084_no_shebang_ok() {
        let code = "echo hello\necho world";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1084_comment_ok() {
        let code = "# just a comment\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    // Edge case tests
    #[test]
    fn test_sc1084_empty_source() {
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1084_reversed_not_on_first_line() {
        let code = "echo hello\n!#/bin/bash";
        let result = check(code);
        // Only check first line
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1084_exclamation_in_code_ok() {
        let code = "if ! grep -q pattern file; then echo missing; fi";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
