#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::restricted::*;

// ============================================================================
// RestrictedAst: validate coverage
// ============================================================================

#[test]
fn test_validate_ast_with_multiple_functions() {
    let ast = RestrictedAst {
        entry_point: "main".to_string(),
        functions: vec![
            Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Expr(Expr::FunctionCall {
                    name: "helper".to_string(),
                    args: vec![],
                })],
            },
            Function {
                name: "helper".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![],
            },
        ],
    };
    assert!(ast.validate().is_ok());
}

#[test]
fn test_validate_ast_invalid_function_fails() {
    let ast = RestrictedAst {
        entry_point: "main".to_string(),
        functions: vec![
            Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![],
            },
            Function {
                name: "".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![],
            },
        ],
    };
    assert!(ast.validate().is_err());
}

#[test]
fn test_validate_ast_recursive_and_external_calls_allowed() {
    // Recursive call
    let ast = RestrictedAst {
        entry_point: "factorial".to_string(),
        functions: vec![Function {
            name: "factorial".to_string(),
            params: vec![Parameter {
                name: "n".to_string(),
                param_type: Type::U32,
            }],
            return_type: Type::U32,
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "factorial".to_string(),
                args: vec![Expr::Variable("n".to_string())],
            })],
        }],
    };
    assert!(ast.validate().is_ok());

    // External call
    let ast2 = RestrictedAst {
        entry_point: "main".to_string(),
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "external".to_string(),
                args: vec![],
            })],
        }],
    };
    assert!(ast2.validate().is_ok());
}

// ============================================================================
// Stmt::validate edge cases
// ============================================================================

#[test]
fn test_stmt_let_unsafe_names() {
    for (name, expected_substr) in [
        ("x\0y", "Null"),
        ("$var", "Unsafe"),
        ("`cmd`", "Unsafe"),
        ("x\\y", "Unsafe"),
    ] {
        let stmt = Stmt::Let {
            name: name.to_string(),
            value: Expr::Literal(Literal::U32(0)),
            declaration: true,
        };
        let err = stmt.validate().unwrap_err();
        assert!(err.contains(expected_substr), "name={name}: {err}");
    }
}

#[test]
fn test_stmt_expr_valid_and_invalid() {
    assert!(Stmt::Expr(Expr::Variable("ok".to_string()))
        .validate()
        .is_ok());
    assert!(Stmt::Expr(Expr::Variable("".to_string()))
        .validate()
        .is_err());
}

#[test]
fn test_stmt_return_variants() {
    assert!(Stmt::Return(Some(Expr::Literal(Literal::U32(0))))
        .validate()
        .is_ok());
    assert!(
        Stmt::Return(Some(Expr::Literal(Literal::Str("ok\0".to_string()))))
            .validate()
            .is_err()
    );
    assert!(Stmt::Return(None).validate().is_ok());
}

#[test]
fn test_stmt_if_validation_branches() {
    // Invalid condition
    assert!(Stmt::If {
        condition: Expr::Variable("".to_string()),
        then_block: vec![],
        else_block: None,
    }
    .validate()
    .is_err());

    // Invalid then block
    assert!(Stmt::If {
        condition: Expr::Literal(Literal::Bool(true)),
        then_block: vec![Stmt::Let {
            name: "".to_string(),
            value: Expr::Literal(Literal::U32(0)),
            declaration: true
        }],
        else_block: None,
    }
    .validate()
    .is_err());

    // Invalid else block
    assert!(Stmt::If {
        condition: Expr::Literal(Literal::Bool(true)),
        then_block: vec![],
        else_block: Some(vec![Stmt::Let {
            name: "".to_string(),
            value: Expr::Literal(Literal::U32(0)),
            declaration: true
        }]),
    }
    .validate()
    .is_err());
}

