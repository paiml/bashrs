//! MAKE003: Unsafe variable expansion in Makefile recipes
//!
//! **Rule**: Detect unquoted variables in shell commands that could cause issues
//!
//! **Why this matters**:
//! Unquoted variables in shell commands can lead to word splitting and
//! globbing issues, especially with rm, cp, and other file operations.
//!
//! **Auto-fix**: Add quotes around variable
//!
//! ## Examples
//!
//! ❌ **BAD** (unsafe):
//! ```makefile
//! clean:
//!     rm -rf $BUILD_DIR
//! ```
//!
//! ✅ **GOOD** (safe):
//! ```makefile
//! clean:
//!     rm -rf "$BUILD_DIR"
//!     rm -rf "$(BUILD_DIR)"
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for unquoted variable expansions in Makefile recipes
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Check if line starts with tab (recipe line)
        if line.starts_with('\t') {
            // Look for dangerous commands with unquoted variables
            let dangerous_commands = ["rm", "cp", "mv", "chmod", "chown"];

            for cmd in &dangerous_commands {
                if line.contains(cmd) {
                    // Look for $VAR or $(VAR) without quotes
                    check_unquoted_vars(line, line_num, &mut result);
                    break;
                }
            }
        }
    }

    result
}

/// Check if character at position is already quoted
fn is_quoted_before(chars: &[char], pos: usize) -> bool {
    if pos == 0 {
        return false;
    }
    let before = chars[pos - 1];
    before == '"' || before == '\''
}

/// Check if character at position is quoted after
fn is_quoted_after(chars: &[char], pos: usize) -> bool {
    if pos >= chars.len() {
        return false;
    }
    let after = chars[pos];
    after == '"' || after == '\''
}

/// Parse variable reference and return (start, end) positions
/// Returns None if not a valid variable reference
fn parse_variable_reference(chars: &[char], i: usize) -> Option<(usize, usize)> {
    let var_start = i;

    if i + 1 >= chars.len() {
        return None;
    }

    if chars[i + 1] == '(' || chars[i + 1] == '{' {
        // $(VAR) or ${VAR}
        let closing = if chars[i + 1] == '(' { ')' } else { '}' };
        find_closing_char(chars, i + 2, closing).map(|end_pos| (var_start, end_pos + 1))
    } else {
        // $VAR
        let mut end = i + 1;
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }
        Some((var_start, end))
    }
}

/// Create diagnostic for unquoted variable
fn create_unquoted_var_diagnostic(
    chars: &[char],
    start: usize,
    end: usize,
    line_num: usize,
) -> Diagnostic {
    let span = Span::new(line_num + 1, start + 1, line_num + 1, end + 1);
    let var_text: String = chars[start..end].iter().collect();
    let fix_replacement = format!("\"{}\"", var_text);

    Diagnostic::new(
        "MAKE003",
        Severity::Warning,
        "Unquoted variable in command - may cause word splitting issues",
        span,
    )
    .with_fix(Fix::new(&fix_replacement))
}

fn check_unquoted_vars(line: &str, line_num: usize, result: &mut LintResult) {
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    let mut in_double_quote = false;
    let mut in_single_quote = false;

    while i < chars.len() {
        let ch = chars[i];

        // Track quote state (F037 fix: proper quoted context tracking)
        if ch == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            i += 1;
            continue;
        }
        if ch == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            i += 1;
            continue;
        }

        if chars[i] == '$' && i + 1 < chars.len() {
            // F037 FIX: If we're inside a quoted string, skip this variable
            if in_double_quote || in_single_quote {
                i += 1;
                continue;
            }

            // Skip if already quoted before (adjacent quote)
            if is_quoted_before(&chars, i) {
                i += 1;
                continue;
            }

            // Parse variable reference
            if let Some((start, end)) = parse_variable_reference(&chars, i) {
                i = end;

                // Check if quoted after
                if !is_quoted_after(&chars, end) {
                    let diag = create_unquoted_var_diagnostic(&chars, start, end, line_num);
                    result.add(diag);
                }
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
}

#[allow(clippy::needless_range_loop)]
fn find_closing_char(chars: &[char], start: usize, closing: char) -> Option<usize> {
    let mut depth = 1;
    for i in start..chars.len() {
        if chars[i] == '(' || chars[i] == '{' {
            depth += 1;
        } else if chars[i] == closing {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_MAKE003_detects_unquoted_var_in_rm() {
        let makefile = "clean:\n\trm -rf $BUILD_DIR";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "MAKE003");
        assert_eq!(diag.severity, Severity::Warning);
    }

    #[test]
    fn test_MAKE003_no_warning_with_quotes() {
        let makefile = "clean:\n\trm -rf \"$BUILD_DIR\"";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE003_detects_paren_syntax() {
        let makefile = "clean:\n\trm -rf $(BUILD_DIR)";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE003_no_warning_paren_quoted() {
        let makefile = "clean:\n\trm -rf \"$(BUILD_DIR)\"";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE003_provides_fix() {
        let makefile = "clean:\n\trm -rf $BUILD_DIR";
        let result = check(makefile);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert!(fix.replacement.contains("\"$BUILD_DIR\""));
    }

    #[test]
    fn test_MAKE003_no_false_positive_outside_recipe() {
        let makefile = "BUILD_DIR = $HOME/build";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE003_detects_cp_command() {
        let makefile = "install:\n\tcp $SOURCE $DEST";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 2); // Both variables
    }

    #[test]
    fn test_MAKE003_no_warning_safe_commands() {
        let makefile = "build:\n\techo $MESSAGE";
        let result = check(makefile);

        // echo is safe, shouldn't warn
        assert_eq!(result.diagnostics.len(), 0);
    }

    /// F037: MAKE003 must recognize quoted context - variable inside quoted string
    /// Issue #118: False positive for quoted variables
    #[test]
    fn test_F037_MAKE003_quoted_context() {
        // Variable inside a quoted string - should NOT trigger warning
        let makefile = r#"clean:
	rm -rf "path/$(BUILD_DIR)/output""#;
        let result = check(makefile);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "F037 FALSIFIED: MAKE003 must NOT flag variables inside quoted strings. Got: {:?}",
            result.diagnostics
        );
    }

    /// F037 variation: Multiple variables in quoted string
    #[test]
    fn test_F037_MAKE003_multiple_vars_in_quoted_string() {
        let makefile = r#"install:
	cp "$(SRC)/file" "$(DEST)/file""#;
        let result = check(makefile);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "F037 FALSIFIED: Multiple variables in quoted strings should not be flagged. Got: {:?}",
            result.diagnostics
        );
    }
}
