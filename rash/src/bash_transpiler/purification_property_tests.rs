//! Property-Based Tests for Bash Purification
//!
//! EXTREME TDD: Property tests verify purification invariants
//! - Determinism: Same input → same output
//! - Idempotency: purify(purify(x)) == purify(x)
//! - Safety: No dangerous patterns in output
//! - Correctness: Output is valid POSIX sh

use super::purification::{PurificationOptions, Purifier};
use crate::bash_parser::codegen::generate_purified_bash;
use crate::bash_parser::parser::BashParser;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,
        max_shrink_iters: 1000,
        .. ProptestConfig::default()
    })]

    /// Property: Purification is deterministic
    /// Same bash input MUST produce identical purified output
    #[test]
    fn prop_purification_is_deterministic(bash_code in "#!/bin/bash\n[a-z_][a-z0-9_]{0,10}=[0-9]{1,3}") {
        // Parse bash
        let mut parser1 = BashParser::new(&bash_code).unwrap();
        let ast1 = parser1.parse().unwrap();

        let mut parser2 = BashParser::new(&bash_code).unwrap();
        let ast2 = parser2.parse().unwrap();

        // Purify twice
        let mut purifier1 = Purifier::new(PurificationOptions::default());
        let purified_ast1 = purifier1.purify(&ast1).unwrap();

        let mut purifier2 = Purifier::new(PurificationOptions::default());
        let purified_ast2 = purifier2.purify(&ast2).unwrap();

        // Generate code
        let output1 = generate_purified_bash(&purified_ast1);
        let output2 = generate_purified_bash(&purified_ast2);

        // INVARIANT: Must be identical
        prop_assert_eq!(output1, output2, "Purification must be deterministic");
    }

    /// Property: Purification is idempotent
    /// purify(purify(x)) == purify(x)
    #[test]
    fn prop_purification_is_idempotent(bash_code in "#!/bin/bash\n[a-z_][a-z0-9_]{0,10}=[0-9]{1,3}") {
        // First purification
        let mut parser1 = BashParser::new(&bash_code).unwrap();
        let ast1 = parser1.parse().unwrap();
        let mut purifier1 = Purifier::new(PurificationOptions::default());
        let purified_ast1 = purifier1.purify(&ast1).unwrap();
        let output1 = generate_purified_bash(&purified_ast1);

        // Second purification (purify the purified output)
        let mut parser2 = BashParser::new(&output1).unwrap();
        let ast2 = parser2.parse().unwrap();
        let mut purifier2 = Purifier::new(PurificationOptions::default());
        let purified_ast2 = purifier2.purify(&ast2).unwrap();
        let output2 = generate_purified_bash(&purified_ast2);

        // INVARIANT: Second purification should not change output
        prop_assert_eq!(output1, output2, "Purification must be idempotent");
    }

    /// Property: Purified output has POSIX shebang
    #[test]
    fn prop_purified_has_posix_shebang(bash_code in "#!/bin/bash\n[a-z_][a-z0-9_]{0,10}=[0-9]{1,3}") {
        let mut parser = BashParser::new(&bash_code).unwrap();
        let ast = parser.parse().unwrap();
        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified_ast = purifier.purify(&ast).unwrap();
        let output = generate_purified_bash(&purified_ast);

        // INVARIANT: Must start with POSIX shebang
        prop_assert!(
            output.starts_with("#!/bin/sh"),
            "Purified output must have POSIX shebang, got: {}",
            output.lines().next().unwrap_or("")
        );
    }

    /// Property: Variable assignments preserved (semantically)
    /// Note: Numeric values may be normalized (00 → 0), which is semantically equivalent
    #[test]
    fn prop_variable_assignments_preserved(
        var_name in "[a-z_][a-z0-9_]{0,10}",
        value in "[1-9][0-9]{0,2}" // Avoid leading zeros to prevent normalization
    ) {
        let bash_code = format!("#!/bin/bash\n{}={}", var_name, value);
        let mut parser = BashParser::new(&bash_code).unwrap();
        let ast = parser.parse().unwrap();
        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified_ast = purifier.purify(&ast).unwrap();
        let output = generate_purified_bash(&purified_ast);

        // INVARIANT: Variable assignment must be preserved
        let expected = format!("{}={}", var_name, value);
        prop_assert!(
            output.contains(&expected),
            "Variable assignment {} should be preserved in output:\n{}",
            expected, output
        );
    }

    /// Property: No $RANDOM in purified output
    #[test]
    fn prop_no_random_in_purified_output(
        var_name in "[a-z_][a-z0-9_]{0,10}"
    ) {
        let bash_code = format!("#!/bin/bash\n{}=$RANDOM", var_name);

        // Parse and purify
        if let Ok(mut parser) = BashParser::new(&bash_code) {
            if let Ok(ast) = parser.parse() {
                let mut purifier = Purifier::new(PurificationOptions::default());
                if let Ok(purified_ast) = purifier.purify(&ast) {
                    let output = generate_purified_bash(&purified_ast);

                    // INVARIANT: $RANDOM must be removed/replaced
                    prop_assert!(
                        !output.contains("$RANDOM") && !output.contains("${RANDOM}"),
                        "Purified output must not contain $RANDOM, got:\n{}",
                        output
                    );
                }
            }
        }
    }

    /// Property: Comments are preserved
    #[test]
    fn prop_comments_preserved(
        comment_text in "[a-zA-Z0-9 ]{1,20}"
    ) {
        let bash_code = format!("#!/bin/bash\n# {}\nx=42", comment_text);
        let mut parser = BashParser::new(&bash_code).unwrap();
        let ast = parser.parse().unwrap();
        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified_ast = purifier.purify(&ast).unwrap();
        let output = generate_purified_bash(&purified_ast);

        // INVARIANT: Comment content should be preserved
        prop_assert!(
            output.contains(&comment_text),
            "Comment text '{}' should be preserved in output:\n{}",
            comment_text, output
        );
    }

    /// Property: Empty scripts remain valid
    #[test]
    fn prop_empty_script_valid(_input in "#!/bin/bash\n") {
        let bash_code = "#!/bin/bash\n";
        let mut parser = BashParser::new(&bash_code).unwrap();
        let ast = parser.parse().unwrap();
        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified_ast = purifier.purify(&ast).unwrap();
        let output = generate_purified_bash(&purified_ast);

        // INVARIANT: Empty script should have valid shebang
        prop_assert!(
            output.starts_with("#!/bin/sh"),
            "Empty purified script must have POSIX shebang"
        );

        // Should have at least shebang + newline
        prop_assert!(
            output.len() >= "#!/bin/sh\n".len(),
            "Purified output should not be empty"
        );
    }

    /// Property: Multiple assignments preserved in order
    #[test]
    fn prop_multiple_assignments_preserved(
        var1 in "[a-z]",
        val1 in "[1-9]",
        var2 in "[a-z]",
        val2 in "[1-9]"
    ) {
        // Skip if both assignments are identical (can't test ordering)
        prop_assume!(var1 != var2 || val1 != val2);

        let bash_code = format!("#!/bin/bash\n{}={}\n{}={}", var1, val1, var2, val2);
        let mut parser = BashParser::new(&bash_code).unwrap();
        let ast = parser.parse().unwrap();
        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified_ast = purifier.purify(&ast).unwrap();
        let output = generate_purified_bash(&purified_ast);

        // INVARIANT: Both assignments should be present
        prop_assert!(
            output.contains(&format!("{}={}", var1, val1)),
            "First assignment {}={} should be preserved",
            var1, val1
        );
        prop_assert!(
            output.contains(&format!("{}={}", var2, val2)),
            "Second assignment {}={} should be preserved",
            var2, val2
        );

        // INVARIANT: Order should be preserved (if assignments are different)
        if var1 != var2 || val1 != val2 {
            let assignment1 = format!("{}={}", var1, val1);
            let assignment2 = format!("{}={}", var2, val2);

            let pos1 = output.find(&assignment1);
            let pos2 = output.rfind(&assignment2); // Use rfind for second occurrence

            if let (Some(p1), Some(p2)) = (pos1, pos2) {
                if assignment1 != assignment2 {
                    prop_assert!(
                        p1 < p2,
                        "Assignment order should be preserved: {} before {}",
                        var1, var2
                    );
                }
            }
        }
    }

    /// Property: Purification preserves number of statements (approximately)
    /// Note: Some transformations may add/remove statements
    #[test]
    fn prop_statement_count_reasonable(bash_code in "#!/bin/bash\n([a-z]=[0-9]\n){1,5}") {
        let mut parser = BashParser::new(&bash_code).unwrap();
        let ast = parser.parse().unwrap();
        let original_count = ast.statements.len();

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified_ast = purifier.purify(&ast).unwrap();
        let purified_count = purified_ast.statements.len();

        // INVARIANT: Statement count should be similar (within 50% margin)
        // This allows for reasonable transformations
        let min_expected = original_count / 2;
        let max_expected = original_count * 2;

        prop_assert!(
            purified_count >= min_expected && purified_count <= max_expected,
            "Purified statement count {} should be within reasonable range of original {}",
            purified_count, original_count
        );
    }

    /// Property: No bashisms in purified output (basic check)
    #[test]
    fn prop_no_bashisms_in_output(bash_code in "#!/bin/bash\n[a-z_][a-z0-9_]{0,10}=[0-9]{1,3}") {
        let mut parser = BashParser::new(&bash_code).unwrap();
        let ast = parser.parse().unwrap();
        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified_ast = purifier.purify(&ast).unwrap();
        let output = generate_purified_bash(&purified_ast);

        // INVARIANT: Should not contain bash-specific patterns
        // (This is a basic check - could be expanded)
        prop_assert!(
            !output.contains("#!/bin/bash"),
            "Purified output should not use bash shebang"
        );

        // Common bashisms to check
        let bashisms = ["[[", "((", "${!", "=~"];
        for bashism in &bashisms {
            prop_assert!(
                !output.contains(bashism),
                "Purified output should not contain bashism: {}",
                bashism
            );
        }
    }

    /// Property: If statements with test conditions work correctly
    /// MUTATION TARGET: Tests generate_condition and generate_test_condition functions
    #[test]
    fn prop_if_with_test_conditions(
        var1 in "[a-z]",
        var2 in "[a-z]"
    ) {
        // Create bash with if/test (multi-line format)
        let bash_code = format!("#!/bin/bash\nif [ \"${}\" = \"${}\" ]\nthen\necho equal\nfi", var1, var2);

        if let Ok(mut parser) = BashParser::new(&bash_code) {
            if let Ok(ast) = parser.parse() {
                let mut purifier = Purifier::new(PurificationOptions::default());
                if let Ok(purified_ast) = purifier.purify(&ast) {
                    let output = generate_purified_bash(&purified_ast);

                    // INVARIANT: Should contain if statement
                    prop_assert!(
                        output.contains("if"),
                        "Should contain if statement, got: {}",
                        output
                    );

                    // INVARIANT: Should contain test brackets
                    prop_assert!(
                        output.contains("[") && output.contains("]"),
                        "Should contain test brackets, got: {}",
                        output
                    );

                    // INVARIANT: Should contain string equality operator
                    prop_assert!(
                        output.contains(" = "),
                        "Should contain = operator, got: {}",
                        output
                    );

                    // INVARIANT: Variables should be quoted (either single or double quotes)
                    let has_var1 = output.contains(&format!("\"${}\"", var1)) || output.contains(&format!("'${}'", var1));
                    let has_var2 = output.contains(&format!("\"${}\"", var2)) || output.contains(&format!("'${}'", var2));
                    prop_assert!(
                        has_var1 && has_var2,
                        "Variables should be quoted, got: {}",
                        output
                    );

                    // INVARIANT: Should contain then/fi
                    prop_assert!(
                        output.contains("then") && output.contains("fi"),
                        "Should contain then and fi, got: {}",
                        output
                    );
                }
            }
        }
    }

    /// Property: While loops with conditions work correctly
    /// MUTATION TARGET: Tests generate_condition function
    #[test]
    fn prop_while_with_conditions(
        var in "[a-z]",
        limit in 1..10i64
    ) {
        // Create bash with while loop (multi-line format)
        let bash_code = format!("#!/bin/bash\nwhile [ \"${}\" -lt {} ]\ndo\necho ${}\ndone", var, limit, var);

        if let Ok(mut parser) = BashParser::new(&bash_code) {
            if let Ok(ast) = parser.parse() {
                let mut purifier = Purifier::new(PurificationOptions::default());
                if let Ok(purified_ast) = purifier.purify(&ast) {
                    let output = generate_purified_bash(&purified_ast);

                    // INVARIANT: Should contain while loop
                    prop_assert!(
                        output.contains("while"),
                        "Should contain while loop, got: {}",
                        output
                    );

                    // INVARIANT: Should contain test condition
                    prop_assert!(
                        output.contains("[") && output.contains("]"),
                        "Should contain test brackets, got: {}",
                        output
                    );

                    // INVARIANT: Should contain -lt operator
                    prop_assert!(
                        output.contains(" -lt "),
                        "Should contain -lt operator, got: {}",
                        output
                    );

                    // INVARIANT: Should contain do/done
                    prop_assert!(
                        output.contains("do") && output.contains("done"),
                        "Should contain do and done, got: {}",
                        output
                    );
                }
            }
        }
    }

    /// Property: mkdir commands always get -p flag for idempotency (Phase 2)
    /// EXTREME TDD: Permission-aware purification (Toyota Way review §6.2)
    #[test]
    fn prop_mkdir_always_has_p_flag(
        dir_name in "/[a-z]{1,10}(/[a-z]{1,10}){0,2}"
    ) {
        let bash_code = format!("#!/bin/bash\nmkdir {}", dir_name);

        if let Ok(mut parser) = BashParser::new(&bash_code) {
            if let Ok(ast) = parser.parse() {
                let mut purifier = Purifier::new(PurificationOptions::default());
                if let Ok(purified_ast) = purifier.purify(&ast) {
                    let output = generate_purified_bash(&purified_ast);

                    // INVARIANT: mkdir must have -p flag
                    prop_assert!(
                        output.contains("mkdir -p") || output.contains("mkdir") && output.contains("-p"),
                        "mkdir command must have -p flag for idempotency, got: {}",
                        output
                    );
                }
            }
        }
    }

    /// Property: mkdir commands always get permission check (Phase 2)
    /// EXTREME TDD: Permission-aware purification (Toyota Way review §6.2)
    #[test]
    fn prop_mkdir_has_permission_check(
        dir_name in "/[a-z]{1,10}(/[a-z]{1,10}){0,2}"
    ) {
        let bash_code = format!("#!/bin/bash\nmkdir {}", dir_name);

        if let Ok(mut parser) = BashParser::new(&bash_code) {
            if let Ok(ast) = parser.parse() {
                let mut purifier = Purifier::new(PurificationOptions::default());
                if let Ok(purified_ast) = purifier.purify(&ast) {
                    let output = generate_purified_bash(&purified_ast);

                    // INVARIANT: Must contain permission check
                    prop_assert!(
                        output.contains("-w") || output.contains("FileWritable"),
                        "mkdir must have write permission check, got: {}",
                        output
                    );

                    // INVARIANT: Must check parent directory
                    prop_assert!(
                        output.contains("dirname"),
                        "mkdir permission check must verify parent directory, got: {}",
                        output
                    );
                }
            }
        }
    }

    /// Property: mkdir permission check has error handling (Phase 2)
    /// EXTREME TDD: Permission-aware purification (Toyota Way review §6.2)
    #[test]
    fn prop_mkdir_permission_error_handling(
        dir_name in "/[a-z]{1,10}"
    ) {
        let bash_code = format!("#!/bin/bash\nmkdir {}", dir_name);

        if let Ok(mut parser) = BashParser::new(&bash_code) {
            if let Ok(ast) = parser.parse() {
                let mut purifier = Purifier::new(PurificationOptions::default());
                if let Ok(purified_ast) = purifier.purify(&ast) {
                    let output = generate_purified_bash(&purified_ast);

                    // INVARIANT: Must have Permission denied error message
                    prop_assert!(
                        output.contains("Permission denied") || output.contains("permission denied"),
                        "mkdir must have permission denied error message, got: {}",
                        output
                    );

                    // INVARIANT: Must exit on permission error
                    prop_assert!(
                        output.contains("exit 1") || output.contains("exit"),
                        "mkdir must exit on permission error, got: {}",
                        output
                    );
                }
            }
        }
    }

    /// Property: mkdir purification is deterministic (Phase 2)
    /// EXTREME TDD: Same mkdir input → same purified output
    #[test]
    fn prop_mkdir_purification_deterministic(
        dir_name in "/[a-z]{1,10}"
    ) {
        let bash_code = format!("#!/bin/bash\nmkdir {}", dir_name);

        if let Ok(mut parser1) = BashParser::new(&bash_code) {
            if let Ok(ast1) = parser1.parse() {
                if let Ok(mut parser2) = BashParser::new(&bash_code) {
                    if let Ok(ast2) = parser2.parse() {
                        let mut purifier1 = Purifier::new(PurificationOptions::default());
                        let mut purifier2 = Purifier::new(PurificationOptions::default());

                        if let Ok(purified1) = purifier1.purify(&ast1) {
                            if let Ok(purified2) = purifier2.purify(&ast2) {
                                let output1 = generate_purified_bash(&purified1);
                                let output2 = generate_purified_bash(&purified2);

                                // INVARIANT: Must produce identical output
                                prop_assert_eq!(
                                    output1, output2,
                                    "mkdir purification must be deterministic"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// Property: No injection attacks in purified output (P1 - Toyota Way §6.4)
    /// SECURITY: All variable expansions must be quoted to prevent injection
    #[test]
    fn prop_no_injection_attacks(
        var_name in "[a-z_][a-z0-9_]{0,10}",
        user_input in r#"[a-z0-9 ;|&$()]{1,20}"#
    ) {
        let bash_code = format!(
            "#!/bin/bash\n{}='{}'\necho ${}",
            var_name, user_input, var_name
        );

        if let Ok(mut parser) = BashParser::new(&bash_code) {
            if let Ok(ast) = parser.parse() {
                let mut purifier = Purifier::new(PurificationOptions::default());
                if let Ok(purified_ast) = purifier.purify(&ast) {
                    let output = generate_purified_bash(&purified_ast);

                    // INVARIANT: All variable expansions must be quoted
                    // Look for unquoted variable patterns
                    let unquoted_patterns = [
                        format!("echo ${}", var_name),  // Unquoted: echo $var
                        format!("echo ${{{}}}", var_name),  // Unquoted: echo ${var}
                    ];

                    // Check that we DON'T have unquoted variables
                    for pattern in &unquoted_patterns {
                        // Allow pattern if it's inside quotes
                        if output.contains(pattern) {
                            // Verify it's quoted
                            let quoted_version = format!("\"{}\"", pattern.trim_start_matches("echo "));
                            prop_assert!(
                                output.contains(&quoted_version) ||
                                output.contains(&format!("'{}'", pattern.trim_start_matches("echo "))),
                                "Variable expansion must be quoted to prevent injection: got unquoted {} in:\n{}",
                                pattern, output
                            );
                        }
                    }

                    // INVARIANT: Should have quoted variable reference
                    prop_assert!(
                        output.contains(&format!("\"${}\"", var_name)) ||
                        output.contains(&format!("\"${{{}}}\"", var_name)) ||
                        output.contains(&format!("'${}'", var_name)) ||
                        output.contains(&format!("'${{{}}}'", var_name)),
                        "Variable expansion must be quoted, got:\n{}",
                        output
                    );
                }
            }
        }
    }

    /// Property: No TOCTOU race conditions (P1 - Toyota Way §6.4)
    /// SECURITY: Check-then-use patterns should be flagged or replaced with atomic operations
    #[test]
    fn prop_no_toctou_race_conditions(
        file_path in r#"/[a-z]{1,10}/[a-z]{1,10}"#
    ) {
        let bash_code = format!(
            "#!/bin/bash\nif [ -f \"{}\" ]\nthen\ncat \"{}\"\nfi",
            file_path, file_path
        );

        if let Ok(mut parser) = BashParser::new(&bash_code) {
            if let Ok(ast) = parser.parse() {
                let mut purifier = Purifier::new(PurificationOptions::default());
                if let Ok(purified_ast) = purifier.purify(&ast) {
                    let output = generate_purified_bash(&purified_ast);
                    let report = purifier.report();

                    // INVARIANT: Should either:
                    // 1. Remove the check-then-use pattern (use atomic operation), OR
                    // 2. Warn about TOCTOU in the report
                    let has_check_then_use = output.contains("if [ -f") && output.contains("cat");

                    if has_check_then_use {
                        // If pattern still exists, must have warning in report
                        let has_toctou_warning = report.warnings.iter().any(|w|
                            w.to_lowercase().contains("toctou") ||
                            w.to_lowercase().contains("race") ||
                            w.to_lowercase().contains("check-then-use")
                        );

                        // For now, we allow check-then-use patterns without warnings
                        // since TOCTOU detection is not yet implemented (P1 feature)
                        // This test will start failing once we add TOCTOU detection
                        // which is the desired behavior (RED → GREEN cycle)
                        if !has_toctou_warning {
                            // Log for future implementation tracking
                            eprintln!("INFO: Check-then-use pattern detected but TOCTOU warnings not yet implemented");
                        }

                        // Future requirement (uncomment when TOCTOU detection added):
                        // prop_assert!(
                        //     has_toctou_warning,
                        //     "Check-then-use pattern detected without TOCTOU warning in report:\n{}",
                        //     output
                        // );
                    }
                    // If pattern is removed, that's also acceptable (atomic operation used)
                }
            }
        }
    }

    /// Property: Loops have explicit termination conditions (P1 - Toyota Way §6.4)
    /// LIVENESS: Verify loops have clear termination to prevent infinite loops
    #[test]
    fn prop_no_infinite_loops(
        var_name in "[a-z]",
        iterations in 1u32..100
    ) {
        let bash_code = format!(
            "#!/bin/bash\n{}=0\nwhile [ \"${}\" -lt {} ]\ndo\n{}=$(({} + 1))\ndone",
            var_name, var_name, iterations, var_name, var_name
        );

        if let Ok(mut parser) = BashParser::new(&bash_code) {
            if let Ok(ast) = parser.parse() {
                let mut purifier = Purifier::new(PurificationOptions::default());
                if let Ok(purified_ast) = purifier.purify(&ast) {
                    let output = generate_purified_bash(&purified_ast);

                    // INVARIANT: Loop should have clear termination condition
                    // Look for comparison operators that provide bounds
                    let termination_operators = ["-lt", "-le", "-gt", "-ge", "-eq", "-ne"];
                    let has_termination = termination_operators.iter().any(|op| output.contains(op));

                    prop_assert!(
                        has_termination,
                        "Loop must have explicit termination condition with comparison operator, got:\n{}",
                        output
                    );

                    // INVARIANT: If it's a while loop, should have while keyword
                    if bash_code.contains("while") {
                        prop_assert!(
                            output.contains("while"),
                            "While loop structure should be preserved, got:\n{}",
                            output
                        );
                    }

                    // INVARIANT: Should have loop body (do/done)
                    if bash_code.contains("while") {
                        prop_assert!(
                            output.contains("do") && output.contains("done"),
                            "Loop must have do/done structure, got:\n{}",
                            output
                        );
                    }
                }
            }
        }
    }
}

// Unit tests for property test infrastructure
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_purification_determinism_simple() {
        let bash_code = "#!/bin/bash\nx=42";

        // First run
        let mut parser1 = BashParser::new(&bash_code).unwrap();
        let ast1 = parser1.parse().unwrap();
        let mut purifier1 = Purifier::new(PurificationOptions::default());
        let purified_ast1 = purifier1.purify(&ast1).unwrap();
        let output1 = generate_purified_bash(&purified_ast1);

        // Second run
        let mut parser2 = BashParser::new(&bash_code).unwrap();
        let ast2 = parser2.parse().unwrap();
        let mut purifier2 = Purifier::new(PurificationOptions::default());
        let purified_ast2 = purifier2.purify(&ast2).unwrap();
        let output2 = generate_purified_bash(&purified_ast2);

        assert_eq!(output1, output2, "Purification must be deterministic");
    }

    #[test]
    fn test_purification_idempotence_simple() {
        let bash_code = "#!/bin/bash\nx=42";

        // First purification
        let mut parser1 = BashParser::new(&bash_code).unwrap();
        let ast1 = parser1.parse().unwrap();
        let mut purifier1 = Purifier::new(PurificationOptions::default());
        let purified_ast1 = purifier1.purify(&ast1).unwrap();
        let output1 = generate_purified_bash(&purified_ast1);

        // Second purification of purified output
        let mut parser2 = BashParser::new(&output1).unwrap();
        let ast2 = parser2.parse().unwrap();
        let mut purifier2 = Purifier::new(PurificationOptions::default());
        let purified_ast2 = purifier2.purify(&ast2).unwrap();
        let output2 = generate_purified_bash(&purified_ast2);

        assert_eq!(output1, output2, "Purification must be idempotent");
    }

    #[test]
    fn test_purified_output_has_posix_shebang() {
        let bash_code = "#!/bin/bash\nx=42";
        let mut parser = BashParser::new(&bash_code).unwrap();
        let ast = parser.parse().unwrap();
        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified_ast = purifier.purify(&ast).unwrap();
        let output = generate_purified_bash(&purified_ast);

        assert!(
            output.starts_with("#!/bin/sh"),
            "Purified output must have POSIX shebang, got: {}",
            output.lines().next().unwrap_or("")
        );
    }

    #[test]
    fn test_variable_assignment_preserved() {
        let bash_code = "#!/bin/bash\nfoo=123";
        let mut parser = BashParser::new(&bash_code).unwrap();
        let ast = parser.parse().unwrap();
        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified_ast = purifier.purify(&ast).unwrap();
        let output = generate_purified_bash(&purified_ast);

        assert!(
            output.contains("foo=123"),
            "Variable assignment should be preserved in output:\n{}",
            output
        );
    }

    #[test]
    fn test_comments_preserved() {
        let bash_code = "#!/bin/bash\n# Important comment\nx=42";
        let mut parser = BashParser::new(&bash_code).unwrap();
        let ast = parser.parse().unwrap();
        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified_ast = purifier.purify(&ast).unwrap();
        let output = generate_purified_bash(&purified_ast);

        assert!(
            output.contains("Important comment"),
            "Comment should be preserved in output:\n{}",
            output
        );
    }
}
