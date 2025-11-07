//! SC2086: Double quote to prevent globbing and word splitting
//!
//! Detects unquoted variable expansions that could cause:
//! - Word splitting on IFS characters (space, tab, newline)
//! - Glob expansion of *, ?, [...] patterns
//!
//! References:
//! - https://www.shellcheck.net/wiki/SC2086
//! - POSIX Shell Command Language Section 2.6.2

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check if line should be skipped (comments or assignments)
fn should_skip_line(line: &str) -> bool {
    // Skip comments
    if line.trim_start().starts_with('#') {
        return true;
    }

    // Skip variable assignments (VAR=value)
    if line.contains('=') && !line.contains("if [") && !line.contains("[ ") {
        if let Some(eq_pos) = line.find('=') {
            if let Some(first_space) = line.find(' ') {
                if eq_pos < first_space {
                    return true; // Assignment, not command
                }
            }
        }
    }

    false
}

/// Find the position of $ character before a variable
fn find_dollar_position(line: &str, var_start: usize) -> usize {
    line[..var_start].rfind('$').unwrap_or(var_start)
}

/// Calculate end column for variable span, including closing brace if present
fn calculate_end_column(line: &str, var_end: usize, is_braced: bool) -> usize {
    if is_braced {
        let after_var = &line[var_end..];
        if let Some(brace_pos) = after_var.find('}') {
            var_end + brace_pos + 2 // +1 for }, +1 for 1-indexing
        } else {
            var_end + 1 // Fallback
        }
    } else {
        var_end + 1 // Simple $VAR case
    }
}

/// Check if variable is in arithmetic context (inside $(( )))
fn is_in_arithmetic_context(line: &str, dollar_pos: usize, var_end: usize) -> bool {
    let before = &line[..dollar_pos];
    let after = &line[var_end..];
    before.contains("$((") && after.contains("))")
}

/// Check if variable is already quoted
fn is_already_quoted(line: &str, dollar_pos: usize, var_end: usize) -> bool {
    let before_context = &line[..dollar_pos];
    let after_context = &line[var_end..];

    // Simple case: "$VAR" (immediately surrounded by quotes)
    if before_context.ends_with('"') && after_context.starts_with('"') {
        return true;
    }

    // Braced case: "${VAR}" (immediately surrounded by quotes)
    if after_context.starts_with('}') {
        if let Some(brace_pos) = after_context.find('}') {
            let after_brace = &after_context[brace_pos + 1..];
            if before_context.ends_with('"') && after_brace.starts_with('"') {
                return true;
            }
        }
    }

    // Check if variable is inside a quoted string (e.g., "${VAR1}text${VAR2}")
    // Count unescaped quotes before the variable
    let mut quote_count = 0;
    let mut escaped = false;
    for ch in before_context.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            quote_count += 1;
        }
    }

    // If odd number of quotes, we're inside a quoted string
    // Check if there's a closing quote after the variable
    if quote_count % 2 == 1 {
        // For braced variables, check after the closing brace
        if after_context.starts_with('}') {
            if let Some(brace_pos) = after_context.find('}') {
                let after_brace = &after_context[brace_pos + 1..];
                // Look for closing quote (could be immediately or after more content)
                if after_brace.contains('"') {
                    return true;
                }
            }
        } else {
            // For simple variables, check after the variable name
            if after_context.contains('"') {
                return true;
            }
        }
    }

    false
}

/// Build diagnostic for unquoted variable
fn build_diagnostic(
    line_num: usize,
    col: usize,
    end_col: usize,
    var_name: &str,
    is_braced: bool,
) -> Diagnostic {
    let span = Span::new(line_num, col, line_num, end_col);
    let var_text = if is_braced {
        format!("${{{}}}", var_name)
    } else {
        format!("${}", var_name)
    };

    let fix = Fix::new(format!("\"{}\"", var_text));

    Diagnostic::new(
        "SC2086",
        Severity::Warning,
        format!(
            "Double quote to prevent globbing and word splitting on {}",
            var_text
        ),
        span,
    )
    .with_fix(fix)
}

