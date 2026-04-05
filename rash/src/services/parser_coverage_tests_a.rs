#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::parser::parse;
use crate::ast::restricted::{BinaryOp, Expr, Literal, Pattern, Stmt, Type};

// ============================================================================
// convert_let_stmt: tuple destructuring (line 213)
// ============================================================================

#[test]
fn test_repeat_expression() {
    let ast = parse(r#"fn main() { let a = [0; 5]; }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let {
            value: Expr::Array(e),
            ..
        } => assert_eq!(e.len(), 5),
        _ => panic!("Expected repeat Array"),
    }
}

#[test]
fn test_cast_strips_to_inner() {
    let ast = parse(r#"fn main() { let x = 5u16; let y = x as u32; }"#).unwrap();
    match &ast.functions[0].body[1] {
        Stmt::Let {
            value: Expr::Variable(n),
            ..
        } => assert_eq!(n, "x"),
        _ => panic!("Expected Variable after cast strip"),
    }
}

#[test]
fn test_exclusive_range() {
    let ast = parse(r#"fn main() { for i in 0..10 { let v = i; } }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::For {
            iter:
                Expr::Range {
                    inclusive,
                    start,
                    end,
                },
            ..
        } => {
            assert!(!inclusive);
            assert!(matches!(**start, Expr::Literal(Literal::U32(0))));
            assert!(matches!(**end, Expr::Literal(Literal::U32(10))));
        }
        _ => panic!("Expected exclusive Range"),
    }
}

#[test]
fn test_inclusive_range() {
    let ast = parse(r#"fn main() { for i in 1..=5 { let v = i; } }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::For {
            iter: Expr::Range { inclusive, .. },
            ..
        } => assert!(*inclusive),
        _ => panic!("Expected inclusive Range"),
    }
}

#[test]
fn test_vec_macro() {
    let ast = parse(r#"fn main() { let v = vec![1, 2, 3]; }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let {
            value: Expr::Array(e),
            ..
        } => assert_eq!(e.len(), 3),
        _ => panic!("Expected Array from vec!"),
    }
}

#[test]
fn test_reference_unwrapped() {
    let ast = parse(r#"fn main() { let x = 5; let r = &x; }"#).unwrap();
    match &ast.functions[0].body[1] {
        Stmt::Let {
            value: Expr::Variable(n),
            ..
        } => assert_eq!(n, "x"),
        _ => panic!("Expected Variable after & unwrap"),
    }
}

// ============================================================================
// Types and impl blocks
// ============================================================================

#[test]
fn test_type_u16_param() {
    let ast = parse(r#"#[bashrs::main] fn p(port: u16) { let x = port; }"#).unwrap();
    assert!(matches!(ast.functions[0].params[0].param_type, Type::U16));
}

#[test]
fn test_type_result_and_option() {
    let ast = parse(r#"#[bashrs::main] fn f() -> Result<String, String> { let x = 1; }"#).unwrap();
    assert!(matches!(ast.functions[0].return_type, Type::Result { .. }));
    let ast2 = parse(r#"#[bashrs::main] fn f(x: Option<u32>) { let v = x; }"#).unwrap();
    assert!(matches!(
        ast2.functions[0].params[0].param_type,
        Type::Option { .. }
    ));
}

#[test]
fn test_impl_methods_extracted() {
    let src = r#"
        struct S { v: u32 }
        impl S { fn inc(&mut self) { self.v = self.v + 1; } }
        fn main() { let x = 0; }
    "#;
    let ast = parse(src).unwrap();
    assert!(ast.functions.iter().any(|f| f.name == "inc"));
}

#[test]
fn test_for_over_array_and_variable_iter() {
    let ast = parse(r#"fn main() { for it in ["a", "b"] { println!("{}", it); } }"#).unwrap();
    assert!(matches!(
        &ast.functions[0].body[0],
        Stmt::For {
            iter: Expr::Array(_),
            ..
        }
    ));
    let ast2 = parse(r#"fn main() { let items = [1]; for x in items { let v = x; } }"#).unwrap();
    match &ast2.functions[0].body[1] {
        Stmt::For {
            iter: Expr::Variable(n),
            ..
        } => assert_eq!(n, "items"),
        _ => panic!("Expected For over variable"),
    }
}
