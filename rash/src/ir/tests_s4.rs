#![allow(clippy::expect_used)]
use super::*;
use crate::ast::restricted::{BinaryOp, Literal, UnaryOp};
use crate::ast::{Expr, Function, RestrictedAst, Stmt, Type};
use proptest::prelude::*;
use rstest::*;

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

fn extract_let_value(ir: &ShellIR) -> &ShellValue {
    match ir {
        ShellIR::Sequence(stmts) => match &stmts[0] {
            ShellIR::Let { value, .. } => value,
            other => panic!("Expected Let, got {:?}", other),
        },
        other => panic!("Expected Sequence, got {:?}", other),
    }
}

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
    from_ast(&ast).expect_err("Expected conversion error")
}

// Helper: wrap a single let statement in a main function and convert to IR

/// RED TEST: Optimization disabled should preserve arithmetic
/// Tests that optimize=false keeps Arithmetic unchanged
#[test]
fn test_optimizer_disabled_preserves_arithmetic() {
    let config = crate::models::Config {
        optimize: false,
        ..Default::default()
    };

    let ir = ShellIR::Let {
        name: "sum".to_string(),
        value: ShellValue::Arithmetic {
            op: crate::ir::shell_ir::ArithmeticOp::Add,
            left: Box::new(ShellValue::String("10".to_string())),
            right: Box::new(ShellValue::String("20".to_string())),
        },
        effects: EffectSet::pure(),
    };

    let result = optimize(ir, &config).unwrap();

    // Should be unchanged when optimization is disabled
    match result {
        ShellIR::Let {
            value: ShellValue::Arithmetic { .. },
            ..
        } => {
            // Good - preserved
        }
        _ => panic!("Expected unoptimized Arithmetic"),
    }
}

// ===== convert_expr_to_value coverage tests =====

#[test]
fn test_IR_COV_001_literal_bool() {
    let ir = convert_let_stmt("flag", Expr::Literal(Literal::Bool(true)));
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::Bool(true)));
}

#[test]
fn test_IR_COV_002_literal_u16() {
    let ir = convert_let_stmt("port", Expr::Literal(Literal::U16(8080)));
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::String(s) if s == "8080"));
}

#[test]
fn test_IR_COV_003_literal_i32() {
    let ir = convert_let_stmt("neg", Expr::Literal(Literal::I32(-42)));
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::String(s) if s == "-42"));
}

#[test]
fn test_IR_COV_004_unary_not() {
    let ir = convert_let_stmt(
        "negated",
        Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(Expr::Literal(Literal::Bool(true))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::LogicalNot { .. }));
}

#[test]
fn test_IR_COV_005_unary_neg() {
    let ir = convert_let_stmt(
        "neg",
        Expr::Unary {
            op: UnaryOp::Neg,
            operand: Box::new(Expr::Literal(Literal::U32(5))),
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Sub,
            ..
        } => {}
        other => panic!("Expected Arithmetic Sub, got {:?}", other),
    }
}

#[test]
fn test_IR_COV_006_binary_eq_string() {
    let ir = convert_let_stmt(
        "result",
        Expr::Binary {
            op: BinaryOp::Eq,
            left: Box::new(Expr::Literal(Literal::Str("hello".to_string()))),
            right: Box::new(Expr::Literal(Literal::Str("world".to_string()))),
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::StrEq,
            ..
        } => {}
        other => panic!("Expected StrEq comparison, got {:?}", other),
    }
}

#[test]
fn test_IR_COV_007_binary_eq_numeric() {
    let ir = convert_let_stmt(
        "result",
        Expr::Binary {
            op: BinaryOp::Eq,
            left: Box::new(Expr::Literal(Literal::U32(1))),
            right: Box::new(Expr::Literal(Literal::U32(2))),
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::NumEq,
            ..
        } => {}
        other => panic!("Expected NumEq comparison, got {:?}", other),
    }
}