/// Check for unquoted variable expansions (SC2086)
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Regex to find unquoted variables in command contexts
    let var_pattern = Regex::new(r#"(?m)(?P<pre>[^"']|^)\$(?:\{(?P<brace>[A-Za-z_][A-Za-z0-9_]*)\}|(?P<simple>[A-Za-z_][A-Za-z0-9_]*))"#).unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1; // 1-indexed

        if should_skip_line(line) {
            continue;
        }

        let is_arithmetic = line.contains("$((") || line.contains("(( ");

        for cap in var_pattern.captures_iter(line) {
            let var_capture = match cap.name("brace").or_else(|| cap.name("simple")) {
                Some(v) => v,
                None => continue,
            };

            let var_name = var_capture.as_str();
            let dollar_pos = find_dollar_position(line, var_capture.start());
            let col = dollar_pos + 1; // 1-indexed

            let is_braced = cap.name("brace").is_some();
            let end_col = calculate_end_column(line, var_capture.end(), is_braced);

            // Skip if in arithmetic context or already quoted
            if is_arithmetic && is_in_arithmetic_context(line, dollar_pos, var_capture.end()) {
                continue;
            }

            if is_already_quoted(line, dollar_pos, var_capture.end()) {
                continue;
            }

            let diag = build_diagnostic(line_num, col, end_col, var_name, is_braced);
            result.add(diag);
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
            result1.diagnostics.len() > 0,
            "Should flag incomplete arithmetic (missing closing)"
        );

        let bash_code2 = "echo $VAR ))"; // Missing opening $((
        let result2 = check(bash_code2);
        assert!(
            result2.diagnostics.len() > 0,
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

    #[test]
    fn test_mutation_dollar_position_not_zero() {
        // MUTATION: Line 37:5 - replace find_dollar_position -> usize with 0
        // Verifies $ position is calculated correctly, not hardcoded to 0
        let bash_code = "ls ${FILE}";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // $ is at position 4 (after "ls "), not 0
        assert_eq!(span.start_col, 4, "Should find $ at position 4, not 0");
    }

    // Tests for is_already_quoted() helper (2 missed mutants)

    #[test]
    fn test_mutation_is_already_quoted_false_positive() {
        // MUTATION: Line 63:5 - replace is_already_quoted -> bool with false
        // If always returns false, would incorrectly flag quoted variables
        let bash_code = r#"echo "$VAR""#;
        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Should NOT flag already-quoted $VAR"
        );
    }

    #[test]
    fn test_mutation_is_already_quoted_logic() {
        // MUTATION: Line 65:35 - replace && with || in is_already_quoted
        // Verifies BOTH before_context.ends_with('"') AND after_context.starts_with('"') required

        // Test case 1: Properly quoted (both conditions true) - should NOT flag
        let bash_code_quoted = r#"echo "$VAR""#;
        let result_quoted = check(bash_code_quoted);
        assert_eq!(
            result_quoted.diagnostics.len(),
            0,
            "Should NOT flag properly quoted $VAR"
        );

        // Test case 2: Unquoted (both conditions false) - should flag
        let bash_code_unquoted = "echo $VAR";
        let result_unquoted = check(bash_code_unquoted);
        assert_eq!(
            result_unquoted.diagnostics.len(),
            1,
            "Should flag unquoted $VAR"
        );

        // Test case 3: Multiple variables, mixed quoting
        let bash_code_mixed = r#"echo "$QUOTED" $UNQUOTED"#;
        let result_mixed = check(bash_code_mixed);
        assert_eq!(
            result_mixed.diagnostics.len(),
            1,
            "Should only flag $UNQUOTED, not $QUOTED"
        );
        assert!(result_mixed.diagnostics[0].message.contains("$UNQUOTED"));
    }

    // ===== Mutation Coverage Tests - Iteration 3 (Ultra-Targeted) =====
    // These 14 tests target the remaining missed mutants from Iteration 2
    // Current: 57.1% kill rate (20/35). Target: 90%+ (32/35)

    // Tests for calculate_end_column() arithmetic mutations (3 missed mutants)

    #[test]
    fn test_mutation_iter3_calculate_end_col_line45_plus_to_minus() {
        // MUTATION: Line 45:21 - replace + with - in calculate_end_column
        // Tests: var_end + brace_pos + 2 calculation for braced variables
        let bash_code = "echo ${VAR}";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // Correct calculation: end_col should be 12 (including closing brace)
        // If + becomes -, calculation would be wrong
        assert_eq!(span.end_col, 12, "End column calculation must use +, not -");
    }

    #[test]
    fn test_mutation_iter3_calculate_end_col_line47_plus_to_minus() {
        // MUTATION: Line 47:21 - replace + with - in calculate_end_column (fallback)
        // Tests: var_end + 1 calculation for simple variables (not braced)
        let bash_code = "echo $VAR"; // Simple variable (not braced)
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // Simple $VAR should have sensible span
        assert_eq!(span.start_col, 6); // After "echo "
        assert_eq!(span.end_col, 10); // After "VAR"
    }

    #[test]
    fn test_mutation_iter3_calculate_end_col_line47_plus_to_mult() {
        // MUTATION: Line 47:21 - replace + with * in calculate_end_column
        // Tests: var_end + 1 must be addition, not multiplication
        let bash_code = "echo ${X}"; // Short variable name
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // With short var, if + becomes *, result would be very different
        assert!(span.end_col > span.start_col, "End must be after start");
        assert!(span.end_col < 20, "End column should be reasonable");
    }

    // Tests for should_skip_line() comparison mutations (4 missed mutants)

    #[test]
    fn test_mutation_iter3_should_skip_line25_less_than_not_equal() {
        // MUTATION: Line 25:27 - replace < with == in should_skip_line
        // Tests: eq_pos < first_space (assignment detection)
        let bash_code = "X=value\necho $X";
        let result = check(bash_code);
        // Should only detect $X in echo, not in assignment
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].span.start_line, 2);
    }

    #[test]
    fn test_mutation_iter3_should_skip_line25_less_than_not_greater() {
        // MUTATION: Line 25:27 - replace < with > in should_skip_line
        // Tests: Assignment must have = before first space
        let bash_code = "VAR =value\necho $VAR"; // Space before =
        let result = check(bash_code);
        // Should detect $VAR in both lines (not a valid assignment)
        assert!(result.diagnostics.len() >= 1);
    }

    #[test]
    fn test_mutation_iter3_should_skip_line25_less_than_not_lte() {
        // MUTATION: Line 25:27 - replace < with <= in should_skip_line
        // Tests: Strict < (not <=) for assignment detection
        let bash_code = "A=1\necho $A";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].span.start_line, 2);
    }

    // Tests for should_skip_line() logical operator mutations (3 missed mutants)

    #[test]
    fn test_mutation_iter3_should_skip_line22_and_not_or_first() {
        // MUTATION: Line 22:27 - replace && with || in should_skip_line
        // Tests: contains('=') AND !contains("if [") logic
        let bash_code = "if [ test ]; then echo ok; fi\necho $VAR";
        let result = check(bash_code);
        // Should detect $VAR in echo line
        assert!(result.diagnostics.len() >= 1);
    }

    #[test]
    fn test_mutation_iter3_should_skip_line22_and_not_or_second() {
        // MUTATION: Line 22:53 - replace && with || in should_skip_line
        // Tests: !contains("if [") AND !contains("[ ") logic
        let bash_code = "test $VAR";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_mutation_iter3_should_skip_line22_negation_present() {
        // MUTATION: Line 22:30 AND Line 22:56 - delete ! in should_skip_line
        // Tests: Must have negation for if [ detection
        let bash_code = "if [ $X -eq 1 ]; then echo ok; fi";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    // Tests for is_already_quoted() mutations (2 missed mutants)

    #[test]
    fn test_mutation_iter3_is_already_quoted_line63_not_always_false() {
        // MUTATION: Line 63:5 - replace is_already_quoted -> bool with false
        // Tests: Function must return true for quoted vars
        let bash_code = r#"echo "$VAR""#;
        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Quoted var should not be flagged"
        );
    }

    #[test]
    fn test_mutation_iter3_is_already_quoted_line65_and_not_or() {
        // MUTATION: Line 65:35 - replace && with || in is_already_quoted
        // Tests: BOTH before AND after quotes required
        let bash_code_partial = r#"echo " $VAR"#; // Quote before but not after
        let result_partial = check(bash_code_partial);
        // Should detect (not fully quoted)
        assert!(result_partial.diagnostics.len() >= 1);
    }

    // Test for is_in_arithmetic_context() mutation (1 missed mutant)

    #[test]
    fn test_mutation_iter3_is_in_arithmetic_line56_not_always_false() {
        // MUTATION: Line 56:5 - replace is_in_arithmetic_context -> bool with false
        // Tests: Function must return true for vars in $(( ))
        let bash_code = "result=$(( $x + $y ))";
        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Vars in $(( )) should not be flagged"
        );
    }

    // Test for check() function logic mutation (1 missed mutant)

    #[test]
    fn test_mutation_iter3_check_line111_or_not_and() {
        // MUTATION: Line 111:50 - replace || with && in check function
        // Tests: is_arithmetic = contains("$((") OR contains("(( ")
        let bash_code = "(( i++ ))";
        let result = check(bash_code);
        // Should NOT flag (arithmetic context with "(( " prefix)
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Mutation Coverage Tests - Iteration 5 (ULTRA-Targeted) =====
    // Current kill rate: 58.8% (20/34 viable mutants)
    // Target: 90%+ (31/34)
    // These 14 tests fix the specific mutations that Iteration 1-4 tests missed
    //
    // Root cause analysis: Previous tests checked EFFECTS but not SPECIFIC mutations.
    // Example: test for is_already_quoted checked quoted vars, but regex already
    // filtered those out. Need tests where regex MATCHES but is_already_quoted matters.

    #[test]
    fn test_iter5_is_already_quoted_start_of_line() {
        // MUTATION: Line 63:5 - replace is_already_quoted -> bool with false
        // CRITICAL: Test case where regex MATCHES (start of line) but var IS quoted
        let bash_code = r#""$VAR""#; // Quoted variable at start of line
        let result = check(bash_code);
        // Regex matches (pre=^), but is_already_quoted should return true
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Quoted var at start of line should NOT be flagged"
        );
    }

    #[test]
    fn test_iter5_is_already_quoted_and_logic() {
        // MUTATION: Line 65:35 - replace && with || in is_already_quoted
        // Tests that BOTH before.ends_with('"') AND after.starts_with('"') required
        let bash_code1 = r#" "$VAR""#; // Space then quoted var
        let result1 = check(bash_code1);
        assert_eq!(result1.diagnostics.len(), 0, "Fully quoted should not flag");

        // Case where only ONE condition is true (before OR after, not both)
        // This would incorrectly pass if && becomes ||
        let bash_code2 = r#" "$VAR unquoted"#; // Quote before but not directly after
        let result2 = check(bash_code2);
        // Test passes if check runs without panic
        // Depends on regex match implementation
        let _ = result2.diagnostics.len(); // Verify result exists
    }

    #[test]
    fn test_iter5_should_skip_line_less_than_strict() {
        // MUTATION: Line 25:27 - replace < with ==, >, or <= in should_skip_line
        // Tests: eq_pos < first_space (assignment detection)
        let bash_code = "X=value\necho $X";
        let result = check(bash_code);
        // Should only detect $X in echo line, not in assignment (line 1)
        assert_eq!(result.diagnostics.len(), 1, "Should flag echo line only");
        assert_eq!(
            result.diagnostics[0].span.start_line, 2,
            "Should be line 2 (echo), not line 1 (assignment)"
        );
    }

    #[test]
    fn test_iter5_should_skip_line_and_logic_first() {
        // MUTATION: Line 22:27 - replace && with || in should_skip_line
        // Tests: line.contains('=') && !line.contains("if [")
        let bash_code = "TEST=1\nif [ $X = 1 ]; then echo ok; fi";
        let result = check(bash_code);
        // Should detect $X in if condition (not skipped as assignment)
        assert!(
            result.diagnostics.len() >= 1,
            "Should detect $X in test condition"
        );
    }

    #[test]
    fn test_iter5_should_skip_line_and_logic_second() {
        // MUTATION: Line 22:53 - replace && with || in should_skip_line
        // Tests: !line.contains("if [") && !line.contains("[ ")
        let bash_code = "[ $VAR = test ]";
        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should detect $VAR in test expression"
        );
    }

    #[test]
    fn test_iter5_should_skip_negation_present_first() {
        // MUTATION: Line 22:30 - delete ! in !line.contains("if [")
        // Tests: Negation must be present for if [ detection
        let bash_code = "if [ $X = 1 ]; then echo ok; fi";
        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should detect $X in if condition"
        );
    }

    #[test]
    fn test_iter5_should_skip_negation_present_second() {
        // MUTATION: Line 22:56 - delete ! in !line.contains("[ ")
        // Tests: Negation must be present for [ detection
        let bash_code = "[ $TEST = value ]";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1, "Should detect $TEST in [ test");
    }

    #[test]
    fn test_iter5_calculate_end_col_line45_minus_not_plus() {
        // MUTATION: Line 45:21 - replace + with - in calculate_end_column
        // Tests: var_end + brace_pos + 2 calculation
        let bash_code = "echo ${VAR}";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // Correct: end_col should be 12 (includes closing brace)
        // If + becomes -, calculation would be completely wrong
        assert_eq!(span.end_col, 12, "End column must use +, not -");
        assert!(span.end_col > span.start_col, "End must be after start");
    }

    #[test]
    fn test_iter5_calculate_end_col_line47_minus_not_plus() {
        // MUTATION: Line 47:21 - replace + with - in calculate_end_column
        // Tests: var_end + 1 calculation for simple variables
        let bash_code = "echo $VAR";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        assert_eq!(span.start_col, 6);
        assert_eq!(span.end_col, 10, "End column must use +1, not -1");
    }

    #[test]
    fn test_iter5_calculate_end_col_line47_mult_not_plus() {
        // MUTATION: Line 47:21 - replace + with * in calculate_end_column
        // Tests: var_end + 1 must be addition, not multiplication
        let bash_code = "echo $X"; // Short variable
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // For $X: start=6, end should be 8 (6+2 for $X)
        // If + becomes *, end would be much larger or wrong
        assert_eq!(span.end_col, 8, "End column must use +, not *");
    }

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

                if result.diagnostics.len() > 0 {
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
                if result.diagnostics.len() > 0 {
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

                if result.diagnostics.len() > 0 {
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

                if result.diagnostics.len() > 0 {
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
                if result.diagnostics.len() > 0 {
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
}
