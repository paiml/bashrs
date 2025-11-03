//! Property-Based Tests for Bash Purification
//!
//! EXTREME TDD: Property tests verify purification invariants
//! - Determinism: Same input → same output
//! - Idempotency: purify(purify(x)) == purify(x)
//! - Safety: No dangerous patterns in output
//! - Correctness: Output is valid POSIX sh

use super::purification::{PurificationOptions, Purifier};
use crate::bash_parser::ast::*;
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
