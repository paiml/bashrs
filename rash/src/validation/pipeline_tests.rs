#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::ast::restricted::Literal;
    use crate::ast::{Expr, Function, RestrictedAst, Stmt};
    use crate::ir::{Command, ShellIR, ShellValue};
    use crate::models::config::Config;

    fn create_test_pipeline(level: ValidationLevel, strict: bool) -> pipeline::ValidationPipeline {
        let config = Config {
            validation_level: Some(level),
            strict_mode: strict,
            ..Config::default()
        };
        pipeline::ValidationPipeline::new(&config)
    }

    #[test]
    fn test_pipeline_creation() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        assert_eq!(pipeline.level, ValidationLevel::Minimal);
        assert!(!pipeline.strict_mode);
    }

    #[test]
    fn test_validate_ast_none_level() {
        let pipeline = create_test_pipeline(ValidationLevel::None, false);
        let ast = RestrictedAst {
            functions: vec![],
            entry_point: "main".to_string(),
        };
        assert!(pipeline.validate_ast(&ast).is_ok());
    }

    #[test]
    fn test_validate_ast_with_statements() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let ast = RestrictedAst {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                return_type: crate::ast::Type::Void,
                body: vec![
                    Stmt::Let {
                        name: "x".to_string(),
                        value: Expr::Literal(Literal::U32(42)),
                    },
                    Stmt::Expr(Expr::Variable("x".to_string())),
                ],
            }],
            entry_point: "main".to_string(),
        };
        assert!(pipeline.validate_ast(&ast).is_ok());
    }

    #[test]
    fn test_validate_ir_none_level() {
        let pipeline = create_test_pipeline(ValidationLevel::None, false);
        let ir = ShellIR::Noop;
        assert!(pipeline.validate_ir(&ir).is_ok());
    }

    #[test]
    fn test_validate_ir_sequence() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let ir = ShellIR::Sequence(vec![
            ShellIR::Let {
                name: "x".to_string(),
                value: ShellValue::String("42".to_string()),
                effects: crate::ir::EffectSet::pure(),
            },
            ShellIR::Exec {
                cmd: Command {
                    program: "echo".to_string(),
                    args: vec![ShellValue::Variable("x".to_string())],
                },
                effects: crate::ir::EffectSet::pure(),
            },
        ]);
        assert!(pipeline.validate_ir(&ir).is_ok());
    }

    #[test]
    fn test_validate_backticks_error() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let value = ShellValue::CommandSubst(Command {
            program: "echo `date`".to_string(),
            args: vec![],
        });
        let result = pipeline.validate_shell_value(&value);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("SC2006"));
    }

    #[test]
    fn test_validate_if_statement() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let ir = ShellIR::If {
            test: ShellValue::String("true".to_string()),
            then_branch: Box::new(ShellIR::Noop),
            else_branch: Some(Box::new(ShellIR::Noop)),
        };
        assert!(pipeline.validate_ir(&ir).is_ok());
    }

    #[test]
    fn test_validate_output_none_level() {
        let pipeline = create_test_pipeline(ValidationLevel::None, false);
        assert!(pipeline.validate_output("#!/bin/sh\necho hello").is_ok());
    }

    #[test]
    fn test_report_error_strict_mode() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, true);
        let error = ValidationError {
            rule: "SC2086",
            message: "Double quote".to_string(),
            severity: Severity::Error,
            suggestion: None,
            auto_fix: None,
            line: Some(1),
            column: Some(1),
        };
        let msg = pipeline.report_error(&error);
        assert!(msg.starts_with("ERROR:"));
    }

    #[test]
    fn test_report_error_non_strict() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let error = ValidationError {
            rule: "SC2086",
            message: "Double quote".to_string(),
            severity: Severity::Warning,
            suggestion: None,
            auto_fix: None,
            line: Some(1),
            column: Some(1),
        };
        let msg = pipeline.report_error(&error);
        assert!(!msg.starts_with("ERROR:"));
    }

    #[test]
    fn test_should_fail_strict_mode() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, true);
        let errors = vec![ValidationError {
            rule: "SC2086",
            message: "Double quote".to_string(),
            severity: Severity::Warning,
            suggestion: None,
            auto_fix: None,
            line: Some(1),
            column: Some(1),
        }];
        assert!(pipeline.should_fail(&errors));
    }

    #[test]
    fn test_should_fail_non_strict_with_error() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let errors = vec![ValidationError {
            rule: "SC2086",
            message: "Double quote".to_string(),
            severity: Severity::Error,
            suggestion: None,
            auto_fix: None,
            line: Some(1),
            column: Some(1),
        }];
        assert!(pipeline.should_fail(&errors));
    }

    #[test]
    fn test_should_not_fail_non_strict_with_warning() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let errors = vec![ValidationError {
            rule: "SC2086",
            message: "Double quote".to_string(),
            severity: Severity::Warning,
            suggestion: None,
            auto_fix: None,
            line: Some(1),
            column: Some(1),
        }];
        assert!(!pipeline.should_fail(&errors));
    }

    #[test]
    fn test_validate_concat_shell_value() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let value = ShellValue::Concat(vec![
            ShellValue::String("Hello ".to_string()),
            ShellValue::Variable("name".to_string()),
        ]);
        assert!(pipeline.validate_shell_value(&value).is_ok());
    }

    #[test]
    fn test_validate_if_with_complex_branches() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let ast = RestrictedAst {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                return_type: crate::ast::Type::Void,
                body: vec![Stmt::If {
                    condition: Expr::Literal(Literal::Bool(true)),
                    then_block: vec![Stmt::Let {
                        name: "x".to_string(),
                        value: Expr::Literal(Literal::U32(1)),
                    }],
                    else_block: Some(vec![Stmt::Let {
                        name: "y".to_string(),
                        value: Expr::Literal(Literal::U32(2)),
                    }]),
                }],
            }],
            entry_point: "main".to_string(),
        };
        assert!(pipeline.validate_ast(&ast).is_ok());
    }

    #[test]
    fn test_validate_expression_unquoted_variable() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let expr = crate::ir::ShellExpression::Variable("test".to_string(), false);
        let result = pipeline.validate_expression(&expr);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("SC2086"));
    }

    #[test]
    fn test_validate_expression_quoted_variable() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let expr = crate::ir::ShellExpression::Variable("test".to_string(), true);
        let result = pipeline.validate_expression(&expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_expression_backtick_command() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let expr = crate::ir::ShellExpression::Command("echo `date`".to_string());
        let result = pipeline.validate_expression(&expr);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("SC2006"));
    }

    #[test]
    fn test_validate_expression_safe_command() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let expr = crate::ir::ShellExpression::Command("echo $(date)".to_string());
        let result = pipeline.validate_expression(&expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_output_with_rules() {
        // This test only runs in debug builds
        #[cfg(debug_assertions)]
        {
            let pipeline = create_test_pipeline(ValidationLevel::Strict, false);
            let script = "#!/bin/sh\necho $unquoted_var";
            // The embedded rules would catch this
            let _ = pipeline.validate_output(script);
        }
    }

    #[test]
    fn test_validate_shell_value_various_types() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        
        // Test Bool value
        let bool_val = ShellValue::Bool(true);
        assert!(pipeline.validate_shell_value(&bool_val).is_ok());
        
        // Test String value
        let str_val = ShellValue::String("test".to_string());
        assert!(pipeline.validate_shell_value(&str_val).is_ok());
        
        // Test empty Concat
        let empty_concat = ShellValue::Concat(vec![]);
        assert!(pipeline.validate_shell_value(&empty_concat).is_ok());
    }

    #[test]
    fn test_validate_ir_exit() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let ir = ShellIR::Exit { code: 0, message: None };
        assert!(pipeline.validate_ir(&ir).is_ok());
    }

    #[test]
    fn test_validate_nested_concat() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let value = ShellValue::Concat(vec![
            ShellValue::String("prefix_".to_string()),
            ShellValue::Concat(vec![
                ShellValue::Variable("var1".to_string()),
                ShellValue::String("_middle_".to_string()),
                ShellValue::Variable("var2".to_string()),
            ]),
            ShellValue::String("_suffix".to_string()),
        ]);
        assert!(pipeline.validate_shell_value(&value).is_ok());
    }

    #[test]
    fn test_validate_command_with_backtick_in_args() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let value = ShellValue::CommandSubst(Command {
            program: "echo".to_string(),
            args: vec![ShellValue::String("test `cmd`".to_string())],
        });
        // The backtick is in the program string, not args
        let result = pipeline.validate_shell_value(&value);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_fail_empty_errors() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        assert!(!pipeline.should_fail(&[]));
        
        let strict_pipeline = create_test_pipeline(ValidationLevel::Minimal, true);
        assert!(!strict_pipeline.should_fail(&[]));
    }

    #[test]
    fn test_validate_ir_with_effects() {
        let pipeline = create_test_pipeline(ValidationLevel::Strict, false);
        let ir = ShellIR::Let {
            name: "test".to_string(),
            value: ShellValue::String("value".to_string()),
            effects: crate::ir::EffectSet::pure(),
        };
        assert!(pipeline.validate_ir(&ir).is_ok());
    }

    #[test]
    fn test_validate_expression_literal() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let expr = crate::ir::ShellExpression::String("test".to_string());
        assert!(pipeline.validate_expression(&expr).is_ok());
    }

    #[test]
    fn test_validate_nested_if_statements() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let inner_if = ShellIR::If {
            test: ShellValue::String("true".to_string()),
            then_branch: Box::new(ShellIR::Noop),
            else_branch: None,
        };
        let outer_if = ShellIR::If {
            test: ShellValue::String("true".to_string()),
            then_branch: Box::new(inner_if),
            else_branch: Some(Box::new(ShellIR::Exit { code: 1, message: None })),
        };
        assert!(pipeline.validate_ir(&outer_if).is_ok());
    }

    #[test]
    fn test_validate_expr_empty_variable_name() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let ast = RestrictedAst {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                return_type: crate::ast::Type::Void,
                body: vec![Stmt::Expr(Expr::Variable("".to_string()))],
            }],
            entry_point: "main".to_string(),
        };
        let result = pipeline.validate_ast(&ast);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty variable name"));
    }

    #[test]
    fn test_validate_expr_variable_with_whitespace() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let ast = RestrictedAst {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                return_type: crate::ast::Type::Void,
                body: vec![Stmt::Expr(Expr::Variable("var name".to_string()))],
            }],
            entry_point: "main".to_string(),
        };
        let result = pipeline.validate_ast(&ast);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("contains whitespace"));
    }

    #[test]
    fn test_validate_expr_empty_function_name() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let ast = RestrictedAst {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                return_type: crate::ast::Type::Void,
                body: vec![Stmt::Expr(Expr::FunctionCall {
                    name: "".to_string(),
                    args: vec![],
                })],
            }],
            entry_point: "main".to_string(),
        };
        let result = pipeline.validate_ast(&ast);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty function name"));
    }

    #[test]
    fn test_validate_expr_empty_method_name() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let ast = RestrictedAst {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                return_type: crate::ast::Type::Void,
                body: vec![Stmt::Expr(Expr::MethodCall {
                    receiver: Box::new(Expr::Variable("obj".to_string())),
                    method: "".to_string(),
                    args: vec![],
                })],
            }],
            entry_point: "main".to_string(),
        };
        let result = pipeline.validate_ast(&ast);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty method name"));
    }

    #[test]
    fn test_validate_expr_binary_operations() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let ast = RestrictedAst {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                return_type: crate::ast::Type::Void,
                body: vec![Stmt::Let {
                    name: "result".to_string(),
                    value: Expr::Binary {
                        op: crate::ast::restricted::BinaryOp::Add,
                        left: Box::new(Expr::Literal(Literal::U32(1))),
                        right: Box::new(Expr::Binary {
                            op: crate::ast::restricted::BinaryOp::Mul,
                            left: Box::new(Expr::Literal(Literal::U32(2))),
                            right: Box::new(Expr::Variable("x".to_string())),
                        }),
                    },
                }],
            }],
            entry_point: "main".to_string(),
        };
        assert!(pipeline.validate_ast(&ast).is_ok());
    }

    #[test]
    fn test_validate_expr_unary_operations() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let ast = RestrictedAst {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                return_type: crate::ast::Type::Void,
                body: vec![Stmt::Let {
                    name: "result".to_string(),
                    value: Expr::Unary {
                        op: crate::ast::restricted::UnaryOp::Not,
                        operand: Box::new(Expr::Unary {
                            op: crate::ast::restricted::UnaryOp::Neg,
                            operand: Box::new(Expr::Variable("x".to_string())),
                        }),
                    },
                }],
            }],
            entry_point: "main".to_string(),
        };
        assert!(pipeline.validate_ast(&ast).is_ok());
    }

    #[test]
    fn test_validate_method_call_with_args() {
        let pipeline = create_test_pipeline(ValidationLevel::Minimal, false);
        let ast = RestrictedAst {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                return_type: crate::ast::Type::Void,
                body: vec![Stmt::Expr(Expr::MethodCall {
                    receiver: Box::new(Expr::Variable("obj".to_string())),
                    method: "process".to_string(),
                    args: vec![
                        Expr::Literal(Literal::Str("arg1".to_string())),
                        Expr::Variable("arg2".to_string()),
                    ],
                })],
            }],
            entry_point: "main".to_string(),
        };
        assert!(pipeline.validate_ast(&ast).is_ok());
    }
}
