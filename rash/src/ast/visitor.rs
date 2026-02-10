use super::restricted::{Expr, Function, RestrictedAst, Stmt};

pub trait Visitor<T> {
    fn visit_ast(&mut self, ast: &RestrictedAst) -> T;
    fn visit_function(&mut self, function: &Function) -> T;
    fn visit_stmt(&mut self, stmt: &Stmt) -> T;
    fn visit_expr(&mut self, expr: &Expr) -> T;
}

pub trait VisitorMut<T> {
    fn visit_ast_mut(&mut self, ast: &mut RestrictedAst) -> T;
    fn visit_function_mut(&mut self, function: &mut Function) -> T;
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) -> T;
    fn visit_expr_mut(&mut self, expr: &mut Expr) -> T;
}

/// Walk an AST and call the visitor for each node
pub fn walk_ast<V, T>(visitor: &mut V, ast: &RestrictedAst) -> T
where
    V: Visitor<T>,
    T: Default,
{
    visitor.visit_ast(ast)
}

/// Transform an AST by calling a function on each expression
pub fn transform_exprs<F>(ast: &mut RestrictedAst, mut transform: F)
where
    F: FnMut(&mut Expr),
{
    for function in &mut ast.functions {
        for stmt in &mut function.body {
            transform_stmt_exprs(stmt, &mut transform);
        }
    }
}

fn transform_stmt_exprs<F>(stmt: &mut Stmt, transform: &mut F)
where
    F: FnMut(&mut Expr),
{
    match stmt {
        Stmt::Let { value, .. } => transform_expr(value, transform),
        Stmt::Expr(expr) => transform_expr(expr, transform),
        Stmt::Return(Some(expr)) => transform_expr(expr, transform),
        Stmt::Return(None) => {}
        Stmt::If {
            condition,
            then_block,
            else_block,
        } => {
            transform_expr(condition, transform);
            for stmt in then_block {
                transform_stmt_exprs(stmt, transform);
            }
            if let Some(else_stmts) = else_block {
                for stmt in else_stmts {
                    transform_stmt_exprs(stmt, transform);
                }
            }
        }
        // Placeholder for new AST nodes - TODO: implement properly
        _ => {} // Match, For, While, Break, Continue
    }
}

