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
