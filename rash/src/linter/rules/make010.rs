//! MAKE010: Missing error handling (|| exit 1)
//!
//! **Rule**: Detect commands without error handling in recipes
//!
//! **Why this matters**:
//! By default, Make only stops on error if the recipe command returns non-zero.
//! However, some commands may fail silently or have side effects that should
//! stop the build. Adding `|| exit 1` ensures the build stops on failure.
//!
//! **Auto-fix**: Add `|| exit 1` to commands that should fail the build
//!
//! ## Examples
//!
//! ❌ **BAD** (no error handling):
//! ```makefile
//! install:
//!     cp app /usr/bin/app
//!     chmod +x /usr/bin/app
//! ```
//!
//! ✅ **GOOD** (with error handling):
//! ```makefile
//! install:
//!     cp app /usr/bin/app || exit 1
//!     chmod +x /usr/bin/app || exit 1
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Commands that should have error handling
const CRITICAL_COMMANDS: &[&str] = &[
    "cp", "mv", "rm", "install", "chmod", "chown", "ln", "mkdir", "curl", "wget", "git",
];

/// Check for missing error handling in recipe commands
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Only check recipe lines (start with tab)
        if !line.starts_with('\t') {
            continue;
        }

        let recipe = line.trim();

        // Skip if already has error handling
        if has_error_handling(recipe) {
            continue;
        }

        // Check if line contains a critical command
        if let Some(cmd) = find_critical_command(recipe) {
            let cmd_pos = line.find(cmd).unwrap_or(0);
            let span = Span::new(
                line_num + 1,
                cmd_pos + 1,
                line_num + 1,
                cmd_pos + cmd.len() + 1,
            );

            // Create fix by adding || exit 1
            let fix_replacement = format!("{} || exit 1", line.trim_start());

            let diag = Diagnostic::new(
                "MAKE010",
                Severity::Warning,
                format!(
                    "Command '{}' missing error handling - consider adding '|| exit 1'",
                    cmd
                ),
                span,
            )
            .with_fix(Fix::new(&fix_replacement));

            result.add(diag);
        }
    }

    result
}

/// Check if a recipe line already has error handling
fn has_error_handling(recipe: &str) -> bool {
    recipe.contains("|| exit") || recipe.contains("set -e") || recipe.contains("&&")
}

/// Find if the recipe contains a critical command
///
/// Returns None if the command keyword is inside:
/// - Quoted strings (echo "install", printf 'rm -rf')
/// - Variable assignments (MSG="install here")
/// - Heredocs
/// - Comments
fn find_critical_command(recipe: &str) -> Option<&'static str> {
    // Skip if this is an echo/printf/cat command with quoted strings
    let trimmed = recipe.trim_start_matches('@').trim_start();

    // Check if line starts with echo, printf, or cat (output commands)
    if trimmed.starts_with("echo ") || trimmed.starts_with("printf ") || trimmed.starts_with("cat ")
    {
        return None;
    }

    // Check if this is a variable assignment (VAR="..." or VAR='...')
    if is_variable_assignment(trimmed) {
        return None;
    }

    // Check if we're in a heredoc context
    if trimmed.contains("<<") {
        return None;
    }

    // Now check for actual critical commands
    CRITICAL_COMMANDS
        .iter()
        .find(|&cmd| {
            recipe.split_whitespace().any(|word| {
                word == *cmd
                    || word.starts_with(&format!("{}@", cmd))
                    || word.starts_with(&format!("{}-", cmd))
            })
        })
        .map(|v| v as _)
}

