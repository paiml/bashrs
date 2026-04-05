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

include!("restricted_tests_tests_expr.rs");
