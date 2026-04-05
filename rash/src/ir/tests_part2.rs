#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::ast::restricted::{BinaryOp, Literal, UnaryOp};
use crate::ast::{Expr, Function, RestrictedAst, Stmt, Type};
use proptest::prelude::*;
use rstest::*;

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


include!("tests_part2_incl2.rs");
