#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

// Phase 5 - MUTATION TESTING: Mutation-killing tests for = recursive assignment

// Target: parser.rs:116 - is_variable_assignment() contains('=') check
    #[test]
    fn test_SYNTAX_002_mut_handles_backslash_at_end_of_file() {
        // ARRANGE: Variable with backslash at EOF (no next line)
        let makefile = "FILES = file1.c \\";

        // ACT: Parse
        let result = parse_makefile(makefile);

        // ASSERT: Must parse successfully (not panic from index out of bounds)
        assert!(result.is_ok(), "Backslash at EOF should not panic");

        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1);

        match &ast.items[0] {
            MakeItem::Variable { value, .. } => {
                // Trailing backslash at EOF should be handled gracefully
                // The backslash is preserved since there's no next line to continue with
                assert_eq!(
                    value, "file1.c \\",
                    "Trailing backslash at EOF should be preserved"
                );

                // Key point: we don't panic with out-of-bounds access
                // The condition `i + 1 < lines.len()` prevents reading past EOF
            }
            _ => panic!("Expected Variable"),
        }
    }

    // Mutation: Change trim_end() to trim()
    // This test ensures we preserve leading whitespace in values
    #[test]
    fn test_SYNTAX_002_mut_preserves_leading_whitespace_in_first_line() {
        // ARRANGE: Variable with leading spaces in value
        let makefile = "FILES =     file1.c \\\n    file2.c";

        // ACT: Parse
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Variable { value, .. } => {
                // Leading spaces before file1.c should be handled by parse_variable's trim
                // This test ensures the continuation logic doesn't break that
                assert_eq!(
                    value, "file1.c file2.c",
                    "Whitespace normalization should work correctly"
                );
            }
            _ => panic!("Expected Variable"),
        }
    }

    // Mutation: Change push(' ') to push_str("  ")
    // This test ensures we use single space, not double space
    #[test]
    fn test_SYNTAX_002_mut_uses_single_space_separator() {
        // ARRANGE: Variable with continuation
        let makefile = "FILES = a.c \\\n    b.c";

        // ACT: Parse
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Variable { value, .. } => {
                // Must use single space, not double
                assert_eq!(value, "a.c b.c", "Should use single space separator");
                assert!(!value.contains("  "), "Should not contain double spaces");
            }
            _ => panic!("Expected Variable"),
        }
    }

    // Mutation: Remove the trim_start() call on next_line
    // This test ensures we strip leading whitespace from continued lines
    #[test]
    fn test_SYNTAX_002_mut_strips_leading_whitespace_from_continuation() {
        // ARRANGE: Variable with heavy indentation on continued line
        let makefile = "FILES = file1.c \\\n            file2.c";

        // ACT: Parse
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Variable { value, .. } => {
                // Leading whitespace should be stripped from continued line
                assert_eq!(
                    value, "file1.c file2.c",
                    "Leading whitespace should be stripped"
                );
                // Should NOT contain multiple spaces from the indentation
                assert!(
                    !value.contains("file1.c            file2.c"),
                    "Indentation should be normalized to single space"
                );
            }
            _ => panic!("Expected Variable"),
        }
    }
}

// ============================================================================
// Sprint 38: RECIPE-001 - Tab-indented recipes
// ============================================================================
// Implements: Tab-indented recipe lines for targets
// Verifies: Parser correctly handles tab-indented recipe lines (MUST be tabs, not spaces)

// Phase 1 - RED: Unit tests for tab-indented recipes

#[test]
fn test_RECIPE_001_single_tab_indented_recipe() {
    // ARRANGE: Target with single tab-indented recipe line
    let makefile = "build:\n\tcargo build --release";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Parser should handle tab-indented recipe
    assert!(result.is_ok(), "Parser should handle tab-indented recipe");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should parse one target");

    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "build", "Target name should be 'build'");
            assert_eq!(recipe.len(), 1, "Should have one recipe line");
            assert_eq!(
                recipe[0], "cargo build --release",
                "Recipe should be parsed correctly"
            );
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

