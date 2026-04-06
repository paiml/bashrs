#![allow(clippy::expect_used)]
use super::*;
use crate::ast::restricted::{BinaryOp, Literal, Pattern};
use proptest::prelude::*;
use rstest::*;

#[test]
fn test_simple_function_parsing() {
    let source = r#"
        fn main() {
            let x = 42;
        }
    "#;

    let ast = parse(source).unwrap();
    assert_eq!(ast.entry_point, "main");
    assert_eq!(ast.functions.len(), 1);
    assert_eq!(ast.functions[0].name, "main");
    assert_eq!(ast.functions[0].body.len(), 1);
}

#[test]
fn test_multiple_functions_parsing() {
    let source = r#"
        fn helper() {
            let y = 10;
        }
        
        fn main() {
            let x = 42;
            helper();
        }
    "#;

    let ast = parse(source).unwrap();
    assert_eq!(ast.entry_point, "main");
    assert_eq!(ast.functions.len(), 2);

    let main_func = ast.functions.iter().find(|f| f.name == "main").unwrap();
    let helper_func = ast.functions.iter().find(|f| f.name == "helper").unwrap();

    assert_eq!(main_func.body.len(), 2);
    assert_eq!(helper_func.body.len(), 1);
}

#[test]
fn test_literal_parsing() {
    let source = r#"
        fn main() {
            let bool_val = true;
            let num_val = 123;
            let str_val = "hello world";
        }
    "#;

    let ast = parse(source).unwrap();
    let main_func = &ast.functions[0];

    match &main_func.body[0] {
        crate::ast::Stmt::Let {
            value: crate::ast::Expr::Literal(Literal::Bool(true)),
            ..
        } => {}
        _ => panic!("Expected boolean literal"),
    }

    match &main_func.body[1] {
        crate::ast::Stmt::Let {
            value: crate::ast::Expr::Literal(Literal::U32(123)),
            ..
        } => {}
        _ => panic!("Expected numeric literal"),
    }

    match &main_func.body[2] {
        crate::ast::Stmt::Let {
            value: crate::ast::Expr::Literal(Literal::Str(s)),
            ..
        } => {
            assert_eq!(s, "hello world");
        }
        _ => panic!("Expected string literal"),
    }
}

#[test]
fn test_function_call_parsing() {
    let source = r#"
        fn main() {
            helper();
            echo("hello", "world");
        }
    "#;

    let ast = parse(source).unwrap();
    let main_func = &ast.functions[0];

    match &main_func.body[0] {
        crate::ast::Stmt::Expr(crate::ast::Expr::FunctionCall { name, args }) => {
            assert_eq!(name, "helper");
            assert_eq!(args.len(), 0);
        }
        _ => panic!("Expected function call"),
    }

    match &main_func.body[1] {
        crate::ast::Stmt::Expr(crate::ast::Expr::FunctionCall { name, args }) => {
            assert_eq!(name, "echo");
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected function call with args"),
    }
}

#[test]
fn test_binary_expression_parsing() {
    let source = r#"
        fn main() {
            let result = 1 + 2;
            let comparison = x == y;
        }
    "#;

    let ast = parse(source).unwrap();
    let main_func = &ast.functions[0];

    match &main_func.body[0] {
        crate::ast::Stmt::Let {
            value: crate::ast::Expr::Binary { op, .. },
            ..
        } => {
            assert!(matches!(op, BinaryOp::Add));
        }
        _ => panic!("Expected binary expression"),
    }

    match &main_func.body[1] {
        crate::ast::Stmt::Let {
            value: crate::ast::Expr::Binary { op, .. },
            ..
        } => {
            assert!(matches!(op, BinaryOp::Eq));
        }
        _ => panic!("Expected comparison expression"),
    }
}

#[test]
fn test_method_call_parsing() {
    let source = r#"
        fn main() {
            let result = obj.method();
            let chained = obj.method(arg).another();
        }
    "#;

    let ast = parse(source).unwrap();
    let main_func = &ast.functions[0];

    match &main_func.body[0] {
        crate::ast::Stmt::Let {
            value:
                crate::ast::Expr::MethodCall {
                    receiver,
                    method,
                    args,
                },
            ..
        } => {
            assert_eq!(method, "method");
            assert_eq!(args.len(), 0);
            assert!(matches!(**receiver, crate::ast::Expr::Variable(_)));
        }
        _ => panic!("Expected method call"),
    }
}

#[test]
fn test_return_statement_parsing() {
    let source = r#"
        fn main() {
            return "success";
        }
    "#;

    let ast = parse(source).unwrap();
    let main_func = &ast.functions[0];

    match &main_func.body[0] {
        crate::ast::Stmt::Return(Some(crate::ast::Expr::Literal(Literal::Str(s)))) => {
            assert_eq!(s, "success");
        }
        _ => panic!(
            "Expected Stmt::Return with string literal, got {:?}",
            main_func.body[0]
        ),
    }
}

#[test]
fn test_variable_reference_parsing() {
    let source = r#"
        fn main() {
            let x = 42;
            let y = x;
        }
    "#;

    let ast = parse(source).unwrap();
    let main_func = &ast.functions[0];

    match &main_func.body[1] {
        crate::ast::Stmt::Let {
            value: crate::ast::Expr::Variable(name),
            ..
        } => {
            assert_eq!(name, "x");
        }
        _ => panic!("Expected variable reference"),
    }
}

#[test]
fn test_parameter_parsing() {
    let source = r#"
        #[bashrs::main]
        fn greet(name: &str, age: u32) {
            let message = "hello";
        }
    "#;

    let ast = parse(source).unwrap();
    let func = &ast.functions[0];

    assert_eq!(func.params.len(), 2);
    assert_eq!(func.params[0].name, "name");
    assert_eq!(func.params[1].name, "age");
}

#[test]
fn test_return_type_parsing() {
    let source = r#"
        #[bashrs::main]
        fn get_number() -> u32 {
            let x = 42;
        }
    "#;

    let ast = parse(source).unwrap();

    let get_number = ast
        .functions
        .iter()
        .find(|f| f.name == "get_number")
        .unwrap();
    assert!(matches!(get_number.return_type, crate::ast::Type::U32));
}

#[test]
fn test_error_on_no_main_function() {
    let source = r#"
        fn helper() {
            let x = 42;
        }
    "#;

    let result = parse(source);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("No #[bashrs::main] function found"));
}

