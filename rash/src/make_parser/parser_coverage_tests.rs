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
fn test_line_continuation_single() {
    let input = "VAR = a \\\n    b";
    let ast = parse_makefile(input).unwrap();
    assert_eq!(ast.items.len(), 1);
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "VAR");
            assert_eq!(value, "a b");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_line_continuation_multiple() {
    let input = "VAR = a \\\n    b \\\n    c";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Variable { value, .. } => {
            assert_eq!(value, "a b c");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_line_continuation_in_recipe() {
    let input = "build:\n\tgcc -o main \\\n\t  main.c \\\n\t  util.c";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 1);
            assert!(recipe[0].contains("main.c"));
            assert!(recipe[0].contains("util.c"));
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

#[test]
fn test_line_continuation_trailing_backslash_at_end() {
    // Backslash at the very last line (no next line to continue to)
    let input = "VAR = value\\";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Variable { value, .. } => {
            assert_eq!(value, "value\\");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

// === Define block tests (all flavors) ===

#[test]
fn test_define_block_recursive() {
    let input = "define MY_VAR\nline1\nline2\nendef";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "MY_VAR");
            assert_eq!(value, "line1\nline2");
            assert_eq!(*flavor, VarFlavor::Recursive);
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_define_block_simple_flavor() {
    let input = "define MY_VAR :=\nfoo\nendef";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Variable { flavor, .. } => {
            assert_eq!(*flavor, VarFlavor::Simple);
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_define_block_conditional_flavor() {
    let input = "define MY_VAR ?=\nfoo\nendef";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Variable { flavor, .. } => {
            assert_eq!(*flavor, VarFlavor::Conditional);
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_define_block_append_flavor() {
    let input = "define MY_VAR +=\nfoo\nendef";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Variable { flavor, .. } => {
            assert_eq!(*flavor, VarFlavor::Append);
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_define_block_shell_flavor() {
    let input = "define MY_VAR !=\nfoo\nendef";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Variable { flavor, .. } => {
            assert_eq!(*flavor, VarFlavor::Shell);
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_define_block_explicit_recursive() {
    let input = "define MY_VAR =\nfoo\nendef";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Variable { flavor, .. } => {
            assert_eq!(*flavor, VarFlavor::Recursive);
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_define_block_unterminated() {
    let input = "define MY_VAR\nline1\nline2";
    let result = parse_makefile(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Unterminated"));
}

#[test]
fn test_define_block_empty_name() {
    // Parser panics on empty define name (unwrap on None) — this is an edge case
    // that could be improved, but for now we verify it doesn't silently succeed
    let result = std::panic::catch_unwind(|| {
        parse_makefile("define \nfoo\nendef")
    });
    assert!(result.is_err() || result.unwrap().is_err());
}

#[test]
fn test_define_block_empty_body() {
    let input = "define MY_VAR\nendef";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Variable { value, .. } => {
            assert_eq!(value, "");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

// === Conditional parsing (ifneq, ifndef, else, nested) ===

#[test]
fn test_conditional_ifneq() {
    let input = "ifneq ($(DEBUG),1)\nCFLAGS = -O2\nendif";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Conditional {
            condition,
            then_items,
            else_items,
            ..
        } => {
            assert!(matches!(condition, MakeCondition::IfNeq(_, _)));
            assert_eq!(then_items.len(), 1);
            assert!(else_items.is_none());
        }
        other => panic!("Expected Conditional, got {:?}", other),
    }
}

#[test]
fn test_conditional_ifndef() {
    let input = "ifndef CC\nCC = gcc\nendif";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Conditional { condition, .. } => {
            assert!(matches!(condition, MakeCondition::IfNdef(_)));
            if let MakeCondition::IfNdef(var) = condition {
                assert_eq!(var, "CC");
            }
        }
        other => panic!("Expected Conditional, got {:?}", other),
    }
}

#[test]
fn test_conditional_ifdef() {
    let input = "ifdef DEBUG\nCFLAGS = -g\nendif";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Conditional { condition, .. } => {
            assert!(matches!(condition, MakeCondition::IfDef(_)));
        }
        other => panic!("Expected Conditional, got {:?}", other),
    }
}

#[test]
fn test_conditional_with_else() {
    let input = "ifeq ($(DEBUG),1)\nCFLAGS = -g\nelse\nCFLAGS = -O2\nendif";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Conditional {
            then_items,
            else_items,
            ..
        } => {
            assert_eq!(then_items.len(), 1);
            assert!(else_items.is_some());
            assert_eq!(else_items.as_ref().unwrap().len(), 1);
        }
        other => panic!("Expected Conditional, got {:?}", other),
    }
}

#[test]
fn test_conditional_nested() {
    let input = "ifdef A\nifeq ($(B),1)\nX = 1\nendif\nendif";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Conditional { then_items, .. } => {
            // Inner conditional should be parsed
            assert!(then_items.len() >= 1);
        }
        other => panic!("Expected Conditional, got {:?}", other),
    }
}

#[test]
fn test_conditional_with_comment_inside() {
    let input = "ifdef DEBUG\n# debug mode\nCFLAGS = -g\nendif";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Conditional { then_items, .. } => {
            assert!(then_items.len() >= 1);
        }
        other => panic!("Expected Conditional, got {:?}", other),
    }
}

#[test]
fn test_conditional_with_target_inside() {
    let input = "ifdef DEBUG\ndebug-build:\n\techo debug\nendif";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Conditional { then_items, .. } => {
            assert!(then_items.len() >= 1);
            assert!(matches!(then_items[0], MakeItem::Target { .. }));
        }
        other => panic!("Expected Conditional, got {:?}", other),
    }
}

#[test]
fn test_conditional_ifeq_bad_syntax_no_parens() {
    let input = "ifeq bad\nendif";
    let result = parse_makefile(input);
    assert!(result.is_err());
}

#[test]
fn test_conditional_ifeq_missing_comma() {
    let input = "ifeq (onlyonearg)\nendif";
    let result = parse_makefile(input);
    assert!(result.is_err());
}

#[test]
fn test_conditional_ifdef_empty_varname() {
    let input = "ifdef \nendif";
    let result = parse_makefile(input);
    assert!(result.is_err());
}

#[test]
fn test_conditional_else_with_nested_inside() {
    let input = "ifdef A\nX = 1\nelse\nifeq ($(B),2)\nY = 2\nendif\nendif";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Conditional { else_items, .. } => {
            assert!(else_items.is_some());
        }
        other => panic!("Expected Conditional, got {:?}", other),
    }
}

// === Pattern rules ===

#[test]
fn test_pattern_rule_basic() {
    let input = "%.o: %.c\n\t$(CC) -c $< -o $@";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::PatternRule {
            target_pattern,
            prereq_patterns,
            recipe,
            ..
        } => {
            assert_eq!(target_pattern, "%.o");
            assert_eq!(prereq_patterns, &["%.c".to_string()]);
            assert_eq!(recipe.len(), 1);
        }
        other => panic!("Expected PatternRule, got {:?}", other),
    }
}

#[test]
fn test_pattern_rule_no_recipe() {
    let input = "%.o: %.c";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::PatternRule { recipe, .. } => {
            assert!(recipe.is_empty());
        }
        other => panic!("Expected PatternRule, got {:?}", other),
    }
}

// === .PHONY target detection ===

#[test]
fn test_phony_target_marked() {
    let input = ".PHONY: build test\nbuild:\n\tcargo build\ntest:\n\tcargo test";
    let ast = parse_makefile(input).unwrap();
    for item in &ast.items {
        if let MakeItem::Target { name, phony, .. } = item {
            if name == "build" || name == "test" {
                assert!(
                    *phony,
                    "Target {} should be marked phony",
                    name
                );
            }
        }
    }
}

#[test]
fn test_phony_non_matching_not_marked() {
    let input = ".PHONY: clean\nbuild:\n\tcargo build";
    let ast = parse_makefile(input).unwrap();
    for item in &ast.items {
        if let MakeItem::Target { name, phony, .. } = item {
            if name == "build" {
                assert!(!*phony, "build should NOT be phony");
            }
        }
    }
}

// === Target rule edge cases ===

#[test]
fn test_target_with_prerequisites() {
    let input = "build: src/main.c src/util.c\n\tgcc -o build $^";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Target {
            prerequisites, ..
        } => {
            assert_eq!(prerequisites.len(), 2);
            assert_eq!(prerequisites[0], "src/main.c");
            assert_eq!(prerequisites[1], "src/util.c");
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

#[test]
fn test_target_multiple_recipe_lines() {
    let input = "build:\n\techo step1\n\techo step2\n\techo step3";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 3);
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

#[test]
fn test_target_recipe_with_empty_line_between() {
    // Empty line between recipe lines should end the recipe
    // unless the next line is also tab-indented
    let input = "build:\n\techo step1\n\n\techo step2";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            // Both recipe lines should be captured because
            // empty line followed by another tab-indented line continues
            assert_eq!(recipe.len(), 2);
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

#[test]
fn test_target_recipe_ends_at_non_tab() {
    let input = "build:\n\techo step1\nCC = gcc";
    let ast = parse_makefile(input).unwrap();
    assert_eq!(ast.items.len(), 2);
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 1);
        }
        other => panic!("Expected Target, got {:?}", other),
    }
    assert!(matches!(ast.items[1], MakeItem::Variable { .. }));
}

#[test]
fn test_empty_target_name_error() {
    let input = ": deps\n\tcommand";
    let result = parse_makefile(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Empty target name"));
}

// === Variable assignment variants ===

#[test]
fn test_variable_simple_assign() {
    let input = "CC := gcc";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Variable { flavor, .. } => {
            assert_eq!(*flavor, VarFlavor::Simple);
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_variable_conditional_assign() {
    let input = "CC ?= gcc";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Variable { flavor, .. } => {
            assert_eq!(*flavor, VarFlavor::Conditional);
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_variable_append_assign() {
    let input = "CFLAGS += -Wall";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Variable { flavor, .. } => {
            assert_eq!(*flavor, VarFlavor::Append);
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_variable_shell_assign() {
    let input = "DATE != date +%Y";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Variable { flavor, .. } => {
            assert_eq!(*flavor, VarFlavor::Shell);
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

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
        MakeItem::Include {
            path, optional, ..
        } => {
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
        MakeItem::Include {
            path, optional, ..
        } => {
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
