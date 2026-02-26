#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Coverage tests for parser.rs -- round 4.
//! Targets remaining uncovered branches after parser_coverage_tests{,2,3}.rs.

use super::parser::parse;
use crate::ast::restricted::{BinaryOp, Expr, Literal, Pattern, Stmt, Type};

// === MacroArgSplitter: escape sequences, brackets, braces ===

#[test]
fn test_format_string_with_escaped_quotes() {
    let ast = parse(r#"fn main() { println!("she said \"hi\""); }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Expr(Expr::FunctionCall { name, args }) => {
            assert_eq!(name, "rash_println");
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected rash_println"),
    }
}

#[test]
fn test_format_string_with_backslash_before_quote() {
    let ast = parse(r#"fn main() { println!("path\\file"); }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Expr(Expr::FunctionCall { name, args }) => {
            assert_eq!(name, "rash_println");
            assert!(matches!(&args[0], Expr::Literal(Literal::Str(s)) if s.contains('\\')));
        }
        _ => panic!("Expected rash_println with backslash"),
    }
}

#[test]
fn test_split_macro_args_brackets_in_args() {
    let ast = parse(r#"fn main() { println!("arr: {}", arr[0]); }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Expr(Expr::FunctionCall { args, .. }) => match &args[0] {
            Expr::FunctionCall { name, args } => {
                assert_eq!(name, "__format_concat");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected __format_concat"),
        },
        _ => panic!("Expected rash_println"),
    }
}

#[test]
fn test_split_macro_args_braces_in_args() {
    let ast = parse(r#"fn main() { let s = format!("{}", { let x = 1; x }); }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let { value, .. } => {
            assert!(!matches!(value, Expr::Literal(Literal::Str(_))));
        }
        _ => panic!("Expected let with format result"),
    }
}

// === convert_macro_expr_format: single-arg fallback ===

#[test]
fn test_format_macro_single_variable() {
    let ast = parse(r#"fn main() { let s = format!(x); }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let {
            value: Expr::Variable(n),
            ..
        } => assert_eq!(n, "x"),
        _ => panic!("Expected Variable from format!(x)"),
    }
}

// === convert_macro_expr_vec ===

#[test]
fn test_vec_macro_with_expressions() {
    let ast = parse(r#"fn main() { let v = vec![1 + 2, 3 * 4]; }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let {
            value: Expr::Array(elems),
            ..
        } => {
            assert_eq!(elems.len(), 2);
            assert!(matches!(
                &elems[0],
                Expr::Binary {
                    op: BinaryOp::Add,
                    ..
                }
            ));
        }
        _ => panic!("Expected Array from vec! with expressions"),
    }
}

#[test]
fn test_vec_macro_empty() {
    let ast = parse(r#"fn main() { let v = vec![]; }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let {
            value: Expr::Array(elems),
            ..
        } => assert_eq!(elems.len(), 0),
        _ => panic!("Expected empty Array"),
    }
}

// === convert_print_format_args: non-string first arg ===

#[test]
fn test_println_with_variable_as_first_arg() {
    let ast = parse(r#"fn main() { println!(msg); }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Expr(Expr::FunctionCall { name, args }) => {
            assert_eq!(name, "rash_println");
            assert!(matches!(&args[0], Expr::Variable(n) if n == "msg"));
        }
        _ => panic!("Expected rash_println with variable"),
    }
}

// === has_multi_stmt_branch: else-block multi-stmt detection ===

#[test]
fn test_if_expr_else_multi_stmt_produces_block() {
    let ast = parse(r#"fn main() { let x = if c { 1 } else { let a = 2; a }; }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let {
            value: Expr::Block(stmts),
            ..
        } => {
            assert!(matches!(&stmts[0], Stmt::If { .. }));
        }
        _ => panic!("Expected Block for multi-stmt else in if-expr"),
    }
}

// === extract_else_value: nested else-if expression ===

#[test]
fn test_if_expr_nested_else_if() {
    let ast = parse(r#"fn main() { let x = if a { 1 } else if b { 2 } else { 3 }; }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let { value, .. } => {
            assert!(
                matches!(value, Expr::FunctionCall { name, .. } if name == "__if_expr")
                    || matches!(value, Expr::Block(_))
            );
        }
        _ => panic!("Expected if-else-if expression"),
    }
}

// === convert_nested_else: recursive else-if chain (4 levels) ===

#[test]
fn test_nested_else_if_chain_four_levels() {
    let src = r#"fn main() {
        if a { let v = 1; }
        else if b { let v = 2; }
        else if c { let v = 3; }
        else if d { let v = 4; }
        else { let v = 5; }
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
                } => match &e3[0] {
                    Stmt::If {
                        else_block: Some(e4),
                        ..
                    } => {
                        assert!(matches!(&e4[0], Stmt::Let { .. }));
                    }
                    _ => panic!("Expected 4th level else"),
                },
                _ => panic!("Expected 3rd level else-if"),
            },
            _ => panic!("Expected 2nd level else-if"),
        },
        _ => panic!("Expected if-else chain"),
    }
}

