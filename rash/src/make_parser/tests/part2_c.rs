#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

// Phase 5 - MUTATION TESTING: Mutation-killing tests for = recursive assignment

// Target: parser.rs:116 - is_variable_assignment() contains('=') check
#[test]
fn test_RECIPE_002_mut_multiple_targets_isolated() {
    // MUTATION TARGET: General recipe parsing logic
    // Mutation: Share recipe Vec between targets or don't reset
    // This test kills mutations that leak recipe lines between targets

    // ARRANGE: Two targets with different recipe counts
    let makefile = "first:\n\tA\n\tB\n\nsecond:\n\tX";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2);

    // ASSERT: First target has 2 lines, second has 1 (must be isolated)
    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "first");
            assert_eq!(recipe.len(), 2, "First target MUST have 2 lines");
            assert_eq!(recipe[0], "A");
            assert_eq!(recipe[1], "B");
        }
        _ => panic!("Expected first Target"),
    }

    match &ast.items[1] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "second");
            assert_eq!(recipe.len(), 1, "Second target MUST have 1 line");
            assert_eq!(recipe[0], "X");
            // CRITICAL: If recipes leak, this would have [A, B, X] (3 lines)
        }
        _ => panic!("Expected second Target"),
    }

    // CRITICAL: If recipe Vec isn't fresh per target, isolation breaks
}

// ============================================================================
// Sprint 40: ECHO-001 - @ prefix for silent recipes
// ============================================================================
// Implements: @ prefix in recipes to suppress command echoing
// Verifies: Parser preserves @ prefix in recipe lines

// Phase 1 - RED: Unit tests for @ prefix in recipes

#[test]
fn test_ECHO_001_single_silent_recipe() {
    // ARRANGE: Target with single @ prefix recipe
    let makefile = "test:\n\t@cargo test";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: @ prefix should be preserved in recipe
    assert!(result.is_ok(), "Parser should handle @ prefix in recipes");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should parse one target");

    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "test", "Target name should be 'test'");
            assert_eq!(recipe.len(), 1, "Should have one recipe line");
            assert_eq!(recipe[0], "@cargo test", "@ prefix must be preserved");
            assert!(recipe[0].starts_with('@'), "Recipe must start with @");
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

#[test]
fn test_ECHO_001_multiple_silent_recipes() {
    // ARRANGE: Target with multiple @ prefix recipes
    let makefile = "build:\n\t@echo 'Building...'\n\t@cargo build --release\n\t@echo 'Done'";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Multiple @ prefix recipes should parse");

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "build");
            assert_eq!(recipe.len(), 3, "Should have three recipe lines");
            assert_eq!(recipe[0], "@echo 'Building...'", "First @ prefix preserved");
            assert_eq!(
                recipe[1], "@cargo build --release",
                "Second @ prefix preserved"
            );
            assert_eq!(recipe[2], "@echo 'Done'", "Third @ prefix preserved");

            // All three lines should start with @
            for (i, line) in recipe.iter().enumerate() {
                assert!(line.starts_with('@'), "Recipe line {} must start with @", i);
            }
        }
        _ => panic!("Expected Target"),
    }
}

#[test]
fn test_ECHO_001_mixed_silent_and_normal_recipes() {
    // ARRANGE: Target with mix of @ prefix and normal recipes
    let makefile = "deploy:\n\tcargo build\n\t@echo 'Deploying...'\n\tscp app server:/opt/\n\t@echo 'Complete'";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 4, "Should have four recipe lines");
            assert_eq!(recipe[0], "cargo build", "Normal recipe (no @)");
            assert_eq!(recipe[1], "@echo 'Deploying...'", "Silent recipe (with @)");
            assert_eq!(recipe[2], "scp app server:/opt/", "Normal recipe (no @)");
            assert_eq!(recipe[3], "@echo 'Complete'", "Silent recipe (with @)");

            // Verify @ only on specific lines
            assert!(!recipe[0].starts_with('@'), "Line 0 should not have @");
            assert!(recipe[1].starts_with('@'), "Line 1 should have @");
            assert!(!recipe[2].starts_with('@'), "Line 2 should not have @");
            assert!(recipe[3].starts_with('@'), "Line 3 should have @");
        }
        _ => panic!("Expected Target"),
    }
}

