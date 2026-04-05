//! SEM003: Unreachable code after exit/return/exec
//!
//! **Rule**: Detect statements that follow unconditional `exit`, `return`, or `exec`
//! at the same block level, which can never be reached.
//!
//! **Why this matters**:
//! Dead code after exit/return/exec is never executed, indicating:
//! - Forgotten cleanup of debugging code
//! - Logic errors in control flow
//! - Misleading code that suggests behavior that never happens
//!
//! ## Examples
//!
//! ❌ **BAD** (unreachable code):
//! ```bash
//! echo "starting"
//! exit 0
//! echo "this never runs"  # SEM003
//! ```
//!
//! ✅ **GOOD** (exit is last statement):
//! ```bash
//! echo "starting"
//! exit 0
//! ```
//!
//! ✅ **OK** (exit inside conditional — code after if IS reachable):
//! ```bash
//! if [ "$1" = "stop" ]; then
//!     exit 0
//! fi
//! echo "continuing"  # reachable when condition is false
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Commands that unconditionally terminate execution in the current scope.
const EXIT_COMMANDS: &[&str] = &["exit", "return", "exec"];

/// Check for unreachable code after exit/return/exec statements.
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Track nesting depth: we only flag dead code at the same nesting level
    // as the exit statement. Code inside if/while/for/case blocks is at a
    // deeper level and doesn't count.
    let mut depth: i32 = 0;
    let mut exit_at_depth_zero: Option<(usize, &str)> = None; // (line_num, command)

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            // If we've seen an exit and this is just whitespace/comment, skip
            continue;
        }

        // Track block depth changes
        let depth_change = compute_depth_change(trimmed);
        depth += depth_change;

        // Only analyze top-level statements (depth 0)
        if depth != 0 {
            // Reset exit tracking when we enter a block
            exit_at_depth_zero = None;
            continue;
        }

        // Check if this line is an exit command at depth 0
        if is_exit_command(trimmed) {
            if exit_at_depth_zero.is_none() {
                let cmd = extract_exit_command(trimmed);
                exit_at_depth_zero = Some((line_num + 1, cmd));
            }
            continue;
        }

        // If we've seen an exit at depth 0 and this is a non-empty statement, flag it
        if let Some((exit_line, exit_cmd)) = exit_at_depth_zero {
            // This line is unreachable
            let msg = format!("Unreachable code after '{exit_cmd}' on line {exit_line}");
            result.add(Diagnostic::new(
                "SEM003",
                Severity::Warning,
                &msg,
                Span::new(line_num + 1, 1, line_num + 1, trimmed.len()),
            ));
            // Only flag the first unreachable line to avoid noise
            exit_at_depth_zero = None;
        }
    }

    result
}

/// Check if a trimmed line is an unconditional exit/return/exec command.
fn is_exit_command(trimmed: &str) -> bool {
    // Match: exit, exit N, return, return N, exec CMD
    // Don't match: exit_handler(), return_value=, execution_log, etc.
    for cmd in EXIT_COMMANDS {
        if trimmed == *cmd
            || trimmed.starts_with(&format!("{cmd} "))
            || trimmed.starts_with(&format!("{cmd}\t"))
            // Handle exit with semicolons: "exit 0;"
            || trimmed.starts_with(&format!("{cmd};"))
        {
            // Make sure it's not a variable assignment or function name
            // e.g., "exit_code=1" should NOT match
            return true;
        }
    }
    false
}

/// Extract the exit command name from a line.
fn extract_exit_command(trimmed: &str) -> &str {
    for cmd in EXIT_COMMANDS {
        if trimmed.starts_with(cmd) {
            return cmd;
        }
    }
    "exit"
}