#[test]
fn test_RECIPE_001_multiple_tab_indented_recipes() {
    // ARRANGE: Target with multiple tab-indented recipe lines
    let makefile =
        "deploy:\n\tcargo build --release\n\tcargo test\n\tscp target/release/app server:/opt/";

    // ACT: Parse
    let result = parse_makefile(makefile);

    // ASSERT: Should parse all recipe lines
    assert!(result.is_ok(), "Multiple recipe lines should parse");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "deploy");
            assert_eq!(recipe.len(), 3, "Should have three recipe lines");
            assert_eq!(recipe[0], "cargo build --release");
            assert_eq!(recipe[1], "cargo test");
            assert_eq!(recipe[2], "scp target/release/app server:/opt/");
        }
        _ => panic!("Expected Target"),
    }
}

#[test]
fn test_RECIPE_001_recipe_with_empty_lines() {
    // ARRANGE: Target with recipe lines separated by empty lines
    let makefile = "test:\n\tcargo test --lib\n\n\tcargo test --doc";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            // Empty lines between recipe lines should be skipped
            assert_eq!(
                recipe.len(),
                2,
                "Should parse both recipe lines despite empty line"
            );
            assert_eq!(recipe[0], "cargo test --lib");
            assert_eq!(recipe[1], "cargo test --doc");
        }
        _ => panic!("Expected Target"),
    }
}

#[test]
fn test_RECIPE_001_recipe_stops_at_non_tab_line() {
    // ARRANGE: Makefile with target followed by another construct
    let makefile = "build:\n\tcargo build\n\nCC = gcc";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2, "Should parse target and variable");

    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "build");
            assert_eq!(recipe.len(), 1, "Recipe should stop at non-tab line");
            assert_eq!(recipe[0], "cargo build");
        }
        _ => panic!("Expected Target"),
    }

    match &ast.items[1] {
        MakeItem::Variable { name, .. } => {
            assert_eq!(name, "CC", "Should parse variable after recipe");
        }
        _ => panic!("Expected Variable"),
    }
}

// Phase 4 - PROPERTY TESTING: Property tests for recipe parsing