#[test]
fn test_ECHO_001_at_prefix_different_targets() {
    // ARRANGE: Multiple targets, some with @ prefix, some without
    let makefile = "test:\n\t@cargo test\n\nverbose:\n\tcargo test --verbose";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2, "Should have two targets");

    // First target: with @
    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "test");
            assert_eq!(recipe.len(), 1);
            assert_eq!(recipe[0], "@cargo test");
            assert!(recipe[0].starts_with('@'));
        }
        _ => panic!("Expected first Target"),
    }

    // Second target: without @
    match &ast.items[1] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "verbose");
            assert_eq!(recipe.len(), 1);
            assert_eq!(recipe[0], "cargo test --verbose");
            assert!(
                !recipe[0].starts_with('@'),
                "verbose target should NOT have @"
            );
        }
        _ => panic!("Expected second Target"),
    }
}

// Phase 4 - PROPERTY TESTING: Property tests for @ prefix

#[cfg(test)]
mod echo_001_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_ECHO_001_at_prefix_always_preserved(
            cmd in "[a-z]{3,10}",
            args in prop::option::of("[a-z0-9 ]{0,20}")
        ) {
            // PROPERTY: @ prefix in recipes is always preserved
            let recipe_cmd = match args {
                Some(a) if !a.trim().is_empty() => format!("@{} {}", cmd, a.trim()),
                _ => format!("@{}", cmd),
            };
            let makefile = format!("target:\n\t{}", recipe_cmd);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok(), "Parser must handle @ prefix");

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    prop_assert_eq!(recipe.len(), 1);
                    prop_assert_eq!(&recipe[0], &recipe_cmd);
                    prop_assert!(recipe[0].starts_with('@'), "@ prefix must be preserved");
                }
                _ => prop_assert!(false, "Expected Target"),
            }
        }

        #[test]
        fn prop_ECHO_001_parsing_is_deterministic(
            silent_cmd in "@[a-z]{3,10}",
            normal_cmd in "[a-z]{3,10}"
        ) {
            // PROPERTY: Parsing @ prefix recipes is deterministic
            let makefile = format!("target:\n\t{}\n\t{}", silent_cmd, normal_cmd);

            let result1 = parse_makefile(&makefile);
            let result2 = parse_makefile(&makefile);

            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());

            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();

            // Same input = same AST
            prop_assert_eq!(ast1.items.len(), ast2.items.len());

            match (&ast1.items[0], &ast2.items[0]) {
                (MakeItem::Target { recipe: r1, .. }, MakeItem::Target { recipe: r2, .. }) => {
                    prop_assert_eq!(r1.len(), r2.len());
                    prop_assert_eq!(&r1[0], &r2[0]);
                    prop_assert_eq!(&r1[1], &r2[1]);
                }
                _ => prop_assert!(false, "Expected Target"),
            }
        }

        #[test]
        fn prop_ECHO_001_mixed_recipes_order_preserved(
            n_silent in 1usize..5,
            n_normal in 1usize..5
        ) {
            // PROPERTY: Order of @ and non-@ recipes is preserved
            let mut makefile = String::from("target:");

            let mut expected_recipes = Vec::new();
            for i in 0..n_silent {
                let recipe = format!("@silent{}", i);
                makefile.push_str(&format!("\n\t{}", recipe));
                expected_recipes.push(recipe);
            }
            for i in 0..n_normal {
                let recipe = format!("normal{}", i);
                makefile.push_str(&format!("\n\t{}", recipe));
                expected_recipes.push(recipe);
            }

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    prop_assert_eq!(recipe.len(), expected_recipes.len());
                    for (i, expected) in expected_recipes.iter().enumerate() {
                        prop_assert_eq!(&recipe[i], expected, "Recipe at index {} must match", i);
                    }
                }
                _ => prop_assert!(false, "Expected Target"),
            }
        }

        #[test]
        fn prop_ECHO_001_at_prefix_with_special_chars(
            prefix in "@+",  // @ or @@
            cmd in "[a-z]{3,8}",
            special in prop::option::of(r"[!$(){}\\|&;<>]")
        ) {
            // PROPERTY: @ prefix preserved even with special shell chars
            let recipe_line = match special {
                Some(s) => format!("{}{} {}", prefix, cmd, s),
                None => format!("{}{}", prefix, cmd),
            };
            let makefile = format!("target:\n\t{}", recipe_line);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok(), "Special chars with @ should parse");

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    prop_assert_eq!(recipe.len(), 1);
                    prop_assert_eq!(&recipe[0], &recipe_line);
                    prop_assert!(recipe[0].starts_with('@'), "@ prefix preserved with special chars");
                }
                _ => prop_assert!(false, "Expected Target"),
            }
        }

        #[test]
        fn prop_ECHO_001_multiple_targets_independent(
            n_targets in 1usize..4
        ) {
            // PROPERTY: @ prefix handling is independent across targets
            let mut makefile = String::new();
            let mut expected: Vec<(String, Vec<String>)> = Vec::new();

            for i in 0..n_targets {
                let target_name = format!("target{}", i);
                makefile.push_str(&format!("{}:\n", target_name));

                let mut target_recipes = Vec::new();

                // Even targets get @ prefix, odd don't
                if i % 2 == 0 {
                    let recipe = format!("@command{}", i);
                    makefile.push_str(&format!("\t{}\n", recipe));
                    target_recipes.push(recipe);
                } else {
                    let recipe = format!("command{}", i);
                    makefile.push_str(&format!("\t{}\n", recipe));
                    target_recipes.push(recipe);
                }

                expected.push((target_name, target_recipes));
            }

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), expected.len());

            for (i, (expected_name, expected_recipes)) in expected.iter().enumerate() {
                match &ast.items[i] {
                    MakeItem::Target { name, recipe, .. } => {
                        prop_assert_eq!(name, expected_name);
                        prop_assert_eq!(recipe.len(), expected_recipes.len());
                        for (j, expected_recipe) in expected_recipes.iter().enumerate() {
                            prop_assert_eq!(&recipe[j], expected_recipe);
                        }
                    }
                    _ => prop_assert!(false, "Expected Target at index {}", i),
                }
            }
        }
    }
}

