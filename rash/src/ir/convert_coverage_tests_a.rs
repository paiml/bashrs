#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Coverage tests for ir/convert.rs and ir/convert_fn.rs.
//!
//! Tests: detect_shadows, replace_var_refs_in_value, convert_stmt_in_function
//! paths, convert_for_iterable, convert_let_block, convert_expr dispatch,
//! effect analysis, and convert_index_to_value branches.

use super::*;
use crate::ast::restricted::{BinaryOp, Function, Literal, MatchArm, Parameter, Pattern, Type};
use crate::ast::{Expr, RestrictedAst, Stmt};

#[test]
fn test_break_continue_return_top_level() {
    let ast = make_main(vec![Stmt::While {
        condition: Expr::Literal(Literal::Bool(true)),
        body: vec![Stmt::Break],
        max_iterations: Some(10000),
    }]);
    assert_seq(&from_ast(&ast).unwrap());

    let ast2 = make_main(vec![Stmt::For {
        pattern: Pattern::Variable("i".into()),
        iter: Expr::Range {
            start: Box::new(Expr::Literal(Literal::U32(0))),
            end: Box::new(Expr::Literal(Literal::U32(5))),
            inclusive: false,
        },
        body: vec![Stmt::Continue],
        max_iterations: Some(1000),
    }]);
    assert_seq(&from_ast(&ast2).unwrap());

    assert_seq(
        &from_ast(&make_main(vec![Stmt::Return(Some(Expr::Literal(
            Literal::U32(42),
        )))]))
        .unwrap(),
    );
    assert_seq(&from_ast(&make_main(vec![Stmt::Return(None)])).unwrap());
}

// ============================================================================
// for over various iterables
// ============================================================================

#[test]
fn test_for_over_iterables() {
    // Array literal
    let ast = make_main(vec![Stmt::For {
        pattern: Pattern::Variable("it".into()),
        iter: Expr::Array(vec![
            Expr::Literal(Literal::Str("a".into())),
            Expr::Literal(Literal::Str("b".into())),
        ]),
        body: vec![Stmt::Expr(Expr::FunctionCall {
            name: "echo".into(),
            args: vec![Expr::Variable("it".into())],
        })],
        max_iterations: Some(1000),
    }]);
    assert_seq(&from_ast(&ast).unwrap());

    // Tracked array variable
    let ast2 = make_main(vec![
        Stmt::Let {
            name: "arr".into(),
            value: Expr::Array(vec![
                Expr::Literal(Literal::U32(1)),
                Expr::Literal(Literal::U32(2)),
            ]),
            declaration: true,
        },
        Stmt::For {
            pattern: Pattern::Variable("x".into()),
            iter: Expr::Variable("arr".into()),
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "echo".into(),
                args: vec![Expr::Variable("x".into())],
            })],
            max_iterations: Some(1000),
        },
    ]);
    assert_seq(&from_ast(&ast2).unwrap());

    // Untracked variable
    let ast3 = make_main(vec![Stmt::For {
        pattern: Pattern::Variable("x".into()),
        iter: Expr::Variable("items".into()),
        body: vec![Stmt::Expr(Expr::FunctionCall {
            name: "echo".into(),
            args: vec![Expr::Variable("x".into())],
        })],
        max_iterations: Some(1000),
    }]);
    assert_seq(&from_ast(&ast3).unwrap());

    // Generic expression
    let ast4 = make_main(vec![Stmt::For {
        pattern: Pattern::Variable("x".into()),
        iter: Expr::FunctionCall {
            name: "get".into(),
            args: vec![],
        },
        body: vec![Stmt::Expr(Expr::FunctionCall {
            name: "echo".into(),
            args: vec![Expr::Variable("x".into())],
        })],
        max_iterations: Some(1000),
    }]);
    assert_seq(&from_ast(&ast4).unwrap());
}

// ============================================================================
// Entry point not found error
// ============================================================================

#[test]
fn test_entry_point_not_found_error() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "helper".into(),
            params: vec![],
            return_type: Type::Void,
            body: vec![],
        }],
        entry_point: "main".into(),
    };
    assert!(from_ast(&ast).is_err());
}

// ============================================================================
// Empty array and PositionalArgs
// ============================================================================

#[test]
fn test_empty_array_and_positional_args() {
    let ast = make_main(vec![Stmt::Let {
        name: "e".into(),
        value: Expr::Array(vec![]),
        declaration: true,
    }]);
    assert_seq(&from_ast(&ast).unwrap());

    let ast2 = make_main(vec![Stmt::Let {
        name: "a".into(),
        value: Expr::PositionalArgs,
        declaration: true,
    }]);
    assert_seq(&from_ast(&ast2).unwrap());
}
