#![allow(clippy::expect_used)]
use super::*;
use crate::ast::restricted::{BinaryOp, Literal, UnaryOp};
use crate::ast::{Expr, Function, RestrictedAst, Stmt, Type};
use proptest::prelude::*;
use rstest::*;

// Helper: wrap a single let statement in a main function and convert to IR

#[test]
fn test_IR_COV_028_arg_position_zero() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "bad".to_string(),
                value: Expr::FunctionCall {
                    name: "arg".to_string(),
                    args: vec![Expr::Literal(Literal::U32(0))],
                },
                declaration: true,
            }],
        }],
        entry_point: "main".to_string(),
    };
    let result = from_ast(&ast);
    assert!(result.is_err());
}

#[test]
fn test_IR_COV_029_env_no_args() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "bad".to_string(),
                value: Expr::FunctionCall {
                    name: "env".to_string(),
                    args: vec![],
                },
                declaration: true,
            }],
        }],
        entry_point: "main".to_string(),
    };
    let result = from_ast(&ast);
    assert!(result.is_err());
}

#[test]
fn test_IR_COV_030_env_non_string_arg() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "bad".to_string(),
                value: Expr::FunctionCall {
                    name: "env".to_string(),
                    args: vec![Expr::Literal(Literal::U32(42))],
                },
                declaration: true,
            }],
        }],
        entry_point: "main".to_string(),
    };
    let result = from_ast(&ast);
    assert!(result.is_err());
}

#[test]
fn test_IR_COV_031_binary_ne_numeric() {
    let ir = convert_let_stmt(
        "result",
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
fn test_IR_COV_032_method_call_unknown_pattern() {
    // MethodCall that doesn't match any recognized pattern → "unknown"
    let ir = convert_let_stmt(
        "result",
        Expr::MethodCall {
            receiver: Box::new(Expr::Variable("foo".to_string())),
            method: "bar".to_string(),
            args: vec![],
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
}

#[test]
fn test_IR_COV_033_func_call_arg_with_i32() {
    let ir = convert_let_stmt(
        "arg2",
        Expr::FunctionCall {
            name: "arg".to_string(),
            args: vec![Expr::Literal(Literal::I32(2))],
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::Arg {
            position: Some(2), ..
        } => {}
        other => panic!("Expected Arg(2), got {:?}", other),
    }
}

#[test]
fn test_IR_COV_034_env_var_or_non_string_default() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "bad".to_string(),
                value: Expr::FunctionCall {
                    name: "env_var_or".to_string(),
                    args: vec![
                        Expr::Literal(Literal::Str("PATH".to_string())),
                        Expr::Literal(Literal::U32(42)),
                    ],
                },
                declaration: true,
            }],
        }],
        entry_point: "main".to_string(),
    };
    let result = from_ast(&ast);
    assert!(result.is_err());
}

#[test]
fn test_IR_COV_035_arg_no_args() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "bad".to_string(),
                value: Expr::FunctionCall {
                    name: "arg".to_string(),
                    args: vec![],
                },
                declaration: true,
            }],
        }],
        entry_point: "main".to_string(),
    };
    let result = from_ast(&ast);
    assert!(result.is_err());
}

#[test]
fn test_IR_COV_036_arg_string_arg() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "bad".to_string(),
                value: Expr::FunctionCall {
                    name: "arg".to_string(),
                    args: vec![Expr::Literal(Literal::Str("not_a_number".to_string()))],
                },
                declaration: true,
            }],
        }],
        entry_point: "main".to_string(),
    };
    let result = from_ast(&ast);
    assert!(result.is_err());
}