// ============================================================================
// INCLUDE-001: Include directive
// ============================================================================

/// RED PHASE: Test for INCLUDE-001 - Basic include directive
///
/// This test validates GNU Make's `include` directive for modular Makefiles.
///
/// Input Makefile:
/// ```makefile
/// include common.mk
/// ```
///
/// Expected AST:
/// - One MakeItem::Include
/// - path: "common.mk"
/// - optional: false
///
/// Reference: GNU Make Manual Section 3.3 "Including Other Makefiles"
#[test]
fn test_INCLUDE_001_basic_include_directive() {
    // ARRANGE: Simple include directive
    let makefile = "include common.mk";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse include directive, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();

    // ASSERT: One item in AST
    assert_eq!(
        ast.items.len(),
        1,
        "Should have exactly one item, got {}",
        ast.items.len()
    );

    // ASSERT: Item is an Include
    match &ast.items[0] {
        MakeItem::Include { path, optional, .. } => {
            assert_eq!(path, "common.mk", "Include path should be common.mk");
            assert!(!optional, "Include should not be optional");
        }
        _ => panic!("Expected MakeItem::Include, got {:?}", ast.items[0]),
    }
}

/// RED PHASE: Test for INCLUDE-001 - Include with path
#[test]
fn test_INCLUDE_001_include_with_path() {
    // ARRANGE: Include directive with path
    let makefile = "include config/build.mk";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse include with path");

    let ast = result.unwrap();

    // ASSERT: Item is an Include with correct path
    match &ast.items[0] {
        MakeItem::Include { path, optional, .. } => {
            assert_eq!(
                path, "config/build.mk",
                "Include path should preserve directories"
            );
            assert!(!optional, "Include should not be optional");
        }
        _ => panic!("Expected MakeItem::Include"),
    }
}

/// RED PHASE: Test for INCLUDE-001 - Multiple include directives
#[test]
fn test_INCLUDE_001_multiple_includes() {
    // ARRANGE: Multiple include directives
    let makefile = "include config.mk\ninclude rules.mk\ninclude targets.mk";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse multiple includes");

    let ast = result.unwrap();

    // ASSERT: Three include items
    assert_eq!(ast.items.len(), 3, "Should have three include items");

    let expected_paths = ["config.mk", "rules.mk", "targets.mk"];
    for (i, expected_path) in expected_paths.iter().enumerate() {
        match &ast.items[i] {
            MakeItem::Include { path, optional, .. } => {
                assert_eq!(path, expected_path, "Include path {} should match", i);
                assert!(!optional, "Include {} should not be optional", i);
            }
            _ => panic!("Expected MakeItem::Include at index {}", i),
        }
    }
}

/// RED PHASE: Test for INCLUDE-001 - Include with variables
#[test]
fn test_INCLUDE_001_include_with_variables() {
    // ARRANGE: Include directive with variable reference
    let makefile = "include $(CONFIG_DIR)/common.mk";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse include with variable");

    let ast = result.unwrap();

    // ASSERT: Variable reference preserved in path
    match &ast.items[0] {
        MakeItem::Include { path, .. } => {
            assert_eq!(
                path, "$(CONFIG_DIR)/common.mk",
                "Include should preserve variable references"
            );
        }
        _ => panic!("Expected MakeItem::Include"),
    }
}

// RED PHASE: Property test - Include directives always parse
#[cfg(test)]
mod include_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
