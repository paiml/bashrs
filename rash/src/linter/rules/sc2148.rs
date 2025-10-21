// SC2148: Tips depend on the shebang - add explicit shebang.
//
// Scripts should have an explicit shebang to indicate which interpreter to use.
// Without a shebang, the script's behavior depends on how it's invoked.
//
// Examples:
// Bad:
//   (no shebang at start of file)
//   echo "Hello World"
//
// Good:
//   #!/bin/sh
//   echo "Hello World"
//
//   #!/bin/bash
//   echo "Hello ${USER}"
//
//   #!/usr/bin/env python3
//   print("Hello")
//
// Impact: Portability - script may run with wrong interpreter

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Only check first line for shebang
    if let Some(first_line) = source.lines().next() {
        // Skip if it's a blank line or comment that's not a shebang
        if first_line.trim().is_empty() {
            return result;
        }

        // Check if first line is a shebang
        if !first_line.starts_with("#!") {
            // File doesn't start with shebang
            let diagnostic = Diagnostic::new(
                "SC2148",
                Severity::Info,
                "Add a shebang to indicate which interpreter should be used".to_string(),
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
    fn test_sc2148_missing_shebang() {
        let code = r#"echo "Hello World""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("shebang"));
    }

    #[test]
    fn test_sc2148_with_shebang_bash_ok() {
        let code = r#"#!/bin/bash
echo "Hello World""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2148_with_shebang_sh_ok() {
        let code = r#"#!/bin/sh
echo "Hello World""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2148_with_env_shebang_ok() {
        let code = r#"#!/usr/bin/env bash
echo "Hello World""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2148_comment_not_shebang() {
        let code = r#"# This is a script
echo "Hello""#;
        let result = check(code);
        // First line is comment, not shebang
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2148_empty_file() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2148_blank_first_line() {
        let code = r#"
echo "Hello""#;
        let result = check(code);
        // Blank first line, no warning
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2148_python_shebang_ok() {
        let code = r#"#!/usr/bin/env python3
print("Hello")"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2148_variable_assignment_without_shebang() {
        let code = r#"FOO=bar
echo "$FOO""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2148_function_without_shebang() {
        let code = r#"foo() {
  echo "bar"
}"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
