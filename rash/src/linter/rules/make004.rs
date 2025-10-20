//! MAKE004: Missing .PHONY declaration for non-file targets
//!
//! **Rule**: Detect targets that should be marked as .PHONY but aren't
//!
//! **Why this matters**:
//! Without .PHONY, make will look for a file with the target's name.
//! If such a file exists, make won't run the recipe. Common targets like
//! "clean", "test", "install" should always be .PHONY.
//!
//! **Auto-fix**: Add `.PHONY: target` declaration
//!
//! ## Examples
//!
//! ❌ **BAD** (missing .PHONY):
//! ```makefile
//! clean:
//! 	rm -f *.o
//!
//! test:
//! 	pytest tests/
//! ```
//!
//! ✅ **GOOD** (has .PHONY):
//! ```makefile
//! .PHONY: clean test
//!
//! clean:
//! 	rm -f *.o
//!
//! test:
//! 	pytest tests/
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use std::collections::HashSet;

/// Common non-file targets that should be .PHONY
const PHONY_TARGETS: &[&str] = &[
    "all",
    "clean",
    "test",
    "install",
    "uninstall",
    "check",
    "build",
    "run",
    "help",
    "dist",
    "distclean",
    "lint",
    "format",
    "fmt",
    "doc",
    "docs",
    "benchmark",
    "bench",
    "coverage",
    "deploy",
    "release",
    "dev",
    "prod",
];

/// Check for missing .PHONY declarations
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Find all .PHONY declarations
    let mut phony_targets = HashSet::new();
    for line in source.lines() {
        if line.trim_start().starts_with(".PHONY:") {
            // Extract target names after .PHONY:
            if let Some(targets_str) = line.split(':').nth(1) {
                for target in targets_str.split_whitespace() {
                    phony_targets.insert(target.to_string());
                }
            }
        }
    }

    // Find all targets and check if common ones are marked .PHONY
    for (line_num, line) in source.lines().enumerate() {
        // Skip .PHONY lines and comments
        if line.trim_start().starts_with(".PHONY") || line.trim_start().starts_with('#') {
            continue;
        }

        // Check if line defines a target (has : and doesn't start with tab)
        if line.contains(':') && !line.starts_with('\t') && !line.trim_start().is_empty() {
            if let Some(target_part) = line.split(':').next() {
                let target = target_part.trim();

                // Skip variable assignments (contains =)
                if line.contains('=') {
                    continue;
                }

                // Check if this is a common phony target
                if PHONY_TARGETS.contains(&target) && !phony_targets.contains(target) {
                    let span = Span::new(line_num + 1, 1, line_num + 1, target.len() + 1);

                    let fix = format!(".PHONY: {}", target);

                    let diag = Diagnostic::new(
                        "MAKE004",
                        Severity::Warning,
                        &format!("Target '{}' should be marked as .PHONY", target),
                        span,
                    )
                    .with_fix(Fix::new(&fix));

                    result.add(diag);
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_MAKE004_detects_missing_phony_clean() {
        let makefile = "clean:\n\trm -f *.o";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "MAKE004");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.contains("clean"));
        assert!(diag.message.contains(".PHONY"));
    }

    #[test]
    fn test_MAKE004_no_warning_with_phony() {
        let makefile = ".PHONY: clean\n\nclean:\n\trm -f *.o";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE004_detects_test_target() {
        let makefile = "test:\n\tpytest tests/";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("test"));
    }

    #[test]
    fn test_MAKE004_provides_fix() {
        let makefile = "clean:\n\trm -f *.o";
        let result = check(makefile);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, ".PHONY: clean");
    }

    #[test]
    fn test_MAKE004_multiple_missing_phony() {
        let makefile = "clean:\n\trm -f *.o\n\ntest:\n\tpytest\n\ninstall:\n\tcp app /usr/bin";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 3);
    }

    #[test]
    fn test_MAKE004_no_warning_for_file_targets() {
        let makefile = "app.o: app.c\n\tgcc -c app.c";
        let result = check(makefile);

        // app.o is a file target, not phony
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE004_phony_with_multiple_targets() {
        let makefile = ".PHONY: clean test\n\nclean:\n\trm -f *.o\n\ntest:\n\tpytest";
        let result = check(makefile);

        // Both targets declared as .PHONY
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE004_no_false_positive_on_variables() {
        let makefile = "CC = gcc\nCFLAGS = -Wall";
        let result = check(makefile);

        // Variable assignments shouldn't trigger
        assert_eq!(result.diagnostics.len(), 0);
    }
}