/// Check if a line is a variable assignment
fn is_variable_assignment(line: &str) -> bool {
    // Pattern: VAR="..." or VAR='...'
    if let Some(eq_pos) = line.find('=') {
        let before_eq = &line[..eq_pos];
        // Variable name should be alphanumeric + underscore only
        let is_valid_var_name = before_eq
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '$');

        if is_valid_var_name {
            let after_eq = &line[eq_pos + 1..];
            // Check if value is quoted
            return after_eq.starts_with('"') || after_eq.starts_with('\'');
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write failing tests first

    #[test]
    fn test_MAKE010_detects_missing_error_handling() {
        let makefile = "install:\n\tcp app /usr/bin/app";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "MAKE010");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.contains("error handling"));
    }

    #[test]
    fn test_MAKE010_no_warning_with_exit_handling() {
        let makefile = "install:\n\tcp app /usr/bin/app || exit 1";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE010_provides_fix() {
        let makefile = "install:\n\tcp app /usr/bin/app";
        let result = check(makefile);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert!(fix.replacement.contains("|| exit 1"));
    }

    #[test]
    fn test_MAKE010_detects_multiple_commands() {
        let makefile = "install:\n\tcp app /usr/bin\n\tchmod +x /usr/bin/app";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_MAKE010_no_warning_for_safe_commands() {
        let makefile = "build:\n\techo Building...";
        let result = check(makefile);

        // echo doesn't need error handling
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE010_no_warning_with_set_e() {
        let makefile = "install:\n\tset -e; cp app /usr/bin/app";
        let result = check(makefile);

        // set -e provides error handling
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE010_detects_git_commands() {
        let makefile = "deploy:\n\tgit pull origin main";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE010_no_warning_with_and_chaining() {
        let makefile = "deploy:\n\tgit pull origin main && make build";
        let result = check(makefile);

        // && chaining provides implicit error handling
        assert_eq!(result.diagnostics.len(), 0);
    }

    // Issue #18: Tests for string literal detection

    #[test]
    fn test_MAKE010_no_warning_for_echo_with_command_keyword() {
        let makefile = "help:\n\t@echo \"Run: make install\"";
        let result = check(makefile);

        // Should NOT warn about 'install' in echo string
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE010_no_warning_for_printf_with_command_keyword() {
        let makefile = "help:\n\t@printf 'Use: rm -rf /tmp\\n'";
        let result = check(makefile);

        // Should NOT warn about 'rm' in printf string
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE010_no_warning_for_variable_assignment() {
        let makefile = "config:\n\t@MSG=\"install here\"";
        let result = check(makefile);

        // Should NOT warn about 'install' in variable assignment
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE010_no_warning_for_heredoc() {
        let makefile = "docs:\n\t@cat << EOF";
        let result = check(makefile);

        // Should NOT warn in heredoc context
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_is_variable_assignment_double_quotes() {
        assert!(is_variable_assignment("MSG=\"install here\""));
        assert!(is_variable_assignment("HELP=\"use rm -rf\""));
    }

    #[test]
    fn test_is_variable_assignment_single_quotes() {
        assert!(is_variable_assignment("MSG='install here'"));
        assert!(is_variable_assignment("HELP='use rm -rf'"));
    }

    #[test]
    fn test_is_variable_assignment_unquoted() {
        assert!(!is_variable_assignment("MSG=install"));
        assert!(!is_variable_assignment("HELP=rm"));
    }

    #[test]
    fn test_is_variable_assignment_shell_var() {
        assert!(is_variable_assignment("$$VAR=\"value\""));
    }

    #[test]
    fn test_is_not_variable_assignment() {
        assert!(!is_variable_assignment("echo test"));
        assert!(!is_variable_assignment("cargo install foo"));
    }

    // Property-based tests for Issue #18

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        // Generate valid command keywords
        fn command_keyword() -> impl Strategy<Value = String> {
            prop::sample::select(vec![
                "install", "cp", "mv", "rm", "chmod", "chown", "ln", "mkdir", "curl", "wget", "git",
            ])
            .prop_map(|s| s.to_string())
        }

        proptest! {
            /// Property: echo/printf with command keywords should never trigger MAKE010
            #[test]
            fn prop_echo_with_command_never_warns(
                cmd in command_keyword(),
                prefix in prop::sample::select(vec!["echo", "printf"]),
                text in "[a-zA-Z0-9 ]+",
            ) {
                let recipe = format!("\t@{} \"{}. Use: {} here\"", prefix, text, cmd);
                let makefile = format!("target:\n{}", recipe);
                let result = check(&makefile);

                // Should NOT trigger MAKE010 for command in echo/printf
                let make010_count = result.diagnostics.iter()
                    .filter(|d| d.code == "MAKE010")
                    .count();

                prop_assert_eq!(make010_count, 0,
                    "echo/printf with '{}' in string should not trigger MAKE010", cmd);
            }

            /// Property: Variable assignments with command keywords should never trigger MAKE010
            #[test]
            fn prop_variable_assignment_never_warns(
                cmd in command_keyword(),
                var_name in "[A-Z][A-Z0-9_]{0,10}",
                text in "[a-zA-Z0-9 ]+",
            ) {
                let recipe = format!("\t@{}=\"{} {}\"", var_name, text, cmd);
                let makefile = format!("target:\n{}", recipe);
                let result = check(&makefile);

                // Should NOT trigger MAKE010 for command in variable assignment
                let make010_count = result.diagnostics.iter()
                    .filter(|d| d.code == "MAKE010")
                    .count();

                prop_assert_eq!(make010_count, 0,
                    "Variable assignment with '{}' in value should not trigger MAKE010", cmd);
            }

            /// Property: Actual commands without error handling should always trigger MAKE010
            #[test]
            fn prop_actual_command_always_warns(
                cmd in command_keyword(),
                args in "[a-zA-Z0-9/._-]+",
            ) {
                let recipe = format!("\t{} {}", cmd, args);
                let makefile = format!("target:\n{}", recipe);
                let result = check(&makefile);

                // SHOULD trigger MAKE010 for actual command
                let make010_count = result.diagnostics.iter()
                    .filter(|d| d.code == "MAKE010")
                    .count();

                prop_assert_eq!(make010_count, 1,
                    "Actual '{}' command without error handling should trigger MAKE010", cmd);
            }

            /// Property: Commands with || exit 1 should never trigger MAKE010
            #[test]
            fn prop_command_with_error_handling_never_warns(
                cmd in command_keyword(),
                args in "[a-zA-Z0-9/._-]+",
            ) {
                let recipe = format!("\t{} {} || exit 1", cmd, args);
                let makefile = format!("target:\n{}", recipe);
                let result = check(&makefile);

                // Should NOT trigger MAKE010 when error handling present
                let make010_count = result.diagnostics.iter()
                    .filter(|d| d.code == "MAKE010")
                    .count();

                prop_assert_eq!(make010_count, 0,
                    "Command '{}' with || exit 1 should not trigger MAKE010", cmd);
            }

            /// Property: is_variable_assignment is deterministic
            #[test]
            fn prop_is_variable_assignment_deterministic(line in ".*") {
                let result1 = is_variable_assignment(&line);
                let result2 = is_variable_assignment(&line);
                prop_assert_eq!(result1, result2,
                    "is_variable_assignment should be deterministic");
            }

            /// Property: is_variable_assignment only accepts quoted values
            #[test]
            fn prop_is_variable_assignment_requires_quotes(
                var_name in "[A-Z][A-Z0-9_]{0,10}",
                value in "[a-zA-Z0-9 ]+",
            ) {
                let unquoted = format!("{}={}", var_name, value);
                let double_quoted = format!("{}=\"{}\"", var_name, value);
                let single_quoted = format!("{}='{}'", var_name, value);

                prop_assert!(!is_variable_assignment(&unquoted),
                    "Unquoted assignment should return false");
                prop_assert!(is_variable_assignment(&double_quoted),
                    "Double-quoted assignment should return true");
                prop_assert!(is_variable_assignment(&single_quoted),
                    "Single-quoted assignment should return true");
            }
        }
    }
}
