#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::parser::parse;
use crate::ast::restricted::{BinaryOp, Expr, Literal, Pattern, Stmt, Type};

// ============================================================================
// convert_let_stmt: tuple destructuring (line 213)
// ============================================================================

#[test]
fn test_tuple_destructuring_two_elements() {
    let ast = parse(r#"fn main() { let (a, b) = f(); }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Expr(Expr::Block(stmts)) => {
            assert_eq!(stmts.len(), 3);
            assert!(matches!(&stmts[0], Stmt::Let { name, .. } if name == "__tuple_tmp"));
            assert!(matches!(&stmts[1], Stmt::Let { name, .. } if name == "a"));
            assert!(matches!(&stmts[2], Stmt::Let { name, .. } if name == "b"));
        }
        _ => panic!("Expected Block for tuple destructuring"),
    }
}

#[test]
fn test_tuple_destructuring_three_elements() {
    let ast = parse(r#"fn main() { let (x, y, z) = triple(); }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Expr(Expr::Block(stmts)) => assert_eq!(stmts.len(), 4),
        _ => panic!("Expected Block with 4 stmts"),
    }
}

#[test]
fn test_let_type_annotated_u32() {
    // Pat::Type branch in convert_let_stmt
    let ast = parse(r#"fn main() { let x: u32 = 5; }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let {
            name,
            value,
            declaration,
        } => {
            assert_eq!(name, "x");
            assert!(*declaration);
            assert!(matches!(value, Expr::Literal(Literal::U32(5))));
        }
        _ => panic!("Expected typed let"),
    }
}

#[test]
fn test_let_type_annotated_str() {
    let ast = parse(r#"fn main() { let msg: &str = "hi"; }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let { name, value, .. } => {
            assert_eq!(name, "msg");
            assert!(matches!(value, Expr::Literal(Literal::Str(_))));
        }
        _ => panic!("Expected typed str let"),
    }
}

// ============================================================================
// convert_assign_stmt: assignment targets (line 316)
// ============================================================================

