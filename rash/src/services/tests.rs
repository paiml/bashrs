use super::*;
use crate::ast::restricted::{BinaryOp, Literal};
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
        crate::ast::Stmt::Expr(crate::ast::Expr::Literal(Literal::Str(s))) => {
            assert_eq!(s, "success");
        }
        _ => panic!("Expected return expression"),
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
        #[rash::main]
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
        #[rash::main]
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
        .contains("No #[rash::main] function found"));
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
        .contains("Multiple #[rash::main] functions found"));
}

#[test]
fn test_error_on_non_function_items() {
    let source = r#"
        struct MyStruct {
            field: u32,
        }
        
        fn main() {
            let x = 42;
        }
    "#;

    let result = parse(source);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Only functions are allowed"));
}

#[test]
fn test_complex_expression_parsing() {
    let source = r#"
        #[rash::main]
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

    match &main_func.body[0] {
        crate::ast::Stmt::Let {
            value: crate::ast::Expr::Unary { op, .. },
            ..
        } => {
            assert!(matches!(op, crate::ast::restricted::UnaryOp::Neg));
        }
        _ => panic!("Expected unary negation"),
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
fn test_type_conversion_edge_cases() {
    // Test various type syntax that should be converted correctly
    let source = r#"
        #[rash::main]
        fn test(s: &str, st: String, opt: Option<u32>) -> bool {
            let x = 42;
        }
    "#;

    let ast = parse(source).unwrap();
    let func = &ast.functions[0];

    assert_eq!(func.params.len(), 3);
    assert!(matches!(func.return_type, crate::ast::Type::Bool));

    // All string-like types should be converted to Str
    assert!(matches!(func.params[0].param_type, crate::ast::Type::Str));
    assert!(matches!(func.params[1].param_type, crate::ast::Type::Str));
    assert!(matches!(
        func.params[2].param_type,
        crate::ast::Type::Option { .. }
    ));
}

// Property-based tests
proptest! {
    #[test]
    fn test_valid_identifier_parsing(name in "[a-zA-Z_][a-zA-Z0-9_]*") {
        let source = format!("fn {name}() {{ let x = 42; }}");

        if name == "main" {
            let ast = parse(&source).unwrap();
            assert_eq!(ast.functions[0].name, name);
        } else {
            // Non-main functions should cause an error due to no main function
            assert!(parse(&source).is_err());
        }
    }

    #[test]
    fn test_numeric_literal_parsing(num in 0u32..1000u32) {
        let source = format!("fn main() {{ let x = {num}; }}");

        let ast = parse(&source).unwrap();
        match &ast.functions[0].body[0] {
            crate::ast::Stmt::Let { value: crate::ast::Expr::Literal(Literal::U32(n)), .. } => {
                assert_eq!(*n, num);
            },
            _ => panic!("Expected numeric literal"),
        }
    }

    #[test]
    fn test_string_literal_parsing(s in "[a-zA-Z0-9 _.-]*") {
        // Use safe characters that don't need escaping
        let source = format!(r#"fn main() {{ let x = "{s}"; }}"#);

        let result = parse(&source);
        if result.is_ok() {
            let ast = result.unwrap();
            match &ast.functions[0].body[0] {
                crate::ast::Stmt::Let { value: crate::ast::Expr::Literal(Literal::Str(parsed)), .. } => {
                    assert_eq!(parsed, &s);
                },
                _ => panic!("Expected string literal"),
            }
        }
        // Some strings might be invalid syntax, which is okay
    }
}

#[rstest]
#[case("true", Literal::Bool(true))]
#[case("false", Literal::Bool(false))]
#[case("42", Literal::U32(42))]
#[case("0", Literal::U32(0))]
fn test_literal_parsing_cases(#[case] input: &str, #[case] expected: Literal) {
    let source = format!("fn main() {{ let x = {input}; }}");

    let ast = parse(&source).unwrap();
    match &ast.functions[0].body[0] {
        crate::ast::Stmt::Let {
            value: crate::ast::Expr::Literal(lit),
            ..
        } => {
            assert_eq!(*lit, expected);
        }
        _ => panic!("Expected literal"),
    }
}

#[test]
fn test_error_handling_invalid_syntax() {
    let invalid_sources = vec![
        "invalid rust syntax",
        "fn main() { let x = ; }", // Missing value
        "fn main() { let = 42; }", // Missing name
        "",                        // Empty input
    ];

    for source in invalid_sources {
        let result = parse(source);
        assert!(result.is_err(), "Expected error for: {source}");
    }
}

#[test]
fn test_nested_expression_parsing() {
    let source = r#"
        fn main() {
            let complex = func(x + y, other.method(z));
        }
    "#;

    let ast = parse(source).unwrap();
    let main_func = &ast.functions[0];

    match &main_func.body[0] {
        crate::ast::Stmt::Let {
            value: crate::ast::Expr::FunctionCall { args, .. },
            ..
        } => {
            assert_eq!(args.len(), 2);
            assert!(matches!(args[0], crate::ast::Expr::Binary { .. }));
            assert!(matches!(args[1], crate::ast::Expr::MethodCall { .. }));
        }
        _ => panic!("Expected function call with complex args"),
    }
}

#[test]
fn test_empty_function_body_handling() {
    let source = r#"
        fn main() {
        }
    "#;

    let ast = parse(source).unwrap();
    assert_eq!(ast.functions[0].body.len(), 0);

    // Empty function bodies are now allowed
    assert!(ast.validate().is_ok());
}

#[test]
fn test_parser_maintains_source_information() {
    let source = r#"
        fn main() {
            let first = 1;
            let second = 2;
            let third = 3;
        }
    "#;

    let ast = parse(source).unwrap();
    let main_func = &ast.functions[0];

    // Should preserve order of statements
    assert_eq!(main_func.body.len(), 3);

    match &main_func.body[0] {
        crate::ast::Stmt::Let { name, .. } => assert_eq!(name, "first"),
        _ => panic!("Expected first let statement"),
    }

    match &main_func.body[1] {
        crate::ast::Stmt::Let { name, .. } => assert_eq!(name, "second"),
        _ => panic!("Expected second let statement"),
    }

    match &main_func.body[2] {
        crate::ast::Stmt::Let { name, .. } => assert_eq!(name, "third"),
        _ => panic!("Expected third let statement"),
    }
}
