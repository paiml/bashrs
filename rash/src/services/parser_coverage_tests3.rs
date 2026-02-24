#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Coverage tests for parser.rs: split_macro_args, parse_format_string,
//! build_format_concat, extract_index_suffix edge cases, convert_field_expr,
//! convert_return_expr, error paths, and type conversions not covered by
//! parser_coverage_tests.rs and parser_coverage_tests2.rs.

use super::parser::parse;
use crate::ast::restricted::{Expr, Literal, Stmt, Type};

// ============================================================================
// Format string parsing: escaped braces, {:fmt} specifiers
// ============================================================================

#[test]
fn test_format_escaped_double_braces() {
    // println!("a {{ b }}") => literal "a { b }" with no placeholders
    let ast = parse(r#"fn main() { println!("a {{ b }}"); }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Expr(Expr::FunctionCall { name, args }) => {
            assert_eq!(name, "rash_println");
            // Single literal arg with literal braces
            assert!(matches!(&args[0], Expr::Literal(Literal::Str(s)) if s.contains('{')));
        }
        _ => panic!("Expected rash_println"),
    }
}

#[test]
fn test_format_with_format_specifier() {
    // println!("{:>10}", x) => format specifier {:>10} treated as placeholder
    // The macro parser uses convert_print_format_args which produces format_concat
    let ast = parse(r#"fn main() { println!("{:>10}", x); }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Expr(Expr::FunctionCall { name, args }) => {
            assert_eq!(name, "rash_println");
            // With {:>10} and one arg, the format concat may collapse to single var
            // since the format spec consumes the only placeholder
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected rash_println"),
    }
}

#[test]
fn test_format_multiple_placeholders() {
    // println!("{} + {} = {}", a, b, c)
    let ast = parse(r#"fn main() { println!("{} + {} = {}", a, b, c); }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Expr(Expr::FunctionCall { name, args }) => {
            assert_eq!(name, "rash_println");
            match &args[0] {
                Expr::FunctionCall { name, args } => {
                    assert_eq!(name, "__format_concat");
                    // " + " and " = " literals plus 3 variables = 5 parts
                    assert!(args.len() >= 5);
                }
                _ => panic!("Expected __format_concat"),
            }
        }
        _ => panic!("Expected rash_println"),
    }
}

#[test]
fn test_format_single_placeholder_no_text() {
    // println!("{}", x) => format_concat with one Variable, collapses to Variable
    let ast = parse(r#"fn main() { println!("{}", x); }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Expr(Expr::FunctionCall { name, args }) => {
            assert_eq!(name, "rash_println");
            // Single placeholder, no surrounding text => collapses to variable
            assert!(matches!(&args[0], Expr::Variable(n) if n == "x"));
        }
        _ => panic!("Expected rash_println with variable"),
    }
}

// ============================================================================
// split_macro_args: nested parens, brackets, braces, strings
// ============================================================================

#[test]
fn test_println_nested_parens_in_args() {
    // println!("result: {}", foo(1, 2))
    let ast = parse(r#"fn main() { println!("result: {}", foo(1, 2)); }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Expr(Expr::FunctionCall { name, args }) => {
            assert_eq!(name, "rash_println");
            // The arg should be __format_concat with a function call
            match &args[0] {
                Expr::FunctionCall { name, args } => {
                    assert_eq!(name, "__format_concat");
                    assert_eq!(args.len(), 2); // "result: " + foo(1,2)
                }
                _ => panic!("Expected __format_concat"),
            }
        }
        _ => panic!("Expected rash_println"),
    }
}

#[test]
fn test_println_string_with_comma_inside() {
    // The comma inside the format string should not split
    let ast = parse(r#"fn main() { println!("hello, world"); }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Expr(Expr::FunctionCall { name, args }) => {
            assert_eq!(name, "rash_println");
            assert!(matches!(&args[0], Expr::Literal(Literal::Str(s)) if s == "hello, world"));
        }
        _ => panic!("Expected rash_println"),
    }
}