fn transform_expr<F>(expr: &mut Expr, transform: &mut F)
where
    F: FnMut(&mut Expr),
{
    match expr {
        Expr::FunctionCall { args, .. } => {
            for arg in args {
                transform_expr(arg, transform);
            }
        }
        Expr::Binary { left, right, .. } => {
            transform_expr(left, transform);
            transform_expr(right, transform);
        }
        Expr::Unary { operand, .. } => {
            transform_expr(operand, transform);
        }
        Expr::MethodCall { receiver, args, .. } => {
            transform_expr(receiver, transform);
            for arg in args {
                transform_expr(arg, transform);
            }
        }
        _ => {}
    }

    transform(expr);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::restricted::{BinaryOp, Literal, Type, UnaryOp};

    // Helper to create a simple AST for testing
    fn create_test_ast() -> RestrictedAst {
        RestrictedAst {
            entry_point: "test_fn".to_string(),
            functions: vec![Function {
                name: "test_fn".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![
                    Stmt::Let {
                        name: "x".to_string(),
                        value: Expr::Literal(Literal::Str("hello".to_string())),
                        declaration: true,
                    },
                    Stmt::Expr(Expr::Variable("x".to_string())),
                ],
            }],
        }
    }

    // Simple visitor implementation for testing
    struct CountingVisitor {
        count: usize,
    }

    impl Visitor<()> for CountingVisitor {
        fn visit_ast(&mut self, ast: &RestrictedAst) -> () {
            self.count += 1;
            for func in &ast.functions {
                self.visit_function(func);
            }
        }

        fn visit_function(&mut self, function: &Function) -> () {
            self.count += 1;
            for stmt in &function.body {
                self.visit_stmt(stmt);
            }
        }

        fn visit_stmt(&mut self, stmt: &Stmt) -> () {
            self.count += 1;
            match stmt {
                Stmt::Let { value, .. } => self.visit_expr(value),
                Stmt::Expr(expr) => self.visit_expr(expr),
                Stmt::Return(Some(expr)) => self.visit_expr(expr),
                _ => (),
            }
        }

        fn visit_expr(&mut self, _expr: &Expr) -> () {
            self.count += 1;
        }
    }

    #[test]
    fn test_walk_ast() {
        let ast = create_test_ast();
        let mut visitor = CountingVisitor { count: 0 };
        walk_ast(&mut visitor, &ast);
        // 1 AST + 1 function + 2 stmts + 2 exprs = 6
        assert_eq!(visitor.count, 6);
    }

    #[test]
    fn test_transform_exprs_let() {
        let mut ast = create_test_ast();
        let mut transform_count = 0;

        transform_exprs(&mut ast, |_expr| {
            transform_count += 1;
        });

        // Should transform exprs in Let and Expr statements
        assert_eq!(transform_count, 2);
    }

    #[test]
    fn test_transform_exprs_empty_ast() {
        let mut ast = RestrictedAst {
            entry_point: "main".to_string(),
            functions: vec![],
        };
        let mut transform_count = 0;

        transform_exprs(&mut ast, |_expr| {
            transform_count += 1;
        });

        assert_eq!(transform_count, 0);
    }

    #[test]
    fn test_transform_exprs_with_if() {
        let mut ast = RestrictedAst {
            entry_point: "test".to_string(),
            functions: vec![Function {
                name: "test".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::If {
                    condition: Expr::Variable("cond".to_string()),
                    then_block: vec![Stmt::Expr(Expr::Literal(Literal::Str("then".to_string())))],
                    else_block: Some(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                        "else".to_string(),
                    )))]),
                }],
            }],
        };

        let mut transform_count = 0;
        transform_exprs(&mut ast, |_expr| {
            transform_count += 1;
        });

        // condition + then expr + else expr = 3
        assert_eq!(transform_count, 3);
    }

    #[test]
    fn test_transform_exprs_with_return() {
        let mut ast = RestrictedAst {
            entry_point: "test".to_string(),
            functions: vec![Function {
                name: "test".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![
                    Stmt::Return(Some(Expr::Literal(Literal::Str("value".to_string())))),
                    Stmt::Return(None),
                ],
            }],
        };

        let mut transform_count = 0;
        transform_exprs(&mut ast, |_expr| {
            transform_count += 1;
        });

        // Only the Return(Some(...)) should be transformed
        assert_eq!(transform_count, 1);
    }

    #[test]
    fn test_transform_expr_function_call() {
        let mut ast = RestrictedAst {
            entry_point: "test".to_string(),
            functions: vec![Function {
                name: "test".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Expr(Expr::FunctionCall {
                    name: "func".to_string(),
                    args: vec![
                        Expr::Literal(Literal::Str("arg1".to_string())),
                        Expr::Literal(Literal::Str("arg2".to_string())),
                    ],
                })],
            }],
        };

        let mut transform_count = 0;
        transform_exprs(&mut ast, |_expr| {
            transform_count += 1;
        });

        // 2 args + 1 function call = 3
        assert_eq!(transform_count, 3);
    }

    #[test]
    fn test_transform_expr_binary() {
        let mut ast = RestrictedAst {
            entry_point: "test".to_string(),
            functions: vec![Function {
                name: "test".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Expr(Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Literal(Literal::U32(1))),
                    right: Box::new(Expr::Literal(Literal::U32(2))),
                })],
            }],
        };

        let mut transform_count = 0;
        transform_exprs(&mut ast, |_expr| {
            transform_count += 1;
        });

        // left + right + binary = 3
        assert_eq!(transform_count, 3);
    }

    #[test]
    fn test_transform_expr_unary() {
        let mut ast = RestrictedAst {
            entry_point: "test".to_string(),
            functions: vec![Function {
                name: "test".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Expr(Expr::Unary {
                    op: UnaryOp::Neg,
                    operand: Box::new(Expr::Literal(Literal::U32(5))),
                })],
            }],
        };

        let mut transform_count = 0;
        transform_exprs(&mut ast, |_expr| {
            transform_count += 1;
        });

        // operand + unary = 2
        assert_eq!(transform_count, 2);
    }

    #[test]
    fn test_transform_expr_method_call() {
        let mut ast = RestrictedAst {
            entry_point: "test".to_string(),
            functions: vec![Function {
                name: "test".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Expr(Expr::MethodCall {
                    receiver: Box::new(Expr::Variable("obj".to_string())),
                    method: "method".to_string(),
                    args: vec![Expr::Literal(Literal::Str("arg".to_string()))],
                })],
            }],
        };

        let mut transform_count = 0;
        transform_exprs(&mut ast, |_expr| {
            transform_count += 1;
        });

        // receiver + arg + method call = 3
        assert_eq!(transform_count, 3);
    }
}
