//! MAKE014: Inefficient shell invocation
//!
//! **Rule**: Detect commands that unnecessarily spawn multiple shells
//!
//! **Why this matters**:
//! Each shell invocation has overhead. Commands like `$(shell cat file)` spawn
//! a shell process just to run a simple command. Using Make built-ins or
//! combining commands is more efficient. Multiple separate shell commands
//! should be combined with && or ; when appropriate.
//!
//! **Auto-fix**: Suggest more efficient alternatives
//!
//! ## Examples
//!
//! ❌ **BAD** (inefficient shell usage):
//! ```makefile
//! VERSION = $(shell cat VERSION)
//! FILES = $(shell ls *.c)
//! ```
//!
//! ✅ **GOOD** (efficient alternatives):
//! ```makefile
//! VERSION = $(file < VERSION)
//! FILES = $(wildcard *.c)
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Inefficient shell patterns to detect
const INEFFICIENT_PATTERNS: &[(&str, &str)] = &[
    ("$(shell cat ", "$(file < "),  // cat file → file function
    ("$(shell ls ", "$(wildcard "), // ls → wildcard
    ("$(shell echo ", "$(info "),   // echo → info/warning
    ("$(shell pwd)", "$(CURDIR)"),  // pwd → CURDIR variable
];

/// Check for inefficient shell invocations
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Check each inefficient pattern
        for (pattern, replacement) in INEFFICIENT_PATTERNS {
            if line.contains(pattern) {
                let span = Span::new(line_num + 1, 1, line_num + 1, line.len());
                let fix_replacement = create_fix(line, pattern, replacement);

                let diag = Diagnostic::new(
                    "MAKE014",
                    Severity::Warning,
                    &format!(
                        "Inefficient shell invocation '{}' - consider using '{}' instead",
                        pattern.trim(),
                        replacement.trim()
                    ),
                    span,
                )
                .with_fix(Fix::new(&fix_replacement));

                result.add(diag);
                break; // Only report once per line
            }
        }
    }

    result
}

/// Create a fix by replacing inefficient pattern with efficient alternative
fn create_fix(line: &str, pattern: &str, replacement: &str) -> String {
    line.replace(pattern, replacement)
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write failing tests first

    #[test]
    fn test_MAKE014_detects_shell_cat() {
        let makefile = "VERSION = $(shell cat VERSION)";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "MAKE014");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.to_lowercase().contains("shell"));
    }

    #[test]
    fn test_MAKE014_detects_shell_ls() {
        let makefile = "FILES = $(shell ls *.c)";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE014_detects_shell_echo() {
        let makefile = "MSG = $(shell echo hello)";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE014_detects_shell_pwd() {
        let makefile = "DIR = $(shell pwd)";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE014_provides_fix() {
        let makefile = "VERSION = $(shell cat VERSION)";
        let result = check(makefile);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        // Fix should suggest $(file < VERSION)
        assert!(fix.replacement.contains("$(file <"));
    }

    #[test]
    fn test_MAKE014_no_warning_for_efficient_commands() {
        let makefile = "VERSION = $(file < VERSION)\nFILES = $(wildcard *.c)";
        let result = check(makefile);

        // Efficient commands are OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE014_detects_multiple_inefficiencies() {
        let makefile = "VERSION = $(shell cat VERSION)\nFILES = $(shell ls *.c)";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_MAKE014_empty_makefile() {
        let makefile = "";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }
}
