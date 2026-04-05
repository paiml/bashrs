
    #[test]
    fn test_iter5_check_line111_or_not_and() {
        // MUTATION: Line 111:50 - replace || with && in check function
        // Tests: is_arithmetic = contains("$((") || contains("(( ")
        let bash_code = "(( i++ ))"; // Has "(( " but not "$(("
        let result = check(bash_code);
        // Should NOT flag (arithmetic context)
        // If || becomes &&, would require BOTH patterns, incorrectly flagging this
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Arithmetic with (( should not flag"
        );
    }

    #[test]
    fn test_iter5_is_in_arithmetic_not_always_false() {
        // MUTATION: Line 56:5 - replace is_in_arithmetic_context -> bool with false
        // Tests: Function must return true for variables in $(( ))
        let bash_code = "x=$(( $a + $b ))";
        let result = check(bash_code);
        // Variables in $(( )) should NOT be flagged
        // If function always returns false, would incorrectly flag these
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Variables in $(( )) arithmetic should not be flagged"
        );
    }

    #[test]
    fn test_iter5_less_than_boundary_equal() {
        // MUTATION: Line 25:27 - replace < with == in should_skip_line
        // Tests boundary: eq_pos < first_space (not ==)
        let bash_code = "Y=123\necho $Y";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].span.start_line, 2);
    }

    #[test]
    fn test_iter5_less_than_boundary_greater() {
        // MUTATION: Line 25:27 - replace < with > in should_skip_line
        // Tests: eq_pos < first_space (not >)
        let bash_code = "Z= value\necho $Z";
        let result = check(bash_code);
        // Should skip assignment and only flag echo
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].span.start_line, 2);
    }

    // ===== Property-Based Tests - Arithmetic Invariants (Iteration 4) =====
    // These property tests catch arithmetic mutations (+ → *, + → -, < → >, etc.)
    // that unit tests miss. Validates mathematical invariants that MUST hold.
    //
    // Based on user feedback: "why not property?" - property tests verify
    // invariants, not just specific outputs. Arithmetic mutations violate these.

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(10))]
            #[test]
            fn prop_column_positions_always_valid(
                var_name in "[a-z]{1,10}",
                leading_spaces in 0usize..20
            ) {
                // PROPERTY: Column positions must always be >= 1 (1-indexed)
                // Catches: + → * mutations (would produce 0), + → - mutations
                let spaces = " ".repeat(leading_spaces);
                let bash_code = format!("{}echo ${}", spaces, var_name);
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
                bash_code.push_str(&format!("echo ${}", var_name));

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
                let bash_code = format!("echo ${}", var_name);
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
            fn prop_braced_variable_span_includes_braces(
                var_name in "[a-z]{1,10}"
            ) {
                // PROPERTY: ${VAR} span must cover entire expression including braces
                // Catches: arithmetic mutations in calculate_end_column
                let bash_code = format!("echo ${{{}}}", var_name);
                let result = check(&bash_code);

                if !result.diagnostics.is_empty() {
                    let span = &result.diagnostics[0].span;
                    // INVARIANT: Span for ${VAR} must be at least length of ${VAR}
                    let expected_min_length = var_name.len() + 3; // ${}
                    let span_length = span.end_col.saturating_sub(span.start_col);
                    prop_assert!(span_length >= expected_min_length,
                        "Span length {} must be >= {} for ${{{}}}", span_length, expected_min_length, var_name);
                }
            }

            #[test]
            fn prop_skip_assignments_correctly(
                var_name in "[a-z]{1,10}",
                value in "[a-z0-9]{1,10}"
            ) {
                // PROPERTY: Variable assignments should be skipped correctly
                // Catches: < → >, < → ==, < → <= mutations in should_skip_line
                let bash_code = format!("{}={}\necho ${}", var_name, value, var_name);
                let result = check(&bash_code);

                // INVARIANT: Should only detect $VAR in echo, not in assignment
                // Assignment is line 1, echo is line 2
                if !result.diagnostics.is_empty() {
                    prop_assert_eq!(result.diagnostics.len(), 1, "Should only flag echo line");
                    prop_assert_eq!(result.diagnostics[0].span.start_line, 2,
                        "Should flag line 2 (echo), not line 1 (assignment)");
                }
            }

            #[test]
            fn prop_arithmetic_context_never_flagged(
                x_val in 0i32..100,
                y_val in 0i32..100
            ) {
                // PROPERTY: Variables in $(( )) should never be flagged
                // Catches: return value mutations in is_in_arithmetic_context
                let bash_code = format!("result=$(( {} + {} ))", x_val, y_val);
                let result = check(&bash_code);

                // INVARIANT: Arithmetic context should never produce diagnostics
                prop_assert_eq!(result.diagnostics.len(), 0,
                    "Variables in $(( )) should not be flagged");
            }

            #[test]
            fn prop_quoted_variables_never_flagged(
                var_name in "[a-z]{1,10}"
            ) {
                // PROPERTY: Already-quoted variables should never be flagged
                // Catches: && → || mutations in is_already_quoted
                let bash_code = format!("echo \"${}\"", var_name);
                let result = check(&bash_code);

                // INVARIANT: Quoted variables should not produce diagnostics
                prop_assert_eq!(result.diagnostics.len(), 0,
                    "Already-quoted variables should not be flagged");
            }

            #[test]
            fn prop_braced_variables_in_quotes_never_flagged(
                var1 in "[a-z]{1,10}",
                var2 in "[a-z]{1,10}",
                text in "[a-z ]{0,20}"
            ) {
                // PROPERTY: Variables inside quoted strings should never be flagged
                // Issue #1: Fixes auto-fix creating invalid syntax
                // Catches: quote-counting logic errors in is_already_quoted
                let bash_code = format!("echo \"${{{}}}{}${{{}}}\"", var1, text, var2);
                let result = check(&bash_code);

                // INVARIANT: Variables inside quoted strings should not produce diagnostics
                prop_assert_eq!(result.diagnostics.len(), 0,
                    "Variables inside quoted strings should not be flagged. Code: '{}'", bash_code);
            }
        }
    }

    // ===== Issue #105: Safe [[ ]] context tests =====
    // Variables in [[ ]] are safe - no word splitting or glob expansion

    #[test]
    fn test_FP_105_double_bracket_n_test_not_flagged() {
        let code = r#"[[ -n $var ]]"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2086 must NOT flag unquoted variable in [[ -n $var ]]"
        );
    }

    #[test]
    fn test_FP_105_double_bracket_z_test_not_flagged() {
        let code = r#"[[ -z $var ]]"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2086 must NOT flag unquoted variable in [[ -z $var ]]"
        );
    }

    #[test]
    fn test_FP_105_double_bracket_equality_not_flagged() {
        let code = r#"[[ $var = "value" ]]"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2086 must NOT flag unquoted variable in [[ $var = ... ]]"
        );
    }

    #[test]
    fn test_FP_105_double_bracket_comparison_not_flagged() {
        let code = r#"[[ $x -eq $y ]]"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2086 must NOT flag unquoted variables in [[ $x -eq $y ]]"
        );
    }

    #[test]
    fn test_FP_105_double_bracket_regex_not_flagged() {
        let code = r#"[[ $var =~ ^[0-9]+$ ]]"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2086 must NOT flag unquoted variable in regex match"
        );
    }

    #[test]
    fn test_FP_105_single_bracket_still_flagged() {
        // Single brackets DO need quoting
        let code = r#"[ -n $var ]"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "SC2086 SHOULD flag unquoted variable in single bracket [ ]"
        );
    }

    // ===== Issue #107: C-style for loop arithmetic context =====
    // Variables inside (( )) are in arithmetic context and don't need quoting

    #[test]
    fn test_FP_107_cstyle_for_loop_not_flagged() {
        // C-style for loop is arithmetic context
        let code = r#"for ((i=0; i<$n; i++)); do echo "loop"; done"#;
        let result = check(code);
        // Should not flag $n inside (( ))
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2086 must NOT flag variables inside C-style for (( )) loop"
        );
    }

    #[test]
    fn test_FP_107_double_paren_arithmetic_not_flagged() {
        // Standalone (( )) is arithmetic context
        let code = r#"(( count = $x + $y ))"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2086 must NOT flag variables inside standalone (( )) arithmetic"
        );
    }

    #[test]
    fn test_FP_107_while_arithmetic_not_flagged() {
        // while (( )) is arithmetic context
        let code = r#"while (( $i < $max )); do echo $i; done"#;
        let result = check(code);
        // Should flag echo $i but NOT the ones inside (( ))
        assert_eq!(
            result.diagnostics.len(),
            1,
            "SC2086 should only flag echo $i, not variables in (( ))"
        );
    }

    #[test]
    fn test_FP_107_if_arithmetic_not_flagged() {
        // if (( )) is arithmetic context
        let code = r#"if (( $x > 0 )); then echo yes; fi"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2086 must NOT flag variables inside if (( )) condition"
        );
    }

    #[test]
    fn test_FP_107_arithmetic_increment_not_flagged() {
        // (( i++ )) and (( i+=1 )) are arithmetic
        let code = r#"(( $count++ ))"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2086 must NOT flag variable in arithmetic increment"
        );
    }

    // ===== F048: C-style for loop variable in body =====
    // Loop variables from C-style for loops are always numeric
    // SC2086 should not flag them even in the loop body

    #[test]
    fn test_FP_048_cstyle_for_loop_var_in_body() {
        // F048: C-style for loop variable used in body
        let code = r#"for ((i=0;i<10;i++)); do echo $i; done"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2086 must NOT flag C-style for loop variable $i in loop body"
        );
    }

    #[test]
    fn test_FP_048_cstyle_for_multiple_uses() {
        // Multiple uses of loop variable in body
        let code = r#"for ((n=1;n<=5;n++)); do echo $n; printf "%d\n" $n; done"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2086 must NOT flag any uses of C-style for loop variable"
        );
    }

    #[test]
    fn test_FP_048_non_loop_var_still_flagged() {
        // Other variables should still be flagged
        let code = r#"for ((i=0;i<10;i++)); do echo $other; done"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "SC2086 SHOULD flag non-loop variable $other"
        );
    }
}
