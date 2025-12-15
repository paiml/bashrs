#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
#![allow(clippy::expect_used)]
//! Property-based tests for Makefile parser
//!
//! These tests use proptest to generate thousands of test cases automatically,
//! discovering edge cases and verifying parser invariants.
//!
//! Following SQLite testing principles: exhaustive edge case discovery

use bashrs::make_parser::{parse_makefile, MakeItem};
use proptest::prelude::*;

// ============================================================================
// Property Test Generators
// ============================================================================

/// Generate valid Makefile target names
fn valid_target_name() -> impl Strategy<Value = String> {
    "[a-zA-Z_][a-zA-Z0-9_.-]{0,63}"
}

/// Generate valid Makefile variable names
fn valid_variable_name() -> impl Strategy<Value = String> {
    "[A-Z_][A-Z0-9_]{0,31}"
}

/// Generate any string (including invalid Makefiles)
fn any_string() -> impl Strategy<Value = String> {
    ".*"
}

// ============================================================================
// Parser Robustness Properties
// ============================================================================

proptest! {
    /// PROPERTY: Parser must always terminate (no infinite loops)
    ///
    /// This is the most critical safety property. The parser must handle
    /// ANY input without hanging, even malformed or malicious input.
    ///
    /// SQLite principle: Parser must be bulletproof
    #[test]
    fn prop_parser_always_terminates(input in any_string()) {
        // Parser must complete in reasonable time for any input
        let _ = parse_makefile(&input);
        // If we get here, parser terminated ✅
    }

    /// PROPERTY: Parser must never panic
    ///
    /// Even with invalid input, parser should return Result::Err
    /// rather than panicking.
    #[test]
    fn prop_parser_never_panics(input in any_string()) {
        let result = std::panic::catch_unwind(|| {
            let _ = parse_makefile(&input);
        });
        assert!(result.is_ok(), "Parser panicked on input");
    }

    /// PROPERTY: Parsing is deterministic
    ///
    /// Same input must always produce same output.
    /// This is essential for reproducible builds.
    #[test]
    fn prop_parsing_deterministic(
        target in valid_target_name(),
        recipe in "[a-z ]{5,20}"
    ) {
        let makefile = format!("{}:\n\t{}", target, recipe);

        let ast1 = parse_makefile(&makefile);
        let ast2 = parse_makefile(&makefile);

        // Same input = same result
        match (ast1, ast2) {
            (Ok(a1), Ok(a2)) => {
                assert_eq!(a1.items.len(), a2.items.len());
                // Note: Full equality would require implementing Eq on MakeItem
            }
            (Err(_), Err(_)) => {}, // Both failed identically ✅
            _ => panic!("Non-deterministic parsing!"),
        }
    }
}

// ============================================================================
// Target Parsing Properties
// ============================================================================

proptest! {
    /// PROPERTY: Valid targets always parse successfully
    #[test]
    fn prop_valid_targets_always_parse(
        target in valid_target_name(),
        recipe in "[a-z ]{5,20}"
    ) {
        let makefile = format!("{}:\n\t{}", target, recipe);
        let result = parse_makefile(&makefile);

        assert!(result.is_ok(), "Valid target failed to parse: {:?}", result);

        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1);
        assert!(matches!(ast.items[0], MakeItem::Target { .. }));
    }

    /// PROPERTY: Target names are preserved exactly
    #[test]
    fn prop_target_names_preserved(
        target in valid_target_name()
    ) {
        let makefile = format!("{}:\n\techo done", target);
        let ast = parse_makefile(&makefile).unwrap();

        match &ast.items[0] {
            MakeItem::Target { name, .. } => {
                assert_eq!(name, &target);
            }
            _ => panic!("Expected Target"),
        }
    }

    /// PROPERTY: Multiple targets are independent
    #[test]
    fn prop_multiple_targets_independent(
        targets in prop::collection::vec(valid_target_name(), 1..10)
    ) {
        let makefile = targets.iter()
            .map(|t| format!("{}:\n\techo {}", t, t))
            .collect::<Vec<_>>()
            .join("\n\n");

        let ast = parse_makefile(&makefile).unwrap();

        // Each target should be parsed independently
        let target_count = ast.items.iter()
            .filter(|item| matches!(item, MakeItem::Target { .. }))
            .count();

        assert_eq!(target_count, targets.len());
    }
}

