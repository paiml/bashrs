#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

// Phase 5 - MUTATION TESTING: Mutation-killing tests for = recursive assignment

// Target: parser.rs:116 - is_variable_assignment() contains('=') check
#[test]
fn test_VAR_FLAVOR_001_mut_equals_detection() {
    // ARRANGE: Makefile with = operator
    let makefile = "CC = gcc";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Must detect = as variable assignment
    assert!(result.is_ok(), "Must parse = as variable assignment");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Must have exactly one item");

    // Must be a Variable (not a Target or Comment)
    match &ast.items[0] {
        MakeItem::Variable { name, flavor, .. } => {
            assert_eq!(name, "CC");
            assert_eq!(*flavor, VarFlavor::Recursive, "Must be Recursive flavor");
        }
        _ => panic!("Must be parsed as Variable, not other type"),
    }
}

// Target: parser.rs:156-157 - Correct string slicing for = operator
#[test]
fn test_VAR_FLAVOR_001_mut_operator_slicing() {
    // ARRANGE: Makefile with = and value after operator
    let makefile = "CFLAGS = -Wall -O2";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            // Must correctly slice at = position
            assert_eq!(name, "CFLAGS", "Name must be before =");
            assert_eq!(value, "-Wall -O2", "Value must be after =");

            // Must NOT include = in name or value
            assert!(!name.contains('='), "Name must not contain =");
            assert!(!value.starts_with('='), "Value must not start with =");
        }
        _ => panic!("Expected Variable"),
    }
}

// Target: parser.rs:156-157 - = not missed in parsing
#[test]
fn test_VAR_FLAVOR_001_mut_recursive_not_missed() {
    // ARRANGE: Makefile with = operator (basic case)
    let makefile = "VAR = value";

    // ACT: Parse
    let result = parse_makefile(makefile);

    // ASSERT: Must successfully parse (not skip the line)
    assert!(result.is_ok(), "Must parse = assignment");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Must parse one variable, not skip");

    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "VAR");
            assert_eq!(value, "value");
            assert_eq!(*flavor, VarFlavor::Recursive);
        }
        _ => panic!("Must parse as Variable"),
    }
}

// Target: parser.rs:116 - = not confused with target rule separator :
#[test]
fn test_VAR_FLAVOR_001_mut_not_confused_with_colon() {
    // ARRANGE: Two inputs - one variable, one target
    let variable = "VAR = value";
    let target = "target: prereq";

    // ACT: Parse both
    let var_result = parse_makefile(variable);
    let target_result = parse_makefile(target);

    // ASSERT: Different types
    assert!(var_result.is_ok());
    assert!(target_result.is_ok());

    let var_ast = var_result.unwrap();
    let target_ast = target_result.unwrap();

    // Variable must be Variable, not Target
    match &var_ast.items[0] {
        MakeItem::Variable { flavor, .. } => {
            assert_eq!(*flavor, VarFlavor::Recursive);
        }
        MakeItem::Target { .. } => panic!("VAR = value must be Variable, not Target"),
        _ => panic!("Unexpected item type"),
    }

    // Target must be Target, not Variable
    match &target_ast.items[0] {
        MakeItem::Target { .. } => {
            // Correct!
        }
        MakeItem::Variable { .. } => panic!("target: must be Target, not Variable"),
        _ => panic!("Unexpected item type"),
    }
}

// Target: parser.rs:156-157 - Correct flavor enum variant assignment
#[test]
fn test_VAR_FLAVOR_001_mut_correct_flavor_enum_variant() {
    // ARRANGE: Makefile with = operator
    let makefile = "TARGET = value";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Variable { flavor, .. } => {
            // Must be EXACTLY VarFlavor::Recursive
            assert!(
                matches!(flavor, VarFlavor::Recursive),
                "Must be VarFlavor::Recursive"
            );

            // Must NOT be any other variant
            assert!(!matches!(flavor, VarFlavor::Simple), "Must NOT be Simple");
            assert!(
                !matches!(flavor, VarFlavor::Conditional),
                "Must NOT be Conditional"
            );
            assert!(!matches!(flavor, VarFlavor::Append), "Must NOT be Append");
            assert!(!matches!(flavor, VarFlavor::Shell), "Must NOT be Shell");
        }
        _ => panic!("Expected Variable"),
    }
}

// ============================================================================
// Sprint 37: SYNTAX-002 - Line Continuation (\)
// ============================================================================
// Implements: Backslash line continuation for variables and recipes
// Verifies: Lines ending with \ are concatenated with next line

// Phase 1 - RED: Unit tests for line continuation

#[test]
fn test_SYNTAX_002_basic_line_continuation_in_variable() {
    // ARRANGE: Variable with backslash continuation
    let makefile = "FILES = file1.c \\\n    file2.c";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Parser should handle backslash line continuation
    assert!(result.is_ok(), "Parser should handle line continuation");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should parse one variable");

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FILES", "Variable name should be FILES");
            // Continuation should concatenate lines (whitespace normalized)
            assert_eq!(
                value, "file1.c file2.c",
                "Value should concatenate continued lines"
            );
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_SYNTAX_002_multiple_line_continuations() {
    // ARRANGE: Variable with multiple backslash continuations
    let makefile = "SOURCES = a.c \\\n    b.c \\\n    c.c";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Parser should handle multiple continuations
    assert!(
        result.is_ok(),
        "Parser should handle multiple continuations"
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should parse one variable");

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "SOURCES", "Variable name should be SOURCES");
            // All three lines should be concatenated
            assert_eq!(
                value, "a.c b.c c.c",
                "Value should concatenate all continued lines"
            );
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_SYNTAX_002_line_continuation_preserves_order() {
    // ARRANGE: Variable with line continuation
    let makefile = "ITEMS = first \\\n    second \\\n    third";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Variable { value, .. } => {
            // Order must be preserved: first, second, third
            assert_eq!(
                value, "first second third",
                "Order should be preserved in continuation"
            );
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SYNTAX_002_continuation_vs_no_continuation() {
    // ARRANGE: Two variables - one with continuation, one without
    let with_continuation = "VAR = a \\\n    b";
    let without_continuation = "VAR = a b";

    // ACT: Parse both
    let result1 = parse_makefile(with_continuation);
    let result2 = parse_makefile(without_continuation);

    // ASSERT: Both should produce same result
    assert!(result1.is_ok());
    assert!(result2.is_ok());

    let ast1 = result1.unwrap();
    let ast2 = result2.unwrap();

    match (&ast1.items[0], &ast2.items[0]) {
        (MakeItem::Variable { value: v1, .. }, MakeItem::Variable { value: v2, .. }) => {
            // Both should produce "a b"
            assert_eq!(
                v1, v2,
                "Continuation and non-continuation should be equivalent"
            );
            assert_eq!(v1, "a b");
        }
        _ => panic!("Expected Variables"),
    }
}

// ============================================================================
// Sprint 37: SYNTAX-002 - Property Tests
// ============================================================================

#[cfg(test)]
#[path = "part2_tests_syntax_002.rs"]
mod tests_extracted;
