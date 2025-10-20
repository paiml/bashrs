//! SEC007: Running Commands as Root Without Validation
//!
//! **Rule**: Detect sudo/root operations without input validation
//!
//! **Why this matters**:
//! Unvalidated root operations can destroy entire systems if variables are
//! empty or contain dangerous values like "/".
//!
//! **Auto-fix**: Manual review required (context-dependent)
//!
//! ## Examples
//!
//! ❌ **UNSAFE ROOT OPERATIONS**:
//! ```bash
//! sudo rm -rf $DIR
//! sudo chmod 777 $FILE
//! sudo chown $USER $PATH
//! ```
//!
//! ✅ **ADD VALIDATION**:
//! ```bash
//! if [ -z "$DIR" ] || [ "$DIR" = "/" ]; then
//!     echo "Error: Invalid directory"
//!     exit 1
//! fi
//! sudo rm -rf "${DIR}"
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Dangerous commands that should never be run with sudo + unquoted vars
const DANGEROUS_SUDO_COMMANDS: &[&str] = &["rm -rf", "chmod 777", "chmod -R", "chown -R"];

/// Check for unsafe sudo operations
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        if line.contains("sudo") {
            // Check for dangerous sudo + command combinations
            for cmd in DANGEROUS_SUDO_COMMANDS {
                if line.contains(cmd) {
                    // Check for unquoted variables after the command
                    if line.contains(" $") {
                        if let Some(col) = line.find("sudo") {
                            let span = Span::new(
                                line_num + 1,
                                col + 1,
                                line_num + 1,
                                col + 5, // "sudo" is 4 chars
                            );

                            let diag = Diagnostic::new(
                                "SEC007",
                                Severity::Warning,
                                format!("Unsafe root operation: sudo {} with unquoted variable - add validation", cmd),
                                span,
                            );
                            // NO AUTO-FIX: requires context-dependent validation logic

                            result.add(diag);
                            break; // Only report once per line
                        }
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
    fn test_SEC007_detects_sudo_rm_rf() {
        let script = "sudo rm -rf $DIR";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC007");
        assert_eq!(diag.severity, Severity::Warning);
    }

    #[test]
    fn test_SEC007_detects_sudo_chmod_777() {
        let script = "sudo chmod 777 $FILE";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_SEC007_no_warning_with_quotes() {
        let script = "sudo rm -rf \"${DIR}\"";
        let result = check(script);

        // Still warns because even quoted vars need validation
        // But this is a simpler pattern matcher
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC007_no_warning_safe_command() {
        let script = "sudo systemctl restart nginx";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC007_no_auto_fix() {
        let script = "sudo rm -rf $TMPDIR";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert!(diag.fix.is_none(), "SEC007 should not provide auto-fix");
    }
}
