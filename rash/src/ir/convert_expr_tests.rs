//! Tests for expression-to-ShellValue conversion (part 1: tests 001-028).
//!
//! Extracted from `mod.rs` to reduce per-file complexity.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::ast::restricted::{BinaryOp, Literal, UnaryOp};
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

/// Helper: wrap a single let statement and expect conversion to fail
fn convert_let_stmt_err(name: &str, value: Expr) -> crate::models::Error {
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
    from_ast(&ast).expect_err("IR conversion should fail")
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

// ===== Literal branches =====

#[test]
fn test_EXPR_VAL_001_literal_bool_true() {
    let ir = convert_let_stmt("flag", Expr::Literal(Literal::Bool(true)));
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::Bool(true)));
}

#[test]
fn test_EXPR_VAL_002_literal_bool_false() {
    let ir = convert_let_stmt("flag", Expr::Literal(Literal::Bool(false)));
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::Bool(false)));
}

#[test]
fn test_EXPR_VAL_003_literal_u16() {
    let ir = convert_let_stmt("port", Expr::Literal(Literal::U16(443)));
    let val = extract_let_value(&ir);
    match val {
        ShellValue::String(s) => assert_eq!(s, "443"),
        other => panic!("Expected String(\"443\"), got {:?}", other),
    }
}

#[test]
fn test_EXPR_VAL_004_literal_u32() {
    let ir = convert_let_stmt("count", Expr::Literal(Literal::U32(100)));
    let val = extract_let_value(&ir);
    match val {
        ShellValue::String(s) => assert_eq!(s, "100"),
        other => panic!("Expected String(\"100\"), got {:?}", other),
    }
}

#[test]
fn test_EXPR_VAL_005_literal_i32() {
    let ir = convert_let_stmt("offset", Expr::Literal(Literal::I32(-99)));
    let val = extract_let_value(&ir);
    match val {
        ShellValue::String(s) => assert_eq!(s, "-99"),
        other => panic!("Expected String(\"-99\"), got {:?}", other),
    }
}

#[test]
fn test_EXPR_VAL_006_literal_str() {
    let ir = convert_let_stmt(
        "msg",
        Expr::Literal(Literal::Str("hello world".to_string())),
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::String(s) => assert_eq!(s, "hello world"),
        other => panic!("Expected String(\"hello world\"), got {:?}", other),
    }
}

// ===== Variable =====

#[test]
fn test_EXPR_VAL_007_variable() {
    let ir = convert_let_stmt("alias", Expr::Variable("original".to_string()));
    let val = extract_let_value(&ir);
    match val {
        ShellValue::Variable(name) => assert_eq!(name, "original"),
        other => panic!("Expected Variable(\"original\"), got {:?}", other),
    }
}

// ===== FunctionCall: stdlib functions =====

#[test]
fn test_EXPR_VAL_008_func_env() {
    let ir = convert_let_stmt(
        "home",
        Expr::FunctionCall {
            name: "env".to_string(),
            args: vec![Expr::Literal(Literal::Str("HOME".to_string()))],
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::EnvVar { name, default } => {
            assert_eq!(name, "HOME");
            assert!(default.is_none());
        }
        other => panic!("Expected EnvVar {{ HOME, None }}, got {:?}", other),
    }
}

#[test]
fn test_EXPR_VAL_009_func_env_var_or() {
    let ir = convert_let_stmt(
        "editor",
        Expr::FunctionCall {
            name: "env_var_or".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("EDITOR".to_string())),
                Expr::Literal(Literal::Str("nano".to_string())),
            ],
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::EnvVar { name, default } => {
            assert_eq!(name, "EDITOR");
            assert_eq!(default.as_deref(), Some("nano"));
        }
        other => panic!("Expected EnvVar {{ EDITOR, Some(nano) }}, got {:?}", other),
    }
}

#[test]
fn test_EXPR_VAL_010_func_arg() {
    let ir = convert_let_stmt(
        "first",
        Expr::FunctionCall {
            name: "arg".to_string(),
            args: vec![Expr::Literal(Literal::U32(1))],
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::Arg { position: Some(1) }));
}

