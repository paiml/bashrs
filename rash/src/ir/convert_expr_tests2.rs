//! Tests for expression-to-ShellValue conversion (part 2: tests 029-041).
//!
//! Split from `convert_expr_tests.rs` to reduce per-file complexity.

#![allow(clippy::unwrap_used)]

use super::*;
use crate::ast::restricted::Literal;
use crate::ast::{Expr, Function, RestrictedAst, Stmt, Type};

/// Helper: wrap a single let statement in a main function and convert to IR
fn convert_let_stmt(name: &str, value: Expr) -> ShellIR {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: name.to_string(),
                value,
                declaration: true,
            }],
        }],
        entry_point: "main".to_string(),
    };
    from_ast(&ast).expect("IR conversion should succeed")
}

/// Helper: extract the ShellValue from a single Let in a Sequence
fn extract_let_value(ir: &ShellIR) -> &ShellValue {
    match ir {
        ShellIR::Sequence(stmts) => match &stmts[0] {
            ShellIR::Let { value, .. } => value,
            other => panic!("Expected Let, got {:?}", other),
        },
        other => panic!("Expected Sequence, got {:?}", other),
    }
}

/// Helper: extract inner sequence from outer Sequence([Sequence([...])])
fn extract_inner_sequence(ir: &ShellIR) -> &[ShellIR] {
    let ShellIR::Sequence(stmts) = ir else {
        panic!("Expected Sequence, got {:?}", ir);
    };
    let ShellIR::Sequence(inner) = &stmts[0] else {
        panic!("Expected inner Sequence, got {:?}", stmts[0]);
    };
    inner
}

/// Helper: extract (name, value) from a Let IR node
fn extract_let_name_value(ir: &ShellIR) -> (&str, &ShellValue) {
    let ShellIR::Let { name, value, .. } = ir else {
        panic!("Expected Let, got {:?}", ir);
    };
    (name, value)
}

// ===== MethodCall: args.get(N).unwrap_or(default) =====

#[test]
fn test_EXPR_VAL_029_method_call_args_get_unwrap_or() {
    // Pattern: args.get(2).unwrap_or("default") -> ${2:-default}
    let expr = Expr::MethodCall {
        receiver: Box::new(Expr::MethodCall {
            receiver: Box::new(Expr::Variable("args".to_string())),
            method: "get".to_string(),
            args: vec![Expr::Literal(Literal::U32(2))],
        }),
        method: "unwrap_or".to_string(),
        args: vec![Expr::Literal(Literal::Str("fallback".to_string()))],
    };
    let ir = convert_let_stmt("param", expr);
    let val = extract_let_value(&ir);
    match val {
        ShellValue::ArgWithDefault { position, default } => {
            assert_eq!(*position, 2);
            assert_eq!(default, "fallback");
        }
        other => panic!("Expected ArgWithDefault {{ 2, fallback }}, got {:?}", other),
    }
}

// ===== MethodCall: std::env::args().nth(N).unwrap_or(default) =====

#[test]
fn test_EXPR_VAL_030_method_call_env_args_nth_unwrap_or() {
    // Pattern: std::env::args().nth(0).unwrap_or("script") -> ${0:-script}
    let expr = Expr::MethodCall {
        receiver: Box::new(Expr::MethodCall {
            receiver: Box::new(Expr::FunctionCall {
                name: "std::env::args".to_string(),
                args: vec![],
            }),
            method: "nth".to_string(),
            args: vec![Expr::Literal(Literal::U32(0))],
        }),
        method: "unwrap_or".to_string(),
        args: vec![Expr::Literal(Literal::Str("default_script".to_string()))],
    };
    let ir = convert_let_stmt("script_name", expr);
    let val = extract_let_value(&ir);
    match val {
        ShellValue::ArgWithDefault { position, default } => {
            assert_eq!(*position, 0);
            assert_eq!(default, "default_script");
        }
        other => panic!(
            "Expected ArgWithDefault {{ 0, default_script }}, got {:?}",
            other
        ),
    }
}

// ===== MethodCall: unrecognized pattern =====

#[test]
fn test_EXPR_VAL_031_method_call_unrecognized_falls_to_unknown() {
    // A method call that doesn't match any recognized pattern
    let expr = Expr::MethodCall {
        receiver: Box::new(Expr::Variable("vec".to_string())),
        method: "len".to_string(),
        args: vec![],
    };
    let ir = convert_let_stmt("length", expr);
    let val = extract_let_value(&ir);
    match val {
        ShellValue::String(s) => assert_eq!(s, "unknown"),
        other => panic!("Expected String(\"unknown\"), got {:?}", other),
    }
}

