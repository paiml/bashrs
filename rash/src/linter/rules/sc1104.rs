// SC1104: Use #! not just ! on the shebang
//
// The first line starts with ! alone (not !#) and looks like it was intended
// as a shebang (e.g., !/bin/bash). The # is missing.
//
// Examples:
// Bad:
//   !/bin/bash
//   !/usr/bin/env sh
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

    // Must start with ! but not !# (that's SC1084)
    if !trimmed.starts_with('!') || trimmed.starts_with("!#") {
        return result;
    }

    // Check if it looks like a shebang path (!/bin/ or !/usr/)
    let after_bang = &trimmed[1..];
    if after_bang.starts_with("/bin/")
        || after_bang.starts_with("/usr/")
        || after_bang.starts_with("/sbin/")
    {
        let diagnostic = Diagnostic::new(
            "SC1104",
            Severity::Error,
            "Use #! for the shebang, not just !. Add the missing #.".to_string(),
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
    fn test_sc1104_missing_hash() {
        let code = "!/bin/bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1104");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1104_missing_hash_env() {
        let code = "!/usr/bin/env bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1104_missing_hash_sh() {
        let code = "!/bin/sh\necho test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    // False-positive avoidance tests
    #[test]
    fn test_sc1104_correct_shebang_ok() {
        let code = "#!/bin/bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1104_reversed_shebang_not_flagged() {
        // !# is SC1084, not SC1104
        let code = "!#/bin/bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1104_normal_code_ok() {
        let code = "echo hello\necho world";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1104_negation_in_code_ok() {
        // ! used as negation should not trigger
        let code = "! true";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    // Edge case tests
    #[test]
    fn test_sc1104_empty_source() {
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1104_bang_alone() {
        let code = "!\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1104_sbin_path() {
        let code = "!/sbin/nologin\n";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
