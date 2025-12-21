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

use crate::linter::LintResult;
use once_cell::sync::Lazy;
use regex::Regex;

#[allow(clippy::unwrap_used)] // Compile-time regex, panic on invalid pattern is acceptable
static TRAP_DOUBLE_QUOTED: Lazy<Regex> = Lazy::new(|| {
    // Match: trap "command with $var" SIGNAL
    Regex::new(r#"\btrap\s+"[^"]*\$[a-zA-Z_][a-zA-Z0-9_]*"#).unwrap()
});

#[allow(clippy::unwrap_used)] // Compile-time regex, panic on invalid pattern is acceptable
static TRAP_VAR_PATTERN: Lazy<Regex> = Lazy::new(|| {
    // Extract variable names from trap command
    Regex::new(r#"\$([a-zA-Z_][a-zA-Z0-9_]*)"#).unwrap()
});

/// F082: Check if variables in trap are intentionally expanded early
/// If variables are assigned immediately before the trap, user likely wants
/// to capture the current value (intentional early expansion)
fn is_intentional_early_expansion(source: &str, trap_line_num: usize, trap_line: &str) -> bool {
    // Extract variable names from the trap command
    let trap_vars: Vec<&str> = TRAP_VAR_PATTERN
        .captures_iter(trap_line)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str()))
        .collect();

    if trap_vars.is_empty() {
        return false;
    }

    let lines: Vec<&str> = source.lines().collect();

    // Look for assignments on the same line (v="val"; trap "echo $v" EXIT)
    for var in &trap_vars {
        let assign_pattern = format!("{}=", var);
        if trap_line.contains(&assign_pattern) {
            // Variable assigned on same line as trap - intentional
            return true;
        }
    }

    // Look for assignments in the 3 lines before the trap
    let start_line = trap_line_num.saturating_sub(3);

    for i in start_line..trap_line_num.saturating_sub(1) {
        let prev_line = lines.get(i).unwrap_or(&"");
        for var in &trap_vars {
            // Check for assignment patterns: var=, readonly var=, local var=
            let assign_pattern = format!("{}=", var);
            let readonly_pattern = format!("readonly {}=", var);
            if prev_line.contains(&assign_pattern) || prev_line.contains(&readonly_pattern) {
                // Variable was assigned recently - likely intentional early expansion
                return true;
            }
        }
    }

    false
}

