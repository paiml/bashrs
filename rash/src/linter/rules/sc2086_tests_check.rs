// SC2086: Double quote to prevent globbing and word splitting - THIN SHIM
// All logic extracted to sc2086_logic.rs

use super::sc2086_logic::*;
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for unquoted variable expansions (SC2086)
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let pattern = get_var_pattern();
    let cstyle_vars = get_cstyle_for_loop_vars(source);
    for (line_num, line) in source.lines().enumerate() {
        for uv in find_unquoted_vars(line, &pattern, &cstyle_vars) {
            let span = Span::new(line_num + 1, uv.col, line_num + 1, uv.end_col);
            let var_text = format_var_text(&uv.var_name, uv.is_braced);
            result.add(
                Diagnostic::new(
                    "SC2086",
                    Severity::Warning,
                    format!(
                        "Double quote to prevent globbing and word splitting on {}",
                        var_text
                    ),
                    span,
                )
                .with_fix(Fix::new(format_quoted_var(&uv.var_name, uv.is_braced))),
            );
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2086_basic_detection() {
        let bash_code = r#"
#!/bin/bash
FILES=$1
ls $FILES
"#;

        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should detect one unquoted variable"
        );
        assert_eq!(result.diagnostics[0].code, "SC2086");
        assert!(result.diagnostics[0].message.contains("Double quote"));
        assert!(result.diagnostics[0].message.contains("$FILES"));
    }

    #[test]
    fn test_sc2086_autofix() {
        let bash_code = "ls $FILES";
        let result = check(bash_code);

        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "\"$FILES\""
        );
    }

    #[test]
    fn test_sc2086_no_false_positive_arithmetic() {
        let bash_code = "result=$(( $x + $y ))";
        let result = check(bash_code);

        // Should NOT trigger SC2086 in arithmetic context
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Should not trigger in arithmetic context"
        );
    }

    #[test]
    fn test_sc2086_multiple_violations() {
        let bash_code = r#"
rm -rf $DIR
cat $FILE1 $FILE2
"#;

        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            3,
            "Should detect three unquoted variables"
        );

        let codes: Vec<&str> = result.diagnostics.iter().map(|d| d.code.as_str()).collect();
        assert_eq!(codes, vec!["SC2086", "SC2086", "SC2086"]);
    }

    #[test]
    fn test_sc2086_braced_variables() {
        let bash_code = r#"echo ${VAR}"#;
        let result = check(bash_code);

        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("${VAR}"));
    }

    #[test]
    fn test_sc2086_skip_comments() {
        let bash_code = r#"
# This is a comment with $VAR
echo $ACTUAL
"#;

        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should only detect variable in echo, not comment"
        );
        assert!(result.diagnostics[0].message.contains("$ACTUAL"));
    }

    #[test]
    fn test_sc2086_skip_quoted() {
        let bash_code = r#"echo "$VAR""#;
        let result = check(bash_code);

        // Should NOT trigger - already quoted
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Should not trigger on already-quoted variables"
        );
    }

    #[test]
    fn test_sc2086_skip_braced_in_quoted_string() {
        // Issue #1: Variables inside quoted strings should not be flagged
        let bash_code = r#"echo "${VAR1}text${VAR2}""#;
        let result = check(bash_code);

        // Should NOT trigger - variables are inside quoted string
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Should not trigger on variables inside quoted strings. Found: {:?}",
            result
                .diagnostics
                .iter()
                .map(|d| &d.message)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_sc2086_skip_color_codes_in_quotes() {
        // Issue #1: Real-world case with color codes
        let bash_code = r#"echo -e "${BLUE}text${NC}""#;
        let result = check(bash_code);

        // Should NOT trigger - variables are inside quoted string
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Should not trigger on color codes in quoted strings. Found: {:?}",
            result
                .diagnostics
                .iter()
                .map(|d| &d.message)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_sc2086_mixed_quoted_unquoted() {
        let bash_code = r#"
echo "$VAR1"
echo $VAR2
echo "$VAR3"
"#;

        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should only detect unquoted $VAR2"
        );
        assert!(result.diagnostics[0].message.contains("$VAR2"));
    }

    #[test]
    fn test_sc2086_severity() {
        let bash_code = "ls $FILES";
        let result = check(bash_code);

        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sc2086_span_accuracy() {
        let bash_code = "ls $FILES";
        let result = check(bash_code);

        let span = result.diagnostics[0].span;
        assert_eq!(span.start_line, 1);
        assert_eq!(span.end_line, 1);
        // Column positions should point to $FILES
        assert!(span.start_col <= 4); // "ls " is 3 chars
        assert!(span.end_col >= span.start_col);
    }

    // ===== Mutation Coverage Tests =====
    // These tests specifically target mutations identified by cargo-mutants

    #[test]
    fn test_mutation_arithmetic_false_positive() {
        // MUTATION: If is_in_arithmetic_context always returns true,
        // this test should fail (we'd skip detection incorrectly)
        let bash_code = "echo $VAR"; // Not in arithmetic
        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should detect unquoted var outside arithmetic"
        );
        assert_eq!(result.diagnostics[0].code, "SC2086");
    }

    #[test]
    fn test_mutation_arithmetic_false_negative() {
        // MUTATION: If is_in_arithmetic_context always returns false,
        // this test should fail (we'd incorrectly flag safe arithmetic)
        let bash_code = "result=$(( $x + $y ))";
        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Should NOT flag variables in arithmetic"
        );
    }

    #[test]
    fn test_mutation_arithmetic_both_conditions() {
        // MUTATION: If && becomes || in is_in_arithmetic_context (line 58),
        // this should fail. Verifies BOTH $(( and )) are required
        let bash_code1 = "echo $(( $VAR"; // Missing closing ))
        let result1 = check(bash_code1);
        assert!(
            !result1.diagnostics.is_empty(),
            "Should flag incomplete arithmetic (missing closing)"
        );

        let bash_code2 = "echo $VAR ))"; // Missing opening $((
        let result2 = check(bash_code2);
        assert!(
            !result2.diagnostics.is_empty(),
            "Should flag incomplete arithmetic (missing opening)"
        );
    }

    #[test]
    fn test_mutation_column_calculation_braced() {
        // MUTATION: If + becomes * or - in calculate_end_column (lines 45, 50),
        // column positions will be wrong
        let bash_code = "echo ${VAR}";
        let result = check(bash_code);

        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;

        // Verify exact column positions
        assert_eq!(span.start_col, 6, "Start should be at $ (column 6)");
        assert_eq!(
            span.end_col, 12,
            "End should include closing }} (column 12)"
        );
    }

    #[test]
    fn test_mutation_column_calculation_simple() {
        // MUTATION: Verifies column calculation for simple $VAR (no braces)
        let bash_code = "echo $VAR";
        let result = check(bash_code);

        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;

        assert_eq!(span.start_col, 6, "Start should be at $ (column 6)");
        assert_eq!(span.end_col, 10, "End should be after VAR (column 10)");
    }

    #[test]
    fn test_mutation_line_numbers() {
        // MUTATION: If + becomes - in check function (line 121),
        // line numbers will be incorrect
        let bash_code = r#"
#!/bin/bash
echo "first"
echo $VAR
echo "last"
"#;

        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should detect one unquoted variable"
        );
        assert_eq!(
            result.diagnostics[0].span.start_line, 4,
            "Should report line 4"
        );
        assert_eq!(
            result.diagnostics[0].span.end_line, 4,
            "End line should also be 4"
        );
    }

    #[test]
    fn test_mutation_arithmetic_check_logic() {
        // MUTATION: If && becomes || in check function (line 127),
        // verify arithmetic detection still works correctly
        let bash_code = "result=$(( $x + $y ))";
        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Arithmetic context check must work"
        );

        // Also verify non-arithmetic is still caught
        let bash_code2 = "echo $x";
        let result2 = check(bash_code2);
        assert_eq!(
            result2.diagnostics.len(),
            1,
            "Non-arithmetic should be flagged"
        );
    }

    #[test]
    fn test_mutation_column_offset() {
        // MUTATION: Additional test for column calculation edge cases
        let bash_code = "ls ${FILE}";
        let result = check(bash_code);

        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;

        // Verify the span covers the entire variable expression
        // "${FILE}" is 7 characters, span.end_col - span.start_col should be 7
        assert_eq!(
            span.end_col - span.start_col,
            7,
            "Span length should cover ${{FILE}}"
        );
        assert!(span.start_col >= 4, "Should start after 'ls '");
    }

    #[test]
    fn test_mutation_multiline_line_calculation() {
        // MUTATION: Ensures line number calculation handles multiple lines correctly
        let bash_code = r#"echo "line 1"
echo "line 2"
echo $VAR1
echo "line 4"
echo $VAR2"#;

        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            2,
            "Should detect two unquoted variables"
        );

        // Verify line numbers are correct
        assert_eq!(
            result.diagnostics[0].span.start_line, 3,
            "First variable on line 3"
        );
        assert_eq!(
            result.diagnostics[1].span.start_line, 5,
            "Second variable on line 5"
        );
    }

    // ===== Mutation Coverage Tests - Iteration 2 (Helper Functions) =====
    // These tests target the 24 missed mutants from Iteration 1

    // Tests for should_skip_line() helper (6 missed mutants)

    #[test]
    fn test_mutation_should_skip_comment_lines() {
        // MUTATION: Line 22:30 - delete ! in !line.contains("if [")
        // Should skip comments, not flag variables in comments
        let bash_code = "# This is a comment with $VAR\necho $ACTUAL";
        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should only detect $ACTUAL, not $VAR in comment"
        );
        assert!(result.diagnostics[0].message.contains("$ACTUAL"));
    }

    #[test]
    fn test_mutation_should_detect_vars_in_test_conditions() {
        // MUTATION: Line 22:27 - && replaced with || in line.contains('=') && !line.contains("if [")
        // MUTATION: Line 22:53 - && replaced with || in !line.contains("if [") && !line.contains("[ ")
        // Should detect unquoted vars in test conditions, not skip them as assignments
        let bash_code = "if [ $VAR = value ]; then echo ok; fi";
        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should detect $VAR in test condition"
        );
        assert!(result.diagnostics[0].message.contains("$VAR"));
    }

    #[test]
    fn test_mutation_should_skip_simple_assignments() {
        // MUTATION: Line 25:27 - < replaced with <=, >, or ==
        // Should skip variable assignments (eq_pos < first_space)
        let bash_code = "VAR=value\necho $VAR";
        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should only detect $VAR in echo, not assignment"
        );
        assert_eq!(
            result.diagnostics[0].span.start_line, 2,
            "Should be on line 2 (echo)"
        );
    }

    #[test]
    fn test_mutation_assignment_position_boundary() {
        // MUTATION: Line 25:27 - < replaced with <= (boundary condition)
        // Verifies eq_pos < first_space (not <=)
        let bash_code = "X= value\necho $X"; // Space after =, eq_pos < first_space still true
        let result = check(bash_code);
        // Should skip the assignment line and only flag echo
        assert_eq!(result.diagnostics.len(), 1, "Should detect $X in echo only");
    }

    #[test]
    fn test_mutation_should_skip_negation_in_contains() {
        // MUTATION: Line 22:56 - delete ! in !line.contains("[ ")
        // Tests that negation logic is correct (skip assignments, not test conditions)
        let bash_code = r#"
VAR=123
if [ $TEST = ok ]; then
    echo done
fi
"#;
        let result = check(bash_code);
        // Should detect $TEST in the if condition
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should detect $TEST in condition"
        );
        assert!(result.diagnostics[0].message.contains("$TEST"));
    }

    // Tests for find_dollar_position() helper (1 missed mutant)


include!("sc2086_part2_incl2.rs");
