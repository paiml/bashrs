//! BASH007: Hardcoded Absolute Paths (Non-portable)
//!
//! **Rule**: Detect hardcoded absolute paths to executables (non-portable)
//!
//! **Why this matters**:
//! Hardcoded absolute paths reduce script portability:
//! - `/usr/bin/python` may not exist on all systems
//! - `/usr/local/bin/jq` varies by installation method
//! - macOS vs Linux have different standard paths
//! - Makes scripts brittle and hard to maintain
//!
//! **Examples**:
//!
//! ❌ **BAD** (hardcoded paths):
//! ```bash
//! /usr/bin/python3 script.py
//! /usr/local/bin/jq '.items[]' data.json
//! /opt/custom/bin/tool --flag
//! ```
//!
//! ✅ **GOOD** (portable):
//! ```bash
//! # Use PATH-based execution
//! python3 script.py
//! jq '.items[]' data.json
//!
//! # Or use command -v for validation
//! PYTHON=$(command -v python3) || exit 1
//! "$PYTHON" script.py
//! ```
//!
//! ## Detection Logic
//!
//! This rule detects:
//! - Absolute paths to binaries: `/usr/bin/tool`, `/usr/local/bin/tool`
//! - Paths in `/opt/`, `/usr/`, `/bin/`, `/sbin/`
//!
//! Does NOT flag:
//! - Shebangs: `#!/bin/bash`, `#!/usr/bin/env`
//! - Standard system paths: `/dev/null`, `/tmp`, `/etc`, `/var`, `/proc`, `/sys`
//! - Variable references: `"$INSTALL_DIR/bin/tool"`
//!
//! ## Auto-fix
//!
//! Suggests:
//! - Use `command -v tool` to find in PATH
//! - Use environment variables for custom paths
//! - Remove absolute path and rely on PATH

use crate::linter::{Diagnostic, Severity, Span};
use crate::linter::LintResult;

/// Check for hardcoded absolute paths to executables
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
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

        // Look for hardcoded paths to executables
        // Patterns: /usr/bin/X, /usr/local/bin/X, /opt/X/bin/X, /bin/X, /sbin/X
        for pattern in &["/usr/bin/", "/usr/local/bin/", "/opt/", "/bin/", "/sbin/"] {
            if code_only.contains(pattern) {
                // Exclude common system paths that are OK
                if is_acceptable_path(code_only, pattern) {
                    continue;
                }

                // Exclude if it's in a variable assignment context
                if is_in_variable_context(code_only) {
                    continue;
                }

                let span = Span::new(line_num + 1, 1, line_num + 1, line.len());

                let diag = Diagnostic::new(
                    "BASH007",
                    Severity::Warning,
                    &format!(
                        "Hardcoded absolute path '{}' reduces portability - use 'command -v' to find in PATH or use environment variable",
                        pattern.trim_end_matches('/')
                    ),
                    span,
                );
                result.add(diag);
                break; // Only report once per line
            }
        }
    }

    result
}

/// Check if path is acceptable (standard system paths)
fn is_acceptable_path(line: &str, pattern: &str) -> bool {
    // Allow /dev/null, /tmp, /etc, /var, /proc, /sys
    if pattern == "/dev/" || pattern == "/tmp/" || pattern == "/etc/" ||
       pattern == "/var/" || pattern == "/proc/" || pattern == "/sys/" {
        return true;
    }

    // Allow /bin/bash, /bin/sh, /usr/bin/env in specific contexts (common shebangs)
    // Use word boundaries to avoid false positives like /bin/sha matching /bin/sh
    if (pattern == "/bin/" || pattern == "/usr/bin/") {
        if is_shebang_path(line) {
            return true;
        }
    }

    false
}

