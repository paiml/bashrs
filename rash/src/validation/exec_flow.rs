//! GH-294 guard. A string literal is emitted single-quoted, where `$( )` and a
//! backtick are inert, so the pre-emission check no longer refuses them on
//! content alone. `exec()` and `capture()` emit `eval`, though, and a value that
//! reaches `eval` through a variable, a parameter or a function's return is
//! parsed again by the shell, where the same bytes run. A literal's own quoting
//! cannot protect it there. So when any `exec`/`capture` argument is dynamic —
//! anything but string literals — every string literal in the program is held
//! to the pre-#294 rule.

use crate::ast::restricted::{Literal, MatchArm, RestrictedAst, Stmt};
use crate::ast::Expr;
use crate::models::error::RashResult;

const EXEC_FNS: [&str; 2] = ["exec", "capture"];

/// Refuse `$( )` and backticks in every string literal when the program
/// executes a string it builds at run time.
pub(crate) fn check(ast: &RestrictedAst) -> RashResult<()> {
    let mut dynamic = false;
    let mut literals: Vec<&str> = Vec::new();
    for function in &ast.functions {
        walk_stmts(&function.body, &mut |expr| {
            if is_dynamic_exec(expr) {
                dynamic = true;
            }
            if let Expr::Literal(Literal::Str(s)) = expr {
                literals.push(s.as_str());
            }
        });
    }
    if !dynamic {
        return Ok(());
    }
    for s in literals {
        super::pipeline::ValidationPipeline::check_substitution_patterns(s)?;
    }
    Ok(())
}

fn is_dynamic_exec(expr: &Expr) -> bool {
    match expr {
        Expr::FunctionCall { name, args } if EXEC_FNS.contains(&name.as_str()) => {
            args.iter().any(|a| !is_static_string(a))
        }
        _ => false,
    }
}

fn is_static_string(expr: &Expr) -> bool {
    match expr {
        Expr::Literal(_) => true,
        Expr::FunctionCall { name, args } if name == "__format_concat" => {
            args.iter().all(is_static_string)
        }
        _ => false,
    }
}

fn walk_stmts<'a>(stmts: &'a [Stmt], f: &mut dyn FnMut(&'a Expr)) {
    for stmt in stmts {
        walk_stmt(stmt, f);
    }
}

fn walk_stmt<'a>(stmt: &'a Stmt, f: &mut dyn FnMut(&'a Expr)) {
    match stmt {
        Stmt::Let { value, .. } | Stmt::Expr(value) => walk_expr(value, f),
        Stmt::Return(Some(expr)) => walk_expr(expr, f),
        Stmt::If {
            condition,
            then_block,
            else_block,
        } => {
            walk_expr(condition, f);
            walk_stmts(then_block, f);
            if let Some(block) = else_block {
                walk_stmts(block, f);
            }
        }
        Stmt::Match { scrutinee, arms } => {
            walk_expr(scrutinee, f);
            for arm in arms {
                walk_arm(arm, f);
            }
        }
        Stmt::For { iter, body, .. } => {
            walk_expr(iter, f);
            walk_stmts(body, f);
        }
        Stmt::While {
            condition, body, ..
        } => {
            walk_expr(condition, f);
            walk_stmts(body, f);
        }
        Stmt::Return(None) | Stmt::Break | Stmt::Continue => {}
    }
}

fn walk_arm<'a>(arm: &'a MatchArm, f: &mut dyn FnMut(&'a Expr)) {
    if let Some(guard) = &arm.guard {
        walk_expr(guard, f);
    }
    walk_stmts(&arm.body, f);
}

fn walk_expr<'a>(expr: &'a Expr, f: &mut dyn FnMut(&'a Expr)) {
    f(expr);
    for child in children(expr) {
        walk_expr(child, f);
    }
    if let Expr::Block(stmts) = expr {
        walk_stmts(stmts, f);
    }
}

/// The direct sub-expressions of `expr` (a block's statements are walked separately).
fn children(expr: &Expr) -> Vec<&Expr> {
    match expr {
        Expr::FunctionCall { args, .. } | Expr::Array(args) => args.iter().collect(),
        Expr::MethodCall { receiver, args, .. } => std::iter::once(by_ref(receiver))
            .chain(args.iter())
            .collect(),
        Expr::Binary { left, right, .. } => vec![by_ref(left), by_ref(right)],
        Expr::Unary { operand, .. } => vec![by_ref(operand)],
        Expr::Index { object, index } => vec![by_ref(object), by_ref(index)],
        Expr::Try { expr } => vec![by_ref(expr)],
        Expr::Range { start, end, .. } => vec![by_ref(start), by_ref(end)],
        Expr::Block(_) | Expr::Literal(_) | Expr::Variable(_) | Expr::PositionalArgs => Vec::new(),
    }
}

/// Deref coercion point: turns `&Box<Expr>` (or `&Expr`) into `&Expr`.
fn by_ref(expr: &Expr) -> &Expr {
    expr
}
