//! PERF002: Command substitution inside loop body
//!
//! **Rule**: Detect `$(...)` inside for/while loop bodies
//!
//! **Why this matters**:
//! Command substitution in a loop forks a subshell on every iteration,
//! causing significant performance degradation for large iteration counts.
//! Moving the substitution outside the loop can dramatically improve performance.
//!
//! **Auto-fix**: None (manual refactoring required)
//!
//! ## Examples
//!
//! Bad (forks subshell each iteration):
//! ```bash
//! for i in $(seq 1 100); do
//!     owner=$(stat -c '%U' "$i")
//!     echo "$owner"
//! done
//! ```
//!
//! Good (compute once outside loop):
//! ```bash
//! owners=$(stat -c '%U' *)
//! for i in $(seq 1 100); do
//!     echo "$i"
//! done
//! ```

use crate::linter::shell_words;
use crate::linter::{Diagnostic, LintResult, Severity, Span};

fn is_loop_header(trimmed: &str) -> bool {
    trimmed.starts_with("for ") || trimmed.starts_with("while ") || trimmed.starts_with("until ")
}

fn is_do_line(trimmed: &str) -> bool {
    trimmed == "do" || trimmed.ends_with("; do") || trimmed.ends_with(";do")
}

fn is_loop_exit(trimmed: &str) -> bool {
    trimmed == "done" || trimmed.starts_with("done ") || trimmed.starts_with("done;")
}

/// 1-indexed byte column of the first real `$( … )` / `` ` … ` `` command
/// substitution marker on `line`, or `None` when there isn't one.
///
/// ## Lexer-context history (PMAT-257, GH-313)
///
/// The original implementation matched the raw substring `"$("` anywhere on
/// the line. `$((...))` arithmetic expansion *starts with* `$(`, so `line.find
/// ("$(")` also matched it — but arithmetic expansion forks no subshell; the
/// shell evaluates it in-process. This now goes through
/// [`crate::linter::shell_words`], the same word/role analysis SC2046,
/// SEC002 and IDEM002 use: `ShellWord::substitutions` never contains an entry
/// for `$((...))` (the lexer special-cases and skips the whole balanced span
/// without recording it), so only a genuine command substitution is found.
/// The lexer also stops at an unquoted `#`, so a substitution written inside
/// a trailing comment is never picked up either — the old manual
/// quote-parity scan for that case is no longer needed.
fn first_command_substitution_col(line: &str) -> Option<usize> {
    shell_words::simple_commands(line)
        .into_iter()
        .flat_map(|c| c.words.into_iter().flat_map(|w| w.substitutions))
        .map(|e| e.col)
        .min()
}

/// Look for a real `$(...)` / `` `...` `` command substitution on `line`
/// (never `$((...))` arithmetic) and, if found, add a PERF002 diagnostic to
/// `result`.
fn check_line_for_subst(result: &mut LintResult, line_num: usize, line: &str) {
    let Some(col) = first_command_substitution_col(line) else {
        return;
    };

    let span = Span::new(line_num + 1, col, line_num + 1, col + 2);
    let diagnostic = Diagnostic::new(
        "PERF002",
        Severity::Warning,
        "Command substitution inside loop body forks a subshell each iteration. Consider moving outside the loop.",
        span,
    );
    result.add(diagnostic);
}

/// Update loop-tracking state for one line. Returns the (possibly updated)
/// `(in_loop_body, loop_depth)` pair.
fn track_loop_state(trimmed: &str, in_loop_body: bool, loop_depth: i32) -> (bool, i32) {
    let mut in_loop_body = in_loop_body || is_loop_header(trimmed);
    let mut loop_depth = loop_depth + i32::from(is_loop_header(trimmed));
    if is_loop_exit(trimmed) {
        loop_depth = (loop_depth - 1).max(0);
        in_loop_body = loop_depth > 0;
    }
    (in_loop_body, loop_depth)
}

/// Check for command substitution inside loop bodies
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let mut in_loop_body = false;
    let mut loop_depth: i32 = 0;

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        (in_loop_body, loop_depth) = track_loop_state(trimmed, in_loop_body, loop_depth);

        if !(in_loop_body && loop_depth > 0) {
            continue;
        }
        // Skip the loop control line itself (for ... in $(cmd) is fine).
        if is_loop_header(trimmed) || is_do_line(trimmed) {
            continue;
        }

        check_line_for_subst(&mut result, line_num, line);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf002_detects_subst_in_for_loop() {
        let script = "for i in 1 2 3; do\n    val=$(echo hello)\ndone";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "PERF002");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_perf002_detects_subst_in_while_loop() {
        let script = "while true; do\n    val=$(date)\ndone";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_perf002_no_flag_outside_loop() {
        let script = "val=$(echo hello)\necho $val";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_perf002_no_flag_loop_control() {
        let script = "for i in $(seq 1 10); do\n    echo $i\ndone";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_perf002_no_fix_provided() {
        let script = "for i in 1 2 3; do\n    val=$(echo hello)\ndone";
        let result = check(script);
        assert!(result.diagnostics[0].fix.is_none());
    }

    #[test]
    fn test_perf002_skip_comments() {
        let script = "for i in 1 2 3; do\n    # val=$(echo hello)\ndone";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    // GH-313: `$((...))` arithmetic expansion is not a command substitution -
    // the shell evaluates it in-process, forking no subshell.
    #[test]
    fn test_PMAT257_gh313_perf002_arithmetic_expansion_is_not_a_subshell() {
        let script = "for i in 1 2 3; do\n    j=$(( i + 1 ))\n    echo $j\ndone";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);

        // A real command substitution in a loop body must still be reported.
        let real = "for i in 1 2 3; do\n    d=$(date)\ndone";
        let result = check(real);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