#[test]
fn test_assign_simple_variable() {
    let ast = parse(r#"fn main() { let mut x = 0; x = 42; }"#).unwrap();
    match &ast.functions[0].body[1] {
        Stmt::Let {
            name,
            value,
            declaration,
        } => {
            assert_eq!(name, "x");
            assert!(!declaration);
            assert!(matches!(value, Expr::Literal(Literal::U32(42))));
        }
        _ => panic!("Expected reassignment"),
    }
}

#[test]
fn test_assign_array_literal_index() {
    // arr[0] = value — flat array convention name "arr_0"
    let ast = parse(r#"fn main() { let arr = [1]; arr[0] = 9; }"#).unwrap();
    match &ast.functions[0].body[1] {
        Stmt::Let {
            name, declaration, ..
        } => {
            assert_eq!(name, "arr_0");
            assert!(!declaration);
        }
        _ => panic!("Expected arr_0"),
    }
}

#[test]
fn test_assign_array_variable_index() {
    let ast = parse(r#"fn main() { let arr = [1]; let i = 0; arr[i] = 5; }"#).unwrap();
    match &ast.functions[0].body[2] {
        Stmt::Let {
            name, declaration, ..
        } => {
            assert_eq!(name, "arr_i");
            assert!(!declaration);
        }
        _ => panic!("Expected arr_i"),
    }
}

#[test]
fn test_assign_field_named_from_impl() {
    let src = r#"
        struct Foo { v: u32 }
        impl Foo { fn set(&mut self) { self.v = 1; } }
        fn main() { let x = 0; }
    "#;
    let ast = parse(src).unwrap();
    let set_fn = ast.functions.iter().find(|f| f.name == "set").unwrap();
    match &set_fn.body[0] {
        Stmt::Let {
            name, declaration, ..
        } => {
            assert_eq!(name, "v");
            assert!(!declaration);
        }
        _ => panic!("Expected field assignment"),
    }
}

#[test]
fn test_assign_deref() {
    let ast = parse(r#"fn main() { let mut x = 0; let p = &mut x; *p = 7; }"#).unwrap();
    match &ast.functions[0].body[2] {
        Stmt::Let {
            name, declaration, ..
        } => {
            assert_eq!(name, "p");
            assert!(!declaration);
        }
        _ => panic!("Expected deref assignment"),
    }
}

// ============================================================================
// convert_expr_stmt dispatch (line 283)
// ============================================================================

#[test]
fn test_expr_stmt_if() {
    let ast = parse(r#"fn main() { if x > 0 { let a = 1; } }"#).unwrap();
    assert!(matches!(
        &ast.functions[0].body[0],
        Stmt::If {
            else_block: None,
            ..
        }
    ));
}

#[test]
fn test_expr_stmt_for_loop_with_range() {
    let ast = parse(r#"fn main() { for i in 0..5 { let v = i; } }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::For {
            pattern,
            max_iterations,
            ..
        } => {
            assert!(matches!(pattern, Pattern::Variable(n) if n == "i"));
            assert_eq!(*max_iterations, Some(1000));
        }
        _ => panic!("Expected For"),
    }
}

#[test]
fn test_expr_stmt_for_wildcard_pattern() {
    let ast = parse(r#"fn main() { for _ in 0..3 { do_it(); } }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::For { pattern, .. } => {
            assert!(matches!(pattern, Pattern::Variable(n) if n == "_unused_"));
        }
        _ => panic!("Expected For with _unused_"),
    }
}

#[test]
fn test_expr_stmt_while() {
    let ast = parse(r#"fn main() { while x < 10 { x = x + 1; } }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::While { max_iterations, .. } => assert_eq!(*max_iterations, Some(10000)),
        _ => panic!("Expected While"),
    }
}

#[test]
fn test_expr_stmt_loop_becomes_while_true() {
    let ast = parse(r#"fn main() { loop { break; } }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::While { condition, .. } => {
            assert!(matches!(condition, Expr::Literal(Literal::Bool(true))));
        }
        _ => panic!("Expected While(true)"),
    }
}

#[test]
fn test_expr_stmt_break_and_continue() {
    let ast = parse(r#"fn main() { while true { break; } }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::While { body, .. } => assert!(matches!(body[0], Stmt::Break)),
        _ => panic!("Expected Break"),
    }
    let ast2 = parse(r#"fn main() { for i in 0..3 { continue; } }"#).unwrap();
    match &ast2.functions[0].body[0] {
        Stmt::For { body, .. } => assert!(matches!(body[0], Stmt::Continue)),
        _ => panic!("Expected Continue"),
    }
}

#[test]
fn test_expr_stmt_return_variants() {
    let ast = parse(r#"fn main() { return 42; }"#).unwrap();
    assert!(matches!(
        &ast.functions[0].body[0],
        Stmt::Return(Some(Expr::Literal(Literal::U32(42))))
    ));
    let ast2 = parse(r#"fn main() { return; }"#).unwrap();
    assert!(matches!(&ast2.functions[0].body[0], Stmt::Return(None)));
}

#[test]
fn test_expr_stmt_match() {
    let ast =
        parse(r#"fn main() { match x { 0 => { let a = 1; } _ => { let b = 2; } } }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Match { arms, .. } => assert_eq!(arms.len(), 2),
        _ => panic!("Expected Match"),
    }
}

#[test]
fn test_match_with_guard() {
    let ast =
        parse(r#"fn main() { match x { n if n > 0 => { let p = true; } _ => {} } }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Match { arms, .. } => assert!(arms[0].guard.is_some()),
        _ => panic!("Expected Match with guard"),
    }
}

// ============================================================================
// if-else as expression
// ============================================================================

#[test]
fn test_if_else_as_expr_single_branch_exprs() {
    let ast = parse(r#"fn main() { let x = if true { 1 } else { 2 }; }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let {
            value: Expr::FunctionCall { name, args },
            ..
        } => {
            assert_eq!(name, "__if_expr");
            assert_eq!(args.len(), 3);
        }
        _ => panic!("Expected __if_expr"),
    }
}

#[test]
fn test_if_expr_no_else_defaults_empty() {
    let ast = parse(r#"fn main() { let x = if f { "y" }; }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let {
            value: Expr::FunctionCall { name, args },
            ..
        } => {
            assert_eq!(name, "__if_expr");
            assert!(matches!(&args[2], Expr::Literal(Literal::Str(s)) if s.is_empty()));
        }
        _ => panic!("Expected __if_expr with empty else"),
    }
}

#[test]
fn test_if_else_multi_stmt_produces_block() {
    let ast = parse(r#"fn main() { let x = if c { let t = 1; t } else { 2 }; }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let {
            value: Expr::Block(stmts),
            ..
        } => {
            assert!(matches!(stmts[0], Stmt::If { .. }));
        }
        _ => panic!("Expected Block for multi-stmt if expr"),
    }
}

// ============================================================================
// Compound assignments
// ============================================================================

#[test]
fn test_compound_add_assign() {
    let ast = parse(r#"fn main() { let mut x = 0; x += 5; }"#).unwrap();
    match &ast.functions[0].body[1] {
        Stmt::Let {
            name,
            value,
            declaration,
        } => {
            assert_eq!(name, "x");
            assert!(!declaration);
            assert!(matches!(
                value,
                Expr::Binary {
                    op: BinaryOp::Add,
                    ..
                }
            ));
        }
        _ => panic!("Expected += compound"),
    }
}

#[test]
fn test_compound_sub_and_mul_assign() {
    let ast_sub = parse(r#"fn main() { let mut c = 10; c -= 3; }"#).unwrap();
    match &ast_sub.functions[0].body[1] {
        Stmt::Let { value, .. } => assert!(matches!(
            value,
            Expr::Binary {
                op: BinaryOp::Sub,
                ..
            }
        )),
        _ => panic!("Expected -= compound"),
    }
    let ast_mul = parse(r#"fn main() { let mut n = 2; n *= 3; }"#).unwrap();
    match &ast_mul.functions[0].body[1] {
        Stmt::Let { value, .. } => assert!(matches!(
            value,
            Expr::Binary {
                op: BinaryOp::Mul,
                ..
            }
        )),
        _ => panic!("Expected *= compound"),
    }
}

// ============================================================================
// Match patterns: Some/None/Ok/Err, range
// ============================================================================

#[test]

include!("parser_coverage_tests_tests_match_some.rs");
