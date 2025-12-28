//! SEC011: Missing Input Validation Before Dangerous Operations
//!
//! **Rule**: Detect missing validation before dangerous operations
//!
//! **Why this matters**:
//! Without input validation, shell scripts can cause catastrophic damage:
//! - `rm -rf "$EMPTY_VAR"` → Deletes current directory
//! - `rm -rf "$VAR"` where `$VAR=/` → Deletes entire filesystem
//! - `chmod -R 777 "$DIR"` with invalid `$DIR` → Opens security holes
//! - SQL injection via unvalidated user input
//!
//! **Examples**:
//!
//! ❌ **DANGEROUS** (no validation):
//! ```bash
//! rm -rf "$BUILD_DIR"  # What if BUILD_DIR is empty or /?
//! chmod -R 777 "$DIR"  # What if DIR is unset?
//! ```
//!
//! ✅ **SAFE** (with validation):
//! ```bash
//! if [ -z "$BUILD_DIR" ] || [ "$BUILD_DIR" = "/" ]; then
//!   echo "Error: Invalid BUILD_DIR"
//!   exit 1
//! fi
//! rm -rf "$BUILD_DIR"
//! ```
//!
//! ## Detection Patterns
//!
//! This rule detects dangerous operations on variables without validation:
//! - `rm -rf "$VAR"` without checking if `$VAR` is empty or `/`
//! - `chmod -R 777 "$VAR"` without validation
//! - File operations on unvalidated paths
//!
//! ## Auto-fix
//!
//! This rule provides **suggestions** but not automatic fixes, because:
//! - Context-dependent validation logic
//! - Different operations need different validation
//! - Requires understanding of script intent

use crate::linter::LintResult;
use crate::linter::{Diagnostic, Severity, Span};

/// Issue #105: Known-safe environment variables that don't need validation
/// These are system-provided or set by the shell, not user input
const SAFE_ENV_VARS: &[&str] = &[
    // User and system info (set by shell/OS)
    "USER",
    "LOGNAME",
    "HOME",
    "SHELL",
    "PWD",
    "OLDPWD",
    "UID",
    "EUID",
    "PPID",
    "HOSTNAME",
    // Temp directories (controlled system paths)
    "TMPDIR",
    "TEMP",
    "TMP",
    // XDG directories (standard locations)
    "XDG_DATA_HOME",
    "XDG_CONFIG_HOME",
    "XDG_CACHE_HOME",
    "XDG_RUNTIME_DIR",
    // Common safe paths
    "PATH",
    "MANPATH",
    "LANG",
    "LC_ALL",
];

/// Issue #105: Check if a variable is a known-safe environment variable
fn is_safe_env_var(var_name: &str) -> bool {
    // Direct match
    if SAFE_ENV_VARS.contains(&var_name) {
        return true;
    }
    // XDG_* variables are generally safe (standard locations)
    if var_name.starts_with("XDG_") {
        return true;
    }
    false
}

/// Check for missing input validation before dangerous operations
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Track which variables have been validated
    let mut validated_vars: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Strip comments (everything after #)
        let code_only = if let Some(pos) = trimmed.find('#') {
            &trimmed[..pos]
        } else {
            trimmed
        };
        let code_only = code_only.trim();

        // Detect validation patterns (mark variables as validated)
        // Pattern: if [ -z "$VAR" ] || [ "$VAR" = "/" ]
        // Pattern: if [ -n "$VAR" ] && [ "$VAR" != "/" ]
        if trimmed.starts_with("if ") && (trimmed.contains("[ -z") || trimmed.contains("[ -n")) {
            // Extract variable name from validation
            if let Some(var_name) = extract_validated_variable(trimmed) {
                validated_vars.insert(var_name);
            }
        }

        // Issue #89: Detect inline validation with && chains
        // Pattern: [ -n "$VAR" ] && [ -d "$VAR" ] && rm -rf "$VAR"
        let inline_validated = extract_inline_validated_vars(code_only);

        // Detect dangerous operations
        // Pattern: rm -rf "$VAR"
        if code_only.contains("rm") && code_only.contains("-rf") {
            if let Some(var_name) = extract_variable_from_rm(code_only) {
                // Issue #105: Skip known-safe environment variables
                if is_safe_env_var(&var_name) {
                    continue;
                }
                // Check if variable is validated (either from previous lines or inline)
                if !validated_vars.contains(&var_name) && !inline_validated.contains(&var_name) {
                    let span = Span::new(line_num + 1, 1, line_num + 1, line.len());

                    let diag = Diagnostic::new(
                        "SEC011",
                        Severity::Error,
                        format!(
                            "Missing validation for '{}' before 'rm -rf' - could delete critical files if variable is empty or '/'",
                            var_name
                        ),
                        span,
                    );

                    result.add(diag);
                }
            }
        }

        // Pattern: chmod -R 777 "$VAR"
        if code_only.contains("chmod") && code_only.contains("-R") && code_only.contains("777") {
            if let Some(var_name) = extract_variable_from_chmod(code_only) {
                // Issue #105: Skip known-safe environment variables
                if is_safe_env_var(&var_name) {
                    continue;
                }
                if !validated_vars.contains(&var_name) && !inline_validated.contains(&var_name) {
                    let span = Span::new(line_num + 1, 1, line_num + 1, line.len());

                    let diag = Diagnostic::new(
                        "SEC011",
                        Severity::Error,
                        format!(
                            "Missing validation for '{}' before 'chmod -R 777' - could expose sensitive files if variable is unset",
                            var_name
                        ),
                        span,
                    );

                    result.add(diag);
                }
            }
        }

        // Pattern: chown -R user:group "$VAR"
        if code_only.contains("chown") && code_only.contains("-R") {
            if let Some(var_name) = extract_variable_from_chown(code_only) {
                // Issue #105: Skip known-safe environment variables
                if is_safe_env_var(&var_name) {
                    continue;
                }
                if !validated_vars.contains(&var_name) && !inline_validated.contains(&var_name) {
                    let span = Span::new(line_num + 1, 1, line_num + 1, line.len());

                    let diag = Diagnostic::new(
                        "SEC011",
                        Severity::Error,
                        format!(
                            "Missing validation for '{}' before 'chown -R' - could change ownership of critical files if variable is unset",
                            var_name
                        ),
                        span,
                    );

                    result.add(diag);
                }
            }
        }
    }

    result
}

