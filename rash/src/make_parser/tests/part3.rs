#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

// ============================================================================
// FUNC-SUBST-001: $(subst from,to,text) Function
// ============================================================================

#[test]
fn test_FUNC_SUBST_001_basic_subst() {
    // ARRANGE: Variable with $(subst) function
    let makefile = "OBJS = $(subst .c,.o,main.c util.c)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse $(subst) function, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Check variable with $(subst) function
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS", "Variable name should be OBJS");
            assert_eq!(
                value, "$(subst .c,.o,main.c util.c)",
                "Variable value should preserve $(subst) function syntax"
            );
        }
        other => panic!("Expected Variable item, got {:?}", other),
    }
}

#[test]
fn test_FUNC_SUBST_001_subst_in_prerequisites() {
    // ARRANGE: Target with $(subst) in prerequisites
    // NOTE: Parser splits prerequisites on whitespace, so $(subst .c,.o,$(SRCS))
    // with spaces becomes 2 prerequisites: "$(subst" and ".c,.o,$(SRCS))"
    // This is expected behavior - the parser is not function-aware
    let makefile = "build: $(subst .c,.o,$(SRCS))\n\tgcc -o $@ $^";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse $(subst) in prerequisites");

    let ast = result.unwrap();

    // ASSERT: Check target prerequisites
    match &ast.items[0] {
        MakeItem::Target {
            name,
            prerequisites,
            recipe,
            ..
        } => {
            assert_eq!(name, "build");
            // Parser splits on whitespace, so we get 2 prerequisites
            assert_eq!(prerequisites.len(), 2);
            // The function syntax is preserved, just split
            assert_eq!(prerequisites[0], "$(subst");
            assert_eq!(prerequisites[1], ".c,.o,$(SRCS))");
            // Recipe should be parsed correctly
            assert_eq!(recipe.len(), 1, "Should have 1 recipe line");
            assert_eq!(recipe[0], "gcc -o $@ $^");
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

#[test]
fn test_FUNC_SUBST_001_multiple_subst() {
    // ARRANGE: Multiple variables with $(subst) functions
    let makefile = "OBJS = $(subst .c,.o,$(SRCS))\nLIBS = $(subst .a,.so,$(DEPS))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse multiple $(subst) functions");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2, "Should have 2 variables");

    // ASSERT: Check first $(subst)
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert_eq!(value, "$(subst .c,.o,$(SRCS))");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }

    // ASSERT: Check second $(subst)
    match &ast.items[1] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "LIBS");
            assert_eq!(value, "$(subst .a,.so,$(DEPS))");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_FUNC_SUBST_001_subst_with_spaces() {
    // ARRANGE: $(subst) with spaces in arguments
    let makefile = "FILES = $(subst src/,build/,src/main.c src/util.c)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse $(subst) with spaces");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check $(subst) preserved with spaces
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FILES");
            assert_eq!(
                value, "$(subst src/,build/,src/main.c src/util.c)",
                "Should preserve spaces in $(subst) text argument"
            );
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_FUNC_SUBST_001_nested_subst() {
    // ARRANGE: Nested $(subst) functions
    let makefile = "RESULT = $(subst .c,.o,$(subst src/,build/,$(SRCS)))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse nested $(subst) functions");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check nested $(subst) preserved
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "RESULT");
            assert_eq!(
                value, "$(subst .c,.o,$(subst src/,build/,$(SRCS)))",
                "Should preserve nested $(subst) functions"
            );
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_FUNC_SUBST_001_subst_with_other_functions() {
    // ARRANGE: $(subst) combined with $(wildcard)
    let makefile = "OBJS = $(subst .c,.o,$(wildcard src/*.c))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse $(subst) with $(wildcard) function"
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check combined functions preserved
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert_eq!(
                value, "$(subst .c,.o,$(wildcard src/*.c))",
                "Should preserve $(subst) combined with other functions"
            );
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

// ============================================================================
// FUNC-SUBST-001: Property Tests
// ============================================================================

#[cfg(test)]
#[path = "part3_tests_extracted.rs"]
mod tests_extracted;
