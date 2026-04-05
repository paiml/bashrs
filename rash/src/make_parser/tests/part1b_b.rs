#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

    #[test]
    fn test_VAR_FLAVOR_003_mut_not_confused_with_question_mark() {
        // ?= in variable assignment vs ? in other contexts
        let makefile = "CONFIG ?= default\ntarget:\n\techo test?";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 2);

        // First item must be Variable with Conditional flavor
        match &ast.items[0] {
            MakeItem::Variable { name, flavor, .. } => {
                assert_eq!(name, "CONFIG");
                assert_eq!(*flavor, VarFlavor::Conditional);
            }
            _ => panic!("Expected Variable"),
        }

        // Second item must be Target
        match &ast.items[1] {
            MakeItem::Target { name, .. } => {
                assert_eq!(name, "target");
            }
            _ => panic!("Expected Target"),
        }
    }

    /// Kill mutant: line 150 VarFlavor enum variant
    ///
    /// Ensure Conditional variant is correctly used (not Simple, Append, etc).
    #[test]
    fn test_VAR_FLAVOR_003_mut_correct_flavor_enum_variant() {
        // Target: line 150 where VarFlavor::Conditional is set
        // Kill mutants that use wrong enum variant
        let makefile = "TEST ?= value";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Variable { flavor, .. } => {
                // Must be exactly Conditional
                assert!(
                    matches!(flavor, VarFlavor::Conditional),
                    "Must be VarFlavor::Conditional"
                );

                // Must NOT be any other variant
                assert!(
                    !matches!(flavor, VarFlavor::Recursive),
                    "Must NOT be Recursive"
                );
                assert!(!matches!(flavor, VarFlavor::Simple), "Must NOT be Simple");
                assert!(!matches!(flavor, VarFlavor::Append), "Must NOT be Append");
                assert!(!matches!(flavor, VarFlavor::Shell), "Must NOT be Shell");
            }
            _ => panic!("Expected Variable"),
        }
    }
}

// ============================================================================
// Sprint 35: VAR-FLAVOR-004 - Append Assignment (+=)
// ============================================================================
//
// Task: VAR-FLAVOR-004 - Document append assignment (+=)
// Status: Verification task (parser already implements += at lines 111, 153)
//
// Implementation:
// - Line 111: is_variable_assignment() detects += operator
// - Line 153: parse_variable() maps to VarFlavor::Append
//
// Test Coverage:
// - Unit tests: Basic +=, spaces, empty values, operator distinction
// - Property tests: Various append scenarios (Phase 4)
// - Mutation-killing tests: Target lines 111, 153 (Phase 5)
// ============================================================================

// ----------------------------------------------------------------------------
// Phase 1: RED - Unit Tests for += Append Assignment
// ----------------------------------------------------------------------------

#[test]
fn test_VAR_FLAVOR_004_basic_append_assignment() {
    // ARRANGE: Makefile with += append operator
    let makefile = "CFLAGS += -O2";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Parser should handle += append assignment
    assert!(result.is_ok(), "Parser should handle += append assignment");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should parse one variable");

    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "CFLAGS", "Variable name should be CFLAGS");
            assert_eq!(value, "-O2", "Value should be -O2");
            assert_eq!(*flavor, VarFlavor::Append, "Should detect += as Append");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_VAR_FLAVOR_004_append_with_spaces() {
    // ARRANGE: Makefile with += and spaces around operator
    let makefile = "LDFLAGS  +=  -lm -lpthread";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Should handle spaces around +=
    assert!(result.is_ok(), "Parser should handle spaces around +=");

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "LDFLAGS");
            assert_eq!(value, "-lm -lpthread", "Should preserve value with spaces");
            assert_eq!(*flavor, VarFlavor::Append);
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_VAR_FLAVOR_004_append_empty_value() {
    // ARRANGE: Makefile with += and empty value
    let makefile = "OPTS +=";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Should handle empty append value
    assert!(result.is_ok(), "Parser should handle empty append value");

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "OPTS");
            assert_eq!(value, "", "Empty value should parse as empty string");
            assert_eq!(*flavor, VarFlavor::Append);
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_VAR_FLAVOR_004_append_vs_other_flavors() {
    // ARRANGE: Makefile with all 5 variable flavors
    let makefile = r#"
V1 = recursive
V2 := simple
V3 ?= conditional
V4 += append
V5 != echo shell
"#;

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Should distinguish += from other operators
    assert!(result.is_ok(), "Parser should handle all 5 flavors");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 5, "Should parse 5 variables");

    // Check each flavor is correct
    let flavors: Vec<VarFlavor> = ast
        .items
        .iter()
        .filter_map(|item| match item {
            MakeItem::Variable { flavor, .. } => Some(flavor.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(flavors.len(), 5);
    assert!(matches!(flavors[0], VarFlavor::Recursive));
    assert!(matches!(flavors[1], VarFlavor::Simple));
    assert!(matches!(flavors[2], VarFlavor::Conditional));
    assert!(
        matches!(flavors[3], VarFlavor::Append),
        "Fourth variable should be Append"
    );
    assert!(matches!(flavors[4], VarFlavor::Shell));

    // Specifically verify V4 is Append
    match &ast.items[3] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "V4");
            assert_eq!(value, "append");
            assert_eq!(*flavor, VarFlavor::Append, "V4 should have Append flavor");
        }
        _ => panic!("Expected Variable at index 3"),
    }
}

