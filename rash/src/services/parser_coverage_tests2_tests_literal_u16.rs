fn test_literal_u16_suffix() {
    let ast = parse(r#"#[bashrs::main] fn f(x: u16) { let v: u16 = 1000u16; }"#).unwrap();
    assert!(
        matches!(&ast.functions[0].body[0], Stmt::Let { value: Expr::Literal(Literal::U16(v)), .. } if *v == 1000)
    );
}

#[test]
fn test_literal_bool_false() {
    let ast = parse(r#"fn main() { let b = false; }"#).unwrap();
    assert!(matches!(
        &ast.functions[0].body[0],
        Stmt::Let {
            value: Expr::Literal(Literal::Bool(false)),
            ..
        }
    ));
}

// ============================================================================
// convert_struct_expr and convert_let_expr
// ============================================================================

#[test]
fn test_struct_expr_becomes_array() {
    let src = r#"struct Point { x: u32, y: u32 } fn main() { let p = Point { x: 1, y: 2 }; }"#;
    let ast = parse(src).unwrap();
    assert!(
        matches!(&ast.functions[0].body[0], Stmt::Let { value: Expr::Array(e), .. } if e.len() == 2)
    );
}

#[test]
fn test_let_expr_in_if_condition() {
    let ast = parse(r#"fn main() { let opt = 1; if let 1 = opt { let a = 1; } }"#).unwrap();
    assert!(matches!(
        &ast.functions[0].body[1],
        Stmt::If {
            condition: Expr::Binary {
                op: BinaryOp::Eq,
                ..
            },
            ..
        }
    ));
}

// ============================================================================
// process_item: item handling
// ============================================================================

#[test]
fn test_item_types_silently_skipped() {
    assert_eq!(
        parse(r#"trait Foo {} fn main() { let x = 0; }"#)
            .unwrap()
            .entry_point,
        "main"
    );
    assert_eq!(
        parse(r#"use std::io; fn main() { let x = 0; }"#)
            .unwrap()
            .entry_point,
        "main"
    );
    assert_eq!(
        parse(r#"const MAX: u32 = 100; fn main() { let x = 0; }"#)
            .unwrap()
            .entry_point,
        "main"
    );
    assert_eq!(
        parse(r#"static NAME: &str = "rash"; fn main() { let x = 0; }"#)
            .unwrap()
            .entry_point,
        "main"
    );
    assert_eq!(
        parse(r#"type MyU32 = u32; fn main() { let x = 0; }"#)
            .unwrap()
            .entry_point,
        "main"
    );
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
    assert!(matches!(
        &ast.functions[0].body[1],
        Stmt::Let {
            value: Expr::Index { .. },
            ..
        }
    ));
}

#[test]
fn test_nested_index_target_two_levels() {
    let ast = parse(r#"fn main() { let arr = [[1]]; arr[0][0] = 5; }"#).unwrap();
    match &ast.functions[0].body[1] {
        Stmt::Let {
            name, declaration, ..
        } => {
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
        Stmt::If {
            else_block: Some(e1),
            ..
        } => match &e1[0] {
            Stmt::If {
                else_block: Some(e2),
                ..
            } => match &e2[0] {
                Stmt::If {
                    else_block: Some(e3),
                    ..
                } => assert_eq!(e3.len(), 1),
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
    let ast = parse(r#"fn main() { match x { 0 => println!("zero"), _ => println!("other") } }"#)
        .unwrap();
    assert!(matches!(&ast.functions[0].body[0], Stmt::Match { arms, .. } if arms.len() == 2));
}
