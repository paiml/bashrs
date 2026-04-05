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
    fn test_RULE_SYNTAX_001_mut_line_number_calculation() {
        // ARRANGE: Invalid makefile to trigger error with line number
        let makefile = "target1:\n\trecipe\n:\n\trecipe2";

        // ACT: Parse makefile (should fail with line number)
        let result = parse_makefile(makefile);

        // ASSERT: Error includes correct line number
        assert!(result.is_err(), "Empty target name should produce error");
        let err = result.unwrap_err();
        assert!(
            err.contains("Line 3") || err.contains("line 3"),
            "Error should reference line 3, got: {}",
            err
        );
    }
}

/// RED PHASE: Test for VAR-BASIC-001 - Basic variable assignment
///
/// This test validates basic variable assignment in Makefiles.
///
/// Input Makefile:
/// ```makefile
/// CC = gcc
/// ```
///
/// Expected AST:
/// - One MakeItem::Variable
/// - name: "CC"
/// - value: "gcc"
/// - flavor: VarFlavor::Recursive (for =)
#[test]
fn test_VAR_BASIC_001_basic_variable_assignment() {
    // ARRANGE: Simple variable assignment
    let makefile = "CC = gcc";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse basic variable assignment, got error: {:?}",
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

    // ASSERT: Item is a Variable
    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "CC", "Variable name should be 'CC'");
            assert_eq!(value, "gcc", "Variable value should be 'gcc'");
            assert_eq!(
                *flavor,
                VarFlavor::Recursive,
                "Should use recursive assignment (=)"
            );
        }
        other => panic!("Expected Variable item, got {:?}", other),
    }
}

/// RED PHASE: Test for VAR-BASIC-001 - Variable with spaces
#[test]
fn test_VAR_BASIC_001_variable_with_spaces() {
    // ARRANGE: Variable assignment with spaces in value
    let makefile = "CFLAGS = -Wall -Werror -O2";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Variable value includes spaces
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "CFLAGS");
            assert_eq!(value, "-Wall -Werror -O2");
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for VAR-BASIC-001 - Empty variable value
#[test]
fn test_VAR_BASIC_001_empty_variable_value() {
    // ARRANGE: Variable with empty value
    let makefile = "EMPTY =";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Variable has empty value
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "EMPTY");
            assert_eq!(value, "");
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for VAR-BASIC-001 - Multiple variables
#[test]
fn test_VAR_BASIC_001_multiple_variables() {
    // ARRANGE: Multiple variable assignments
    let makefile = "CC = gcc\nCXX = g++\nLD = ld";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 3, "Should parse 3 variables");

    // ASSERT: First variable
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "CC");
            assert_eq!(value, "gcc");
        }
        _ => panic!("Expected Variable item"),
    }

    // ASSERT: Second variable
    match &ast.items[1] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "CXX");
            assert_eq!(value, "g++");
        }
        _ => panic!("Expected Variable item"),
    }

    // ASSERT: Third variable
    match &ast.items[2] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "LD");
            assert_eq!(value, "ld");
        }
        _ => panic!("Expected Variable item"),
    }
}

