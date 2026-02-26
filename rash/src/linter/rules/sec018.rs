//! SEC018: Race Condition in File Operations (TOCTOU)
//!
//! **Rule**: Detect Time-of-Check, Time-of-Use (TOCTOU) race conditions
//!
//! **Why this matters**:
//! TOCTOU vulnerabilities occur when file state is checked, then used later:
//! - Between check and use, attacker can modify the file
//! - Symlink attacks can redirect to sensitive files
//! - Concurrent modifications can cause security issues
//! - Critical for privilege escalation prevention
//!
//! **Examples**:
//!
//! ❌ **DANGEROUS** (TOCTOU race condition):
//! ```bash
//! # Race: file could change between check and use
//! if [ -f "$CONFIG" ]; then
//!   source "$CONFIG"  # Attacker could replace with malicious file
//! fi
//!
//! # Race: symlink attack possible
//! [ -w "$LOGFILE" ] && echo "data" > "$LOGFILE"
//! ```
//!
//! ✅ **SAFE** (avoid TOCTOU):
//! ```bash
//! # Use file descriptor (atomic)
//! exec 3< "$CONFIG" && source /dev/fd/3
//!
//! # Use command's built-in checks
//! source "$CONFIG" 2>/dev/null || echo "Config not found"
//!
//! # For deletion, rm -f doesn't need check
//! rm -f "$FILE"  # Safe: doesn't fail if file doesn't exist
//! ```
//!
//! ## Detection Patterns
//!
//! This rule detects:
//! - `[ -f "$FILE" ] && cat/source/read "$FILE"` - Check then read
//! - `[ -e "$FILE" ] && rm "$FILE"` - Check then delete (use `rm -f`)
//! - `[ -w "$FILE" ] && echo/write to "$FILE"` - Check then write
//! - `test -f "$FILE" && operation` - Alternative test syntax
//!
//! ## Auto-fix
//!
//! Suggests:
//! - Use file descriptors for atomic operations
//! - Use command's built-in error handling (e.g., `rm -f`)
//! - Use proper file locking mechanisms

use crate::linter::LintResult;
use crate::linter::{Diagnostic, Severity, Span};

/// Check for TOCTOU race conditions in file operations
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

        // Pattern 1: [ -f "$FILE" ] && cat/source "$FILE"
        // Pattern 2: [ -e "$FILE" ] && operation "$FILE"
        // Pattern 3: [ -w "$FILE" ] && write to "$FILE"
        if (code_only.contains("[ -f")
            || code_only.contains("[ -e")
            || code_only.contains("[ -w")
            || code_only.contains("test -f")
            || code_only.contains("test -e")
            || code_only.contains("test -w"))
            && code_only.contains("&&")
        {
            // Check if same variable appears in test and operation
            if let Some(var_name) = extract_test_variable(code_only) {
                if code_only[code_only.find("&&").unwrap() + 2..].contains(&var_name) {
                    // Detect specific dangerous operations
                    let after_test = &code_only[code_only.find("&&").unwrap() + 2..];

                    if after_test.contains("cat")
                        || after_test.contains("source")
                        || after_test.contains("rm")
                        || after_test.contains('>')
                        || after_test.contains("cp")
                        || after_test.contains("mv")
                    {
                        let span = Span::new(line_num + 1, 1, line_num + 1, line.len());
                        let diag = Diagnostic::new(
                            "SEC018",
                            Severity::Warning,
                            format!(
                                "TOCTOU race condition: file '{}' is checked then used - file could change between check and use, use file descriptors or atomic operations",
                                var_name
                            ),
                            span,
                        );
                        result.add(diag);
                    }
                }
            }
        }

        // Pattern 4: if [ -f "$FILE" ]; then ... operation on $FILE
        // This is a multi-line pattern, more complex to detect accurately
        // For now, we'll focus on single-line patterns which are more common
    }

    result
}

