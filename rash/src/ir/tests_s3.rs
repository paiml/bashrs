#![allow(clippy::expect_used)]
use super::*;
use crate::ast::restricted::{BinaryOp, Literal, UnaryOp};
use crate::ast::{Expr, Function, RestrictedAst, Stmt, Type};
use proptest::prelude::*;
use rstest::*;

// Helper: wrap a single let statement in a main function and convert to IR

/// RED TEST: arg(1) call should convert to Arg variant in IR
/// Tests that arg(1) is properly recognized and converted to ShellValue::Arg
#[test]
fn test_arg_call_converts_to_ir() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "first".to_string(),
                value: Expr::FunctionCall {
                    name: "arg".to_string(),
                    args: vec![Expr::Literal(Literal::U32(1))],
                },
                declaration: true,
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    match ir {
        ShellIR::Sequence(stmts) => {
            match &stmts[0] {
                ShellIR::Let { name, value, .. } => {
                    assert_eq!(name, "first");
                    // RED: This will fail until we implement Arg variant
                    match value {
                        ShellValue::Arg { position } => {
                            assert_eq!(position, &Some(1));
                        }
                        other => panic!("Expected Arg, got {:?}", other),
                    }
                }
                _ => panic!("Expected Let statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

/// RED TEST: args() call should convert to Arg variant with None position
/// Tests that args() converts to Arg { position: None } (representing $@)
#[test]
fn test_args_call_converts_to_ir() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "all".to_string(),
                value: Expr::FunctionCall {
                    name: "args".to_string(),
                    args: vec![],
                },
                declaration: true,
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    match ir {
        ShellIR::Sequence(stmts) => {
            match &stmts[0] {
                ShellIR::Let { name, value, .. } => {
                    assert_eq!(name, "all");
                    // RED: This will fail until we implement Arg variant with None
                    match value {
                        ShellValue::Arg { position } => {
                            assert_eq!(position, &None);
                        }
                        other => panic!("Expected Arg with None position, got {:?}", other),
                    }
                }
                _ => panic!("Expected Let statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

/// RED TEST: arg_count() call should convert to ArgCount variant
/// Tests that arg_count() is properly recognized and converted to ShellValue::ArgCount
#[test]
fn test_arg_count_converts_to_ir() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "count".to_string(),
                value: Expr::FunctionCall {
                    name: "arg_count".to_string(),
                    args: vec![],
                },
                declaration: true,
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    match ir {
        ShellIR::Sequence(stmts) => {
            match &stmts[0] {
                ShellIR::Let { name, value, .. } => {
                    assert_eq!(name, "count");
                    // RED: This will fail until we implement ArgCount variant
                    match value {
                        ShellValue::ArgCount => {
                            // Success!
                        }
                        other => panic!("Expected ArgCount, got {:?}", other),
                    }
                }
                _ => panic!("Expected Let statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

/// RED TEST: arg(0) should be rejected (validation)
/// Tests that arg(0) is rejected because shell arguments start at $1
#[test]
fn test_arg_rejects_zero_position() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "x".to_string(),
                value: Expr::FunctionCall {
                    name: "arg".to_string(),
                    args: vec![Expr::Literal(Literal::U32(0))],
                },
                declaration: true,
            }],
        }],
        entry_point: "main".to_string(),
    };

    // RED: This will fail until we implement position validation
    let result = from_ast(&ast);
    assert!(result.is_err(), "arg(0) should be rejected");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("position must be >= 1")
            || error_msg.contains("position")
            || error_msg.contains("1"),
        "Error message should mention position requirement, got: {}",
        error_msg
    );
}

// ============= Sprint 27c: Exit Code Handling - RED PHASE =============

/// RED TEST: exit_code() call should convert to ExitCode variant in IR
/// Tests that exit_code() is properly recognized and converted to ShellValue::ExitCode
#[test]
fn test_exit_code_call_converts_to_ir() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "status".to_string(),
                value: Expr::FunctionCall {
                    name: "exit_code".to_string(),
                    args: vec![],
                },
                declaration: true,
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    match ir {
        ShellIR::Sequence(stmts) => {
            match &stmts[0] {
                ShellIR::Let { name, value, .. } => {
                    assert_eq!(name, "status");
                    // RED: This will fail until we implement ExitCode variant
                    match value {
                        ShellValue::ExitCode => {
                            // Success!
                        }
                        other => panic!("Expected ExitCode, got {:?}", other),
                    }
                }
                _ => panic!("Expected Let statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

/// RED TEST: exit_code() in comparison context
/// Tests that exit_code() works in if condition comparisons
#[test]
fn test_exit_code_in_comparison() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::If {
                condition: Expr::Binary {
                    op: BinaryOp::Eq,
                    left: Box::new(Expr::FunctionCall {
                        name: "exit_code".to_string(),
                        args: vec![],
                    }),
                    right: Box::new(Expr::Literal(Literal::Str("0".to_string()))),
                },
                then_block: vec![Stmt::Expr(Expr::Literal(Literal::Str(
                    "success".to_string(),
                )))],
                else_block: None,
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    // RED: This will fail until ExitCode variant exists
    match ir {
        ShellIR::Sequence(stmts) => {
            match &stmts[0] {
                ShellIR::If { test, .. } => {
                    // Should contain Comparison with ExitCode on the left
                    match test {
                        ShellValue::Comparison { left, .. } => {
                            assert!(
                                matches!(**left, ShellValue::ExitCode),
                                "Expected ExitCode in comparison"
                            );
                        }
                        _ => panic!("Expected Comparison in if condition"),
                    }
                }
                _ => panic!("Expected If statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

/// RED TEST: Multiple exit_code() calls in sequence
/// Tests that multiple exit_code() calls work correctly
#[test]
fn test_multiple_exit_code_calls() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![
                Stmt::Let {
                    name: "status1".to_string(),
                    value: Expr::FunctionCall {
                        name: "exit_code".to_string(),
                        args: vec![],
                    },
                    declaration: true,
                },
                Stmt::Let {
                    name: "status2".to_string(),
                    value: Expr::FunctionCall {
                        name: "exit_code".to_string(),
                        args: vec![],
                    },
                    declaration: true,
                },
            ],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    // RED: This will fail until ExitCode variant exists
    match ir {
        ShellIR::Sequence(stmts) => {
            assert_eq!(stmts.len(), 2);

            // Check both calls convert to ExitCode
            match &stmts[0] {
                ShellIR::Let { value, .. } => {
                    assert!(
                        matches!(value, ShellValue::ExitCode),
                        "First exit_code() should be ExitCode variant"
                    );
                }
                _ => panic!("Expected Let statement"),
            }

            match &stmts[1] {
                ShellIR::Let { value, .. } => {
                    assert!(
                        matches!(value, ShellValue::ExitCode),
                        "Second exit_code() should be ExitCode variant"
                    );
                }
                _ => panic!("Expected Let statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

// ============= Optimizer Enhancement: Arithmetic Constant Folding - RED PHASE =============

/// RED TEST: Arithmetic addition constant folding
/// Tests that $((10 + 20)) → "30" at compile time
#[test]
fn test_optimizer_arithmetic_addition_folding() {
    let config = crate::models::Config::default(); // optimize = true

    let ir = ShellIR::Let {
        name: "sum".to_string(),
        value: ShellValue::Arithmetic {
            op: crate::ir::shell_ir::ArithmeticOp::Add,
            left: Box::new(ShellValue::String("10".to_string())),
            right: Box::new(ShellValue::String("20".to_string())),
        },
        effects: EffectSet::pure(),
    };

    let optimized = optimize(ir, &config).unwrap();

    // Should fold to constant "30"
    match optimized {
        ShellIR::Let {
            value: ShellValue::String(s),
            ..
        } => {
            assert_eq!(s, "30", "10 + 20 should fold to 30");
        }
        _ => panic!("Expected optimized constant string"),
    }
}

/// RED TEST: Arithmetic subtraction constant folding
/// Tests that $((50 - 12)) → "38" at compile time
#[test]
fn test_optimizer_arithmetic_subtraction_folding() {
    let config = crate::models::Config::default();

    let ir = ShellIR::Let {
        name: "diff".to_string(),
        value: ShellValue::Arithmetic {
            op: crate::ir::shell_ir::ArithmeticOp::Sub,
            left: Box::new(ShellValue::String("50".to_string())),
            right: Box::new(ShellValue::String("12".to_string())),
        },
        effects: EffectSet::pure(),
    };

    let optimized = optimize(ir, &config).unwrap();

    match optimized {
        ShellIR::Let {
            value: ShellValue::String(s),
            ..
        } => {
            assert_eq!(s, "38", "50 - 12 should fold to 38");
        }
        _ => panic!("Expected optimized constant string"),
    }
}

/// RED TEST: Arithmetic multiplication constant folding
/// Tests that $((10 * 1024 * 1024)) → "10485760" (10MB) at compile time
#[test]
fn test_optimizer_arithmetic_multiplication_folding() {
    let config = crate::models::Config::default();

    // First multiply: 10 * 1024 = 10240
    let inner_mul = ShellValue::Arithmetic {
        op: crate::ir::shell_ir::ArithmeticOp::Mul,
        left: Box::new(ShellValue::String("10".to_string())),
        right: Box::new(ShellValue::String("1024".to_string())),
    };

    // Second multiply: (10 * 1024) * 1024 = 10485760
    let ir = ShellIR::Let {
        name: "bytes".to_string(),
        value: ShellValue::Arithmetic {
            op: crate::ir::shell_ir::ArithmeticOp::Mul,
            left: Box::new(inner_mul),
            right: Box::new(ShellValue::String("1024".to_string())),
        },
        effects: EffectSet::pure(),
    };

    let optimized = optimize(ir, &config).unwrap();

    match optimized {
        ShellIR::Let {
            value: ShellValue::String(s),
            ..
        } => {
            assert_eq!(s, "10485760", "10 * 1024 * 1024 should fold to 10485760");
        }
        _ => panic!("Expected optimized constant string"),
    }
}

/// RED TEST: Arithmetic division constant folding
/// Tests that $((100 / 5)) → "20" at compile time
#[test]
fn test_optimizer_arithmetic_division_folding() {
    let config = crate::models::Config::default();

    let ir = ShellIR::Let {
        name: "quotient".to_string(),
        value: ShellValue::Arithmetic {
            op: crate::ir::shell_ir::ArithmeticOp::Div,
            left: Box::new(ShellValue::String("100".to_string())),
            right: Box::new(ShellValue::String("5".to_string())),
        },
        effects: EffectSet::pure(),
    };

    let optimized = optimize(ir, &config).unwrap();

    match optimized {
        ShellIR::Let {
            value: ShellValue::String(s),
            ..
        } => {
            assert_eq!(s, "20", "100 / 5 should fold to 20");
        }
        _ => panic!("Expected optimized constant string"),
    }
}

/// RED TEST: Arithmetic with non-constant should NOT fold
/// Tests that $((x + 10)) stays as Arithmetic (cannot fold with variable)
#[test]
fn test_optimizer_arithmetic_with_variable_no_fold() {
    let config = crate::models::Config::default();

    let ir = ShellIR::Let {
        name: "result".to_string(),
        value: ShellValue::Arithmetic {
            op: crate::ir::shell_ir::ArithmeticOp::Add,
            left: Box::new(ShellValue::Variable("x".to_string())),
            right: Box::new(ShellValue::String("10".to_string())),
        },
        effects: EffectSet::pure(),
    };

    let optimized = optimize(ir, &config).unwrap();

    // Should NOT fold (variable involved)
    match optimized {
        ShellIR::Let {
            value: ShellValue::Arithmetic { .. },
            ..
        } => {
            // Good - still Arithmetic, not folded
        }
        _ => panic!("Expected unoptimized Arithmetic (variable involved)"),
    }
}