/// Compute the net depth change for a line.
///
/// Positive = entering a block, negative = leaving a block.
/// This is approximate but handles common patterns:
/// - `if/then/while/for/case` → +1
/// - `fi/done/esac` → -1
/// - `else/elif` → 0 (same level)
fn compute_depth_change(trimmed: &str) -> i32 {
    let mut change = 0i32;

    // "then" / "do" on own line — already counted via if/while/for
    if trimmed == "then" || trimmed == "do" {
        return 0;
    }

    // Block openers
    let is_opener = trimmed.starts_with("if ") || trimmed == "if"
        || trimmed.starts_with("while ")
        || trimmed.starts_with("until ")
        || trimmed.starts_with("for ")
        || trimmed.starts_with("case ");
    if is_opener {
        change += 1;
    }

    // Block closers
    let closers = ["fi", "done", "esac"];
    for closer in closers {
        if trimmed == closer
            || trimmed.starts_with(&format!("{closer};"))
            || trimmed.starts_with(&format!("{closer} "))
        {
            change -= 1;
        }
    }

    // Handle "if ...; then ... fi" all on one line (net 0)
    if (trimmed.contains("; then") && trimmed.contains("; fi"))
        || (trimmed.contains("; do") && trimmed.contains("; done"))
    {
        return 0;
    }

    change
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pmat213_exit_then_code() {
        let src = "#!/bin/bash\nexit 0\necho unreachable";
        let result = check(src);
        assert!(
            result.diagnostics.iter().any(|d| d.code == "SEM003"),
            "Should detect unreachable code after exit"
        );
    }

    #[test]
    fn test_pmat213_return_then_code() {
        let src = "#!/bin/bash\nreturn 1\necho unreachable";
        let result = check(src);
        assert!(result.diagnostics.iter().any(|d| d.code == "SEM003"));
    }

    #[test]
    fn test_pmat213_exec_then_code() {
        let src = "#!/bin/bash\nexec /bin/sh\necho unreachable";
        let result = check(src);
        assert!(result.diagnostics.iter().any(|d| d.code == "SEM003"));
    }

    #[test]
    fn test_pmat213_exit_last_line_no_warning() {
        let src = "#!/bin/bash\necho hello\nexit 0";
        let result = check(src);
        assert!(
            result.diagnostics.is_empty(),
            "Exit as last statement should not trigger SEM003"
        );
    }

    #[test]
    fn test_pmat213_exit_in_if_not_dead() {
        let src = "#!/bin/bash\nif [ -f /tmp/x ]; then\n  exit 1\nfi\necho reachable";
        let result = check(src);
        assert!(
            !result.diagnostics.iter().any(|d| d.code == "SEM003"),
            "Code after if-block with exit should NOT be flagged as dead"
        );
    }

    #[test]
    fn test_pmat213_no_false_positive_on_exit_variable() {
        let src = "#!/bin/bash\nexit_code=1\necho $exit_code";
        let result = check(src);
        assert!(
            !result.diagnostics.iter().any(|d| d.code == "SEM003"),
            "exit_code= should not trigger SEM003"
        );
    }

    #[test]
    fn test_pmat213_comments_after_exit_not_flagged() {
        let src = "#!/bin/bash\nexit 0\n# This is a comment\n";
        let result = check(src);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_pmat213_is_exit_command() {
        assert!(is_exit_command("exit"));
        assert!(is_exit_command("exit 0"));
        assert!(is_exit_command("exit 1"));
        assert!(is_exit_command("return"));
        assert!(is_exit_command("return 0"));
        assert!(is_exit_command("exec /bin/sh"));
        assert!(!is_exit_command("exit_code=1"));
        assert!(!is_exit_command("return_value"));
        assert!(!is_exit_command("execution_log"));
    }

    #[test]
    fn test_pmat213_depth_tracking() {
        assert_eq!(compute_depth_change("if [ -f x ]; then"), 1);
        assert_eq!(compute_depth_change("fi"), -1);
        assert_eq!(compute_depth_change("while true; do"), 1);
        assert_eq!(compute_depth_change("done"), -1);
        assert_eq!(compute_depth_change("echo hello"), 0);
    }
}
