//! SC1007: Remove space after = in variable assignment
//!
//! Detects `VAR = value` patterns where spaces surround the `=` in what
//! looks like a variable assignment. In shell, `VAR = value` runs `VAR`
//! as a command with `=` and `value` as arguments.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! FOO = bar
//! MY_VAR = "hello"
//! ```
//!
//! Good:
//! ```bash
//! FOO=bar
//! MY_VAR="hello"
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim();

        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        // Skip lines that are clearly test/conditional contexts
        // [ $x = $y ], [[ $x = $y ]], test $x = $y, if/while/until
        if trimmed.starts_with('[')
            || trimmed.starts_with("if ")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("until ")
            || trimmed.starts_with("elif ")
            || trimmed.starts_with("test ")
            || trimmed.contains("[ ")
            || trimmed.contains("[[ ")
        {
            continue;
        }

        // Skip lines containing == (comparison, not assignment)
        if trimmed.contains("==") {
            continue;
        }

        // Look for pattern: identifier followed by space(s) then = then space(s) then value
        // Must start at the beginning of the (trimmed) line to be an assignment context
        let bytes = trimmed.as_bytes();
        let mut i = 0;

        // Check for valid identifier start
        if i < bytes.len() && (bytes[i].is_ascii_alphabetic() || bytes[i] == b'_') {
            i += 1;
            // Continue through identifier chars
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let ident_end = i;

            // Check for space(s) before =
            let has_space_before = i < bytes.len() && bytes[i] == b' ';
            while i < bytes.len() && bytes[i] == b' ' {
                i += 1;
            }

            // Check for = (but not ==)
            if i < bytes.len() && bytes[i] == b'=' {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    continue; // This is ==, skip
                }
                let eq_pos = i;
                i += 1;

                // Check for space(s) after =
                let has_space_after = i < bytes.len() && bytes[i] == b' ';

                if has_space_before || has_space_after {
                    // Skip if identifier looks like a command (common words)
                    let ident = &trimmed[..ident_end];
                    if matches!(
                        ident,
                        "echo" | "printf" | "return" | "exit" | "export" | "local" | "readonly"
                    ) {
                        continue;
                    }

                    let col = line.find(trimmed).unwrap_or(0) + eq_pos + 1;
                    result.add(Diagnostic::new(
                        "SC1007",
                        Severity::Error,
                        "Remove space after = if this is intended as an assignment",
                        Span::new(line_num, col, line_num, col + 1),
                    ));
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1007_space_around_equals() {
        let result = check("FOO = bar");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1007");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1007_space_after_equals() {
        let result = check("FOO= bar");
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1007_space_before_equals() {
        let result = check("FOO =bar");
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1007_no_space_ok() {
        let result = check("FOO=bar");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1007_test_context_not_flagged() {
        let result = check("[ $x = $y ]");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1007_double_bracket_not_flagged() {
        let result = check("[[ $x = $y ]]");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1007_if_context_not_flagged() {
        let result = check("if [ $x = $y ]; then");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1007_double_equals_not_flagged() {
        let result = check("x == y");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1007_comment_not_flagged() {
        let result = check("# FOO = bar");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1007_multiple_assignments() {
        let script = "A = 1\nB = 2\n";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