/// Issue #89: Extract variables validated inline with && chains
/// Example: `[ -n "$VAR" ] && [ -d "$VAR" ] && rm -rf "$VAR"` → {"VAR"}
fn extract_inline_validated_vars(line: &str) -> std::collections::HashSet<String> {
    let mut validated = std::collections::HashSet::new();

    // Look for [ -n "$VAR" ] or [ -d "$VAR" ] patterns before && rm/chmod/chown
    // This validates the variable is non-empty or is a directory

    // Find all [ -n "$VAR" ] patterns
    for pattern in ["[ -n \"$", "[ -d \"$", "[ -e \"$", "[ -f \"$"] {
        let mut search_start = 0;
        while let Some(start) = line[search_start..].find(pattern) {
            let abs_start = search_start + start + pattern.len();
            if let Some(end) = line[abs_start..].find('"') {
                let var_name = &line[abs_start..abs_start + end];
                // Only count as validated if this test precedes a dangerous operation via &&
                // Check if there's && after this test and before the dangerous operation
                let after_test = &line[abs_start + end..];
                if after_test.contains("&&")
                    && (after_test.contains("rm ")
                        || after_test.contains("chmod ")
                        || after_test.contains("chown "))
                {
                    validated.insert(var_name.to_string());
                }
            }
            search_start = abs_start;
        }
    }

    validated
}

/// Extract variable name from validation pattern
/// Example: `if [ -z "$VAR" ]` → Some("VAR")
fn extract_validated_variable(line: &str) -> Option<String> {
    // Match: [ -z "$VAR" ] or [ -n "$VAR" ]
    if let Some(start) = line.find("\"$") {
        if let Some(end) = line[start + 2..].find('"') {
            let var_name = &line[start + 2..start + 2 + end];
            return Some(var_name.to_string());
        }
    }
    None
}

