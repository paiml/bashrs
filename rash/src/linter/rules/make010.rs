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
fn find_critical_command(recipe: &str) -> Option<&'static str> {
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
}
