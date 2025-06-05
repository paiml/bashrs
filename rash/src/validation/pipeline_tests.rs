#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::ast::{Expr, Function, RestrictedAst, Stmt};
    use crate::ast::restricted::Literal;
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
}