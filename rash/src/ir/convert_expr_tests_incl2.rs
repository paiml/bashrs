fn test_EXPR_VAL_021_unary_neg() {
    let ir = convert_let_stmt(
        "neg",
        Expr::Unary {
            op: UnaryOp::Neg,
            operand: Box::new(Expr::Literal(Literal::U32(7))),
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Sub,
            left,
            right,
        } => {
            // Negation is 0 - operand
            assert!(matches!(**left, ShellValue::String(ref s) if s == "0"));
            assert!(matches!(**right, ShellValue::String(ref s) if s == "7"));
        }
        other => panic!("Expected Arithmetic(Sub, 0, 7), got {:?}", other),
    }
}

// ===== Binary: comparison ops =====

#[test]
fn test_EXPR_VAL_022_binary_eq_string_vs_numeric() {
    // String operands -> StrEq
    let ir_str = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::Eq,
            left: Box::new(Expr::Literal(Literal::Str("abc".to_string()))),
            right: Box::new(Expr::Literal(Literal::Str("def".to_string()))),
        },
    );
    let val_str = extract_let_value(&ir_str);
    assert!(matches!(
        val_str,
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::StrEq,
            ..
        }
    ));

    // Numeric operands -> NumEq
    let ir_num = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::Eq,
            left: Box::new(Expr::Literal(Literal::U32(5))),
            right: Box::new(Expr::Literal(Literal::U32(5))),
        },
    );
    let val_num = extract_let_value(&ir_num);
    assert!(matches!(
        val_num,
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::NumEq,
            ..
        }
    ));
}

#[test]
fn test_EXPR_VAL_023_binary_ne() {
    let ir = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::Ne,
            left: Box::new(Expr::Literal(Literal::U32(1))),
            right: Box::new(Expr::Literal(Literal::U32(2))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(
        val,
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::NumNe,
            ..
        }
    ));
}

#[test]
fn test_EXPR_VAL_024_binary_all_comparison_ops() {
    // Gt
    let ir = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::Gt,
            left: Box::new(Expr::Literal(Literal::U32(10))),
            right: Box::new(Expr::Literal(Literal::U32(5))),
        },
    );
    assert!(matches!(
        extract_let_value(&ir),
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::Gt,
            ..
        }
    ));

    // Ge
    let ir = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::Ge,
            left: Box::new(Expr::Literal(Literal::U32(5))),
            right: Box::new(Expr::Literal(Literal::U32(5))),
        },
    );
    assert!(matches!(
        extract_let_value(&ir),
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::Ge,
            ..
        }
    ));

    // Lt
    let ir = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::Lt,
            left: Box::new(Expr::Literal(Literal::U32(3))),
            right: Box::new(Expr::Literal(Literal::U32(5))),
        },
    );
    assert!(matches!(
        extract_let_value(&ir),
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::Lt,
            ..
        }
    ));

    // Le
    let ir = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::Le,
            left: Box::new(Expr::Literal(Literal::U32(3))),
            right: Box::new(Expr::Literal(Literal::U32(3))),
        },
    );
    assert!(matches!(
        extract_let_value(&ir),
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::Le,
            ..
        }
    ));
}

// ===== Binary: arithmetic ops =====

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
