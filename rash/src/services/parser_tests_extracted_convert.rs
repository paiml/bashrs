
    #[test]
    fn test_convert_stmt_else_if_with_final_else() {
        let source = r#"
            fn main() {
                if true {
                    let x = 1;
                } else if false {
                    let y = 2;
                } else {
                    let z = 3;
                }
            }
        "#;
        let ast = parse(source).unwrap();

        match &ast.functions[0].body[0] {
            Stmt::If { else_block, .. } => {
                let first_else = else_block.as_ref().unwrap();

                // Second level should be else-if with an else block
                match &first_else[0] {
                    Stmt::If {
                        else_block: second_else_block,
                        ..
                    } => {
                        assert!(second_else_block.is_some());
                        let final_else = second_else_block.as_ref().unwrap();
                        assert_eq!(final_else.len(), 1);

                        // Final else should contain a Let statement, not another If
                        assert!(matches!(final_else[0], Stmt::Let { .. }));
                    }
                    _ => panic!("Expected second-level If statement"),
                }
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_convert_stmt_expr_call() {
        let source = r#"
            fn main() {
                echo("test");
            }
        "#;
        let ast = parse(source).unwrap();

        match &ast.functions[0].body[0] {
            Stmt::Expr(expr) => {
                assert!(matches!(expr, Expr::FunctionCall { .. }));
            }
            _ => panic!("Expected Expr statement"),
        }
    }

    #[test]
    fn test_convert_stmt_loop_supported() {
        // Loop is now supported: `loop { }` converts to `while true { }`
        let source = r#"
            fn main() {
                loop { }
            }
        "#;
        let result = parse(source);
        assert!(
            result.is_ok(),
            "loop {{}} should be supported (converts to while true): {:?}",
            result.err()
        );
    }

    // Tests for convert_expr function
    #[test]
    fn test_convert_expr_literal_bool() {
        let source = r#"
            fn main() {
                let x = true;
            }
        "#;
        let ast = parse(source).unwrap();
        match &ast.functions[0].body[0] {
            Stmt::Let { value, .. } => {
                assert!(matches!(value, Expr::Literal(Literal::Bool(true))));
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_convert_expr_literal_int() {
        let source = r#"
            fn main() {
                let x = 42;
            }
        "#;
        let ast = parse(source).unwrap();
        match &ast.functions[0].body[0] {
            Stmt::Let { value, .. } => {
                assert!(matches!(value, Expr::Literal(Literal::U32(42))));
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_convert_expr_variable() {
        let source = r#"
            fn main() {
                let x = 42;
                let y = x;
            }
        "#;
        let ast = parse(source).unwrap();
        match &ast.functions[0].body[1] {
            Stmt::Let { value, .. } => match value {
                Expr::Variable(name) => assert_eq!(name, "x"),
                _ => panic!("Expected Variable expression"),
            },
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_convert_expr_function_call() {
        let source = r#"
            fn main() {
                echo("test");
            }
        "#;
        let ast = parse(source).unwrap();
        match &ast.functions[0].body[0] {
            Stmt::Expr(expr) => match expr {
                Expr::FunctionCall { name, args } => {
                    assert_eq!(name, "echo");
                    assert_eq!(args.len(), 1);
                }
                _ => panic!("Expected FunctionCall expression"),
            },
            _ => panic!("Expected Expr statement"),
        }
    }

    #[test]
    fn test_convert_expr_binary_op() {
        let source = r#"
            fn main() {
                let x = 1 + 2;
            }
        "#;
        let ast = parse(source).unwrap();
        match &ast.functions[0].body[0] {
            Stmt::Let { value, .. } => {
                assert!(matches!(value, Expr::Binary { .. }));
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_convert_expr_unary_op() {
        let source = r#"
            fn main() {
                let x = !true;
            }
        "#;
        let ast = parse(source).unwrap();
        match &ast.functions[0].body[0] {
            Stmt::Let { value, .. } => {
                assert!(matches!(value, Expr::Unary { .. }));
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_convert_expr_parenthesized() {
        let source = r#"
            fn main() {
                let x = (42);
            }
        "#;
        let ast = parse(source).unwrap();
        match &ast.functions[0].body[0] {
            Stmt::Let { value, .. } => {
                // Parentheses should be unwrapped
                assert!(matches!(value, Expr::Literal(Literal::U32(42))));
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_convert_expr_nested_binary() {
        let source = r#"
            fn main() {
                let x = 1 + 2 * 3;
            }
        "#;
        let ast = parse(source).unwrap();
        match &ast.functions[0].body[0] {
            Stmt::Let { value, .. } => match value {
                Expr::Binary { left, right, .. } => {
                    assert!(matches!(**left, Expr::Literal(Literal::U32(1))));
                    assert!(matches!(**right, Expr::Binary { .. }));
                }
                _ => panic!("Expected Binary expression"),
            },
            _ => panic!("Expected Let statement"),
        }
    }

    // Tests for parse function entry point
    #[test]
    fn test_parse_simple_main() {
        let source = r#"
            fn main() {
                let x = 42;
            }
        "#;
        let ast = parse(source).unwrap();
        assert_eq!(ast.entry_point, "main");
        assert_eq!(ast.functions.len(), 1);
        assert_eq!(ast.functions[0].name, "main");
    }

    #[test]
    fn test_parse_with_bashrs_main_attribute() {
        let source = r#"
            #[bashrs::main]
            fn custom_entry() {
                let x = 1;
            }
        "#;
        let ast = parse(source).unwrap();
        assert_eq!(ast.entry_point, "custom_entry");
        assert_eq!(ast.functions[0].name, "custom_entry");
    }

    #[test]
    fn test_parse_multiple_functions() {
        let source = r#"
            fn main() {
                helper();
            }

            fn helper() {
                let x = 1;
            }
        "#;
        let ast = parse(source).unwrap();
        assert_eq!(ast.entry_point, "main");
        assert_eq!(ast.functions.len(), 2);
        assert_eq!(ast.functions[0].name, "main");
        assert_eq!(ast.functions[1].name, "helper");
    }

    #[test]
    fn test_parse_no_main_function_error() {
        let source = r#"
            fn helper() {
                let x = 1;
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
    fn test_parse_multiple_main_functions_error() {
        let source = r#"
            fn main() {
                let x = 1;
            }

            #[bashrs::main]
            fn another_main() {
                let y = 2;
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
    fn test_parse_non_function_item_skipped() {
        // Non-function items (const, struct, enum, etc.) are now gracefully skipped
        let source = r#"
            const X: u32 = 42;

            fn main() {
                let x = 1;
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
    fn test_parse_legacy_rash_main_attribute() {
        let source = r#"
            #[rash::main]
            fn entry() {
                let x = 1;
            }
        "#;
        let ast = parse(source).unwrap();
        assert_eq!(ast.entry_point, "entry");
    }
