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
