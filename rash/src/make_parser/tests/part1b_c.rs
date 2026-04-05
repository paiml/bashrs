#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

#[test]
fn test_VAR_FLAVOR_004_mut_append_not_missed() {
    // ARRANGE: Test multiple += assignments
    let makefile = r#"
V1 += first
V2 += second
V3 += third
"#;

    // ACT: Parse
    let result = parse_makefile(makefile);

    // ASSERT: All three must be parsed as Append
    assert!(result.is_ok());
    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 3, "Must parse all 3 variables");

    for (idx, item) in ast.items.iter().enumerate() {
        match item {
            MakeItem::Variable { flavor, .. } => {
                assert_eq!(
                    *flavor,
                    VarFlavor::Append,
                    "Variable {} must be Append flavor",
                    idx
                );
            }
            _ => panic!("All items must be Variables"),
        }
    }
}

// Target: parser.rs:152-153 - Not confused with other operators containing '+'
#[test]
fn test_VAR_FLAVOR_004_mut_not_confused_with_plus() {
    // ARRANGE: Test that += is not confused with values containing '+'
    let append = "FLAGS += -O2";
    let recursive_with_plus = "VALUE = 1+2"; // Contains '+' but not '+='

    // ACT: Parse both
    let append_result = parse_makefile(append);
    let recursive_result = parse_makefile(recursive_with_plus);

    // ASSERT: Different flavors
    assert!(append_result.is_ok());
    assert!(recursive_result.is_ok());

    let append_ast = append_result.unwrap();
    let recursive_ast = recursive_result.unwrap();

    match (&append_ast.items[0], &recursive_ast.items[0]) {
        (MakeItem::Variable { flavor: f1, .. }, MakeItem::Variable { flavor: f2, .. }) => {
            // First must be Append
            assert_eq!(*f1, VarFlavor::Append, "FLAGS must be Append");

            // Second must be Recursive (not Append!)
            assert_eq!(*f2, VarFlavor::Recursive, "VALUE must be Recursive");
            assert_ne!(*f1, *f2, "Flavors must be different");
        }
        _ => panic!("Expected Variable items"),
    }
}

// Target: parser.rs:152-153 - Correct flavor enum variant assignment
#[test]
fn test_VAR_FLAVOR_004_mut_correct_flavor_enum_variant() {
    // ARRANGE: Makefile with += operator
    let makefile = "TARGET += value";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Variable { flavor, .. } => {
            // Must be EXACTLY VarFlavor::Append
            assert!(
                matches!(flavor, VarFlavor::Append),
                "Must be VarFlavor::Append"
            );

            // Must NOT be any other variant
            assert!(
                !matches!(flavor, VarFlavor::Recursive),
                "Must NOT be Recursive"
            );
            assert!(!matches!(flavor, VarFlavor::Simple), "Must NOT be Simple");
            assert!(
                !matches!(flavor, VarFlavor::Conditional),
                "Must NOT be Conditional"
            );
            assert!(!matches!(flavor, VarFlavor::Shell), "Must NOT be Shell");
        }
        _ => panic!("Expected Variable"),
    }
}

// ============================================================================
// Sprint 36: VAR-FLAVOR-001 - Recursive Assignment (=)
// ============================================================================
// Target: parser.rs:116 (detection), parser.rs:156-157 (parsing)
// Verifies: = operator maps to VarFlavor::Recursive

// Phase 1 - RED: Unit tests for = recursive assignment

