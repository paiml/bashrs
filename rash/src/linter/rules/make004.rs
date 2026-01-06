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
//!     rm -f *.o
//!
//! test:
//!     pytest tests/
//! ```
//!
//! ✅ **GOOD** (has .PHONY):
//! ```makefile
//! .PHONY: clean test
//!
//! clean:
//!     rm -f *.o
//!
//! test:
//!     pytest tests/
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

/// Check if line is a .PHONY declaration
fn is_phony_line(line: &str) -> bool {
    line.trim_start().starts_with(".PHONY:")
}

/// Parse targets from .PHONY line
fn parse_phony_line(line: &str) -> Vec<String> {
    line.split(':')
        .nth(1)
        .map(|targets_str| {
            targets_str
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Parse all .PHONY declarations from source
/// F038 FIX: Handle multi-line .PHONY declarations with backslash continuations
fn parse_phony_targets(source: &str) -> HashSet<String> {
    let mut targets = HashSet::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        if is_phony_line(line) {
            // Start collecting .PHONY targets
            let mut combined_line = line.to_string();

            // F038: Handle line continuations (lines ending with \)
            while combined_line.trim_end().ends_with('\\') && i + 1 < lines.len() {
                // Remove the trailing backslash
                combined_line = combined_line.trim_end().trim_end_matches('\\').to_string();
                i += 1;
                // Add the continuation line
                combined_line.push(' ');
                combined_line.push_str(lines[i].trim());
            }

            // Parse all targets from the combined line
            for target in parse_phony_line(&combined_line) {
                targets.insert(target);
            }
        }
        i += 1;
    }

    targets
}

/// Check if line should be skipped (comments or .PHONY)
fn should_skip_line(line: &str) -> bool {
    line.trim_start().starts_with(".PHONY") || line.trim_start().starts_with('#')
}

/// Check if line defines a target
fn is_target_line(line: &str) -> bool {
    line.contains(':') && !line.starts_with('\t') && !line.trim_start().is_empty()
}

/// Check if line is a variable assignment
fn is_variable_assignment(line: &str) -> bool {
    line.contains('=')
}

/// Extract target name from line
fn extract_target_name(line: &str) -> Option<&str> {
    line.split(':').next().map(|s| s.trim())
}

/// Check if target should be marked as .PHONY
fn should_be_phony(target: &str) -> bool {
    PHONY_TARGETS.contains(&target)
}

/// Build diagnostic for missing .PHONY
fn build_phony_diagnostic(line_num: usize, target: &str) -> Diagnostic {
    let span = Span::new(line_num + 1, 1, line_num + 1, target.len() + 1);
    let fix = format!(".PHONY: {}", target);
    let message = format!("Target '{}' should be marked as .PHONY", target);

    Diagnostic::new("MAKE004", Severity::Warning, message, span).with_fix(Fix::new(&fix))
}

/// Check for missing .PHONY declarations
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let phony_targets = parse_phony_targets(source);

    for (line_num, line) in source.lines().enumerate() {
        if should_skip_line(line) || !is_target_line(line) || is_variable_assignment(line) {
            continue;
        }

        if let Some(target) = extract_target_name(line) {
            if should_be_phony(target) && !phony_targets.contains(target) {
                let diag = build_phony_diagnostic(line_num, target);
                result.add(diag);
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

    /// F038: MAKE004 must handle multi-line .PHONY declarations
    /// Issue #119: Multi-line .PHONY not recognized
    #[test]
    fn test_F038_MAKE004_multiline_phony() {
        // Multi-line .PHONY with backslash continuations
        let makefile = r#".PHONY: clean \
        test \
        install

clean:
	rm -f *.o

test:
	pytest

install:
	cp app /usr/bin"#;
        let result = check(makefile);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "F038 FALSIFIED: MAKE004 must recognize multi-line .PHONY declarations. Got: {:?}",
            result.diagnostics
        );
    }

    /// F038 variation: Mixed single and multi-line .PHONY
    #[test]
    fn test_F038_MAKE004_mixed_phony_declarations() {
        let makefile = r#".PHONY: clean
.PHONY: test \
        install

clean:
	rm -f *.o

test:
	pytest

install:
	cp app /usr/bin"#;
        let result = check(makefile);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "F038 FALSIFIED: Mixed .PHONY declarations should work. Got: {:?}",
            result.diagnostics
        );
    }
}
