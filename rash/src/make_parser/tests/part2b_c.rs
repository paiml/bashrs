#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

        #[test]
        fn prop_COND_001_ifdef_always_parses(
            var_name in "[A-Z_]{2,10}"
        ) {
            let makefile = format!("ifdef {}\nCFLAGS += -v\nendif", var_name);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Conditional { condition, .. } => {
                    match condition {
                        MakeCondition::IfDef(name) => {
                            prop_assert_eq!(name, &var_name);
                        }
                        other => return Err(TestCaseError::fail(format!("Expected IfDef, got {:?}", other))),
                    }
                }
                other => return Err(TestCaseError::fail(format!("Expected Conditional, got {:?}", other))),
            }
        }

        /// Property: ifndef conditionals always parse successfully
        #[test]
        fn prop_COND_001_ifndef_always_parses(
            var_name in "[A-Z_]{2,10}"
        ) {
            let makefile = format!("ifndef {}\n{} = default\nendif", var_name, var_name);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Conditional { condition, .. } => {
                    match condition {
                        MakeCondition::IfNdef(name) => {
                            prop_assert_eq!(name, &var_name);
                        }
                        other => return Err(TestCaseError::fail(format!("Expected IfNdef, got {:?}", other))),
                    }
                }
                other => return Err(TestCaseError::fail(format!("Expected Conditional, got {:?}", other))),
            }
        }

        /// Property: Conditionals with else branches always parse correctly
        #[test]
        fn prop_COND_001_else_branches_work(
            var_name in "[A-Z]{2,8}",
            then_val in "[a-z]{1,5}",
            else_val in "[a-z]{1,5}"
        ) {
            let makefile = format!(
                "ifeq ($(DEBUG),1)\nFLAGS = {}\nelse\nFLAGS = {}\nendif",
                then_val, else_val
            );

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Conditional { then_items, else_items, .. } => {
                    prop_assert_eq!(then_items.len(), 1);
                    prop_assert!(else_items.is_some());

                    let else_vec = else_items.as_ref().unwrap();
                    prop_assert_eq!(else_vec.len(), 1);

                    // Verify then branch value
                    match &then_items[0] {
                        MakeItem::Variable { value, .. } => {
                            prop_assert_eq!(value, &then_val);
                        }
                        _ => return Err(TestCaseError::fail("Expected Variable in then branch")),
                    }

                    // Verify else branch value
                    match &else_vec[0] {
                        MakeItem::Variable { value, .. } => {
                            prop_assert_eq!(value, &else_val);
                        }
                        _ => return Err(TestCaseError::fail("Expected Variable in else branch")),
                    }
                }
                other => return Err(TestCaseError::fail(format!("Expected Conditional, got {:?}", other))),
            }
        }

        /// Property: Parsing with conditionals is deterministic
        #[test]
        fn prop_COND_001_parsing_is_deterministic(
            var_name in "[A-Z]{2,8}",
            value in "[0-9]{1,3}"
        ) {
            let makefile = format!("ifeq ($({}),{})\nBUILD = yes\nendif", var_name, value);

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
                    MakeItem::Conditional { condition: c1, then_items: t1, else_items: e1, .. },
                    MakeItem::Conditional { condition: c2, then_items: t2, else_items: e2, .. }
                ) => {
                    // Conditions should match
                    prop_assert!(matches!((c1, c2), (MakeCondition::IfEq(_, _), MakeCondition::IfEq(_, _))));

                    // Then items should match
                    prop_assert_eq!(t1.len(), t2.len());

                    // Else items should match
                    prop_assert_eq!(e1.is_some(), e2.is_some());
                }
                _ => return Err(TestCaseError::fail("Expected Conditional in both ASTs")),
            }
        }

        /// Property: ifneq conditionals parse correctly
        #[test]
        fn prop_COND_001_ifneq_parses(
            left in "[a-z]{2,6}",
            right in "[a-z]{2,6}"
        ) {
            let makefile = format!("ifneq ({},{})\nTEST = 1\nendif", left, right);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Conditional { condition, .. } => {
                    match condition {
                        MakeCondition::IfNeq(l, r) => {
                            prop_assert_eq!(l, &left);
                            prop_assert_eq!(r, &right);
                        }
                        other => return Err(TestCaseError::fail(format!("Expected IfNeq, got {:?}", other))),
                    }
                }
                other => return Err(TestCaseError::fail(format!("Expected Conditional, got {:?}", other))),
            }
        }
    }
}

