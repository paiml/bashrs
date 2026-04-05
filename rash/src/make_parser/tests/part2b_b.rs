#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

#[test]
fn test_PATTERN_002_automatic_variable_question() {
    // ARRANGE: Target with $? automatic variable
    let makefile = "archive.a: foo.o bar.o\n\tar rcs $@ $?";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Recipe contains $?
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 1);
            assert!(recipe[0].contains("$?"), "Recipe should contain $?");
            assert_eq!(recipe[0], "ar rcs $@ $?");
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

// ==============================================================================
// PATTERN-002: Property Tests
// ==============================================================================

#[cfg(test)]
mod prop_pattern_002 {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Automatic variables in recipes are always preserved
        #[test]
        fn prop_PATTERN_002_automatic_vars_always_preserved(
            target in "[a-z]{1,10}",
            prereq in "[a-z]{1,10}\\.[a-z]{1,3}"
        ) {
            let makefile = format!("{}: {}\n\t$(CC) $< -o $@", target, prereq);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    prop_assert!(recipe[0].contains("$<"));
                    prop_assert!(recipe[0].contains("$@"));
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        /// Property: All common automatic variables are preserved
        #[test]
        fn prop_PATTERN_002_all_auto_vars_preserved(
            target in "[a-z]{1,8}"
        ) {
            // Test $@, $<, $^, $?
            let makefile = format!("{}: a.o b.o\n\techo $@ $< $^ $?", target);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    prop_assert!(recipe[0].contains("$@"));
                    prop_assert!(recipe[0].contains("$<"));
                    prop_assert!(recipe[0].contains("$^"));
                    prop_assert!(recipe[0].contains("$?"));
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        /// Property: Automatic variables in pattern rules are preserved
        #[test]
        fn prop_PATTERN_002_pattern_rules_preserve_auto_vars(
            ext1 in "[a-z]{1,3}",
            ext2 in "[a-z]{1,3}"
        ) {
            let makefile = format!("%.{}: %.{}\n\t$(CC) -c $< -o $@", ext1, ext2);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::PatternRule { recipe, .. } => {
                    prop_assert!(recipe[0].contains("$<"));
                    prop_assert!(recipe[0].contains("$@"));
                }
                _ => return Err(TestCaseError::fail("Expected PatternRule")),
            }
        }

        /// Property: Parsing with automatic variables is deterministic
        #[test]
        fn prop_PATTERN_002_parsing_is_deterministic(
            target in "[a-z]{1,10}",
            prereq in "[a-z]{1,10}\\.[a-z]{1,3}"
        ) {
            let makefile = format!("{}: {}\n\t$(CC) $^ -o $@", target, prereq);

            // Parse twice
            let result1 = parse_makefile(&makefile);
            let result2 = parse_makefile(&makefile);

            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());

            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();

            // Should produce identical ASTs
            match (&ast1.items[0], &ast2.items[0]) {
                (
                    MakeItem::Target { recipe: r1, .. },
                    MakeItem::Target { recipe: r2, .. }
                ) => {
                    prop_assert_eq!(r1, r2);
                }
                _ => return Err(TestCaseError::fail("Expected Target in both")),
            }
        }

        /// Property: Mix of automatic variables and normal text preserved
        #[test]
        fn prop_PATTERN_002_mixed_content_preserved(
            target in "[a-z]{1,8}",
            flag in "-[a-zA-Z]"
        ) {
            let makefile = format!("{}: a.o\n\tgcc {} $< -o $@", target, flag);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    // Verify automatic variables preserved
                    prop_assert!(recipe[0].contains("$<"));
                    prop_assert!(recipe[0].contains("$@"));
                    // Verify flag preserved
                    prop_assert!(recipe[0].contains(&flag));
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }
    }
}

// ==============================================================================
// COND-001: ifeq Conditionals
// ==============================================================================