/// Extract just the variable name (stop at first non-var character)
/// Example: "HOME/.cache" → "HOME"
fn extract_var_name_only(s: &str) -> String {
    s.chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// Extract variable name from rm command
/// Example: `rm -rf "$BUILD_DIR"` → Some("BUILD_DIR")
/// Example: `rm -rf "$HOME/.cache"` → Some("HOME")
fn extract_variable_from_rm(line: &str) -> Option<String> {
    // Find "$VAR" pattern specifically after rm -rf
    // First find rm -rf or rm -r or rm --recursive
    let rm_pos = if let Some(pos) = line.find("rm -rf") {
        pos
    } else if let Some(pos) = line.find("rm -r ") {
        pos
    } else if let Some(pos) = line.find("rm --recursive") {
        pos
    } else {
        return None;
    };

    // Search for "$VAR" after the rm command
    let after_rm = &line[rm_pos..];
    if let Some(start) = after_rm.find("\"$") {
        let var_start = start + 2;
        let rest = &after_rm[var_start..];
        let var_name = extract_var_name_only(rest);
        if !var_name.is_empty() {
            return Some(var_name);
        }
    }
    None
}

/// Extract variable name from chmod command
/// Example: `chmod -R 777 "$DIR"` → Some("DIR")
/// Example: `chmod -R 777 "$HOME/.local"` → Some("HOME")
fn extract_variable_from_chmod(line: &str) -> Option<String> {
    // Find chmod command position first
    let chmod_pos = line.find("chmod")?;
    let after_chmod = &line[chmod_pos..];

    if let Some(start) = after_chmod.find("\"$") {
        let var_start = start + 2;
        let rest = &after_chmod[var_start..];
        let var_name = extract_var_name_only(rest);
        if !var_name.is_empty() {
            return Some(var_name);
        }
    }
    None
}

/// Extract variable name from chown command
/// Example: `chown -R user:group "$DIR"` → Some("DIR")
fn extract_variable_from_chown(line: &str) -> Option<String> {
    // Find chown command position first
    let chown_pos = line.find("chown")?;
    let after_chown = &line[chown_pos..];

    if let Some(start) = after_chown.find("\"$") {
        let var_start = start + 2;
        let rest = &after_chown[var_start..];
        let var_name = extract_var_name_only(rest);
        if !var_name.is_empty() {
            return Some(var_name);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED Phase: Write failing tests first (EXTREME TDD)

    /// RED TEST 1: Detect rm -rf without validation
    #[test]
    fn test_SEC011_detects_rm_rf_without_validation() {
        let script = r#"#!/bin/bash
rm -rf "$BUILD_DIR"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC011");
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("BUILD_DIR"));
        assert!(diag.message.contains("rm -rf"));
    }

    /// RED TEST 2: Pass when rm -rf has validation
    #[test]
    fn test_SEC011_passes_rm_rf_with_validation() {
        let script = r#"#!/bin/bash
if [ -z "$BUILD_DIR" ] || [ "$BUILD_DIR" = "/" ]; then
  echo "Error: Invalid BUILD_DIR"
  exit 1
fi
rm -rf "$BUILD_DIR"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Should pass with validation");
    }

    /// RED TEST 3: Detect chmod -R 777 without validation
    #[test]
    fn test_SEC011_detects_chmod_without_validation() {
        let script = r#"#!/bin/bash
chmod -R 777 "$DIR"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC011");
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("DIR"));
        assert!(diag.message.contains("chmod"));
    }

    /// RED TEST 4: Pass when chmod has validation
    #[test]
    fn test_SEC011_passes_chmod_with_validation() {
        let script = r#"#!/bin/bash
if [ -n "$DIR" ] && [ "$DIR" != "/" ]; then
  chmod -R 777 "$DIR"
fi
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Should pass with validation");
    }

    /// RED TEST 5: Detect chown -R without validation
    #[test]
    fn test_SEC011_detects_chown_without_validation() {
        let script = r#"#!/bin/bash
chown -R user:group "$DIR"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC011");
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("DIR"));
        assert!(diag.message.contains("chown"));
    }

    /// RED TEST 6: Multiple violations in same script
    #[test]
    fn test_SEC011_detects_multiple_violations() {
        let script = r#"#!/bin/bash
rm -rf "$BUILD_DIR"
chmod -R 777 "$TEMP_DIR"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 2);
        assert_eq!(result.diagnostics[0].code, "SEC011");
        assert_eq!(result.diagnostics[1].code, "SEC011");
    }

    /// RED TEST 7: Safe operations without dangerous flags
    #[test]
    fn test_SEC011_passes_safe_operations() {
        let script = r#"#!/bin/bash
rm file.txt       # Not rm -rf
chmod 644 "$FILE" # Not chmod -R 777
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Safe operations should pass");
    }

    /// RED TEST 8: Validation with -n (non-zero check)
    #[test]
    fn test_SEC011_recognizes_n_validation() {
        let script = r#"#!/bin/bash
if [ -n "$VAR" ]; then
  rm -rf "$VAR"
fi
"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Should recognize -n validation"
        );
    }

    // Issue #89: Inline validation with && chains
    #[test]
    fn test_SEC011_issue_89_inline_validation_with_and_chain() {
        // From issue #89 reproduction case
        let script = r#"[ -n "$TEMP_DIR" ] && [ -d "$TEMP_DIR" ] && rm -rf "$TEMP_DIR""#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "SEC011 should recognize inline validation with && chains"
        );
    }

    #[test]
    fn test_SEC011_issue_89_inline_n_validation() {
        let script = r#"[ -n "$VAR" ] && rm -rf "$VAR""#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "SEC011 should recognize [ -n ] inline validation"
        );
    }

    #[test]
    fn test_SEC011_issue_89_inline_d_validation() {
        let script = r#"[ -d "$DIR" ] && rm -rf "$DIR""#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "SEC011 should recognize [ -d ] inline validation"
        );
    }

    #[test]
    fn test_SEC011_issue_89_inline_if_block_validation() {
        // Another valid pattern from issue #89
        let script = r#"if [ -n "$VAR" ] && [ -d "$VAR" ]; then
    rm -rf "$VAR"
fi"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "SEC011 should recognize if-block validation"
        );
    }

    #[test]
    fn test_SEC011_issue_89_still_detects_unvalidated() {
        // Should still detect rm -rf without validation
        let script = r#"rm -rf "$TEMP_DIR""#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            1,
            "SEC011 should still detect unvalidated rm -rf"
        );
    }

    #[test]
    fn test_SEC011_issue_89_validation_wrong_var() {
        // Validation of different variable shouldn't count
        let script = r#"[ -n "$OTHER" ] && rm -rf "$TARGET""#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            1,
            "SEC011 should flag when different variable is validated"
        );
    }

    // Issue #105: Safe environment variables tests

    #[test]
    fn test_SEC011_105_user_env_var_safe() {
        // Issue #105: $USER is a safe environment variable
        let script = r#"rm -rf "/home/$USER/cache""#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "SEC011 should not flag $USER - it's a safe env var"
        );
    }

    #[test]
    fn test_SEC011_105_home_env_var_safe() {
        // Issue #105: $HOME is a safe environment variable
        let script = r#"rm -rf "$HOME/.cache""#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "SEC011 should not flag $HOME - it's a safe env var"
        );
    }

    #[test]
    fn test_SEC011_105_tmpdir_env_var_safe() {
        // Issue #105: $TMPDIR is a safe environment variable
        let script = r#"rm -rf "$TMPDIR/build""#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "SEC011 should not flag $TMPDIR - it's a safe env var"
        );
    }

    #[test]
    fn test_SEC011_105_xdg_cache_safe() {
        // Issue #105: XDG_* variables are safe
        let script = r#"rm -rf "$XDG_CACHE_HOME/myapp""#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "SEC011 should not flag XDG_CACHE_HOME - it's a safe env var"
        );
    }

    #[test]
    fn test_SEC011_105_custom_xdg_safe() {
        // Issue #105: Any XDG_* variable should be safe
        let script = r#"rm -rf "$XDG_CUSTOM_DIR/data""#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "SEC011 should not flag XDG_* vars - they're safe env vars"
        );
    }

    #[test]
    fn test_SEC011_105_pwd_env_var_safe() {
        // Issue #105: $PWD is a safe environment variable
        let script = r#"rm -rf "$PWD/build""#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "SEC011 should not flag $PWD - it's a safe env var"
        );
    }

    #[test]
    fn test_SEC011_105_unknown_var_still_flagged() {
        // Issue #105: Unknown variables should still be flagged
        let script = r#"rm -rf "$UNKNOWN_VAR""#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            1,
            "SEC011 should still flag unknown variables"
        );
    }

    #[test]
    fn test_SEC011_105_chmod_with_home() {
        // Issue #105: chmod with $HOME should be safe
        let script = r#"chmod -R 777 "$HOME/.local""#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "SEC011 should not flag chmod with $HOME"
        );
    }

    #[test]
    fn test_SEC011_105_chown_with_user() {
        // Issue #105: chown with $USER should be safe
        let script = r#"chown -R "$USER":staff "/shared/docs""#;
        let result = check(script);

        // Note: USER is in the chown user:group part, not the path
        // The path is hardcoded "/shared/docs" - should be safe
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SEC011 should not flag chown with hardcoded path"
        );
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(10))]
        /// PROPERTY TEST 1: Never panics on any input
        #[test]
        fn prop_sec011_never_panics(s in ".*") {
            let _ = check(&s);
        }

        /// PROPERTY TEST 2: Always detects rm -rf without validation
        #[test]
        fn prop_sec011_detects_rm_rf(
            var_name in "[A-Z_]{1,20}",
        ) {
            let script = format!("rm -rf \"${}\"", var_name);
            let result = check(&script);

            prop_assert_eq!(result.diagnostics.len(), 1);
            prop_assert_eq!(result.diagnostics[0].code.as_str(), "SEC011");
        }

        /// PROPERTY TEST 3: Always passes when validation present
        #[test]
        fn prop_sec011_passes_with_validation(
            var_name in "[A-Z_]{1,20}",
        ) {
            let script = format!(
                "if [ -z \"${}\" ]; then exit 1; fi\nrm -rf \"${}\"",
                var_name, var_name
            );
            let result = check(&script);

            prop_assert_eq!(result.diagnostics.len(), 0);
        }
    }
}