#[test]
fn test_EXPR_VAL_011_func_args() {
    let ir = convert_let_stmt(
        "all",
        Expr::FunctionCall {
            name: "args".to_string(),
            args: vec![],
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::Arg { position: None }));
}

#[test]
fn test_EXPR_VAL_012_func_arg_count() {
    let ir = convert_let_stmt(
        "n",
        Expr::FunctionCall {
            name: "arg_count".to_string(),
            args: vec![],
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::ArgCount));
}

#[test]
fn test_EXPR_VAL_013_func_exit_code() {
    let ir = convert_let_stmt(
        "rc",
        Expr::FunctionCall {
            name: "exit_code".to_string(),
            args: vec![],
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::ExitCode));
}

// ===== FunctionCall: validation errors =====

#[test]
fn test_EXPR_VAL_014_env_no_args_error() {
    let err = convert_let_stmt_err(
        "bad",
        Expr::FunctionCall {
            name: "env".to_string(),
            args: vec![],
        },
    );
    let msg = err.to_string();
    assert!(
        msg.contains("requires at least one argument"),
        "Expected 'requires at least one argument', got: {}",
        msg
    );
}

#[test]
fn test_EXPR_VAL_015_env_non_string_arg_error() {
    let err = convert_let_stmt_err(
        "bad",
        Expr::FunctionCall {
            name: "env".to_string(),
            args: vec![Expr::Literal(Literal::U32(42))],
        },
    );
    let msg = err.to_string();
    assert!(
        msg.contains("string literal"),
        "Expected error about string literal, got: {}",
        msg
    );
}

#[test]
fn test_EXPR_VAL_016_env_invalid_var_name_special_chars() {
    let err = convert_let_stmt_err(
        "bad",
        Expr::FunctionCall {
            name: "env".to_string(),
            args: vec![Expr::Literal(Literal::Str("BAD-NAME".to_string()))],
        },
    );
    let msg = err.to_string();
    assert!(
        msg.contains("Invalid environment variable name"),
        "Expected invalid var name error, got: {}",
        msg
    );
}

#[test]
fn test_EXPR_VAL_017_env_var_or_non_string_default_error() {
    let err = convert_let_stmt_err(
        "bad",
        Expr::FunctionCall {
            name: "env_var_or".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("PATH".to_string())),
                Expr::Literal(Literal::U32(99)),
            ],
        },
    );
    let msg = err.to_string();
    assert!(
        msg.contains("string literal for default value"),
        "Expected default value error, got: {}",
        msg
    );
}

#[test]
fn test_EXPR_VAL_018_arg_zero_position_error() {
    let err = convert_let_stmt_err(
        "bad",
        Expr::FunctionCall {
            name: "arg".to_string(),
            args: vec![Expr::Literal(Literal::U32(0))],
        },
    );
    let msg = err.to_string();
    assert!(
        msg.contains("position must be >= 1"),
        "Expected position >= 1 error, got: {}",
        msg
    );
}

// ===== FunctionCall: regular (non-stdlib) =====

#[test]
fn test_EXPR_VAL_019_func_call_regular_becomes_command_subst() {
    let ir = convert_let_stmt(
        "output",
        Expr::FunctionCall {
            name: "whoami".to_string(),
            args: vec![],
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::CommandSubst(cmd) => {
            assert_eq!(cmd.program, "whoami");
            assert!(cmd.args.is_empty());
        }
        other => panic!("Expected CommandSubst(whoami), got {:?}", other),
    }
}

// ===== Unary: Not, Neg =====

#[test]
fn test_EXPR_VAL_020_unary_not() {
    let ir = convert_let_stmt(
        "negated",
        Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(Expr::Literal(Literal::Bool(false))),
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::LogicalNot { operand } => {
            assert!(matches!(**operand, ShellValue::Bool(false)));
        }
        other => panic!("Expected LogicalNot, got {:?}", other),
    }
}

#[test]

include!("convert_expr_tests_tests_EXPR.rs");
