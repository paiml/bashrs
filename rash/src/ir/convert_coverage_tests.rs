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

fn make_main(body: Vec<Stmt>) -> RestrictedAst {
    RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body,
        }],
        entry_point: "main".to_string(),
    }
}

fn make_with_fn(
    name: &str,
    p: Vec<Parameter>,
    ret: Type,
    body: Vec<Stmt>,
    main_body: Vec<Stmt>,
) -> RestrictedAst {
    RestrictedAst {
        functions: vec![
            Function {
                name: name.to_string(),
                params: p,
                return_type: ret,
                body,
            },
            Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: main_body,
            },
        ],
        entry_point: "main".to_string(),
    }
}

fn call_main(fn_name: &str) -> Vec<Stmt> {
    vec![Stmt::Expr(Expr::FunctionCall {
        name: fn_name.to_string(),
        args: vec![],
    })]
}

fn assert_seq(ir: &ShellIR) {
    assert!(
        matches!(ir, ShellIR::Sequence(_)),
        "Expected Sequence, got {:?}",
        ir
    );
}

// ============================================================================
// replace_var_refs_in_value: all ShellValue branches
// ============================================================================

#[test]
fn test_replace_var_refs_all_branches() {
    // Variable match and no-match
    let r = IrConverter::replace_var_refs_in_value(&ShellValue::Variable("x".into()), "x", "s");
    assert!(matches!(r, ShellValue::Variable(ref n) if n == "s"));
    let r = IrConverter::replace_var_refs_in_value(&ShellValue::Variable("y".into()), "x", "s");
    assert!(matches!(r, ShellValue::Variable(ref n) if n == "y"));

    // Arithmetic
    let v = ShellValue::Arithmetic {
        op: shell_ir::ArithmeticOp::Add,
        left: Box::new(ShellValue::Variable("x".into())),
        right: Box::new(ShellValue::String("1".into())),
    };
    match IrConverter::replace_var_refs_in_value(&v, "x", "s") {
        ShellValue::Arithmetic { left, .. } => {
            assert!(matches!(*left, ShellValue::Variable(ref n) if n == "s"))
        }
        _ => panic!("Expected Arithmetic"),
    }

    // Concat
    let v = ShellValue::Concat(vec![
        ShellValue::Variable("x".into()),
        ShellValue::String("w".into()),
    ]);
    match IrConverter::replace_var_refs_in_value(&v, "x", "s") {
        ShellValue::Concat(p) => assert!(matches!(&p[0], ShellValue::Variable(ref n) if n == "s")),
        _ => panic!("Expected Concat"),
    }

    // Comparison
    let v = ShellValue::Comparison {
        op: shell_ir::ComparisonOp::Gt,
        left: Box::new(ShellValue::Variable("x".into())),
        right: Box::new(ShellValue::Variable("y".into())),
    };
    match IrConverter::replace_var_refs_in_value(&v, "x", "s") {
        ShellValue::Comparison { left, .. } => {
            assert!(matches!(*left, ShellValue::Variable(ref n) if n == "s"))
        }
        _ => panic!("Expected Comparison"),
    }

    // LogicalNot
    let v = ShellValue::LogicalNot {
        operand: Box::new(ShellValue::Variable("x".into())),
    };
    match IrConverter::replace_var_refs_in_value(&v, "x", "s") {
        ShellValue::LogicalNot { operand } => {
            assert!(matches!(*operand, ShellValue::Variable(ref n) if n == "s"))
        }
        _ => panic!("Expected LogicalNot"),
    }

    // LogicalAnd
    let v = ShellValue::LogicalAnd {
        left: Box::new(ShellValue::Variable("x".into())),
        right: Box::new(ShellValue::Bool(true)),
    };
    match IrConverter::replace_var_refs_in_value(&v, "x", "s") {
        ShellValue::LogicalAnd { left, .. } => {
            assert!(matches!(*left, ShellValue::Variable(ref n) if n == "s"))
        }
        _ => panic!("Expected LogicalAnd"),
    }

    // LogicalOr
    let v = ShellValue::LogicalOr {
        left: Box::new(ShellValue::Bool(false)),
        right: Box::new(ShellValue::Variable("x".into())),
    };
    match IrConverter::replace_var_refs_in_value(&v, "x", "s") {
        ShellValue::LogicalOr { right, .. } => {
            assert!(matches!(*right, ShellValue::Variable(ref n) if n == "s"))
        }
        _ => panic!("Expected LogicalOr"),
    }

    // CommandSubst: not recursed
    let v = ShellValue::CommandSubst(shell_ir::Command {
        program: "echo".into(),
        args: vec![ShellValue::Variable("x".into())],
    });
    assert!(matches!(
        IrConverter::replace_var_refs_in_value(&v, "x", "s"),
        ShellValue::CommandSubst(_)
    ));

    // DynamicArrayAccess: array name match
    let v = ShellValue::DynamicArrayAccess {
        array: "x".into(),
        index: Box::new(ShellValue::Variable("x".into())),
    };
    match IrConverter::replace_var_refs_in_value(&v, "x", "s") {
        ShellValue::DynamicArrayAccess { array, index } => {
            assert_eq!(array, "s");
            assert!(matches!(*index, ShellValue::Variable(ref n) if n == "s"));
        }
        _ => panic!("Expected DynamicArrayAccess"),
    }

    // DynamicArrayAccess: array name no match
    let v = ShellValue::DynamicArrayAccess {
        array: "arr".into(),
        index: Box::new(ShellValue::Variable("i".into())),
    };
    match IrConverter::replace_var_refs_in_value(&v, "x", "s") {
        ShellValue::DynamicArrayAccess { array, .. } => assert_eq!(array, "arr"),
        _ => panic!("Expected DynamicArrayAccess unchanged"),
    }

    // String: unchanged
    let r = IrConverter::replace_var_refs_in_value(&ShellValue::String("hello".into()), "x", "s");
    assert!(matches!(r, ShellValue::String(ref s) if s == "hello"));
}