// === extract_branch_value: empty block returns empty string ===

#[test]
fn test_if_expr_empty_then_block() {
    let ast = parse(r#"fn main() { let x = if c {} else { 1 }; }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let {
            value: Expr::FunctionCall { name, args },
            ..
        } => {
            assert_eq!(name, "__if_expr");
            assert!(matches!(&args[1], Expr::Literal(Literal::Str(s)) if s.is_empty()));
        }
        _ => panic!("Expected __if_expr with empty then"),
    }
}

// === convert_type: complex unsupported types ===

#[test]
fn test_type_fn_pointer_is_error() {
    let result = parse(r#"#[bashrs::main] fn f(cb: fn(u32) -> u32) { let x = 1; }"#);
    assert!(result.is_err());
}

// === Pattern: TupleStruct with non-standard variant name ===

#[test]
fn test_match_custom_tuple_struct_pattern() {
    let ast = parse(r#"fn main() { match v { Custom(x) => { let a = x; } _ => {} } }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Match { arms, .. } => {
            assert!(matches!(
                &arms[0].pattern,
                Pattern::Literal(Literal::Str(s)) if s == "Custom"
            ));
        }
        _ => panic!("Expected Match with custom tuple struct pattern"),
    }
}

// === extract_pattern_literal: positive range bounds ===

#[test]
fn test_range_pattern_positive_bounds() {
    let ast = parse(r#"fn main() { match x { 1..=10 => { let a = 1; } _ => {} } }"#).unwrap();
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
        _ => panic!("Expected Match"),
    }
}

// === FnArg::Receiver skip path ===

#[test]
fn test_impl_method_self_param_skipped() {
    let src = r#"
        struct S { val: u32 }
        impl S { fn get(&self) -> u32 { let x = self.val; } }
        fn main() { let x = 0; }
    "#;
    let ast = parse(src).unwrap();
    let get_fn = ast.functions.iter().find(|f| f.name == "get").unwrap();
    assert!(get_fn.params.is_empty());
}

#[test]
fn test_impl_method_mut_self_plus_param() {
    let src = r#"
        struct S { val: u32 }
        impl S { fn set(&mut self, v: u32) { self.val = v; } }
        fn main() { let x = 0; }
    "#;
    let ast = parse(src).unwrap();
    let set_fn = ast.functions.iter().find(|f| f.name == "set").unwrap();
    assert_eq!(set_fn.params.len(), 1);
    assert_eq!(set_fn.params[0].name, "v");
    assert!(matches!(set_fn.params[0].param_type, Type::U32));
}

// === Assignment error paths ===

#[test]
fn test_assign_complex_deref_target_error() {
    assert!(parse(r#"fn main() { let mut a = 0; *(a + 1) = 5; }"#).is_err());
}

#[test]
fn test_compound_assign_complex_deref_error() {
    assert!(parse(r#"fn main() { let mut a = 0; *(a + 1) += 1; }"#).is_err());
}

// === Match arm with non-block body ===

#[test]
fn test_match_arm_with_if_body() {
    let ast =
        parse(r#"fn main() { match x { 0 => if true { let a = 1; }, _ => { let b = 2; } } }"#)
            .unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Match { arms, .. } => {
            assert!(matches!(&arms[0].body[0], Stmt::If { .. }));
        }
        _ => panic!("Expected Match"),
    }
}

#[test]
fn test_match_arm_with_return_body() {
    let ast = parse(r#"fn main() { match x { 0 => return 1, _ => return 2 } }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Match { arms, .. } => {
            assert!(matches!(&arms[0].body[0], Stmt::Return(Some(_))));
        }
        _ => panic!("Expected Match with return arms"),
    }
}

// === Nested index read ===

#[test]
fn test_nested_index_read_expr() {
    let ast = parse(r#"fn main() { let arr = [[1]]; let v = arr[0][0]; }"#).unwrap();
    match &ast.functions[0].body[1] {
        Stmt::Let {
            value: Expr::Index { object, .. },
            ..
        } => {
            assert!(matches!(**object, Expr::Index { .. }));
        }
        _ => panic!("Expected nested Index read"),
    }
}

// === eprintln! in match arm (expression form) ===

#[test]
fn test_eprintln_macro_expr_in_match() {
    let ast =
        parse(r#"fn main() { match x { 0 => eprintln!("err: {}", msg), _ => {} } }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Match { arms, .. } => match &arms[0].body[0] {
            Stmt::Expr(Expr::FunctionCall { name, .. }) => {
                assert_eq!(name, "rash_eprintln");
            }
            _ => panic!("Expected rash_eprintln"),
        },
        _ => panic!("Expected Match"),
    }
}

// === Match expression in let position ===

#[test]
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
