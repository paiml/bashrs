//! BASH005: Repeated Tool Dependency Checks (DRY Violation)
//!
//! **Rule**: Detect repeated `command -v` or `which` checks (DRY violation)
//!
//! **Why this matters**:
//! Repeated dependency checks violate the DRY (Don't Repeat Yourself) principle:
//! - Code duplication increases maintenance burden
//! - Inconsistent error messages across checks
//! - Harder to update dependency checking logic
//! - Reduces code readability
//!
//! **Examples**:
//!
//! ❌ **BAD** (repeated checks):
//! ```bash
//! if ! command -v git >/dev/null 2>&1; then
//!   echo "Error: git not found"
//!   exit 1
//! fi
//!
//! if ! command -v docker >/dev/null 2>&1; then
//!   echo "Error: docker not found"
//!   exit 1
//! fi
//!
//! if ! command -v jq >/dev/null 2>&1; then
//!   echo "Error: jq not found"
//!   exit 1
//! fi
//! ```
//!
//! ✅ **GOOD** (DRY with helper function):
//! ```bash
//! require_command() {
//!   if ! command -v "$1" >/dev/null 2>&1; then
//!     echo "Error: $1 not found" >&2
//!     exit 1
//!   fi
//! }
//!
//! require_command git
//! require_command docker
//! require_command jq
//! ```
//!
//! ## Detection Logic
//!
//! This rule detects:
//! - 3+ occurrences of `command -v` in a script
//! - 3+ occurrences of `which` for tool checking
//!
//! Suggests creating a reusable helper function.
//!
//! ## Auto-fix
//!
//! Suggests creating a `require_command()` helper function.

use crate::linter::LintResult;
use crate::linter::{Diagnostic, Severity, Span};

/// Check for repeated tool dependency checks
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let mut command_v_count = 0;
    let mut which_count = 0;
    let mut first_command_v_line = 0;
    let mut first_which_line = 0;

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Strip comments
        let code_only = if let Some(pos) = trimmed.find('#') {
            &trimmed[..pos]
        } else {
            trimmed
        };
        let code_only = code_only.trim();

        // Count command -v occurrences
        if code_only.contains("command -v") || code_only.contains("command  -v") {
            if command_v_count == 0 {
                first_command_v_line = line_num;
            }
            command_v_count += 1;
        }

        // Count which occurrences (used for tool checking)
        if code_only.contains("which ") && !code_only.starts_with("which") {
            // Only count which when used in conditions, not standalone
            if code_only.contains("if") || code_only.contains('!') {
                if which_count == 0 {
                    first_which_line = line_num;
                }
                which_count += 1;
            }
        }
    }

    // If 3+ command -v checks, suggest DRY refactor
    if command_v_count >= 3 {
        let span = Span::new(
            first_command_v_line + 1,
            1,
            first_command_v_line + 1,
            source.lines().nth(first_command_v_line).unwrap_or("").len(),
        );

        let diag = Diagnostic::new(
            "BASH005",
            Severity::Info,
            format!(
                "Repeated tool dependency checks ({} occurrences of 'command -v') - violates DRY principle; consider creating a 'require_command()' helper function",
                command_v_count
            ),
            span,
        );
        result.add(diag);
    }

    // If 3+ which checks, suggest DRY refactor
    if which_count >= 3 {
        let span = Span::new(
            first_which_line + 1,
            1,
            first_which_line + 1,
            source.lines().nth(first_which_line).unwrap_or("").len(),
        );

        let diag = Diagnostic::new(
            "BASH005",
            Severity::Info,
            format!(
                "Repeated tool dependency checks ({} occurrences of 'which') - violates DRY principle; consider creating a 'require_command()' helper function",
                which_count
            ),
            span,
        );
        result.add(diag);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED Phase: Write failing tests first (EXTREME TDD)

    /// RED TEST 1: Detect 3+ command -v checks
    #[test]
    fn test_BASH005_detects_repeated_command_v() {
        let script = r#"#!/bin/bash
if ! command -v git >/dev/null 2>&1; then exit 1; fi
if ! command -v docker >/dev/null 2>&1; then exit 1; fi
if ! command -v jq >/dev/null 2>&1; then exit 1; fi
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH005");
        assert_eq!(diag.severity, Severity::Info);
        assert!(diag.message.contains("command -v"));
        assert!(diag.message.contains("DRY"));
    }

    /// RED TEST 2: Pass with only 2 checks (not enough for DRY warning)
    #[test]
    fn test_BASH005_passes_with_two_checks() {
        let script = r#"#!/bin/bash
if ! command -v git >/dev/null 2>&1; then exit 1; fi
if ! command -v docker >/dev/null 2>&1; then exit 1; fi
"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "2 checks should not trigger warning"
        );
    }

    /// RED TEST 3: Detect 3+ which checks
    #[test]
    fn test_BASH005_detects_repeated_which() {
        let script = r#"#!/bin/bash
if ! which git >/dev/null; then exit 1; fi
if ! which docker >/dev/null; then exit 1; fi
if ! which jq >/dev/null; then exit 1; fi
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH005");
        assert!(diag.message.contains("which"));
    }

    /// RED TEST 4: Pass when using helper function
    #[test]
    fn test_BASH005_passes_with_helper_function() {
        let script = r#"#!/bin/bash
require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Error: $1 not found" >&2
    exit 1
  fi
}

