fn test_match_expr_multi_arm_in_let() {
    let ast = parse(r#"fn main() { let x = match v { 0 => 10, 1 => 20, _ => 30 }; }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let {
            value: Expr::Block(stmts),
            ..
        } => match &stmts[0] {
            Stmt::Match { arms, .. } => assert_eq!(arms.len(), 3),
            _ => panic!("Expected Match inside Block"),
        },
        _ => panic!("Expected Block with Match"),
    }
}

// === Field access on function call result ===

#[test]
fn test_field_access_on_function_call_result() {
    let ast = parse(r#"fn main() { let v = foo().bar; }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let {
            value: Expr::Index { object, index },
            ..
        } => {
            assert!(matches!(**object, Expr::FunctionCall { .. }));
            assert!(matches!(**index, Expr::Literal(Literal::I32(0))));
        }
        _ => panic!("Expected Index from field access on call"),
    }
}

// === Unsupported expression type ===

#[test]
fn test_async_not_supported() {
    assert!(parse(r#"fn main() { let f = async { 1 }; }"#).is_err());
}

// === All compound assignment operators ===

#[test]
fn test_compound_assign_all_ops() {
    let src = r#"fn main() {
        let mut x = 100;
        x += 1; x -= 2; x *= 3; x /= 4; x %= 5;
        x &= 6; x |= 7; x ^= 8; x <<= 9; x >>= 10;
    }"#;
    let ast = parse(src).unwrap();
    let s = &ast.functions[0].body;
    assert!(matches!(
        &s[1],
        Stmt::Let {
            value: Expr::Binary {
                op: BinaryOp::Add,
                ..
            },
            ..
        }
    ));
    assert!(matches!(
        &s[2],
        Stmt::Let {
            value: Expr::Binary {
                op: BinaryOp::Sub,
                ..
            },
            ..
        }
    ));
    assert!(matches!(
        &s[3],
        Stmt::Let {
            value: Expr::Binary {
                op: BinaryOp::Mul,
                ..
            },
            ..
        }
    ));
    assert!(matches!(
        &s[4],
        Stmt::Let {
            value: Expr::Binary {
                op: BinaryOp::Div,
                ..
            },
            ..
        }
    ));
    assert!(matches!(
        &s[5],
        Stmt::Let {
            value: Expr::Binary {
                op: BinaryOp::Rem,
                ..
            },
            ..
        }
    ));
    assert!(matches!(
        &s[6],
        Stmt::Let {
            value: Expr::Binary {
                op: BinaryOp::BitAnd,
                ..
            },
            ..
        }
    ));
    assert!(matches!(
        &s[7],
        Stmt::Let {
            value: Expr::Binary {
                op: BinaryOp::BitOr,
                ..
            },
            ..
        }
    ));
    assert!(matches!(
        &s[8],
        Stmt::Let {
            value: Expr::Binary {
                op: BinaryOp::BitXor,
                ..
            },
            ..
        }
    ));
    assert!(matches!(
        &s[9],
        Stmt::Let {
            value: Expr::Binary {
                op: BinaryOp::Shl,
                ..
            },
            ..
        }
    ));
    assert!(matches!(
        &s[10],
        Stmt::Let {
            value: Expr::Binary {
                op: BinaryOp::Shr,
                ..
            },
            ..
        }
    ));
}

// === Unnamed field assignment ===

#[test]
fn test_unnamed_field_plain_assign() {
    let src = r#"
        struct W(u32);
        impl W { fn reset(&mut self) { self.0 = 0; } }
        fn main() { let x = 0; }
    "#;
    let ast = parse(src).unwrap();
    let f = ast.functions.iter().find(|f| f.name == "reset").unwrap();
    match &f.body[0] {
        Stmt::Let {
            name, declaration, ..
        } => {
            assert_eq!(name, "field_0");
            assert!(!declaration);
        }
        _ => panic!("Expected unnamed field assignment"),
    }
}

// === Closure with if-expr body ===

#[test]
fn test_closure_with_if_expr_body() {
    let ast = parse(r#"fn main() { let f = |x| if x > 0 { x } else { 0 }; }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let {
            value: Expr::FunctionCall { name, .. },
            ..
        } => {
            assert_eq!(name, "__if_expr");
        }
        _ => panic!("Expected __if_expr from closure"),
    }
}

// === Return expression in expression position ===

#[test]
fn test_return_expr_with_value() {
    let ast = parse(r#"fn main() { let f = |x| return 42; }"#).unwrap();
    assert!(matches!(
        &ast.functions[0].body[0],
        Stmt::Let {
            value: Expr::Literal(Literal::U32(42)),
            ..
        }
    ));
}

// === All skippable items ===

#[test]
fn test_all_skippable_items_combined() {
    let src = r#"
        use std::io; const MAX: u32 = 100; static NAME: &str = "rash";
        type Alias = u32; struct S { v: u32 } enum E { A, B }
        trait T { fn go(&self); }
        fn main() { let x = 0; }
    "#;
    let ast = parse(src).unwrap();
    assert_eq!(ast.entry_point, "main");
    assert_eq!(ast.functions.len(), 1);
}

// === Logical operators ===

#[test]
fn test_logical_and_or_operators() {
    let ast = parse(r#"fn main() { let a = true && false; let b = true || false; }"#).unwrap();
    assert!(matches!(
        &ast.functions[0].body[0],
        Stmt::Let {
            value: Expr::Binary {
                op: BinaryOp::And,
                ..
            },
            ..
        }
    ));
    assert!(matches!(
        &ast.functions[0].body[1],
        Stmt::Let {
            value: Expr::Binary {
                op: BinaryOp::Or,
                ..
            },
            ..
        }
    ));
}

// === Tuple with mixed element types ===

#[test]
fn test_tuple_mixed_types() {
    let ast = parse(r#"fn main() { let t = (1, "two", true); }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let {
            value: Expr::Array(elems),
            ..
        } => {
            assert_eq!(elems.len(), 3);
            assert!(matches!(&elems[0], Expr::Literal(Literal::U32(1))));
            assert!(matches!(&elems[2], Expr::Literal(Literal::Bool(true))));
        }
        _ => panic!("Expected Array from tuple"),
    }
}

// === Pat::Path for non-None paths ===

#[test]
fn test_match_path_pattern_non_none() {
    if let Ok(ast) = parse(r#"fn main() { match v { std::io::Error => { let a = 0; } _ => {} } }"#)
    {
        match &ast.functions[0].body[0] {
            Stmt::Match { arms, .. } => assert!(!arms.is_empty()),
            _ => panic!("Expected Match"),
        }
    }
}
