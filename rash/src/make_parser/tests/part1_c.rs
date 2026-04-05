#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

/// RED PHASE: Test for RULE-SYNTAX-001 - Basic rule syntax
///
/// This test validates the fundamental building block of Makefiles:
/// a target with prerequisites and a recipe.
///
/// Input Makefile:
/// ```makefile
/// target: prerequisites
///     recipe
/// ```
///
/// Expected AST:
/// - One MakeItem::Target
/// - name: "target"
/// - prerequisites: ["prerequisites"]
/// - recipe: ["recipe"]
/// - phony: false (will be detected/added in purification)
        #[test]
        fn test_VAR_BASIC_002_prop_var_refs_preserved_in_recipes(
            varname in "[A-Z][A-Z0-9_]{0,15}",
            recipe_prefix in "[a-z]{1,10}",
            recipe_suffix in "[a-z]{1,10}"
        ) {
            // ARRANGE: Recipe with variable reference
            let makefile = format!(
                "{}=value\ntarget:\n\t{} $({}) {}",
                varname, recipe_prefix, varname, recipe_suffix
            );

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Parsing succeeds
            prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 2);

            // ASSERT: Recipe preserves variable reference
            if let MakeItem::Target { recipe, .. } = &ast.items[1] {
                let var_ref = format!("$({})", varname);
                prop_assert!(
                    recipe[0].contains(&var_ref),
                    "Recipe should contain {}, got: {}",
                    var_ref,
                    recipe[0]
                );
            } else {
                return Err(TestCaseError::fail("Expected Target item"));
            }
        }

        /// Property: Variable references are preserved in variable values
        ///
        /// Verifies that $(VAR) in variable values is not expanded during parsing.
        #[test]
        fn test_VAR_BASIC_002_prop_var_refs_in_values(
            var1 in "[A-Z]{1,10}",
            var2 in "[A-Z]{1,10}",
            value_prefix in "[a-z]{0,10}",
            value_suffix in "[a-z]{0,10}"
        ) {
            // Ensure different variable names
            prop_assume!(var1 != var2);

            let makefile = format!(
                "{} = firstvalue\n{} = {}$({}){}",
                var1, var2, value_prefix, var1, value_suffix
            );

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            if let MakeItem::Variable { value, .. } = &ast.items[1] {
                let var_ref = format!("$({})", var1);
                prop_assert!(
                    value.contains(&var_ref),
                    "Variable value should contain {}, got: {}",
                    var_ref,
                    value
                );
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }

        /// Property: Curly brace syntax ${VAR} is also preserved
        ///
        /// Tests that both $(VAR) and ${VAR} syntaxes work.
        #[test]
        fn test_VAR_BASIC_002_prop_curly_brace_preserved(
            varname in "[A-Z]{1,10}",
            use_parens in prop::bool::ANY
        ) {
            let var_ref = if use_parens {
                format!("$({})", varname)
            } else {
                format!("${{{}}}", varname)
            };

            let makefile = format!(
                "{} = value\ntarget:\n\techo {}",
                varname, var_ref
            );

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            if let MakeItem::Target { recipe, .. } = &ast.items[1] {
                prop_assert!(
                    recipe[0].contains(&var_ref),
                    "Recipe should contain {}, got: {}",
                    var_ref,
                    recipe[0]
                );
            } else {
                return Err(TestCaseError::fail("Expected Target item"));
            }
        }

        /// Property: Multiple variable references in same line
        ///
        /// Verifies that multiple $(VAR) references are all preserved.
        #[test]
        fn test_VAR_BASIC_002_prop_multiple_refs_preserved(
            vars in prop::collection::vec("[A-Z]{1,8}", 2..5)
        ) {
            // Create variable definitions
            let var_defs: Vec<String> = vars.iter()
                .map(|v| format!("{} = value", v))
                .collect();

            // Create recipe with all variable references
            let var_refs: Vec<String> = vars.iter()
                .map(|v| format!("$({})", v))
                .collect();
            let recipe = var_refs.join(" ");

            let makefile = format!(
                "{}\ntarget:\n\t{}",
                var_defs.join("\n"),
                recipe
            );

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            // Find the target (last item)
            if let Some(MakeItem::Target { recipe: target_recipe, .. }) = ast.items.last() {
                // Verify all variable references are preserved
                for var in &vars {
                    let var_ref = format!("$({})", var);
                    prop_assert!(
                        target_recipe[0].contains(&var_ref),
                        "Recipe should contain {}, got: {}",
                        var_ref,
                        target_recipe[0]
                    );
                }
            } else {
                return Err(TestCaseError::fail("Expected Target as last item"));
            }
        }

        /// Property: Variable references in prerequisites are preserved
        ///
        /// Verifies that $(VAR) in target prerequisites is preserved.
        #[test]
        fn test_VAR_BASIC_002_prop_refs_in_prerequisites(
            varname in "[A-Z]{1,10}",
            target_name in "[a-z]{1,10}"
        ) {
            let makefile = format!(
                "{} = deps\n{}: $({}) file.o\n\techo done",
                varname, target_name, varname
            );

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            if let MakeItem::Target { prerequisites, .. } = &ast.items[1] {
                let var_ref = format!("$({})", varname);
                prop_assert!(
                    prerequisites[0].contains(&var_ref),
                    "Prerequisites should contain {}, got: {:?}",
                    var_ref,
                    prerequisites
                );
            } else {
                return Err(TestCaseError::fail("Expected Target item"));
            }
        }
    }
}

