#[cfg(test)]
mod tests {
    use super::*;

    // ===== RestrictedAst validation tests =====

    fn create_valid_ast() -> RestrictedAst {
        RestrictedAst {
            entry_point: "main".to_string(),
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![],
            }],
        }
    }

    #[test]
    fn test_valid_ast_validates() {
        let ast = create_valid_ast();
        assert!(ast.validate().is_ok());
    }

    #[test]
    fn test_missing_entry_point() {
        let ast = RestrictedAst {
            entry_point: "nonexistent".to_string(),
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![],
            }],
        };
        let result = ast.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Entry point function"));
    }

    #[test]
    fn test_recursion_allowed_direct() {
        // Recursive functions are allowed — shell supports them
        let ast = RestrictedAst {
            entry_point: "a".to_string(),
            functions: vec![Function {
                name: "a".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Expr(Expr::FunctionCall {
                    name: "a".to_string(),
                    args: vec![],
                })],
            }],
        };
        let result = ast.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_recursion_allowed_indirect() {
        // Indirect recursion is also allowed
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
                        name: "a".to_string(),
                        args: vec![],
                    })],
                },
            ],
        };
        let result = ast.validate();
        assert!(result.is_ok());
    }

    // ===== Function validation tests =====

    #[test]
    fn test_function_empty_name() {
        let func = Function {
            name: "".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![],
        };
        let result = func.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_function_null_char_in_name() {
        let func = Function {
            name: "func\0name".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![],
        };
        let result = func.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Null"));
    }

    #[test]
    fn test_function_unsafe_chars_in_name() {
        for c in ["$", "`", "\\"] {
            let func = Function {
                name: format!("func{}name", c),
                params: vec![],
                return_type: Type::Void,
                body: vec![],
            };
            let result = func.validate();
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("Unsafe"));
        }
    }

    #[test]
    fn test_function_duplicate_params() {
        let func = Function {
            name: "test".to_string(),
            params: vec![
                Parameter {
                    name: "x".to_string(),
                    param_type: Type::U32,
                },
                Parameter {
                    name: "x".to_string(),
                    param_type: Type::U32,
                },
            ],
            return_type: Type::Void,
            body: vec![],
        };
        let result = func.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate parameter"));
    }

    #[test]
    fn test_function_collect_calls() {
        let func = Function {
            name: "test".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![
                Stmt::Expr(Expr::FunctionCall {
                    name: "foo".to_string(),
                    args: vec![],
                }),
                Stmt::Expr(Expr::FunctionCall {
                    name: "bar".to_string(),
                    args: vec![],
                }),
            ],
        };
        let mut calls = vec![];
        func.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["foo", "bar"]);
    }

    // ===== Type tests =====

    #[test]
    fn test_type_is_allowed_basic() {
        assert!(Type::Void.is_allowed());
        assert!(Type::Bool.is_allowed());
        assert!(Type::U32.is_allowed());
        assert!(Type::Str.is_allowed());
    }

    #[test]
    fn test_type_is_allowed_result() {
        let result_type = Type::Result {
            ok_type: Box::new(Type::U32),
            err_type: Box::new(Type::Str),
        };
        assert!(result_type.is_allowed());
    }

    #[test]
    fn test_type_is_allowed_option() {
        let option_type = Type::Option {
            inner_type: Box::new(Type::Bool),
        };
        assert!(option_type.is_allowed());
    }

    // ===== Statement validation tests =====

include!("restricted_tests_extracted_stmt.rs");
