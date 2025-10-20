//! MAKE009: Hardcoded paths (non-portable)
//!
//! **Rule**: Detect hardcoded installation paths that reduce portability
//!
//! **Why this matters**:
//! Hardcoding paths like /usr/local makes Makefiles non-portable. Different
//! systems use different install prefixes. Using variables like $(PREFIX)
//! allows users to customize installation locations.
//!
//! **Auto-fix**: Suggest using $(PREFIX) variable
//!
//! ## Examples
//!
//! ❌ **BAD** (hardcoded path):
//! ```makefile
//! install:
//! \tcp app /usr/local/bin/app
//! ```
//!
//! ✅ **GOOD** (with variable):
//! ```makefile
//! PREFIX ?= /usr/local
//!
//! install:
//! \tcp app $(PREFIX)/bin/app
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Hardcoded paths that should use variables
const HARDCODED_PATHS: &[&str] = &[
    "/usr/local/bin",
    "/usr/local/lib",
    "/usr/local/include",
    "/usr/local/share",
];

/// Check for hardcoded installation paths
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Only check recipe lines (start with tab)
        if !line.starts_with('\t') {
            continue;
        }

        // Check if line contains hardcoded paths
        for path in HARDCODED_PATHS {
            if line.contains(path) {
                let span = Span::new(line_num + 1, 1, line_num + 1, line.len() + 1);

                // Create fix by replacing hardcoded path with $(PREFIX)
                let fix_replacement = create_fix(line, path);

                let diag = Diagnostic::new(
                    "MAKE009",
                    Severity::Warning,
                    &format!(
                        "Hardcoded path '{}' reduces portability - consider using $(PREFIX)",
                        path
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

/// Create fix by replacing hardcoded /usr/local with $(PREFIX)
fn create_fix(line: &str, hardcoded_path: &str) -> String {
    // Replace /usr/local/bin with $(PREFIX)/bin, etc.
    line.replace("/usr/local", "$(PREFIX)")
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write failing tests first

    #[test]
    fn test_MAKE009_detects_hardcoded_usr_local_bin() {
        let makefile = "install:\n\tcp app /usr/local/bin/app";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "MAKE009");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.to_lowercase().contains("hardcoded"));
    }

    #[test]
    fn test_MAKE009_no_warning_with_prefix_variable() {
        let makefile = "PREFIX ?= /usr/local\n\ninstall:\n\tcp app $(PREFIX)/bin/app";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE009_provides_fix() {
        let makefile = "install:\n\tcp app /usr/local/bin/app";
        let result = check(makefile);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert!(fix.replacement.contains("$(PREFIX)"));
    }

    #[test]
    fn test_MAKE009_detects_hardcoded_lib_path() {
        let makefile = "install:\n\tcp libfoo.so /usr/local/lib/libfoo.so";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE009_detects_multiple_hardcoded_paths() {
        let makefile = "install:\n\tcp app /usr/local/bin/app\n\tcp lib.so /usr/local/lib/lib.so";
        let result = check(makefile);

        // Should detect both hardcoded paths
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_MAKE009_no_warning_for_non_usr_local() {
        let makefile = "install:\n\tcp app /opt/bin/app";
        let result = check(makefile);

        // /opt is not in our hardcoded paths list
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE009_detects_include_path() {
        let makefile = "install:\n\tcp header.h /usr/local/include/header.h";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE009_empty_makefile() {
        let makefile = "";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }
}