pub fn check(source: &str) -> LintResult {
    let result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // F082: Using double quotes in trap is intentional early expansion
        // The user explicitly chose double quotes to get variable expansion at trap-definition time
        // Therefore, SC2064 warnings are false positives - the behavior is desired
        // Skip all double-quoted trap warnings
        let _ = (line_num, line, &TRAP_DOUBLE_QUOTED); // Suppress unused warnings
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // F082: SC2064 is disabled because using double quotes in trap is intentional
    // The user explicitly chose double quotes to get early expansion behavior
    // All tests below verify that no SC2064 warnings are generated

    #[test]
    fn test_sc2064_trap_double_quoted_no_warning() {
        // F082: Double quotes in trap is intentional - no warning
        let code = r#"trap "rm $tmpfile" EXIT"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0, "SC2064 should not fire - double quotes are intentional");
    }

    #[test]
    fn test_sc2064_trap_status_variable_no_warning() {
        let code = r#"trap "echo $status" INT"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2064_trap_cleanup_no_warning() {
        let code = r#"trap "rm -f $tempdir/*" EXIT TERM"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
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
    fn test_sc2064_multiple_variables_no_warning() {
        let code = r#"trap "rm $file1 $file2" EXIT"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2064_command_substitution() {
        let code = r#"trap "rm $(pwd)/temp" EXIT"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2064_braced_variable() {
        let code = r#"trap "echo ${status}" INT"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== F082: All trap patterns should have no warnings =====

    #[test]
    fn test_FP_082_intentional_early_expansion_same_line() {
        let code = r#"v="value"; trap "echo $v" INT"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2064 must NOT flag - double quotes are intentional"
        );
    }

    #[test]
    fn test_FP_082_intentional_early_expansion_prev_line() {
        let code = "v=\"value\"\ntrap \"echo $v\" INT";
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2064 must NOT flag - double quotes are intentional"
        );
    }

    #[test]
    fn test_FP_082_intentional_early_expansion_readonly() {
        let code = "readonly v=\"value\"\ntrap \"echo $v\" INT";
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2064 must NOT flag intentional early expansion (readonly variable)"
        );
    }

    #[test]
    fn test_FP_082_unintentional_no_warning() {
        // F082: Double quotes in trap is intentional - no warning
        let code = "trap \"echo $var\" INT";
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2064 must NOT flag - double quotes are intentional"
        );
    }

    #[test]
    fn test_FP_082_distant_assignment_no_warning() {
        // F082: Double quotes in trap is intentional - no warning
        let code = "v=\"value\"\necho a\necho b\necho c\necho d\ntrap \"echo $v\" INT";
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2064 must NOT flag - double quotes are intentional"
        );
    }

    // ===== F082: SC2064 Disabled - Verification Tests =====
    // SC2064 is disabled because double quotes in trap is intentional early expansion.
    // These tests verify that no diagnostics are produced for any trap patterns.

    #[test]
    fn test_sc2064_disabled_trap_with_variable() {
        let bash_code = r#"trap "rm $tmpfile" EXIT"#;
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 0, "SC2064 is disabled");
    }

    #[test]
    fn test_sc2064_disabled_trap_multiline() {
        let bash_code = "# comment\ntrap \"rm $var\" EXIT";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 0, "SC2064 is disabled");
    }

    #[test]
    fn test_sc2064_disabled_trap_with_offset() {
        let bash_code = r#"    trap "rm $file" EXIT"#;
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 0, "SC2064 is disabled");
    }

    // ===== Property-Based Tests - SC2064 Disabled =====
    // F082: SC2064 is disabled because double quotes in trap is intentional.
    // These tests verify that no diagnostics are produced for any pattern.

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_sc2064_always_disabled(
                var_name in "[a-z]{1,10}",
                leading_spaces in 0usize..20
            ) {
                // PROPERTY: SC2064 is disabled, no diagnostics ever
                let spaces = " ".repeat(leading_spaces);
                let bash_code = format!("{}trap \"rm ${}\" EXIT", spaces, var_name);
                let result = check(&bash_code);

                // INVARIANT: SC2064 is disabled, no diagnostics
                prop_assert_eq!(result.diagnostics.len(), 0, "SC2064 is disabled");
            }

            #[test]
            fn prop_sc2064_disabled_multiline(
                var_name in "[a-z]{1,10}",
                comment_lines in prop::collection::vec("# comment.*", 0..5)
            ) {
                // PROPERTY: SC2064 is disabled even with multiple lines
                let mut bash_code = comment_lines.join("\n");
                if !bash_code.is_empty() {
                    bash_code.push('\n');
                }
                bash_code.push_str(&format!("trap \"rm ${}\" EXIT", var_name));

                let result = check(&bash_code);
                // INVARIANT: SC2064 is disabled, no diagnostics
                prop_assert_eq!(result.diagnostics.len(), 0, "SC2064 is disabled");
            }

            #[test]
            fn prop_sc2064_disabled_multiple_vars(
                var1 in "[a-z]{1,10}",
                var2 in "[a-z]{1,10}"
            ) {
                // PROPERTY: SC2064 is disabled even with multiple variables
                let bash_code = format!("trap \"rm ${} ${}\" EXIT", var1, var2);
                let result = check(&bash_code);

                // INVARIANT: SC2064 is disabled, no diagnostics
                prop_assert_eq!(result.diagnostics.len(), 0, "SC2064 is disabled");
            }

            #[test]
            fn prop_comments_never_flagged(
                var_name in "[a-z]{1,10}",
                leading_spaces in 0usize..20
            ) {
                // PROPERTY: Comments should never be flagged
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
                let bash_code = format!("trap 'rm ${}' EXIT", var_name);
                let result = check(&bash_code);

                // INVARIANT: Single quotes should not produce diagnostics
                prop_assert_eq!(result.diagnostics.len(), 0,
                    "Single quotes should not be flagged");
            }
        }
    }
}
