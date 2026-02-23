//! Tests for expression-to-ShellValue conversion (part 3: tests 039-056).
//!
//! Split from `convert_expr_tests2.rs` to reduce per-file complexity.

#![allow(clippy::unwrap_used)]

use super::*;
use crate::ast::restricted::{BinaryOp, Literal};
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

// ===== FunctionCall: arg with i32 =====

#[test]
fn test_EXPR_VAL_039_func_arg_with_i32_position() {
    let ir = convert_let_stmt(
        "second",
        Expr::FunctionCall {
            name: "arg".to_string(),
            args: vec![Expr::Literal(Literal::I32(2))],
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::Arg { position: Some(2) }));
}

// ===== FunctionCall: arg() with no args (error) =====

#[test]
fn test_EXPR_VAL_040_func_arg_no_args_error() {
    let err = convert_let_stmt_err(
        "bad",
        Expr::FunctionCall {
            name: "arg".to_string(),
            args: vec![],
        },
    );
    let msg = err.to_string();
    assert!(
        msg.contains("requires at least one argument"),
        "Expected arg() requires at least one argument, got: {}",
        msg
    );
}

// ===== FunctionCall: arg() with string arg (error) =====

#[test]
fn test_EXPR_VAL_041_func_arg_string_position_error() {
    let err = convert_let_stmt_err(
        "bad",
        Expr::FunctionCall {
            name: "arg".to_string(),
            args: vec![Expr::Literal(Literal::Str("one".to_string()))],
        },
    );
    let msg = err.to_string();
    assert!(
        msg.contains("integer literal"),
        "Expected integer literal error, got: {}",
        msg
    );
}

// ===== FunctionCall: regular function with args becomes CommandSubst =====

#[test]
fn test_EXPR_VAL_042_func_call_with_args_becomes_command_subst() {
    let ir = convert_let_stmt(
        "output",
        Expr::FunctionCall {
            name: "date".to_string(),
            args: vec![Expr::Literal(Literal::Str("+%Y".to_string()))],
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::CommandSubst(cmd) => {
            assert_eq!(cmd.program, "date");
            assert_eq!(cmd.args.len(), 1);
        }
        other => panic!("Expected CommandSubst(date), got {:?}", other),
    }
}

// ===== MethodCall: env::args().nth() with non-U32 falls through =====

#[test]
fn test_EXPR_VAL_043_method_env_args_nth_non_u32_falls_through() {
    // std::env::args().nth("abc").unwrap() - nth arg is not U32
    let expr = Expr::MethodCall {
        receiver: Box::new(Expr::MethodCall {
            receiver: Box::new(Expr::FunctionCall {
                name: "std::env::args".to_string(),
                args: vec![],
            }),
            method: "nth".to_string(),
            args: vec![Expr::Literal(Literal::Str("abc".to_string()))],
        }),
        method: "unwrap".to_string(),
        args: vec![],
    };
    let ir = convert_let_stmt("val", expr);
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
}

// ===== MethodCall: unwrap_or with non-string default falls through =====

#[test]
fn test_EXPR_VAL_044_method_get_unwrap_or_non_string_default() {
    // args.get(1).unwrap_or(42) - default is not Str
    let expr = Expr::MethodCall {
        receiver: Box::new(Expr::MethodCall {
            receiver: Box::new(Expr::Variable("args".to_string())),
            method: "get".to_string(),
            args: vec![Expr::Literal(Literal::U32(1))],
        }),
        method: "unwrap_or".to_string(),
        args: vec![Expr::Literal(Literal::U32(42))],
    };
    let ir = convert_let_stmt("val", expr);
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
}

// ===== MethodCall: unwrap on non-MethodCall receiver =====

#[test]
fn test_EXPR_VAL_045_method_unwrap_on_non_method_receiver() {
    // variable.unwrap() - receiver is Variable, not MethodCall
    let expr = Expr::MethodCall {
        receiver: Box::new(Expr::Variable("option_val".to_string())),
        method: "unwrap".to_string(),
        args: vec![],
    };
    let ir = convert_let_stmt("val", expr);
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
}

// ===== Binary::Sub (missed in EXPR_VAL_025) =====

#[test]
fn test_EXPR_VAL_046_binary_sub() {
    let ir = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::Sub,
            left: Box::new(Expr::Literal(Literal::U32(10))),
            right: Box::new(Expr::Literal(Literal::U32(3))),
        },
    );
    assert!(matches!(
        extract_let_value(&ir),
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Sub,
            ..
        }
    ));
}

// ===== Binary::Ne with string operands -> StrNe =====

#[test]
fn test_EXPR_VAL_047_binary_ne_string() {
    let ir = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::Ne,
            left: Box::new(Expr::Literal(Literal::Str("hello".to_string()))),
            right: Box::new(Expr::Literal(Literal::Str("world".to_string()))),
        },
    );
    assert!(matches!(
        extract_let_value(&ir),
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::StrNe,
            ..
        }
    ));
}

// ===== FunctionCall: stdlib function as value -> rash_<name> =====