/// RED PHASE: Test for SYNTAX-001 - Comment parsing
///
/// This test validates that Makefile comments are parsed and included in the AST.
///
/// Input Makefile:
/// ```makefile
/// # This is a comment
/// target:
///     recipe
/// ```
///
/// Expected: Comment should be parsed as MakeItem::Comment
#[test]
fn test_SYNTAX_001_basic_comment() {
    // ARRANGE: Makefile with a comment
    let makefile = "# This is a comment\ntarget:\n\trecipe";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse comment, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();

    // ASSERT: Two items (comment + target)
    assert_eq!(ast.items.len(), 2, "Should have comment and target");

    // ASSERT: First item is a comment
    match &ast.items[0] {
        MakeItem::Comment { text, .. } => {
            assert_eq!(
                text, "This is a comment",
                "Comment text should be preserved"
            );
        }
        other => panic!("Expected Comment item, got {:?}", other),
    }

    // ASSERT: Second item is target
    match &ast.items[1] {
        MakeItem::Target { name, .. } => {
            assert_eq!(name, "target");
        }
        _ => panic!("Expected Target item"),
    }
}

/// RED PHASE: Test for SYNTAX-001 - Multiple comments
#[test]
fn test_SYNTAX_001_multiple_comments() {
    // ARRANGE: Makefile with multiple comments
    let makefile = "# Comment 1\n# Comment 2\ntarget:\n\trecipe";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: Three items (2 comments + target)
    assert_eq!(ast.items.len(), 3, "Should have 2 comments and target");

    // ASSERT: First two items are comments
    match &ast.items[0] {
        MakeItem::Comment { text, .. } => {
            assert_eq!(text, "Comment 1");
        }
        _ => panic!("Expected Comment item"),
    }

    match &ast.items[1] {
        MakeItem::Comment { text, .. } => {
            assert_eq!(text, "Comment 2");
        }
        _ => panic!("Expected Comment item"),
    }
}

/// RED PHASE: Test for SYNTAX-001 - Empty comment
#[test]
fn test_SYNTAX_001_empty_comment() {
    // ARRANGE: Comment with just #
    let makefile = "#\ntarget:\n\trecipe";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: Comment with empty text
    match &ast.items[0] {
        MakeItem::Comment { text, .. } => {
            assert_eq!(text, "", "Empty comment should have empty text");
        }
        _ => panic!("Expected Comment item"),
    }
}

