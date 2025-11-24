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

        // Detect dangerous operations
        // Pattern: rm -rf "$VAR"
        if code_only.contains("rm") && code_only.contains("-rf") {
            if let Some(var_name) = extract_variable_from_rm(code_only) {
                if !validated_vars.contains(&var_name) {
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
                if !validated_vars.contains(&var_name) {
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
                if !validated_vars.contains(&var_name) {
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

/// Extract variable name from rm command
/// Example: `rm -rf "$BUILD_DIR"` → Some("BUILD_DIR")
fn extract_variable_from_rm(line: &str) -> Option<String> {
    // Find "$VAR" pattern after rm -rf
    if let Some(start) = line.find("\"$") {
        if let Some(end) = line[start + 2..].find('"') {
            let var_name = &line[start + 2..start + 2 + end];
            return Some(var_name.to_string());
        }
    }
    None
}

/// Extract variable name from chmod command
/// Example: `chmod -R 777 "$DIR"` → Some("DIR")
fn extract_variable_from_chmod(line: &str) -> Option<String> {
    if let Some(start) = line.find("\"$") {
        if let Some(end) = line[start + 2..].find('"') {
            let var_name = &line[start + 2..start + 2 + end];
            return Some(var_name.to_string());
        }
    }
    None
}

/// Extract variable name from chown command
/// Example: `chown -R user:group "$DIR"` → Some("DIR")
fn extract_variable_from_chown(line: &str) -> Option<String> {
    if let Some(start) = line.find("\"$") {
        if let Some(end) = line[start + 2..].find('"') {
            let var_name = &line[start + 2..start + 2 + end];
            return Some(var_name.to_string());
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
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
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
