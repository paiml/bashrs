//! SEC001: Command Injection via eval
//!
//! **Rule**: Detect `eval` usage with user-controlled input
//!
//! **Why this matters**:
//! `eval` with user input is the #1 command injection vector in shell scripts.
//! Attackers can execute arbitrary commands by injecting shell metacharacters.
//!
//! **Auto-fix**: Manual review required (not auto-fixable)
//!
//! ## Examples
//!
//! ❌ **CRITICAL VULNERABILITY**:
//! ```bash
//! eval "rm -rf $USER_INPUT"
//! eval "$CMD"
//! ```
//!
//! ✅ **SAFE ALTERNATIVE**:
//! ```bash
//! # Use array and proper quoting instead of eval
//! # Or use explicit command construction with validation
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check whether `eval` at the given column is a standalone command (word boundaries).
fn is_standalone_eval(line: &str, col: usize) -> bool {
    let before_ok = if col == 0 {
        true
    } else {
        let char_before = line.chars().nth(col - 1);
        matches!(
            char_before,
            Some(' ') | Some('\t') | Some(';') | Some('&') | Some('|') | Some('(')
        )
    };

    let after_idx = col + 4; // "eval" is 4 chars
    let after_ok = if after_idx >= line.len() {
        true
    } else {
        let char_after = line.chars().nth(after_idx);
        matches!(
            char_after,
            Some(' ') | Some('\t') | Some('"') | Some('\'') | Some(';')
        )
    };

    before_ok && after_ok
}

/// Check whether this eval is a safe POSIX variable indirection pattern:
/// `$(eval "printf '%s' ...")` is common for dynamic array access in POSIX sh.
fn is_safe_eval_indirection(line: &str, col: usize) -> bool {
    let before_ctx = if col >= 2 { &line[..col] } else { "" };
    before_ctx.ends_with("$(") && line[col..].contains("printf")
}

/// Check for command injection via eval
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        if let Some(col) = line.find("eval") {
            if !is_standalone_eval(line, col) {
                continue;
            }
            if is_safe_eval_indirection(line, col) {
                continue;
            }

            let span = Span::new(
                line_num + 1,
                col + 1,
                line_num + 1,
                col + 5, // "eval" is 4 chars, +1 for 1-indexed
            );

            let diag = Diagnostic::new(
                "SEC001",
                Severity::Error,
                "Command injection risk via eval - manual review required",
                span,
            );

            result.add(diag);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_SEC001_detects_eval_with_variable() {
        let script = r#"eval "rm -rf $USER_INPUT""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC001");
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("Command injection"));
    }

    #[test]
    fn test_SEC001_detects_eval_simple() {
        let script = "eval \"$CMD\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC001");
    }

    #[test]
    fn test_SEC001_no_false_positive_comment() {
        let script = "# This is evaluation, not eval";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC001_no_false_positive_text() {
        let script = r#"echo "medieval times""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC001_no_auto_fix() {
        let script = "eval \"$USER_CMD\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert!(diag.fix.is_none(), "SEC001 should not provide auto-fix");
    }

    #[test]
    fn test_SEC001_multiple_eval() {
        let script = "eval \"$CMD1\"\neval \"$CMD2\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 2);
    }

    // Mutation Coverage Tests - Following SC2064 pattern (100% kill rate)

    #[test]
    fn test_mutation_sec001_eval_start_col_exact() {
        // MUTATION: Line 60:25 - replace + with * in col + 1
        // Tests start column calculation
        let bash_code = "eval \"$cmd\""; // eval starts at column 0 (0-indexed)
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // With +1: start_col = 1 (1-indexed for display)
        // With *1: start_col = 0
        assert_eq!(
            span.start_col, 1,
            "Start column must use +1, not *1 (would be 0 with *1)"
        );
    }

    #[test]
    fn test_mutation_sec001_eval_end_col_exact() {
        // MUTATION: Line 61:30 - replace + with * in col + 5
        // Tests end column calculation ("eval" = 4 chars, +1 for display)
        let bash_code = "eval \"$cmd\""; // "eval" at columns 0-3
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // "eval" starts at 0, length 4, so col=0, col+5=5
        // With +5: end_col = 5
        // With *5: end_col = 0
        assert_eq!(
            span.end_col, 5,
            "End column must be col + 5, not col * 5 (would be 0 with *5)"
        );
    }

    #[test]
    fn test_mutation_sec001_eval_line_num_calculation() {
        // MUTATIONS: Line 59:30 and 62:25 - replace + with * in line_num + 1
        // Tests line number calculation for multiline input
        let bash_code = "# comment\neval \"$var\""; // eval on line 2 (1-indexed)
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // Line 0 (0-indexed) → line_num + 1 = 1 (display)
        // But eval is on line 1 (0-indexed) → line_num + 1 = 2 (display)
        // With +1: line = 2
        // With *1: line = 1
        assert_eq!(
            span.start_line, 2,
            "Line number must use +1, not *1 (would be 1 with *1)"
        );
        assert_eq!(
            span.end_line, 2,
            "End line number must use +1, not *1 (would be 1 with *1)"
        );
    }

    #[test]
    fn test_mutation_sec001_column_with_offset() {
        // Tests column calculations with leading whitespace
        // Catches mutations in col + 1 and col + 5
        let bash_code = "    eval \"$cmd\""; // eval starts at column 4
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // eval at column 4 (0-indexed) → col + 1 = 5 (1-indexed display)
        assert_eq!(
            span.start_col, 5,
            "Must account for leading whitespace in start column"
        );
        // "eval" ends at column 7 (0-indexed) → col + 5 = 9
        assert_eq!(
            span.end_col, 9,
            "End must be col + 5 to span the 'eval' keyword"
        );
    }

    #[test]
    fn test_mutation_sec001_char_before_calculation() {
        // MUTATIONS: Line 39:56 - replace - with / or + in col - 1
        // Tests the char_before boundary check
        let bash_code = " eval \"$cmd\""; // Space before eval at col 0
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        // Test verifies that col - 1 correctly checks the space character
        // Without this test, mutations like col / 1 or col + 1 would escape
        // The space at position 0 should be correctly identified as valid separator
    }

    #[test]
    fn test_mutation_sec001_char_before_at_boundary() {
        // MUTATIONS: Line 39:56 - additional test for col - 1 edge case
        // Tests char_before when eval is at start (col = 0)
        let bash_code = "eval \"$cmd\""; // No character before eval
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        // With col = 0:
        // col - 1 = -1 (no char before, uses bounds check)
        // col / 1 = 0 (would try to access char at 0, which is 'e')
        // col + 1 = 1 (would try to access char at 1, which is 'v')
        // Test ensures boundary condition is handled correctly
    }
}