require_command git
require_command docker
require_command jq
"#;
        let result = check(script);

        // The helper function itself has only 1 command -v, so no warning
        assert_eq!(result.diagnostics.len(), 0, "Helper function should pass");
    }

    /// RED TEST 5: Detect many checks (5+)
    #[test]
    fn test_BASH005_detects_many_checks() {
        let script = r#"#!/bin/bash
if ! command -v git >/dev/null 2>&1; then exit 1; fi
if ! command -v docker >/dev/null 2>&1; then exit 1; fi
if ! command -v jq >/dev/null 2>&1; then exit 1; fi
if ! command -v curl >/dev/null 2>&1; then exit 1; fi
if ! command -v wget >/dev/null 2>&1; then exit 1; fi
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert!(diag.message.contains("5 occurrences"));
    }

    /// RED TEST 6: Ignore command -v in comments
    #[test]
    fn test_BASH005_ignores_comments() {
        let script = r#"#!/bin/bash
# if ! command -v git >/dev/null 2>&1; then exit 1; fi
# if ! command -v docker >/dev/null 2>&1; then exit 1; fi
# if ! command -v jq >/dev/null 2>&1; then exit 1; fi
echo "No actual checks"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Comments should be ignored");
    }

    /// RED TEST 7: Detect mixed spacing in command -v
    #[test]
    fn test_BASH005_detects_varied_spacing() {
        let script = r#"#!/bin/bash
if ! command -v git >/dev/null 2>&1; then exit 1; fi
if ! command  -v docker >/dev/null 2>&1; then exit 1; fi
if ! command -v jq >/dev/null 2>&1; then exit 1; fi
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH005");
    }

    /// RED TEST 8: Message contains count
    #[test]
    fn test_BASH005_message_contains_count() {
        let script = r#"#!/bin/bash
if ! command -v git >/dev/null 2>&1; then exit 1; fi
if ! command -v docker >/dev/null 2>&1; then exit 1; fi
if ! command -v jq >/dev/null 2>&1; then exit 1; fi
if ! command -v curl >/dev/null 2>&1; then exit 1; fi
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert!(diag.message.contains("4 occurrences"));
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
        fn prop_bash005_never_panics(s in ".*") {
            let _ = check(&s);
        }

        /// PROPERTY TEST 2: Always detects 3+ command -v checks
        #[test]
        fn prop_bash005_detects_three_or_more(
            count in 3usize..10,
        ) {
            let mut script = String::from("#!/bin/bash\n");
            for i in 0..count {
                script.push_str(&format!("if ! command -v tool{} >/dev/null 2>&1; then exit 1; fi\n", i));
            }
            let result = check(&script);

            prop_assert_eq!(result.diagnostics.len(), 1);
            prop_assert_eq!(result.diagnostics[0].code.as_str(), "BASH005");
        }

        /// PROPERTY TEST 3: Passes with fewer than 3 checks
        #[test]
        fn prop_bash005_passes_with_fewer(
            count in 0usize..3,
        ) {
            let mut script = String::from("#!/bin/bash\n");
            for i in 0..count {
                script.push_str(&format!("if ! command -v tool{} >/dev/null 2>&1; then exit 1; fi\n", i));
            }
            let result = check(&script);

            prop_assert_eq!(result.diagnostics.len(), 0);
        }
    }
}
