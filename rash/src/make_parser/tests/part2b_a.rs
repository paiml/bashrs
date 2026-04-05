#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

        #[test]
        fn prop_INCLUDE_002_paths_with_directories(
            dir in "[a-z]{3,10}",
            file in "[a-z]{3,10}\\.mk"
        ) {
            // ARRANGE: Optional include with directory path
            let makefile = format!("-include {}/{}", dir, file);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Path preserved with directory
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Include { path, optional, .. } => {
                    prop_assert!(path.contains('/'));
                    prop_assert!(path.ends_with(".mk"));
                    prop_assert!(*optional);
                }
                other => return Err(TestCaseError::fail(format!("Expected Include, got {:?}", other))),
            }
        }

        #[test]
        fn prop_INCLUDE_002_var_refs_preserved(
            var_name in "[A-Z_]{2,10}",
            file in "[a-z]{3,10}\\.mk"
        ) {
            // ARRANGE: Optional include with variable reference
            let makefile = format!("-include $({})/{}", var_name, file);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Variable reference preserved
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Include { path, optional, .. } => {
                    prop_assert!(path.contains("$("));
                    prop_assert!(path.contains(&var_name));
                    prop_assert!(*optional);
                }
                other => return Err(TestCaseError::fail(format!("Expected Include, got {:?}", other))),
            }
        }
    }
}

// ==============================================================================
// PATTERN-001: Pattern Rules (%.o: %.c)
// ==============================================================================

/// RED PHASE: Test for PATTERN-001 - Basic pattern rule
///
/// Pattern rules use % to match file stems. This is foundational for
/// automatic compilation rules like %.o: %.c
///
/// Input Makefile:
/// ```makefile
/// %.o: %.c
///     $(CC) -c $< -o $@
/// ```
///
/// Expected AST:
/// - One MakeItem::PatternRule
/// - target_pattern: "%.o"
/// - prereq_patterns: ["%.c"]
/// - recipe: ["$(CC) -c $< -o $@"]
#[test]
fn test_PATTERN_001_basic_pattern_rule() {
    // ARRANGE: Simple pattern rule
    let makefile = "%.o: %.c\n\t$(CC) -c $< -o $@";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse basic pattern rule, got error: {:?}",
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

    // ASSERT: Item is a PatternRule
    match &ast.items[0] {
        MakeItem::PatternRule {
            target_pattern,
            prereq_patterns,
            recipe,
            ..
        } => {
            assert_eq!(target_pattern, "%.o", "Target pattern should be '%.o'");
            assert_eq!(
                prereq_patterns.len(),
                1,
                "Should have one prerequisite pattern"
            );
            assert_eq!(
                prereq_patterns[0], "%.c",
                "Prerequisite pattern should be '%.c'"
            );
            assert_eq!(recipe.len(), 1, "Should have one recipe line");
            assert_eq!(
                recipe[0], "$(CC) -c $< -o $@",
                "Recipe should contain automatic variables"
            );
        }
        other => panic!("Expected PatternRule item, got {:?}", other),
    }
}

/// RED PHASE: Test for PATTERN-001 - Pattern rule with multiple prerequisites
#[test]
fn test_PATTERN_001_pattern_rule_multiple_prerequisites() {
    // ARRANGE: Pattern rule with multiple prerequisites
    let makefile = "%.o: %.c %.h\n\t$(CC) -c $< -o $@";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check prerequisites
    match &ast.items[0] {
        MakeItem::PatternRule {
            target_pattern,
            prereq_patterns,
            ..
        } => {
            assert_eq!(target_pattern, "%.o");
            assert_eq!(prereq_patterns.len(), 2);
            assert_eq!(prereq_patterns[0], "%.c");
            assert_eq!(prereq_patterns[1], "%.h");
        }
        other => panic!("Expected PatternRule, got {:?}", other),
    }
}

/// RED PHASE: Test for PATTERN-001 - Pattern rule without recipe
#[test]
fn test_PATTERN_001_pattern_rule_empty_recipe() {
    // ARRANGE: Pattern rule with no recipe (just dependencies)
    let makefile = "%.o: %.c";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Empty recipe
    match &ast.items[0] {
        MakeItem::PatternRule {
            target_pattern,
            prereq_patterns,
            recipe,
            ..
        } => {
            assert_eq!(target_pattern, "%.o");
            assert_eq!(prereq_patterns.len(), 1);
            assert_eq!(prereq_patterns[0], "%.c");
            assert_eq!(recipe.len(), 0, "Recipe should be empty");
        }
        other => panic!("Expected PatternRule, got {:?}", other),
    }
}

