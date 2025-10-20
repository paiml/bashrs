//! MAKE013: Missing .SUFFIXES (performance issue)
//!
//! **Rule**: Detect Makefiles without .SUFFIXES to disable built-in rules
//!
//! **Why this matters**:
//! GNU Make has many built-in implicit rules that search for various file
//! extensions (.c, .f, .p, etc.). This wastes time searching for files that
//! don't exist. Clearing .SUFFIXES disables these searches, improving performance.
//!
//! **Auto-fix**: Add .SUFFIXES: at top to clear built-in suffix rules
//!
//! ## Examples
//!
//! ❌ **BAD** (uses built-in implicit rules - slow):
//! ```makefile
//! all: app
//!
//! app: main.o
//! \tgcc main.o -o app
//! ```
//!
//! ✅ **GOOD** (disables built-in rules - fast):
//! ```makefile
//! .SUFFIXES:
//!
//! all: app
//!
//! app: main.o
//! \tgcc main.o -o app
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for missing .SUFFIXES to disable built-in implicit rules
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Empty Makefile doesn't need .SUFFIXES
    if source.trim().is_empty() {
        return result;
    }

    // Check if .SUFFIXES is present (case-sensitive)
    if has_suffixes(source) {
        return result;
    }

    // Missing .SUFFIXES - create diagnostic
    let span = Span::new(1, 1, 1, 1); // Point to start of file

    // Create fix by adding .SUFFIXES: at top
    let fix_replacement = format!(".SUFFIXES:\n\n{}", source);

    let diag = Diagnostic::new(
        "MAKE013",
        Severity::Warning,
        "Missing .SUFFIXES - built-in implicit rules slow down Make (consider adding .SUFFIXES: to disable)",
        span,
    )
    .with_fix(Fix::new(&fix_replacement));

    result.add(diag);
    result
}

/// Check if Makefile has .SUFFIXES directive (case-sensitive)
fn has_suffixes(source: &str) -> bool {
    for line in source.lines() {
        let trimmed = line.trim();
        // Check for .SUFFIXES: or .SUFFIXES (with optional colon and extensions)
        if trimmed.starts_with(".SUFFIXES") {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write failing tests first

    #[test]
    fn test_MAKE013_detects_missing_suffixes() {
        let makefile = "all: app\n\napp: main.o\n\tgcc main.o -o app";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "MAKE013");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.to_lowercase().contains("suffixes"));
    }

    #[test]
    fn test_MAKE013_no_warning_with_suffixes() {
        let makefile = ".SUFFIXES:\n\nall: app\n\napp: main.o\n\tgcc main.o -o app";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE013_provides_fix() {
        let makefile = "all: app\n\napp: main.o\n\tgcc main.o -o app";
        let result = check(makefile);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert!(fix.replacement.contains(".SUFFIXES:"));
    }

    #[test]
    fn test_MAKE013_detects_in_complex_makefile() {
        let makefile = "# Makefile\nCC = gcc\n\nall: app\n\napp: main.o\n\tgcc main.o -o app";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE013_no_warning_with_suffixes_anywhere() {
        let makefile = "all: app\n\n.SUFFIXES:\n\napp: main.o\n\tgcc main.o -o app";
        let result = check(makefile);

        // .SUFFIXES can appear anywhere
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE013_case_sensitive() {
        let makefile = ".suffixes:\n\nall: app\n\napp: main.o\n\tgcc main.o -o app";
        let result = check(makefile);

        // .suffixes (lowercase) is NOT valid - must be .SUFFIXES
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE013_no_warning_with_custom_suffixes() {
        let makefile = ".SUFFIXES: .c .o\n\nall: app";
        let result = check(makefile);

        // Custom suffixes defined - OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE013_empty_makefile() {
        let makefile = "";
        let result = check(makefile);

        // Empty Makefile doesn't need .SUFFIXES
        assert_eq!(result.diagnostics.len(), 0);
    }
}
