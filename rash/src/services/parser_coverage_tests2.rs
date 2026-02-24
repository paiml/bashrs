#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::parser::parse;
use crate::ast::restricted::{BinaryOp, Expr, Literal, Pattern, Stmt, Type, UnaryOp};

// ============================================================================
// convert_method_call_expr: method calls on receivers
// ============================================================================

#[test]
fn test_method_call_basic_methods() {
    // push, len, trim, to_string all produce MethodCall nodes
    let ast = parse(r#"fn main() { let mut v = vec![1]; v.push(2); }"#).unwrap();
    assert!(matches!(&ast.functions[0].body[1], Stmt::Expr(Expr::MethodCall { method, .. }) if method == "push"));
    let ast2 = parse(r#"fn main() { let v = vec![1]; let n = v.len(); }"#).unwrap();
    assert!(matches!(&ast2.functions[0].body[1], Stmt::Let { value: Expr::MethodCall { method, .. }, .. } if method == "len"));
    let ast3 = parse(r#"fn main() { let s = "hi"; let t = s.trim(); }"#).unwrap();
    assert!(matches!(&ast3.functions[0].body[1], Stmt::Let { value: Expr::MethodCall { method, .. }, .. } if method == "trim"));
}

#[test]
fn test_method_call_with_args_and_chained() {
    let ast = parse(r#"fn main() { let s = "hi"; let b = s.contains("h"); }"#).unwrap();
    match &ast.functions[0].body[1] {
        Stmt::Let { value: Expr::MethodCall { method, args, .. }, .. } => {
            assert_eq!(method, "contains");
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected MethodCall contains"),
    }
    // Chained method call: receiver is itself a MethodCall
    let ast2 = parse(r#"fn main() { let s = "hi"; let t = s.trim().to_string(); }"#).unwrap();
    match &ast2.functions[0].body[1] {
        Stmt::Let { value: Expr::MethodCall { method, receiver, .. }, .. } => {
            assert_eq!(method, "to_string");
            assert!(matches!(**receiver, Expr::MethodCall { .. }));
        }
        _ => panic!("Expected chained MethodCall"),
    }
}

#[test]
fn test_method_call_env_args_collect_to_positional() {
    let ast = parse(r#"fn main() { let args: Vec<String> = std::env::args().collect(); }"#).unwrap();
    assert!(matches!(&ast.functions[0].body[0], Stmt::Let { value: Expr::PositionalArgs, .. }));
}

// ============================================================================
// convert_macro_stmt and convert_macro_expr
// ============================================================================

#[test]
fn test_macro_stmt_variants() {
    // println! single arg
    let ast = parse(r#"fn main() { println!("hello"); }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Expr(Expr::FunctionCall { name, args }) => {
            assert_eq!(name, "rash_println");
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected rash_println single arg"),
    }
    // eprintln! multi-arg produces __format_concat
    let ast2 = parse(r#"fn main() { eprintln!("error: {}", msg); }"#).unwrap();
    match &ast2.functions[0].body[0] {
        Stmt::Expr(Expr::FunctionCall { name, args }) => {
            assert_eq!(name, "rash_eprintln");
            assert!(matches!(&args[0], Expr::FunctionCall { name, .. } if name == "__format_concat"));
        }
        _ => panic!("Expected rash_eprintln with format_concat"),
    }
    // print! no-newline
    let ast3 = parse(r#"fn main() { print!("x"); }"#).unwrap();
    assert!(matches!(&ast3.functions[0].body[0], Stmt::Expr(Expr::FunctionCall { name, .. }) if name == "rash_print"));
    // unsupported macro returns error
    assert!(parse(r#"fn main() { panic!("boom"); }"#).is_err());
}

#[test]
fn test_macro_expr_format() {
    // format! with single arg produces a plain Str literal
    let ast = parse(r#"fn main() { let s = format!("hello"); }"#).unwrap();
    assert!(matches!(&ast.functions[0].body[0], Stmt::Let { value: Expr::Literal(Literal::Str(_)), .. }));
    // format! with multiple args produces __format_concat
    let ast2 = parse(r#"fn main() { let s = format!("hi {}", name); }"#).unwrap();
    assert!(matches!(&ast2.functions[0].body[0], Stmt::Let { value: Expr::FunctionCall { name, .. }, .. } if name == "__format_concat"));
    // unsupported macro in expression returns error
    assert!(parse(r#"fn main() { let x = assert!(true); }"#).is_err());
}

// ============================================================================
// convert_call_expr: function calls
// ============================================================================

#[test]
fn test_function_call_arities() {
    let ast = parse(r#"fn main() { let r = foo(); }"#).unwrap();
    assert!(matches!(&ast.functions[0].body[0], Stmt::Let { value: Expr::FunctionCall { name, args }, .. } if name == "foo" && args.is_empty()));
    let ast2 = parse(r#"fn main() { let r = add(1, 2, 3); }"#).unwrap();
    match &ast2.functions[0].body[0] {
        Stmt::Let { value: Expr::FunctionCall { name, args }, .. } => {
            assert_eq!(name, "add");
            assert_eq!(args.len(), 3);
        }
        _ => panic!("Expected add(1,2,3)"),
    }
    // nested path like std::env::var
    let ast3 = parse(r#"fn main() { let r = std::env::var("HOME"); }"#).unwrap();
    assert!(matches!(&ast3.functions[0].body[0], Stmt::Let { value: Expr::FunctionCall { name, .. }, .. } if name == "std::env::var"));
    // closure call not supported
    assert!(parse(r#"fn main() { let f = (|x| x)(5); }"#).is_err());
}

// ============================================================================
// convert_impl_block
// ============================================================================

#[test]
fn test_impl_block_multiple_methods() {
    let src = r#"
        struct Counter { count: u32 }
        impl Counter {
            fn increment(&mut self) { self.count = self.count + 1; }
            fn reset(&mut self) { self.count = 0; }
        }
        fn main() { let x = 0; }
    "#;
    let ast = parse(src).unwrap();
    let names: Vec<&str> = ast.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"increment"));
    assert!(names.contains(&"reset"));
}

#[test]
fn test_impl_block_empty_skipped() {
    let src = r#"struct Foo {} impl Foo {} fn main() { let x = 0; }"#;
    let ast = parse(src).unwrap();
    assert_eq!(ast.entry_point, "main");
}

// ============================================================================
// convert_closure
// ============================================================================

#[test]
fn test_closure_body_extracted() {
    let ast = parse(r#"fn main() { let f = |x| x + 1; }"#).unwrap();
    assert!(matches!(&ast.functions[0].body[0], Stmt::Let { value: Expr::Binary { op: BinaryOp::Add, .. }, .. }));
}

#[test]
fn test_closure_with_block_body() {
    let ast = parse(r#"fn main() { let f = |x| { let y = x; y }; }"#).unwrap();
    assert!(matches!(&ast.functions[0].body[0], Stmt::Let { value: Expr::Block(_), .. }));
}

// ============================================================================
// convert_unary_expr
// ============================================================================

#[test]
fn test_unary_operators() {
    // !b → Unary Not
    let ast = parse(r#"fn main() { let b = true; let n = !b; }"#).unwrap();
    assert!(matches!(&ast.functions[0].body[1], Stmt::Let { value: Expr::Unary { op: UnaryOp::Not, .. }, .. }));
    // -var → Unary Neg on variable
    let ast2 = parse(r#"fn main() { let x = 5; let n = -x; }"#).unwrap();
    assert!(matches!(&ast2.functions[0].body[1], Stmt::Let { value: Expr::Unary { op: UnaryOp::Neg, .. }, .. }));
    // -5 literal → I32(-5) directly (try_negate_int_literal path)
    let ast3 = parse(r#"fn main() { let n = -5; }"#).unwrap();
    assert!(matches!(&ast3.functions[0].body[0], Stmt::Let { value: Expr::Literal(Literal::I32(-5)), .. }));
    // *r → deref strips to variable (Deref path)
    let ast4 = parse(r#"fn main() { let x = 5; let r = &x; let d = *r; }"#).unwrap();
    assert!(matches!(&ast4.functions[0].body[2], Stmt::Let { value: Expr::Variable(n), .. } if n == "r"));
}

#[test]
fn test_unary_neg_i32_min_edge_case() {
    let ast = parse(r#"fn main() { let n = -2147483648; }"#).unwrap();
    assert!(matches!(&ast.functions[0].body[0], Stmt::Let { value: Expr::Literal(Literal::I32(v)), .. } if *v == i32::MIN));
}

// ============================================================================
// convert_binary_op: bitwise + shift + rem operators
// ============================================================================

#[test]
fn test_binary_ops_bitwise_and_shift() {
    let ast = parse(r#"fn main() { let a = 3; let b = 5; let c = a & b; }"#).unwrap();
    assert!(matches!(&ast.functions[0].body[2], Stmt::Let { value: Expr::Binary { op: BinaryOp::BitAnd, .. }, .. }));
    let ast2 = parse(r#"fn main() { let a = 3; let b = 5; let c = a | b; }"#).unwrap();
    assert!(matches!(&ast2.functions[0].body[2], Stmt::Let { value: Expr::Binary { op: BinaryOp::BitOr, .. }, .. }));
    let ast3 = parse(r#"fn main() { let a = 3; let b = 5; let c = a ^ b; }"#).unwrap();
    assert!(matches!(&ast3.functions[0].body[2], Stmt::Let { value: Expr::Binary { op: BinaryOp::BitXor, .. }, .. }));
    let ast4 = parse(r#"fn main() { let a = 1; let b = a << 2; }"#).unwrap();
    assert!(matches!(&ast4.functions[0].body[1], Stmt::Let { value: Expr::Binary { op: BinaryOp::Shl, .. }, .. }));
    let ast5 = parse(r#"fn main() { let a = 8; let b = a >> 2; }"#).unwrap();
    assert!(matches!(&ast5.functions[0].body[1], Stmt::Let { value: Expr::Binary { op: BinaryOp::Shr, .. }, .. }));
    let ast6 = parse(r#"fn main() { let a = 10; let b = a % 3; }"#).unwrap();
    assert!(matches!(&ast6.functions[0].body[1], Stmt::Let { value: Expr::Binary { op: BinaryOp::Rem, .. }, .. }));
}

// ============================================================================
// compound assignment: &=, |=, ^=, <<=, >>=, /=, %=
// ============================================================================

#[test]
fn test_compound_bitwise_assigns() {
    let ast = parse(r#"fn main() { let mut x = 0xFF; x &= 0x0F; }"#).unwrap();
    assert!(matches!(&ast.functions[0].body[1], Stmt::Let { value: Expr::Binary { op: BinaryOp::BitAnd, .. }, .. }));
    let ast2 = parse(r#"fn main() { let mut x = 0; x |= 1; }"#).unwrap();
    assert!(matches!(&ast2.functions[0].body[1], Stmt::Let { value: Expr::Binary { op: BinaryOp::BitOr, .. }, .. }));
    let ast3 = parse(r#"fn main() { let mut x = 5; x ^= 3; }"#).unwrap();
    assert!(matches!(&ast3.functions[0].body[1], Stmt::Let { value: Expr::Binary { op: BinaryOp::BitXor, .. }, .. }));
    let ast4 = parse(r#"fn main() { let mut x = 1; x <<= 2; }"#).unwrap();
    assert!(matches!(&ast4.functions[0].body[1], Stmt::Let { value: Expr::Binary { op: BinaryOp::Shl, .. }, .. }));
    let ast5 = parse(r#"fn main() { let mut x = 8; x >>= 1; }"#).unwrap();
    assert!(matches!(&ast5.functions[0].body[1], Stmt::Let { value: Expr::Binary { op: BinaryOp::Shr, .. }, .. }));
    let ast6 = parse(r#"fn main() { let mut x = 10; x /= 2; }"#).unwrap();
    assert!(matches!(&ast6.functions[0].body[1], Stmt::Let { value: Expr::Binary { op: BinaryOp::Div, .. }, .. }));
    let ast7 = parse(r#"fn main() { let mut x = 10; x %= 3; }"#).unwrap();
    assert!(matches!(&ast7.functions[0].body[1], Stmt::Let { value: Expr::Binary { op: BinaryOp::Rem, .. }, .. }));
}

// ============================================================================
// convert_literal: suffix variants
// ============================================================================

#[test]
fn test_literal_u16_suffix() {
    let ast = parse(r#"#[bashrs::main] fn f(x: u16) { let v: u16 = 1000u16; }"#).unwrap();
    assert!(matches!(&ast.functions[0].body[0], Stmt::Let { value: Expr::Literal(Literal::U16(v)), .. } if *v == 1000));
}

#[test]
fn test_literal_bool_false() {
    let ast = parse(r#"fn main() { let b = false; }"#).unwrap();
    assert!(matches!(&ast.functions[0].body[0], Stmt::Let { value: Expr::Literal(Literal::Bool(false)), .. }));
}

// ============================================================================
// convert_struct_expr and convert_let_expr
// ============================================================================

#[test]
fn test_struct_expr_becomes_array() {
    let src = r#"struct Point { x: u32, y: u32 } fn main() { let p = Point { x: 1, y: 2 }; }"#;
    let ast = parse(src).unwrap();
    assert!(matches!(&ast.functions[0].body[0], Stmt::Let { value: Expr::Array(e), .. } if e.len() == 2));
}

#[test]
fn test_let_expr_in_if_condition() {
    let ast = parse(r#"fn main() { let opt = 1; if let 1 = opt { let a = 1; } }"#).unwrap();
    assert!(matches!(&ast.functions[0].body[1], Stmt::If { condition: Expr::Binary { op: BinaryOp::Eq, .. }, .. }));
}

// ============================================================================
// process_item: item handling
// ============================================================================

#[test]
fn test_item_types_silently_skipped() {
    assert_eq!(parse(r#"trait Foo {} fn main() { let x = 0; }"#).unwrap().entry_point, "main");
    assert_eq!(parse(r#"use std::io; fn main() { let x = 0; }"#).unwrap().entry_point, "main");
    assert_eq!(parse(r#"const MAX: u32 = 100; fn main() { let x = 0; }"#).unwrap().entry_point, "main");
    assert_eq!(parse(r#"static NAME: &str = "rash"; fn main() { let x = 0; }"#).unwrap().entry_point, "main");
    assert_eq!(parse(r#"type MyU32 = u32; fn main() { let x = 0; }"#).unwrap().entry_point, "main");
}

#[test]
fn test_unsupported_item_returns_error() {
    assert!(parse(r#"extern "C" { fn c_func(); } fn main() { let x = 0; }"#).is_err());
}

// ============================================================================
// convert_type: various type annotations
// ============================================================================

#[test]
fn test_type_annotations() {
    let ast = parse(r#"#[bashrs::main] fn f(n: i32) { let x = n; }"#).unwrap();
    assert!(matches!(ast.functions[0].params[0].param_type, Type::U32));
    let ast2 = parse(r#"#[bashrs::main] fn f(s: String) { let x = s; }"#).unwrap();
    assert!(matches!(ast2.functions[0].params[0].param_type, Type::Str));
    let ast3 = parse(r#"#[bashrs::main] fn f(b: bool) { let x = b; }"#).unwrap();
    assert!(matches!(ast3.functions[0].params[0].param_type, Type::Bool));
    let ast4 = parse(r#"#[bashrs::main] fn f(s: &str) { let x = s; }"#).unwrap();
    assert!(matches!(ast4.functions[0].params[0].param_type, Type::Str));
    let ast5 = parse(r#"fn main() { let x = 0; }"#).unwrap();
    assert!(matches!(ast5.functions[0].return_type, Type::Void));
    let ast6 = parse(r#"#[bashrs::main] fn f() -> String { let x = "ok"; }"#).unwrap();
    assert!(matches!(ast6.functions[0].return_type, Type::Str));
}

// ============================================================================
// convert_pattern: pat_path for None, path patterns
// ============================================================================

#[test]
fn test_match_none_via_path_pattern() {
    // syn parses bare None as Pat::Ident, convert_pattern maps to Variable("None")
    let ast = parse(r#"fn main() { match opt { None => { let x = 0; } _ => {} } }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Match { arms, .. } => {
            assert!(matches!(&arms[0].pattern, Pattern::Variable(n) if n == "None"));
        }
        _ => panic!("Expected Match with None pattern"),
    }
}

// ============================================================================
// extract_index_suffix and nested index targets
// ============================================================================

#[test]
fn test_array_access_with_binary_index() {
    let ast = parse(r#"fn main() { let arr = [1, 2, 3]; let v = arr[0 + 1]; }"#).unwrap();
    assert!(matches!(&ast.functions[0].body[1], Stmt::Let { value: Expr::Index { .. }, .. }));
}

#[test]
fn test_nested_index_target_two_levels() {
    let ast = parse(r#"fn main() { let arr = [[1]]; arr[0][0] = 5; }"#).unwrap();
    match &ast.functions[0].body[1] {
        Stmt::Let { name, declaration, .. } => {
            assert!(!declaration);
            assert!(name.contains('_'));
        }
        _ => panic!("Expected nested index assignment"),
    }
}

// ============================================================================
// else-if chains and multiple entry points
// ============================================================================

#[test]
fn test_else_if_chain_with_final_else_three_levels() {
    let src = r#"fn main() {
        if a { let x = 1; }
        else if b { let y = 2; }
        else if c { let z = 3; }
        else { let w = 4; }
    }"#;
    let ast = parse(src).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::If { else_block: Some(e1), .. } => match &e1[0] {
            Stmt::If { else_block: Some(e2), .. } => match &e2[0] {
                Stmt::If { else_block: Some(e3), .. } => assert_eq!(e3.len(), 1),
                _ => panic!("Expected third-level if"),
            },
            _ => panic!("Expected second-level if"),
        },
        _ => panic!("Expected chained else-if"),
    }
}

#[test]
fn test_multiple_main_functions_is_error() {
    assert!(parse(r#"fn main() { let x = 1; } fn main() { let y = 2; }"#).is_err());
}

#[test]
fn test_no_main_function_is_error() {
    assert!(parse(r#"fn foo() { let x = 1; }"#).is_err());
}

// ============================================================================
// convert_match_stmt: arm bodies
// ============================================================================

#[test]
fn test_match_arm_non_block_body() {
    let ast = parse(r#"fn main() { match x { 0 => foo(), _ => bar() } }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Match { arms, .. } => {
            assert_eq!(arms.len(), 2);
            assert_eq!(arms[0].body.len(), 1);
        }
        _ => panic!("Expected Match"),
    }
}

#[test]
fn test_match_arm_with_macro_body() {
    let ast = parse(r#"fn main() { match x { 0 => println!("zero"), _ => println!("other") } }"#).unwrap();
    assert!(matches!(&ast.functions[0].body[0], Stmt::Match { arms, .. } if arms.len() == 2));
}
