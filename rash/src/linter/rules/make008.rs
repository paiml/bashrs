//! MAKE008: Tab vs spaces in recipes (CRITICAL)
//!
//! **Rule**: Detect spaces instead of tabs in recipe lines (fatal Make error)
//!
//! **Why this matters**:
//! GNU Make REQUIRES recipe lines to start with a TAB character (ASCII 0x09).
//! Using spaces instead of tabs causes a fatal "missing separator" error.
//! This is one of the most common and frustrating Make errors.
//!
//! **Auto-fix**: Replace leading spaces with a single tab
//!
//! ## Examples
//!
//! ❌ **BAD** (spaces - will fail):
//! ```makefile
//! build:
//!     gcc main.c -o app
//! ```
//!
//! ✅ **GOOD** (tab):
//! ```makefile
//! build:
//! 	gcc main.c -o app
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for spaces instead of tabs in recipe lines
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let lines: Vec<&str> = source.lines().collect();
    let mut in_recipe = false;
    let mut current_target = String::new();

    for (line_num, line) in lines.iter().enumerate() {
        // Check if this is a target line (contains ':' and doesn't start with whitespace)
        if line.contains(':')
            && !line.starts_with(char::is_whitespace)
            && !line.trim_start().starts_with('#')
        {
            // Extract target name before ':'
            if let Some(colon_pos) = line.find(':') {
                current_target = line[..colon_pos].trim().to_string();
                in_recipe = true;
            }
        }
        // Check if this is a recipe line (should start with tab)
        else if in_recipe && !line.is_empty() && !line.trim().is_empty() {
            // Check if line starts with spaces instead of tab
            if line.starts_with(' ') && !line.starts_with('\t') {
                // This is an error - recipe line must start with tab
                let leading_spaces = line.chars().take_while(|c| *c == ' ').count();
                let rest_of_line = line.trim_start();

                let span = Span::new(line_num + 1, 1, line_num + 1, leading_spaces + 1);

                // Create fix: replace leading spaces with a single tab
                let fix_replacement = format!("\t{}", rest_of_line);

                let diag = Diagnostic::new(
                    "MAKE008",
                    Severity::Error, // This is a CRITICAL error
                    &format!(
                        "Recipe line starts with spaces instead of tab (fatal Make error){}",
                        if !current_target.is_empty() {
                            format!(" in target '{}'", current_target)
                        } else {
                            String::new()
                        }
                    ),
                    span,
                )
                .with_fix(Fix::new(&fix_replacement));

                result.add(diag);
            }
            // If line doesn't start with tab or space, we've left the recipe
            else if !line.starts_with('\t') && !line.starts_with(' ') {
                in_recipe = false;
                current_target.clear();
            }
        }
        // Empty line or comment - stay in recipe state
        else if line.trim().is_empty() || line.trim_start().starts_with('#') {
            // Don't change in_recipe state
        }
        // Non-empty, non-recipe line - we've left the recipe
        else if !line.starts_with('\t') && !line.starts_with(' ') {
            in_recipe = false;
            current_target.clear();
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write failing tests first

    #[test]
    fn test_MAKE008_detects_spaces_in_recipe() {
        let makefile = "build:\n    gcc main.c -o app";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "MAKE008");
        assert_eq!(diag.severity, Severity::Error); // CRITICAL error
        assert!(diag.message.contains("tab"));
    }

    #[test]
    fn test_MAKE008_detects_multiple_space_lines() {
        let makefile = "build:\n    gcc main.c\n    strip app";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_MAKE008_no_warning_with_tab() {
        let makefile = "build:\n\tgcc main.c -o app";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE008_provides_fix() {
        let makefile = "build:\n    gcc main.c -o app";
        let result = check(makefile);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert!(fix.replacement.starts_with('\t'));
    }

    #[test]
    fn test_MAKE008_no_false_positive_on_target_line() {
        let makefile = "build: main.c utils.c\n\tgcc main.c utils.c -o app";
        let result = check(makefile);

        // Target line can have spaces, only recipe lines need tabs
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE008_detects_mixed_spaces_tabs() {
        let makefile = "build:\n  \tgcc main.c"; // 2 spaces + tab
        let result = check(makefile);

        // Should warn about leading spaces before tab
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE008_no_warning_for_empty_lines() {
        let makefile = "build:\n\tgcc main.c\n\ninstall:\n\tcp app /usr/bin";
        let result = check(makefile);

        // Empty lines between targets are OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE008_multiple_targets() {
        let makefile = r#"build:
	gcc main.c

install:
    cp app /usr/bin"#;
        let result = check(makefile);

        // Only install target has space error
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("install"));
    }
}