#[test]
fn test_VAR_FLAVOR_001_basic_recursive_assignment() {
    // ARRANGE: Makefile with = recursive assignment operator
    let makefile = "CC = gcc";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Parser should handle = recursive assignment
    assert!(
        result.is_ok(),
        "Parser should handle = recursive assignment"
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should parse one variable");

    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "CC", "Variable name should be CC");
            assert_eq!(value, "gcc", "Value should be gcc");
            assert_eq!(
                *flavor,
                VarFlavor::Recursive,
                "Should detect = as Recursive"
            );
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_VAR_FLAVOR_001_recursive_with_spaces() {
    // ARRANGE: Makefile with spaces around = operator
    let makefile = "CFLAGS   =   -Wall -O2   ";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Parser should handle spaces around = operator
    assert!(
        result.is_ok(),
        "Parser should handle spaces around = operator"
    );

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
            assert_eq!(value, "-Wall -O2", "Value should be trimmed");
            assert_eq!(*flavor, VarFlavor::Recursive, "Should be Recursive flavor");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_VAR_FLAVOR_001_recursive_empty_value() {
    // ARRANGE: Makefile with = operator and empty value
    let makefile = "EMPTY = ";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Parser should handle empty values with = operator
    assert!(
        result.is_ok(),
        "Parser should handle empty values with = operator"
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should parse one variable");

    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "EMPTY", "Variable name should be EMPTY");
            assert_eq!(value, "", "Value should be empty string");
            assert_eq!(*flavor, VarFlavor::Recursive, "Should be Recursive flavor");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_VAR_FLAVOR_001_recursive_vs_other_flavors() {
    // ARRANGE: Makefiles with different operators
    let simple = "VAR := value"; // Simple assignment
    let recursive = "VAR = value"; // Recursive assignment

    // ACT: Parse both makefiles
    let simple_result = parse_makefile(simple);
    let recursive_result = parse_makefile(recursive);

    // ASSERT: Different flavors
    assert!(simple_result.is_ok());
    assert!(recursive_result.is_ok());

    let simple_ast = simple_result.unwrap();
    let recursive_ast = recursive_result.unwrap();

    match (&simple_ast.items[0], &recursive_ast.items[0]) {
        (MakeItem::Variable { flavor: f1, .. }, MakeItem::Variable { flavor: f2, .. }) => {
            // First must be Simple
            assert_eq!(*f1, VarFlavor::Simple, "VAR must be Simple");

            // Second must be Recursive (not Simple!)
            assert_eq!(*f2, VarFlavor::Recursive, "VAR must be Recursive");
            assert_ne!(*f1, *f2, "Flavors must be different");
        }
        _ => panic!("Expected Variable items"),
    }
}

// Phase 4 - PROPERTY TESTING: Property tests for = recursive assignment

#[cfg(test)]
mod property_tests_var_flavor_001 {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_VAR_FLAVOR_001_prop_recursive_always_parses(
            varname in "[A-Z][A-Z0-9_]{0,20}",
            value in "[a-zA-Z0-9_./+ -]{0,50}"
        ) {
            let makefile = format!("{} = {}", varname, value);
            let result = parse_makefile(&makefile);

            prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);
            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            if let MakeItem::Variable { name, value: val, flavor, .. } = &ast.items[0] {
                prop_assert_eq!(name, &varname);
                prop_assert_eq!(val, &value.trim());
                prop_assert_eq!(flavor, &VarFlavor::Recursive);
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }

        #[test]
        fn test_VAR_FLAVOR_001_prop_recursive_is_deterministic(
            varname in "[A-Z]{1,15}",
            value in "[a-z0-9]{0,30}"
        ) {
            let makefile = format!("{} = {}", varname, value);

            // Parse twice
            let result1 = parse_makefile(&makefile);
            let result2 = parse_makefile(&makefile);

            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());

            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();

            // Must produce identical results
            match (&ast1.items[0], &ast2.items[0]) {
                (
                    MakeItem::Variable { name: n1, value: v1, flavor: f1, .. },
                    MakeItem::Variable { name: n2, value: v2, flavor: f2, .. }
                ) => {
                    prop_assert_eq!(n1, n2);
                    prop_assert_eq!(v1, v2);
                    prop_assert_eq!(f1, f2);
                    prop_assert_eq!(f1, &VarFlavor::Recursive);
                }
                _ => return Err(TestCaseError::fail("Expected Variable items")),
            }
        }

        #[test]
        fn test_VAR_FLAVOR_001_prop_not_confused_with_other_operators(
            varname in "[A-Z]{1,10}",
            value in "[a-z]{0,20}"
        ) {
            // Test that = is NOT parsed as :=, ?=, +=, or !=
            let makefile = format!("{} = {}", varname, value);
            let result = parse_makefile(&makefile);

            prop_assert!(result.is_ok());
            let ast = result.unwrap();

            if let MakeItem::Variable { flavor, .. } = &ast.items[0] {
                // Must be Recursive, NOT other flavors
                prop_assert_eq!(flavor, &VarFlavor::Recursive);
                prop_assert_ne!(flavor, &VarFlavor::Simple);
                prop_assert_ne!(flavor, &VarFlavor::Conditional);
                prop_assert_ne!(flavor, &VarFlavor::Append);
                prop_assert_ne!(flavor, &VarFlavor::Shell);
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }

        #[test]
        fn test_VAR_FLAVOR_001_prop_handles_various_values(
            varname in "[A-Z_][A-Z0-9_]*",
            value in ".*"
        ) {
            // Filter out values containing special operators to avoid confusion
            if value.contains(":=") || value.contains("?=") ||
               value.contains("+=") || value.contains("!=") {
                return Ok(());
            }

            let makefile = format!("{} = {}", varname, value);
            let result = parse_makefile(&makefile);

            prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);
            let ast = result.unwrap();

            if let MakeItem::Variable { name, value: val, flavor, .. } = &ast.items[0] {
                prop_assert_eq!(name, &varname);
                prop_assert_eq!(val, &value.trim());
                prop_assert_eq!(flavor, &VarFlavor::Recursive);
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }

        #[test]
        fn test_VAR_FLAVOR_001_prop_handles_special_chars_in_values(
            varname in "[A-Z]{3,10}",
            value in "[a-zA-Z0-9@#$%^&*()_+\\-=\\[\\]{}|;:',.<>/?~ ]{0,40}"
        ) {
            // Avoid values with special operators
            if value.contains(":=") || value.contains("?=") ||
               value.contains("+=") || value.contains("!=") {
                return Ok(());
            }

            let makefile = format!("{} = {}", varname, value);
            let result = parse_makefile(&makefile);

            prop_assert!(result.is_ok());
            let ast = result.unwrap();

            if let MakeItem::Variable { flavor, .. } = &ast.items[0] {
                prop_assert_eq!(flavor, &VarFlavor::Recursive);
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }
    }
}
