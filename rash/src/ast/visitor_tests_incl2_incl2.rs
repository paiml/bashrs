fn test_transform_exprs_deep_nested_modification() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::If {
                condition: Expr::Binary {
                    op: BinaryOp::Lt,
                    left: Box::new(Expr::Variable("x".to_string())),
                    right: Box::new(Expr::Literal(Literal::U32(10))),
                },
                then_block: vec![Stmt::Let {
                    name: "y".to_string(),
                    value: Expr::Unary {
                        op: UnaryOp::Neg,
                        operand: Box::new(Expr::Literal(Literal::I32(5))),
                    },
                    declaration: true,
                }],
                else_block: Some(vec![Stmt::Return(Some(Expr::Literal(Literal::U32(0))))]),
            }],
        }],
        entry_point: "main".to_string(),
    };

    let mut modified_count = 0;
    transform_exprs(&mut ast, |expr| {
        // Count variables
        if matches!(expr, Expr::Variable(_)) {
            modified_count += 1;
        }
    });

    assert_eq!(modified_count, 1); // Only "x" variable
}

#[test]
fn test_walk_ast_with_return_value() {
    struct CountFunctionsVisitor {
        count: usize,
    }

    impl Visitor<usize> for CountFunctionsVisitor {
        fn visit_ast(&mut self, ast: &RestrictedAst) -> usize {
            self.count = ast.functions.len();
            self.count
        }
        fn visit_function(&mut self, _function: &Function) -> usize {
            0
        }
        fn visit_stmt(&mut self, _stmt: &Stmt) -> usize {
            0
        }
        fn visit_expr(&mut self, _expr: &Expr) -> usize {
            0
        }
    }

    let mut visitor = CountFunctionsVisitor { count: 0 };
    let ast = RestrictedAst {
        functions: vec![
            Function {
                name: "f1".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![],
            },
            Function {
                name: "f2".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![],
            },
            Function {
                name: "f3".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![],
            },
        ],
        entry_point: "f1".to_string(),
    };

    let result = walk_ast(&mut visitor, &ast);
    assert_eq!(result, 3);
}

#[test]
fn test_transform_exprs_index_expression() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Expr(Expr::Index {
                object: Box::new(Expr::Variable("arr".to_string())),
                index: Box::new(Expr::Literal(Literal::U32(0))),
            })],
        }],
        entry_point: "main".to_string(),
    };

    let mut transform_count = 0;
    transform_exprs(&mut ast, |_expr| {
        transform_count += 1;
    });

    // Index expression doesn't recurse in transform_expr (it's in the _ => {} branch)
    assert_eq!(transform_count, 1);
}

#[test]
fn test_transform_exprs_array_expression() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "arr".to_string(),
                value: Expr::Array(vec![
                    Expr::Literal(Literal::U32(1)),
                    Expr::Literal(Literal::U32(2)),
                ]),
                declaration: true,
            }],
        }],
        entry_point: "main".to_string(),
    };

    let mut transform_count = 0;
    transform_exprs(&mut ast, |_expr| {
        transform_count += 1;
    });

    // Array doesn't recurse (it's in the _ => {} branch)
    assert_eq!(transform_count, 1);
}

#[test]
fn test_transform_exprs_try_expression() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "result".to_string(),
                value: Expr::Try {
                    expr: Box::new(Expr::FunctionCall {
                        name: "fallible".to_string(),
                        args: vec![],
                    }),
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

    // Try doesn't recurse (it's in the _ => {} branch)
    assert_eq!(transform_count, 1);
}

#[test]
fn test_transform_exprs_block_expression() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "x".to_string(),
                value: Expr::Block(vec![Stmt::Return(Some(Expr::Literal(Literal::U32(42))))]),
                declaration: true,
            }],
        }],
        entry_point: "main".to_string(),
    };

    let mut transform_count = 0;
    transform_exprs(&mut ast, |_expr| {
        transform_count += 1;
    });

    // Block doesn't recurse (it's in the _ => {} branch)
    assert_eq!(transform_count, 1);
}

#[test]
fn test_transform_exprs_complex_nested_if() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::If {
                condition: Expr::Variable("a".to_string()),
                then_block: vec![Stmt::If {
                    condition: Expr::Variable("b".to_string()),
                    then_block: vec![Stmt::Expr(Expr::Literal(Literal::U32(1)))],
                    else_block: None,
                }],
                else_block: Some(vec![Stmt::If {
                    condition: Expr::Variable("c".to_string()),
                    then_block: vec![Stmt::Expr(Expr::Literal(Literal::U32(2)))],
                    else_block: Some(vec![Stmt::Expr(Expr::Literal(Literal::U32(3)))]),
                }]),
            }],
        }],
        entry_point: "main".to_string(),
    };

    let mut var_count = 0;
    transform_exprs(&mut ast, |expr| {
        if matches!(expr, Expr::Variable(_)) {
            var_count += 1;
        }
    });

    // Should count a, b, c = 3 variables
    assert_eq!(var_count, 3);
}

#[test]
fn test_expr_type_visitor_all_types() {
    let mut visitor = ExprTypeVisitor::new();

    // Test all expression types
    visitor.visit_expr(&Expr::Literal(Literal::U32(42)));
    visitor.visit_expr(&Expr::Variable("x".to_string()));
    visitor.visit_expr(&Expr::FunctionCall {
        name: "f".to_string(),
        args: vec![],
    });
    visitor.visit_expr(&Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(Expr::Literal(Literal::U32(1))),
        right: Box::new(Expr::Literal(Literal::U32(2))),
    });
    visitor.visit_expr(&Expr::Unary {
        op: UnaryOp::Not,
        operand: Box::new(Expr::Literal(Literal::Bool(true))),
    });
    visitor.visit_expr(&Expr::MethodCall {
        receiver: Box::new(Expr::Variable("obj".to_string())),
        method: "m".to_string(),
        args: vec![],
    });
    visitor.visit_expr(&Expr::Index {
        object: Box::new(Expr::Variable("arr".to_string())),
        index: Box::new(Expr::Literal(Literal::U32(0))),
    });
    visitor.visit_expr(&Expr::Array(vec![]));
    visitor.visit_expr(&Expr::Try {
        expr: Box::new(Expr::Variable("x".to_string())),
    });
    visitor.visit_expr(&Expr::Block(vec![]));
    visitor.visit_expr(&Expr::Range {
        start: Box::new(Expr::Literal(Literal::U32(0))),
        end: Box::new(Expr::Literal(Literal::U32(10))),
        inclusive: false,
    });

    assert_eq!(
        visitor.types,
        vec![
            "Literal",
            "Variable",
            "FunctionCall",
            "Binary",
            "Unary",
            "MethodCall",
            "Index",
            "Array",
            "Try",
            "Block",
            "Range",
        ]
    );
}
