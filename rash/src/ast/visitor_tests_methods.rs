fn test_visitor_traits_exist() {
    // Test that the traits are properly defined
    fn test_visitor<V: Visitor<String>>(_v: V) {}
    fn test_visitor_mut<V: VisitorMut<String>>(_v: V) {}

    // These should compile if the traits are properly defined
    struct TestVisitor;
    impl Visitor<String> for TestVisitor {
        fn visit_ast(&mut self, _ast: &RestrictedAst) -> String {
            String::new()
        }
        fn visit_function(&mut self, _function: &Function) -> String {
            String::new()
        }
        fn visit_stmt(&mut self, _stmt: &Stmt) -> String {
            String::new()
        }
        fn visit_expr(&mut self, _expr: &Expr) -> String {
            String::new()
        }
    }

    struct TestVisitorMut;
    impl VisitorMut<String> for TestVisitorMut {
        fn visit_ast_mut(&mut self, _ast: &mut RestrictedAst) -> String {
            String::new()
        }
        fn visit_function_mut(&mut self, _function: &mut Function) -> String {
            String::new()
        }
        fn visit_stmt_mut(&mut self, _stmt: &mut Stmt) -> String {
            String::new()
        }
        fn visit_expr_mut(&mut self, _expr: &mut Expr) -> String {
            String::new()
        }
    }

    test_visitor(TestVisitor);
    test_visitor_mut(TestVisitorMut);
}

// ============================================================================
// Additional tests for statement types (Match, For, While, Break, Continue)
// ============================================================================

#[test]
fn test_transform_exprs_match_stmt() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Match {
                scrutinee: Expr::Variable("x".to_string()),
                arms: vec![
                    MatchArm {
                        pattern: Pattern::Literal(Literal::U32(1)),
                        guard: None,
                        body: vec![Stmt::Expr(Expr::Literal(Literal::Str("one".to_string())))],
                    },
                    MatchArm {
                        pattern: Pattern::Wildcard,
                        guard: Some(Expr::Literal(Literal::Bool(true))),
                        body: vec![Stmt::Expr(Expr::Literal(Literal::Str("other".to_string())))],
                    },
                ],
            }],
        }],
        entry_point: "main".to_string(),
    };

    let mut transform_count = 0;
    transform_exprs(&mut ast, |_expr| {
        transform_count += 1;
    });

    // Match statement is not yet fully supported in transform_exprs (placeholder)
    // So it won't transform the expressions inside - this tests the placeholder path
    assert_eq!(transform_count, 0);
}

#[test]
fn test_transform_exprs_for_loop() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::For {
                pattern: Pattern::Variable("i".to_string()),
                iter: Expr::Range {
                    start: Box::new(Expr::Literal(Literal::U32(0))),
                    end: Box::new(Expr::Literal(Literal::U32(10))),
                    inclusive: false,
                },
                body: vec![Stmt::Expr(Expr::Variable("i".to_string()))],
                max_iterations: None,
            }],
        }],
        entry_point: "main".to_string(),
    };

    let mut transform_count = 0;
    transform_exprs(&mut ast, |_expr| {
        transform_count += 1;
    });

    // For loop is a placeholder in transform_exprs
    assert_eq!(transform_count, 0);
}

#[test]
fn test_transform_exprs_while_loop() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::While {
                condition: Expr::Literal(Literal::Bool(true)),
                body: vec![Stmt::Break],
                max_iterations: None,
            }],
        }],
        entry_point: "main".to_string(),
    };

    let mut transform_count = 0;
    transform_exprs(&mut ast, |_expr| {
        transform_count += 1;
    });

    // While loop is a placeholder in transform_exprs
    assert_eq!(transform_count, 0);
}

#[test]
fn test_transform_exprs_break_continue() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Break, Stmt::Continue],
        }],
        entry_point: "main".to_string(),
    };

    let mut transform_count = 0;
    transform_exprs(&mut ast, |_expr| {
        transform_count += 1;
    });

    // Break and Continue have no expressions to transform
    assert_eq!(transform_count, 0);
}

// ============================================================================
// Additional expression type tests
// ============================================================================

