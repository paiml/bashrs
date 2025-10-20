//! MAKE019: Environment variable pollution
//!
//! **Rule**: Detect unnecessary export statements that pollute the environment
//!
//! **Why this matters**:
//! Using `export` in Make exports variables to all sub-processes, which can:
//! - Pollute the environment with unnecessary variables
//! - Slow down Make (exported vars passed to every command)
//! - Cause unexpected behavior in subprocesses
//! - Make builds less reproducible
//!
//! Only export variables that subprocesses actually need.
//!
//! **Auto-fix**: Remove export keyword (keep variable assignment)
//!
//! ## Examples
//!
//! ❌ **BAD** (unnecessary export):
//! ```makefile
//! export CC = gcc
//! export CFLAGS = -Wall
//! ```
//!
//! ✅ **GOOD** (no export - variables only for Make):
//! ```makefile
//! CC = gcc
//! CFLAGS = -Wall
//! ```
//!
//! ✅ **GOOD** (export when needed by subprocesses):
//! ```makefile
//! export PATH := $(PATH):/usr/local/bin
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Variables that commonly should NOT be exported (Make-internal)
const INTERNAL_VARS: &[&str] = &[
    "CC", "CXX", "AR", "LD", "AS", // Compilers/linkers
    "CFLAGS", "CXXFLAGS", "LDFLAGS", // Compiler flags
    "SOURCES", "OBJECTS", "TARGET", // Build artifacts
    "PREFIX", "DESTDIR", "BINDIR", // Installation paths
];

/// Check for unnecessary export statements
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Check if line starts with "export "
        let trimmed = line.trim();
        if !trimmed.starts_with("export ") {
            continue;
        }

        // Extract variable name from export statement
        if let Some(var_name) = extract_var_name(trimmed) {
            // Check if this is an internal variable that shouldn't be exported
            if is_internal_variable(&var_name) {
                let span = Span::new(line_num + 1, 1, line_num + 1, line.len());
                let fix_replacement = create_fix(line);

                let diag = Diagnostic::new(
                    "MAKE019",
                    Severity::Warning,
                    &format!("Unnecessary export of '{}' - variable is Make-internal and doesn't need to be in environment", var_name),
                    span,
                )
                .with_fix(Fix::new(&fix_replacement));

                result.add(diag);
            }
        }
    }

    result
}

/// Extract variable name from export statement
/// e.g., "export CC = gcc" → "CC"
fn extract_var_name(line: &str) -> Option<String> {
    // Remove "export " prefix
    let after_export = line.strip_prefix("export ")?;

    // Extract variable name (before = or :=)
    if let Some(eq_pos) = after_export.find('=') {
        let var_name = after_export[..eq_pos].trim();
        return Some(var_name.to_string());
    }

    None
}

/// Check if variable is internal (shouldn't be exported)
fn is_internal_variable(var_name: &str) -> bool {
    // Check against list of known internal variables
    for internal_var in INTERNAL_VARS {
        if var_name == *internal_var {
            return true;
        }
    }

    // PATH and other environment variables should be allowed to export
    // Only flag Make-specific build variables
    false
}

/// Create a fix by removing "export " keyword
fn create_fix(line: &str) -> String {
    line.replace("export ", "")
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write failing tests first

    #[test]
    fn test_MAKE019_detects_exported_cc() {
        let makefile = "export CC = gcc\n\nall:\n\t$(CC) main.c";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "MAKE019");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(
            diag.message.to_lowercase().contains("export")
                || diag.message.to_lowercase().contains("variable")
        );
    }

    #[test]
    fn test_MAKE019_detects_exported_cflags() {
        let makefile = "export CFLAGS = -Wall\n\nall:\n\t$(CC) $(CFLAGS) main.c";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE019_detects_multiple_exports() {
        let makefile = "export CC = gcc\nexport CFLAGS = -Wall\n\nall:\n\t$(CC) $(CFLAGS) main.c";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_MAKE019_provides_fix() {
        let makefile = "export CC = gcc\n\nall:\n\t$(CC) main.c";
        let result = check(makefile);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        // Fix should remove export keyword
        assert!(!fix.replacement.contains("export CC"));
        assert!(fix.replacement.contains("CC = gcc"));
    }

    #[test]
    fn test_MAKE019_no_warning_for_non_exported() {
        let makefile = "CC = gcc\nCFLAGS = -Wall\n\nall:\n\t$(CC) $(CFLAGS) main.c";
        let result = check(makefile);

        // No export - OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE019_no_warning_for_path() {
        let makefile = "export PATH := $(PATH):/usr/local/bin\n\nall:\n\t./script.sh";
        let result = check(makefile);

        // PATH export is OK (commonly needed by subprocesses)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE019_detects_exported_sources() {
        let makefile = "export SOURCES = main.c utils.c\n\nall:\n\t$(CC) $(SOURCES)";
        let result = check(makefile);

        // SOURCES is Make-internal, shouldn't be exported
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE019_empty_makefile() {
        let makefile = "";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }
}
