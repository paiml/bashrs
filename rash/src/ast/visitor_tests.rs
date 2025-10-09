//! Comprehensive tests for the visitor module

use super::restricted::*;
use super::visitor::*;

/// Test visitor that counts nodes
struct CountingVisitor {
    ast_count: usize,
    function_count: usize,
    stmt_count: usize,
    expr_count: usize,
}

impl CountingVisitor {
    fn new() -> Self {
        Self {
            ast_count: 0,
            function_count: 0,
            stmt_count: 0,
            expr_count: 0,
        }
    }
}

impl Visitor<()> for CountingVisitor {
    fn visit_ast(&mut self, _ast: &RestrictedAst) {
        self.ast_count += 1;
    }

    fn visit_function(&mut self, _function: &Function) {
        self.function_count += 1;
    }

    fn visit_stmt(&mut self, _stmt: &Stmt) {
        self.stmt_count += 1;
    }

    fn visit_expr(&mut self, _expr: &Expr) {
        self.expr_count += 1;
    }
}

/// Test visitor that collects expression types
struct ExprTypeVisitor {
    types: Vec<String>,
}

impl ExprTypeVisitor {
    fn new() -> Self {
        Self { types: Vec::new() }
    }
}

impl Visitor<()> for ExprTypeVisitor {
    fn visit_ast(&mut self, _ast: &RestrictedAst) {}
    fn visit_function(&mut self, _function: &Function) {}
    fn visit_stmt(&mut self, _stmt: &Stmt) {}

    fn visit_expr(&mut self, expr: &Expr) {
        let type_name = match expr {
            Expr::Literal(_) => "Literal",
            Expr::Variable(_) => "Variable",
            Expr::FunctionCall { .. } => "FunctionCall",
            Expr::Binary { .. } => "Binary",
            Expr::Unary { .. } => "Unary",
            Expr::MethodCall { .. } => "MethodCall",
            Expr::Index { .. } => "Index",
            Expr::Array(_) => "Array",
            Expr::Try { .. } => "Try",
            Expr::Block(_) => "Block",
            Expr::Range { .. } => "Range",
        };
        self.types.push(type_name.to_string());
    }
}

/// Test mutable visitor that transforms expressions
#[allow(dead_code)]
struct ExprTransformVisitor;

impl VisitorMut<()> for ExprTransformVisitor {
    fn visit_ast_mut(&mut self, _ast: &mut RestrictedAst) {}
    fn visit_function_mut(&mut self, _function: &mut Function) {}
    fn visit_stmt_mut(&mut self, _stmt: &mut Stmt) {}
    fn visit_expr_mut(&mut self, _expr: &mut Expr) {}
}

#[test]
fn test_counting_visitor() {
    let mut visitor = CountingVisitor::new();

    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![
                Stmt::Let {
                    name: "x".to_string(),
                    value: Expr::Literal(Literal::U32(42)),
                },
                Stmt::Return(Some(Expr::Variable("x".to_string()))),
            ],
        }],
        entry_point: "main".to_string(),
    };

    walk_ast(&mut visitor, &ast);
    assert_eq!(visitor.ast_count, 1);
}

#[test]
fn test_expr_type_visitor() {
    let mut visitor = ExprTypeVisitor::new();

    let expr = Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(Expr::Literal(Literal::U32(1))),
        right: Box::new(Expr::Literal(Literal::U32(2))),
    };

    visitor.visit_expr(&expr);
    assert_eq!(visitor.types, vec!["Binary"]);
}

#[test]
fn test_transform_exprs_literal() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "x".to_string(),
                value: Expr::Literal(Literal::U32(42)),
            }],
        }],
        entry_point: "main".to_string(),
    };

    let mut transform_count = 0;
    transform_exprs(&mut ast, |_expr| {
        transform_count += 1;
    });

    assert_eq!(transform_count, 1);
}

#[test]
fn test_transform_exprs_function_call() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "print".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("hello".to_string())),
                    Expr::Literal(Literal::U32(42)),
                ],
            })],
        }],
        entry_point: "main".to_string(),
    };

    let mut transform_count = 0;
    transform_exprs(&mut ast, |_expr| {
        transform_count += 1;
    });

    // Should transform: the function call itself + 2 arguments = 3 total
    assert_eq!(transform_count, 3);
}

