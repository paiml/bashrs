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

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for command substitution inside loop bodies
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let lines: Vec<&str> = source.lines().collect();
    let mut in_loop_body = false;
    let mut loop_depth: i32 = 0;

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        // Track loop entry
        if trimmed.starts_with("for ")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("until ")
        {
            loop_depth += 1;
            in_loop_body = true;
        }

        // Track 'do' keyword to confirm loop body start
        if trimmed == "do" || trimmed.ends_with("; do") || trimmed.ends_with(";do") {
            // Already set in_loop_body from the for/while line
        }

        // Track loop exit
        if trimmed == "done" || trimmed.starts_with("done ") || trimmed.starts_with("done;") {
            loop_depth -= 1;
            if loop_depth <= 0 {
                loop_depth = 0;
                in_loop_body = false;
            }
        }

        // Check for command substitution inside loop body
        if in_loop_body && loop_depth > 0 {
            // Skip the loop header line itself (for ... in $(cmd) is fine)
            if trimmed.starts_with("for ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("until ")
            {
                continue;
            }
            if trimmed == "do" || trimmed.ends_with("; do") || trimmed.ends_with(";do") {
                continue;
            }

            // Look for $(...) pattern - but not on the loop control line
            if let Some(col) = line.find("$(") {
                // Skip if inside a comment
                let before = &line[..col];
                if before.contains('#') {
                    let hash_pos = before.rfind('#').unwrap_or(0);
                    let pre_hash = &before[..hash_pos];
                    let singles = pre_hash.matches('\'').count();
                    let doubles = pre_hash.matches('"').count();
                    if singles % 2 == 0 && doubles % 2 == 0 {
                        continue;
                    }
                }

                let span = Span::new(line_num + 1, col + 1, line_num + 1, col + 3);

                let diagnostic = Diagnostic::new(
                    "PERF002",
                    Severity::Warning,
                    "Command substitution inside loop body forks a subshell each iteration. Consider moving outside the loop.",
                    span,
                );

                result.add(diagnostic);
            }
        }
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
}
