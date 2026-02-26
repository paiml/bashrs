//! Coverage tests for `validate_expr_in_exec_context` exercised via the public API.
//!
//! Since `validate_expr_in_exec_context` is private, we exercise it by calling
//! `validate_expr` on a `FunctionCall { name: "exec", args: [...] }` expression,
//! which internally dispatches to `validate_expr_in_exec_context` for each arg.
//! We also test via `validate_ast` and `validate_ir`.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

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

    #[test]
    fn test_exec_ctx_array_arg() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(Expr::FunctionCall {
            name: "exec".to_string(),
            args: vec![Expr::Array(vec![
                Expr::Literal(Literal::Str("ls".to_string())),
                Expr::Literal(Literal::Str("-la".to_string())),
            ])],
        })]);
        assert!(p.validate_ast(&ast).is_ok(), "Array arg in exec() ok");
    }

    #[test]
    fn test_exec_ctx_array_arg_shellshock() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(Expr::FunctionCall {
            name: "exec".to_string(),
            args: vec![Expr::Array(vec![
                Expr::Literal(Literal::Str("safe".to_string())),
                Expr::Literal(Literal::Str("() { :; }".to_string())),
            ])],
        })]);
        assert!(
            p.validate_ast(&ast).is_err(),
            "Shellshock in array arg blocked"
        );
    }

    #[test]
    fn test_exec_ctx_index_arg() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(Expr::FunctionCall {
            name: "exec".to_string(),
            args: vec![Expr::Index {
                object: Box::new(Expr::Variable("arr".to_string())),
                index: Box::new(Expr::Literal(Literal::U32(0))),
            }],
        })]);
        assert!(p.validate_ast(&ast).is_ok(), "Index arg in exec() ok");
    }

    #[test]
    fn test_exec_ctx_try_arg() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(Expr::FunctionCall {
            name: "exec".to_string(),
            args: vec![Expr::Try {
                expr: Box::new(Expr::Variable("result".to_string())),
            }],
        })]);
        assert!(p.validate_ast(&ast).is_ok(), "Try arg in exec() ok");
    }

    #[test]
    fn test_exec_ctx_range_arg() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(Expr::FunctionCall {
            name: "exec".to_string(),
            args: vec![Expr::Range {
                start: Box::new(Expr::Literal(Literal::U32(0))),
                end: Box::new(Expr::Literal(Literal::U32(10))),
                inclusive: false,
            }],
        })]);
        assert!(p.validate_ast(&ast).is_ok(), "Range arg in exec() ok");
    }

    #[test]
    fn test_exec_ctx_positional_args() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(Expr::FunctionCall {
            name: "exec".to_string(),
            args: vec![Expr::PositionalArgs],
        })]);
        assert!(p.validate_ast(&ast).is_ok(), "PositionalArgs in exec() ok");
    }

    #[test]
    fn test_exec_ctx_method_call_arg() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(Expr::FunctionCall {
            name: "exec".to_string(),
            args: vec![Expr::MethodCall {
                receiver: Box::new(Expr::Variable("cmd".to_string())),
                method: "to_string".to_string(),
                args: vec![],
            }],
        })]);
        assert!(p.validate_ast(&ast).is_ok(), "MethodCall arg in exec() ok");
    }

    #[test]
    fn test_exec_ctx_method_call_empty_method_rejected() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(Expr::FunctionCall {
            name: "exec".to_string(),
            args: vec![Expr::MethodCall {
                receiver: Box::new(Expr::Variable("cmd".to_string())),
                method: String::new(),
                args: vec![],
            }],
        })]);
        assert!(
            p.validate_ast(&ast).is_err(),
            "Empty method name in exec() rejected"
        );
    }

    #[test]
    fn test_exec_ctx_block_arg() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(Expr::FunctionCall {
            name: "exec".to_string(),
            args: vec![Expr::Block(vec![Stmt::Let {
                name: "x".to_string(),
                value: Expr::Literal(Literal::U32(42)),
                declaration: true,
            }])],
        })]);
        assert!(p.validate_ast(&ast).is_ok(), "Block arg in exec() ok");
    }

    // ── None level bypasses everything ──

    #[test]
    fn test_none_level_bypasses_exec_context() {
        let p = none_pipeline();
        // Even shellshock passes at None level
        let ast = ast_with_body(vec![Stmt::Expr(exec_call("() { :; }; dangerous"))]);
        assert!(
            p.validate_ast(&ast).is_ok(),
            "None level bypasses all checks"
        );
    }

    // ── validate_ir exercises various IR nodes ──

    #[test]
    fn test_validate_ir_function_node() {
        let p = strict_pipeline();
        let ir = ShellIR::Function {
            name: "my_func".to_string(),
            params: vec![],
            body: Box::new(ShellIR::Noop),
        };
        assert!(p.validate_ir(&ir).is_ok());
    }

    #[test]
    fn test_validate_ir_break_continue() {
        let p = strict_pipeline();
        assert!(p.validate_ir(&ShellIR::Break).is_ok());
        assert!(p.validate_ir(&ShellIR::Continue).is_ok());
    }

    #[test]
    fn test_validate_ir_return_with_value() {
        let p = strict_pipeline();
        let ir = ShellIR::Return {
            value: Some(ShellValue::String("ok".to_string())),
        };
        assert!(p.validate_ir(&ir).is_ok());
    }

    #[test]
    fn test_validate_ir_return_without_value() {
        let p = strict_pipeline();
        let ir = ShellIR::Return { value: None };
        assert!(p.validate_ir(&ir).is_ok());
    }

    #[test]
    fn test_validate_ir_while_loop() {
        let p = strict_pipeline();
        let ir = ShellIR::While {
            condition: ShellValue::String("1".to_string()),
            body: Box::new(ShellIR::Noop),
        };
        assert!(p.validate_ir(&ir).is_ok());
    }

    #[test]
    fn test_validate_ir_for_in() {
        let p = strict_pipeline();
        let ir = ShellIR::ForIn {
            var: "x".to_string(),
            items: vec![
                ShellValue::String("a".to_string()),
                ShellValue::String("b".to_string()),
            ],
            body: Box::new(ShellIR::Noop),
        };
        assert!(p.validate_ir(&ir).is_ok());
    }

    #[test]
    fn test_validate_ir_for_in_empty_var_rejected() {
        let p = strict_pipeline();
        let ir = ShellIR::ForIn {
            var: String::new(),
            items: vec![],
            body: Box::new(ShellIR::Noop),
        };
        assert!(p.validate_ir(&ir).is_err(), "Empty for-in var rejected");
    }

    #[test]
    fn test_validate_ir_case_empty_arms_rejected() {
        let p = strict_pipeline();
        let ir = ShellIR::Case {
            scrutinee: ShellValue::String("x".to_string()),
            arms: vec![], // zero arms → error
        };
        assert!(
            p.validate_ir(&ir).is_err(),
            "Case with zero arms should fail"
        );
    }

    #[test]
    fn test_validate_ir_case_with_arm() {
        let p = strict_pipeline();
        let ir = ShellIR::Case {
            scrutinee: ShellValue::String("x".to_string()),
            arms: vec![CaseArm {
                pattern: CasePattern::Wildcard,
                guard: None,
                body: Box::new(ShellIR::Noop),
            }],
        };
        assert!(p.validate_ir(&ir).is_ok());
    }

    #[test]
    fn test_validate_ir_case_with_guard() {
        let p = strict_pipeline();
        let ir = ShellIR::Case {
            scrutinee: ShellValue::String("x".to_string()),
            arms: vec![CaseArm {
                pattern: CasePattern::Literal("1".to_string()),
                guard: Some(ShellValue::String("1".to_string())),
                body: Box::new(ShellIR::Noop),
            }],
        };
        assert!(p.validate_ir(&ir).is_ok());
    }

    #[test]
    fn test_validate_ir_echo() {
        let p = strict_pipeline();
        let ir = ShellIR::Echo {
            value: ShellValue::String("hello".to_string()),
        };
        assert!(p.validate_ir(&ir).is_ok());
    }

    // ── validate_ir with For loop ──

    #[test]
    fn test_validate_ir_for_empty_var_rejected() {
        let p = strict_pipeline();
        let ir = ShellIR::For {
            var: String::new(),
            start: ShellValue::String("0".to_string()),
            end: ShellValue::String("10".to_string()),
            body: Box::new(ShellIR::Noop),
        };
        assert!(p.validate_ir(&ir).is_err(), "Empty for-var rejected");
    }

    #[test]
    fn test_validate_ir_for_valid() {
        let p = strict_pipeline();
        let ir = ShellIR::For {
            var: "i".to_string(),
            start: ShellValue::String("0".to_string()),
            end: ShellValue::String("10".to_string()),
            body: Box::new(ShellIR::Noop),
        };
        assert!(p.validate_ir(&ir).is_ok());
    }

    // ── validate_shell_value: CommandSubst backtick check ──

    #[test]
    fn test_validate_shell_value_backtick_in_cmd_subst_rejected() {
        let p = minimal_pipeline();
        let val = ShellValue::CommandSubst(Command {
            program: "`whoami`".to_string(),
            args: vec![],
        });
        assert!(
            p.validate_shell_value(&val).is_err(),
            "Backtick in CommandSubst rejected"
        );
    }

    #[test]
    fn test_validate_shell_value_concat() {
        let p = strict_pipeline();
        let val = ShellValue::Concat(vec![
            ShellValue::String("hello".to_string()),
            ShellValue::Variable("name".to_string()),
        ]);
        assert!(p.validate_shell_value(&val).is_ok());
    }

    #[test]
    fn test_validate_shell_value_variable() {
        let p = strict_pipeline();
        let val = ShellValue::Variable("my_var".to_string());
        assert!(p.validate_shell_value(&val).is_ok());
    }

    // ── report_error and should_fail ──

    #[test]
    fn test_report_error_strict_mode() {
        use crate::validation::{Severity, ValidationError};
        let p = strict_pipeline();
        let err = ValidationError {
            rule: "TEST-001",
            severity: Severity::Error,
            message: "test error".to_string(),
            suggestion: None,
            auto_fix: None,
            line: None,
            column: None,
        };
        let report = p.report_error(&err);
        assert!(
            report.contains("ERROR"),
            "Strict mode errors should say ERROR"
        );
    }

    #[test]
    fn test_report_error_non_strict_warning() {
        use crate::validation::{Severity, ValidationError};
        let p = minimal_pipeline();
        let err = ValidationError {
            rule: "TEST-002",
            severity: Severity::Warning,
            message: "test warning".to_string(),
            suggestion: None,
            auto_fix: None,
            line: None,
            column: None,
        };
        let report = p.report_error(&err);
        assert!(
            report.contains("warning"),
            "Non-strict warns with warning label"
        );
    }

    #[test]
    fn test_should_fail_strict_mode_any_error() {
        use crate::validation::{Severity, ValidationError};
        let p = strict_pipeline();
        let errors = vec![ValidationError {
            rule: "TEST-001",
            severity: Severity::Warning,
            message: "test".to_string(),
            suggestion: None,
            auto_fix: None,
            line: None,
            column: None,
        }];
        assert!(p.should_fail(&errors), "Strict mode fails on any error");
    }

    #[test]
    fn test_should_fail_non_strict_only_on_error_severity() {
        use crate::validation::{Severity, ValidationError};
        let p = minimal_pipeline();
        let warnings = vec![ValidationError {
            rule: "TEST-001",
            severity: Severity::Warning,
            message: "warning".to_string(),
            suggestion: None,
            auto_fix: None,
            line: None,
            column: None,
        }];
        assert!(
            !p.should_fail(&warnings),
            "Non-strict mode: warnings don't fail"
        );

        let errors = vec![ValidationError {
            rule: "TEST-002",
            severity: Severity::Error,
            message: "error".to_string(),
            suggestion: None,
            auto_fix: None,
            line: None,
            column: None,
        }];
        assert!(p.should_fail(&errors), "Non-strict mode: errors do fail");
    }

    // ── validate_ast: full function traversal ──

    #[test]
    fn test_validate_ast_with_if_else_block() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::If {
            condition: Expr::Literal(Literal::Bool(true)),
            then_block: vec![Stmt::Expr(Expr::Variable("x".to_string()))],
            else_block: Some(vec![Stmt::Expr(Expr::Variable("y".to_string()))]),
        }]);
        assert!(p.validate_ast(&ast).is_ok());
    }

    #[test]
    fn test_validate_ast_function_name_reserved_builtin() {
        let p = strict_pipeline();
        let ast = RestrictedAst {
            functions: vec![Function {
                name: "exit".to_string(), // reserved
                params: vec![],
                return_type: Type::Void,
                body: vec![],
            }],
            entry_point: "exit".to_string(),
        };
        assert!(
            p.validate_ast(&ast).is_err(),
            "Reserved builtin 'exit' rejected"
        );
    }

    #[test]
    fn test_validate_ast_nested_if_in_then_and_else() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::If {
            condition: Expr::Literal(Literal::Bool(true)),
            then_block: vec![Stmt::If {
                condition: Expr::Literal(Literal::Bool(false)),
                then_block: vec![],
                else_block: None,
            }],
            else_block: Some(vec![Stmt::Expr(Expr::Variable("z".to_string()))]),
        }]);
        assert!(p.validate_ast(&ast).is_ok());
    }
}
