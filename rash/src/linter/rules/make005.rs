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

/// Check if a line should be skipped (comment, empty, or recipe)
fn should_skip_line(line: &str) -> bool {
    line.trim_start().starts_with('#') || line.trim().is_empty() || line.starts_with('\t')
}

/// Get the character before equals sign
fn get_char_before_eq(line: &str, eq_pos: usize) -> Option<char> {
    if eq_pos > 0 {
        line.chars().nth(eq_pos - 1)
    } else {
        None
    }
}

/// Check if assignment is a special operator (:=, !=, +=, ?=)
fn is_special_operator(before_eq: Option<char>) -> bool {
    before_eq == Some(':')
        || before_eq == Some('!')
        || before_eq == Some('+')
        || before_eq == Some('?')
}

/// Check if value contains $(shell ...)
fn contains_shell_command(value: &str) -> bool {
    value.contains("$(shell")
}

/// Create diagnostic for recursive shell assignment
fn create_shell_diagnostic(
    var_name: &str,
    after_eq: &str,
    line_num: usize,
    eq_pos: usize,
) -> Diagnostic {
    let span = Span::new(line_num + 1, eq_pos + 1, line_num + 1, eq_pos + 2);
    let fix_replacement = format!("{}:={}", var_name, after_eq);

    Diagnostic::new(
        "MAKE005",
        Severity::Warning,
        format!(
            "Variable '{}' uses recursive expansion (=) with $(shell ...) - use := for immediate expansion",
            var_name
        ),
        span,
    )
    .with_fix(Fix::new(&fix_replacement))
}

/// Check for recursive variable assignments with $(shell ...)
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        if should_skip_line(line) {
            continue;
        }

        // Check for variable assignment with =
        if let Some(eq_pos) = line.find('=') {
            let before_eq = get_char_before_eq(line, eq_pos);

            if is_special_operator(before_eq) {
                continue;
            }

            // Check if the value contains $(shell ...)
            let after_eq = &line[eq_pos + 1..];
            if contains_shell_command(after_eq) {
                let var_name = line[..eq_pos].trim();
                let diag = create_shell_diagnostic(var_name, after_eq, line_num, eq_pos);
                result.add(diag);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Manual Property Tests =====

    #[test]
    fn prop_make005_comments_never_diagnosed() {
        // Property: Comment lines should never produce diagnostics
        let test_cases = vec![
            "# VERSION = $(shell git describe)",
            "  # TIMESTAMP = $(shell date)",
            "\t# VAR = $(shell cmd)",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_make005_recipe_lines_never_diagnosed() {
        // Property: Recipe lines (starting with tab) should not be diagnosed
        let test_cases = vec![
            "\techo $(shell date)",
            "\tVERSION=$(shell git describe)",
            "\t@echo $(shell pwd)",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_make005_immediate_expansion_never_diagnosed() {
        // Property: Immediate expansion (:=) should never be diagnosed
        let test_cases = vec![
            "VERSION := $(shell git describe)",
            "TIMESTAMP := $(shell date +%s)",
            "PWD := $(shell pwd)",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_make005_other_operators_never_diagnosed() {
        // Property: +=, ?=, != operators should never be diagnosed
        let test_cases = vec![
            "FLAGS += $(shell pkg-config --cflags)",
            "CC ?= $(shell which gcc)",
            "VAR != $(shell echo test)",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_make005_recursive_with_shell_always_diagnosed() {
        // Property: Recursive (=) with $(shell ...) should always be diagnosed
        let test_cases = vec![
            ("VERSION = $(shell git describe)", "VERSION"),
            ("TIMESTAMP = $(shell date +%s)", "TIMESTAMP"),
            ("PWD = $(shell pwd)", "PWD"),
        ];

        for (code, var_name) in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
            assert!(result.diagnostics[0].message.contains(var_name));
        }
    }

    #[test]
    fn prop_make005_simple_assignments_never_diagnosed() {
        // Property: Simple assignments without $(shell) should not be diagnosed
        let test_cases = vec!["PREFIX = /usr/local", "VERSION = 1.0.0", "NAME = myproject"];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_make005_all_diagnostics_have_fix() {
        // Property: All MAKE005 diagnostics must provide a fix
        let code = "VERSION = $(shell git describe)\nDATE = $(shell date)";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert!(
                diagnostic.fix.is_some(),
                "All MAKE005 diagnostics should have a fix"
            );
        }
    }

    #[test]
    fn prop_make005_diagnostic_code_always_make005() {
        // Property: All diagnostics must have code "MAKE005"
        let code = "V1 = $(shell cmd1)\nV2 = $(shell cmd2)";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(&diagnostic.code, "MAKE005");
        }
    }

    #[test]
    fn prop_make005_diagnostic_severity_always_warning() {
        // Property: All diagnostics must be Warning severity
        let code = "VERSION = $(shell git describe)";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Warning);
        }
    }

    #[test]
    fn prop_make005_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Original Unit Tests =====

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