/// RED PHASE: Test for PATTERN-001 - Distinguish pattern rule from normal target
#[test]
fn test_PATTERN_001_pattern_vs_normal_target() {
    // ARRANGE: Both pattern rule and normal target
    let makefile = "%.o: %.c\n\t$(CC) -c $< -o $@\n\nmain.o: main.c\n\t$(CC) -c main.c";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2);

    // ASSERT: First is pattern, second is normal target
    match &ast.items[0] {
        MakeItem::PatternRule { target_pattern, .. } => {
            assert_eq!(target_pattern, "%.o");
        }
        other => panic!("First item should be PatternRule, got {:?}", other),
    }

    match &ast.items[1] {
        MakeItem::Target { name, .. } => {
            assert_eq!(name, "main.o");
        }
        other => panic!("Second item should be Target, got {:?}", other),
    }
}

// ==============================================================================
// PATTERN-001: Property Tests
// ==============================================================================

#[cfg(test)]
mod prop_pattern_001 {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Pattern rules with % are always parsed as PatternRule
        #[test]
        fn prop_PATTERN_001_percent_always_creates_pattern_rule(
            target_ext in "[a-z]{1,3}",
            prereq_ext in "[a-z]{1,3}"
        ) {
            let makefile = format!("%.{}: %.{}\n\techo test", target_ext, prereq_ext);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            // Verify it's a PatternRule
            match &ast.items[0] {
                MakeItem::PatternRule { target_pattern, prereq_patterns, .. } => {
                    prop_assert_eq!(target_pattern, &format!("%.{}", target_ext));
                    prop_assert_eq!(prereq_patterns.len(), 1);
                    prop_assert_eq!(&prereq_patterns[0], &format!("%.{}", prereq_ext));
                }
                other => return Err(TestCaseError::fail(format!("Expected PatternRule, got {:?}", other))),
            }
        }

        /// Property: Targets without % are never parsed as PatternRule
        #[test]
        fn prop_PATTERN_001_no_percent_creates_normal_target(
            target in "[a-z]{1,10}\\.o",
            prereq in "[a-z]{1,10}\\.c"
        ) {
            let makefile = format!("{}: {}\n\techo test", target, prereq);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            // Verify it's a Target, not PatternRule
            match &ast.items[0] {
                MakeItem::Target { name, .. } => {
                    prop_assert_eq!(name, &target);
                }
                MakeItem::PatternRule { .. } => {
                    return Err(TestCaseError::fail("Should not create PatternRule without %"));
                }
                other => return Err(TestCaseError::fail(format!("Expected Target, got {:?}", other))),
            }
        }

        /// Property: Pattern rules with multiple prerequisites preserve order
        #[test]
        fn prop_PATTERN_001_pattern_prereq_order_preserved(
            ext1 in "[a-z]{1,3}",
            ext2 in "[a-z]{1,3}",
            ext3 in "[a-z]{1,3}"
        ) {
            let makefile = format!("%.o: %.{} %.{} %.{}\n\techo test", ext1, ext2, ext3);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::PatternRule { prereq_patterns, .. } => {
                    prop_assert_eq!(prereq_patterns.len(), 3);
                    prop_assert_eq!(&prereq_patterns[0], &format!("%.{}", ext1));
                    prop_assert_eq!(&prereq_patterns[1], &format!("%.{}", ext2));
                    prop_assert_eq!(&prereq_patterns[2], &format!("%.{}", ext3));
                }
                other => return Err(TestCaseError::fail(format!("Expected PatternRule, got {:?}", other))),
            }
        }

        /// Property: Pattern rule parsing is deterministic
        #[test]
        fn prop_PATTERN_001_parsing_is_deterministic(
            target_ext in "[a-z]{1,5}",
            prereq_ext in "[a-z]{1,5}"
        ) {
            let makefile = format!("%.{}: %.{}\n\t$(CC) -c $< -o $@", target_ext, prereq_ext);

            // Parse twice
            let result1 = parse_makefile(&makefile);
            let result2 = parse_makefile(&makefile);

            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());

            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();

            // Should produce identical ASTs
            prop_assert_eq!(ast1.items.len(), ast2.items.len());

            match (&ast1.items[0], &ast2.items[0]) {
                (
                    MakeItem::PatternRule { target_pattern: t1, prereq_patterns: p1, recipe: r1, .. },
                    MakeItem::PatternRule { target_pattern: t2, prereq_patterns: p2, recipe: r2, .. }
                ) => {
                    prop_assert_eq!(t1, t2);
                    prop_assert_eq!(p1, p2);
                    prop_assert_eq!(r1, r2);
                }
                _ => return Err(TestCaseError::fail("Both should be PatternRule")),
            }
        }

        /// Property: Empty recipes are handled correctly for pattern rules
        #[test]
        fn prop_PATTERN_001_empty_recipes_allowed(
            target_ext in "[a-z]{1,4}",
            prereq_ext in "[a-z]{1,4}"
        ) {
            let makefile = format!("%.{}: %.{}", target_ext, prereq_ext);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::PatternRule { target_pattern, prereq_patterns, recipe, .. } => {
                    prop_assert_eq!(target_pattern, &format!("%.{}", target_ext));
                    prop_assert_eq!(prereq_patterns.len(), 1);
                    prop_assert_eq!(recipe.len(), 0, "Recipe should be empty");
                }
                other => return Err(TestCaseError::fail(format!("Expected PatternRule, got {:?}", other))),
            }
        }
    }
}

