// SC2083: Don't add spaces after the shebang (#!)
//
// The shebang must be exactly #! with no spaces before the interpreter path.
// Spaces after #! prevent proper interpreter detection.
//
// Examples:
// Bad:
//   #! /bin/bash                 // Space after #!
//   #!  /usr/bin/env python      // Multiple spaces
//
// Good:
//   #!/bin/bash                  // No space
//   #!/usr/bin/env python        // Correct
//
// Impact: Script won't execute with correct interpreter

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Only check first line
    if let Some(first_line) = source.lines().next() {
        let trimmed = first_line.trim_start();

        // Check if it's a shebang with spaces after #!
        if trimmed.starts_with("#! ") || trimmed.starts_with("#!\t") {
            let diagnostic = Diagnostic::new(
                "SC2083",
                Severity::Error,
                "Don't add spaces after the shebang. Use #!/bin/bash, not #! /bin/bash".to_string(),
                Span::new(1, 1, 1, first_line.len() + 1),
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
    fn test_sc2083_space_after_shebang() {
        let code = "#! /bin/bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2083_multiple_spaces() {
        let code = "#!  /usr/bin/env python\nprint('hello')";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2083_tab_after_shebang() {
        let code = "#!\t/bin/sh\necho test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2083_correct_shebang_ok() {
        let code = "#!/bin/bash\necho hello";
        let result = check(code);
        // No space, correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2083_env_shebang_ok() {
        let code = "#!/usr/bin/env python\nprint('hello')";
        let result = check(code);
        // Correct format
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2083_no_shebang_ok() {
        let code = "echo hello\necho world";
        let result = check(code);
        // No shebang at all
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2083_comment_not_shebang_ok() {
        let code = "# This is a comment\necho hello";
        let result = check(code);
        // Regular comment, not shebang
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2083_shebang_not_first_line_ok() {
        let code = "echo test\n#! /bin/bash";
        let result = check(code);
        // Shebang on second line doesn't count
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2083_with_args() {
        let code = "#! /bin/bash -e\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2083_whitespace_prefix() {
        let code = "  #! /bin/bash\necho hello";
        let result = check(code);
        // Whitespace before shebang
        assert_eq!(result.diagnostics.len(), 1);
    }
}