#[test]
fn test_stmt_match_validation_branches() {
    // Invalid scrutinee
    assert!(Stmt::Match {
        scrutinee: Expr::Variable("".to_string()),
        arms: vec![]
    }
    .validate()
    .is_err());
    // Invalid pattern in arm
    assert!(Stmt::Match {
        scrutinee: Expr::Variable("x".to_string()),
        arms: vec![MatchArm {
            pattern: Pattern::Variable("".to_string()),
            guard: None,
            body: vec![]
        }],
    }
    .validate()
    .is_err());
    // Invalid guard
    assert!(Stmt::Match {
        scrutinee: Expr::Variable("x".to_string()),
        arms: vec![MatchArm {
            pattern: Pattern::Wildcard,
            guard: Some(Expr::Variable("".to_string())),
            body: vec![]
        }],
    }
    .validate()
    .is_err());
    // Invalid body
    assert!(Stmt::Match {
        scrutinee: Expr::Variable("x".to_string()),
        arms: vec![MatchArm {
            pattern: Pattern::Wildcard,
            guard: None,
            body: vec![Stmt::Let {
                name: "".to_string(),
                value: Expr::Literal(Literal::U32(0)),
                declaration: true
            }],
        }],
    }
    .validate()
    .is_err());
}

#[test]
fn test_stmt_for_validation_branches() {
    // Valid
    assert!(Stmt::For {
        pattern: Pattern::Variable("i".to_string()),
        iter: Expr::Literal(Literal::U32(0)),
        body: vec![],
        max_iterations: Some(100),
    }
    .validate()
    .is_ok());
    // No max_iterations
    assert!(Stmt::For {
        pattern: Pattern::Variable("i".to_string()),
        iter: Expr::Literal(Literal::U32(0)),
        body: vec![],
        max_iterations: None,
    }
    .validate()
    .is_err());
    // Invalid pattern
    assert!(Stmt::For {
        pattern: Pattern::Variable("".to_string()),
        iter: Expr::Literal(Literal::U32(0)),
        body: vec![],
        max_iterations: Some(10),
    }
    .validate()
    .is_err());
    // Invalid iter
    assert!(Stmt::For {
        pattern: Pattern::Variable("i".to_string()),
        iter: Expr::Variable("".to_string()),
        body: vec![],
        max_iterations: Some(10),
    }
    .validate()
    .is_err());
    // Invalid body
    assert!(Stmt::For {
        pattern: Pattern::Variable("i".to_string()),
        iter: Expr::Literal(Literal::U32(0)),
        body: vec![Stmt::Let {
            name: "".to_string(),
            value: Expr::Literal(Literal::U32(0)),
            declaration: true
        }],
        max_iterations: Some(10),
    }
    .validate()
    .is_err());
}

#[test]
fn test_stmt_while_validation_branches() {
    assert!(Stmt::While {
        condition: Expr::Literal(Literal::Bool(true)),
        body: vec![Stmt::Break],
        max_iterations: Some(100),
    }
    .validate()
    .is_ok());
    // No max_iterations
    assert!(Stmt::While {
        condition: Expr::Literal(Literal::Bool(true)),
        body: vec![],
        max_iterations: None,
    }
    .validate()
    .is_err());
    // Invalid condition
    assert!(Stmt::While {
        condition: Expr::Variable("".to_string()),
        body: vec![],
        max_iterations: Some(10),
    }
    .validate()
    .is_err());
    // Invalid body
    assert!(Stmt::While {
        condition: Expr::Literal(Literal::Bool(true)),
        body: vec![Stmt::Let {
            name: "".to_string(),
            value: Expr::Literal(Literal::U32(0)),
            declaration: true
        }],
        max_iterations: Some(10),
    }
    .validate()
    .is_err());
}

// ============================================================================
// Stmt::collect_function_calls coverage
// ============================================================================

#[test]
fn test_stmt_collect_calls_variants() {
    // Let
    let mut calls = vec![];
    Stmt::Let {
        name: "x".to_string(),
        value: Expr::FunctionCall {
            name: "foo".to_string(),
            args: vec![],
        },
        declaration: true,
    }
    .collect_function_calls(&mut calls);
    assert_eq!(calls, vec!["foo"]);

    // Return(Some)
    let mut calls = vec![];
    Stmt::Return(Some(Expr::FunctionCall {
        name: "compute".to_string(),
        args: vec![],
    }))
    .collect_function_calls(&mut calls);
    assert_eq!(calls, vec!["compute"]);

    // Return(None), Break, Continue produce no calls
    let mut calls = vec![];
    Stmt::Return(None).collect_function_calls(&mut calls);
    Stmt::Break.collect_function_calls(&mut calls);
    Stmt::Continue.collect_function_calls(&mut calls);
    assert!(calls.is_empty());
}