/// RED PHASE: Test for COND-001 - Basic ifeq conditional
///
/// This test validates parsing of ifeq conditionals, which allow
/// conditional variable assignment and target rules in Makefiles.
///
/// Input Makefile:
/// ```makefile
/// ifeq ($(DEBUG),1)
/// CFLAGS = -g
/// endif
/// ```
///
/// Expected AST:
/// - One MakeItem::Conditional
/// - condition: IfEq("$(DEBUG)", "1")
/// - then_items: [Variable assignment CFLAGS = -g]
/// - else_items: None
#[test]
fn test_COND_001_basic_ifeq() {
    // ARRANGE: Simple ifeq conditional
    let makefile = "ifeq ($(DEBUG),1)\nCFLAGS = -g\nendif";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse basic ifeq conditional, got error: {:?}",
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

    // ASSERT: Item is a Conditional
    match &ast.items[0] {
        MakeItem::Conditional {
            condition,
            then_items,
            else_items,
            ..
        } => {
            // Check condition type
            match condition {
                MakeCondition::IfEq(left, right) => {
                    assert_eq!(left, "$(DEBUG)", "Left side should be $(DEBUG)");
                    assert_eq!(right, "1", "Right side should be 1");
                }
                other => panic!("Expected IfEq condition, got {:?}", other),
            }

            // Check then branch
            assert_eq!(then_items.len(), 1, "Should have one item in then branch");
            match &then_items[0] {
                MakeItem::Variable { name, value, .. } => {
                    assert_eq!(name, "CFLAGS", "Variable name should be CFLAGS");
                    assert_eq!(value, "-g", "Variable value should be -g");
                }
                other => panic!("Expected Variable in then branch, got {:?}", other),
            }

            // Check no else branch
            assert!(else_items.is_none(), "Should have no else branch");
        }
        other => panic!("Expected Conditional item, got {:?}", other),
    }
}

/// RED PHASE: Test for COND-001 - ifeq with else branch
#[test]
fn test_COND_001_ifeq_with_else() {
    // ARRANGE: ifeq conditional with else branch
    let makefile = "ifeq ($(DEBUG),1)\nCFLAGS = -g\nelse\nCFLAGS = -O2\nendif";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse ifeq with else, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check conditional structure
    match &ast.items[0] {
        MakeItem::Conditional {
            condition,
            then_items,
            else_items,
            ..
        } => {
            // Check condition
            match condition {
                MakeCondition::IfEq(left, right) => {
                    assert_eq!(left, "$(DEBUG)");
                    assert_eq!(right, "1");
                }
                other => panic!("Expected IfEq, got {:?}", other),
            }

            // Check then branch
            assert_eq!(then_items.len(), 1);
            match &then_items[0] {
                MakeItem::Variable { value, .. } => {
                    assert_eq!(value, "-g");
                }
                other => panic!("Expected Variable, got {:?}", other),
            }

            // Check else branch
            assert!(else_items.is_some(), "Should have else branch");
            let else_vec = else_items.as_ref().unwrap();
            assert_eq!(else_vec.len(), 1);
            match &else_vec[0] {
                MakeItem::Variable { value, .. } => {
                    assert_eq!(value, "-O2");
                }
                other => panic!("Expected Variable in else, got {:?}", other),
            }
        }
        other => panic!("Expected Conditional, got {:?}", other),
    }
}

/// RED PHASE: Test for COND-001 - ifdef conditional
#[test]
fn test_COND_001_ifdef() {
    // ARRANGE: ifdef conditional (checks if variable is defined)
    let makefile = "ifdef VERBOSE\nCFLAGS += -v\nendif";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse ifdef conditional, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check ifdef condition
    match &ast.items[0] {
        MakeItem::Conditional {
            condition,
            then_items,
            ..
        } => {
            match condition {
                MakeCondition::IfDef(var_name) => {
                    assert_eq!(var_name, "VERBOSE", "Should check VERBOSE variable");
                }
                other => panic!("Expected IfDef, got {:?}", other),
            }

            assert_eq!(then_items.len(), 1);
        }
        other => panic!("Expected Conditional, got {:?}", other),
    }
}

