//! MAKE015: Missing .DELETE_ON_ERROR
//!
//! **Rule**: Detect Makefiles without .DELETE_ON_ERROR special target
//!
//! **Why this matters**:
//! By default, Make leaves partially-built target files when a recipe fails.
//! This can cause corrupted builds. .DELETE_ON_ERROR tells Make to delete
//! the target file if the recipe fails, ensuring clean builds.
//!
//! **Auto-fix**: Add .DELETE_ON_ERROR: at top of Makefile
//!
//! ## Examples
//!
//! ❌ **BAD** (missing .DELETE_ON_ERROR):
//! ```makefile
//! .PHONY: all
//! all: build
//!
//! build:
//! \tcc main.c -o app
//! ```
//!
//! ✅ **GOOD** (with .DELETE_ON_ERROR):
//! ```makefile
//! .DELETE_ON_ERROR:
//! .PHONY: all
//! all: build
//!
//! build:
//! \tcc main.c -o app
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for missing .DELETE_ON_ERROR special target
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Empty Makefile doesn't need .DELETE_ON_ERROR
    if source.trim().is_empty() {
        return result;
    }

    // Check if .DELETE_ON_ERROR is present (case-sensitive)
    if has_delete_on_error(source) {
        return result;
    }

    // Missing .DELETE_ON_ERROR - create diagnostic
    let span = Span::new(1, 1, 1, 1); // Point to start of file

    // Create fix by adding .DELETE_ON_ERROR: at top
    let fix_replacement = create_fix(source);

    let diag = Diagnostic::new(
        "MAKE015",
        Severity::Warning,
        "Makefile missing .DELETE_ON_ERROR - partially-built files may be left on error",
        span,
    )
    .with_fix(Fix::new(&fix_replacement));

    result.add(diag);
    result
}

/// Create fix by adding .DELETE_ON_ERROR: at top of Makefile
fn create_fix(source: &str) -> String {
    format!(".DELETE_ON_ERROR:\n{}", source)
}

/// Check if Makefile has .DELETE_ON_ERROR (case-sensitive)
fn has_delete_on_error(source: &str) -> bool {
    for line in source.lines() {
        let trimmed = line.trim();
        // Check for .DELETE_ON_ERROR: or .DELETE_ON_ERROR (with optional colon)
        if trimmed == ".DELETE_ON_ERROR:" || trimmed == ".DELETE_ON_ERROR" {
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
    fn test_MAKE015_detects_missing_delete_on_error() {
        let makefile = ".PHONY: all\nall:\n\techo test";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "MAKE015");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.contains(".DELETE_ON_ERROR"));
    }

    #[test]
    fn test_MAKE015_no_warning_with_delete_on_error() {
        let makefile = ".DELETE_ON_ERROR:\n.PHONY: all\nall:\n\techo test";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE015_provides_fix() {
        let makefile = ".PHONY: all\nall:\n\techo test";
        let result = check(makefile);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert!(fix.replacement.contains(".DELETE_ON_ERROR:"));
    }

    #[test]
    fn test_MAKE015_detects_in_complex_makefile() {
        let makefile = r#"
# Complex Makefile
CC = gcc
CFLAGS = -Wall

.PHONY: all clean

all: app

app: main.o
\tcc main.o -o app

clean:
\trm -f *.o app
"#;
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE015_no_warning_with_delete_on_error_anywhere() {
        let makefile = r#"
# Makefile
.PHONY: all

all: build

.DELETE_ON_ERROR:

build:
\techo building
"#;
        let result = check(makefile);

        // .DELETE_ON_ERROR can appear anywhere (not just at top)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE015_case_sensitive() {
        let makefile = ".delete_on_error:\n.PHONY: all\nall:\n\techo test";
        let result = check(makefile);

        // .delete_on_error (lowercase) is NOT valid - must be .DELETE_ON_ERROR
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE015_fix_adds_at_top() {
        let makefile = "# Comment\n.PHONY: all\nall:\n\techo test";
        let result = check(makefile);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        // Fix should add .DELETE_ON_ERROR: at top
        assert!(fix.replacement.starts_with(".DELETE_ON_ERROR:"));
    }

    #[test]
    fn test_MAKE015_empty_makefile() {
        let makefile = "";
        let result = check(makefile);

        // Empty Makefile doesn't need .DELETE_ON_ERROR
        assert_eq!(result.diagnostics.len(), 0);
    }
}
