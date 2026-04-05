
    #[test]
    fn test_MAKE_BUILD_012_non_main_function_as_target() {
        // Non-main functions with echo/println become helper targets
        let ast = RestrictedAst {
            functions: vec![
                Function {
                    name: "main".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Let {
                        name: "cc".to_string(),
                        value: Expr::Literal(Literal::Str("gcc".to_string())),
                        declaration: true,
                    }],
                },
                Function {
                    name: "help".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("Usage: make build".to_string()))],
                    })],
                },
            ],
            entry_point: "main".to_string(),
        };

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("CC := gcc"), "Main var in: {result}");
        assert!(result.contains("help"), "Helper target in: {result}");
    }

    #[test]
    fn test_MAKE_BUILD_013_ignored_stmt_types() {
        // Non-Let, non-Expr statements should be ignored
        let ast = make_simple_ast(vec![
            Stmt::Let {
                name: "cc".to_string(),
                value: Expr::Literal(Literal::Str("gcc".to_string())),
                declaration: true,
            },
            Stmt::Expr(Expr::FunctionCall {
                name: "unknown_func".to_string(),
                args: vec![],
            }),
        ]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("CC := gcc"));
    }

    // --- Coverage tests for uncovered lines ---

    #[test]
    fn test_MAKE_COV_001_convert_stmt_return_catchall() {
        // Line 118: _ => Ok(None) in convert_stmt — Stmt::Return is not Let or Expr
        let ast = make_simple_ast(vec![
            Stmt::Let {
                name: "cc".to_string(),
                value: Expr::Literal(Literal::Str("gcc".to_string())),
                declaration: true,
            },
            Stmt::Return(None),
        ]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("CC := gcc"));
    }

    #[test]
    fn test_MAKE_COV_002_convert_expr_non_functioncall() {
        // Line 144: _ => Ok(None) in convert_expr — Expr::Variable is not FunctionCall
        let ast = make_simple_ast(vec![
            Stmt::Let {
                name: "cc".to_string(),
                value: Expr::Literal(Literal::Str("gcc".to_string())),
                declaration: true,
            },
            Stmt::Expr(Expr::Variable("some_var".to_string())),
        ]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("CC := gcc"));
    }

    #[test]
    fn test_MAKE_COV_003_non_main_function_non_echo_body() {
        // Lines 87-88: Function body stmt is FunctionCall but NOT echo/println
        let ast = RestrictedAst {
            functions: vec![
                Function {
                    name: "main".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Let {
                        name: "cc".to_string(),
                        value: Expr::Literal(Literal::Str("gcc".to_string())),
                        declaration: true,
                    }],
                },
                Function {
                    name: "setup".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Expr(Expr::FunctionCall {
                        name: "run_cmd".to_string(),
                        args: vec![Expr::Literal(Literal::Str("init".to_string()))],
                    })],
                },
            ],
            entry_point: "main".to_string(),
        };

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("CC := gcc"));
        // "setup" function has no echo/println so no target is generated
        assert!(!result.contains("setup:"));
    }

    #[test]
    fn test_MAKE_COV_004_non_main_function_non_expr_stmt() {
        // Lines 87-88: Function body stmt is NOT Stmt::Expr(FunctionCall{..})
        let ast = RestrictedAst {
            functions: vec![
                Function {
                    name: "main".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Let {
                        name: "cc".to_string(),
                        value: Expr::Literal(Literal::Str("gcc".to_string())),
                        declaration: true,
                    }],
                },
                Function {
                    name: "init".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Return(None)],
                },
            ],
            entry_point: "main".to_string(),
        };

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("CC := gcc"));
    }

    #[test]
    fn test_MAKE_COV_005_extract_string_array_dunder_array() {
        // Lines 204-209: __array__ function call in extract_string_array
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "target".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("build".to_string())),
                Expr::FunctionCall {
                    name: "__array__".to_string(),
                    args: vec![
                        Expr::Literal(Literal::Str("dep1".to_string())),
                        Expr::Literal(Literal::Str("dep2".to_string())),
                    ],
                },
                Expr::Array(vec![Expr::Literal(Literal::Str(
                    "gcc -o build".to_string(),
                ))]),
            ],
        })]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("build"), "Target in: {result}");
    }

    #[test]
    fn test_MAKE_COV_006_extract_string_array_fallback_expr() {
        // Line 217: catch-all _ => in extract_string_array — non-string expr
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "target".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("build".to_string())),
                Expr::Variable("deps_var".to_string()),
                Expr::Array(vec![Expr::Literal(Literal::Str(
                    "gcc -o build".to_string(),
                ))]),
            ],
        })]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("$(DEPS_VAR)"), "Variable ref in: {result}");
    }

    #[test]
    fn test_MAKE_COV_007_expr_to_string_array_succeeds() {
        // Arrays are now converted to space-separated Makefile values
        let ast = make_simple_ast(vec![Stmt::Let {
            name: "data".to_string(),
            value: Expr::Array(vec![
                Expr::Literal(Literal::Str("a".to_string())),
                Expr::Literal(Literal::Str("b".to_string())),
            ]),
            declaration: true,
        }]);

        let result = emit_makefile(&ast);
        assert!(
            result.is_ok(),
            "Array should convert to Makefile value: {:?}",
            result
        );
        let output = result.expect("verified ok above");
        assert!(output.contains("DATA := a b"), "Output: {}", output);
    }

    #[test]
    fn test_MAKE_COV_007b_expr_to_string_unsupported() {
        // Truly unsupported expression types still fail
        let ast = make_simple_ast(vec![Stmt::Let {
            name: "data".to_string(),
            value: Expr::Block(vec![]),
            declaration: true,
        }]);

        let err = emit_makefile(&ast).unwrap_err();
        assert!(
            format!("{err}").contains("Cannot convert expression"),
            "Error: {err}"
        );
    }

    #[test]
    fn test_MAKE_COV_008_non_main_function_empty_recipes() {
        // Line 104: !recipes.is_empty() is false — function with no echo
        let ast = RestrictedAst {
            functions: vec![
                Function {
                    name: "main".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Let {
                        name: "cc".to_string(),
                        value: Expr::Literal(Literal::Str("gcc".to_string())),
                        declaration: true,
                    }],
                },
                Function {
                    name: "check".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Let {
                        name: "x".to_string(),
                        value: Expr::Literal(Literal::Str("val".to_string())),
                        declaration: true,
                    }],
                },
            ],
            entry_point: "main".to_string(),
        };

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("CC := gcc"));
        // check function has no echo → no target
        assert!(!result.contains("check:"));
    }

    #[test]
    fn test_MAKE_BUILD_014_combined_vars_and_targets() {
        let ast = make_simple_ast(vec![
            Stmt::Let {
                name: "cc".to_string(),
                value: Expr::Literal(Literal::Str("gcc".to_string())),
                declaration: true,
            },
            Stmt::Let {
                name: "cflags".to_string(),
                value: Expr::Literal(Literal::Str("-O2".to_string())),
                declaration: true,
            },
            Stmt::Expr(Expr::FunctionCall {
                name: "phony_target".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("all".to_string())),
                    Expr::Array(vec![Expr::Literal(Literal::Str("build".to_string()))]),
                    Expr::Array(vec![]),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "target".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("build".to_string())),
                    Expr::Array(vec![Expr::Literal(Literal::Str("main.c".to_string()))]),
                    Expr::Array(vec![Expr::Literal(Literal::Str(
                        "$(CC) $(CFLAGS) -o build main.c".to_string(),
                    ))]),
                ],
            }),
        ]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("CC := gcc"));
        assert!(result.contains("CFLAGS := -O2"));
        assert!(result.contains("all"), "all target in: {result}");
        assert!(result.contains("build"), "build target in: {result}");
    }
}