/// RED PHASE: Test for SYNTAX-001 - Comment with leading/trailing spaces
#[test]
fn test_SYNTAX_001_comment_with_spaces() {
    // ARRANGE: Comment with spaces
    let makefile = "#   Comment with spaces   \ntarget:\n\trecipe";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: Comment text should be trimmed
    match &ast.items[0] {
        MakeItem::Comment { text, .. } => {
            assert_eq!(text, "Comment with spaces", "Comment should be trimmed");
        }
        _ => panic!("Expected Comment item"),
    }
}

// PROPERTY TESTING PHASE: Tests for SYNTAX-001
//
// These property tests verify comment parsing works across various inputs.
#[cfg(test)]
mod syntax_001_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Any line starting with # should be parsed as a comment
        ///
        /// This test generates 100+ random comment texts to ensure
        /// the parser handles a wide variety of comment content.
        #[test]
        fn test_SYNTAX_001_prop_any_hash_line_is_comment(
            comment_text in "[a-zA-Z0-9 ]{0,50}"
        ) {
            // ARRANGE: Create comment line
            let makefile = format!("# {}", comment_text);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Successfully parsed
            prop_assert!(result.is_ok(), "Failed to parse comment: {}", makefile);
            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            // ASSERT: First item is a comment
            match &ast.items[0] {
                MakeItem::Comment { text, .. } => {
                    prop_assert_eq!(text, comment_text.trim());
                }
                _ => return Err(TestCaseError::fail("Expected Comment item")),
            }
        }

        /// Property: Multiple comments should all be preserved
        ///
        /// Verifies that N consecutive comments all parse correctly.
        #[test]
        fn test_SYNTAX_001_prop_multiple_comments_preserved(
            count in 1..10usize
        ) {
            // ARRANGE: Create multiple comment lines
            let lines: Vec<String> = (0..count)
                .map(|i| format!("# Comment {}", i))
                .collect();
            let makefile = lines.join("\n");

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Successfully parsed with correct count
            prop_assert!(result.is_ok());
            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), count);

            // ASSERT: All items are comments
            for (i, item) in ast.items.iter().enumerate() {
                match item {
                    MakeItem::Comment { text, .. } => {
                        prop_assert_eq!(text, &format!("Comment {}", i));
                    }
                    _ => return Err(TestCaseError::fail("Expected Comment item")),
                }
            }
        }

        /// Property: Comments with special characters should be preserved
        ///
        /// Verifies that comments can contain special shell/makefile characters.
        #[test]
        fn test_SYNTAX_001_prop_special_chars_preserved(
            special_chars in "[!@$%^&*()+=\\[\\]{}|;:,.<>?/~`-]{1,20}"
        ) {
            // ARRANGE: Comment with special characters
            let makefile = format!("# {}", special_chars);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Successfully parsed
            prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);
            let ast = result.unwrap();

            // ASSERT: Special characters preserved
            match &ast.items[0] {
                MakeItem::Comment { text, .. } => {
                    prop_assert_eq!(text, special_chars.trim());
                }
                _ => return Err(TestCaseError::fail("Expected Comment item")),
            }
        }

        /// Property: Comments mixed with targets should parse correctly
        ///
        /// Verifies that comments interspersed with targets are all captured.
        #[test]
        fn test_SYNTAX_001_prop_comments_with_targets(
            target_count in 1..5usize
        ) {
            // ARRANGE: Alternating comments and targets
            let mut lines = Vec::new();
            for i in 0..target_count {
                lines.push(format!("# Comment for target {}", i));
                lines.push(format!("target{}:", i));
                lines.push("\techo 'test'".to_string());
            }
            let makefile = lines.join("\n");

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Successfully parsed
            prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);
            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), target_count * 2);

            // ASSERT: Comments and targets alternate
            for i in 0..target_count {
                let comment_idx = i * 2;
                let target_idx = comment_idx + 1;

                match &ast.items[comment_idx] {
                    MakeItem::Comment { .. } => {},
                    _ => return Err(TestCaseError::fail("Expected Comment")),
                }

                match &ast.items[target_idx] {
                    MakeItem::Target { .. } => {},
                    _ => return Err(TestCaseError::fail("Expected Target")),
                }
            }
        }

        /// Property: Empty comments (just #) should parse with empty text
        ///
        /// Verifies that # with only whitespace produces empty comment text.