/// RED PHASE: Test for COND-001 - ifndef conditional
#[test]
fn test_COND_001_ifndef() {
    // ARRANGE: ifndef conditional (checks if variable is NOT defined)
    let makefile = "ifndef CC\nCC = gcc\nendif";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse ifndef conditional, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check ifndef condition
    match &ast.items[0] {
        MakeItem::Conditional {
            condition,
            then_items,
            ..
        } => {
            match condition {
                MakeCondition::IfNdef(var_name) => {
                    assert_eq!(var_name, "CC", "Should check CC variable");
                }
                other => panic!("Expected IfNdef, got {:?}", other),
            }

            assert_eq!(then_items.len(), 1);
            match &then_items[0] {
                MakeItem::Variable { name, value, .. } => {
                    assert_eq!(name, "CC");
                    assert_eq!(value, "gcc");
                }
                other => panic!("Expected Variable, got {:?}", other),
            }
        }
        other => panic!("Expected Conditional, got {:?}", other),
    }
}

/// RED PHASE: Test for COND-001 - Conditional with targets in branches
#[test]
fn test_COND_001_conditional_with_targets() {
    // ARRANGE: Conditional containing target rules
    let makefile = "ifeq ($(OS),Linux)\ninstall:\n\tcp app /usr/bin/app\nendif";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse conditional with targets, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check conditional contains target
    match &ast.items[0] {
        MakeItem::Conditional { then_items, .. } => {
            assert_eq!(then_items.len(), 1);
            match &then_items[0] {
                MakeItem::Target { name, recipe, .. } => {
                    assert_eq!(name, "install");
                    assert_eq!(recipe.len(), 1);
                    assert_eq!(recipe[0], "cp app /usr/bin/app");
                }
                other => panic!("Expected Target in then branch, got {:?}", other),
            }
        }
        other => panic!("Expected Conditional, got {:?}", other),
    }
}

/// RED PHASE: Test for COND-001 - ifneq conditional
#[test]
fn test_COND_001_ifneq() {
    // ARRANGE: ifneq conditional (inequality test)
    let makefile = "ifneq ($(DEBUG),0)\nCFLAGS += -g\nendif";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse ifneq conditional, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check ifneq condition
    match &ast.items[0] {
        MakeItem::Conditional { condition, .. } => match condition {
            MakeCondition::IfNeq(left, right) => {
                assert_eq!(left, "$(DEBUG)");
                assert_eq!(right, "0");
            }
            other => panic!("Expected IfNeq, got {:?}", other),
        },
        other => panic!("Expected Conditional, got {:?}", other),
    }
}

// ==============================================================================
// COND-001: Property Tests
// ==============================================================================

#[cfg(test)]
mod prop_cond_001 {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: ifeq conditionals always parse successfully with valid syntax
        #[test]
        fn prop_COND_001_ifeq_always_parses(
            var_name in "[A-Z]{2,8}",
            value in "[a-z0-9]{1,5}"
        ) {
            let makefile = format!("ifeq ($({}),{})\nCFLAGS = -g\nendif", var_name, value);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            match &ast.items[0] {
                MakeItem::Conditional { condition, then_items, else_items, .. } => {
                    // Verify condition is IfEq
                    match condition {
                        MakeCondition::IfEq(left, right) => {
                            prop_assert!(left.contains(&var_name));
                            prop_assert_eq!(right, &value);
                        }
                        other => return Err(TestCaseError::fail(format!("Expected IfEq, got {:?}", other))),
                    }

                    // Verify then branch has variable
                    prop_assert_eq!(then_items.len(), 1);
                    prop_assert!(else_items.is_none());
                }
                other => return Err(TestCaseError::fail(format!("Expected Conditional, got {:?}", other))),
            }
        }

        /// Property: ifdef conditionals always parse successfully