#[test]
fn test_transform_exprs_variable() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Expr(Expr::Variable("x".to_string()))],
        }],
        entry_point: "main".to_string(),
    };

    let mut transform_count = 0;
    transform_exprs(&mut ast, |_expr| {
        transform_count += 1;
    });

    // Variable expression
    assert_eq!(transform_count, 1);
}

#[test]
fn test_transform_exprs_literal_standalone() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Expr(Expr::Literal(Literal::Str("hello".to_string())))],
        }],
        entry_point: "main".to_string(),
    };

    let mut transform_count = 0;
    transform_exprs(&mut ast, |_expr| {
        transform_count += 1;
    });

    // Literal expression
    assert_eq!(transform_count, 1);
}

#[test]
fn test_transform_exprs_range() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "r".to_string(),
                value: Expr::Range {
                    start: Box::new(Expr::Literal(Literal::U32(1))),
                    end: Box::new(Expr::Literal(Literal::U32(10))),
                    inclusive: true,
                },
                declaration: true,
            }],
        }],
        entry_point: "main".to_string(),
    };

    let mut transform_count = 0;
    transform_exprs(&mut ast, |_expr| {
        transform_count += 1;
    });

    // Range is in the "other" category (doesn't recurse)
    assert_eq!(transform_count, 1);
}

#[test]
fn test_transform_exprs_if_without_else() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::If {
                condition: Expr::Literal(Literal::Bool(true)),
                then_block: vec![Stmt::Expr(Expr::Literal(Literal::U32(1)))],
                else_block: None,
            }],
        }],
        entry_point: "main".to_string(),
    };

    let mut transform_count = 0;
    transform_exprs(&mut ast, |_expr| {
        transform_count += 1;
    });

    // Should transform: condition + then expr = 2 total
    assert_eq!(transform_count, 2);
}

#[test]
fn test_transform_exprs_multiple_functions() {
    let mut ast = RestrictedAst {
        functions: vec![
            Function {
                name: "func1".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Expr(Expr::Literal(Literal::U32(1)))],
            },
            Function {
                name: "func2".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Expr(Expr::Literal(Literal::U32(2)))],
            },
        ],
        entry_point: "func1".to_string(),
    };

    let mut transform_count = 0;
    transform_exprs(&mut ast, |_expr| {
        transform_count += 1;
    });

    // Should transform expressions in both functions
    assert_eq!(transform_count, 2);
}

// ============================================================================
// Additional visitor tests
// ============================================================================

#[test]
fn test_visitor_mut_implementation() {
    struct ReplaceVisitor;
    impl VisitorMut<usize> for ReplaceVisitor {
        fn visit_ast_mut(&mut self, ast: &mut RestrictedAst) -> usize {
            ast.functions.len()
        }
        fn visit_function_mut(&mut self, _function: &mut Function) -> usize {
            1
        }
        fn visit_stmt_mut(&mut self, _stmt: &mut Stmt) -> usize {
            1
        }
        fn visit_expr_mut(&mut self, _expr: &mut Expr) -> usize {
            1
        }
    }

    let mut visitor = ReplaceVisitor;
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "test".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![],
        }],
        entry_point: "test".to_string(),
    };

    let result = visitor.visit_ast_mut(&mut ast);
    assert_eq!(result, 1);
}

#[test]
fn test_transform_exprs_actual_modification() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "x".to_string(),
                value: Expr::Literal(Literal::U32(0)),
                declaration: true,
            }],
        }],
        entry_point: "main".to_string(),
    };

    // Transform all literals to replace 0 with 42
    transform_exprs(&mut ast, |expr| {
        if let Expr::Literal(Literal::U32(0)) = expr {
            *expr = Expr::Literal(Literal::U32(42));
        }
    });

    // Verify the transformation occurred
    if let Stmt::Let { value, .. } = &ast.functions[0].body[0] {
        if let Expr::Literal(Literal::U32(n)) = value {
            assert_eq!(*n, 42);
        } else {
            panic!("Expected U32 literal");
        }
    } else {
        panic!("Expected Let statement");
    }
}

#[test]

include!("visitor_tests_tests_transform_ex.rs");
