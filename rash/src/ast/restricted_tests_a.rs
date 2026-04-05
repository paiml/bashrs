#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::restricted::*;

// ============================================================================
// RestrictedAst: validate coverage
// ============================================================================

#[test]
fn test_nesting_depth_range() {
    let expr = Expr::Range {
        start: Box::new(Expr::Unary {
            op: UnaryOp::Neg,
            operand: Box::new(Expr::Literal(Literal::U32(1))),
        }),
        end: Box::new(Expr::Literal(Literal::U32(10))),
        inclusive: true,
    };
    assert_eq!(expr.nesting_depth(), 2);
}

// ============================================================================
// Expr::collect_function_calls
// ============================================================================

#[test]
fn test_expr_collect_calls_method_and_unary() {
    let mut calls = vec![];
    Expr::MethodCall {
        receiver: Box::new(Expr::FunctionCall {
            name: "get".to_string(),
            args: vec![],
        }),
        method: "do_thing".to_string(),
        args: vec![Expr::FunctionCall {
            name: "helper".to_string(),
            args: vec![],
        }],
    }
    .collect_function_calls(&mut calls);
    assert_eq!(calls, vec!["get", "helper"]);

    let mut calls = vec![];
    Expr::Unary {
        op: UnaryOp::Not,
        operand: Box::new(Expr::FunctionCall {
            name: "check".to_string(),
            args: vec![],
        }),
    }
    .collect_function_calls(&mut calls);
    assert_eq!(calls, vec!["check"]);
}

#[test]
fn test_expr_collect_calls_no_calls_from_atoms() {
    let mut calls = vec![];
    Expr::Variable("x".to_string()).collect_function_calls(&mut calls);
    Expr::Literal(Literal::U32(5)).collect_function_calls(&mut calls);
    Expr::PositionalArgs.collect_function_calls(&mut calls);
    assert!(calls.is_empty());
}

// ============================================================================
// Pattern edge cases
// ============================================================================

#[test]
fn test_pattern_validation_edge_cases() {
    assert!(Pattern::Variable("$bad".to_string()).validate().is_err());
    assert!(Pattern::Struct {
        name: "P".to_string(),
        fields: vec![(
            "x".to_string(),
            Pattern::Literal(Literal::Str("n\0".to_string()))
        )],
    }
    .validate()
    .is_err());
    assert!(
        Pattern::Tuple(vec![Pattern::Wildcard, Pattern::Variable("".to_string())])
            .validate()
            .is_err()
    );
    assert!(Pattern::Range {
        start: Literal::U32(0),
        end: Literal::U32(100),
        inclusive: true
    }
    .validate()
    .is_ok());
}

#[test]
fn test_pattern_binds_variable_range() {
    assert!(!Pattern::Range {
        start: Literal::U32(0),
        end: Literal::U32(10),
        inclusive: false
    }
    .binds_variable("x"));
}

// ============================================================================
// Type::is_allowed edge cases
// ============================================================================

#[test]
fn test_type_nested_is_allowed() {
    assert!(Type::U16.is_allowed());
    assert!(Type::Result {
        ok_type: Box::new(Type::Option {
            inner_type: Box::new(Type::U32)
        }),
        err_type: Box::new(Type::Str),
    }
    .is_allowed());
    assert!(Type::Option {
        inner_type: Box::new(Type::Result {
            ok_type: Box::new(Type::Bool),
            err_type: Box::new(Type::Str),
        }),
    }
    .is_allowed());
}

// ============================================================================
// Function validation edge cases
// ============================================================================

#[test]
fn test_function_body_and_param_validation() {
    assert!(Function {
        name: "test".to_string(),
        params: vec![],
        return_type: Type::Void,
        body: vec![Stmt::Let {
            name: "".to_string(),
            value: Expr::Literal(Literal::U32(0)),
            declaration: true
        }],
    }
    .validate()
    .is_err());
    assert!(Function {
        name: "test".to_string(),
        params: vec![Parameter {
            name: "$invalid".to_string(),
            param_type: Type::U32
        }],
        return_type: Type::Void,
        body: vec![],
    }
    .validate()
    .is_err());
}

// ============================================================================
// Literal PartialEq
// ============================================================================

#[test]
fn test_literal_equality() {
    assert_eq!(Literal::U16(100), Literal::U16(100));
    assert_ne!(Literal::U16(100), Literal::U16(200));
    assert_ne!(Literal::U32(42), Literal::I32(42));
    assert_ne!(Literal::Bool(true), Literal::U32(1));
}
