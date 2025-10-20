//! MAKE012: Recursive make considered harmful
//!
//! **Rule**: Detect recursive make invocations that can cause build issues
//!
//! **Why this matters**:
//! Recursive make (calling make from within make) is problematic:
//! - Breaks dependency tracking across subdirectories
//! - Prevents parallel builds from working correctly
//! - Makes it impossible to accurately track what needs rebuilding
//! - Famous paper: "Recursive Make Considered Harmful" by Peter Miller
//!
//! **Auto-fix**: Suggest using include directives or non-recursive make
//!
//! ## Examples
//!
//! ❌ **BAD** (recursive make):
//! ```makefile
//! subdirs:
//! \t$(MAKE) -C subdir1
//! \t$(MAKE) -C subdir2
//! ```
//!
//! ✅ **GOOD** (non-recursive with include):
//! ```makefile
//! include subdir1/module.mk
//! include subdir2/module.mk
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Patterns that indicate recursive make
const RECURSIVE_MAKE_PATTERNS: &[&str] = &["$(MAKE)", "${MAKE}", "make -C", "make --directory"];

/// Check for recursive make invocations
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Only check recipe lines (start with tab)
        if !line.starts_with('\t') {
            continue;
        }

        // Check if line contains recursive make patterns
        for pattern in RECURSIVE_MAKE_PATTERNS {
            if line.contains(pattern) {
                let span = Span::new(line_num + 1, 1, line_num + 1, line.len() + 1);

                // Create fix suggesting include directive
                let fix_replacement = create_fix(line);

                let diag = Diagnostic::new(
                    "MAKE012",
                    Severity::Warning,
                    "Recursive make invocation - consider using 'include' directives (see 'Recursive Make Considered Harmful')",
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

/// Create fix by suggesting include directive instead of recursive make
fn create_fix(line: &str) -> String {
    // Extract subdirectory if present
    let subdir = extract_subdir(line);

    if let Some(dir) = subdir {
        format!("# Consider: include {}/module.mk", dir)
    } else {
        "# Consider: include subdirs.mk or use non-recursive make".to_string()
    }
}

/// Extract subdirectory from make -C or --directory command
fn extract_subdir(line: &str) -> Option<String> {
    // Look for -C <dir> pattern
    if let Some(pos) = line.find("-C ") {
        let after_c = &line[pos + 3..];
        let dir = after_c.split_whitespace().next()?;
        return Some(dir.to_string());
    }

    // Look for --directory=<dir> pattern
    if let Some(pos) = line.find("--directory=") {
        let after_dir = &line[pos + 12..];
        let dir = after_dir.split_whitespace().next()?;
        return Some(dir.to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write failing tests first

    #[test]
    fn test_MAKE012_detects_dollar_make() {
        let makefile = "subdirs:\n\t$(MAKE) -C subdir";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "MAKE012");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.to_lowercase().contains("recursive"));
    }

    #[test]
    fn test_MAKE012_detects_curly_brace_make() {
        let makefile = "subdirs:\n\t${MAKE} -C subdir";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE012_detects_make_dash_c() {
        let makefile = "subdirs:\n\tmake -C subdir";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE012_provides_fix() {
        let makefile = "subdirs:\n\t$(MAKE) -C subdir";
        let result = check(makefile);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert!(fix.replacement.contains("include"));
    }

    #[test]
    fn test_MAKE012_detects_multiple_recursive_makes() {
        let makefile = "subdirs:\n\t$(MAKE) -C sub1\n\t$(MAKE) -C sub2";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_MAKE012_no_warning_for_regular_commands() {
        let makefile = "build:\n\tgcc main.c -o app";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE012_detects_make_directory() {
        let makefile = "subdirs:\n\tmake --directory=subdir all";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE012_empty_makefile() {
        let makefile = "";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }
}
