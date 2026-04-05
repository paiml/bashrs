#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

    #[test]
    fn test_RULE_SYNTAX_002_mut_prerequisite_count_exact() {
        // Target: line 203-206 collection logic
        // Kill mutants that incorrectly count prerequisites
        let test_cases = vec![
            ("target: a\n\techo", 1),
            ("target: a b\n\techo", 2),
            ("target: a b c\n\techo", 3),
            ("target: a b c d e\n\techo", 5),
            ("target: a b c d e f g h i j\n\techo", 10),
        ];

        for (makefile, expected_count) in test_cases {
            let result = parse_makefile(makefile);
            assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { prerequisites, .. } => {
                    assert_eq!(
                        prerequisites.len(),
                        expected_count,
                        "Prerequisite count must be exact for: {}",
                        makefile
                    );
                }
                _ => panic!("Expected Target"),
            }
        }
    }

    #[test]
    fn test_RULE_SYNTAX_002_mut_empty_prerequisites_handling() {
        // Target: line 203 - parts[1] which could be empty
        // Kill mutants that don't handle empty prerequisite lists
        let test_cases = vec![
            "target:\n\techo",       // Empty after colon
            "target: \n\techo",      // Just space after colon
            "target:  \t  \n\techo", // Multiple whitespace, no prerequisites
        ];

        for makefile in test_cases {
            let result = parse_makefile(makefile);
            assert!(
                result.is_ok(),
                "Must handle empty prerequisites for: {}",
                makefile
            );

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { prerequisites, .. } => {
                    assert_eq!(
                        prerequisites.len(),
                        0,
                        "Empty prerequisites should result in empty vec for: {}",
                        makefile
                    );
                }
                _ => panic!("Expected Target"),
            }
        }
    }

    #[test]
    fn test_RULE_SYNTAX_002_mut_prerequisite_string_conversion() {
        // Target: line 205 - .to_string() conversion
        // Kill mutants that break string ownership
        let makefile = "target: prereq1 prereq2\n\techo";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Target {
                prerequisites,
                name,
                ..
            } => {
                // Verify independent ownership (not references)
                assert_eq!(prerequisites[0], "prereq1");
                assert_eq!(prerequisites[1], "prereq2");

                // Verify no lifetime issues by accessing after parsing
                let prereq1_clone = prerequisites[0].clone();
                let prereq2_clone = prerequisites[1].clone();
                assert_eq!(prereq1_clone, "prereq1");
                assert_eq!(prereq2_clone, "prereq2");

                // Ensure target name independent from prerequisites
                assert_eq!(name, "target");
                assert_ne!(&prerequisites[0], name);
            }
            _ => panic!("Expected Target"),
        }
    }

    #[test]
    fn test_RULE_SYNTAX_002_mut_prerequisite_order_matters() {
        // Target: line 203-206 - collect() preserves order
        // Kill mutants that break prerequisite ordering
        let makefile = "link: z.o y.o x.o w.o v.o\n\tgcc -o app";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Target { prerequisites, .. } => {
                assert_eq!(prerequisites.len(), 5);
                // Order MUST be preserved (important for linking order)
                assert_eq!(prerequisites[0], "z.o", "First must be z.o");
                assert_eq!(prerequisites[1], "y.o", "Second must be y.o");
                assert_eq!(prerequisites[2], "x.o", "Third must be x.o");
                assert_eq!(prerequisites[3], "w.o", "Fourth must be w.o");
                assert_eq!(prerequisites[4], "v.o", "Fifth must be v.o");

                // Verify not sorted
                assert_ne!(prerequisites[0], "v.o", "Must NOT be sorted");
            }
            _ => panic!("Expected Target"),
        }
    }
}

// ============================================================================
// VAR-FLAVOR-003: Conditional Assignment ?= Tests
// Task: Verify parser correctly handles ?= (conditional) assignment operator
// ============================================================================

#[cfg(test)]
mod var_flavor_003_tests {
    use crate::make_parser::{ast::VarFlavor, parse_makefile, MakeItem};

    // Unit Tests
    #[test]
    fn test_VAR_FLAVOR_003_basic_conditional_assignment() {
        let makefile = "PREFIX ?= /usr/local";
        let result = parse_makefile(makefile);
        assert!(
            result.is_ok(),
            "Parser should handle ?= conditional assignment"
        );

        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1);

