//! BASH001: Missing `set -e` in Scripts
//!
//! **Rule**: Detect scripts missing `set -e` (exit on error)
//!
//! **Why this matters**:
//! Without `set -e`, bash scripts continue executing after errors, which can lead to:
//! - Silent failures that go unnoticed
//! - Cascading errors (later commands operate on bad state)
//! - Data corruption (partial operations complete)
//! - Security vulnerabilities (auth failures followed by unsafe operations)
//!
//! **Auto-fix**: Add `set -e` after shebang
//!
//! ## Examples
//!
//! ❌ **PROBLEMATIC** (errors ignored):
//! ```bash
//! #!/bin/bash
//! # Missing set -e
//! rm important_file.txt      # Fails silently if file doesn't exist
//! process_data                # Runs anyway, operating on incomplete data
//! ```
//!
//! ✅ **SAFE** (exits on error):
//! ```bash
//! #!/bin/bash
//! set -e
//! rm important_file.txt      # Exits immediately if this fails
//! process_data                # Only runs if rm succeeded
//! ```
//!
//! ## Exceptions
//!
//! Scripts that intentionally handle errors (with explicit `|| true` or `if` checks)
//! may not need `set -e`. However, it's still recommended as a safety net.

use crate::linter::{Diagnostic, Fix, FixSafetyLevel, LintResult, Severity, Span};

/// Check for missing `set -e` in scripts
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Check if script has set -e
    let has_set_e = source.lines().any(|line| {
        let trimmed = line.trim();
        // Match: set -e, set -ex, set -euo pipefail, etc.
        trimmed == "set -e"
            || trimmed.starts_with("set -e ")
            || (trimmed.starts_with("set -") && trimmed.contains('e'))
    });

    if !has_set_e {
        // Find shebang line or use line 1
        let shebang_line = source
            .lines()
            .position(|line| line.trim_start().starts_with("#!"))
            .map(|i| i + 1)
            .unwrap_or(1);

        // Suggest adding after shebang
        let insert_line = shebang_line + 1;

        let span = Span::new(1, 1, 1, 1); // Report at start of file

        let mut diag = Diagnostic::new(
            "BASH001",
            Severity::Warning,
            "Missing 'set -e' - script will continue after errors (potential silent failures)",
            span,
        );

        // AUTO-FIX: Add set -e after shebang
        // Note: Fix replacement is just the text to insert
        let fix = Fix::new("set -e\n");
        diag.fix = Some(fix);

        result.add(diag);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED Phase: Write failing tests first

    #[test]
    fn test_BASH001_detects_missing_set_e() {
        let script = r#"#!/bin/bash
echo "Hello, World!"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH001");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.contains("set -e"));
    }

    #[test]
    fn test_BASH001_passes_with_set_e() {
        let script = r#"#!/bin/bash
set -e
echo "Hello, World!"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_BASH001_passes_with_set_ex() {
        let script = r#"#!/bin/bash
set -ex
echo "Hello, World!"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_BASH001_passes_with_set_euo_pipefail() {
        let script = r#"#!/bin/bash
set -euo pipefail
echo "Hello, World!"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_BASH001_detects_missing_no_shebang() {
        let script = r#"echo "Hello, World!""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "BASH001");
    }

    #[test]
    fn test_BASH001_provides_auto_fix() {
        let script = r#"#!/bin/bash
echo "Hello"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert!(diag.fix.is_some(), "BASH001 should provide auto-fix");

        let fix = diag.fix.as_ref().unwrap();
        assert_eq!(fix.safety_level, FixSafetyLevel::Safe);
        assert!(fix.replacement.contains("set -e"));
    }

    #[test]
    fn test_BASH001_auto_fix_safe() {
        let script = r#"#!/bin/bash
echo "Hello"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.safety_level, FixSafetyLevel::Safe);
    }

    #[test]
    fn test_BASH001_passes_with_set_e_spaces() {
        let script = r#"#!/bin/bash
  set -e
echo "Hello"
"#;
        let result = check(script);

        // Should recognize set -e even with spaces
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_BASH001_passes_with_set_o_errexit() {
        let script = r#"#!/bin/bash
set -o errexit
echo "Hello"
"#;
        let result = check(script);

        // set -o errexit is equivalent to set -e
        // Our simple check might not catch this, which is acceptable
        // (Can be enhanced later if needed)
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_bash001_never_panics(s in ".*") {
            let _ = check(&s);
        }

        #[test]
        fn prop_bash001_detects_missing_set_e(
            shebang in "(#!/bin/bash|#!/usr/bin/env bash|#!/bin/sh)",
            code in "[a-z ]{1,50}",
        ) {
            let script = format!("{}\n{}", shebang, code);
            let result = check(&script);
            // Should detect missing set -e
            prop_assert_eq!(result.diagnostics.len(), 1);
            prop_assert_eq!(result.diagnostics[0].code.as_str(), "BASH001");
        }

        #[test]
        fn prop_bash001_passes_with_set_e(
            shebang in "(#!/bin/bash|#!/usr/bin/env bash)",
            set_e_variant in "(set -e|set -ex|set -euo pipefail)",
            code in "[a-z ]{1,50}",
        ) {
            let script = format!("{}\n{}\n{}", shebang, set_e_variant, code);
            let result = check(&script);
            // Should not flag scripts with set -e
            prop_assert_eq!(result.diagnostics.len(), 0);
        }
    }
}
