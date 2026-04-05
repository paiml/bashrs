
    #[test]
    fn test_stmt_let_empty_name() {
        let stmt = Stmt::Let {
            name: "".to_string(),
            value: Expr::Literal(Literal::U32(1)),
            declaration: true,
        };
        let result = stmt.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_stmt_for_without_max_iterations() {
        let stmt = Stmt::For {
            pattern: Pattern::Variable("i".to_string()),
            iter: Expr::Range {
                start: Box::new(Expr::Literal(Literal::U32(0))),
                end: Box::new(Expr::Literal(Literal::U32(10))),
                inclusive: false,
            },
            body: vec![],
            max_iterations: None,
        };
        let result = stmt.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("bounded iterations"));
    }

    #[test]
    fn test_stmt_while_without_max_iterations() {
        let stmt = Stmt::While {
            condition: Expr::Literal(Literal::Bool(true)),
            body: vec![],
            max_iterations: None,
        };
        let result = stmt.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("bounded iterations"));
    }

    #[test]
    fn test_stmt_break_continue_validate() {
        assert!(Stmt::Break.validate().is_ok());
        assert!(Stmt::Continue.validate().is_ok());
    }

    #[test]
    fn test_stmt_return_none_validates() {
        assert!(Stmt::Return(None).validate().is_ok());
    }

    #[test]
    fn test_stmt_if_validation() {
        let stmt = Stmt::If {
            condition: Expr::Variable("x".to_string()),
            then_block: vec![Stmt::Return(None)],
            else_block: Some(vec![Stmt::Break]),
        };
        assert!(stmt.validate().is_ok());
    }

    #[test]
    fn test_stmt_match_validation() {
        let stmt = Stmt::Match {
            scrutinee: Expr::Variable("x".to_string()),
            arms: vec![MatchArm {
                pattern: Pattern::Wildcard,
                guard: Some(Expr::Literal(Literal::Bool(true))),
                body: vec![Stmt::Return(None)],
            }],
        };
        assert!(stmt.validate().is_ok());
    }

    #[test]
    fn test_stmt_collect_calls_if() {
        let stmt = Stmt::If {
            condition: Expr::FunctionCall {
                name: "cond".to_string(),
                args: vec![],
            },
            then_block: vec![Stmt::Expr(Expr::FunctionCall {
                name: "then_fn".to_string(),
                args: vec![],
            })],
            else_block: Some(vec![Stmt::Expr(Expr::FunctionCall {
                name: "else_fn".to_string(),
                args: vec![],
            })]),
        };
        let mut calls = vec![];
        stmt.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["cond", "then_fn", "else_fn"]);
    }

    #[test]
    fn test_stmt_collect_calls_match() {
        let stmt = Stmt::Match {
            scrutinee: Expr::FunctionCall {
                name: "scrut".to_string(),
                args: vec![],
            },
            arms: vec![MatchArm {
                pattern: Pattern::Wildcard,
                guard: Some(Expr::FunctionCall {
                    name: "guard".to_string(),
                    args: vec![],
                }),
                body: vec![Stmt::Expr(Expr::FunctionCall {
                    name: "body".to_string(),
                    args: vec![],
                })],
            }],
        };
        let mut calls = vec![];
        stmt.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["scrut", "guard", "body"]);
    }

    #[test]
    fn test_stmt_collect_calls_for_while() {
        let for_stmt = Stmt::For {
            pattern: Pattern::Variable("i".to_string()),
            iter: Expr::FunctionCall {
                name: "iter".to_string(),
                args: vec![],
            },
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "loop_fn".to_string(),
                args: vec![],
            })],
            max_iterations: Some(10),
        };
        let mut calls = vec![];
        for_stmt.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["iter", "loop_fn"]);

        let while_stmt = Stmt::While {
            condition: Expr::FunctionCall {
                name: "cond".to_string(),
                args: vec![],
            },
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "body".to_string(),
                args: vec![],
            })],
            max_iterations: Some(10),
        };
        let mut calls = vec![];
        while_stmt.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["cond", "body"]);
    }

    // ===== Expression validation tests =====

    #[test]
    fn test_expr_literal_null_string() {
        let expr = Expr::Literal(Literal::Str("hello\0world".to_string()));
        let result = expr.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Null"));
    }

    #[test]
    fn test_expr_variable_empty_name() {
        let expr = Expr::Variable("".to_string());
        let result = expr.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_expr_function_call_empty_name() {
        let expr = Expr::FunctionCall {
            name: "".to_string(),
            args: vec![],
        };
        let result = expr.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_expr_method_call_empty_method() {
        let expr = Expr::MethodCall {
            receiver: Box::new(Expr::Variable("obj".to_string())),
            method: "".to_string(),
            args: vec![],
        };
        let result = expr.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_expr_nesting_depth() {
        // Create deeply nested expression
        let mut expr = Expr::Literal(Literal::U32(1));
        for _ in 0..35 {
            expr = Expr::Unary {
                op: UnaryOp::Neg,
                operand: Box::new(expr),
            };
        }
        let result = expr.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("nesting too deep"));
    }

    #[test]
    fn test_expr_nesting_depth_binary() {
        let leaf = Expr::Literal(Literal::U32(1));
        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(leaf.clone()),
            right: Box::new(leaf),
        };
        assert_eq!(expr.nesting_depth(), 1);
    }

    #[test]
    fn test_expr_collect_calls_nested() {
        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::FunctionCall {
                name: "left".to_string(),
                args: vec![],
            }),
            right: Box::new(Expr::FunctionCall {
                name: "right".to_string(),
                args: vec![],
            }),
        };
        let mut calls = vec![];
        expr.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["left", "right"]);
    }

    #[test]
    fn test_expr_collect_calls_array() {
        let expr = Expr::Array(vec![
            Expr::FunctionCall {
                name: "a".to_string(),
                args: vec![],
            },
            Expr::FunctionCall {
                name: "b".to_string(),
                args: vec![],
            },
        ]);
        let mut calls = vec![];
        expr.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["a", "b"]);
    }

    #[test]
    fn test_expr_collect_calls_index() {
        let expr = Expr::Index {
            object: Box::new(Expr::FunctionCall {
                name: "arr".to_string(),
                args: vec![],
            }),
            index: Box::new(Expr::FunctionCall {
                name: "idx".to_string(),
                args: vec![],
            }),
        };
        let mut calls = vec![];
        expr.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["arr", "idx"]);
    }

    #[test]
    fn test_expr_collect_calls_try() {
        let expr = Expr::Try {
            expr: Box::new(Expr::FunctionCall {
                name: "fallible".to_string(),
                args: vec![],
            }),
        };
        let mut calls = vec![];
        expr.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["fallible"]);
    }

    #[test]
    fn test_expr_collect_calls_block() {
        let expr = Expr::Block(vec![Stmt::Expr(Expr::FunctionCall {
            name: "inner".to_string(),
            args: vec![],
        })]);
        let mut calls = vec![];
        expr.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["inner"]);
    }

    #[test]
    fn test_expr_collect_calls_range() {
        let expr = Expr::Range {
            start: Box::new(Expr::FunctionCall {
                name: "start".to_string(),
                args: vec![],
            }),
            end: Box::new(Expr::FunctionCall {
                name: "end".to_string(),
                args: vec![],
            }),
            inclusive: false,
        };
        let mut calls = vec![];
        expr.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["start", "end"]);
    }

    // ===== Pattern validation tests =====

    #[test]
    fn test_pattern_literal_null_string() {
        let pattern = Pattern::Literal(Literal::Str("hello\0world".to_string()));
        let result = pattern.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Null"));
    }

    #[test]
    fn test_pattern_variable_empty() {
        let pattern = Pattern::Variable("".to_string());
        let result = pattern.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_pattern_wildcard_validates() {
        assert!(Pattern::Wildcard.validate().is_ok());
    }

    #[test]
    fn test_pattern_tuple_empty() {
        let pattern = Pattern::Tuple(vec![]);
        let result = pattern.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Empty tuple"));
    }

    #[test]
    fn test_pattern_tuple_valid() {
        let pattern = Pattern::Tuple(vec![Pattern::Variable("a".to_string()), Pattern::Wildcard]);
        assert!(pattern.validate().is_ok());
    }

    #[test]
    fn test_pattern_struct_empty() {
        let pattern = Pattern::Struct {
            name: "MyStruct".to_string(),
            fields: vec![],
        };
        let result = pattern.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Empty struct"));
    }

    #[test]
    fn test_pattern_struct_invalid_name() {
        let pattern = Pattern::Struct {
            name: "".to_string(),
            fields: vec![("x".to_string(), Pattern::Wildcard)],
        };
        let result = pattern.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_pattern_struct_invalid_field_name() {
        let pattern = Pattern::Struct {
            name: "MyStruct".to_string(),
            fields: vec![("".to_string(), Pattern::Wildcard)],
        };
        let result = pattern.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_pattern_binds_variable() {
        let pattern = Pattern::Variable("x".to_string());
        assert!(pattern.binds_variable("x"));
        assert!(!pattern.binds_variable("y"));
    }

    #[test]
    fn test_pattern_binds_variable_tuple() {
        let pattern = Pattern::Tuple(vec![
            Pattern::Variable("a".to_string()),
            Pattern::Variable("b".to_string()),
        ]);
        assert!(pattern.binds_variable("a"));
        assert!(pattern.binds_variable("b"));
        assert!(!pattern.binds_variable("c"));
    }

    #[test]
    fn test_pattern_binds_variable_struct() {
        let pattern = Pattern::Struct {
            name: "Point".to_string(),
            fields: vec![
                ("x".to_string(), Pattern::Variable("px".to_string())),
                ("y".to_string(), Pattern::Variable("py".to_string())),
            ],
        };
        assert!(pattern.binds_variable("px"));
        assert!(pattern.binds_variable("py"));
        assert!(!pattern.binds_variable("x"));
    }

    #[test]
    fn test_pattern_binds_variable_wildcard() {
        assert!(!Pattern::Wildcard.binds_variable("x"));
    }

    #[test]
    fn test_pattern_binds_variable_literal() {
        let pattern = Pattern::Literal(Literal::U32(42));
        assert!(!pattern.binds_variable("x"));
    }

    // ===== Literal tests =====

    #[test]
    fn test_literal_eq() {
        assert_eq!(Literal::Bool(true), Literal::Bool(true));
        assert_ne!(Literal::Bool(true), Literal::Bool(false));
        assert_eq!(Literal::U32(42), Literal::U32(42));
        assert_eq!(Literal::I32(-5), Literal::I32(-5));
        assert_eq!(
            Literal::Str("hello".to_string()),
            Literal::Str("hello".to_string())
        );
    }

    // ===== No recursion with multiple functions =====

    #[test]
    fn test_no_recursion_chain() {
        let ast = RestrictedAst {
            entry_point: "a".to_string(),
            functions: vec![
                Function {
                    name: "a".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Expr(Expr::FunctionCall {
                        name: "b".to_string(),
                        args: vec![],
                    })],
                },
                Function {
                    name: "b".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Expr(Expr::FunctionCall {
                        name: "c".to_string(),
                        args: vec![],
                    })],
                },
                Function {
                    name: "c".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![],
                },
            ],
        };
        assert!(ast.validate().is_ok());

    }
