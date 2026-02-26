//! BASH010: Missing Script Header (Shebang/Description)
//!
//! **Rule**: Detect scripts without shebang or description comment
//!
//! **Why this matters**:
//! Scripts without headers are harder to maintain:
//! - No indication of intended interpreter
//! - Unclear script purpose
//! - Poor documentation for team members
//! - May execute with wrong shell
//!
//! **Examples**:
//!
//! ❌ **BAD** (missing header):
//! ```bash
//! echo "Starting backup"
//! rsync -av /data /backup
//! ```
//!
//! ✅ **GOOD** (complete header):
//! ```bash
//! #!/bin/bash
//! # Backup script - syncs data directory to backup location
//! # Usage: ./backup.sh
//!
//! echo "Starting backup"
//! rsync -av /data /backup
//! ```
//!
//! ## Detection Logic
//!
//! This rule detects:
//! - Scripts without shebang (#!) on first line
//! - Scripts without description comment after shebang
//!
//! ## Auto-fix
//!
//! Suggests adding:
//! ```bash
//! #!/bin/bash
//! # [Script description]
//! # Usage: [usage instructions]
//! ```

use crate::linter::LintResult;
use crate::linter::{Diagnostic, Severity, Span};

/// Check for missing script header
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let lines: Vec<&str> = source.lines().collect();

    if lines.is_empty() {
        return result; // Empty file, no warning
    }

    let mut has_shebang = false;
    let mut has_description = false;

    // Check first line for shebang
    if let Some(first_line) = lines.first() {
        if first_line.trim().starts_with("#!") {
            has_shebang = true;
        }
    }

    // Check for description comment (in first few lines)
    for (idx, line) in lines.iter().enumerate().take(10) {
        let trimmed = line.trim();

        // Skip shebang line
        if trimmed.starts_with("#!") {
            continue;
        }

        // Look for description comment (not empty comment)
        if trimmed.starts_with('#') && trimmed.len() > 2 {
            has_description = true;
            break;
        }

        // Stop if we hit code (non-comment, non-empty)
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            break;
        }
    }

    // Report missing shebang
    if !has_shebang {
        let span = Span::new(1, 1, 1, lines.first().map_or(0, |l| l.len()));

        let diag = Diagnostic::new(
            "BASH010",
            Severity::Info,
            "Script missing shebang - add '#!/bin/bash' or '#!/usr/bin/env bash' to specify interpreter",
            span,
        );
        result.add(diag);
    }

    // Report missing description (only if shebang exists)
    if has_shebang && !has_description {
        let span = Span::new(2, 1, 2, lines.get(1).map_or(0, |l| l.len()));

        let diag = Diagnostic::new(
            "BASH010",
            Severity::Info,
            "Script missing description comment - add comment explaining purpose and usage",
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

    /// RED TEST 1: Detect missing shebang
    #[test]
    fn test_BASH010_detects_missing_shebang() {
        let script = r#"echo "Hello"
echo "World"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH010");
        assert_eq!(diag.severity, Severity::Info);
        assert!(diag.message.contains("shebang"));
    }

    /// RED TEST 2: Detect missing description (has shebang)
    #[test]
    fn test_BASH010_detects_missing_description() {
        let script = r#"#!/bin/bash
echo "Hello"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH010");
        assert!(diag.message.contains("description"));
    }

    /// RED TEST 3: Pass with complete header
    #[test]
    fn test_BASH010_passes_complete_header() {
        let script = r#"#!/bin/bash
# This script greets the user
echo "Hello"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Complete header should pass");
    }

    /// RED TEST 4: Detect both missing shebang and description
    #[test]
    fn test_BASH010_detects_missing_both() {
        let script = r#"echo "Hello""#;
        let result = check(script);

        // Should only report missing shebang (description check requires shebang)
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "BASH010");
        assert!(result.diagnostics[0].message.contains("shebang"));
    }

    /// RED TEST 5: Pass with env shebang
    #[test]
    fn test_BASH010_passes_env_shebang() {
        let script = r#"#!/usr/bin/env bash
# Script description
echo "Hello"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "env shebang should pass");
    }

    /// RED TEST 6: Empty comment doesn't count as description
    #[test]
    fn test_BASH010_empty_comment_not_description() {
        let script = r#"#!/bin/bash
#
echo "Hello"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("description"));
    }

    /// RED TEST 7: Pass with multi-line description
    #[test]
    fn test_BASH010_passes_multiline_description() {
        let script = r#"#!/bin/bash
# Backup script
# Syncs data to backup location
# Usage: ./backup.sh
echo "Starting backup"
"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Multi-line description should pass"
        );
    }

    /// RED TEST 8: Empty file passes (no warning)
    #[test]
    fn test_BASH010_passes_empty_file() {
        let script = "";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Empty file should not warn");
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
        fn prop_bash010_never_panics(s in ".*") {
            let _ = check(&s);
        }

        /// PROPERTY TEST 2: Always detects script without shebang
        #[test]
        fn prop_bash010_detects_no_shebang(
            cmd in "[a-z]{3,10}",
        ) {
            let script = format!("echo {}", cmd);
            let result = check(&script);

            prop_assert_eq!(result.diagnostics.len(), 1);
            prop_assert_eq!(result.diagnostics[0].code.as_str(), "BASH010");
        }

        /// PROPERTY TEST 3: Passes with shebang and description
        #[test]
        fn prop_bash010_passes_complete(
            desc in "[a-zA-Z ]{10,50}",
        ) {
            let script = format!("#!/bin/bash\n# {}\necho test", desc);
            let result = check(&script);

            prop_assert_eq!(result.diagnostics.len(), 0);
        }
    }
}