#[test]
fn test_error_on_multiple_main_functions() {
    let source = r#"
        fn main() {
            let x = 42;
        }
        
        fn main() {
            let y = 24;
        }
    "#;

    let result = parse(source);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Multiple #[bashrs::main] functions found"));
}

#[test]
fn test_non_function_items_skipped() {
    // Non-function items (struct, enum, impl, etc.) are now gracefully skipped
    let source = r#"
        struct MyStruct {
            field: u32,
        }

        fn main() {
            let x = 42;
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Non-function items should be gracefully skipped: {:?}",
        result.err()
    );
    let ast = result.expect("parse should succeed");
    assert_eq!(ast.functions.len(), 1);
    assert_eq!(ast.entry_point, "main");
}

#[test]
fn test_complex_expression_parsing() {
    let source = r#"
        #[bashrs::main]
        fn main() {
            let result = (x + y) * (a - b);
            let nested = call(other(value));
        }
    "#;

    let ast = parse(source).unwrap();
    let main_func = &ast.functions[0];

    // Should parse complex expressions without errors
    assert_eq!(main_func.body.len(), 2);

    // Verify the structure is reasonable
    match &main_func.body[0] {
        crate::ast::Stmt::Let { value, .. } => {
            assert!(matches!(value, crate::ast::Expr::Binary { .. }));
        }
        _ => panic!("Expected let statement with binary expression"),
    }
}

#[test]
fn test_unary_expression_parsing() {
    let source = r#"
        fn main() {
            let negated = -42;
            let inverted = !true;
        }
    "#;

    let ast = parse(source).unwrap();
    let main_func = &ast.functions[0];

    // After TICKET-5003 fix: -42 is simplified to Literal::I32(-42), not Unary
    match &main_func.body[0] {
        crate::ast::Stmt::Let {
            value: crate::ast::Expr::Literal(crate::ast::restricted::Literal::I32(n)),
            ..
        } => {
            assert_eq!(*n, -42, "Negative literal should be -42");
        }
        _ => panic!("Expected negative integer literal (simplified from unary)"),
    }

    match &main_func.body[1] {
        crate::ast::Stmt::Let {
            value: crate::ast::Expr::Unary { op, .. },
            ..
        } => {
            assert!(matches!(op, crate::ast::restricted::UnaryOp::Not));
        }
        _ => panic!("Expected unary not"),
    }
}

#[test]

include!("tests_extracted_parse.rs");
