#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::ast::restricted::{Function, Type};

    fn make_simple_ast(stmts: Vec<Stmt>) -> RestrictedAst {
        RestrictedAst {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: stmts,
            }],
            entry_point: "main".to_string(),
        }
    }

    #[test]
    fn test_MAKE_BUILD_001_basic_variable() {
        let ast = make_simple_ast(vec![Stmt::Let {
            name: "cc".to_string(),
            value: Expr::Literal(Literal::Str("gcc".to_string())),
            declaration: true,
        }]);

        let result = emit_makefile(&ast).unwrap();
        assert!(
            result.contains("CC := gcc"),
            "Expected 'CC := gcc' in: {}",
            result
        );
    }

    #[test]
    fn test_MAKE_BUILD_002_multiple_variables() {
        let ast = make_simple_ast(vec![
            Stmt::Let {
                name: "cc".to_string(),
                value: Expr::Literal(Literal::Str("gcc".to_string())),
                declaration: true,
            },
            Stmt::Let {
                name: "cflags".to_string(),
                value: Expr::Literal(Literal::Str("-O2 -Wall".to_string())),
                declaration: true,
            },
        ]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("CC := gcc"));
        assert!(result.contains("CFLAGS := -O2 -Wall"));
    }

    #[test]
    fn test_MAKE_BUILD_003_target_with_deps_and_recipes() {
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "target".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("build".to_string())),
                Expr::Array(vec![Expr::Literal(Literal::Str("src/main.c".to_string()))]),
                Expr::Array(vec![Expr::Literal(Literal::Str(
                    "$(CC) -o build src/main.c".to_string(),
                ))]),
            ],
        })]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("build"), "Target name 'build' in: {result}");
        assert!(result.contains("src/main.c"), "Dep in: {result}");
    }

    #[test]
    fn test_MAKE_BUILD_004_phony_target() {
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "phony_target".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("clean".to_string())),
                Expr::Array(vec![]),
                Expr::Array(vec![Expr::Literal(Literal::Str("rm -f build".to_string()))]),
            ],
        })]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("clean"), "Phony target in: {result}");
        assert!(result.contains(".PHONY"), ".PHONY in: {result}");
    }

    #[test]
    fn test_MAKE_BUILD_005_target_deps_only() {
        // target with 2 args (no recipes)
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "target".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("all".to_string())),
                Expr::Array(vec![
                    Expr::Literal(Literal::Str("build".to_string())),
                    Expr::Literal(Literal::Str("test".to_string())),
                ]),
            ],
        })]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("all"), "Target 'all' in: {result}");
    }

    #[test]
    fn test_MAKE_BUILD_006_target_too_few_args() {
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "target".to_string(),
            args: vec![Expr::Literal(Literal::Str("build".to_string()))],
        })]);

        let err = emit_makefile(&ast).unwrap_err();
        assert!(
            format!("{err}").contains("at least 2 arguments"),
            "Error: {err}"
        );
    }

    #[test]
    fn test_MAKE_BUILD_007_target_non_string_name() {
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "target".to_string(),
            args: vec![Expr::Literal(Literal::I32(42)), Expr::Array(vec![])],
        })]);

        let err = emit_makefile(&ast).unwrap_err();
        assert!(format!("{err}").contains("string literal"), "Error: {err}");
    }

    #[test]
    fn test_MAKE_BUILD_008_variable_expr_to_string() {
        let ast = make_simple_ast(vec![Stmt::Let {
            name: "output".to_string(),
            value: Expr::Variable("cc".to_string()),
            declaration: true,
        }]);

        let result = emit_makefile(&ast).unwrap();
        assert!(
            result.contains("$(CC)"),
            "Variable ref should be $(CC) in: {result}"
        );
    }

    #[test]
    fn test_MAKE_BUILD_009_numeric_literals() {
        let ast = make_simple_ast(vec![
            Stmt::Let {
                name: "port".to_string(),
                value: Expr::Literal(Literal::U16(8080)),
                declaration: true,
            },
            Stmt::Let {
                name: "count".to_string(),
                value: Expr::Literal(Literal::I32(42)),
                declaration: true,
            },
            Stmt::Let {
                name: "size".to_string(),
                value: Expr::Literal(Literal::U32(1024)),
                declaration: true,
            },
            Stmt::Let {
                name: "verbose".to_string(),
                value: Expr::Literal(Literal::Bool(true)),
                declaration: true,
            },
        ]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("8080"), "U16 in: {result}");
        assert!(result.contains("42"), "I32 in: {result}");
        assert!(result.contains("1024"), "U32 in: {result}");
        assert!(result.contains("true"), "Bool in: {result}");
    }

    #[test]
    fn test_MAKE_BUILD_010_extract_string_array_single() {
        // Single string treated as array of one
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "target".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("build".to_string())),
                Expr::Literal(Literal::Str("main.c".to_string())),
                Expr::Array(vec![Expr::Literal(Literal::Str(
                    "gcc -o build main.c".to_string(),
                ))]),
            ],
        })]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("build"), "Target in: {result}");
    }

    #[test]
    fn test_MAKE_BUILD_011_extract_string_array_empty() {
        // Empty string treated as empty deps
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "target".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("clean".to_string())),
                Expr::Literal(Literal::Str("".to_string())),
                Expr::Array(vec![Expr::Literal(Literal::Str("rm -f build".to_string()))]),
            ],
        })]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("clean"), "Target in: {result}");
    }

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
