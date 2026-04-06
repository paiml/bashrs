#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_stmt_simple_let_binding() {
        let source = r#"
            fn main() {
                let x = 42;
            }
        "#;
        let ast = parse(source).unwrap();
        assert_eq!(ast.functions.len(), 1);
        assert_eq!(ast.functions[0].body.len(), 1);

        match &ast.functions[0].body[0] {
            Stmt::Let { name, value, .. } => {
                assert_eq!(name, "x");
                assert!(matches!(value, Expr::Literal(Literal::U32(42))));
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_convert_stmt_string_let_binding() {
        let source = r#"
            fn main() {
                let greeting = "Hello, world!";
            }
        "#;
        let ast = parse(source).unwrap();

        match &ast.functions[0].body[0] {
            Stmt::Let { name, value, .. } => {
                assert_eq!(name, "greeting");
                assert!(matches!(value, Expr::Literal(Literal::Str(_))));
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_convert_stmt_let_without_init() {
        let source = r#"
            fn main() {
                let x;
            }
        "#;
        let result = parse(source);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must have initializers"));
    }

    #[test]
    fn test_convert_stmt_simple_if() {
        let source = r#"
            fn main() {
                if true {
                    let x = 1;
                }
            }
        "#;
        let ast = parse(source).unwrap();

        match &ast.functions[0].body[0] {
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
                assert!(matches!(condition, Expr::Literal(Literal::Bool(true))));
                assert_eq!(then_block.len(), 1);
                assert!(else_block.is_none());
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_convert_stmt_if_else() {
        let source = r#"
            fn main() {
                if true {
                    let x = 1;
                } else {
                    let y = 2;
                }
            }
        "#;
        let ast = parse(source).unwrap();

        match &ast.functions[0].body[0] {
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
                assert!(matches!(condition, Expr::Literal(Literal::Bool(true))));
                assert_eq!(then_block.len(), 1);
                assert!(else_block.is_some());
                assert_eq!(else_block.as_ref().unwrap().len(), 1);
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_convert_stmt_else_if_chain_two_levels() {
        let source = r#"
            fn main() {
                if true {
                    let x = 1;
                } else if false {
                    let y = 2;
                }
            }
        "#;
        let ast = parse(source).unwrap();

        match &ast.functions[0].body[0] {
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
                assert!(matches!(condition, Expr::Literal(Literal::Bool(true))));
                assert_eq!(then_block.len(), 1);

                // Verify else block contains nested if
                assert!(else_block.is_some());
                let else_stmts = else_block.as_ref().unwrap();
                assert_eq!(else_stmts.len(), 1);

                match &else_stmts[0] {
                    Stmt::If {
                        condition: nested_cond,
                        then_block: nested_then,
                        else_block: nested_else,
                    } => {
                        assert!(matches!(nested_cond, Expr::Literal(Literal::Bool(false))));
                        assert_eq!(nested_then.len(), 1);
                        assert!(nested_else.is_none());
                    }
                    _ => panic!("Expected nested If statement in else block"),
                }
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_convert_stmt_else_if_chain_three_levels() {
        let source = r#"
            fn main() {
                if true {
                    let x = 1;
                } else if false {
                    let y = 2;
                } else if true {
                    let z = 3;
                }
            }
        "#;
        let ast = parse(source).unwrap();

        // Verify first level if
        let first_else = extract_if_else_block(&ast.functions[0].body[0]);
        assert_eq!(first_else.len(), 1);

        // Verify second level else-if
        let second_else = extract_if_else_block(&first_else[0]);
        assert_eq!(second_else.len(), 1);

        // Verify third level else-if
        match &second_else[0] {
            Stmt::If {
                condition,
                else_block: third_else,
                ..
            } => {
                assert!(matches!(condition, Expr::Literal(Literal::Bool(true))));
                assert!(third_else.is_none());
            }
            _ => panic!("Expected third-level If statement"),
        }
    }

    /// Helper: extract the else block from an If statement, panicking if not found
    fn extract_if_else_block(stmt: &Stmt) -> &Vec<Stmt> {
        match stmt {
            Stmt::If { else_block, .. } => {
                assert!(else_block.is_some(), "Expected else block");
                else_block.as_ref().unwrap()
            }
            _ => panic!("Expected If statement"),
        }
    }

}

include!("parser_tests_extracted_convert.rs");