#[test]
fn test_IR_COV_008_binary_ne_string() {
    let ir = convert_let_stmt(
        "result",
        Expr::Binary {
            op: BinaryOp::Ne,
            left: Box::new(Expr::Literal(Literal::Str("a".to_string()))),
            right: Box::new(Expr::Literal(Literal::Str("b".to_string()))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(
        val,
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::StrNe,
            ..
        }
    ));
}

#[test]
fn test_IR_COV_009_binary_gt() {
    let ir = convert_let_stmt(
        "result",
        Expr::Binary {
            op: BinaryOp::Gt,
            left: Box::new(Expr::Literal(Literal::U32(5))),
            right: Box::new(Expr::Literal(Literal::U32(3))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(
        val,
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::Gt,
            ..
        }
    ));
}

#[test]
fn test_IR_COV_010_binary_ge() {
    let ir = convert_let_stmt(
        "result",
        Expr::Binary {
            op: BinaryOp::Ge,
            left: Box::new(Expr::Literal(Literal::U32(5))),
            right: Box::new(Expr::Literal(Literal::U32(5))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(
        val,
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::Ge,
            ..
        }
    ));
}

#[test]
fn test_IR_COV_011_binary_lt() {
    let ir = convert_let_stmt(
        "result",
        Expr::Binary {
            op: BinaryOp::Lt,
            left: Box::new(Expr::Literal(Literal::U32(3))),
            right: Box::new(Expr::Literal(Literal::U32(5))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(
        val,
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::Lt,
            ..
        }
    ));
}

#[test]
fn test_IR_COV_012_binary_le() {
    let ir = convert_let_stmt(
        "result",
        Expr::Binary {
            op: BinaryOp::Le,
            left: Box::new(Expr::Literal(Literal::U32(3))),
            right: Box::new(Expr::Literal(Literal::U32(3))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(
        val,
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::Le,
            ..
        }
    ));
}

#[test]
fn test_IR_COV_013_binary_sub() {
    let ir = convert_let_stmt(
        "diff",
        Expr::Binary {
            op: BinaryOp::Sub,
            left: Box::new(Expr::Literal(Literal::U32(10))),
            right: Box::new(Expr::Literal(Literal::U32(3))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(
        val,
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Sub,
            ..
        }
    ));
}

#[test]
fn test_IR_COV_014_binary_mul() {
    let ir = convert_let_stmt(
        "product",
        Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expr::Literal(Literal::U32(4))),
            right: Box::new(Expr::Literal(Literal::U32(5))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(
        val,
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Mul,
            ..
        }
    ));
}

#[test]
fn test_IR_COV_015_binary_div() {
    let ir = convert_let_stmt(
        "quotient",
        Expr::Binary {
            op: BinaryOp::Div,
            left: Box::new(Expr::Literal(Literal::U32(10))),
            right: Box::new(Expr::Literal(Literal::U32(2))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(
        val,
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Div,
            ..
        }
    ));
}

#[test]
fn test_IR_COV_016_binary_rem() {
    let ir = convert_let_stmt(
        "remainder",
        Expr::Binary {
            op: BinaryOp::Rem,
            left: Box::new(Expr::Literal(Literal::U32(10))),
            right: Box::new(Expr::Literal(Literal::U32(3))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(
        val,
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Mod,
            ..
        }
    ));
}

#[test]
fn test_IR_COV_017_binary_and() {
    let ir = convert_let_stmt(
        "both",
        Expr::Binary {
            op: BinaryOp::And,
            left: Box::new(Expr::Literal(Literal::Bool(true))),
            right: Box::new(Expr::Literal(Literal::Bool(false))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::LogicalAnd { .. }));
}

#[test]
fn test_IR_COV_018_binary_or() {
    let ir = convert_let_stmt(
        "either",
        Expr::Binary {
            op: BinaryOp::Or,
            left: Box::new(Expr::Literal(Literal::Bool(false))),
            right: Box::new(Expr::Literal(Literal::Bool(true))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::LogicalOr { .. }));
}

#[test]
fn test_IR_COV_019_func_call_env() {
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
        other => panic!("Expected EnvVar, got {:?}", other),
    }
}

#[test]
fn test_IR_COV_020_func_call_env_var_or() {
    let ir = convert_let_stmt(
        "editor",
        Expr::FunctionCall {
            name: "env_var_or".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("EDITOR".to_string())),
                Expr::Literal(Literal::Str("vi".to_string())),
            ],
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::EnvVar { name, default } => {
            assert_eq!(name, "EDITOR");
            assert_eq!(default.as_deref(), Some("vi"));
        }
        other => panic!("Expected EnvVar with default, got {:?}", other),
    }
}

#[test]
fn test_IR_COV_021_func_call_arg() {
    let ir = convert_let_stmt(
        "first",
        Expr::FunctionCall {
            name: "arg".to_string(),
            args: vec![Expr::Literal(Literal::U32(1))],
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::Arg {
            position: Some(1), ..
        } => {}
        other => panic!("Expected Arg(1), got {:?}", other),
    }
}

#[test]
fn test_IR_COV_022_func_call_args() {
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
fn test_IR_COV_023_func_call_arg_count() {
    let ir = convert_let_stmt(
        "count",
        Expr::FunctionCall {
            name: "arg_count".to_string(),
            args: vec![],
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::ArgCount));
}

#[test]
fn test_IR_COV_024_func_call_exit_code() {
    let ir = convert_let_stmt(
        "code",
        Expr::FunctionCall {
            name: "exit_code".to_string(),
            args: vec![],
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::ExitCode));
}

#[test]
fn test_IR_COV_025_positional_args() {
    let ir = convert_let_stmt("all_args", Expr::PositionalArgs);
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::Arg { position: None }));
}

#[test]
fn test_IR_COV_026_func_call_generic() {
    // Non-special function call becomes CommandSubst
    let ir = convert_let_stmt(
        "output",
        Expr::FunctionCall {
            name: "whoami".to_string(),
            args: vec![],
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::CommandSubst(_)));
}

#[test]
fn test_IR_COV_027_env_invalid_var_name() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "bad".to_string(),
                value: Expr::FunctionCall {
                    name: "env".to_string(),
                    args: vec![Expr::Literal(Literal::Str("BAD-NAME".to_string()))],
                },
                declaration: true,
            }],
        }],
        entry_point: "main".to_string(),
    };
    let result = from_ast(&ast);
    assert!(result.is_err());
}