/// Check if line contains acceptable shebang paths
/// Uses word boundaries to avoid false positives (e.g., /bin/sha shouldn't match /bin/sh)
fn is_shebang_path(line: &str) -> bool {
    // Check for exact matches with word boundaries
    for path in &["/bin/bash", "/bin/sh", "/usr/bin/env", "/usr/bin/bash", "/usr/bin/sh"] {
        if line.contains(path) {
            // Verify it's a word boundary (followed by space, quote, or end of string)
            if let Some(pos) = line.find(path) {
                let after_pos = pos + path.len();
                if after_pos >= line.len() {
                    return true; // Path is at end of line
                }
                let next_char = line.chars().nth(after_pos);
                if matches!(next_char, Some(' ') | Some('\t') | Some('"') | Some('\'') | Some(';')) {
                    return true; // Path is followed by whitespace or quote
                }
            }
        }
    }
    false
}

/// Check if path is in a variable assignment or reference
fn is_in_variable_context(line: &str) -> bool {
    // Check if line is assigning to a variable: VAR=/usr/bin/tool
    if line.contains('=') && !line.contains("==") {
        let before_equals = line.split('=').next().unwrap_or("");
        // Simple heuristic: if before = looks like variable name
        if before_equals.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED Phase: Write failing tests first (EXTREME TDD)

    /// RED TEST 1: Detect /usr/bin/ absolute path
    #[test]
    fn test_BASH007_detects_usr_bin_path() {
        let script = r#"#!/bin/bash
/usr/bin/python3 script.py
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH007");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.contains("/usr/bin"));
        assert!(diag.message.contains("portability"));
    }

    /// RED TEST 2: Detect /usr/local/bin/ absolute path
    #[test]
    fn test_BASH007_detects_usr_local_bin_path() {
        let script = r#"#!/bin/bash
/usr/local/bin/jq '.items[]' data.json
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH007");
        assert!(diag.message.contains("/usr/local/bin"));
    }

    /// RED TEST 3: Detect /opt/ custom paths
    #[test]
    fn test_BASH007_detects_opt_path() {
        let script = r#"#!/bin/bash
/opt/custom/bin/tool --flag
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH007");
        assert!(diag.message.contains("/opt"));
    }

    /// RED TEST 4: Pass with portable command
    #[test]
    fn test_BASH007_passes_portable_command() {
        let script = r#"#!/bin/bash
python3 script.py
jq '.items[]' data.json
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Portable commands should pass");
    }

    /// RED TEST 5: Pass with command -v usage
    #[test]
    fn test_BASH007_passes_command_v() {
        let script = r#"#!/bin/bash
PYTHON=$(command -v python3) || exit 1
"$PYTHON" script.py
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "command -v usage should pass");
    }

    /// RED TEST 6: Pass with /dev/null (standard system path)
    #[test]
    fn test_BASH007_passes_dev_null() {
        let script = r#"#!/bin/bash
command > /dev/null 2>&1
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "/dev/null should pass");
    }

    /// RED TEST 7: Pass with variable assignment
    #[test]
    fn test_BASH007_passes_variable_assignment() {
        let script = r#"#!/bin/bash
CUSTOM_BIN=/usr/local/bin/custom_tool
"$CUSTOM_BIN" --flag
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Variable assignments should pass");
    }

    /// RED TEST 8: Ignore hardcoded paths in comments
    #[test]
    fn test_BASH007_ignores_comments() {
        let script = r#"#!/bin/bash
# Use /usr/bin/python3 if available
python3 script.py
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Comments should be ignored");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// PROPERTY TEST 1: Never panics on any input
        #[test]
        fn prop_bash007_never_panics(s in ".*") {
            let _ = check(&s);
        }

        /// PROPERTY TEST 2: Always detects /usr/bin/ paths
        #[test]
        fn prop_bash007_detects_usr_bin(
            tool in "[a-z]{3,10}",
        ) {
            let script = format!("/usr/bin/{} --flag", tool);
            let result = check(&script);

            prop_assert_eq!(result.diagnostics.len(), 1);
            prop_assert_eq!(result.diagnostics[0].code.as_str(), "BASH007");
        }

        /// PROPERTY TEST 3: Passes with portable commands
        #[test]
        fn prop_bash007_passes_portable(
            tool in "[a-z]{3,10}",
        ) {
            let script = format!("{} --flag", tool);
            let result = check(&script);

            prop_assert_eq!(result.diagnostics.len(), 0);
        }
    }
}
