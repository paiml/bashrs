fn test_expr_collect_function_calls_comprehensive() {
    let mut calls = Vec::new();

    // Test Array expression
    let array_expr = Expr::Array(vec![
        Expr::FunctionCall {
            name: "array_func".to_string(),
            args: vec![],
        },
        Expr::Literal(restricted::Literal::U32(1)),
    ]);
    array_expr.collect_function_calls(&mut calls);
    assert!(calls.contains(&"array_func".to_string()));

    calls.clear();

    // Test Index expression
    let index_expr = Expr::Index {
        object: Box::new(Expr::FunctionCall {
            name: "get_array".to_string(),
            args: vec![],
        }),
        index: Box::new(Expr::FunctionCall {
            name: "get_index".to_string(),
            args: vec![],
        }),
    };
    index_expr.collect_function_calls(&mut calls);
    assert!(calls.contains(&"get_array".to_string()));
    assert!(calls.contains(&"get_index".to_string()));

    calls.clear();

    // Test Try expression
    let try_expr = Expr::Try {
        expr: Box::new(Expr::FunctionCall {
            name: "fallible_func".to_string(),
            args: vec![],
        }),
    };
    try_expr.collect_function_calls(&mut calls);
    assert!(calls.contains(&"fallible_func".to_string()));

    calls.clear();

    // Test Block expression
    let block_expr = Expr::Block(vec![Stmt::Expr(Expr::FunctionCall {
        name: "block_func".to_string(),
        args: vec![],
    })]);
    block_expr.collect_function_calls(&mut calls);
    assert!(calls.contains(&"block_func".to_string()));
}

#[test]
fn test_stmt_match_validation_and_collection() {
    let match_stmt = Stmt::Match {
        scrutinee: Expr::Variable("x".to_string()),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(restricted::Literal::U32(1)),
                guard: Some(Expr::FunctionCall {
                    name: "guard_func".to_string(),
                    args: vec![],
                }),
                body: vec![Stmt::Expr(Expr::FunctionCall {
                    name: "arm_func".to_string(),
                    args: vec![],
                })],
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: vec![Stmt::Return(None)],
            },
        ],
    };

    // Test validation
    assert!(match_stmt.validate().is_ok());

    // Test function call collection
    let mut calls = Vec::new();
    match_stmt.collect_function_calls(&mut calls);
    assert!(calls.contains(&"guard_func".to_string()));
    assert!(calls.contains(&"arm_func".to_string()));
}

#[test]
fn test_stmt_for_while_bounded_validation() {
    // Test For loop with bounded iterations (should pass)
    let for_stmt = Stmt::For {
        pattern: Pattern::Variable("i".to_string()),
        iter: Expr::Variable("collection".to_string()),
        body: vec![Stmt::Return(None)],
        max_iterations: Some(100),
    };
    assert!(for_stmt.validate().is_ok());

    // Test For loop without bounded iterations (should fail)
    let unbounded_for = Stmt::For {
        pattern: Pattern::Variable("i".to_string()),
        iter: Expr::Variable("collection".to_string()),
        body: vec![],
        max_iterations: None,
    };
    assert!(unbounded_for.validate().is_err());

    // Test While loop with bounded iterations (should pass)
    let while_stmt = Stmt::While {
        condition: Expr::Literal(restricted::Literal::Bool(true)),
        body: vec![Stmt::Break],
        max_iterations: Some(50),
    };
    assert!(while_stmt.validate().is_ok());

    // Test While loop without bounded iterations (should fail)
    let unbounded_while = Stmt::While {
        condition: Expr::Literal(restricted::Literal::Bool(true)),
        body: vec![],
        max_iterations: None,
    };
    assert!(unbounded_while.validate().is_err());
}

#[test]
fn test_stmt_for_while_function_call_collection() {
    let mut calls = Vec::new();

    // Test For loop
    let for_stmt = Stmt::For {
        pattern: Pattern::Variable("i".to_string()),
        iter: Expr::FunctionCall {
            name: "get_iter".to_string(),
            args: vec![],
        },
        body: vec![Stmt::Expr(Expr::FunctionCall {
            name: "process".to_string(),
            args: vec![],
        })],
        max_iterations: Some(100),
    };
    for_stmt.collect_function_calls(&mut calls);
    assert!(calls.contains(&"get_iter".to_string()));
    assert!(calls.contains(&"process".to_string()));

    calls.clear();

    // Test While loop
    let while_stmt = Stmt::While {
        condition: Expr::FunctionCall {
            name: "check_condition".to_string(),
            args: vec![],
        },
        body: vec![Stmt::Expr(Expr::FunctionCall {
            name: "loop_body".to_string(),
            args: vec![],
        })],
        max_iterations: Some(50),
    };
    while_stmt.collect_function_calls(&mut calls);
    assert!(calls.contains(&"check_condition".to_string()));
    assert!(calls.contains(&"loop_body".to_string()));
}

#[test]
fn test_stmt_break_continue() {
    // Test Break and Continue validation
    assert!(Stmt::Break.validate().is_ok());
    assert!(Stmt::Continue.validate().is_ok());

    // Test Break and Continue function call collection (should be empty)
    let mut calls = Vec::new();
    Stmt::Break.collect_function_calls(&mut calls);
    assert!(calls.is_empty());

    Stmt::Continue.collect_function_calls(&mut calls);
    assert!(calls.is_empty());
}

#[test]
fn test_validate_public_api() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "x".to_string(),
                value: Expr::Literal(restricted::Literal::U32(42)),
                declaration: true,
            }],
        }],
        entry_point: "main".to_string(),
    };

    // Test the public validate function
    assert!(validate(&ast).is_ok());
}

#[test]
fn test_invalid_ast_returns_validation_error() {
    let ast = RestrictedAst {
        functions: vec![],
        entry_point: "main".to_string(),
    };

    match validate(&ast) {
        Err(crate::models::Error::Validation(_)) => (), // Expected
        _ => panic!("Expected validation error"),

    }
}
