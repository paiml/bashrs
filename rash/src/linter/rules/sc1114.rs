// SC1114: Leading spaces before shebang
//
// The shebang line has whitespace before the #! sequence. The kernel requires
// the shebang to be the very first bytes of the file, so leading spaces or
// tabs will prevent it from being recognized.
//
// Examples:
// Bad:
//   #!/bin/bash               // Space before #!
//   	#!/bin/bash               // Tab before #!
//
// Good:
//   #!/bin/bash                // No leading whitespace

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let first_line = match source.lines().next() {
        Some(line) => line,
        None => return result,
    };

    // Check if the line has leading whitespace followed by #!
    let trimmed = first_line.trim_start();
    if trimmed.starts_with("#!") && first_line != trimmed {
        let diagnostic = Diagnostic::new(
            "SC1114",
            Severity::Error,
            "Leading spaces before the shebang. The #! must be the first two bytes of the file."
                .to_string(),
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
    fn test_sc1114_leading_space() {
        let code = " #!/bin/bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1114");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1114_leading_spaces() {
        let code = "   #!/bin/bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1114_leading_tab() {
        let code = "\t#!/bin/bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1114_mixed_whitespace() {
        let code = " \t #!/usr/bin/env bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    // False-positive avoidance tests
    #[test]
    fn test_sc1114_correct_shebang_ok() {
        let code = "#!/bin/bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1114_no_shebang_ok() {
        let code = "echo hello\necho world";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1114_comment_ok() {
        let code = "  # just a comment\necho hello";
        let result = check(code);
        // Regular comment with spaces is fine (not a shebang)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1114_indented_code_ok() {
        let code = "  echo hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    // Edge case tests
    #[test]
    fn test_sc1114_empty_source() {
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1114_env_form_with_space() {
        let code = " #!/usr/bin/env sh\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1114_shebang_on_second_line_ok() {
        // Whitespace before #! only matters on first line
        let code = "echo hello\n #!/bin/bash";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
