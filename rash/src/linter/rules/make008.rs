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
//!     gcc main.c -o app
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check if line is a target definition (contains ':' and doesn't start with whitespace)
fn is_target_line(line: &str) -> bool {
    line.contains(':')
        && !line.starts_with(char::is_whitespace)
        && !line.trim_start().starts_with('#')
}

/// Extract target name from target line
fn extract_target_name(line: &str) -> Option<String> {
    line.find(':')
        .map(|colon_pos| line[..colon_pos].trim().to_string())
}

/// Check if line is a recipe line that starts with spaces (error)
fn is_recipe_with_spaces(line: &str) -> bool {
    line.starts_with(' ') && !line.starts_with('\t')
}

/// Count leading spaces in line
fn count_leading_spaces(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ').count()
}

/// Create fix replacement with tab
fn create_tab_fix(line: &str) -> String {
    let rest_of_line = line.trim_start();
    format!("\t{}", rest_of_line)
}

/// Build diagnostic for recipe line with spaces
fn build_diagnostic(
    line_num: usize,
    leading_spaces: usize,
    fix_replacement: &str,
    current_target: &str,
) -> Diagnostic {
    let span = Span::new(line_num + 1, 1, line_num + 1, leading_spaces + 1);

    let message = if !current_target.is_empty() {
        format!(
            "Recipe line starts with spaces instead of tab (fatal Make error) in target '{}'",
            current_target
        )
    } else {
        "Recipe line starts with spaces instead of tab (fatal Make error)".to_string()
    };

    Diagnostic::new("MAKE008", Severity::Error, message, span).with_fix(Fix::new(fix_replacement))
}

/// Check if line should exit recipe state
fn should_exit_recipe(line: &str) -> bool {
    !line.starts_with('\t') && !line.starts_with(' ')
}

/// Check if line is empty or comment (stay in recipe state)
fn is_empty_or_comment(line: &str) -> bool {
    line.trim().is_empty() || line.trim_start().starts_with('#')
}

/// Check if previous line is a continuation line (ends with \)
fn is_continuation_line(line: &str) -> bool {
    line.trim_end().ends_with('\\')
}

/// Check for spaces instead of tabs in recipe lines
/// F039 FIX: Handle continuation lines - don't flag them as recipe errors
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut in_recipe = false;
    let mut current_target = String::new();
    let mut in_continuation = false;

    for (line_num, line) in lines.iter().enumerate() {
        // F039 FIX: If we're in a continuation, skip this line
        if in_continuation {
            // Check if this line also continues
            in_continuation = is_continuation_line(line);
            continue;
        }

        // Check if this line starts a continuation
        if is_continuation_line(line) {
            in_continuation = true;
            // Still need to check if it's a target line
            if is_target_line(line) {
                if let Some(target) = extract_target_name(line) {
                    current_target = target;
                    in_recipe = true;
                }
            }
            continue;
        }

        if is_target_line(line) {
            if let Some(target) = extract_target_name(line) {
                current_target = target;
                in_recipe = true;
            }
        } else if in_recipe && !line.is_empty() && !line.trim().is_empty() {
            if is_recipe_with_spaces(line) {
                let leading_spaces = count_leading_spaces(line);
                let fix_replacement = create_tab_fix(line);
                let diag =
                    build_diagnostic(line_num, leading_spaces, &fix_replacement, &current_target);
                result.add(diag);
            } else if should_exit_recipe(line) {
                in_recipe = false;
                current_target.clear();
            }
        } else if is_empty_or_comment(line) {
            // Stay in recipe state
        } else if should_exit_recipe(line) {
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

    /// F039: MAKE008 must handle continuation lines
    /// Issue #121: MAKE008 triggers on .PHONY continuation
    #[test]
    fn test_F039_MAKE008_phony_continuation() {
        // .PHONY with line continuation - should NOT trigger MAKE008
        let makefile = r#".PHONY: clean \
        test \
        install

clean:
	rm -f *.o"#;
        let result = check(makefile);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "F039 FALSIFIED: MAKE008 must NOT flag continuation lines. Got: {:?}",
            result.diagnostics
        );
    }

    /// F039 variation: Target with line continuation
    #[test]
    fn test_F039_MAKE008_target_continuation() {
        // Target with continuation - should NOT flag the continuation as recipe with spaces
        let makefile = r#"SRCS = main.c \
       utils.c \
       helpers.c

build:
	gcc $(SRCS) -o app"#;
        let result = check(makefile);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "F039 FALSIFIED: MAKE008 must NOT flag variable continuation lines. Got: {:?}",
            result.diagnostics
        );
    }
}
