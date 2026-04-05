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
    let result = std::panic::catch_unwind(|| parse_makefile("define \nfoo\nendef"));
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

include!("parser_coverage_tests_tests_phony_target.rs");