// ----------------------------------------------------------------------------
// Phase 4: PROPERTY TESTING - Property Tests for += Append Assignment
// ----------------------------------------------------------------------------

#[cfg(test)]
mod property_tests_var_flavor_004 {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_VAR_FLAVOR_004_prop_append_always_parses(
            varname in "[A-Z][A-Z0-9_]{0,20}",
            value in "[a-zA-Z0-9_./+ -]{0,50}"
        ) {
            // ARRANGE: Generate makefile with += operator
            let makefile = format!("{} += {}", varname, value);

            // ACT: Parse the makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Should always parse successfully
            prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1, "Should have exactly 1 item");

            if let MakeItem::Variable { name, value: val, flavor, .. } = &ast.items[0] {
                prop_assert_eq!(name, &varname, "Name should match");
                prop_assert_eq!(val, &value.trim(), "Value should match (trimmed)");
                prop_assert_eq!(flavor, &VarFlavor::Append, "Flavor should be Append");
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }

        #[test]
        fn test_VAR_FLAVOR_004_prop_parsing_is_deterministic(
            varname in "[A-Z][A-Z_]{0,15}",
            value in "[a-zA-Z0-9 ]{0,30}"
        ) {
            // ARRANGE: Generate makefile with += operator
            let makefile = format!("{} += {}", varname, value);

            // ACT: Parse twice
            let result1 = parse_makefile(&makefile);
            let result2 = parse_makefile(&makefile);

            // ASSERT: Both should succeed
            prop_assert!(result1.is_ok() && result2.is_ok());

            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();

            // Extract variable items
            let var1 = &ast1.items[0];
            let var2 = &ast2.items[0];

            // Should be identical
            if let (
                MakeItem::Variable { name: n1, value: v1, flavor: f1, .. },
                MakeItem::Variable { name: n2, value: v2, flavor: f2, .. }
            ) = (var1, var2) {
                prop_assert_eq!(n1, n2, "Names should be identical");
                prop_assert_eq!(v1, v2, "Values should be identical");
                prop_assert_eq!(f1, f2, "Flavors should be identical");
                prop_assert!(matches!(f1, VarFlavor::Append));
            } else {
                return Err(TestCaseError::fail("Expected Variable items"));
            }
        }

        #[test]
        fn test_VAR_FLAVOR_004_prop_operator_not_confused(
            varname in "[A-Z][A-Z_]{0,10}",
            value in "[a-z]{1,20}"
        ) {
            // ARRANGE: Test that += is not confused with other operators
            let append_makefile = format!("{} += {}", varname, value);
            let recursive_makefile = format!("{} = {}", varname, value);
            let simple_makefile = format!("{} := {}", varname, value);

            // ACT: Parse all three
            let append_result = parse_makefile(&append_makefile);
            let recursive_result = parse_makefile(&recursive_makefile);
            let simple_result = parse_makefile(&simple_makefile);

            // ASSERT: All should parse
            prop_assert!(append_result.is_ok());
            prop_assert!(recursive_result.is_ok());
            prop_assert!(simple_result.is_ok());

            let append_ast = append_result.unwrap();
            let recursive_ast = recursive_result.unwrap();
            let simple_ast = simple_result.unwrap();

            // Extract flavors
            if let (
                MakeItem::Variable { flavor: f_append, .. },
                MakeItem::Variable { flavor: f_recursive, .. },
                MakeItem::Variable { flavor: f_simple, .. }
            ) = (&append_ast.items[0], &recursive_ast.items[0], &simple_ast.items[0]) {
                // Each should have correct flavor
                prop_assert!(matches!(f_append, VarFlavor::Append), "Should be Append");
                prop_assert!(matches!(f_recursive, VarFlavor::Recursive), "Should be Recursive");
                prop_assert!(matches!(f_simple, VarFlavor::Simple), "Should be Simple");

                // They should all be different
                prop_assert_ne!(f_append, f_recursive);
                prop_assert_ne!(f_append, f_simple);
            } else {
                return Err(TestCaseError::fail("Expected Variable items"));
            }
        }

