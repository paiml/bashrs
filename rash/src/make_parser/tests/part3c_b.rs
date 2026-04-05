#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

#[test]
fn test_DEFINE_007_recursive_expansion() {
    // ARRANGE: define with recursive expansion (=)
    let makefile = r#"define RECURSIVE =
This is $(FLAVOR) expansion
endef"#;

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should handle = flavor
    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "RECURSIVE");
            assert!(
                value.contains("$(FLAVOR)"),
                "Should contain variable reference"
            );
            assert_eq!(
                *flavor,
                VarFlavor::Recursive,
                "Should be recursive expansion"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for simple expansion define (:=)
///
/// Input:
/// define SIMPLE :=
/// Expanded at $(shell date)
/// endef
#[test]
fn test_DEFINE_008_simple_expansion() {
    // ARRANGE: define with simple expansion (:=)
    let makefile = r#"define SIMPLE :=
Expanded at $(shell date)
endef"#;

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should handle := flavor
    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "SIMPLE");
            assert!(
                value.contains("shell") || value.contains("date"),
                "Should contain function call"
            );
            assert_eq!(*flavor, VarFlavor::Simple, "Should be simple expansion");
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for define with nested variable expansion
///
/// Input:
/// define COMPLEX
/// SRC = $(wildcard src/*.c)
/// OBJ = $(patsubst %.c,%.o,$(SRC))
/// endef
#[test]
fn test_DEFINE_009_nested_variables() {
    // ARRANGE: define block with nested variable assignments
    let makefile = r#"define COMPLEX
SRC = $(wildcard src/*.c)
OBJ = $(patsubst %.c,%.o,$(SRC))
endef"#;

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should preserve nested content as multi-line value
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "COMPLEX");
            assert!(value.contains("SRC ="), "Should contain SRC assignment");
            assert!(value.contains("OBJ ="), "Should contain OBJ assignment");
            assert!(
                value.contains("$(wildcard"),
                "Should contain wildcard function"
            );
            assert!(
                value.contains("$(patsubst"),
                "Should contain patsubst function"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for real-world define example
///
/// Input: Complex real-world define block from Linux kernel
#[test]
fn test_DEFINE_010_real_world_example() {
    // ARRANGE: Real-world complex define block
    let makefile = r#"define COMPILE_TEMPLATE
$(1)_OBJS := $$(patsubst %.c,%.o,$$($(1)_SOURCES))
$(1)_DEPS := $$($(1)_OBJS:.o=.d)

$(1): $$($(1)_OBJS)
	$$(CC) $$(CFLAGS) -o $$@ $$^

-include $$($(1)_DEPS)
endef"#;

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should preserve complex multi-line template
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "COMPILE_TEMPLATE");
            assert!(value.contains("_OBJS"), "Should contain OBJS assignment");
            assert!(value.contains("_DEPS"), "Should contain DEPS assignment");
            assert!(value.contains("$(CC)"), "Should contain CC variable");
            assert!(value.contains("$$@"), "Should contain automatic variable");
            assert!(
                value.contains("-include"),
                "Should contain include directive"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

// =============================================================================
// Conditional Edge Cases (Day 6)
// =============================================================================

/// RED PHASE: Test for nested conditionals (ifeq inside ifdef)
#[test]
fn test_COND_EDGE_001_nested_ifeq_ifdef() {
    let makefile = r#"
ifdef DEBUG
ifeq ($(VERBOSE),1)
CFLAGS += -DDEBUG_VERBOSE
endif
endif
"#;

    // ARRANGE: Parse Makefile with nested conditionals
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();

    // ASSERT: Should have conditional structure
    // Outer ifdef DEBUG should contain inner ifeq
    let has_ifdef = ast
        .items
        .iter()
        .any(|item| matches!(item, MakeItem::Conditional { .. }));

    assert!(
        has_ifdef,
        "Should have conditional items for nested structure"
    );
}

/// RED PHASE: Test for conditionals with function calls in condition
#[test]
fn test_COND_EDGE_002_conditional_with_functions() {
    let makefile = r#"
ifeq ($(shell uname),Linux)
PLATFORM = linux
else
PLATFORM = other
endif
"#;

    // ARRANGE: Parse Makefile with function call in condition
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();

    // ASSERT: Should parse ifeq with shell function
    let has_conditional = ast
        .items
        .iter()
        .any(|item| matches!(item, MakeItem::Conditional { .. }));

    assert!(has_conditional, "Should have conditional item");

    // ASSERT: Should have variable assignment in then or else branch
    let has_var_in_conditional = ast.items.iter().any(|item| {
        if let MakeItem::Conditional {
            then_items,
            else_items,
            ..
        } = item
        {
            let in_then = then_items
                .iter()
                .any(|i| matches!(i, MakeItem::Variable { name, .. } if name == "PLATFORM"));
            let in_else = else_items
                .as_ref()
                .map(|items| {
                    items
                        .iter()
                        .any(|i| matches!(i, MakeItem::Variable { name, .. } if name == "PLATFORM"))
                })
                .unwrap_or(false);
            in_then || in_else
        } else {
            false
        }
    });

    assert!(
        has_var_in_conditional,
        "Should have PLATFORM variable in conditional branches"
    );
}

/// RED PHASE: Test for empty conditional blocks
#[test]
fn test_COND_EDGE_003_empty_conditional_blocks() {
    let makefile = r#"
ifdef DEBUG
# Empty then block
endif

ifndef RELEASE
# Empty then block
else
# Empty else block
endif
"#;

    // ARRANGE: Parse Makefile with empty conditional blocks
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed for empty blocks");

    let ast = result.unwrap();

    // ASSERT: Should have conditional items even if empty
    let conditional_count = ast
        .items
        .iter()
        .filter(|item| matches!(item, MakeItem::Conditional { .. }))
        .count();

    assert!(
        conditional_count >= 2,
        "Should have 2 conditional items (ifdef + ifndef)"
    );
}

/// RED PHASE: Test for complex real-world nesting
#[test]
fn test_COND_EDGE_004_complex_nesting_real_world() {
    let makefile = r#"
ifdef USE_PYTHON
PYTHON := python3
ifeq ($(shell which python3),)
$(error Python 3 not found)
endif
else ifdef USE_PYTHON2
PYTHON := python2
else
PYTHON := python
endif

ifneq ($(PYTHON),)
PYTHON_CONFIG := $(PYTHON)-config
endif
"#;

    // ARRANGE: Parse complex real-world conditional nesting
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed for complex nesting");

    let ast = result.unwrap();

    // ASSERT: Should have multiple conditional items
    let conditional_count = ast
        .items
        .iter()
        .filter(|item| matches!(item, MakeItem::Conditional { .. }))
        .count();

    assert!(
        conditional_count >= 2,
        "Should have at least 2 conditional items"
    );

    // ASSERT: Should have variable assignments in conditional branches
    let has_python_var = ast.items.iter().any(|item| {
        if let MakeItem::Conditional { then_items, else_items, .. } = item {
            let check_items = |items: &[MakeItem]| {
                items.iter().any(|i| {
                    matches!(i, MakeItem::Variable { name, .. } if name == "PYTHON" || name == "PYTHON_CONFIG")
                })
            };

            let in_then = check_items(then_items);
            let in_else = else_items.as_ref().map(|items| check_items(items)).unwrap_or(false);
            in_then || in_else
        } else {
            false
        }
    });

    assert!(
        has_python_var,
        "Should have PYTHON or PYTHON_CONFIG variable in conditional branches"
    );
}

/// RED PHASE: Test for multiple nested conditional levels
#[test]
fn test_COND_EDGE_005_multiple_nested_levels() {
    let makefile = r#"
ifdef ENABLE_FEATURE_A
FEATURE_A = 1
ifdef ENABLE_FEATURE_A_VERBOSE
FEATURE_A_FLAGS = -v
else
FEATURE_A_FLAGS =
endif
else
FEATURE_A = 0
endif
"#;

    // ARRANGE: Parse conditional with multiple nesting levels
    let result = parse_makefile(makefile);
    assert!(
        result.is_ok(),
        "Parsing should succeed for multiple nesting levels"
    );

    let ast = result.unwrap();

    // ASSERT: Should have conditional item
    let has_conditional = ast
        .items
        .iter()
        .any(|item| matches!(item, MakeItem::Conditional { .. }));

    assert!(has_conditional, "Should have conditional item");

    // ASSERT: Should have variable assignments in conditional branches
    let has_var = ast.items.iter().any(|item| {
        if let MakeItem::Conditional {
            then_items,
            else_items,
            ..
        } = item
        {
            let check_items = |items: &[MakeItem]| {
                items.iter().any(|i| {
                    matches!(i, MakeItem::Variable { name, .. }
                        if name == "FEATURE_A" || name == "FEATURE_A_FLAGS")
                })
            };

            let in_then = check_items(then_items);
            let in_else = else_items
                .as_ref()
                .map(|items| check_items(items))
                .unwrap_or(false);
            in_then || in_else
        } else {
            false
        }
    });

    assert!(
        has_var,
        "Should have FEATURE_A or FEATURE_A_FLAGS variable in conditional branches"
    );
}
