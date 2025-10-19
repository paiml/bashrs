//! SEC002: Unquoted Variable in Command
//!
//! **Rule**: Detect unquoted variables in potentially dangerous commands
//!
//! **Why this matters**:
//! Unquoted variables can lead to command injection if they contain spaces
//! or special characters. This is especially dangerous in commands that
//! interact with the network, filesystem, or execute other commands.
//!
//! **Auto-fix**: Safe (add quotes)
//!
//! ## Examples
//!
//! ❌ **UNSAFE**:
//! ```bash
//! curl $URL
//! wget $FILE_PATH
//! ssh $HOST
//! git clone $REPO
//! ```
//!
//! ✅ **SAFE** (auto-fixable):
//! ```bash
//! curl "${URL}"
//! wget "${FILE_PATH}"
//! ssh "${HOST}"
//! git clone "${REPO}"
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Dangerous commands that should never have unquoted variables
const DANGEROUS_COMMANDS: &[&str] = &[
    "curl",
    "wget",
    "ssh",
    "scp",
    "git",
    "rsync",
    "docker",
    "kubectl",
];

/// Check for unquoted variables in dangerous commands
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Skip comments
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check each dangerous command
        for cmd in DANGEROUS_COMMANDS {
            if line.contains(cmd) {
                // Look for unquoted variable usage: $VAR (not "$VAR" or "${VAR}")
                // Simple pattern: find $WORD where WORD is not inside quotes
                let mut chars = line.chars().peekable();
                let mut col = 0;
                let mut in_double_quotes = false;
                let mut in_single_quotes = false;

                while let Some(ch) = chars.next() {
                    col += 1;

                    match ch {
                        '"' if !in_single_quotes => in_double_quotes = !in_double_quotes,
                        '\'' if !in_double_quotes => in_single_quotes = !in_single_quotes,
                        '$' if !in_double_quotes && !in_single_quotes => {
                            // Found unquoted $
                            // Check if followed by variable name
                            if chars.peek().map(|c| c.is_alphanumeric() || *c == '_').unwrap_or(false) {
                                let span = Span::new(
                                    line_num + 1,
                                    col,
                                    line_num + 1,
                                    col + 1,
                                );

                                let diag = Diagnostic::new(
                                    "SEC002",
                                    Severity::Error,
                                    format!("Unquoted variable in {} command - add quotes", cmd),
                                    span,
                                )
                                .with_fix(Fix::new("\"$VAR\""));

                                result.add(diag);
                                // Only report once per line
                                break;
                            }
                        }
                        _ => {}
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
    fn test_SEC002_detects_unquoted_curl() {
        let script = "curl $URL";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC002");
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("curl"));
    }

    #[test]
    fn test_SEC002_detects_unquoted_wget() {
        let script = "wget $FILE_PATH";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_SEC002_detects_unquoted_ssh() {
        let script = "ssh $HOST";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_SEC002_no_warning_with_quotes() {
        let script = r#"curl "${URL}""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC002_no_warning_with_double_quotes() {
        let script = "wget \"$FILE_PATH\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC002_provides_fix() {
        let script = "curl $URL";
        let result = check(script);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, "\"$VAR\"");
    }

    #[test]
    fn test_SEC002_no_false_positive_comment() {
        let script = "# curl $URL";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }
}