// ============================================================================
// Variable Parsing Properties
// ============================================================================

proptest! {
    /// PROPERTY: Variable assignments preserve name and value
    #[test]
    fn prop_variables_preserved(
        name in valid_variable_name(),
        value in "[a-z0-9/ ]{1,50}"
    ) {
        let makefile = format!("{} = {}", name, value);
        let ast = parse_makefile(&makefile).unwrap();

        match &ast.items[0] {
            MakeItem::Variable { name: parsed_name, value: parsed_value, .. } => {
                assert_eq!(parsed_name, &name);
                assert_eq!(parsed_value, &value.trim());
            }
            _ => panic!("Expected Variable"),
        }
    }

    /// PROPERTY: All 5 variable flavors are parsed correctly
    #[test]
    fn prop_all_variable_flavors_parse(
        name in valid_variable_name(),
        value in "[a-z]{3,10}",
        flavor in prop::sample::select(vec!["=", ":=", "?=", "+=", "!="])
    ) {
        let makefile = format!("{} {} {}", name, flavor, value);
        let result = parse_makefile(&makefile);

        assert!(result.is_ok(), "Variable with {} failed to parse", flavor);

        let ast = result.unwrap();
        assert!(matches!(ast.items[0], MakeItem::Variable { .. }));
    }
}

// ============================================================================
// Comment Parsing Properties
// ============================================================================

proptest! {
    /// PROPERTY: Comments are always parsed
    #[test]
    fn prop_comments_always_parse(
        comment in "[a-zA-Z0-9 ]{0,80}"
    ) {
        let makefile = format!("# {}", comment);
        let ast = parse_makefile(&makefile).unwrap();

        assert_eq!(ast.items.len(), 1);
        assert!(matches!(ast.items[0], MakeItem::Comment { .. }));
    }

    /// PROPERTY: Comment text is preserved (trimmed)
    #[test]
    fn prop_comment_text_preserved(
        comment in "[a-zA-Z0-9 ]{1,80}"
    ) {
        let makefile = format!("#  {}  ", comment);
        let ast = parse_makefile(&makefile).unwrap();

        match &ast.items[0] {
            MakeItem::Comment { text, .. } => {
                assert_eq!(text, &comment.trim());
            }
            _ => panic!("Expected Comment"),
        }
    }
}

// ============================================================================
// Recipe Parsing Properties
// ============================================================================

proptest! {
    /// PROPERTY: Recipes preserve leading @ prefix
    #[test]
    fn prop_at_prefix_preserved(
        target in valid_target_name(),
        cmd in "[a-z ]{3,20}"
    ) {
        let makefile = format!("{}:\n\t@{}", target, cmd);
        let ast = parse_makefile(&makefile).unwrap();

        match &ast.items[0] {
            MakeItem::Target { recipe, .. } => {
                assert_eq!(recipe.len(), 1);
                assert!(recipe[0].starts_with('@'));
            }
            _ => panic!("Expected Target"),
        }
    }

    /// PROPERTY: Recipe order is preserved
    #[test]
    fn prop_recipe_order_preserved(
        target in valid_target_name(),
        recipes in prop::collection::vec("[a-z]{3,10}", 1..10)
    ) {
        let recipe_lines = recipes.iter()
            .map(|r| format!("\t{}", r))
            .collect::<Vec<_>>()
            .join("\n");

        let makefile = format!("{}:\n{}", target, recipe_lines);
        let ast = parse_makefile(&makefile).unwrap();

        match &ast.items[0] {
            MakeItem::Target { recipe, .. } => {
                assert_eq!(recipe.len(), recipes.len());

                for (i, expected) in recipes.iter().enumerate() {
                    assert_eq!(&recipe[i], expected);
                }
            }
            _ => panic!("Expected Target"),
        }
    }
}

