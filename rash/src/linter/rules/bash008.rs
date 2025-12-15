//! BASH008: Missing Error Messages Before Exit
//!
//! **Rule**: Detect `exit` without preceding error message
//!
//! **Why this matters**:
//! Silent failures are hard to debug:
//! - Users don't know what went wrong
//! - No context for troubleshooting
//! - Automation pipelines fail mysteriously
//! - Wastes developer time investigating
//!
//! **Examples**:
//!
//! ❌ **BAD** (silent failure):
//! ```bash
//! if [ ! -f "$CONFIG" ]; then
//!   exit 1  # Why did it fail? User has no idea!
//! fi
//!
//! command || exit 1  # Silent failure
//! ```
//!
//! ✅ **GOOD** (informative error):
//! ```bash
//! if [ ! -f "$CONFIG" ]; then
//!   echo "Error: Config file not found: $CONFIG" >&2
//!   exit 1
//! fi
//!
//! command || { echo "Error: Command failed" >&2; exit 1; }
//! ```
//!
//! ## Detection Logic
//!
//! This rule detects:
//! - `exit 1` or `exit <non-zero>` without preceding error message
//! - `exit` without explicit code (defaults to last command's exit code)
//!
//! Does NOT flag:
//! - `exit 0` - success, no error message needed
//! - Lines with both error message and exit: `echo "Error" >&2; exit 1`
//! - Error message on previous line
//!
//! ## Auto-fix
//!
//! Suggests:
//! ```bash
//! echo "Error: [description]" >&2
//! exit 1
//! ```

use crate::linter::LintResult;
use crate::linter::{Diagnostic, Severity, Span};

/// Check for exit without error message
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let lines: Vec<&str> = source.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        // Strip comments from code
        let code_only = if let Some(pos) = trimmed.find('#') {
            &trimmed[..pos]
        } else {
            trimmed
        };
        let code_only = code_only.trim();

        // Detect exit statements (non-zero exit codes)
        if has_exit_statement(code_only) {
            // Check if exit code is 0 (success - no error message needed)
            if code_only.contains("exit 0") {
                continue;
            }

            // Check if error message is on same line (e.g., echo "Error" >&2; exit 1)
            if code_only.contains("echo") && code_only.contains(">&2") {
                continue;
            }

            // Check if previous line has error message
            if line_num > 0 && has_error_message_on_line(lines[line_num - 1]) {
                continue;
            }

            let span = Span::new(line_num + 1, 1, line_num + 1, line.len());

            let diag = Diagnostic::new(
                "BASH008",
                Severity::Info,
                "Exit without error message - add 'echo \"Error: [description]\" >&2' before exit for better debugging",
                span,
            );
            result.add(diag);
        }
    }

    result
}

/// Check if line has exit statement (non-zero)
fn has_exit_statement(line: &str) -> bool {
    // Match: exit, exit 1, exit <number>
    // But not: exit 0
    if line.contains("exit") {
        // Exclude "exit 0"
        if line.contains("exit 0") {
            return false;
        }
        // Match standalone "exit" or "exit <number>"
        return true;
    }
    false
}

/// Check if line has error message to stderr
fn has_error_message_on_line(line: &str) -> bool {
    let trimmed = line.trim();

    // Strip comments
    let code_only = if let Some(pos) = trimmed.find('#') {
        &trimmed[..pos]
    } else {
        trimmed
    };

    // Look for echo/printf to stderr
    (code_only.contains("echo") || code_only.contains("printf")) && code_only.contains(">&2")
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED Phase: Write failing tests first (EXTREME TDD)

    /// RED TEST 1: Detect exit 1 without error message
    #[test]
    fn test_BASH008_detects_exit_without_message() {
        let script = r#"#!/bin/bash
if [ ! -f "$CONFIG" ]; then
  exit 1
fi
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH008");
        assert_eq!(diag.severity, Severity::Info);
        assert!(diag.message.contains("error message"));
        assert!(diag.message.contains(">&2"));
    }

    /// RED TEST 2: Pass when error message precedes exit
    #[test]
    fn test_BASH008_passes_with_error_message() {
        let script = r#"#!/bin/bash
if [ ! -f "$CONFIG" ]; then
  echo "Error: Config not found" >&2
  exit 1
fi
"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Should pass with error message"
        );
    }

    /// RED TEST 3: Pass when exit 0 (success)
    #[test]
    fn test_BASH008_passes_exit_0() {
        let script = r#"#!/bin/bash
echo "Success"
exit 0
"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "exit 0 doesn't need error message"
        );
    }

    /// RED TEST 4: Pass when error message on same line
    #[test]
    fn test_BASH008_passes_inline_error() {
        let script = r#"#!/bin/bash
command || { echo "Error: Command failed" >&2; exit 1; }
"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Inline error message should pass"
        );
    }

    /// RED TEST 5: Detect standalone exit
    #[test]
    fn test_BASH008_detects_standalone_exit() {
        let script = r#"#!/bin/bash
if [ -z "$VAR" ]; then
  exit
fi
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH008");
    }

    /// RED TEST 6: Detect exit with specific code
    #[test]
    fn test_BASH008_detects_exit_with_code() {
        let script = r#"#!/bin/bash
exit 2
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH008");
    }

    /// RED TEST 7: Ignore exit in comments
    #[test]
    fn test_BASH008_ignores_comments() {
        let script = r#"#!/bin/bash
# exit 1 if something fails
echo "Running"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Comments should be ignored");
    }

    /// RED TEST 8: Pass with printf to stderr
    #[test]
    fn test_BASH008_passes_printf_stderr() {
        let script = r#"#!/bin/bash
printf "Error: Failed\n" >&2
exit 1
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "printf to stderr should pass");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// PROPERTY TEST 1: Never panics on any input
        #[test]
        fn prop_bash008_never_panics(s in ".*") {
            let _ = check(&s);
        }

        /// PROPERTY TEST 2: Always detects bare exit 1
        #[test]
        fn prop_bash008_detects_exit_1(
            code in 1u8..10,
        ) {
            let script = format!("exit {}", code);
            let result = check(&script);

            prop_assert_eq!(result.diagnostics.len(), 1);
            prop_assert_eq!(result.diagnostics[0].code.as_str(), "BASH008");
        }

        /// PROPERTY TEST 3: Passes with exit 0
        #[test]
        fn prop_bash008_passes_exit_0(
            _x in 0..100,
        ) {
            let script = "exit 0";
            let result = check(script);

            prop_assert_eq!(result.diagnostics.len(), 0);
        }
    }
}
