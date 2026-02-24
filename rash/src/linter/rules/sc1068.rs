//! SC1068: Don't put spaces around = in assignments
//!
//! Detects spaces around `=` in `let` and `declare`/`typeset` assignments.
//! Unlike SC1007 which handles bare assignments, this rule targets keyword-
//! prefixed assignments where spaces are also invalid.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! let x = 1
//! declare y = "hello"
//! typeset z = 42
//! ```
//!
//! Good:
//! ```bash
//! let x=1
//! declare y="hello"
//! typeset z=42
//! let "x = 1"  # Quoted expression is OK
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

        // Check for `let var = value` or `let var =value` or `let var= value`
        for prefix in &["let ", "declare ", "typeset ", "local "] {
            if !trimmed.starts_with(prefix) {
                continue;
            }

            let after_kw = &trimmed[prefix.len()..];

            // Skip if the assignment is inside quotes: let "x = 1"
            if after_kw.starts_with('"') || after_kw.starts_with('\'') {
                continue;
            }

            // Skip flags like declare -i, declare -r, etc.
            let rest = if after_kw.starts_with('-') {
                // Skip past flags
                if let Some(space_pos) = after_kw.find(' ') {
                    &after_kw[space_pos + 1..]
                } else {
                    continue;
                }
            } else {
                after_kw
            };

            // Now rest should be "var = value" or similar
            // Look for identifier followed by spaces around =
            let bytes = rest.as_bytes();
            let mut i = 0;

            // Skip identifier
            if i < bytes.len() && (bytes[i].is_ascii_alphabetic() || bytes[i] == b'_') {
                i += 1;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }

                // Check for space before =
                let has_space_before = i < bytes.len() && bytes[i] == b' ';
                while i < bytes.len() && bytes[i] == b' ' {
                    i += 1;
                }

                if i < bytes.len() && bytes[i] == b'=' {
                    if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                        continue; // ==, not assignment
                    }
                    i += 1;

                    let has_space_after = i < bytes.len() && bytes[i] == b' ';

                    if has_space_before || has_space_after {
                        let line_offset = line.find(trimmed).unwrap_or(0);
                        let col = line_offset + 1;
                        result.add(Diagnostic::new(
                            "SC1068",
                            Severity::Error,
                            format!(
                                "Don't put spaces around the = in '{}' assignments",
                                prefix.trim()
                            ),
                            Span::new(line_num, col, line_num, col + trimmed.len()),
                        ));
                    }
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
    fn test_sc1068_let_with_spaces() {
        let result = check("let x = 1");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1068");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1068_declare_with_spaces() {
        let result = check("declare y = hello");
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1068_typeset_with_spaces() {
        let result = check("typeset z = 42");
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1068_local_with_spaces() {
        let result = check("local var = value");
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1068_no_spaces_ok() {
        let result = check("let x=1");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1068_declare_no_spaces_ok() {
        let result = check("declare y=hello");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1068_quoted_expression_ok() {
        let result = check(r#"let "x = 1""#);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1068_declare_with_flag_ok() {
        let result = check("declare -i count=0");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1068_comment_not_flagged() {
        let result = check("# let x = 1");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1068_space_after_only() {
        let result = check("let x= 1");
        assert_eq!(result.diagnostics.len(), 1);
    }
}
