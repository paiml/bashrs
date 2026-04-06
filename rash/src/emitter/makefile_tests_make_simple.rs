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

}

include!("makefile_tests_extracted_MAKE.rs");