// ===========================================================================
// VAR-SUBST-001: Variable Substitution Tests
// ===========================================================================
// Task: Document variable substitution ($(VAR:suffix=replacement))
// Input: OBJS = $(SRCS:.c=.o)
// Goal: Parser preserves variable substitution syntax in variable values

#[test]
fn test_VAR_SUBST_001_basic_substitution() {
    // ARRANGE: Variable with substitution reference
    let makefile = "OBJS = $(SRCS:.c=.o)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse variable substitution, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Check variable with substitution
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS", "Variable name should be OBJS");
            assert_eq!(
                value, "$(SRCS:.c=.o)",
                "Variable value should preserve substitution syntax"
            );
        }
        other => panic!("Expected Variable item, got {:?}", other),
    }
}

#[test]
fn test_VAR_SUBST_001_multiple_substitutions() {
    // ARRANGE: Multiple variables with substitutions
    let makefile = "OBJS = $(SRCS:.c=.o)\nLIBS = $(DEPS:.a=.so)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse multiple substitutions");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2, "Should have 2 variables");

    // ASSERT: Check first substitution
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert_eq!(value, "$(SRCS:.c=.o)");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }

    // ASSERT: Check second substitution
    match &ast.items[1] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "LIBS");
            assert_eq!(value, "$(DEPS:.a=.so)");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_VAR_SUBST_001_substitution_with_path() {
    // ARRANGE: Substitution with path patterns
    let makefile = "OBJS = $(SRCS:src/%.c=build/%.o)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse path substitution");

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Variable { value, .. } => {
            assert_eq!(
                value, "$(SRCS:src/%.c=build/%.o)",
                "Should preserve path patterns in substitution"
            );
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_VAR_SUBST_001_substitution_in_recipe() {
    // ARRANGE: Substitution used in recipe
    let makefile = "build: $(SRCS:.c=.o)\n\t$(CC) $^ -o $@";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse substitution in prerequisites");

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Target { prerequisites, .. } => {
            assert_eq!(prerequisites.len(), 1);
            assert_eq!(
                prerequisites[0], "$(SRCS:.c=.o)",
                "Should preserve substitution in prerequisites"
            );
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

#[test]
fn test_VAR_SUBST_001_percent_substitution() {
    // ARRANGE: Substitution with % pattern
    let makefile = "OBJS = $(SRCS:%.c=%.o)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse % pattern substitution");

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Variable { value, .. } => {
            assert_eq!(
                value, "$(SRCS:%.c=%.o)",
                "Should preserve % pattern in substitution"
            );
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_VAR_SUBST_001_complex_substitution() {
    // ARRANGE: Complex substitution with multiple parts
    let makefile = "FILES = $(wildcard *.c)\nOBJS = $(FILES:.c=.o)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse complex substitutions");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2);

    // First variable has wildcard function
    match &ast.items[0] {
        MakeItem::Variable { value, .. } => {
            assert_eq!(value, "$(wildcard *.c)");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }

    // Second variable has substitution
    match &ast.items[1] {
        MakeItem::Variable { value, .. } => {
            assert_eq!(value, "$(FILES:.c=.o)");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

// ===========================================================================
// VAR-SUBST-001: Property Tests
// ===========================================================================

#[cfg(test)]
mod var_subst_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_VAR_SUBST_001_substitution_always_preserved(
            var_name in "[A-Z]{2,8}",
            ref_name in "[A-Z]{2,8}",
            from_ext in "\\.[a-z]{1,3}",
            to_ext in "\\.[a-z]{1,3}"
        ) {
            // ARRANGE: Variable with substitution pattern
            let makefile = format!("{} = $({}:{}={})", var_name, ref_name, from_ext, to_ext);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Always parses successfully
            prop_assert!(result.is_ok(), "Substitution should parse");

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1, "Should have 1 variable");

            // ASSERT: Substitution syntax preserved
            match &ast.items[0] {
                MakeItem::Variable { name, value, .. } => {
                    prop_assert_eq!(name, &var_name);
                    let expected = format!("$({}:{}={})", ref_name, from_ext, to_ext);
                    prop_assert_eq!(value, &expected, "Substitution syntax should be preserved");
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }
        }

        #[test]
        fn prop_VAR_SUBST_001_percent_patterns_preserved(
            var_name in "[A-Z]{2,8}",
            ref_name in "[A-Z]{2,8}",
            from_pattern in "%\\.[a-z]{1,3}",
            to_pattern in "%\\.[a-z]{1,3}"
        ) {
            // ARRANGE: Substitution with % patterns
            let makefile = format!("{} = $({}:{}={})", var_name, ref_name, from_pattern, to_pattern);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: % patterns preserved
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Variable { value, .. } => {
                    let expected = format!("$({}:{}={})", ref_name, from_pattern, to_pattern);
                    prop_assert_eq!(value, &expected, "% patterns should be preserved");
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }
        }

        #[test]
        fn prop_VAR_SUBST_001_parsing_is_deterministic(
            var_name in "[A-Z]{2,8}",
            ref_name in "[A-Z]{2,8}"
        ) {
            // ARRANGE: Simple substitution
            let makefile = format!("{} = $({}:.c=.o)", var_name, ref_name);

            // ACT: Parse twice
            let result1 = parse_makefile(&makefile);
            let result2 = parse_makefile(&makefile);

            // ASSERT: Results are identical (deterministic)
            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());

            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();

            prop_assert_eq!(ast1.items.len(), ast2.items.len());

            match (&ast1.items[0], &ast2.items[0]) {
                (MakeItem::Variable { value: v1, .. }, MakeItem::Variable { value: v2, .. }) => {
                    prop_assert_eq!(v1, v2, "Parsing should be deterministic");
                }
                _ => return Err(TestCaseError::fail("Both should be Variables")),
            }
        }

        #[test]
        fn prop_VAR_SUBST_001_path_patterns_preserved(
            ref_name in "[A-Z]{2,8}",
            from_dir in "[a-z]{3,6}",
            to_dir in "[a-z]{3,6}"
        ) {
            // ARRANGE: Substitution with path patterns
            let makefile = format!("OBJS = $({}:{}/%={}/%)", ref_name, from_dir, to_dir);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Path patterns preserved
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Variable { value, .. } => {
                    let expected = format!("$({}:{}/%={}/%)", ref_name, from_dir, to_dir);
                    prop_assert_eq!(value, &expected, "Path patterns should be preserved");
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }
        }

        #[test]
        fn prop_VAR_SUBST_001_in_prerequisites_preserved(
            target_name in "[a-z]{3,8}",
            ref_name in "[A-Z]{2,8}",
            from_ext in "\\.[a-z]{1,3}",
            to_ext in "\\.[a-z]{1,3}"
        ) {
            // ARRANGE: Substitution in target prerequisites
            let makefile = format!("{}: $({}:{}={})\n\techo test", target_name, ref_name, from_ext, to_ext);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Substitution preserved in prerequisites
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { prerequisites, .. } => {
                    prop_assert_eq!(prerequisites.len(), 1);
                    let expected = format!("$({}:{}={})", ref_name, from_ext, to_ext);
                    prop_assert_eq!(&prerequisites[0], &expected, "Substitution should be preserved in prerequisites");
                }
                other => return Err(TestCaseError::fail(format!("Expected Target, got {:?}", other))),
            }
        }