// ============================================================================
// Expr::validate edge cases
// ============================================================================

#[test]
fn test_expr_literal_non_string_validates() {
    assert!(Expr::Literal(Literal::Bool(true)).validate().is_ok());
    assert!(Expr::Literal(Literal::U16(42)).validate().is_ok());
    assert!(Expr::Literal(Literal::U32(100)).validate().is_ok());
    assert!(Expr::Literal(Literal::I32(-10)).validate().is_ok());
}

#[test]
fn test_expr_function_call_validates_args() {
    let expr = Expr::FunctionCall {
        name: "foo".to_string(),
        args: vec![Expr::Literal(Literal::Str("ok\0bad".to_string()))],
    };
    assert!(expr.validate().is_err());
}

#[test]
fn test_expr_binary_validates_both_sides() {
    assert!(Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(Expr::Variable("".to_string())),
        right: Box::new(Expr::Literal(Literal::U32(1))),
    }
    .validate()
    .is_err());
    assert!(Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(Expr::Literal(Literal::U32(1))),
        right: Box::new(Expr::Variable("".to_string())),
    }
    .validate()
    .is_err());
}

#[test]
fn test_expr_unary_validates_operand() {
    assert!(Expr::Unary {
        op: UnaryOp::Not,
        operand: Box::new(Expr::Variable("".to_string())),
    }
    .validate()
    .is_err());
}

#[test]
fn test_expr_method_call_validates() {
    // Invalid receiver
    assert!(Expr::MethodCall {
        receiver: Box::new(Expr::Variable("".to_string())),
        method: "len".to_string(),
        args: vec![],
    }
    .validate()
    .is_err());
    // Invalid arg
    assert!(Expr::MethodCall {
        receiver: Box::new(Expr::Variable("obj".to_string())),
        method: "push".to_string(),
        args: vec![Expr::Literal(Literal::Str("null\0".to_string()))],
    }
    .validate()
    .is_err());
}

#[test]
fn test_expr_range_validates_both_ends() {
    assert!(Expr::Range {
        start: Box::new(Expr::Variable("".to_string())),
        end: Box::new(Expr::Literal(Literal::U32(10))),
        inclusive: false,
    }
    .validate()
    .is_err());
    assert!(Expr::Range {
        start: Box::new(Expr::Literal(Literal::U32(0))),
        end: Box::new(Expr::Variable("".to_string())),
        inclusive: true,
    }
    .validate()
    .is_err());
}

#[test]
fn test_expr_wildcard_arms_validate_ok() {
    assert!(Expr::Array(vec![]).validate().is_ok());
    assert!(Expr::Block(vec![]).validate().is_ok());
    assert!(Expr::PositionalArgs.validate().is_ok());
    assert!(Expr::Try {
        expr: Box::new(Expr::Literal(Literal::U32(0)))
    }
    .validate()
    .is_ok());
    assert!(Expr::Index {
        object: Box::new(Expr::Variable("arr".to_string())),
        index: Box::new(Expr::Literal(Literal::U32(0))),
    }
    .validate()
    .is_ok());
}

// ============================================================================
// Expr::nesting_depth
// ============================================================================

#[test]
fn test_nesting_depth_base_cases() {
    assert_eq!(Expr::Literal(Literal::U32(1)).nesting_depth(), 0);
    assert_eq!(Expr::Variable("x".to_string()).nesting_depth(), 0);
    assert_eq!(Expr::PositionalArgs.nesting_depth(), 0);
    assert_eq!(
        Expr::FunctionCall {
            name: "f".to_string(),
            args: vec![]
        }
        .nesting_depth(),
        1
    );
}

#[test]
fn test_nesting_depth_method_call() {
    let expr = Expr::MethodCall {
        receiver: Box::new(Expr::MethodCall {
            receiver: Box::new(Expr::Variable("x".to_string())),
            method: "trim".to_string(),
            args: vec![],
        }),
        method: "len".to_string(),
        args: vec![Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Literal(Literal::U32(1))),
            right: Box::new(Expr::Literal(Literal::U32(2))),
        }],
    };
    assert_eq!(expr.nesting_depth(), 2);
}