/// Extract variable name from test expression
/// Example: `[ -f "$CONFIG" ]` → Some("CONFIG")
/// Example: `test -w "$LOGFILE"` → Some("LOGFILE")
fn extract_test_variable(line: &str) -> Option<String> {
    // Find the test part (before &&)
    let test_part = if let Some(pos) = line.find("&&") {
        &line[..pos]
    } else {
        line
    };

    // Extract variable from "$VAR" pattern
    if let Some(start) = test_part.find("\"$") {
        if let Some(end) = test_part[start + 2..].find('"') {
            let var_name = &test_part[start + 2..start + 2 + end];
            return Some(var_name.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED Phase: Write failing tests first (EXTREME TDD)

    /// RED TEST 1: Detect [ -f ] && cat (check then read)
    #[test]
    fn test_SEC018_detects_check_then_cat() {
        let script = r#"#!/bin/bash
[ -f "$CONFIG" ] && cat "$CONFIG"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC018");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.contains("TOCTOU"));
        assert!(diag.message.contains("CONFIG"));
    }

    /// RED TEST 2: Detect [ -e ] && rm (check then delete)
    #[test]
    fn test_SEC018_detects_check_then_rm() {
        let script = r#"#!/bin/bash
[ -e "$TMPFILE" ] && rm "$TMPFILE"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC018");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.contains("race condition"));
    }

    /// RED TEST 3: Detect [ -w ] && write (check then write)
    #[test]
    fn test_SEC018_detects_check_then_write() {
        let script = r#"#!/bin/bash
[ -w "$LOGFILE" ] && echo "data" > "$LOGFILE"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC018");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.contains("LOGFILE"));
    }

    /// RED TEST 4: Detect test -f syntax
    #[test]
    fn test_SEC018_detects_test_syntax() {
        let script = r#"#!/bin/bash
test -f "$FILE" && source "$FILE"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC018");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.contains("FILE"));
    }

    /// RED TEST 5: Pass safe rm -f usage (no check needed)
    #[test]
    fn test_SEC018_passes_safe_rm_f() {
        let script = r#"#!/bin/bash
# Safe: rm -f doesn't need existence check
rm -f "$TMPFILE"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Safe rm -f should pass");
    }

    /// RED TEST 6: Pass when different variables used
    #[test]
    fn test_SEC018_passes_different_variables() {
        let script = r#"#!/bin/bash
# Different variables, not TOCTOU
[ -f "$CONFIG1" ] && cat "$CONFIG2"
"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Different variables should pass"
        );
    }

    /// RED TEST 7: Detect [ -f ] && cp (check then copy)
    #[test]
    fn test_SEC018_detects_check_then_cp() {
        let script = r#"#!/bin/bash
[ -f "$SOURCE" ] && cp "$SOURCE" /dest/
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC018");
        assert_eq!(diag.severity, Severity::Warning);
    }

    /// RED TEST 8: Detect [ -f ] && mv (check then move)
    #[test]
    fn test_SEC018_detects_check_then_mv() {
        let script = r#"#!/bin/bash
[ -e "$OLD" ] && mv "$OLD" "$NEW"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC018");
        assert_eq!(diag.severity, Severity::Warning);
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
        fn prop_sec018_never_panics(s in ".*") {
            let _ = check(&s);
        }

        /// PROPERTY TEST 2: Always detects [ -f ] && cat pattern
        #[test]
        fn prop_sec018_detects_check_then_cat(
            var_name in "[A-Z_]{1,20}",
        ) {
            let script = format!("[ -f \"${}\" ] && cat \"${}\"", var_name, var_name);
            let result = check(&script);

            prop_assert_eq!(result.diagnostics.len(), 1);
            prop_assert_eq!(result.diagnostics[0].code.as_str(), "SEC018");
        }

        /// PROPERTY TEST 3: Passes when no && operator
        #[test]
        fn prop_sec018_passes_without_and(
            var_name in "[A-Z_]{1,20}",
        ) {
            let script = format!("cat \"${}\"", var_name);
            let result = check(&script);

            prop_assert_eq!(result.diagnostics.len(), 0);
        }
    }
}
