#[cfg(test)]
mod tests {
    use crate::ast::restricted::{BinaryOp, Literal, UnaryOp};
    use crate::ast::{Expr, Function, RestrictedAst, Stmt, Type};
    use crate::ir::{shell_ir::CaseArm, shell_ir::CasePattern, Command, ShellIR, ShellValue};
    use crate::models::config::Config;
    use crate::validation::pipeline::ValidationPipeline;
    use crate::validation::ValidationLevel;

    fn strict_pipeline() -> ValidationPipeline {
        let config = Config {
            validation_level: Some(ValidationLevel::Strict),
            strict_mode: true,
            ..Config::default()
        };
        ValidationPipeline::new(&config)
    }

    fn none_pipeline() -> ValidationPipeline {
        let config = Config {
            validation_level: Some(ValidationLevel::None),
            strict_mode: false,
            ..Config::default()
        };
        ValidationPipeline::new(&config)
    }

    fn minimal_pipeline() -> ValidationPipeline {
        let config = Config {
            validation_level: Some(ValidationLevel::Minimal),
            strict_mode: false,
            ..Config::default()
        };
        ValidationPipeline::new(&config)
    }

    /// Build a RestrictedAst with a single main function containing the given body.
    fn ast_with_body(body: Vec<Stmt>) -> RestrictedAst {
        RestrictedAst {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body,
            }],
            entry_point: "main".to_string(),
        }
    }

    /// Build an exec() call expression with the given string arg.
    fn exec_call(arg: &str) -> Expr {
        Expr::FunctionCall {
            name: "exec".to_string(),
            args: vec![Expr::Literal(Literal::Str(arg.to_string()))],
        }
    }

    // ── validate_ast exercises validate_expr → exec-context dispatch ──

    #[test]
    fn test_exec_ctx_pipe_allowed_via_ast() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(exec_call("ldd /usr/bin/ls | grep libc"))]);
        // Pipe is allowed in exec() args
        assert!(p.validate_ast(&ast).is_ok(), "Pipe in exec() should be ok");
    }

    #[test]
    fn test_exec_ctx_and_operator_allowed_via_ast() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(exec_call("cmd1 && cmd2"))]);
        assert!(p.validate_ast(&ast).is_ok(), "AND in exec() should be ok");
    }

    #[test]
    fn test_exec_ctx_or_operator_allowed_via_ast() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(exec_call("cmd1 || cmd2"))]);
        assert!(p.validate_ast(&ast).is_ok(), "OR in exec() should be ok");
    }

    #[test]
    fn test_exec_ctx_shellshock_blocked_via_ast() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(exec_call("() { :; }; echo pwned"))]);
        assert!(
            p.validate_ast(&ast).is_err(),
            "Shellshock blocked in exec()"
        );
    }

    #[test]
    fn test_exec_ctx_command_substitution_blocked_via_ast() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(exec_call("echo $(whoami)"))]);
        assert!(p.validate_ast(&ast).is_err(), "$(cmd) blocked in exec()");
    }

    #[test]
    fn test_exec_ctx_backtick_blocked_via_ast() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(exec_call("echo `id`"))]);
        assert!(p.validate_ast(&ast).is_err(), "Backtick blocked in exec()");
    }

    #[test]
    fn test_exec_ctx_safe_string_allowed() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(exec_call("ls -la /tmp"))]);
        assert!(p.validate_ast(&ast).is_ok(), "Safe string ok in exec()");
    }

    // ── Exec with multiple args — all dispatched through exec context ──

    #[test]
    fn test_exec_ctx_multi_arg_all_safe() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(Expr::FunctionCall {
            name: "exec".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("grep".to_string())),
                Expr::Literal(Literal::Str("-r".to_string())),
                Expr::Variable("pattern".to_string()),
            ],
        })]);
        assert!(p.validate_ast(&ast).is_ok());
    }

    #[test]
    fn test_exec_ctx_multi_arg_one_bad() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(Expr::FunctionCall {
            name: "exec".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("safe".to_string())),
                Expr::Literal(Literal::Str("() { :; }".to_string())),
            ],
        })]);
        assert!(
            p.validate_ast(&ast).is_err(),
            "Shellshock in second arg blocked"
        );
    }

    // ── exec() with non-Literal args (Variable, Binary, etc.) ──

    #[test]
    fn test_exec_ctx_variable_arg() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(Expr::FunctionCall {
            name: "exec".to_string(),
            args: vec![Expr::Variable("cmd".to_string())],
        })]);
        assert!(p.validate_ast(&ast).is_ok(), "Variable arg in exec() ok");
    }

    #[test]
    fn test_exec_ctx_empty_variable_name_rejected() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(Expr::FunctionCall {
            name: "exec".to_string(),
            args: vec![Expr::Variable(String::new())],
        })]);
        assert!(p.validate_ast(&ast).is_err(), "Empty var name blocked");
    }

    #[test]
    fn test_exec_ctx_binary_expr_arg() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(Expr::FunctionCall {
            name: "exec".to_string(),
            args: vec![Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Literal(Literal::U32(1))),
                right: Box::new(Expr::Literal(Literal::U32(2))),
            }],
        })]);
        assert!(p.validate_ast(&ast).is_ok(), "Binary expr arg in exec() ok");
    }

    #[test]
    fn test_exec_ctx_binary_with_shellshock_in_left() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(Expr::FunctionCall {
            name: "exec".to_string(),
            args: vec![Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Literal(Literal::Str("() { :; }".to_string()))),
                right: Box::new(Expr::Literal(Literal::U32(2))),
            }],
        })]);
        assert!(
            p.validate_ast(&ast).is_err(),
            "Shellshock in binary left blocked"
        );
    }

    #[test]
    fn test_exec_ctx_unary_arg() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(Expr::FunctionCall {
            name: "exec".to_string(),
            args: vec![Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(Expr::Literal(Literal::Bool(false))),
            }],
        })]);
        assert!(p.validate_ast(&ast).is_ok(), "Unary arg in exec() ok");
    }
}

#[cfg(test)]
mod pipeline_coverage_tests_tests_extracted_exec {
    use super::*;
}

include!("pipeline_coverage_tests_tests_extracted_exec.rs");
