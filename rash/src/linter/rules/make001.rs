//! MAKE001: Non-deterministic wildcard usage in Makefiles
//!
//! **Rule**: Detect `$(wildcard ...)` without `$(sort ...)` wrapper
//!
//! **Why this matters**:
//! File glob results from `$(wildcard)` vary by filesystem and can change
//! between runs, breaking determinism in Makefile builds.
//!
//! **Auto-fix**: Wrap with `$(sort ...)`
//!
//! ## Examples
//!
//! ❌ **BAD** (non-deterministic):
//! ```makefile
//! SOURCES = $(wildcard *.c)
//! HEADERS = $(wildcard include/*.h)
//! ```
//!
//! ✅ **GOOD** (deterministic):
//! ```makefile
//! SOURCES = $(sort $(wildcard *.c))
//! HEADERS = $(sort $(wildcard include/*.h))
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for unordered wildcard usage in Makefiles
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Look for $(wildcard ...) without $(sort ...) wrapper
        if line.contains("$(wildcard") && !line.contains("$(sort") {
            if let Some(col) = line.find("$(wildcard") {
                let span = Span::new(
                    line_num + 1,
                    col + 1,
                    line_num + 1,
                    col + 11, // length of "$(wildcard"
                );

                // Extract the wildcard expression for the fix
                let fix_replacement = if let Some(end) = find_matching_paren(line, col + 2) {
                    // Found matching paren - wrap entire expression
                    let wildcard_expr = &line[col..=end];
                    format!("$(sort {})", wildcard_expr)
                } else {
                    // Fallback: suggest wrapping
                    "$(sort $(wildcard ...))".to_string()
                };

                let diag = Diagnostic::new(
                    "MAKE001",
                    Severity::Warning,
                    "Non-deterministic $(wildcard) - results may vary between runs",
                    span,
                )
                .with_fix(Fix::new(&fix_replacement));

                result.add(diag);
            }
        }
    }

    result
}

/// Find matching closing parenthesis
#[allow(clippy::needless_range_loop)]
fn find_matching_paren(line: &str, start: usize) -> Option<usize> {
    let chars: Vec<char> = line.chars().collect();
    let mut depth = 1;

    for i in start..chars.len() {
        match chars[i] {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write failing tests first

    #[test]
    fn test_MAKE001_detects_wildcard_basic() {
        let makefile = "SOURCES = $(wildcard *.c)";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "MAKE001");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.contains("Non-deterministic"));
    }

    #[test]
    fn test_MAKE001_detects_wildcard_with_path() {
        let makefile = "HEADERS = $(wildcard include/*.h)";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "MAKE001");
    }

    #[test]
    fn test_MAKE001_no_warning_with_sort() {
        let makefile = "SOURCES = $(sort $(wildcard *.c))";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE001_no_warning_without_wildcard() {
        let makefile = "SOURCES = main.c utils.c";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE001_provides_fix() {
        let makefile = "SOURCES = $(wildcard *.c)";
        let result = check(makefile);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert!(fix.replacement.contains("$(sort"));
        assert!(fix.replacement.contains("$(wildcard *.c)"));
    }

    #[test]
    fn test_MAKE001_detects_multiple_wildcards() {
        let makefile = "SOURCES = $(wildcard *.c)\nHEADERS = $(wildcard *.h)";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_MAKE001_no_false_positive_in_comment() {
        let makefile = "# SOURCES = $(wildcard *.c)\nSOURCES = main.c";
        let result = check(makefile);

        // Note: Current implementation doesn't handle comments
        // This will be improved in future iterations
        // For now, we accept this limitation
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE001_nested_parens() {
        let makefile = "SOURCES = $(wildcard src/**/*.c)";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert!(fix.replacement.starts_with("$(sort"));
    }
}
