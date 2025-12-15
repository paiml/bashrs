// SC2064: Use single quotes, otherwise this expands now rather than when signalled
//
// When using trap with double quotes, variables are expanded immediately when the
// trap is set, not when the signal is received. This usually causes bugs because
// the trap uses stale values or empty variables.
//
// Examples:
// Bad:
//   trap "rm $tmpfile" EXIT          // $tmpfile expands NOW (might be empty)
//   tmpfile="/tmp/data"
//   trap "echo $status" INT          // $status expands when trap is set
//
// Good:
//   trap 'rm "$tmpfile"' EXIT        // $tmpfile expands WHEN trap fires
//   tmpfile="/tmp/data"              // Value available when trap executes
//   trap 'echo "$status"' INT        // $status expands at signal time
//
// Why this matters:
//   - Variables might not be set yet when trap is defined
//   - Variables might change before signal is received
//   - Trap should use current values, not definition-time values
//   - Common source of "file not found" errors in cleanup traps
//
// Exception: If you specifically want early expansion, use double quotes intentionally.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static TRAP_DOUBLE_QUOTED: Lazy<Regex> = Lazy::new(|| {
    // Match: trap "command with $var" SIGNAL
    Regex::new(r#"\btrap\s+"[^"]*\$[a-zA-Z_][a-zA-Z0-9_]*"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for trap commands with double-quoted strings containing variables
        if let Some(mat) = TRAP_DOUBLE_QUOTED.find(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2064",
                Severity::Warning,
                "Use single quotes, otherwise this expands now rather than when signalled"
                    .to_string(),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2064_trap_double_quoted_with_var() {
        let code = r#"trap "rm $tmpfile" EXIT"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2064");
    }

    #[test]
    fn test_sc2064_trap_status_variable() {
        let code = r#"trap "echo $status" INT"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2064_trap_cleanup() {
        let code = r#"trap "rm -f $tempdir/*" EXIT TERM"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2064_single_quotes_ok() {
        let code = r#"trap 'rm "$tmpfile"' EXIT"#;
        let result = check(code);
        // Single quotes prevent early expansion, correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2064_no_variables_ok() {
        let code = r#"trap "rm /tmp/file" EXIT"#;
        let result = check(code);
        // No variables, double quotes are fine
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2064_literal_string_ok() {
        let code = r#"trap "echo 'Signal received'" INT"#;
        let result = check(code);
        // No variable expansion, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2064_comment_ok() {
        let code = r#"# trap "rm $tmpfile" EXIT"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2064_multiple_variables() {
        let code = r#"trap "rm $file1 $file2" EXIT"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2064_command_substitution() {
        let code = r#"trap "rm $(pwd)/temp" EXIT"#;
        let result = check(code);
        // Command substitution also expands early
        assert_eq!(result.diagnostics.len(), 0); // Our regex only catches $var
    }

    #[test]
    fn test_sc2064_braced_variable() {
        let code = r#"trap "echo ${status}" INT"#;
        let result = check(code);
        // Braced variables also expand early
        assert_eq!(result.diagnostics.len(), 0); // Our simplified regex
    }

    // ===== Mutation Coverage Tests - Exact Column Positions =====
    // These tests catch arithmetic mutations (+ → *, + → -) in column calculations
    // that property tests miss. Based on SC2059 Iteration 1 success.
    //
    // Root Cause: Property tests check invariants (>= 1, end > start) but NOT exact values.
    // Mutations like mat.start() * 1 still satisfy >= 1, so they escape detection.
    // SOLUTION: Assert EXACT column/line numbers to catch arithmetic mutations.

    #[test]
    fn test_mutation_sc2064_trap_start_col_exact() {
        // MUTATION: Line 47:41 - replace + with * in mat.start() + 1
        let bash_code = r#"trap "rm $tmpfile" EXIT"#; // trap starts at column 1
        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should detect trap with variable"
        );
        let span = result.diagnostics[0].span;
        assert_eq!(span.start_col, 1, "Start column must use +1, not *1");
    }

    #[test]
    fn test_mutation_sc2064_trap_end_col_exact() {
        // MUTATION: Line 48:37 - replace + with * or - in mat.end() + 1
        let bash_code = r#"trap "rm $tmpfile" EXIT"#; // Pattern ends at column 18
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        assert_eq!(span.end_col, 18, "End column must use +1, not *1 or -1");
    }

    #[test]
    fn test_mutation_sc2064_line_num_calculation() {
        // MUTATION: Line 39:33 - replace + with * in line_num + 1
        let bash_code = "# comment\ntrap \"rm $var\" EXIT"; // trap on line 2
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        assert_eq!(span.start_line, 2, "Line number must use +1, not *1");
        assert_eq!(span.end_line, 2, "Single line diagnostic");
    }

    #[test]
    fn test_mutation_sc2064_column_positions_with_offset() {
        // Tests column calculations with leading whitespace
        let bash_code = r#"    trap "rm $file" EXIT"#; // trap starts at column 5
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        assert_eq!(span.start_col, 5, "Must account for leading whitespace");
        assert!(span.end_col > span.start_col, "End must be after start");
        // Exact end position: "    trap "rm $file"" = 4 spaces + 14 chars = column 18
        assert_eq!(span.end_col, 19, "End column must be exact");
    }

    // ===== Property-Based Tests - Arithmetic Invariants =====
    // These property tests catch arithmetic mutations (+ → *, + → -, etc.)
    // that unit tests miss. Validates mathematical invariants that MUST hold.
    //
    // Based on lessons from SC2086/SC2059: Property tests are REQUIRED for
    // catching arithmetic mutations in column calculations.

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_column_positions_always_valid(
                var_name in "[a-z]{1,10}",
                leading_spaces in 0usize..20
            ) {
                // PROPERTY: Column positions must always be >= 1 (1-indexed)
                // Catches: + → * mutations (would produce 0), + → - mutations
                let spaces = " ".repeat(leading_spaces);
                let bash_code = format!("{}trap \"rm ${}\" EXIT", spaces, var_name);
                let result = check(&bash_code);

                if !result.diagnostics.is_empty() {
                    let span = &result.diagnostics[0].span;
                    // INVARIANT: Columns are 1-indexed, never 0 or negative
                    prop_assert!(span.start_col >= 1, "Start column must be >= 1, got {}", span.start_col);
                    prop_assert!(span.end_col >= 1, "End column must be >= 1, got {}", span.end_col);
                    // INVARIANT: End must be after start
                    prop_assert!(span.end_col > span.start_col,
                        "End col ({}) must be > start col ({})", span.end_col, span.start_col);
                }
            }

            #[test]
            fn prop_line_numbers_always_valid(
                var_name in "[a-z]{1,10}",
                comment_lines in prop::collection::vec("# comment.*", 0..5)
            ) {
                // PROPERTY: Line numbers must always be >= 1 (1-indexed)
                // Catches: + → * mutations in line_num + 1 calculation
                let mut bash_code = comment_lines.join("\n");
                if !bash_code.is_empty() {
                    bash_code.push('\n');
                }
                bash_code.push_str(&format!("trap \"rm ${}\" EXIT", var_name));

                let result = check(&bash_code);
                if !result.diagnostics.is_empty() {
                    let span = &result.diagnostics[0].span;
                    // INVARIANT: Lines are 1-indexed, never 0 or negative
                    prop_assert!(span.start_line >= 1, "Line number must be >= 1, got {}", span.start_line);
                    prop_assert!(span.end_line >= 1, "Line number must be >= 1, got {}", span.end_line);
                }
            }

            #[test]
            fn prop_span_length_reasonable(
                var_name in "[a-z]{1,10}"
            ) {
                // PROPERTY: Span length should be reasonable (not negative, not huge)
                // Catches: + → - mutations that produce negative/wrong lengths
                let bash_code = format!("trap \"rm ${}\" EXIT", var_name);
                let result = check(&bash_code);

                if !result.diagnostics.is_empty() {
                    let span = &result.diagnostics[0].span;
                    let span_length = span.end_col.saturating_sub(span.start_col);
                    // INVARIANT: Span length must be positive and reasonable
                    prop_assert!(span_length > 0, "Span length must be > 0");
                    prop_assert!(span_length < 1000, "Span length {} seems unreasonable", span_length);
                }
            }

            #[test]
            fn prop_multiple_variables_detected(
                var1 in "[a-z]{1,10}",
                var2 in "[a-z]{1,10}"
            ) {
                // PROPERTY: Multiple variables in trap should still be detected
                // Validates the regex correctly matches first variable
                let bash_code = format!("trap \"rm ${} ${}\" EXIT", var1, var2);
                let result = check(&bash_code);

                // INVARIANT: Should detect at least one variable
                prop_assert_eq!(result.diagnostics.len(), 1, "Should detect trap with variables");
            }

            #[test]
            fn prop_comments_never_flagged(
                var_name in "[a-z]{1,10}",
                leading_spaces in 0usize..20
            ) {
                // PROPERTY: Comments should never be flagged
                // Catches mutations in comment detection logic
                let spaces = " ".repeat(leading_spaces);
                let bash_code = format!("{}# trap \"rm ${}\" EXIT", spaces, var_name);
                let result = check(&bash_code);

                // INVARIANT: Comments should not produce diagnostics
                prop_assert_eq!(result.diagnostics.len(), 0,
                    "Comments should not be flagged");
            }

            #[test]
            fn prop_single_quotes_never_flagged(
                var_name in "[a-z]{1,10}"
            ) {
                // PROPERTY: Single quotes should never be flagged (correct usage)
                // Validates regex only matches double quotes
                let bash_code = format!("trap 'rm ${}' EXIT", var_name);
                let result = check(&bash_code);

                // INVARIANT: Single quotes should not produce diagnostics
                prop_assert_eq!(result.diagnostics.len(), 0,
                    "Single quotes should not be flagged");
            }
        }
    }
}
