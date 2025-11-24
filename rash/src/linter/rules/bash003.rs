//! BASH003: `cd && command` Anti-pattern
//!
//! **Rule**: Detect `cd dir && command` pattern (fails silently if cd fails)
//!
//! **Why this matters**:
//! If `cd` fails, the subsequent command runs in the wrong directory:
//! - Data corruption (writing to wrong location)
//! - Security issues (operating on wrong files)
//! - Build failures in CI/CD pipelines
//! - Silent failures that are hard to debug
//!
//! **Examples**:
//!
//! ❌ **DANGEROUS** (`cd && command`):
//! ```bash
//! cd /var/data && rm -rf *
//! # If cd fails, rm runs in current directory! (catastrophic)
//!
//! cd "$BUILD_DIR" && make clean
//! # If BUILD_DIR is unset or invalid, make runs in wrong directory
//! ```
//!
//! ✅ **SAFE** (explicit error handling):
//! ```bash
//! # Option 1: Explicit error check
//! cd /var/data || exit 1
//! rm -rf *
//!
//! # Option 2: Subshell (isolated, safer)
//! (cd /var/data && rm -rf *)
//!
//! # Option 3: Check variable first
//! [ -d "$BUILD_DIR" ] || exit 1
//! cd "$BUILD_DIR" || exit 1
//! make clean
//! ```
//!
//! ## Detection Logic
//!
//! This rule detects:
//! - `cd <path> && <command>` - Command runs in potentially wrong directory
//! - Multiple commands after cd: `cd <path> && cmd1 && cmd2`
//!
//! Does NOT flag:
//! - `cd <path> || exit` - Has error handling
//! - `(cd <path> && cmd)` - Subshell (isolated)
//! - Just `cd <path>` - No subsequent command
//!
//! ## Auto-fix
//!
//! Suggests:
//! - Add explicit error handling: `cd dir || exit 1; command`
//! - Use subshell: `(cd dir && command)`
//! - Use pushd/popd for directory stack management

use crate::linter::LintResult;
use crate::linter::{Diagnostic, Severity, Span};

/// Check for dangerous `cd && command` pattern
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Strip comments
        let code_only = if let Some(pos) = trimmed.find('#') {
            &trimmed[..pos]
        } else {
            trimmed
        };
        let code_only = code_only.trim();

        // Skip empty lines
        if code_only.is_empty() {
            continue;
        }

        // Pattern: cd <path> && <command>
        // But NOT: (cd <path> && <command>) - subshell is safe
        if code_only.contains("cd ") && code_only.contains("&&") {
            // Skip if it's in a subshell
            if code_only.trim_start().starts_with('(') {
                continue;
            }

            // Check if there's a command after cd && ...
            if let Some(cd_pos) = code_only.find("cd ") {
                if let Some(and_pos) = code_only[cd_pos..].find("&&") {
                    let after_and = &code_only[cd_pos + and_pos + 2..].trim();

                    // If there's a command after &&, it's dangerous
                    if !after_and.is_empty() {
                        let span = Span::new(line_num + 1, 1, line_num + 1, line.len());

                        let diag = Diagnostic::new(
                            "BASH003",
                            Severity::Warning,
                            "Dangerous 'cd && command' pattern - if cd fails, command runs in wrong directory; use 'cd dir || exit 1' or '(cd dir && cmd)' in subshell",
                            span,
                        );
                        result.add(diag);
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

    // RED Phase: Write failing tests first (EXTREME TDD)

    /// RED TEST 1: Detect cd && command pattern
    #[test]
    fn test_BASH003_detects_cd_and_command() {
        let script = r#"#!/bin/bash
cd /var/data && rm -rf *
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH003");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.contains("cd && command"));
        assert!(diag.message.contains("wrong directory"));
    }

    /// RED TEST 2: Detect cd with variable && command
    #[test]
    fn test_BASH003_detects_cd_variable_and_command() {
        let script = r#"#!/bin/bash
cd "$BUILD_DIR" && make clean
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH003");
    }

    /// RED TEST 3: Detect multiple commands after cd
    #[test]
    fn test_BASH003_detects_cd_and_multiple_commands() {
        let script = r#"#!/bin/bash
cd src && make && make install
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH003");
    }

    /// RED TEST 4: Pass when using subshell (safe)
    #[test]
    fn test_BASH003_passes_subshell() {
        let script = r#"#!/bin/bash
(cd /var/data && rm -rf *)
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Subshell pattern is safe");
    }

    /// RED TEST 5: Pass when cd has no subsequent command
    #[test]
    fn test_BASH003_passes_cd_alone() {
        let script = r#"#!/bin/bash
cd /var/data
rm -rf *
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "cd alone is not flagged");
    }

    /// RED TEST 6: Pass when cd has error handling
    #[test]
    fn test_BASH003_passes_cd_with_error_handling() {
        let script = r#"#!/bin/bash
cd /var/data || exit 1
rm -rf *
"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "cd with error handling is safe"
        );
    }

    /// RED TEST 7: Detect cd in function
    #[test]
    fn test_BASH003_detects_cd_in_function() {
        let script = r#"#!/bin/bash
build() {
  cd "$1" && make
}
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH003");
    }

    /// RED TEST 8: Ignore cd in comments
    #[test]
    fn test_BASH003_ignores_comments() {
        let script = r#"#!/bin/bash
# cd /tmp && rm -rf *
echo "Safe"
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
        fn prop_bash003_never_panics(s in ".*") {
            let _ = check(&s);
        }

        /// PROPERTY TEST 2: Always detects cd && command
        #[test]
        fn prop_bash003_detects_cd_and(
            dir in "[a-z/_]{3,20}",
            cmd in "[a-z]{3,10}",
        ) {
            let script = format!("cd {} && {}", dir, cmd);
            let result = check(&script);

            prop_assert_eq!(result.diagnostics.len(), 1);
            prop_assert_eq!(result.diagnostics[0].code.as_str(), "BASH003");
        }

        /// PROPERTY TEST 3: Passes when using subshell
        #[test]
        fn prop_bash003_passes_subshell(
            dir in "[a-z/_]{3,20}",
            cmd in "[a-z]{3,10}",
        ) {
            let script = format!("(cd {} && {})", dir, cmd);
            let result = check(&script);

            prop_assert_eq!(result.diagnostics.len(), 0);
        }
    }
}
