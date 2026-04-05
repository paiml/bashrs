//! Tests for expression-to-ShellValue conversion (part 1: tests 001-028).
//!
//! Extracted from `mod.rs` to reduce per-file complexity.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::ast::restricted::{BinaryOp, Literal, UnaryOp};
use crate::ast::{Expr, Function, RestrictedAst, Stmt, Type};

/// Helper: wrap a single let statement in a main function and convert to IR
#[test]
fn test_EXPR_VAL_025_binary_arithmetic_ops() {
    // Add
    let ir = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Literal(Literal::U32(2))),
            right: Box::new(Expr::Literal(Literal::U32(3))),
        },
    );
    assert!(matches!(
        extract_let_value(&ir),
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Add,
            ..
        }
    ));

    // Mul
    let ir = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expr::Literal(Literal::U32(4))),
            right: Box::new(Expr::Literal(Literal::U32(5))),
        },
    );
    assert!(matches!(
        extract_let_value(&ir),
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Mul,
            ..
        }
    ));

    // Div
    let ir = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::Div,
            left: Box::new(Expr::Literal(Literal::U32(10))),
            right: Box::new(Expr::Literal(Literal::U32(2))),
        },
    );
    assert!(matches!(
        extract_let_value(&ir),
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Div,
            ..
        }
    ));

    // Rem
    let ir = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::Rem,
            left: Box::new(Expr::Literal(Literal::U32(10))),
            right: Box::new(Expr::Literal(Literal::U32(3))),
        },
    );
    assert!(matches!(
        extract_let_value(&ir),
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Mod,
            ..
        }
    ));
}

// ===== Binary: logical ops =====

#[test]
fn test_EXPR_VAL_026_binary_logical_and() {
    let ir = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::And,
            left: Box::new(Expr::Literal(Literal::Bool(true))),
            right: Box::new(Expr::Literal(Literal::Bool(false))),
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::LogicalAnd { left, right } => {
            assert!(matches!(**left, ShellValue::Bool(true)));
            assert!(matches!(**right, ShellValue::Bool(false)));
        }
        other => panic!("Expected LogicalAnd, got {:?}", other),
    }
}

#[test]
fn test_EXPR_VAL_027_binary_logical_or() {
    let ir = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::Or,
            left: Box::new(Expr::Literal(Literal::Bool(false))),
            right: Box::new(Expr::Literal(Literal::Bool(true))),
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::LogicalOr { left, right } => {
            assert!(matches!(**left, ShellValue::Bool(false)));
            assert!(matches!(**right, ShellValue::Bool(true)));
        }
        other => panic!("Expected LogicalOr, got {:?}", other),
    }
}

// ===== MethodCall: std::env::args().nth(N).unwrap() =====

#[test]
fn test_EXPR_VAL_028_method_call_env_args_nth_unwrap() {
    // Pattern: std::env::args().nth(1).unwrap() -> $1
    let expr = Expr::MethodCall {
        receiver: Box::new(Expr::MethodCall {
            receiver: Box::new(Expr::FunctionCall {
                name: "std::env::args".to_string(),
                args: vec![],
            }),
            method: "nth".to_string(),
            args: vec![Expr::Literal(Literal::U32(1))],
        }),
        method: "unwrap".to_string(),
        args: vec![],
    };
    let ir = convert_let_stmt("first_arg", expr);
    let val = extract_let_value(&ir);
    match val {
        ShellValue::Arg { position } => {
            assert_eq!(*position, Some(1));
        }
        other => panic!("Expected Arg {{ position: Some(1) }}, got {:?}", other),
    }
}