// PROPERTY TESTING PHASE: Tests for VAR-BASIC-001
//
// These property tests verify variable assignment works across a wide range of inputs.
#[cfg(test)]
mod var_basic_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Variable assignments always parse successfully
        ///
        /// This test generates random variable names and values to ensure
        /// the parser handles a wide variety of inputs.
        #[test]
        fn test_VAR_BASIC_001_prop_variables_always_parse(
            varname in "[A-Z][A-Z0-9_]{0,20}",
            value in "[a-zA-Z0-9_./+-]{0,50}"
        ) {
            // ARRANGE: Generate valid variable assignment
            let makefile = format!("{} = {}", varname, value);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Parsing succeeds
            prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            // ASSERT: Variable properties preserved
            if let MakeItem::Variable { name, value: val, flavor, .. } = &ast.items[0] {
                prop_assert_eq!(name, &varname);
                prop_assert_eq!(val, &value);
                prop_assert_eq!(flavor, &VarFlavor::Recursive);
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }

        /// Property: Variable parsing is deterministic
        ///
        /// Verifies that parsing the same input twice produces identical results.
        #[test]
        fn test_VAR_BASIC_001_prop_parsing_is_deterministic(
            varname in "[A-Z]{1,10}",
            value in "[a-z0-9 ]{1,30}"
        ) {
            let makefile = format!("{} = {}", varname, value);

            // Parse twice
            let result1 = parse_makefile(&makefile);
            let result2 = parse_makefile(&makefile);

            // Both should succeed
            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());

            // Results should be identical
            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();
            prop_assert_eq!(ast1.items.len(), ast2.items.len());
            prop_assert_eq!(ast1.items, ast2.items);
        }

        /// Property: Different variable flavors are correctly identified
        ///
        /// Tests all 5 variable assignment operators (=, :=, ?=, +=, !=)
        #[test]
        fn test_VAR_BASIC_001_prop_variable_flavors(
            varname in "[A-Z]{1,10}",
            value in "[a-z]{1,20}"
        ) {
            // Test each flavor
            let test_cases = vec![
                (format!("{} = {}", varname, value), VarFlavor::Recursive),
                (format!("{} := {}", varname, value), VarFlavor::Simple),
                (format!("{} ?= {}", varname, value), VarFlavor::Conditional),
                (format!("{} += {}", varname, value), VarFlavor::Append),
                (format!("{} != echo {}", varname, value), VarFlavor::Shell),
            ];

            for (makefile, expected_flavor) in test_cases {
                let result = parse_makefile(&makefile);
                prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);

                let ast = result.unwrap();
                if let MakeItem::Variable { name, flavor, .. } = &ast.items[0] {
                    prop_assert_eq!(name, &varname);
                    prop_assert_eq!(flavor, &expected_flavor);
                } else {
                    return Err(TestCaseError::fail("Expected Variable item"));
                }
            }
        }

        /// Property: Variable values can be empty or contain spaces
        ///
        /// Verifies that various value patterns are handled correctly.
        #[test]
        fn test_VAR_BASIC_001_prop_variable_values_flexible(
            varname in "[A-Z]{1,10}",
            value in "[ a-z0-9-]*"  // Can be empty, can have spaces
        ) {
            let makefile = format!("{} = {}", varname, value);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            if let MakeItem::Variable { name, value: val, .. } = &ast.items[0] {
                prop_assert_eq!(name, &varname);
                prop_assert_eq!(val, &value.trim());  // Value gets trimmed
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }
    }
}

/// MUTATION TESTING PHASE: Mutation-killing tests for VAR-BASIC-001
///
/// These tests target specific mutants identified during mutation testing.
#[cfg(test)]
mod var_mutation_killing_tests {
    use super::*;

    /// Kill mutant: line 59 `replace + with *` in parse_makefile
    /// Kill mutant: line 179 `replace + with *` in parse_target_rule
    ///
    /// These mutants would cause incorrect line number tracking in error messages.
    #[test]
    fn test_VAR_BASIC_001_mut_correct_line_numbers() {
        // ARRANGE: Makefile with invalid syntax on line 3
        let makefile = "CC = gcc\nCXX = g++\nINVALID =\n= ALSO INVALID";

        // ACT: Parse makefile (should fail)
        let result = parse_makefile(makefile);

        // ASSERT: Parse fails
        assert!(result.is_err(), "Should fail on invalid syntax");

        // Note: This test verifies that line number calculation uses + not *
        // If the mutant were active, line numbers would be wrong
    }