#[test]
fn test_EXPR_VAL_048_func_stdlib_becomes_rash_prefixed_command_subst() {
    let ir = convert_let_stmt(
        "trimmed",
        Expr::FunctionCall {
            name: "string_trim".to_string(),
            args: vec![Expr::Literal(Literal::Str("  hello  ".to_string()))],
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::CommandSubst(cmd) => {
            assert_eq!(cmd.program, "rash_string_trim");
            assert_eq!(cmd.args.len(), 1);
        }
        other => panic!("Expected CommandSubst(rash_string_trim), got {:?}", other),
    }
}

// ===== MethodCall: unwrap_or on non-MethodCall receiver =====

#[test]
fn test_EXPR_VAL_049_method_unwrap_or_on_variable_receiver() {
    // variable.unwrap_or("default") - receiver is Variable, not MethodCall
    let expr = Expr::MethodCall {
        receiver: Box::new(Expr::Variable("maybe_val".to_string())),
        method: "unwrap_or".to_string(),
        args: vec![Expr::Literal(Literal::Str("fallback".to_string()))],
    };
    let ir = convert_let_stmt("val", expr);
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
}

// ===== MethodCall: unwrap_or + args.get(N) where N is not U32 =====

#[test]
fn test_EXPR_VAL_050_method_get_unwrap_or_non_u32_position() {
    // args.get("abc").unwrap_or("default") - position is Str, not U32
    let expr = Expr::MethodCall {
        receiver: Box::new(Expr::MethodCall {
            receiver: Box::new(Expr::Variable("args".to_string())),
            method: "get".to_string(),
            args: vec![Expr::Literal(Literal::Str("abc".to_string()))],
        }),
        method: "unwrap_or".to_string(),
        args: vec![Expr::Literal(Literal::Str("default".to_string()))],
    };
    let ir = convert_let_stmt("val", expr);
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
}

// ===== MethodCall: env::args().nth(N).unwrap_or() where N is not U32 =====

#[test]
fn test_EXPR_VAL_051_method_env_args_nth_unwrap_or_non_u32_position() {
    // std::env::args().nth("x").unwrap_or("default") - nth arg is Str
    let expr = Expr::MethodCall {
        receiver: Box::new(Expr::MethodCall {
            receiver: Box::new(Expr::FunctionCall {
                name: "std::env::args".to_string(),
                args: vec![],
            }),
            method: "nth".to_string(),
            args: vec![Expr::Literal(Literal::Str("x".to_string()))],
        }),
        method: "unwrap_or".to_string(),
        args: vec![Expr::Literal(Literal::Str("default".to_string()))],
    };
    let ir = convert_let_stmt("val", expr);
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
}

// ===== MethodCall: env::args().nth(N).unwrap_or() where default is not Str =====

#[test]
fn test_EXPR_VAL_052_method_env_args_nth_unwrap_or_non_str_default() {
    // std::env::args().nth(0).unwrap_or(42) - default is U32, not Str
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
        args: vec![Expr::Literal(Literal::U32(42))],
    };
    let ir = convert_let_stmt("val", expr);
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
}

// ===== MethodCall: non-env::args().nth().unwrap_or() =====

#[test]
fn test_EXPR_VAL_053_method_nth_unwrap_or_non_env_args_receiver() {
    // other_func().nth(0).unwrap_or("default") - receiver is not std::env::args
    let expr = Expr::MethodCall {
        receiver: Box::new(Expr::MethodCall {
            receiver: Box::new(Expr::FunctionCall {
                name: "some_other_func".to_string(),
                args: vec![],
            }),
            method: "nth".to_string(),
            args: vec![Expr::Literal(Literal::U32(0))],
        }),
        method: "unwrap_or".to_string(),
        args: vec![Expr::Literal(Literal::Str("default".to_string()))],
    };
    let ir = convert_let_stmt("val", expr);
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
}

// ===== MethodCall: env::args().nth().unwrap() where env::args has args =====

#[test]
fn test_EXPR_VAL_054_method_env_args_with_args_nth_unwrap() {
    // std::env::args(42).nth(0).unwrap() - fn_args is not empty
    let expr = Expr::MethodCall {
        receiver: Box::new(Expr::MethodCall {
            receiver: Box::new(Expr::FunctionCall {
                name: "std::env::args".to_string(),
                args: vec![Expr::Literal(Literal::U32(42))],
            }),
            method: "nth".to_string(),
            args: vec![Expr::Literal(Literal::U32(0))],
        }),
        method: "unwrap".to_string(),
        args: vec![],
    };
    let ir = convert_let_stmt("val", expr);
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
}

// ===== MethodCall: nth().unwrap() where receiver is not FunctionCall =====

#[test]
fn test_EXPR_VAL_055_method_nth_unwrap_variable_receiver() {
    // iter.nth(0).unwrap() - inner receiver of nth is Variable, not FunctionCall
    let expr = Expr::MethodCall {
        receiver: Box::new(Expr::MethodCall {
            receiver: Box::new(Expr::Variable("iter".to_string())),
            method: "nth".to_string(),
            args: vec![Expr::Literal(Literal::U32(0))],
        }),
        method: "unwrap".to_string(),
        args: vec![],
    };
    let ir = convert_let_stmt("val", expr);
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
}

// ===== MethodCall: nth().unwrap_or() where inner receiver is Variable =====

#[test]
fn test_EXPR_VAL_056_method_nth_unwrap_or_variable_inner_receiver() {
    // iter.nth(0).unwrap_or("default") - inner receiver of nth is Variable
    let expr = Expr::MethodCall {
        receiver: Box::new(Expr::MethodCall {
            receiver: Box::new(Expr::Variable("iter".to_string())),
            method: "nth".to_string(),
            args: vec![Expr::Literal(Literal::U32(0))],
        }),
        method: "unwrap_or".to_string(),
        args: vec![Expr::Literal(Literal::Str("default".to_string()))],
    };
    let ir = convert_let_stmt("val", expr);
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
}
