//! MAKE005: Recursive variable assignment in Makefiles
//!
//! **Rule**: Detect `=` (recursive expansion) used with `$(shell ...)` that should use `:=` (immediate expansion)
//!
//! **Why this matters**:
//! Using `=` with `$(shell ...)` causes the shell command to be re-executed
//! every time the variable is referenced, leading to non-deterministic behavior
//! and performance issues. Use `:=` for immediate, one-time expansion.
//!
//! **Auto-fix**: Change `=` to `:=` for shell-based assignments
//!
//! ## Examples
//!
//! ❌ **BAD** (recursive expansion):
//! ```makefile
//! VERSION = $(shell git describe)
//! TIMESTAMP = $(shell date +%s)
//! ```
//!
//! ✅ **GOOD** (immediate expansion):
//! ```makefile
//! VERSION := $(shell git describe)
//! TIMESTAMP := $(shell date +%s)
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for recursive variable assignments with $(shell ...)
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Skip comments and empty lines
        if line.trim_start().starts_with('#') || line.trim().is_empty() {
            continue;
        }

        // Skip recipe lines (start with tab)
        if line.starts_with('\t') {
            continue;
        }

        // Check for variable assignment with =
        if let Some(eq_pos) = line.find('=') {
            // Make sure it's not :=, !=, +=, or ?=
            let before_eq = if eq_pos > 0 {
                line.chars().nth(eq_pos - 1)
            } else {
                None
            };

            if before_eq == Some(':') || before_eq == Some('!') ||
               before_eq == Some('+') || before_eq == Some('?') {
                continue;
            }

            // Check if the value contains $(shell ...)
            let after_eq = &line[eq_pos + 1..];
            if after_eq.contains("$(shell") {
                // Get variable name for better diagnostics
                let var_name = line[..eq_pos].trim();

                let span = Span::new(
                    line_num + 1,
                    eq_pos + 1,
                    line_num + 1,
                    eq_pos + 2,
                );

                // Create fix by replacing = with :=
                let fix_replacement = format!("{}:={}", var_name, after_eq);

                let diag = Diagnostic::new(
                    "MAKE005",
                    Severity::Warning,
                    &format!(
                        "Variable '{}' uses recursive expansion (=) with $(shell ...) - use := for immediate expansion",
                        var_name
                    ),
                    span,
                )
                .with_fix(Fix::new(&fix_replacement));

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
    fn test_MAKE005_detects_shell_with_recursive_expansion() {
        let makefile = "VERSION = $(shell git describe)";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "MAKE005");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.contains("VERSION"));
        assert!(diag.message.contains(":="));
    }

    #[test]
    fn test_MAKE005_no_warning_with_immediate_expansion() {
        let makefile = "VERSION := $(shell git describe)";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE005_detects_timestamp_shell() {
        let makefile = "TIMESTAMP = $(shell date +%s)";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("TIMESTAMP"));
    }

    #[test]
    fn test_MAKE005_no_warning_for_simple_assignment() {
        let makefile = "PREFIX = /usr/local";
        let result = check(makefile);

        // No $(shell ...), so no warning
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE005_provides_fix() {
        let makefile = "VERSION = $(shell git describe)";
        let result = check(makefile);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert!(fix.replacement.contains("VERSION:="));
        assert!(fix.replacement.contains("$(shell git describe)"));
    }

    #[test]
    fn test_MAKE005_no_false_positive_on_plus_equals() {
        let makefile = "FLAGS += $(shell pkg-config --cflags)";
        let result = check(makefile);

        // += is safe, shouldn't warn
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE005_no_false_positive_on_question_equals() {
        let makefile = "CC ?= $(shell which gcc)";
        let result = check(makefile);

        // ?= is safe, shouldn't warn
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE005_multiple_shell_assignments() {
        let makefile = "VERSION = $(shell git describe)\nDATE = $(shell date +%Y%m%d)";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 2);
        assert!(result.diagnostics[0].message.contains("VERSION"));
        assert!(result.diagnostics[1].message.contains("DATE"));
    }
}