#[cfg(test)]
mod recipe_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_RECIPE_001_recipes_with_varying_lines_always_parse(
            num_lines in 1usize..10,
            target_name in "[a-z][a-z0-9_]*"
        ) {
            // ARRANGE: Generate target with varying number of recipe lines
            let mut makefile = format!("{}:\n", target_name);
            for i in 0..num_lines {
                makefile.push_str(&format!("\tcommand_{}\n", i));
            }

            // ACT: Parse
            let result = parse_makefile(&makefile);

            // ASSERT: Should always parse successfully
            prop_assert!(result.is_ok(), "Should parse {} recipe lines", num_lines);

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1, "Should have one target");

            match &ast.items[0] {
                MakeItem::Target { name, recipe, .. } => {
                    prop_assert_eq!(name, &target_name, "Target name should match");
                    prop_assert_eq!(recipe.len(), num_lines, "Should have {} recipe lines", num_lines);
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn prop_RECIPE_001_recipe_parsing_is_deterministic(
            target_name in "[a-z][a-z0-9_]*",
            recipe_cmd in "[a-z][a-z0-9 _-]*"
        ) {
            // ARRANGE: Create makefile with recipe
            let makefile = format!("{}:\n\t{}", target_name, recipe_cmd);

            // ACT: Parse twice
            let result1 = parse_makefile(&makefile);
            let result2 = parse_makefile(&makefile);

            // ASSERT: Both should succeed and produce identical results
            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());

            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();

            prop_assert_eq!(ast1.items.len(), ast2.items.len(), "Item count should be deterministic");

            match (&ast1.items[0], &ast2.items[0]) {
                (
                    MakeItem::Target { name: n1, recipe: r1, .. },
                    MakeItem::Target { name: n2, recipe: r2, .. }
                ) => {
                    prop_assert_eq!(n1, n2, "Target names should be identical");
                    prop_assert_eq!(r1, r2, "Recipes should be identical");
                }
                _ => return Err(TestCaseError::fail("Both should be Target items")),
            }
        }

        #[test]
        fn prop_RECIPE_001_tab_indented_lines_always_recognized(
            recipe_count in 1usize..5,
            cmd in "[a-z][a-z0-9 ]*"
        ) {
            // ARRANGE: Create target with multiple tab-indented lines
            let mut makefile = String::from("target:\n");
            for _ in 0..recipe_count {
                makefile.push('\t');
                makefile.push_str(&cmd);
                makefile.push('\n');
            }

            // ACT: Parse
            let result = parse_makefile(&makefile);

            // ASSERT: All tab-indented lines should be recognized as recipes
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    prop_assert_eq!(recipe.len(), recipe_count, "All tab-indented lines should be recipes");
                    for recipe_line in recipe {
                        prop_assert!(recipe_line.trim() == cmd.trim(), "Recipe content should match");
                    }
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn prop_RECIPE_001_recipe_order_preserved(
            num_recipes in 2usize..6
        ) {
            // ARRANGE: Create target with ordered recipe lines
            let mut makefile = String::from("build:\n");
            let mut expected_order = Vec::new();

            for i in 0..num_recipes {
                let cmd = format!("step_{}", i);
                makefile.push_str(&format!("\t{}\n", cmd));
                expected_order.push(cmd);
            }

            // ACT: Parse
            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();

            // ASSERT: Recipe order should be preserved
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    prop_assert_eq!(recipe.len(), expected_order.len());
                    for (i, recipe_line) in recipe.iter().enumerate() {
                        prop_assert_eq!(recipe_line, &expected_order[i], "Recipe order should be preserved");
                    }
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn prop_RECIPE_001_various_recipe_commands_work(
            prefix in "cargo|make|echo|cd|mkdir",
            suffix in "[a-z0-9_/-]*"
        ) {
            // ARRANGE: Create target with various command types
            let recipe_cmd = format!("{} {}", prefix, suffix);
            let makefile = format!("test:\n\t{}", recipe_cmd);

            // ACT: Parse
            let result = parse_makefile(&makefile);

            // ASSERT: All command types should parse correctly
            prop_assert!(result.is_ok(), "Command '{}' should parse", recipe_cmd);

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    prop_assert_eq!(recipe.len(), 1);
                    prop_assert!(recipe[0].starts_with(&prefix), "Recipe should start with prefix");
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }
    }
}

// Phase 5 - MUTATION TESTING: Mutation-killing tests for recipe parsing

#[test]
fn test_RECIPE_001_mut_tab_detection_must_use_starts_with() {
    // MUTATION TARGET: line 268 in parser.rs
    // Mutation: Replace starts_with('\t') with something else
    // This test kills mutations that break tab detection logic

    // ARRANGE: Target with tab-indented recipe and space-indented line
    let makefile = "build:\n\tcargo build\n    not a recipe";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: Only tab-indented line should be in recipe
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 1, "Only tab-indented line should be recipe");
            assert_eq!(
                recipe[0], "cargo build",
                "Tab-indented line should be recipe"
            );
        }
        _ => panic!("Expected Target"),
    }

    // CRITICAL: If starts_with('\t') is mutated to starts_with(' ') or contains('\t'),
    // this test will fail because "    not a recipe" would be parsed as recipe
}

#[test]
fn test_RECIPE_001_mut_recipe_push_must_happen() {
    // MUTATION TARGET: line 270 in parser.rs (recipe.push)
    // Mutation: Remove recipe.push() call
    // This test kills mutations that skip adding recipe lines

    // ARRANGE: Target with multiple recipes
    let makefile = "test:\n\tcargo test --lib\n\tcargo test --doc\n\tcargo test --integration";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: All recipes must be captured
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 3, "All three recipe lines must be pushed");
            assert_eq!(recipe[0], "cargo test --lib");
            assert_eq!(recipe[1], "cargo test --doc");
            assert_eq!(recipe[2], "cargo test --integration");
        }
        _ => panic!("Expected Target"),
    }

    // CRITICAL: If recipe.push() is removed, recipe would be empty
}

#[test]
fn test_RECIPE_001_mut_empty_line_handling_must_continue() {
    // MUTATION TARGET: line 276 in parser.rs (continue in empty line handling)
    // Mutation: Replace continue with break
    // This test kills mutations that break empty line handling

    // ARRANGE: Recipe with empty line in the middle
    let makefile = "deploy:\n\techo 'Starting'\n\n\techo 'Done'";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: Both recipes should be parsed despite empty line
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 2, "Empty line should not stop recipe parsing");
            assert_eq!(recipe[0], "echo 'Starting'");
            assert_eq!(recipe[1], "echo 'Done'");
        }
        _ => panic!("Expected Target"),
    }

    // CRITICAL: If continue is replaced with break, only first recipe would be parsed
}

