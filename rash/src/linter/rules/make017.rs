//! MAKE017: Missing .ONESHELL
//!
//! **Rule**: Detect Makefiles without .ONESHELL directive for multi-line recipes
//!
//! **Why this matters**:
//! By default, Make executes each line of a recipe in a separate shell. This means
//! variables set in one line won't be visible in the next line, and `cd` commands
//! don't persist. .ONESHELL tells Make to execute the entire recipe in a single
//! shell, making multi-line recipes behave as expected.
//!
//! **Auto-fix**: Add .ONESHELL: at top of Makefile
//!
//! ## Examples
//!
//! ❌ **BAD** (without .ONESHELL - each line in separate shell):
//! ```makefile
//! test:
//! \tcd test_dir
//! \t./run_tests.sh
//! ```
//! (./run_tests.sh runs in original directory, not test_dir)
//!
//! ✅ **GOOD** (with .ONESHELL - all lines in same shell):
//! ```makefile
//! .ONESHELL:
//!
//! test:
//! \tcd test_dir
//! \t./run_tests.sh
//! ```
//! (./run_tests.sh runs in test_dir as expected)

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for missing .ONESHELL directive
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Empty Makefile doesn't need .ONESHELL
    if source.trim().is_empty() {
        return result;
    }

    // Check if .ONESHELL is present (case-sensitive)
    if has_oneshell(source) {
        return result;
    }

    // Check if there are any multi-line recipes
    if !has_multiline_recipes(source) {
        return result;
    }

    // Missing .ONESHELL with multi-line recipes - create diagnostic
    let span = Span::new(1, 1, 1, 1); // Point to start of file

    // Create fix by adding .ONESHELL: at top
    let fix_replacement = format!(".ONESHELL:\n\n{}", source);

    let diag = Diagnostic::new(
        "MAKE017",
        Severity::Warning,
        "Missing .ONESHELL - multi-line recipes execute in separate shells (consider adding .ONESHELL: for consistent behavior)",
        span,
    )
    .with_fix(Fix::new(&fix_replacement));

    result.add(diag);
    result
}

/// Check if Makefile has .ONESHELL directive (case-sensitive)
fn has_oneshell(source: &str) -> bool {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(".ONESHELL") {
            return true;
        }
    }
    false
}

/// Check if Makefile has any multi-line recipes (targets with 2+ recipe lines)
fn has_multiline_recipes(source: &str) -> bool {
    let mut in_recipe = false;
    let mut recipe_line_count = 0;

    for line in source.lines() {
        if line.starts_with('\t') {
            // Recipe line
            if in_recipe {
                recipe_line_count += 1;
                if recipe_line_count >= 2 {
                    return true; // Found multi-line recipe
                }
            } else {
                in_recipe = true;
                recipe_line_count = 1;
            }
        } else if !line.trim().is_empty() {
            // Non-recipe, non-empty line - reset
            in_recipe = false;
            recipe_line_count = 0;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write failing tests first

    #[test]
    fn test_MAKE017_detects_missing_oneshell() {
        let makefile = "test:\n\tcd test_dir\n\t./run_tests.sh";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "MAKE017");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.to_lowercase().contains("oneshell"));
    }

    #[test]
    fn test_MAKE017_detects_multiline_recipe_without_oneshell() {
        let makefile = "build:\n\tVERSION=1.0\n\techo $VERSION";
        let result = check(makefile);

        // Multi-line recipe without .ONESHELL
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE017_provides_fix() {
        let makefile = "test:\n\tcd test_dir\n\t./run_tests.sh";
        let result = check(makefile);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        // Fix should add .ONESHELL: at top
        assert!(fix.replacement.contains(".ONESHELL:"));
    }

    #[test]
    fn test_MAKE017_no_warning_with_oneshell() {
        let makefile = ".ONESHELL:\n\ntest:\n\tcd test_dir\n\t./run_tests.sh";
        let result = check(makefile);

        // .ONESHELL present - OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE017_no_warning_for_single_line_recipes() {
        let makefile = "test:\n\t./run_tests.sh";
        let result = check(makefile);

        // Single-line recipe doesn't need .ONESHELL
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE017_detects_in_complex_makefile() {
        let makefile =
            "CC = gcc\n\nbuild:\n\tVERSION=1.0\n\techo $VERSION\n\ntest:\n\tcd test\n\t./run.sh";
        let result = check(makefile);

        // Has multi-line recipes, no .ONESHELL
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE017_case_sensitive() {
        let makefile = ".oneshell:\n\ntest:\n\tcd test_dir\n\t./run_tests.sh";
        let result = check(makefile);

        // .oneshell (lowercase) is NOT valid - must be .ONESHELL
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE017_empty_makefile() {
        let makefile = "";
        let result = check(makefile);

        // Empty Makefile doesn't need .ONESHELL
        assert_eq!(result.diagnostics.len(), 0);
    }
}
