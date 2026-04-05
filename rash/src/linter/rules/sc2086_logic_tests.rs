#[cfg(test)]
mod tests {
    use super::*;

    // ===== SHOULD SKIP LINE =====

    #[test]
    fn test_should_skip_line_comment() {
        assert!(should_skip_line("# this is a comment"));
        assert!(should_skip_line("  # indented comment"));
    }

    #[test]
    fn test_should_skip_line_assignment() {
        // Only skips if there's a space after the assignment
        assert!(should_skip_line("FOO=bar baz")); // assignment before space
                                                  // No space = no skip (simple assignment handled elsewhere)
        assert!(!should_skip_line("VAR=value"));
    }

    #[test]
    fn test_should_skip_line_not_assignment() {
        assert!(!should_skip_line("echo $VAR"));
        assert!(!should_skip_line("ls -la"));
        assert!(!should_skip_line("if [ $x = 1 ]; then")); // condition, not assignment
    }

    #[test]
    fn test_should_skip_line_if_bracket() {
        assert!(!should_skip_line("if [ $x = 1 ]; then"));
        assert!(!should_skip_line("[ $x = 1 ]"));
    }

    #[test]
    fn test_should_skip_line_equals_after_space() {
        // Equals sign is after space - NOT an assignment
        assert!(!should_skip_line("echo x = y")); // command, not assignment
    }

    #[test]
    fn test_should_skip_line_no_space() {
        // No space at all - simple assignment, but no space makes it not skip
        assert!(!should_skip_line("VAR=value"));
    }

    // ===== FIND DOLLAR POSITION =====

    #[test]
    fn test_find_dollar_position_simple() {
        let line = "echo $VAR";
        assert_eq!(find_dollar_position(line, 6), 5); // $ is at position 5
    }

    #[test]
    fn test_find_dollar_position_braced() {
        let line = "echo ${VAR}";
        assert_eq!(find_dollar_position(line, 7), 5); // $ is at position 5
    }

    #[test]
    fn test_find_dollar_position_multiple() {
        let line = "echo $A $B";
        // Looking for B which starts at 9, $ at 8
        assert_eq!(find_dollar_position(line, 9), 8);
    }

    // ===== CALCULATE END COLUMN =====

    #[test]
    fn test_calculate_end_column_simple() {
        let line = "echo $VAR more";
        // var_end is after "VAR" (position 9)
        assert_eq!(calculate_end_column(line, 9, false), 10);
    }

    #[test]
    fn test_calculate_end_column_braced() {
        let line = "echo ${VAR} more";
        // var_end is after "VAR" (position 10), brace at 10
        assert_eq!(calculate_end_column(line, 10, true), 12);
    }

    #[test]
    fn test_calculate_end_column_braced_no_brace() {
        // Edge case: braced=true but no closing brace in remainder
        let line = "echo ${VAR";
        // var_end is at 10, no } found - fallback to var_end + 1
        assert_eq!(calculate_end_column(line, 10, true), 11);
    }

    // ===== IS IN ARITHMETIC CONTEXT =====

    #[test]
    fn test_is_in_arithmetic_context_command_sub() {
        let line = "x=$(( a + b ))";
        // dollar_pos=0 for a, var_end=7
        // Actually we need to find the variable positions correctly
        // "a" is at position 6
        assert!(is_in_arithmetic_context(line, 6, 7));
    }

    #[test]
    fn test_is_in_arithmetic_context_standalone() {
        let line = "(( i++ ))";
        // "i" is at position 3
        assert!(is_in_arithmetic_context(line, 3, 4));
    }

    #[test]
    fn test_is_in_arithmetic_context_for_loop() {
        // In real usage, this checks $var positions, not raw identifiers
        let line = "for (( i=0; i<$n; i++ )); do";
        // $n starts at position 14
        assert!(is_in_arithmetic_context(line, 14, 16));
    }

    #[test]
    fn test_is_in_arithmetic_context_not_arithmetic() {
        let line = "echo $VAR";
        assert!(!is_in_arithmetic_context(line, 5, 8));
    }

    // ===== GET CSTYLE FOR LOOP VARS =====

    #[test]
    fn test_get_cstyle_for_loop_vars_single() {
        let source = "for (( i=0; i<10; i++ )); do echo $i; done";
        let vars = get_cstyle_for_loop_vars(source);
        assert!(vars.contains("i"));
        assert_eq!(vars.len(), 1);
    }

    #[test]
    fn test_get_cstyle_for_loop_vars_multiple() {
        let source =
            "for ((i=0; i<10; i++)); do\n  for ((j=0; j<5; j++)); do\n    echo\n  done\ndone";
        let vars = get_cstyle_for_loop_vars(source);
        assert!(vars.contains("i"));
        assert!(vars.contains("j"));
        assert_eq!(vars.len(), 2);
    }