/// Test that return inside while loop in a function produces ShellIR::Return,
/// not ShellIR::Exit with debug format. Regression test for the bug where
/// `return expr` in loop bodies emitted `{value:?}` debug representation.
#[test]
fn test_return_inside_while_in_function_produces_return_ir() {
    use crate::ast::restricted::Parameter;
    // fn find(n: u32) -> u32 { let i = 0; while i < n { return i + 1; } 0 }
    let ast = RestrictedAst {
        functions: vec![
            Function {
                name: "find".to_string(),
                params: vec![Parameter {
                    name: "n".to_string(),
                    param_type: Type::U32,
                }],
                return_type: Type::U32,
                body: vec![
                    Stmt::Let {
                        name: "i".to_string(),
                        value: Expr::Literal(Literal::U32(0)),
                        declaration: true,
                    },
                    Stmt::While {
                        condition: Expr::Binary {
                            op: BinaryOp::Lt,
                            left: Box::new(Expr::Variable("i".to_string())),
                            right: Box::new(Expr::Variable("n".to_string())),
                        },
                        body: vec![Stmt::Return(Some(Expr::Binary {
                            op: BinaryOp::Add,
                            left: Box::new(Expr::Variable("i".to_string())),
                            right: Box::new(Expr::Literal(Literal::U32(1))),
                        }))],
                        max_iterations: Some(1000),
                    },
                    Stmt::Expr(Expr::Literal(Literal::U32(0))),
                ],
            },
            Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Let {
                    name: "r".to_string(),
                    value: Expr::FunctionCall {
                        name: "find".to_string(),
                        args: vec![Expr::Literal(Literal::U32(5))],
                    },
                    declaration: true,
                }],
            },
        ],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).expect("Should convert successfully");

    // The function body's while loop should contain Return, not Exit
    fn contains_return_not_exit(ir: &ShellIR) -> bool {
        match ir {
            ShellIR::Return { .. } => true,
            ShellIR::Exit { .. } => false,
            ShellIR::Sequence(items) => items.iter().any(contains_return_not_exit),
            ShellIR::While { body, .. } => contains_return_not_exit(body),
            ShellIR::For { body, .. } => contains_return_not_exit(body),
            ShellIR::If {
                then_branch,
                else_branch,
                ..
            } => {
                contains_return_not_exit(then_branch)
                    || else_branch
                        .as_ref()
                        .is_some_and(|e| contains_return_not_exit(e))
            }
            ShellIR::Function { body, .. } => contains_return_not_exit(body),
            _ => false,
        }
    }

    // The IR should have a Function with Return inside its while loop
    assert!(
        contains_return_not_exit(&ir),
        "Return inside while loop in function should produce ShellIR::Return, not ShellIR::Exit"
    );
}

#[test]
fn test_let_match_expression_produces_case_with_assignment() {
    use crate::ast::restricted::{MatchArm, Parameter, Pattern};

    // Test: let score = match bucket { 0 => 10, 1 => 5, _ => 1 }
    // Should produce Case with Let assignments, NOT score='unknown'
    let ast = RestrictedAst {
        functions: vec![
            Function {
                name: "classify".to_string(),
                params: vec![Parameter {
                    name: "n".to_string(),
                    param_type: Type::U32,
                }],
                return_type: Type::U32,
                body: vec![
                    Stmt::Let {
                        name: "bucket".to_string(),
                        value: Expr::Binary {
                            op: BinaryOp::Rem,
                            left: Box::new(Expr::Variable("n".to_string())),
                            right: Box::new(Expr::Literal(Literal::U32(4))),
                        },
                        declaration: true,
                    },
                    Stmt::Let {
                        name: "score".to_string(),
                        // Parser produces Expr::Block([Stmt::Match{...}]) for match-in-let
                        value: Expr::Block(vec![Stmt::Match {
                            scrutinee: Expr::Variable("bucket".to_string()),
                            arms: vec![
                                MatchArm {
                                    pattern: Pattern::Literal(Literal::U32(0)),
                                    guard: None,
                                    body: vec![Stmt::Expr(Expr::Binary {
                                        op: BinaryOp::Mul,
                                        left: Box::new(Expr::Variable("n".to_string())),
                                        right: Box::new(Expr::Literal(Literal::U32(10))),
                                    })],
                                },
                                MatchArm {
                                    pattern: Pattern::Literal(Literal::U32(1)),
                                    guard: None,
                                    body: vec![Stmt::Expr(Expr::Literal(Literal::U32(5)))],
                                },
                                MatchArm {
                                    pattern: Pattern::Wildcard,
                                    guard: None,
                                    body: vec![Stmt::Expr(Expr::Literal(Literal::U32(1)))],
                                },
                            ],
                        }]),
                        declaration: true,
                    },
                    Stmt::Return(Some(Expr::Variable("score".to_string()))),
                ],
            },
            Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Let {
                    name: "r".to_string(),
                    value: Expr::FunctionCall {
                        name: "classify".to_string(),
                        args: vec![Expr::Literal(Literal::U32(8))],
                    },
                    declaration: true,
                }],
            },
        ],
        entry_point: "main".to_string(),
    };

    // We just need to verify it doesn't produce ShellValue::String("unknown")
    // by checking that the IR contains a Case node (not just a Let with "unknown")
    let ir = from_ast(&ast).expect("Should convert successfully");

    fn contains_case(ir: &ShellIR) -> bool {
        match ir {
            ShellIR::Case { .. } => true,
            ShellIR::Sequence(items) => items.iter().any(contains_case),
            ShellIR::Function { body, .. } => contains_case(body),
            ShellIR::While { body, .. } => contains_case(body),
            ShellIR::If {
                then_branch,
                else_branch,
                ..
            } => {
                contains_case(then_branch) || else_branch.as_ref().is_some_and(|e| contains_case(e))
            }
            _ => false,
        }
    }

    assert!(
        contains_case(&ir),
        "let x = match y {{ ... }} should produce ShellIR::Case, not Let with 'unknown'"
    );
}