#[test]
fn test_transform_exprs_binary() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "x".to_string(),
                value: Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Literal(Literal::U32(1))),
                    right: Box::new(Expr::Literal(Literal::U32(2))),
                },
            }],
        }],
        entry_point: "main".to_string(),
    };

    let mut transform_count = 0;
    transform_exprs(&mut ast, |_expr| {
        transform_count += 1;
    });

    // Should transform: left literal + right literal + binary expr = 3 total
    assert_eq!(transform_count, 3);
}

#[test]
fn test_transform_exprs_unary() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "x".to_string(),
                value: Expr::Unary {
                    op: UnaryOp::Not,
                    operand: Box::new(Expr::Literal(Literal::Bool(true))),
                },
            }],
        }],
        entry_point: "main".to_string(),
    };

    let mut transform_count = 0;
    transform_exprs(&mut ast, |_expr| {
        transform_count += 1;
    });

    // Should transform: operand + unary expr = 2 total
    assert_eq!(transform_count, 2);
}

#[test]
fn test_transform_exprs_method_call() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Expr(Expr::MethodCall {
                receiver: Box::new(Expr::Variable("obj".to_string())),
                method: "method".to_string(),
                args: vec![Expr::Literal(Literal::U32(1))],
            })],
        }],
        entry_point: "main".to_string(),
    };

    let mut transform_count = 0;
    transform_exprs(&mut ast, |_expr| {
        transform_count += 1;
    });

    // Should transform: receiver + arg + method call = 3 total
    assert_eq!(transform_count, 3);
}

#[test]
fn test_transform_exprs_return_stmt() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![
                Stmt::Return(Some(Expr::Literal(Literal::U32(42)))),
                Stmt::Return(None),
            ],
        }],
        entry_point: "main".to_string(),
    };

    let mut transform_count = 0;
    transform_exprs(&mut ast, |_expr| {
        transform_count += 1;
    });

    // Should transform only the expression in the first return: 1 total
    assert_eq!(transform_count, 1);
}

#[test]
fn test_transform_exprs_if_stmt() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::If {
                condition: Expr::Literal(Literal::Bool(true)),
                then_block: vec![Stmt::Let {
                    name: "x".to_string(),
                    value: Expr::Literal(Literal::U32(1)),
                }],
                else_block: Some(vec![Stmt::Let {
                    name: "y".to_string(),
                    value: Expr::Literal(Literal::U32(2)),
                }]),
            }],
        }],
        entry_point: "main".to_string(),
    };

    let mut transform_count = 0;
    transform_exprs(&mut ast, |_expr| {
        transform_count += 1;
    });

    // Should transform: condition + then expr + else expr = 3 total
    assert_eq!(transform_count, 3);
}

#[test]
fn test_transform_exprs_empty_function() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "empty".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![],
        }],
        entry_point: "empty".to_string(),
    };

    let mut transform_count = 0;
    transform_exprs(&mut ast, |_expr| {
        transform_count += 1;
    });

    assert_eq!(transform_count, 0);
}

#[test]
fn test_transform_exprs_nested_expressions() {
    let mut ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "result".to_string(),
                value: Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::FunctionCall {
                        name: "func".to_string(),
                        args: vec![Expr::Literal(Literal::U32(1))],
                    }),
                    right: Box::new(Expr::Unary {
                        op: UnaryOp::Neg,
                        operand: Box::new(Expr::Literal(Literal::U32(2))),
                    }),
                },
            }],
        }],
        entry_point: "main".to_string(),
    };

    let mut transform_count = 0;
    transform_exprs(&mut ast, |_expr| {
        transform_count += 1;
    });

    // Should transform:
    // - function call arg (1)
    // - function call itself (func)
    // - unary operand (2)
    // - unary expr (-)
    // - binary expr (+)
    // Total: 5
    assert_eq!(transform_count, 5);
}

#[test]
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
