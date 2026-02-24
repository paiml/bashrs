// SC1115: Space between # and ! in shebang
//
// The first line has a space between # and ! (e.g., `# !/bin/bash`).
// The shebang must be exactly `#!` with no space between them.
//
// Examples:
// Bad:
//   # !/bin/bash
//   # !/usr/bin/env sh
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

    // Look for `# !` followed by a path-like pattern
    if let Some(after_hash) = trimmed.strip_prefix('#') {
        // Already a proper shebang
        if after_hash.starts_with('!') {
            return result;
        }

        // Check for `# !` followed by a path
        let after_hash_trimmed = after_hash.trim_start_matches(' ').trim_start_matches('\t');
        if after_hash_trimmed.starts_with("!/") {
            let diagnostic = Diagnostic::new(
                "SC1115",
                Severity::Error,
                "Use #! for the shebang, not # !. Remove the space between # and !.".to_string(),
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

    // Detection tests
    #[test]
    fn test_sc1115_space_between_hash_bang() {
        let code = "# !/bin/bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1115");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1115_space_between_env() {
        let code = "# !/usr/bin/env sh\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1115_multiple_spaces() {
        let code = "#  !/bin/sh\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1115_tab_between() {
        let code = "#\t!/bin/bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    // False-positive avoidance tests
    #[test]
    fn test_sc1115_correct_shebang_ok() {
        let code = "#!/bin/bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1115_regular_comment_ok() {
        let code = "# This is a comment\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1115_no_shebang_ok() {
        let code = "echo hello\necho world";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1115_comment_with_exclamation_ok() {
        let code = "# ! important note\necho hello";
        let result = check(code);
        // Does not start with !/ so not a path
        assert_eq!(result.diagnostics.len(), 0);
    }

    // Edge case tests
    #[test]
    fn test_sc1115_empty_source() {
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1115_hash_alone() {
        let code = "#\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1115_second_line_ok() {
        let code = "echo hello\n# !/bin/bash";
        let result = check(code);
        // Only check first line
        assert_eq!(result.diagnostics.len(), 0);
    }
}
