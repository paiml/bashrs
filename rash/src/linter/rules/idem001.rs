//! IDEM001: Non-idempotent mkdir
//!
//! **Rule**: Detect `mkdir` without `-p` flag
//!
//! **Why this matters**:
//! `mkdir` without `-p` fails if directory exists, making scripts non-idempotent.
//! Re-running the script will fail instead of succeeding.
//!
//! **Auto-fix**: Add `-p` flag
//!
//! ## Examples
//!
//! ❌ **BAD** (non-idempotent):
//! ```bash
//! mkdir /app/releases
//! ```
//!
//! ✅ **GOOD** (idempotent):
//! ```bash
//! mkdir -p /app/releases
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for mkdir without -p flag
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();

        // Skip comment lines entirely (Issue #107)
        if trimmed.starts_with('#') {
            continue;
        }

        // Strip inline comments before checking (Issue #107)
        let code_only = if let Some(pos) = line.find('#') {
            // Make sure we're not in a quoted string
            let before_hash = &line[..pos];
            let single_quotes = before_hash.matches('\'').count();
            let double_quotes = before_hash.matches('"').count();
            // If quotes are balanced, treat # as comment start
            if single_quotes % 2 == 0 && double_quotes % 2 == 0 {
                &line[..pos]
            } else {
                line
            }
        } else {
            line
        };

        // Look for mkdir without -p
        if code_only.contains("mkdir ") && !code_only.contains("mkdir -p") {
            if let Some(col) = code_only.find("mkdir ") {
                let span = Span::new(line_num + 1, col + 1, line_num + 1, col + 6);

                let fix = Fix::new_with_assumptions(
                    "mkdir -p",
                    vec!["Directory creation failure is not a critical error".to_string()],
                );

                let diag = Diagnostic::new(
                    "IDEM001",
                    Severity::Warning,
                    "Non-idempotent mkdir - add -p flag (SAFE-WITH-ASSUMPTIONS)",
                    span,
                )
                .with_fix(fix);

                result.add(diag);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_IDEM001_detects_mkdir_without_p() {
        let script = "mkdir /app/releases";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "IDEM001");
        assert_eq!(diag.severity, Severity::Warning);
    }

    #[test]
    fn test_IDEM001_no_warning_with_p_flag() {
        let script = "mkdir -p /app/releases";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_IDEM001_provides_fix() {
        let script = "mkdir /tmp/foo";
        let result = check(script);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, "mkdir -p");
    }

    #[test]
    fn test_IDEM001_multiple_mkdir() {
        let script = "mkdir /a\nmkdir /b";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_IDEM001_107_skip_comment_lines() {
        // Issue #107: Should not flag mkdir in comments
        let script = "# Safe mkdir with path validation\nmkdir -p /tmp";
        let result = check(script);

        // Should not flag the comment, and mkdir -p is fine
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_IDEM001_107_skip_inline_comments() {
        // Issue #107: Should not flag mkdir in inline comments
        let script = "echo 'hello' # mkdir /tmp would create dir";
        let result = check(script);

        // mkdir is in a comment, should not be flagged
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_IDEM001_107_actual_command_still_flagged() {
        // Actual mkdir command should still be flagged
        let script = "mkdir /tmp/foo # create directory";
        let result = check(script);

        // The actual mkdir should be flagged
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_IDEM001_107_hash_in_string_not_comment() {
        // Hash inside quotes is not a comment
        let script = r#"mkdir "/path/with#hash""#;
        let result = check(script);

        // mkdir without -p should be flagged even with # in path
        assert_eq!(result.diagnostics.len(), 1);
    }
}
