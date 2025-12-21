//! BASH001: Missing `set -e` in Scripts
//!
//! Detects scripts that are missing `set -e` (exit on error).
//!
//! ## Rationale
//! Without `set -e`, scripts continue executing after errors, which can hide
//! failures and lead to unexpected behavior. `set -e` causes scripts to exit
//! immediately when a command fails (returns non-zero exit status).
//!
//! ## Examples
//!
//! **Problematic** (missing `set -e`):
//! ```bash
//! #!/bin/bash
//! # Script continues even if commands fail
//! command_that_might_fail
//! rm -rf /important/data  # Runs even if previous command failed!
//! ```
//!
//! **Recommended** (with `set -e`):
//! ```bash
//! #!/bin/bash
//! set -e
//! command_that_might_fail  # Script exits here if this fails
//! rm -rf /important/data   # Only runs if previous succeeded
//! ```
//!
//! ## Configuration
//! - **Severity**: Warning
//! - **Enabled by default**: Yes
//! - **Auto-fix**: Suggest adding `set -e` after shebang

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for missing `set -e` in scripts
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let lines: Vec<&str> = source.lines().collect();
    if lines.is_empty() {
        return result; // Empty file, no warning
    }

    let mut has_set_e = false;
    let mut has_shebang = false;
    let mut shebang_line = 0;

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Check for shebang on first line
        if line_num == 0 && trimmed.starts_with("#!") {
            has_shebang = true;
            shebang_line = line_num;
        }

        // Check for various forms of set -e
        // set -e, set -ex, set -euo pipefail, etc.
        if (trimmed.starts_with("set ") || trimmed == "set") && trimmed.contains("-") {
            // Check if 'e' is in the flags
            if let Some(flags_start) = trimmed.find('-') {
                let flags_part = &trimmed[flags_start..];
                // Handle multiple flag groups like: set -e -u -o pipefail
                for flag_group in flags_part.split_whitespace() {
                    if flag_group.starts_with('-') && !flag_group.starts_with("--") {
                        // Single dash flags like -e, -ex, -euo
                        if flag_group.contains('e') {
                            has_set_e = true;
                            break;
                        }
                    }
                }
            }
        }

        // Also check for `set -o errexit` (equivalent to set -e)
        if trimmed.contains("set") && trimmed.contains("-o") && trimmed.contains("errexit") {
            has_set_e = true;
        }
    }

    // Only warn if script has shebang but missing set -e
    // Scripts without shebang might be sourced libraries
    if has_shebang && !has_set_e {
        let span = Span::new(
            shebang_line + 1,
            1,
            shebang_line + 1,
            lines[shebang_line].len(),
        );

        let diag = Diagnostic::new(
            "BASH001",
            Severity::Warning,
            "Missing 'set -e' in script. Without it, script continues after errors. Add 'set -e' after shebang to exit on first error. Consider 'set -euo pipefail' for stricter error handling.",
            span,
        );
        result.add(diag);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // Unit Tests (8 tests)
    // ========================================

    #[test]
    fn test_BASH001_detects_missing_set_e() {
        let code = r#"#!/bin/bash
# Script without set -e
echo "Hello"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("set -e"));
    }

    #[test]
    fn test_BASH001_passes_with_set_e() {
        let code = r#"#!/bin/bash
set -e
echo "Hello"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_BASH001_passes_with_set_ex() {
        let code = r#"#!/bin/bash
set -ex
echo "Hello"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_BASH001_passes_with_set_euo_pipefail() {
        let code = r#"#!/bin/bash
set -euo pipefail
echo "Hello"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_BASH001_passes_with_set_o_errexit() {
        let code = r#"#!/bin/bash
set -o errexit
echo "Hello"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_BASH001_passes_without_shebang() {
        // Sourced libraries don't need shebang or set -e
        let code = r#"# Library file
function helper() {
  echo "helper"
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_BASH001_passes_empty_file() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_BASH001_passes_with_multiple_set_flags() {
        let code = r#"#!/bin/bash
set -e -u -o pipefail
echo "Hello"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ========================================
    // Property Tests (3 tests)
    // ========================================

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(10))]
            #[test]
            fn prop_bash001_never_panics(code in ".*") {
                let _ = check(&code);
            }

            #[test]
            fn prop_bash001_detects_missing_when_shebang_present(
                shebang in r"#!/bin/(bash|sh|dash)",
                commands in prop::collection::vec("[a-z]+", 1..5)
            ) {
                let script = format!("{}\n{}\n", shebang, commands.join("\n"));
                let result = check(&script);
                // Should detect missing set -e when shebang is present
                prop_assert_eq!(result.diagnostics.len(), 1);
            }

            #[test]
            fn prop_bash001_passes_with_set_e(
                shebang in r"#!/bin/(bash|sh|dash)",
                commands in prop::collection::vec("[a-z]+", 1..5)
            ) {
                let script = format!("{}\nset -e\n{}\n", shebang, commands.join("\n"));
                let result = check(&script);
                // Should pass when set -e is present
                prop_assert_eq!(result.diagnostics.len(), 0);
            }
        }
    }
}
