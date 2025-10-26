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
fn test_type_conversion_edge_cases() {
    // Test various type syntax that should be converted correctly
    let source = r#"
        #[bashrs::main]
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
fn test_rash_main_attribute_parsing() {
    let source = r#"
        #[bashrs::main]
        fn my_installer() {
            let x = 42;
        }
    "#;

    let ast = parse(source).unwrap();
    assert_eq!(ast.entry_point, "my_installer");
    assert_eq!(ast.functions[0].name, "my_installer");
}

#[test]
fn test_reject_invalid_attributes() {
    // Test that non-bashrs::main attributes don't mark function as main
    let source = r#"
        #[some::other]
        fn not_main() {
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

// ============================================================================
// MUTATION TESTING: Targeted tests for MISSED mutants (Sprint 25 RASH-2501)
// ============================================================================

#[test]
fn test_is_main_attribute_requires_both_conditions() {
    // RED: Targets mutation at line 62: replace && with ||
    // This test ensures BOTH conditions must be true for is_main_attribute

    // Case 1: Only bashrs segment, wrong second segment (should reject)
    let source_wrong_segment = r#"
        #[bashrs::wrong]
        fn not_main() {
            let x = 42;
        }
    "#;
    let result = parse(source_wrong_segment);
    assert!(result.is_err(), "Should reject non-main attribute");

    // Case 2: Wrong first segment, correct second segment (should reject)
    let source_wrong_namespace = r#"
        #[other::main]
        fn also_not_main() {
            let x = 42;
        }
    "#;
    let result2 = parse(source_wrong_namespace);
    assert!(result2.is_err(), "Should reject wrong namespace");

    // Case 3: Both conditions true - bashrs::main (should accept)
    let source_valid = r#"
        #[bashrs::main]
        fn installer() {
            let x = 42;
        }
    "#;
    let result3 = parse(source_valid);
    assert!(result3.is_ok(), "Should accept bashrs::main");
    assert_eq!(result3.unwrap().entry_point, "installer");

    // Case 4: Both conditions true - rash::main (should accept)
    let source_rash = r#"
        #[rash::main]
        fn script() {
            let x = 42;
        }
    "#;
    let result4 = parse(source_rash);
    assert!(result4.is_ok(), "Should accept rash::main");
    assert_eq!(result4.unwrap().entry_point, "script");
}

#[test]
fn test_binary_op_not_equal_conversion() {
    // RED: Targets mutation at line 452: delete match arm BinOp::Ne(_)
    // This test ensures the != operator is properly converted

    let source = r#"
        fn main() {
            let not_equal = x != y;
            let also_ne = a != 42;
        }
    "#;

    let ast = parse(source).unwrap();
    let main_func = &ast.functions[0];

    // First != expression
    match &main_func.body[0] {
        crate::ast::Stmt::Let {
            value: crate::ast::Expr::Binary { op, .. },
            ..
        } => {
            assert!(
                matches!(op, BinaryOp::Ne),
                "Expected Ne (not equal) operator"
            );
        }
        _ => panic!("Expected binary expression with != operator"),
    }

    // Second != expression
    match &main_func.body[1] {
        crate::ast::Stmt::Let {
            value: crate::ast::Expr::Binary { op, .. },
            ..
        } => {
            assert!(
                matches!(op, BinaryOp::Ne),
                "Expected Ne (not equal) operator"
            );
        }
        _ => panic!("Expected binary expression with != operator"),
    }
}

#[test]
fn test_all_binary_operators_converted() {
    // RED: Comprehensive test for all binary operators including Ne
    // Ensures complete coverage of convert_binary_op function

    let source = r#"
        fn main() {
            let add = a + b;
            let sub = a - b;
            let mul = a * b;
            let div = a / b;
            let eq = a == b;
            let ne = a != b;
            let lt = a < b;
            let le = a <= b;
            let gt = a > b;
            let ge = a >= b;
        }
    "#;

    let ast = parse(source).unwrap();
    let main_func = &ast.functions[0];

    // Verify all operators are present
    let expected_ops = vec![
        BinaryOp::Add,
        BinaryOp::Sub,
        BinaryOp::Mul,
        BinaryOp::Div,
        BinaryOp::Eq,
        BinaryOp::Ne, // Critical: ensures Ne branch is tested
        BinaryOp::Lt,
        BinaryOp::Le,
        BinaryOp::Gt,
        BinaryOp::Ge,
    ];

    for (i, expected_op) in expected_ops.iter().enumerate() {
        match &main_func.body[i] {
            crate::ast::Stmt::Let {
                value: crate::ast::Expr::Binary { op, .. },
                ..
            } => {
                assert!(
                    std::mem::discriminant(op) == std::mem::discriminant(expected_op),
                    "Expected {:?} at position {}, got {:?}",
                    expected_op,
                    i,
                    op
                );
            }
            _ => panic!("Expected binary expression at position {}", i),
        }
    }
}

#[test]
fn test_pattern_wildcard_vs_identifier() {
    // RED: Targets mutation at line 567: replace == with !=
    // Tests the condition: if name == "_" for wildcard detection
    // This must distinguish between "_" (wildcard) and named identifiers

    // Test wildcard pattern (_)
    let source_wildcard = r#"
        fn main() {
            match value {
                _ => {
                    let default = true;
                }
            }
        }
    "#;
    let ast = parse(source_wildcard).unwrap();
    let main_func = &ast.functions[0];

    match &main_func.body[0] {
        crate::ast::Stmt::Match { arms, .. } => {
            assert!(!arms.is_empty(), "Should have at least one match arm");
            // Wildcard pattern should be detected
            match &arms[0].pattern {
                Pattern::Wildcard => {
                    // Success: _ was recognized as Wildcard
                }
                _ => panic!("Expected Wildcard pattern for _, got {:?}", arms[0].pattern),
            }
        }
        _ => panic!("Expected match statement"),
    }

    // Test named identifier pattern (not wildcard)
    let source_ident = r#"
        fn main() {
            match value {
                x => {
                    let named = x;
                }
            }
        }
    "#;
    let ast2 = parse(source_ident).unwrap();
    let main_func2 = &ast2.functions[0];

    match &main_func2.body[0] {
        crate::ast::Stmt::Match { arms, .. } => {
            assert!(!arms.is_empty(), "Should have at least one match arm");
            // Named identifier should NOT be Wildcard
            match &arms[0].pattern {
                Pattern::Variable(name) => {
                    assert_eq!(name, "x", "Expected variable pattern 'x'");
                }
                Pattern::Wildcard => {
                    panic!("Named identifier 'x' should NOT be treated as Wildcard");
                }
                _ => panic!("Expected Variable pattern for x, got {:?}", arms[0].pattern),
            }
        }
        _ => panic!("Expected match statement"),
    }
}

#[test]
fn test_pattern_ident_arm_execution() {
    // RED: Targets mutation at line 564: delete match arm Pat::Ident(ident_pat)
    // This ensures the Pat::Ident branch in convert_pattern is exercised

    let source = r#"
        fn main() {
            match status {
                0 => {
                    let success = true;
                }
                code => {
                    let error_code = code;
                }
                _ => {
                    let unknown = true;
                }
            }
        }
    "#;

    let ast = parse(source).unwrap();
    let main_func = &ast.functions[0];

    match &main_func.body[0] {
        crate::ast::Stmt::Match { arms, .. } => {
            assert_eq!(arms.len(), 3, "Should have 3 match arms");

            // First arm: literal pattern (0)
            match &arms[0].pattern {
                Pattern::Literal(Literal::U32(0)) => {
                    // Correct
                }
                _ => panic!("Expected literal pattern 0"),
            }

            // Second arm: identifier pattern (code) - this tests Pat::Ident!
            match &arms[1].pattern {
                Pattern::Variable(name) => {
                    assert_eq!(name, "code", "Expected variable pattern 'code'");
                }
                _ => panic!(
                    "Expected identifier pattern 'code', got {:?}",
                    arms[1].pattern
                ),
            }

            // Third arm: wildcard pattern (_)
            match &arms[2].pattern {
                Pattern::Wildcard => {
                    // Correct
                }
                _ => panic!("Expected wildcard pattern"),
            }
        }
        _ => panic!("Expected match statement"),
    }
}

#[test]
fn test_comprehensive_pattern_matching() {
    // RED: Comprehensive test covering all pattern types
    // Ensures complete coverage of convert_pattern function

    let source = r#"
        fn main() {
            match value {
                42 => { let num = true; }
                "test" => { let str = true; }
                true => { let bool = true; }
                x => { let var = x; }
                _ => { let wild = true; }
            }
        }
    "#;

    let ast = parse(source).unwrap();
    let main_func = &ast.functions[0];

    match &main_func.body[0] {
        crate::ast::Stmt::Match { arms, .. } => {
            assert_eq!(arms.len(), 5, "Should have 5 match arms");

            // Numeric literal
            assert!(
                matches!(&arms[0].pattern, Pattern::Literal(Literal::U32(42))),
                "Expected numeric literal 42"
            );

            // String literal
            assert!(
                matches!(&arms[1].pattern, Pattern::Literal(Literal::Str(_))),
                "Expected string literal"
            );

            // Boolean literal
            assert!(
                matches!(&arms[2].pattern, Pattern::Literal(Literal::Bool(true))),
                "Expected boolean literal true"
            );

            // Variable pattern (identifier)
            match &arms[3].pattern {
                Pattern::Variable(name) => {
                    assert_eq!(name, "x", "Expected variable 'x'");
                }
                _ => panic!("Expected variable pattern"),
            }

            // Wildcard pattern
            assert!(
                matches!(&arms[4].pattern, Pattern::Wildcard),
                "Expected wildcard pattern"
            );
        }
        _ => panic!("Expected match statement"),
    }
}
