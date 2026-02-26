//! MAKE007: Silent recipe errors (missing @ prefix)
//!
//! **Rule**: Detect echo/printf commands without @ prefix for silent output
//!
//! **Why this matters**:
//! By default, Make prints every command before executing it. For echo/printf
//! commands, this creates duplicate output (the command itself + its output).
//! Using @ prefix silences the command printing, showing only the output.
//!
//! **Auto-fix**: Add @ prefix to echo/printf commands
//!
//! ## Examples
//!
//! ❌ **BAD** (without @ prefix - duplicate output):
//! ```makefile
//! build:
//! \techo "Building..."
//! # Output:
//! # echo "Building..."
//! # Building...
//! ```
//!
//! ✅ **GOOD** (with @ prefix - clean output):
//! ```makefile
//! build:
//! \t@echo "Building..."
//! # Output:
//! # Building...
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Commands that should typically be silent
const SILENT_COMMANDS: &[&str] = &["echo", "printf"];

/// Check for echo/printf commands without @ prefix
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        // Only check recipe lines (start with tab)
        if !line.starts_with('\t') {
            continue;
        }

        // Skip if this is a shell continuation line (previous line ended with \)
        if line_num > 0 && is_continuation_line(&lines, line_num) {
            continue;
        }

        // Check if line contains echo or printf without @ prefix
        if let Some(diag) = check_recipe_line(line, line_num) {
            result.add(diag);
        }
    }

    result
}

/// Check if this line is a shell script continuation
fn is_continuation_line(lines: &[&str], line_num: usize) -> bool {
    if line_num == 0 {
        return false;
    }

    let prev_line = lines[line_num - 1].trim_end();

    // Previous line ends with backslash = continuation
    if prev_line.ends_with('\\') {
        return true;
    }

    false
}

/// Check a recipe line for echo/printf without @ prefix
fn check_recipe_line(line: &str, line_num: usize) -> Option<Diagnostic> {
    let trimmed = line.trim_start_matches('\t').trim_start();

    // Already has @ prefix - OK
    if trimmed.starts_with('@') {
        return None;
    }

    // Check if this is an echo or printf command
    for cmd in SILENT_COMMANDS {
        if is_command(trimmed, cmd) {
            let span = Span::new(line_num + 1, 1, line_num + 1, line.len() + 1);
            let fix_replacement = line.replacen('\t', "\t@", 1);

            return Some(
                Diagnostic::new(
                    "MAKE007",
                    Severity::Warning,
                    format!(
                        "Command '{}' without @ prefix - will show duplicate output",
                        cmd
                    ),
                    span,
                )
                .with_fix(Fix::new(&fix_replacement)),
            );
        }
    }

    None
}

/// Check if trimmed line starts with the given command
fn is_command(line: &str, cmd: &str) -> bool {
    // Check if line starts with the command (as whole word)
    if line.starts_with(cmd) {
        // Ensure it's a whole word (followed by space, tab, or nothing)
        if line.len() == cmd.len() {
            return true;
        }
        let next_char = line.chars().nth(cmd.len());
        matches!(next_char, Some(' ' | '\t'))
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write failing tests first

    #[test]
    fn test_MAKE007_detects_echo_without_at() {
        let makefile = "build:\n\techo Building...";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "MAKE007");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.contains("@"));
    }

    #[test]
    fn test_MAKE007_no_warning_with_at_prefix() {
        let makefile = "build:\n\t@echo Building...";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE007_provides_fix() {
        let makefile = "build:\n\techo Building...";
        let result = check(makefile);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert!(fix.replacement.contains("@echo"));
    }

    #[test]
    fn test_MAKE007_detects_printf_without_at() {
        let makefile = "test:\n\tprintf \"Testing...\\n\"";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE007_detects_multiple_echo() {
        let makefile = "build:\n\techo Starting...\n\tgcc main.c\n\techo Done!";
        let result = check(makefile);

        // Should detect both echo commands
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_MAKE007_no_warning_for_non_echo_commands() {
        let makefile = "build:\n\tgcc main.c -o app";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE007_detects_echo_with_flags() {
        let makefile = "build:\n\techo -n Building...";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE007_empty_makefile() {
        let makefile = "";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE007_no_warning_for_continuation_lines() {
        // Shell conditionals with backslash continuations
        let makefile = "check:\n\t@if test -f file; then \\\n\t\techo Found; \\\n\telse \\\n\t\techo Not found; \\\n\tfi";
        let result = check(makefile);

        // Should not warn about echo in continuation lines
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Continuation lines should not trigger MAKE007"
        );
    }

    #[test]
    fn test_MAKE007_no_warning_for_multiline_shell() {
        // Common pattern: if/then/else with backslash continuations
        let makefile = r#"validate:
	@if command -v tool >/dev/null 2>&1; then \
		echo Tool found; \
	else \
		echo Tool not found; \
	fi"#;
        let result = check(makefile);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Should not warn about echo in shell conditionals"
        );
    }

    #[test]
    fn test_MAKE007_warns_for_top_level_echo() {
        // Top-level echo without @ should still warn
        let makefile = "build:\n\techo Starting\n\t@if true; then \\\n\t\techo Inside; \\\n\tfi";
        let result = check(makefile);

        // Should only warn about the first echo, not the one in continuation
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("echo"));
    }
}