// ===== PositionalArgs =====

#[test]
fn test_EXPR_VAL_032_positional_args() {
    let ir = convert_let_stmt("all_args", Expr::PositionalArgs);
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::Arg { position: None }));
}

// ===== Fallback (_) branch =====

#[test]
fn test_EXPR_VAL_033_array_expr_expands_to_indexed_lets() {
    // Array in let context: let arr = [1, 2] -> arr_0=1; arr_1=2
    let expr = Expr::Array(vec![
        Expr::Literal(Literal::U32(1)),
        Expr::Literal(Literal::U32(2)),
    ]);
    let ir = convert_let_stmt("arr", expr);
    // Should produce Sequence([Sequence([Let arr_0=1, Let arr_1=2]])
    let inner = extract_inner_sequence(&ir);
    assert_eq!(inner.len(), 2);

    let (name0, val0) = extract_let_name_value(&inner[0]);
    assert_eq!(name0, "arr_0");
    assert!(matches!(val0, ShellValue::String(s) if s == "1"));

    let (name1, val1) = extract_let_name_value(&inner[1]);
    assert_eq!(name1, "arr_1");
    assert!(matches!(val1, ShellValue::String(s) if s == "2"));
}

#[test]
fn test_EXPR_VAL_034_index_expr_becomes_variable() {
    // arr[0] -> $arr_0
    let expr = Expr::Index {
        object: Box::new(Expr::Variable("arr".to_string())),
        index: Box::new(Expr::Literal(Literal::U32(0))),
    };
    let ir = convert_let_stmt("elem", expr);
    let val = extract_let_value(&ir);
    match val {
        ShellValue::Variable(name) => assert_eq!(name, "arr_0"),
        other => panic!("Expected Variable(\"arr_0\"), got {:?}", other),
    }
}

#[test]
fn test_EXPR_VAL_035_fallback_range_expr() {
    // Range expressions hit the fallback _ branch
    let expr = Expr::Range {
        start: Box::new(Expr::Literal(Literal::U32(0))),
        end: Box::new(Expr::Literal(Literal::U32(10))),
        inclusive: false,
    };
    let ir = convert_let_stmt("rng", expr);
    let val = extract_let_value(&ir);
    match val {
        ShellValue::String(s) => assert_eq!(s, "unknown"),
        other => panic!("Expected String(\"unknown\") fallback, got {:?}", other),
    }
}

// ===== Edge cases: MethodCall partial matches that fall through =====

#[test]
fn test_EXPR_VAL_036_method_unwrap_non_nth_receiver() {
    // .unwrap() on something that is NOT .nth() -> falls to "unknown"
    let expr = Expr::MethodCall {
        receiver: Box::new(Expr::MethodCall {
            receiver: Box::new(Expr::Variable("x".to_string())),
            method: "first".to_string(), // not "nth"
            args: vec![Expr::Literal(Literal::U32(0))],
        }),
        method: "unwrap".to_string(),
        args: vec![],
    };
    let ir = convert_let_stmt("val", expr);
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
}

#[test]
fn test_EXPR_VAL_037_method_unwrap_or_non_get_non_nth() {
    // .unwrap_or(default) on something that is NOT .get() or .nth() -> falls to "unknown"
    let expr = Expr::MethodCall {
        receiver: Box::new(Expr::MethodCall {
            receiver: Box::new(Expr::Variable("x".to_string())),
            method: "find".to_string(), // not "get" or "nth"
            args: vec![Expr::Literal(Literal::U32(0))],
        }),
        method: "unwrap_or".to_string(),
        args: vec![Expr::Literal(Literal::Str("default".to_string()))],
    };
    let ir = convert_let_stmt("val", expr);
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
}

#[test]
fn test_EXPR_VAL_038_method_unwrap_with_args_not_recognized() {
    // .unwrap() with non-empty args -> not the recognized pattern -> falls through
    let expr = Expr::MethodCall {
        receiver: Box::new(Expr::Variable("x".to_string())),
        method: "unwrap".to_string(),
        args: vec![Expr::Literal(Literal::U32(42))], // unwrap doesn't take args normally
    };
    let ir = convert_let_stmt("val", expr);
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
}
