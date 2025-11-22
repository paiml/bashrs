//! MAKE002: Non-idempotent mkdir in Makefile recipes
//!
//! **Rule**: Detect `mkdir` without `-p` flag in recipe commands
//!
//! **Why this matters**:
//! Without `-p`, mkdir fails if the directory already exists, making the
//! Makefile non-idempotent (not safe to re-run).
//!
//! **Auto-fix**: Add `-p` flag
//!
//! ## Examples
//!
//! ❌ **BAD** (non-idempotent):
//! ```makefile
//! build:
//!     mkdir build
//!     gcc -o app main.c
//! ```
//!
//! ✅ **GOOD** (idempotent):
//! ```makefile
//! build:
//!     mkdir -p build
//!     gcc -o app main.c
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check if line is a Makefile recipe line (starts with tab and contains mkdir)
fn is_recipe_with_mkdir(line: &str) -> bool {
    line.starts_with('\t') && line.contains("mkdir")
}

/// Check if mkdir already has -p flag
fn has_p_flag(line: &str, mkdir_pos: usize) -> bool {
    let after_mkdir = &line[mkdir_pos + 5..];
    after_mkdir.trim_start().starts_with("-p")
}

/// Create diagnostic for non-idempotent mkdir
fn create_mkdir_diagnostic(line_num: usize, mkdir_pos: usize) -> Diagnostic {
    let span = Span::new(
        line_num + 1,
        mkdir_pos + 1,
        line_num + 1,
        mkdir_pos + 6, // length of "mkdir"
    );

    Diagnostic::new(
        "MAKE002",
        Severity::Warning,
        "Non-idempotent mkdir - will fail if directory exists",
        span,
    )
    .with_fix(Fix::new("mkdir -p"))
}

/// Check for non-idempotent mkdir usage in Makefile recipes
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        if !is_recipe_with_mkdir(line) {
            continue;
        }

        if let Some(mkdir_pos) = line.find("mkdir") {
            // Skip if already has -p flag
            if has_p_flag(line, mkdir_pos) {
                continue;
            }

            let diag = create_mkdir_diagnostic(line_num, mkdir_pos);
            result.add(diag);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write failing tests first

    #[test]
    fn test_MAKE002_detects_mkdir_without_p() {
        let makefile = "build:\n\tmkdir build";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "MAKE002");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.contains("Non-idempotent"));
    }

    #[test]
    fn test_MAKE002_no_warning_with_p_flag() {
        let makefile = "build:\n\tmkdir -p build";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE002_no_warning_outside_recipe() {
        let makefile = "# mkdir without -p in comment\nDIR = mkdir";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE002_provides_fix() {
        let makefile = "build:\n\tmkdir build";
        let result = check(makefile);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, "mkdir -p");
    }

    #[test]
    fn test_MAKE002_detects_multiple_mkdir() {
        let makefile = "build:\n\tmkdir build\n\tmkdir dist";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_MAKE002_with_path_argument() {
        let makefile = "install:\n\tmkdir /usr/local/bin";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE002_mkdir_with_other_flags() {
        let makefile = "build:\n\tmkdir -m 755 build";
        let result = check(makefile);

        // Should still warn since -p is not present
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE002_multiple_recipes() {
        let makefile = r#"build:
	mkdir build
	gcc main.c

install:
	mkdir -p /usr/local/bin
	cp app /usr/local/bin"#;
        let result = check(makefile);

        // Only first mkdir should warn
        assert_eq!(result.diagnostics.len(), 1);
    }
}