// ============================================================================
// Shadow detection and save/restore in loops
// ============================================================================

#[test]
fn test_for_loop_with_shadow_variable() {
    let ast = make_main(vec![
        Stmt::Let {
            name: "x".into(),
            value: Expr::Literal(Literal::U32(0)),
            declaration: true,
        },
        Stmt::For {
            pattern: Pattern::Variable("i".into()),
            iter: Expr::Range {
                start: Box::new(Expr::Literal(Literal::U32(0))),
                end: Box::new(Expr::Literal(Literal::U32(3))),
                inclusive: false,
            },
            body: vec![Stmt::Let {
                name: "x".into(),
                value: Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Variable("i".into())),
                    right: Box::new(Expr::Literal(Literal::U32(1))),
                },
                declaration: true,
            }],
            max_iterations: Some(1000),
        },
    ]);
    assert_seq(&from_ast(&ast).unwrap());
}

#[test]
fn test_while_loop_with_shadow_variable() {
    let ast = make_main(vec![
        Stmt::Let {
            name: "c".into(),
            value: Expr::Literal(Literal::U32(0)),
            declaration: true,
        },
        Stmt::While {
            condition: Expr::Binary {
                op: BinaryOp::Lt,
                left: Box::new(Expr::Variable("c".into())),
                right: Box::new(Expr::Literal(Literal::U32(5))),
            },
            body: vec![Stmt::Let {
                name: "c".into(),
                value: Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Variable("c".into())),
                    right: Box::new(Expr::Literal(Literal::U32(1))),
                },
                declaration: true,
            }],
            max_iterations: Some(10000),
        },
    ]);
    assert_seq(&from_ast(&ast).unwrap());
}

// ============================================================================
// convert_stmt_in_function paths
// ============================================================================

#[test]
fn test_fn_context_return_none() {
    let ast = make_with_fn(
        "h",
        vec![],
        Type::Void,
        vec![Stmt::Return(None)],
        call_main("h"),
    );
    assert_seq(&from_ast(&ast).unwrap());
}

#[test]
fn test_fn_context_while_loop() {
    let ast = make_with_fn(
        "ctr",
        vec![],
        Type::U32,
        vec![
            Stmt::Let {
                name: "n".into(),
                value: Expr::Literal(Literal::U32(0)),
                declaration: true,
            },
            Stmt::While {
                condition: Expr::Binary {
                    op: BinaryOp::Lt,
                    left: Box::new(Expr::Variable("n".into())),
                    right: Box::new(Expr::Literal(Literal::U32(5))),
                },
                body: vec![Stmt::Let {
                    name: "n".into(),
                    value: Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Variable("n".into())),
                        right: Box::new(Expr::Literal(Literal::U32(1))),
                    },
                    declaration: false,
                }],
                max_iterations: Some(10000),
            },
            Stmt::Expr(Expr::Variable("n".into())),
        ],
        call_main("ctr"),
    );
    assert_seq(&from_ast(&ast).unwrap());
}

#[test]
fn test_fn_context_for_range() {
    let ast = make_with_fn(
        "sum",
        vec![],
        Type::U32,
        vec![
            Stmt::Let {
                name: "t".into(),
                value: Expr::Literal(Literal::U32(0)),
                declaration: true,
            },
            Stmt::For {
                pattern: Pattern::Variable("i".into()),
                iter: Expr::Range {
                    start: Box::new(Expr::Literal(Literal::U32(1))),
                    end: Box::new(Expr::Literal(Literal::U32(5))),
                    inclusive: true,
                },
                body: vec![Stmt::Let {
                    name: "t".into(),
                    value: Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Variable("t".into())),
                        right: Box::new(Expr::Variable("i".into())),
                    },
                    declaration: false,
                }],
                max_iterations: Some(1000),
            },
            Stmt::Expr(Expr::Variable("t".into())),
        ],
        call_main("sum"),
    );
    assert_seq(&from_ast(&ast).unwrap());
}

#[test]
fn test_fn_context_for_array_and_variable() {
    // Array iter
    let ast = make_with_fn(
        "f",
        vec![],
        Type::Str,
        vec![Stmt::For {
            pattern: Pattern::Variable("it".into()),
            iter: Expr::Array(vec![
                Expr::Literal(Literal::Str("a".into())),
                Expr::Literal(Literal::Str("b".into())),
            ]),
            body: vec![Stmt::Return(Some(Expr::Variable("it".into())))],
            max_iterations: Some(1000),
        }],
        call_main("f"),
    );
    assert_seq(&from_ast(&ast).unwrap());

    // Variable iter (untracked)
    let ast2 = make_with_fn(
        "g",
        vec![],
        Type::Str,
        vec![Stmt::For {
            pattern: Pattern::Variable("x".into()),
            iter: Expr::Variable("items".into()),
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "echo".into(),
                args: vec![Expr::Variable("x".into())],
            })],
            max_iterations: Some(1000),
        }],
        call_main("g"),
    );
    assert_seq(&from_ast(&ast2).unwrap());
}

#[test]

include!("convert_coverage_tests_tests_cont.rs");