// ==============================================================================
// PATTERN-002: Automatic Variables ($@, $<, $^)
// ==============================================================================

/// RED PHASE: Test for PATTERN-002 - Automatic variable $@ (target name)
///
/// Automatic variables are special variables set by make for each rule.
/// $@ expands to the target name.
///
/// Input Makefile:
/// ```makefile
/// program: main.o
///     $(CC) -o $@ main.o
/// ```
///
/// Expected: Recipe preserves "$@" exactly as-is
#[test]
fn test_PATTERN_002_automatic_variable_at() {
    // ARRANGE: Target with $@ automatic variable
    let makefile = "program: main.o\n\t$(CC) -o $@ main.o";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Recipe contains $@
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 1);
            assert!(recipe[0].contains("$@"), "Recipe should contain $@");
            assert_eq!(recipe[0], "$(CC) -o $@ main.o");
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

/// RED PHASE: Test for PATTERN-002 - Automatic variable $< (first prerequisite)
///
/// $< expands to the name of the first prerequisite.
/// Commonly used in pattern rules.
#[test]
fn test_PATTERN_002_automatic_variable_less_than() {
    // ARRANGE: Pattern rule with $< automatic variable
    let makefile = "%.o: %.c\n\t$(CC) -c $< -o $@";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Recipe contains $<
    match &ast.items[0] {
        MakeItem::PatternRule { recipe, .. } => {
            assert_eq!(recipe.len(), 1);
            assert!(recipe[0].contains("$<"), "Recipe should contain $<");
            assert!(recipe[0].contains("$@"), "Recipe should contain $@");
            assert_eq!(recipe[0], "$(CC) -c $< -o $@");
        }
        other => panic!("Expected PatternRule, got {:?}", other),
    }
}

/// RED PHASE: Test for PATTERN-002 - Automatic variable $^ (all prerequisites)
///
/// $^ expands to the names of all prerequisites, with spaces between them.
#[test]
fn test_PATTERN_002_automatic_variable_caret() {
    // ARRANGE: Target with $^ automatic variable
    let makefile = "program: main.o util.o\n\t$(CC) $^ -o $@";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Recipe contains $^
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 1);
            assert!(recipe[0].contains("$^"), "Recipe should contain $^");
            assert!(recipe[0].contains("$@"), "Recipe should contain $@");
            assert_eq!(recipe[0], "$(CC) $^ -o $@");
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

/// RED PHASE: Test for PATTERN-002 - Multiple automatic variables in one recipe
#[test]
fn test_PATTERN_002_multiple_automatic_variables() {
    // ARRANGE: Recipe with multiple automatic variables
    let makefile = "%.o: %.c %.h\n\t$(CC) -c $< -o $@ -I $^";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: All automatic variables preserved
    match &ast.items[0] {
        MakeItem::PatternRule { recipe, .. } => {
            assert_eq!(recipe.len(), 1);
            assert!(recipe[0].contains("$<"));
            assert!(recipe[0].contains("$@"));
            assert!(recipe[0].contains("$^"));
        }
        other => panic!("Expected PatternRule, got {:?}", other),
    }
}

/// RED PHASE: Test for PATTERN-002 - Automatic variable $? (newer prerequisites)