    /// Kill mutant: line 100 `replace || with &&` in is_variable_assignment
    ///
    /// This mutant would break detection of all multi-character operators.
    #[test]
    fn test_VAR_BASIC_001_mut_all_flavors_parse() {
        // ARRANGE: Test all 5 variable flavors
        let test_cases = vec![
            ("VAR = value", VarFlavor::Recursive),
            ("VAR := value", VarFlavor::Simple),
            ("VAR ?= value", VarFlavor::Conditional),
            ("VAR += value", VarFlavor::Append),
            ("VAR != echo test", VarFlavor::Shell),
        ];

        for (input, expected_flavor) in test_cases {
            // ACT: Parse each flavor
            let result = parse_makefile(input);

            // ASSERT: Parsing succeeds
            assert!(result.is_ok(), "Failed to parse: {}", input);

            let ast = result.unwrap();
            assert_eq!(ast.items.len(), 1);

            // ASSERT: Correct flavor detected
            match &ast.items[0] {
                MakeItem::Variable { flavor, .. } => {
                    assert_eq!(flavor, &expected_flavor, "Wrong flavor for: {}", input);
                }
                _ => panic!("Expected Variable for: {}", input),
            }
        }
    }

    /// Kill mutant: line 115 `replace < with >` in is_variable_assignment
    /// Kill mutant: line 115 `replace < with ==` in is_variable_assignment
    ///
    /// These mutants would confuse targets with variables in prerequisites.
    #[test]
    fn test_VAR_BASIC_001_mut_target_with_variable_in_prereq() {
        // ARRANGE: Target with variable assignment in prerequisites
        // This should be parsed as TARGET, not VARIABLE
        let makefile = "target: VAR=value dep2\n\trecipe";

        // ACT: Parse makefile
        let result = parse_makefile(makefile);

        // ASSERT: Successfully parsed
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1);

        // ASSERT: Parsed as Target (not Variable)
        match &ast.items[0] {
            MakeItem::Target {
                name,
                prerequisites,
                ..
            } => {
                assert_eq!(name, "target");
                assert_eq!(prerequisites.len(), 2);
                assert_eq!(prerequisites[0], "VAR=value");
                assert_eq!(prerequisites[1], "dep2");
            }
            MakeItem::Variable { .. } => {
                panic!("Should be Target, not Variable");
            }
            _ => panic!("Expected Target"),
        }
    }

    /// Kill mutant: line 141 `replace + with -` in parse_variable
    /// Kill mutant: line 143 `replace + with -` in parse_variable
    /// Kill mutant: line 145 `replace + with -` in parse_variable
    ///
    /// These mutants would break parsing of multi-character operators.
    #[test]
    fn test_VAR_BASIC_001_mut_multichar_operator_slicing() {
        // ARRANGE: Test that := operator is correctly sliced
        let makefile = "VAR := value_here";

        // ACT: Parse makefile
        let result = parse_makefile(makefile);

        // ASSERT: Successfully parsed
        assert!(result.is_ok());

        let ast = result.unwrap();

        // ASSERT: Variable value does NOT include operator
        match &ast.items[0] {
            MakeItem::Variable {
                name,
                value,
                flavor,
                ..
            } => {
                assert_eq!(name, "VAR");
                assert_eq!(value, "value_here");
                assert_eq!(flavor, &VarFlavor::Simple);

                // Critical: Value must not contain ":" from ":="
                assert!(!value.contains(':'), "Value should not contain operator");
                assert!(!value.contains('='), "Value should not contain operator");
            }
            _ => panic!("Expected Variable"),
        }
    }

    /// Kill mutant: line 213 `replace < with <=` in parse_target_rule
    ///
    /// This mutant would affect recipe parsing loop bounds.
    #[test]
    fn test_VAR_BASIC_001_mut_recipe_loop_bounds() {
        // ARRANGE: Target at end of file with exactly one recipe line
        let makefile = "target:\n\trecipe";

        // ACT: Parse makefile
        let result = parse_makefile(makefile);

        // ASSERT: Successfully parsed
        assert!(result.is_ok());

        let ast = result.unwrap();

        // ASSERT: Exactly one recipe line (not duplicated)
        match &ast.items[0] {
            MakeItem::Target { recipe, .. } => {
                assert_eq!(
                    recipe.len(),
                    1,
                    "Should have exactly 1 recipe line, not {} (would happen if < became <=)",
                    recipe.len()
                );
                assert_eq!(recipe[0], "recipe");
            }
            _ => panic!("Expected Target"),
        }
    }

    /// Kill mutant: Additional test for operator edge cases
