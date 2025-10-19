//! SEC003: Unquoted find -exec {} Pattern
//!
//! **Rule**: Detect unquoted `{}` in `find -exec` commands
//!
//! **Why this matters**:
//! Filenames with spaces or special characters can break or execute unintended
//! commands when {} is not quoted in find -exec.
//!
//! **Auto-fix**: Safe (add quotes)
//!
//! ## Examples
//!
//! ❌ **UNSAFE**:
//! ```bash
//! find . -name "*.sh" -exec chmod +x {} \;
//! find /tmp -type f -exec rm {} \;
//! ```
//!
//! ✅ **SAFE** (auto-fixable):
//! ```bash
//! find . -name "*.sh" -exec chmod +x "{}" \;
//! find /tmp -type f -exec rm "{}" \;
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for unquoted {} in find -exec
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Look for find -exec pattern with unquoted {}
        if line.contains("find ") && line.contains("-exec") {
            // Check for {} that is NOT "{}"
            if let Some(col) = line.find(" {} ") {
                // Make sure it's not already quoted
                let before = &line[..col];
                if !before.ends_with('"') {
                    let span = Span::new(
                        line_num + 1,
                        col + 2,  // Space before {}
                        line_num + 1,
                        col + 4,  // {} is 2 chars
                    );

                    let diag = Diagnostic::new(
                        "SEC003",
                        Severity::Error,
                        "Unquoted {} in find -exec - filenames with spaces will break",
                        span,
                    )
                    .with_fix(Fix::new("\"{}\""));

                    result.add(diag);
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
    fn test_SEC003_detects_unquoted_find_exec() {
        let script = r#"find . -name "*.sh" -exec chmod +x {} \;"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC003");
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("Unquoted"));
    }

    #[test]
    fn test_SEC003_detects_unquoted_rm() {
        let script = "find /tmp -type f -exec rm {} \\;";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_SEC003_no_warning_with_quotes() {
        let script = r#"find . -name "*.sh" -exec chmod +x "{}" \;"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC003_provides_fix() {
        let script = "find . -exec cat {} \\;";
        let result = check(script);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, "\"{}\"");
    }

    #[test]
    fn test_SEC003_no_false_positive_no_find() {
        let script = "echo {} something";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }
}
