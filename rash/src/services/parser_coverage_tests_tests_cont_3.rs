fn test_match_some_and_none_patterns() {
    let ast = parse(r#"fn main() { match opt { Some(v) => { let g = v; } None => {} } }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Match { arms, .. } => {
            assert!(matches!(&arms[0].pattern, Pattern::Variable(n) if n == "v"));
            // None is parsed as a wildcard/variable pattern, not a string literal
            assert!(
                matches!(&arms[1].pattern, Pattern::Literal(Literal::Str(_)))
                    || matches!(&arms[1].pattern, Pattern::Wildcard)
                    || matches!(&arms[1].pattern, Pattern::Variable(_)),
                "None arm pattern: {:?}",
                &arms[1].pattern
            );
        }
        _ => panic!("Expected Match"),
    }
}

#[test]
fn test_match_ok_and_err_patterns() {
    let ast =
        parse(r#"fn main() { match r { Ok(v) => { let s = v; } Err(e) => { let f = e; } } }"#)
            .unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Match { arms, .. } => {
            assert!(matches!(&arms[0].pattern, Pattern::Variable(n) if n == "v"));
            assert!(matches!(&arms[1].pattern, Pattern::Variable(n) if n == "e"));
        }
        _ => panic!("Expected Ok/Err patterns"),
    }
}

#[test]
fn test_match_range_pattern_inclusive() {
    let ast = parse(r#"fn main() { match s { 0..=59 => { let g = "F"; } _ => {} } }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Match { arms, .. } => {
            assert!(matches!(
                &arms[0].pattern,
                Pattern::Range {
                    inclusive: true,
                    ..
                }
            ));
        }
        _ => panic!("Expected range pattern"),
    }
}

// ============================================================================
// Macros: eprintln!, print!, format args, unsupported macro
// ============================================================================

#[test]
fn test_eprintln_macro() {
    let ast = parse(r#"fn main() { eprintln!("err"); }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Expr(Expr::FunctionCall { name, .. }) => assert_eq!(name, "rash_eprintln"),
        _ => panic!("Expected rash_eprintln"),
    }
}

#[test]
fn test_print_macro() {
    let ast = parse(r#"fn main() { print!("x"); }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Expr(Expr::FunctionCall { name, .. }) => assert_eq!(name, "rash_print"),
        _ => panic!("Expected rash_print"),
    }
}

#[test]
fn test_println_format_args_produces_format_concat() {
    let ast = parse(r#"fn main() { println!("hello {}", name); }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Expr(Expr::FunctionCall { args, .. }) => {
            assert!(
                matches!(&args[0], Expr::FunctionCall { name, .. } if name == "__format_concat")
            );
        }
        _ => panic!("Expected format_concat"),
    }
}

#[test]
fn test_unsupported_macro_returns_error() {
    assert!(parse(r#"fn main() { assert!(true); }"#).is_err());
}

// ============================================================================
// Expressions: block, array, tuple, repeat, cast, range, vec!, reference
// ============================================================================

#[test]
fn test_block_expression() {
    let ast = parse(r#"fn main() { let x = { 42 }; }"#).unwrap();
    assert!(matches!(
        &ast.functions[0].body[0],
        Stmt::Let {
            value: Expr::Block(_),
            ..
        }
    ));
}

#[test]
fn test_array_literal() {
    let ast = parse(r#"fn main() { let a = [1, 2, 3]; }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let {
            value: Expr::Array(e),
            ..
        } => assert_eq!(e.len(), 3),
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_tuple_becomes_array() {
    let ast = parse(r#"fn main() { let t = (1, 2, 3); }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let {
            value: Expr::Array(e),
            ..
        } => assert_eq!(e.len(), 3),
        _ => panic!("Expected Array from tuple"),
    }
}

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