    #[test]
    fn test_get_cstyle_for_loop_vars_none() {
        let source = "for item in *.txt; do echo $item; done";
        let vars = get_cstyle_for_loop_vars(source);
        assert!(vars.is_empty());
    }

    // ===== IS IN DOUBLE BRACKET CONTEXT =====

    #[test]
    fn test_is_in_double_bracket_context_true() {
        let line = "[[ -n $var ]]";
        // $var: dollar at 6, var ends at 10
        assert!(is_in_double_bracket_context(line, 6, 10));
    }

    #[test]
    fn test_is_in_double_bracket_context_false_single() {
        let line = "[ -n $var ]";
        // Single bracket - word splitting occurs
        assert!(!is_in_double_bracket_context(line, 5, 9));
    }

    #[test]
    fn test_is_in_double_bracket_context_comparison() {
        let line = "[[ $a == $b ]]";
        // $a: dollar at 3, a ends at 5
        assert!(is_in_double_bracket_context(line, 3, 5));
    }

    // ===== IS ALREADY QUOTED =====

    #[test]
    fn test_is_already_quoted_simple() {
        let line = r#"echo "$VAR""#;
        // $VAR: dollar at 6, VAR ends at 10
        assert!(is_already_quoted(line, 6, 10));
    }

    #[test]
    fn test_is_already_quoted_braced() {
        let line = r#"echo "${VAR}""#;
        // ${VAR}: dollar at 6, VAR ends at 11
        assert!(is_already_quoted(line, 6, 11));
    }

    #[test]
    fn test_is_already_quoted_inside_string() {
        let line = r#"echo "prefix${VAR}suffix""#;
        // ${VAR}: dollar at 13, VAR ends at 18
        assert!(is_already_quoted(line, 13, 18));
    }

    #[test]
    fn test_is_already_quoted_not_quoted() {
        let line = "echo $VAR";
        assert!(!is_already_quoted(line, 5, 9));
    }

    #[test]
    fn test_is_already_quoted_edge_case() {
        // Pattern looks quoted due to adjacent quotes
        let line = r#"echo \"$VAR""#;
        // Before ends with " and after starts with " - treated as quoted
        assert!(is_already_quoted(line, 7, 11));
    }

    #[test]
    fn test_is_already_quoted_braced_not_immediately() {
        // Brace starts but no quote immediately around
        let line = r#"cmd ${VAR}end"#;
        // dollar at 4, var ends at 8, after starts with }
        // No quote immediately before or after
        assert!(!is_already_quoted(line, 4, 8));
    }

    #[test]
    fn test_is_already_quoted_odd_quote_braced() {
        // Inside quoted string with braced var - odd quote count
        let line = r#"x="prefix${VAR}suffix""#;
        // One quote before the $, then }suffix" has quote
        // dollar at 10, var ends at 14 (VAR)
        assert!(is_already_quoted(line, 10, 14));
    }

    #[test]
    fn test_is_already_quoted_escaped_quote_before() {
        // Escaped quote doesn't count
        let line = r#"echo \"$VAR"#;
        // The \" is escaped, so quote count is 0 (even)
        // dollar at 7, ends at 11
        assert!(!is_already_quoted(line, 7, 11));
    }

    // ===== FORMAT VAR TEXT =====

    #[test]
    fn test_format_var_text_simple() {
        assert_eq!(format_var_text("VAR", false), "$VAR");
        assert_eq!(format_var_text("foo", false), "$foo");
    }

    #[test]
    fn test_format_var_text_braced() {
        assert_eq!(format_var_text("VAR", true), "${VAR}");
        assert_eq!(format_var_text("foo", true), "${foo}");
    }

    // ===== FORMAT QUOTED VAR =====

    #[test]
    fn test_format_quoted_var_simple() {
        assert_eq!(format_quoted_var("VAR", false), "\"$VAR\"");
    }

    #[test]
    fn test_format_quoted_var_braced() {
        assert_eq!(format_quoted_var("VAR", true), "\"${VAR}\"");
    }

    // ===== LINE HAS ARITHMETIC MARKERS =====

    #[test]
    fn test_line_has_arithmetic_markers_command_sub() {
        assert!(line_has_arithmetic_markers("x=$(( a + b ))"));
    }

    #[test]
    fn test_line_has_arithmetic_markers_standalone() {
        assert!(line_has_arithmetic_markers("(( i++ ))"));
    }

    #[test]
    fn test_line_has_arithmetic_markers_none() {
        assert!(!line_has_arithmetic_markers("echo $VAR"));
        assert!(!line_has_arithmetic_markers("ls -la"));
    }
}
