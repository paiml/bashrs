//! SEC001: Command Injection via eval
//!
//! **Rule**: Detect `eval` usage with user-controlled input
//!
//! **Why this matters**:
//! `eval` with user input is the #1 command injection vector in shell scripts.
//! Attackers can execute arbitrary commands by injecting shell metacharacters.
//!
//! **Auto-fix**: Manual review required (not auto-fixable)
//!
//! ## Examples
//!
//! ❌ **CRITICAL VULNERABILITY**:
//! ```bash
//! eval "rm -rf $USER_INPUT"
//! eval "$CMD"
//! ```
//!
//! ✅ **SAFE ALTERNATIVE**:
//! ```bash
//! # Use array and proper quoting instead of eval
//! # Or use explicit command construction with validation
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for command injection via eval
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Look for eval usage as a command (not part of another word)
        // Valid patterns: "eval ", "eval\"", "eval'", or eval at end of line
        if let Some(col) = line.find("eval") {
            // Check if it's a standalone command (word boundary)
            let before_ok = if col == 0 {
                true
            } else {
                let char_before = line.chars().nth(col - 1);
                matches!(char_before, Some(' ') | Some('\t') | Some(';') | Some('&') | Some('|') | Some('('))
            };

            let after_idx = col + 4; // "eval" is 4 chars
            let after_ok = if after_idx >= line.len() {
                true
            } else {
                let char_after = line.chars().nth(after_idx);
                matches!(char_after, Some(' ') | Some('\t') | Some('"') | Some('\'') | Some(';'))
            };

            if before_ok && after_ok {
                let span = Span::new(
                    line_num + 1,
                    col + 1,
                    line_num + 1,
                    col + 5,  // "eval" is 4 chars, +1 for 1-indexed
                );

                let diag = Diagnostic::new(
                    "SEC001",
                    Severity::Error,
                    "Command injection risk via eval - manual review required",
                    span,
                );
                // NO AUTO-FIX: eval replacement requires manual review

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
    fn test_SEC001_detects_eval_with_variable() {
        let script = r#"eval "rm -rf $USER_INPUT""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC001");
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("Command injection"));
    }

    #[test]
    fn test_SEC001_detects_eval_simple() {
        let script = "eval \"$CMD\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC001");
    }

    #[test]
    fn test_SEC001_no_false_positive_comment() {
        let script = "# This is evaluation, not eval";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC001_no_false_positive_text() {
        let script = r#"echo "medieval times""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC001_no_auto_fix() {
        let script = "eval \"$USER_CMD\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert!(diag.fix.is_none(), "SEC001 should not provide auto-fix");
    }

    #[test]
    fn test_SEC001_multiple_eval() {
        let script = "eval \"$CMD1\"\neval \"$CMD2\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 2);
    }
}
