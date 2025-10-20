//! MAKE020: Missing include guard
//!
//! **Rule**: Detect included Makefiles without include guards (double-inclusion prevention)
//!
//! **Why this matters**:
//! When Makefiles are included multiple times (directly or transitively), variables
//! can be redefined multiple times, rules can be duplicated, and builds become slower.
//! Include guards (like C header guards) prevent this by ensuring a file is only
//! processed once.
//!
//! **Auto-fix**: Add include guard pattern at top of file
//!
//! ## Examples
//!
//! ❌ **BAD** (no include guard - can be included multiple times):
//! ```makefile
//! # common.mk
//! CC = gcc
//! CFLAGS = -Wall
//! ```
//!
//! ✅ **GOOD** (with include guard):
//! ```makefile
//! # common.mk
//! ifndef COMMON_MK_INCLUDED
//! COMMON_MK_INCLUDED := 1
//!
//! CC = gcc
//! CFLAGS = -Wall
//!
//! endif
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for missing include guards in Makefiles meant for inclusion
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Empty Makefile doesn't need guard
    if source.trim().is_empty() {
        return result;
    }

    // If already has ifndef (even if not a proper guard), don't flag
    // to avoid false positives
    if has_ifndef(source) {
        return result;
    }

    // Check if this Makefile has content that should be guarded
    // (variable definitions that could be problematic if included multiple times)
    if !should_have_guard(source) {
        return result;
    }

    // Missing include guard - create diagnostic
    let span = Span::new(1, 1, 1, 1); // Point to start of file

    // Create fix by adding include guard
    let fix_replacement = create_guard_fix(source);

    let diag = Diagnostic::new(
        "MAKE020",
        Severity::Warning,
        "Missing include guard - Makefile may be included multiple times (consider adding ifndef/endif guard)",
        span,
    )
    .with_fix(Fix::new(&fix_replacement));

    result.add(diag);
    result
}

/// Check if Makefile has any ifndef directive
fn has_ifndef(source: &str) -> bool {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("ifndef ") {
            return true;
        }
    }
    false
}

/// Check if Makefile should have an include guard
/// (has variable definitions but is not just targets)
fn should_have_guard(source: &str) -> bool {
    let mut has_variables = false;

    for line in source.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Skip recipe lines
        if line.starts_with('\t') {
            continue;
        }

        // Check for variable definitions (contains = but not :)
        if trimmed.contains('=') && !trimmed.starts_with("export ") {
            has_variables = true;
            break;
        }
    }

    has_variables
}

/// Create fix by adding include guard around entire file
fn create_guard_fix(source: &str) -> String {
    // Generate guard name based on typical convention
    let guard_name = "MAKEFILE_INCLUDED";

    format!(
        "ifndef {}\n{} := 1\n\n{}\n\nendif",
        guard_name, guard_name, source
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write failing tests first

    #[test]
    fn test_MAKE020_detects_missing_guard() {
        let makefile = "# common.mk\nCC = gcc\nCFLAGS = -Wall";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "MAKE020");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(
            diag.message.to_lowercase().contains("guard")
                || diag.message.to_lowercase().contains("include")
        );
    }

    #[test]
    fn test_MAKE020_detects_makefile_with_variables() {
        let makefile = "VERSION = 1.0\nPREFIX = /usr/local";
        let result = check(makefile);

        // Has variables but no guard
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE020_provides_fix() {
        let makefile = "# common.mk\nCC = gcc\nCFLAGS = -Wall";
        let result = check(makefile);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        // Fix should add ifndef/endif guard
        assert!(fix.replacement.contains("ifndef"));
        assert!(fix.replacement.contains("endif"));
    }

    #[test]
    fn test_MAKE020_no_warning_with_guard() {
        let makefile = "ifndef COMMON_MK\nCOMMON_MK := 1\n\nCC = gcc\n\nendif";
        let result = check(makefile);

        // Has include guard - OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE020_no_warning_for_simple_targets_only() {
        let makefile = "all:\n\t$(CC) main.c\n\nclean:\n\trm -f *.o";
        let result = check(makefile);

        // Only targets, no variables to guard - OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE020_detects_complex_makefile() {
        let makefile =
            "# config.mk\nCC = gcc\nCXX = g++\nAR = ar\n\nCFLAGS = -Wall\nLDFLAGS = -L/usr/lib";
        let result = check(makefile);

        // Complex config file needs guard
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE020_no_warning_with_ifndef_anywhere() {
        let makefile = "CC = gcc\n\nifndef DEBUG\nCFLAGS = -O2\nendif";
        let result = check(makefile);

        // Has ifndef (even if not a guard) - don't flag to avoid false positives
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE020_empty_makefile() {
        let makefile = "";
        let result = check(makefile);

        // Empty Makefile doesn't need guard
        assert_eq!(result.diagnostics.len(), 0);
    }
}