#[test]
fn test_nesting_depth_range() {
    let expr = Expr::Range {
        start: Box::new(Expr::Unary {
            op: UnaryOp::Neg,
            operand: Box::new(Expr::Literal(Literal::U32(1))),
        }),
        end: Box::new(Expr::Literal(Literal::U32(10))),
        inclusive: true,
    };
    assert_eq!(expr.nesting_depth(), 2);
}

// ============================================================================
// Expr::collect_function_calls
// ============================================================================

#[test]
fn test_expr_collect_calls_method_and_unary() {
    let mut calls = vec![];
    Expr::MethodCall {
        receiver: Box::new(Expr::FunctionCall {
            name: "get".to_string(),
            args: vec![],
        }),
        method: "do_thing".to_string(),
        args: vec![Expr::FunctionCall {
            name: "helper".to_string(),
            args: vec![],
        }],
    }
    .collect_function_calls(&mut calls);
    assert_eq!(calls, vec!["get", "helper"]);

    let mut calls = vec![];
    Expr::Unary {
        op: UnaryOp::Not,
        operand: Box::new(Expr::FunctionCall {
            name: "check".to_string(),
            args: vec![],
        }),
    }
    .collect_function_calls(&mut calls);
    assert_eq!(calls, vec!["check"]);
}

#[test]
fn test_expr_collect_calls_no_calls_from_atoms() {
    let mut calls = vec![];
    Expr::Variable("x".to_string()).collect_function_calls(&mut calls);
    Expr::Literal(Literal::U32(5)).collect_function_calls(&mut calls);
    Expr::PositionalArgs.collect_function_calls(&mut calls);
    assert!(calls.is_empty());
}

// ============================================================================
// Pattern edge cases
// ============================================================================

#[test]
fn test_pattern_validation_edge_cases() {
    assert!(Pattern::Variable("$bad".to_string()).validate().is_err());
    assert!(Pattern::Struct {
        name: "P".to_string(),
        fields: vec![(
            "x".to_string(),
            Pattern::Literal(Literal::Str("n\0".to_string()))
        )],
    }
    .validate()
    .is_err());
    assert!(
        Pattern::Tuple(vec![Pattern::Wildcard, Pattern::Variable("".to_string())])
            .validate()
            .is_err()
    );
    assert!(Pattern::Range {
        start: Literal::U32(0),
        end: Literal::U32(100),
        inclusive: true
    }
    .validate()
    .is_ok());
}

#[test]
fn test_pattern_binds_variable_range() {
    assert!(!Pattern::Range {
        start: Literal::U32(0),
        end: Literal::U32(10),
        inclusive: false
    }
    .binds_variable("x"));
}

// ============================================================================
// Type::is_allowed edge cases
// ============================================================================

#[test]
fn test_type_nested_is_allowed() {
    assert!(Type::U16.is_allowed());
    assert!(Type::Result {
        ok_type: Box::new(Type::Option {
            inner_type: Box::new(Type::U32)
        }),
        err_type: Box::new(Type::Str),
    }
    .is_allowed());
    assert!(Type::Option {
        inner_type: Box::new(Type::Result {
            ok_type: Box::new(Type::Bool),
            err_type: Box::new(Type::Str),
        }),
    }
    .is_allowed());
}

// ============================================================================
// Function validation edge cases
// ============================================================================

#[test]
fn test_function_body_and_param_validation() {
    assert!(Function {
        name: "test".to_string(),
        params: vec![],
        return_type: Type::Void,
        body: vec![Stmt::Let {
            name: "".to_string(),
            value: Expr::Literal(Literal::U32(0)),
            declaration: true
        }],
    }
    .validate()
    .is_err());
    assert!(Function {
        name: "test".to_string(),
        params: vec![Parameter {
            name: "$invalid".to_string(),
            param_type: Type::U32
        }],
        return_type: Type::Void,
        body: vec![],
    }
    .validate()
    .is_err());
}

// ============================================================================
// Literal PartialEq
// ============================================================================

#[test]
fn test_literal_equality() {
    assert_eq!(Literal::U16(100), Literal::U16(100));
    assert_ne!(Literal::U16(100), Literal::U16(200));
    assert_ne!(Literal::U32(42), Literal::I32(42));
    assert_ne!(Literal::Bool(true), Literal::U32(1));
}
