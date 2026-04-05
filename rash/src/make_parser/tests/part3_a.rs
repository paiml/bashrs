#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

// ============================================================================
// FUNC-SUBST-001: $(subst from,to,text) Function
// ============================================================================

#[test]
fn test_PARSER_FUNC_004_nested_wildcard() {
    // ARRANGE: CRITICAL - nested function calls
    // This is the key pattern for recursive purification
    let makefile = "OBJS := $(filter %.o, $(wildcard *.c))";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse nested $(wildcard) inside $(filter)
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert!(value.contains("$(filter"));
            assert!(value.contains("$(wildcard"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_005_word() {
    // ARRANGE: word function (extracts Nth word)
    let makefile = "SECOND := $(word 2, foo bar baz)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse word function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "SECOND");
            assert!(value.contains("$(word"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_006_notdir() {
    // ARRANGE: notdir function (remove directory part)
    let makefile = "FILES := $(notdir src/main.c include/util.h)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse notdir function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FILES");
            assert!(value.contains("$(notdir"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_007_addsuffix() {
    // ARRANGE: addsuffix function
    let makefile = "OBJS := $(addsuffix .o, foo bar baz)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse addsuffix function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert!(value.contains("$(addsuffix"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_008_addprefix() {
    // ARRANGE: addprefix function
    let makefile = "OBJS := $(addprefix obj/, foo.o bar.o)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse addprefix function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert!(value.contains("$(addprefix"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_009_filter_out() {
    // ARRANGE: filter-out function (inverse of filter)
    let makefile = "SOURCES := $(filter-out test_%, main.c test_foo.c util.c)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse filter-out function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "SOURCES");
            assert!(value.contains("$(filter-out"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_010_wordlist() {
    // ARRANGE: wordlist function (extract range of words)
    let makefile = "MIDDLE := $(wordlist 2, 4, foo bar baz qux quux)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse wordlist function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "MIDDLE");
            assert!(value.contains("$(wordlist"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_011_words() {
    // ARRANGE: words function (count words)
    let makefile = "COUNT := $(words foo bar baz)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse words function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "COUNT");
            assert!(value.contains("$(words"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_012_firstword() {
    // ARRANGE: firstword function
    let makefile = "FIRST := $(firstword foo bar baz)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse firstword function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FIRST");
            assert!(value.contains("$(firstword"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_013_lastword() {
    // ARRANGE: lastword function
    let makefile = "LAST := $(lastword foo bar baz)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse lastword function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "LAST");
            assert!(value.contains("$(lastword"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_014_suffix() {
    // ARRANGE: suffix function (extract file suffixes)
    let makefile = "SUFFIXES := $(suffix foo.c bar.o baz.txt)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse suffix function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "SUFFIXES");
            assert!(value.contains("$(suffix"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_015_basename() {
    // ARRANGE: basename function (remove suffix)
    let makefile = "BASES := $(basename foo.c bar.o baz.txt)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse basename function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "BASES");
            assert!(value.contains("$(basename"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

// ============================================================================
// Sprint 65: Recursive Semantic Analysis Tests
// ============================================================================
// These tests verify that semantic analysis detects non-deterministic patterns
// even when they're nested inside function arguments.
//
// Example: $(filter %.c, $(wildcard src/*.c))
//                         ^^^^^^^^^^^^^^^^^
//                         Nested wildcard needs detection

#[test]
fn test_SEMANTIC_RECURSIVE_001_detect_wildcard_in_filter_args() {
    // ARRANGE: wildcard nested in filter arguments
    let makefile = "FILES := $(filter %.c, $(wildcard src/*.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Check if value contains nested wildcard
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FILES");
            // This test verifies we can DETECT the pattern
            // (actual semantic analysis implementation will come in GREEN phase)
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("$(filter"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_002_detect_wildcard_in_sort_args() {
    // ARRANGE: wildcard nested in sort arguments
    let makefile = "SORTED := $(sort $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify nested pattern detected
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "SORTED");
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("$(sort"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_003_detect_shell_date_in_addsuffix_args() {
    // ARRANGE: shell date nested in addsuffix arguments
    let makefile = "TIMESTAMPED := $(addsuffix -$(shell date +%s), foo bar)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify nested shell date detected
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "TIMESTAMPED");
            assert!(value.contains("$(shell date"));
            assert!(value.contains("$(addsuffix"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_004_detect_random_in_word_args() {
    // ARRANGE: $RANDOM nested in word arguments
    let makefile = "PICK := $(word $RANDOM, foo bar baz)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify nested $RANDOM detected
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "PICK");
            assert!(value.contains("$RANDOM") || value.contains("RANDOM"));
            assert!(value.contains("$(word"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_005_detect_shell_find_in_filter_args() {
    // ARRANGE: shell find nested in filter arguments
    let makefile = "FOUND := $(filter %.c, $(shell find src -name '*.c'))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify nested shell find detected
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FOUND");
            assert!(value.contains("$(shell find"));
            assert!(value.contains("$(filter"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_006_detect_deeply_nested_wildcard() {
    // ARRANGE: deeply nested - wildcard in filter in sort
    let makefile = "DEEP := $(sort $(filter %.c, $(wildcard src/*.c)))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify all nesting levels detected
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "DEEP");
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("$(filter"));
            assert!(value.contains("$(sort"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_007_detect_multiple_nested_wildcards() {
    // ARRANGE: multiple wildcard calls in single function
    let makefile = "MULTI := $(filter %.c %.h, $(wildcard src/*.c) $(wildcard inc/*.h))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify multiple nested patterns detected
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "MULTI");
            // Should contain two wildcard calls
            let wildcard_count = value.matches("$(wildcard").count();
            assert!(
                wildcard_count >= 2,
                "Expected at least 2 wildcard calls, found {}",
                wildcard_count
            );
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_008_safe_filter_no_wildcard() {
    // ARRANGE: filter without nested non-deterministic code (SAFE)
    let makefile = "SAFE := $(filter %.c, foo.c bar.c baz.c)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify no non-deterministic patterns
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "SAFE");
            // Should NOT contain wildcard/shell/random
            assert!(!value.contains("$(wildcard"));
            assert!(!value.contains("$(shell"));
            assert!(!value.contains("$RANDOM"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_009_purified_sort_wrapped_wildcard() {
    // ARRANGE: PURIFIED - wildcard wrapped with sort (SAFE)
    let makefile = "PURIFIED := $(filter %.c, $(sort $(wildcard src/*.c)))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify purified pattern (sort wraps wildcard)
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "PURIFIED");
            // Should contain both wildcard (non-deterministic) and sort (purifier)
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("$(sort"));
            // Verify sort comes BEFORE wildcard (wrapping)
            let sort_pos = value.find("$(sort").unwrap();
            let wildcard_pos = value.find("$(wildcard").unwrap();
            assert!(sort_pos < wildcard_pos, "sort should wrap wildcard");
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_010_detect_wildcard_in_firstword() {
    // ARRANGE: wildcard in firstword (HIGH RISK - returns different results)
    let makefile = "FIRST := $(firstword $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Critical case - firstword depends on order
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FIRST");
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("$(firstword"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_011_detect_wildcard_in_lastword() {
    // ARRANGE: wildcard in lastword (HIGH RISK - returns different results)
    let makefile = "LAST := $(lastword $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Critical case - lastword depends on order
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "LAST");
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("$(lastword"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_012_detect_wildcard_in_wordlist() {
    // ARRANGE: wildcard in wordlist (HIGH RISK - indices depend on order)
    let makefile = "RANGE := $(wordlist 2, 4, $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Critical case - wordlist indices depend on order
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "RANGE");
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("$(wordlist"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_013_detect_shell_date_in_filter() {
    // ARRANGE: shell date in filter arguments (timestamp in filtering)
    let makefile = "DATED := $(filter release-%, $(shell date +%Y%m%d))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify shell date nested in filter
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "DATED");
            assert!(value.contains("$(shell date"));
            assert!(value.contains("$(filter"));
        }
        _ => panic!("Expected Variable"),
    }
}

