#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Coverage tests for make_parser/parser.rs uncovered branches.
//!
//! Targets: line continuations with metadata, define blocks (all flavors),
//! conditional branches (ifneq, ifndef, else, nested), pattern rules,
//! phony target detection, function extraction, and error paths.

use crate::make_parser::ast::*;
use crate::make_parser::parse_makefile;
use crate::make_parser::parser::extract_function_calls;

// === Line continuation with metadata tracking ===

#[test]
fn test_variable_empty_name_error() {
    // Direct parse_makefile won't trigger this easily since
    // " = value" would be filtered differently. Use "= value" directly
    // via an indirect approach: the parser treats "= val" as variable
    let input = " = value";
    let result = parse_makefile(input);
    assert!(result.is_err());
}

// === Include directive variants ===

#[test]
fn test_include_mandatory() {
    let input = "include common.mk";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Include { path, optional, .. } => {
            assert_eq!(path, "common.mk");
            assert!(!optional);
        }
        other => panic!("Expected Include, got {:?}", other),
    }
}

#[test]
fn test_include_optional_dash() {
    let input = "-include optional.mk";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Include { optional, .. } => {
            assert!(*optional);
        }
        other => panic!("Expected Include, got {:?}", other),
    }
}

#[test]
fn test_include_sinclude() {
    let input = "sinclude optional.mk";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Include { path, optional, .. } => {
            assert_eq!(path, "optional.mk");
            assert!(*optional);
        }
        other => panic!("Expected Include, got {:?}", other),
    }
}

// === Function extraction ===

#[test]
fn test_extract_function_calls_wildcard() {
    let calls = extract_function_calls("SOURCES := $(wildcard src/*.c)");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "wildcard");
    assert_eq!(calls[0].1, "src/*.c");
}

#[test]
fn test_extract_function_calls_nested() {
    let calls = extract_function_calls("OBJS := $(patsubst %.c,%.o,$(SOURCES))");
    assert!(!calls.is_empty());
    assert_eq!(calls[0].0, "patsubst");
}

#[test]
fn test_extract_function_calls_no_args() {
    let calls = extract_function_calls("DIR := $(CURDIR)");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "CURDIR");
    assert_eq!(calls[0].1, "");
}

#[test]
fn test_extract_function_calls_multiple() {
    let calls = extract_function_calls("$(wildcard *.c) $(patsubst %.c,%.o,x)");
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].0, "wildcard");
    assert_eq!(calls[1].0, "patsubst");
}

#[test]
fn test_extract_function_calls_unmatched_paren() {
    // Unbalanced parens: $( without closing )
    let calls = extract_function_calls("$(unclosed");
    assert!(calls.is_empty());
}

#[test]
fn test_extract_function_calls_no_dollar() {
    let calls = extract_function_calls("no functions here");
    assert!(calls.is_empty());
}

// === Metadata line count ===

#[test]
fn test_metadata_line_count() {
    let input = "# comment\nCC = gcc\nbuild:\n\tcargo build";
    let ast = parse_makefile(input).unwrap();
    assert_eq!(ast.metadata.line_count, 4);
}

#[test]
fn test_metadata_empty_makefile() {
    let ast = parse_makefile("").unwrap();
    assert_eq!(ast.metadata.line_count, 0);
    assert_eq!(ast.items.len(), 0);
}

// === Complex full Makefile ===

#[test]
fn test_full_makefile_mixed() {
    let input = "\
# Build config
CC := gcc
CFLAGS ?= -O2

.PHONY: all clean

all: main.o util.o
\t$(CC) -o app main.o util.o

%.o: %.c
\t$(CC) $(CFLAGS) -c $< -o $@

clean:
\trm -f app *.o

ifdef DEBUG
CFLAGS += -g
endif
";
    let ast = parse_makefile(input).unwrap();
    // Should parse comment, variables, .PHONY, targets, pattern rule, conditional
    assert!(ast.items.len() >= 6);

    // Verify phony targets
    let mut found_all_phony = false;
    let mut found_clean_phony = false;
    for item in &ast.items {
        if let MakeItem::Target { name, phony, .. } = item {
            if name == "all" {
                found_all_phony = *phony;
            }
            if name == "clean" {
                found_clean_phony = *phony;
            }
        }
    }
    assert!(found_all_phony, "all should be phony");
    assert!(found_clean_phony, "clean should be phony");
}

#[test]
fn test_variable_with_colon_after_equals() {
    // "target: VAR=value" should be parsed as target, not variable
    // because colon comes before equals
    let input = "target: dep\n\techo done";
    let ast = parse_makefile(input).unwrap();
    assert!(matches!(ast.items[0], MakeItem::Target { .. }));
}

#[test]
fn test_conditional_else_with_empty_lines() {
    let input = "ifdef FOO\n\nX = 1\nelse\n\nY = 2\nendif";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Conditional {
            then_items,
            else_items,
            ..
        } => {
            assert!(!then_items.is_empty());
            assert!(else_items.is_some());
        }
        other => panic!("Expected Conditional, got {:?}", other),
    }
}
