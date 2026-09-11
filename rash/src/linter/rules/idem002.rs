//! IDEM002: Non-idempotent rm
//!
//! **Rule**: Detect `rm` without `-f` flag
//!
//! **Why this matters**:
//! `rm` without `-f` fails if file doesn't exist, making scripts non-idempotent.
//! Re-running the script will fail instead of succeeding.
//!
//! **Auto-fix**: Add `-f` flag
//!
//! ## Examples
//!
//! ❌ **BAD** (non-idempotent):
//! ```bash
//! rm /app/current
//! ```
//!
//! ✅ **GOOD** (idempotent):
//! ```bash
//! rm -f /app/current
//! ```
//!
//! ## Lexer-context history (PMAT-257, GH-314)
//!
//! The original implementation matched the raw substring `"rm "` anywhere on
//! the line (gated only by the absence of `"rm -"`, to allow `-f`/`-rf`/`-fr`).
//! That reads the *letters* `rm` as a command even when they are text: `rm`
//! inside a quoted sentence (`"the pipe form fails"` contains the substring
//! `"rm "` inside `"form fails"`) or a substring of a longer identifier
//! (`confirm `, `alarm `) was reported as a non-idempotent `rm` invocation.
//! `rm` is only a command when it is in command position — this now goes
//! through [`crate::linter::shell_words`], the same word/role analysis
//! SC2046 and SEC002 use, so quoted text and word substrings are never
//! mistaken for a command.

use crate::linter::shell_words::{self, WordRole};
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// True when `cmd` invokes `rm` with no flag at all — matching the
/// pre-existing "allow -f, -rf, -fr, or any other flag" behaviour: any
/// argument starting with `-` (e.g. `-v`, `-i`) suppresses the warning, not
/// just `-f`, so this rewrite is a pure context fix, not a stricter rule.
fn is_bare_rm(cmd: &shell_words::SimpleCommand) -> bool {
    cmd.name.as_deref() == Some("rm")
        && !cmd
            .words
            .iter()
            .any(|w| w.role == WordRole::Argument && w.literal.starts_with('-'))
}

/// Check for rm without -f flag
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_idx, line) in source.lines().enumerate() {
        let line_num = line_idx + 1;

        for cmd in shell_words::simple_commands(line) {
            if !is_bare_rm(&cmd) {
                continue;
            }
            let Some(name_word) = cmd.words.iter().find(|w| w.role == WordRole::CommandName)
            else {
                continue;
            };
            let col = name_word.col;
            let span = Span::new(line_num, col, line_num, col + 2);

            let fix = Fix::new_with_assumptions(
                "rm -f",
                vec!["Missing file is not an error condition".to_string()],
            );

            let diag = Diagnostic::new(
                "IDEM002",
                Severity::Warning,
                "Non-idempotent rm - add -f flag (SAFE-WITH-ASSUMPTIONS)",
                span,
            )
            .with_fix(fix);

            result.add(diag);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_IDEM002_detects_rm_without_f() {
        let script = "rm /app/current";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "IDEM002");
        assert_eq!(diag.severity, Severity::Warning);
    }

    #[test]
    fn test_IDEM002_no_warning_with_f_flag() {
        let script = "rm -f /app/current";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_IDEM002_no_warning_with_rf() {
        let script = "rm -rf /app/releases";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_IDEM002_provides_fix() {
        let script = "rm /tmp/foo";
        let result = check(script);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, "rm -f");
    }
}