// ============================================================================
// Prerequisite Parsing Properties
// ============================================================================

proptest! {
    /// PROPERTY: Prerequisites are parsed in order
    #[test]
    fn prop_prerequisites_order_preserved(
        target in valid_target_name(),
        prereqs in prop::collection::vec(valid_target_name(), 0..10)
    ) {
        let prereq_str = prereqs.join(" ");
        let makefile = format!("{}: {}\n\techo done", target, prereq_str);

        let ast = parse_makefile(&makefile).unwrap();

        match &ast.items[0] {
            MakeItem::Target { prerequisites, .. } => {
                assert_eq!(prerequisites.len(), prereqs.len());

                for (i, expected) in prereqs.iter().enumerate() {
                    assert_eq!(&prerequisites[i], expected);
                }
            }
            _ => panic!("Expected Target"),
        }
    }

    /// PROPERTY: Whitespace in prerequisites is normalized
    #[test]
    fn prop_prerequisites_whitespace_normalized(
        target in valid_target_name(),
        prereq1 in valid_target_name(),
        prereq2 in valid_target_name(),
        spaces in 1usize..10
    ) {
        let space_str = " ".repeat(spaces);
        let makefile = format!("{}: {}{}{}\n\techo done",
            target, prereq1, space_str, prereq2);

        let ast = parse_makefile(&makefile).unwrap();

        match &ast.items[0] {
            MakeItem::Target { prerequisites, .. } => {
                assert_eq!(prerequisites.len(), 2);
                assert_eq!(&prerequisites[0], &prereq1);
                assert_eq!(&prerequisites[1], &prereq2);
            }
            _ => panic!("Expected Target"),
        }
    }
}

// ============================================================================
// Empty and Edge Case Properties
// ============================================================================

proptest! {
    /// PROPERTY: Empty input is valid (produces empty AST)
    #[test]
    fn prop_empty_input_valid(_unit in prop::bool::ANY) {
        let ast = parse_makefile("").unwrap();
        assert_eq!(ast.items.len(), 0);
    }

    /// PROPERTY: Whitespace-only input is valid
    #[test]
    fn prop_whitespace_only_valid(
        spaces in prop::collection::vec(" \t\n", 0..20)
    ) {
        let whitespace = spaces.join("");
        let ast = parse_makefile(&whitespace).unwrap();
        assert_eq!(ast.items.len(), 0);
    }

    /// PROPERTY: Targets with no recipes are valid
    #[test]
    fn prop_empty_recipe_valid(target in valid_target_name()) {
        let makefile = format!("{}:", target);
        let ast = parse_makefile(&makefile).unwrap();

        match &ast.items[0] {
            MakeItem::Target { recipe, .. } => {
                assert_eq!(recipe.len(), 0);
            }
            _ => panic!("Expected Target"),
        }
    }

    /// PROPERTY: Variables with empty values are valid
    #[test]
    fn prop_empty_variable_valid(name in valid_variable_name()) {
        let makefile = format!("{} =", name);
        let ast = parse_makefile(&makefile).unwrap();

        match &ast.items[0] {
            MakeItem::Variable { value, .. } => {
                assert_eq!(value, "");
            }
            _ => panic!("Expected Variable"),
        }
    }
}

// ============================================================================
// Mixed Content Properties
// ============================================================================

proptest! {
    /// PROPERTY: Mixed targets, variables, and comments parse correctly
    #[test]
    fn prop_mixed_content_parses(
        target in valid_target_name(),
        var_name in valid_variable_name(),
        var_value in "[a-z]{3,10}",
        comment in "[a-z ]{5,20}"
    ) {
        let makefile = format!(
            "# {}\n{} = {}\n{}:\n\techo done",
            comment, var_name, var_value, target
        );

        let ast = parse_makefile(&makefile).unwrap();

        assert_eq!(ast.items.len(), 3);
        assert!(matches!(ast.items[0], MakeItem::Comment { .. }));
        assert!(matches!(ast.items[1], MakeItem::Variable { .. }));
        assert!(matches!(ast.items[2], MakeItem::Target { .. }));
    }
}