#[test]
fn test_exclusive_for_range_with_variable_end_subtracts_one() {
    use crate::ast::restricted::Pattern;

    // Test: for i in 0..n should produce seq 0 $((n-1)), not seq 0 $n
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::For {
                pattern: Pattern::Variable("i".to_string()),
                iter: Expr::Range {
                    start: Box::new(Expr::Literal(Literal::U32(0))),
                    end: Box::new(Expr::Variable("n".to_string())),
                    inclusive: false,
                },
                body: vec![Stmt::Expr(Expr::Variable("i".to_string()))],
                max_iterations: Some(1000),
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).expect("Should convert successfully");

    // The For node's end value should be Arithmetic { Sub, Variable("n"), String("1") }
    fn has_adjusted_end(ir: &ShellIR) -> bool {
        match ir {
            ShellIR::For { end, .. } => matches!(
                end,
                ShellValue::Arithmetic {
                    op: crate::ir::shell_ir::ArithmeticOp::Sub,
                    ..
                }
            ),
            ShellIR::Sequence(items) => items.iter().any(has_adjusted_end),
            _ => false,
        }
    }

    assert!(
        has_adjusted_end(&ir),
        "for i in 0..n should produce end=$((n-1)), not end=$n"
    );
}

#[test]
fn test_nested_match_in_match_arm_produces_nested_case() {
    // Regression: `let next = match state { 0 => match bit { ... }, ... }`
    // should produce nested Case statements, not flat assignments to '0'.
    let source = r#"
fn dispatch(state: u32, bit: u32) -> u32 {
    let next = match state {
        0 => match bit { 0 => 10, _ => 20, },
        _ => match bit { 0 => 30, _ => 40, },
    };
    return next;
}
fn main() { println!("{}", dispatch(0, 1)); }
"#;

    let ast = crate::services::parser::parse(source).expect("should parse");
    let ir = super::from_ast(&ast).expect("should lower");

    assert!(
        ir_has_nested_case(&ir),
        "nested match-in-match-arm should produce nested Case IR"
    );
}

#[test]
fn test_if_else_expression_in_match_block_arm_produces_if_assignment() {
    let source = r#"
fn categorize(x: u32) -> u32 {
    let r = match x % 3 {
        0 => {
            let half = x / 2;
            if half > 5 { half * 10 } else { half }
        },
        _ => x,
    };
    return r;
}
fn main() { println!("{}", categorize(12)); }
"#;

    let ast = crate::services::parser::parse(source).expect("should parse");
    let ir = super::from_ast(&ast).expect("should lower");

    assert!(
        ir_has_if_in_case(&ir),
        "if-else expression in match block arm should produce If IR inside Case arm"
    );
}

/// Walk the IR tree to find a nested Case inside a Case arm
fn ir_has_nested_case(ir: &super::ShellIR) -> bool {
    match ir {
        super::ShellIR::Case { arms, .. } => arms.iter().any(|arm| {
            matches!(&*arm.body, super::ShellIR::Case { .. }) || ir_has_nested_case(&arm.body)
        }),
        super::ShellIR::Sequence(stmts) => stmts.iter().any(ir_has_nested_case),
        super::ShellIR::Function { body, .. } => ir_has_nested_case(body),
        _ => false,
    }
}

/// Walk IR tree to find If inside a Case arm
fn ir_has_if_in_case(ir: &super::ShellIR) -> bool {
    match ir {
        super::ShellIR::Case { arms, .. } => arms.iter().any(|arm| ir_has_if_inside(&arm.body)),
        super::ShellIR::Sequence(stmts) => stmts.iter().any(ir_has_if_in_case),
        super::ShellIR::Function { body, .. } => ir_has_if_in_case(body),
        _ => false,
    }
}

/// Check if IR node contains an If statement
fn ir_has_if_inside(ir: &super::ShellIR) -> bool {
    match ir {
        super::ShellIR::If { .. } => true,
        super::ShellIR::Sequence(stmts) => stmts.iter().any(ir_has_if_inside),
        _ => false,
    }
}