        match &ast.items[0] {
            MakeItem::Variable {
                name,
                value,
                flavor,
                ..
            } => {
                assert_eq!(name, "PREFIX");
                assert_eq!(value, "/usr/local");
                assert_eq!(
                    *flavor,
                    VarFlavor::Conditional,
                    "Should detect ?= as Conditional"
                );
            }
            other => panic!("Expected Variable, got {:?}", other),
        }
    }

    #[test]
    fn test_VAR_FLAVOR_003_conditional_with_spaces() {
        let makefile = "CC ?= gcc -Wall -O2";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Variable {
                name,
                value,
                flavor,
                ..
            } => {
                assert_eq!(name, "CC");
                assert_eq!(value, "gcc -Wall -O2");
                assert_eq!(*flavor, VarFlavor::Conditional);
            }
            _ => panic!("Expected Variable"),
        }
    }

    #[test]
    fn test_VAR_FLAVOR_003_conditional_empty_value() {
        let makefile = "EMPTY ?=";
        let result = parse_makefile(makefile);
        assert!(
            result.is_ok(),
            "Conditional assignment can have empty value"
        );

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Variable {
                name,
                value,
                flavor,
                ..
            } => {
                assert_eq!(name, "EMPTY");
                assert_eq!(value, "");
                assert_eq!(*flavor, VarFlavor::Conditional);
            }
            _ => panic!("Expected Variable"),
        }
    }

    #[test]
    fn test_VAR_FLAVOR_003_conditional_vs_other_flavors() {
        // Test that ?= is correctly distinguished from other operators
        let test_cases = vec![
            ("VAR = val", VarFlavor::Recursive),
            ("VAR := val", VarFlavor::Simple),
            ("VAR ?= val", VarFlavor::Conditional), // The focus of this sprint
            ("VAR += val", VarFlavor::Append),
            ("VAR != echo val", VarFlavor::Shell),
        ];

        for (makefile, expected_flavor) in test_cases {
            let result = parse_makefile(makefile);
            assert!(result.is_ok(), "Failed to parse: {}", makefile);

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Variable { name, flavor, .. } => {
                    assert_eq!(name, "VAR");
                    assert_eq!(*flavor, expected_flavor, "Wrong flavor for: {}", makefile);
                }
                _ => panic!("Expected Variable for: {}", makefile),
            }
        }
    }
}

#[cfg(test)]
mod var_flavor_003_property_tests {
    use crate::make_parser::{ast::VarFlavor, parse_makefile, MakeItem};
    use proptest::prelude::*;