        #[test]
        fn test_VAR_FLAVOR_004_prop_values_flexible(
            varname in "[A-Z]{2,10}",
            value in "[-a-zA-Z0-9_./ ]{0,40}"
        ) {
            // ARRANGE: Test flexible value formats
            let makefile = format!("{} += {}", varname, value);

            // ACT: Parse
            let result = parse_makefile(&makefile);

            // ASSERT: Should always parse
            prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);

            let ast = result.unwrap();
            if let MakeItem::Variable { name, value: val, flavor, .. } = &ast.items[0] {
                prop_assert_eq!(name, &varname);
                prop_assert_eq!(val, &value.trim());
                prop_assert_eq!(flavor, &VarFlavor::Append);

                // Value can be empty or non-empty
                if value.trim().is_empty() {
                    prop_assert_eq!(val, "");
                } else {
                    prop_assert!(!val.is_empty());
                }
            } else {
                return Err(TestCaseError::fail("Expected Variable"));
            }
        }

        #[test]
        fn test_VAR_FLAVOR_004_prop_special_values(
            varname in "[A-Z]{2,8}",
            flags in prop::collection::vec("-[a-zA-Z][a-zA-Z0-9]*", 0..5)
        ) {
            // ARRANGE: Test compiler flag-like values (common use case for +=)
            let value = flags.join(" ");
            let makefile = format!("{} += {}", varname, value);

            // ACT: Parse
            let result = parse_makefile(&makefile);

            // ASSERT: Should parse
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            if let MakeItem::Variable { name, value: val, flavor, .. } = &ast.items[0] {
                prop_assert_eq!(name, &varname);
                prop_assert_eq!(val, &value.trim());
                prop_assert_eq!(flavor, &VarFlavor::Append);

                // If we had flags, verify they're preserved
                if !flags.is_empty() {
                    prop_assert!(!val.is_empty());
                    for flag in &flags {
                        prop_assert!(val.contains(flag), "Value should contain flag: {}", flag);
                    }
                }
            } else {
                return Err(TestCaseError::fail("Expected Variable"));
            }
        }
    }
}

// ----------------------------------------------------------------------------
// Phase 5: MUTATION TESTING - Mutation-Killing Tests for += Append Assignment
// ----------------------------------------------------------------------------

// Target: parser.rs:111 - is_variable_assignment() contains("+=") check
#[test]
fn test_VAR_FLAVOR_004_mut_operator_detection() {
    // ARRANGE: Makefile with += operator
    let makefile = "CFLAGS += -O2";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Must detect += as variable assignment
    assert!(result.is_ok(), "Must parse += as variable assignment");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Must have exactly one item");

    // Must be a Variable (not a Target or Comment)
    match &ast.items[0] {
        MakeItem::Variable { name, flavor, .. } => {
            assert_eq!(name, "CFLAGS");
            assert_eq!(*flavor, VarFlavor::Append, "Must be Append flavor");
        }
        _ => panic!("Must be parsed as Variable, not other type"),
    }
}

// Target: parser.rs:152-153 - parse_variable() slicing for "+=" operator
#[test]
fn test_VAR_FLAVOR_004_mut_operator_slicing() {
    // ARRANGE: Makefile with += and value with special chars
    let makefile = "LDFLAGS += -lm -lpthread";

    // ACT: Parse
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
            assert_eq!(name, "LDFLAGS");
            assert_eq!(*flavor, VarFlavor::Append);

            // CRITICAL: Value must NOT include "+" or "="
            // This tests the slicing at line 153: &trimmed[pos + 2..]
            assert_eq!(value, "-lm -lpthread");
            assert!(!value.contains('+'), "Value should not contain '+'");
            assert!(!value.contains('='), "Value should not contain '='");
            assert!(value.starts_with('-'), "Value should start with -");
        }
        _ => panic!("Expected Variable"),
    }
}

// Target: parser.rs:152 - Append assignment not missed
