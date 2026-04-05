#![allow(clippy::unwrap_used)]

use super::*;
use crate::bash_parser::ast::*;

// ============================================================================
// TypeAnnotation Parsing Tests
// ============================================================================

#[test]
fn test_annotation_hint_missing_returns_none() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![BashStmt::Assignment {
        name: "x".to_string(),
        index: None,
        value: BashExpr::Literal("5".to_string()),
        exported: false,
        span: Span::dummy(),
    }]);

    checker.check_ast(&ast);
    assert_eq!(checker.annotation_hint("x"), None);
}

// ============================================================================
// Boolean Literal Inference Tests
// ============================================================================

#[test]
fn test_infer_true_as_boolean() {
    let mut checker = TypeChecker::new();
    let ty = checker.infer_expr(&BashExpr::Literal("true".to_string()));
    assert_eq!(ty, Some(ShellType::Boolean));
}

#[test]
fn test_infer_false_as_boolean() {
    let mut checker = TypeChecker::new();
    let ty = checker.infer_expr(&BashExpr::Literal("false".to_string()));
    assert_eq!(ty, Some(ShellType::Boolean));
}

#[test]
fn test_bool_annotation_with_true_literal_no_warning() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![
        BashStmt::Comment {
            text: " @type debug: bool".to_string(),
            span: Span::dummy(),
        },
        BashStmt::Assignment {
            name: "debug".to_string(),
            index: None,
            value: BashExpr::Literal("true".to_string()),
            exported: false,
            span: Span::dummy(),
        },
    ]);

    let diags = checker.check_ast(&ast);
    assert!(
        diags.is_empty(),
        "true should be compatible with bool annotation"
    );
}