    proptest! {
        /// Property: ?= conditional assignments always parse successfully
        ///
        /// This test generates random variable names and values to ensure
        /// the parser handles conditional assignment across diverse inputs.
        #[test]
        fn test_VAR_FLAVOR_003_prop_conditional_always_parses(
            varname in "[A-Z][A-Z0-9_]{0,20}",
            value in "[a-zA-Z0-9_./+ -]{0,50}"
        ) {
            // ARRANGE: Generate valid conditional assignment
            let makefile = format!("{} ?= {}", varname, value);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Parsing succeeds
            prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            // ASSERT: Correct flavor detected
            if let MakeItem::Variable { name, value: val, flavor, .. } = &ast.items[0] {
                prop_assert_eq!(name, &varname);
                prop_assert_eq!(val, &value.trim());
                prop_assert_eq!(flavor, &VarFlavor::Conditional, "Must detect ?= as Conditional");
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }

        /// Property: ?= parsing is deterministic
        ///
        /// Verifies that parsing the same conditional assignment twice produces identical results.
        #[test]
        fn test_VAR_FLAVOR_003_prop_parsing_is_deterministic(
            varname in "[A-Z]{1,15}",
            value in "[a-z0-9 ]{1,40}"
        ) {
            let makefile = format!("{} ?= {}", varname, value);

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

        /// Property: ?= is correctly identified among other operators
        ///
        /// Tests that ?= is not confused with ?, =, or other operators.
        #[test]
        fn test_VAR_FLAVOR_003_prop_operator_not_confused(
            varname in "[A-Z]{1,10}",
            value in "[a-z]{1,20}"
        ) {
            // Test ?= specifically (not := or += which also contain =)
            let makefile = format!("{} ?= {}", varname, value);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            if let MakeItem::Variable { flavor, .. } = &ast.items[0] {
                // Must be Conditional, not any other flavor
                prop_assert_eq!(flavor, &VarFlavor::Conditional);
                prop_assert_ne!(flavor, &VarFlavor::Recursive);
                prop_assert_ne!(flavor, &VarFlavor::Simple);
                prop_assert_ne!(flavor, &VarFlavor::Append);
                prop_assert_ne!(flavor, &VarFlavor::Shell);
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }

        /// Property: ?= handles empty and whitespace values correctly
        ///
        /// Verifies that conditional assignment works with various value patterns.
        #[test]
        fn test_VAR_FLAVOR_003_prop_values_flexible(
            varname in "[A-Z]{1,12}",
            value in "[ a-z0-9-]*"  // Can be empty, can have spaces
        ) {
            let makefile = format!("{} ?= {}", varname, value);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            if let MakeItem::Variable { name, value: val, flavor, .. } = &ast.items[0] {
                prop_assert_eq!(name, &varname);
                prop_assert_eq!(val, &value.trim());  // Value gets trimmed
                prop_assert_eq!(flavor, &VarFlavor::Conditional);
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }

        /// Property: ?= works with paths and special characters in values
        ///
        /// Tests that conditional assignment handles filesystem paths and common patterns.
        #[test]
        fn test_VAR_FLAVOR_003_prop_special_values(
            varname in "[A-Z]{1,10}",
            path_parts in prop::collection::vec("[a-z]{1,8}", 1..4)
        ) {
            // Create value like /usr/local/bin or src/include/main
            let value = path_parts.join("/");
            let makefile = format!("{} ?= {}", varname, value);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok(), "Failed to parse path value: {}", makefile);

            let ast = result.unwrap();
            if let MakeItem::Variable { name, value: val, flavor, .. } = &ast.items[0] {
                prop_assert_eq!(name, &varname);
                prop_assert_eq!(val, &value);
                prop_assert_eq!(flavor, &VarFlavor::Conditional);
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }
    }
}

#[cfg(test)]
mod var_flavor_003_mutation_killing_tests {
    use crate::make_parser::{ast::VarFlavor, parse_makefile, MakeItem};

    /// Kill mutant: line 110 - `replace || with &&` in is_variable_assignment
    ///
    /// This mutant would break detection of ?= operator.
    #[test]
    fn test_VAR_FLAVOR_003_mut_operator_detection() {
        // Target: line 110 where ?= is checked
        // Kill mutants that break ?= detection in is_variable_assignment
        let test_cases = vec![
            ("VAR?=value", true),   // No spaces
            ("VAR ?=value", true),  // Space before
            ("VAR?= value", true),  // Space after
            ("VAR ?= value", true), // Spaces both sides
        ];

        for (makefile, should_be_conditional) in test_cases {
            let result = parse_makefile(makefile);
            assert!(result.is_ok(), "Failed to parse: {}", makefile);

            let ast = result.unwrap();
            if should_be_conditional {
                match &ast.items[0] {
                    MakeItem::Variable { flavor, .. } => {
                        assert_eq!(
                            *flavor,
                            VarFlavor::Conditional,
                            "?= must be detected as Conditional for: {}",
                            makefile
                        );
                    }
                    _ => panic!("Expected Variable for: {}", makefile),
                }
            }
        }
    }

    /// Kill mutant: line 150 - `replace + with -` or `replace 2 with 1` in parse_variable
    ///
    /// This mutant would break string slicing for ?= operator.
    #[test]
    fn test_VAR_FLAVOR_003_mut_operator_slicing() {
        // Target: line 150 where "?=" is found and sliced
        // pos + 2 must correctly skip past "?=" to get value
        let makefile = "PREFIX ?= /usr/local/bin";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Variable {
                name,
                value,
                flavor,
                ..
            } => {
                assert_eq!(name, "PREFIX");
                assert_eq!(*flavor, VarFlavor::Conditional);

                // Critical: Value must NOT include "?" or "="
                assert_eq!(value, "/usr/local/bin");
                assert!(!value.contains('?'), "Value should not contain '?'");
                assert!(!value.contains('='), "Value should not contain '='");

                // Value must be clean (trimmed, no operator)
                assert!(value.starts_with('/'), "Value should start with /");
            }
            _ => panic!("Expected Variable"),
        }
    }

    /// Kill mutant: line 110 - `replace contains with !contains`
    ///
    /// This would make parser miss ?= and treat it as something else.
    #[test]
    fn test_VAR_FLAVOR_003_mut_conditional_not_missed() {
        // Target: Ensure ?= is recognized and not skipped
        let test_cases = vec![
            ("A?=1", VarFlavor::Conditional),
            ("B?=2", VarFlavor::Conditional),
            ("C?=3", VarFlavor::Conditional),
        ];

        for (input, expected_flavor) in test_cases {
            let result = parse_makefile(input);
            assert!(result.is_ok(), "Must parse: {}", input);

            let ast = result.unwrap();
            assert_eq!(ast.items.len(), 1, "Must parse exactly one variable");

            match &ast.items[0] {
                MakeItem::Variable { flavor, .. } => {
                    assert_eq!(
                        flavor, &expected_flavor,
                        "?= must be detected for: {}",
                        input
                    );
                }
                _ => panic!("Expected Variable for: {}", input),
            }
        }
    }

    /// Kill mutant: Ensure ?= doesn't get confused with ? in target names
    ///
    /// Tests edge case where ? might appear in other contexts.