// ============================================================================
// format! macro expression
// ============================================================================

#[test]
fn test_format_macro_multi_arg() {
    let ast = parse(r#"fn main() { let s = format!("x={}", v); }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let { value: Expr::FunctionCall { name, .. }, .. } => {
            assert_eq!(name, "__format_concat");
        }
        _ => panic!("Expected format! -> __format_concat"),
    }
}

#[test]
fn test_format_macro_single_arg_is_literal() {
    let ast = parse(r#"fn main() { let s = format!("hello"); }"#).unwrap();
    assert!(matches!(
        &ast.functions[0].body[0],
        Stmt::Let { value: Expr::Literal(Literal::Str(s)), .. } if s == "hello"
    ));
}

// ============================================================================
// extract_index_suffix edge cases: paren, method, unary, call
// ============================================================================

#[test]
fn test_index_with_parenthesized_expr() {
    // arr[(i)] — paren delegates to inner
    let ast = parse(r#"fn main() { let arr = [1]; arr[(0)] = 5; }"#).unwrap();
    match &ast.functions[0].body[1] {
        Stmt::Let { name, declaration, .. } => {
            assert!(!declaration);
            assert_eq!(name, "arr_0");
        }
        _ => panic!("Expected arr_0"),
    }
}

#[test]
fn test_index_with_method_call_suffix() {
    // arr[s.len()] => name contains "s_len"
    let ast = parse(r#"fn main() { let arr = [1, 2]; let s = "hi"; arr[s.len()] = 9; }"#).unwrap();
    match &ast.functions[0].body[2] {
        Stmt::Let { name, declaration, .. } => {
            assert!(!declaration);
            assert!(name.contains("s_len"), "Expected s_len in name, got {}", name);
        }
        _ => panic!("Expected index assignment"),
    }
}

#[test]
fn test_index_with_unary_minus() {
    // arr[-1] is parsed with unary stripped for suffix
    let ast = parse(r#"fn main() { let arr = [1]; let v = arr[-1]; }"#).unwrap();
    match &ast.functions[0].body[1] {
        Stmt::Let { value: Expr::Index { .. }, .. } => {}
        _ => panic!("Expected Index expression"),
    }
}

#[test]
fn test_index_with_function_call() {
    // arr[hash(key)] => extract_call_index_suffix => "hash_key"
    let ast = parse(r#"fn main() { let arr = [1]; arr[hash(key)] = 5; }"#).unwrap();
    match &ast.functions[0].body[1] {
        Stmt::Let { name, .. } => {
            assert!(name.contains("hash"), "Expected hash in name, got {}", name);
            assert!(name.contains("key"), "Expected key in name, got {}", name);
        }
        _ => panic!("Expected index assignment"),
    }
}

#[test]
fn test_index_with_function_call_no_args() {
    // arr[f()] => extract_call_index_suffix with no args => just "f"
    let ast = parse(r#"fn main() { let arr = [1]; arr[f()] = 5; }"#).unwrap();
    match &ast.functions[0].body[1] {
        Stmt::Let { name, .. } => {
            assert!(name.contains("f"), "Expected f in name, got {}", name);
        }
        _ => panic!("Expected index assignment"),
    }
}

// ============================================================================
// convert_field_expr: field access becomes Index
// ============================================================================

#[test]
fn test_field_access_expr() {
    let src = r#"
        struct Point { x: u32, y: u32 }
        fn main() { let p = Point { x: 1, y: 2 }; let v = p.x; }
    "#;
    let ast = parse(src).unwrap();
    // Field access becomes Index with I32(0) index
    match &ast.functions[0].body[1] {
        Stmt::Let { value: Expr::Index { index, .. }, .. } => {
            assert!(matches!(**index, Expr::Literal(Literal::I32(0))));
        }
        _ => panic!("Expected Index from field access"),
    }
}

// ============================================================================
// convert_return_expr: return without value in expr position
// ============================================================================

#[test]
fn test_return_no_value_in_expr_produces_block() {
    // Bare return in closure body is a block expression
    let ast = parse(r#"fn main() { let f = |x| { return; }; }"#).unwrap();
    // Closure body: Block([Return(None)])
    match &ast.functions[0].body[0] {
        Stmt::Let { value: Expr::Block(stmts), .. } => {
            assert!(matches!(&stmts[0], Stmt::Return(None)));
        }
        _ => panic!("Expected Block with Return(None) from closure"),
    }
}

// ============================================================================
// Type conversions: tuple type, array type, complex unsupported
// ============================================================================

#[test]
fn test_type_tuple_param() {
    let ast = parse(r#"#[bashrs::main] fn f(t: (u32, u32)) { let x = t; }"#).unwrap();
    // Tuple type → Str
    assert!(matches!(ast.functions[0].params[0].param_type, Type::Str));
}

#[test]
fn test_type_array_param() {
    let ast = parse(r#"#[bashrs::main] fn f(a: [u32; 3]) { let x = a; }"#).unwrap();
    assert!(matches!(ast.functions[0].params[0].param_type, Type::Str));
}

#[test]
fn test_type_slice_reference() {
    let ast = parse(r#"#[bashrs::main] fn f(s: &[u32]) { let x = s; }"#).unwrap();
    assert!(matches!(ast.functions[0].params[0].param_type, Type::Str));
}

#[test]
fn test_type_unknown_path_defaults_to_str() {
    let ast = parse(r#"#[bashrs::main] fn f(v: Vec<String>) { let x = v; }"#).unwrap();
    assert!(matches!(ast.functions[0].params[0].param_type, Type::Str));
}

// ============================================================================
// Error paths: let without init, complex patterns, unsupported stmt
// ============================================================================

#[test]
fn test_let_tuple_without_init_is_error() {
    assert!(parse(r#"fn main() { let (a, b); }"#).is_err());
}

#[test]
fn test_complex_param_pattern_is_error() {
    assert!(parse(r#"fn main((a, b): (u32, u32)) { let x = a; }"#).is_err());
}

#[test]
fn test_match_as_expression() {
    // match in expression position => Block([Match{...}])
    let ast = parse(r#"fn main() { let x = match v { 0 => 1, _ => 2 }; }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let { value: Expr::Block(stmts), .. } => {
            assert!(matches!(&stmts[0], Stmt::Match { .. }));
        }
        _ => panic!("Expected Block with Match from match expression"),
    }
}

// ============================================================================
// Compound assignments on field and deref targets
// ============================================================================

#[test]
fn test_compound_assign_on_field() {
    let src = r#"
        struct S { v: u32 }
        impl S { fn inc(&mut self) { self.v += 1; } }
        fn main() { let x = 0; }
    "#;
    let ast = parse(src).unwrap();
    let inc_fn = ast.functions.iter().find(|f| f.name == "inc").unwrap();
    match &inc_fn.body[0] {
        Stmt::Let { name, declaration, .. } => {
            assert_eq!(name, "v");
            assert!(!declaration);
        }
        _ => panic!("Expected field compound assignment"),
    }
}

#[test]
fn test_compound_assign_on_index() {
    let ast = parse(r#"fn main() { let mut arr = [1, 2]; arr[0] += 5; }"#).unwrap();
    match &ast.functions[0].body[1] {
        Stmt::Let { name, declaration, .. } => {
            assert_eq!(name, "arr_0");
            assert!(!declaration);
        }
        _ => panic!("Expected arr_0 compound assignment"),
    }
}

#[test]
fn test_compound_assign_on_deref() {
    let ast = parse(r#"fn main() { let mut x = 0; let p = &mut x; *p += 1; }"#).unwrap();
    match &ast.functions[0].body[2] {
        Stmt::Let { name, declaration, .. } => {
            assert_eq!(name, "p");
            assert!(!declaration);
        }
        _ => panic!("Expected deref compound assignment"),
    }
}

// ============================================================================
// convert_let_expr: non-literal pattern falls through
// ============================================================================

#[test]
fn test_let_expr_with_variable_pattern() {
    // if let x = opt { ... } where pattern is not a literal
    let ast = parse(r#"fn main() { let opt = 1; if let x = opt { let a = x; } }"#).unwrap();
    // When pattern is a variable (not a literal), convert_let_expr returns rhs
    assert!(matches!(&ast.functions[0].body[1], Stmt::If { .. }));
}

// ============================================================================
// Unsupported literal type in expression
// ============================================================================

#[test]
fn test_char_literal_is_error() {
    assert!(parse(r#"fn main() { let c = 'a'; }"#).is_err());
}

// ============================================================================
// rash::main attribute alias
// ============================================================================

#[test]
fn test_rash_main_attribute() {
    let ast = parse(r#"#[rash::main] fn my_entry() { let x = 1; }"#).unwrap();
    assert_eq!(ast.entry_point, "my_entry");
}

// ============================================================================
// Field assignment with unnamed fields
// ============================================================================

#[test]
fn test_unnamed_field_assignment_in_compound() {
    // self.0 += 1 via unnamed member
    let src = r#"
        struct Wrapper(u32);
        impl Wrapper { fn bump(&mut self) { self.0 += 1; } }
        fn main() { let x = 0; }
    "#;
    let ast = parse(src).unwrap();
    let bump_fn = ast.functions.iter().find(|f| f.name == "bump").unwrap();
    match &bump_fn.body[0] {
        Stmt::Let { name, .. } => {
            assert!(name.starts_with("field_"), "Expected field_ prefix, got {}", name);
        }
        _ => panic!("Expected unnamed field compound assignment"),
    }
}

// ============================================================================
// Enum items are gracefully skipped
// ============================================================================

#[test]
fn test_enum_item_skipped() {
    let src = r#"enum Color { Red, Green } fn main() { let x = 0; }"#;
    let ast = parse(src).unwrap();
    assert_eq!(ast.entry_point, "main");
}

// ============================================================================
// Closure with no block: expression body
// ============================================================================

#[test]
fn test_closure_expr_body() {
    let ast = parse(r#"fn main() { let f = |x| x * 2; }"#).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Let { value: Expr::Binary { .. }, .. } => {}
        _ => panic!("Expected Binary from closure expression body"),
    }
}

// ============================================================================
// Index expression in expr position (read, not assign)
// ============================================================================

#[test]
fn test_index_read_expr() {
    let ast = parse(r#"fn main() { let arr = [10, 20]; let v = arr[1]; }"#).unwrap();
    match &ast.functions[0].body[1] {
        Stmt::Let { value: Expr::Index { object, index }, .. } => {
            assert!(matches!(**object, Expr::Variable(ref n) if n == "arr"));
            assert!(matches!(**index, Expr::Literal(Literal::U32(1))));
        }
        _ => panic!("Expected Index read"),
    }
}

// ============================================================================
// Negative range pattern
// ============================================================================

#[test]
fn test_negative_range_pattern() {
    let src = r#"fn main() { match x { -10..=-1 => { let a = 1; } _ => {} } }"#;
    let ast = parse(src).unwrap();
    match &ast.functions[0].body[0] {
        Stmt::Match { arms, .. } => {
            match &arms[0].pattern {
                crate::ast::restricted::Pattern::Range { start, end, inclusive } => {
                    assert!(matches!(start, Literal::I32(-10)));
                    assert!(matches!(end, Literal::I32(-1)));
                    assert!(*inclusive);
                }
                _ => panic!("Expected Range pattern"),
            }
        }
        _ => panic!("Expected Match"),
    }
}
