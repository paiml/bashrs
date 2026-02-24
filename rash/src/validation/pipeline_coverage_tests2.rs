//! Extended coverage tests for `validate_expr_in_exec_context` and related paths.
//!
//! Focuses on exercising branches not yet covered:
//! - Nested expression types in exec context (MethodCall with args, deeply nested)
//! - Range with shellshock in start/end
//! - Block with complex inner statements
//! - FunctionCall inside exec context that itself recurses to normal validate_expr
//! - Literal variants (Bool, U16, U32, I32) in exec context
//! - validate_literal_in_exec_context for each Literal variant

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

#[cfg(test)]
mod tests {
    use crate::ast::restricted::{BinaryOp, Literal, UnaryOp};
    use crate::ast::{Expr, Function, RestrictedAst, Stmt, Type};
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

    /// Build an exec() call with a single expression argument.
    fn exec_with_arg(arg: Expr) -> Expr {
        Expr::FunctionCall {
            name: "exec".to_string(),
            args: vec![arg],
        }
    }

    // ── Literal variants in exec context ──

    #[test]
    fn test_exec_ctx_all_literal_types_allowed() {
        let p = strict_pipeline();
        let literals = vec![
            Expr::Literal(Literal::Bool(true)),
            Expr::Literal(Literal::U16(42)),
            Expr::Literal(Literal::U32(100_000)),
            Expr::Literal(Literal::I32(-5)),
            Expr::Literal(Literal::Str("echo hello world".to_string())),
        ];
        for lit in literals {
            let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(lit))]);
            assert!(p.validate_ast(&ast).is_ok(), "Literal in exec() should be ok");
        }
    }

    // ── MethodCall in exec context with method args ──

    #[test]
    fn test_exec_ctx_method_call_with_args() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::MethodCall {
            receiver: Box::new(Expr::Variable("cmd".to_string())),
            method: "replace".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("old".to_string())),
                Expr::Literal(Literal::Str("new".to_string())),
            ],
        }))]);
        assert!(
            p.validate_ast(&ast).is_ok(),
            "MethodCall with args in exec() ok"
        );
    }

    #[test]
    fn test_exec_ctx_method_call_shellshock_in_arg() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::MethodCall {
            receiver: Box::new(Expr::Variable("cmd".to_string())),
            method: "replace".to_string(),
            args: vec![Expr::Literal(Literal::Str(
                "() { :; }; evil".to_string(),
            ))],
        }))]);
        // MethodCall args inside exec context go through validate_expr_in_exec_context
        // which for strings checks shellshock
        assert!(
            p.validate_ast(&ast).is_err(),
            "Shellshock in method arg in exec() should be blocked"
        );
    }

    #[test]
    fn test_exec_ctx_method_call_shellshock_in_receiver() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::MethodCall {
            receiver: Box::new(Expr::Literal(Literal::Str(
                "() { :; }; pwned".to_string(),
            ))),
            method: "trim".to_string(),
            args: vec![],
        }))]);
        assert!(
            p.validate_ast(&ast).is_err(),
            "Shellshock in receiver in exec() should be blocked"
        );
    }

    // ── Nested Binary inside exec context ──

    #[test]
    fn test_exec_ctx_nested_binary_deep() {
        let p = strict_pipeline();
        // (1 + (2 + (3 + 4)))
        let deep = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Literal(Literal::U32(1))),
            right: Box::new(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Literal(Literal::U32(2))),
                right: Box::new(Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Literal(Literal::U32(3))),
                    right: Box::new(Expr::Literal(Literal::U32(4))),
                }),
            }),
        };
        let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(deep))]);
        assert!(p.validate_ast(&ast).is_ok(), "Nested binary in exec() ok");
    }

    #[test]
    fn test_exec_ctx_binary_shellshock_in_right() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Literal(Literal::U32(1))),
            right: Box::new(Expr::Literal(Literal::Str("() { :; }".to_string()))),
        }))]);
        assert!(
            p.validate_ast(&ast).is_err(),
            "Shellshock in binary right in exec() blocked"
        );
    }

    // ── Unary with shellshock operand ──

    #[test]
    fn test_exec_ctx_unary_with_shellshock_operand() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(Expr::Literal(Literal::Str("() { :; }".to_string()))),
        }))]);
        assert!(
            p.validate_ast(&ast).is_err(),
            "Shellshock in unary operand in exec() blocked"
        );
    }

    // ── Array with mixed items in exec context ──

    #[test]
    fn test_exec_ctx_array_mixed_types() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::Array(vec![
            Expr::Literal(Literal::Bool(true)),
            Expr::Literal(Literal::U32(42)),
            Expr::Variable("x".to_string()),
            Expr::Literal(Literal::Str("safe".to_string())),
        ])))]);
        assert!(
            p.validate_ast(&ast).is_ok(),
            "Mixed array in exec() ok"
        );
    }

    #[test]
    fn test_exec_ctx_array_empty() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::Array(vec![])))]);
        assert!(p.validate_ast(&ast).is_ok(), "Empty array in exec() ok");
    }

    // ── Index with shellshock in object/index ──

    #[test]
    fn test_exec_ctx_index_shellshock_in_object_or_index() {
        let p = strict_pipeline();
        let ast1 = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::Index {
            object: Box::new(Expr::Literal(Literal::Str("() { :; }".to_string()))),
            index: Box::new(Expr::Literal(Literal::U32(0))),
        }))]);
        assert!(p.validate_ast(&ast1).is_err(), "Shellshock in index object blocked");

        let ast2 = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::Index {
            object: Box::new(Expr::Variable("arr".to_string())),
            index: Box::new(Expr::Literal(Literal::Str("() { :; }".to_string()))),
        }))]);
        assert!(p.validate_ast(&ast2).is_err(), "Shellshock in index expr blocked");
    }

    // ── Try with shellshock ──

    #[test]
    fn test_exec_ctx_try_with_shellshock() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::Try {
            expr: Box::new(Expr::Literal(Literal::Str("() { :; }".to_string()))),
        }))]);
        assert!(
            p.validate_ast(&ast).is_err(),
            "Shellshock in try in exec() blocked"
        );
    }

    // ── Range with shellshock in start/end ──

    #[test]
    fn test_exec_ctx_range_shellshock_and_safe() {
        let p = strict_pipeline();
        // Shellshock in start
        let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::Range {
            start: Box::new(Expr::Literal(Literal::Str("() { :; }".to_string()))),
            end: Box::new(Expr::Literal(Literal::U32(10))),
            inclusive: false,
        }))]);
        assert!(p.validate_ast(&ast).is_err(), "Shellshock in range start blocked");

        // Shellshock in end
        let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::Range {
            start: Box::new(Expr::Literal(Literal::U32(0))),
            end: Box::new(Expr::Literal(Literal::Str("() { :; }".to_string()))),
            inclusive: true,
        }))]);
        assert!(p.validate_ast(&ast).is_err(), "Shellshock in range end blocked");

        // Safe inclusive range
        let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::Range {
            start: Box::new(Expr::Literal(Literal::U32(0))),
            end: Box::new(Expr::Literal(Literal::U32(100))),
            inclusive: true,
        }))]);
        assert!(p.validate_ast(&ast).is_ok(), "Safe inclusive range ok");
    }

    // ── Block inside exec context ──

    #[test]
    fn test_exec_ctx_block_with_let_and_expr() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::Block(vec![
            Stmt::Let {
                name: "a".to_string(),
                value: Expr::Literal(Literal::U32(1)),
                declaration: true,
            },
            Stmt::Expr(Expr::Variable("a".to_string())),
        ])))]);
        assert!(
            p.validate_ast(&ast).is_ok(),
            "Block with let+expr in exec() ok"
        );
    }

    #[test]
    fn test_exec_ctx_block_with_if_statement() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::Block(vec![
            Stmt::If {
                condition: Expr::Literal(Literal::Bool(true)),
                then_block: vec![Stmt::Expr(Expr::Literal(Literal::Str(
                    "echo yes".to_string(),
                )))],
                else_block: Some(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                    "echo no".to_string(),
                )))]),
            },
        ])))]);
        assert!(
            p.validate_ast(&ast).is_ok(),
            "Block with if-else in exec() ok"
        );
    }

    #[test]
    fn test_exec_ctx_block_empty() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::Block(vec![])))]);
        assert!(p.validate_ast(&ast).is_ok(), "Empty block in exec() ok");
    }

    // ── FunctionCall inside exec context (non-exec name) ──

    #[test]
    fn test_exec_ctx_function_call_inner_validates_normally() {
        let p = strict_pipeline();
        // exec(some_fn("safe string"))
        let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::FunctionCall {
            name: "some_fn".to_string(),
            args: vec![Expr::Literal(Literal::Str("safe string".to_string()))],
        }))]);
        assert!(
            p.validate_ast(&ast).is_ok(),
            "Inner function call in exec() ok"
        );
    }

    #[test]
    fn test_exec_ctx_function_call_inner_with_injection() {
        let p = strict_pipeline();
        // exec(some_fn("$(whoami)")) - inner fn uses normal validation which blocks $()
        let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::FunctionCall {
            name: "some_fn".to_string(),
            args: vec![Expr::Literal(Literal::Str("$(whoami)".to_string()))],
        }))]);
        assert!(
            p.validate_ast(&ast).is_err(),
            "Command substitution in inner fn should be blocked"
        );
    }

    #[test]
    fn test_exec_ctx_function_call_empty_name_rejected() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::FunctionCall {
            name: String::new(),
            args: vec![],
        }))]);
        assert!(
            p.validate_ast(&ast).is_err(),
            "Empty function name in exec() rejected"
        );
    }

    // ── None level bypasses exec context validation ──

    #[test]
    fn test_none_level_exec_ctx_bypasses_all_checks() {
        let p = none_pipeline();
        for dangerous in &["() { :; }; dangerous", "`whoami`"] {
            let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::Literal(
                Literal::Str(dangerous.to_string()),
            )))]);
            assert!(p.validate_ast(&ast).is_ok(), "None level bypasses: {}", dangerous);
        }
    }

    // ── Exec context with command substitution blocked ──

    #[test]
    fn test_exec_ctx_cmd_subst_in_nested_exprs() {
        let p = strict_pipeline();
        // $() in binary
        let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Literal(Literal::Str("prefix".to_string()))),
            right: Box::new(Expr::Literal(Literal::Str("$(rm -rf /)".to_string()))),
        }))]);
        assert!(p.validate_ast(&ast).is_err(), "$() in binary blocked");

        // Backtick in unary
        let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::Unary {
            op: UnaryOp::Neg,
            operand: Box::new(Expr::Literal(Literal::Str("`id`".to_string()))),
        }))]);
        assert!(p.validate_ast(&ast).is_err(), "Backtick in unary blocked");
    }

    // ── Multiple exec args with varied expression types ──

    #[test]
    fn test_exec_ctx_multiple_diverse_args_all_safe() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(Expr::FunctionCall {
            name: "exec".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("echo".to_string())),
                Expr::Variable("msg".to_string()),
                Expr::Literal(Literal::U32(42)),
                Expr::Literal(Literal::Bool(false)),
                Expr::Array(vec![Expr::Literal(Literal::Str("a".to_string()))]),
                Expr::Range {
                    start: Box::new(Expr::Literal(Literal::U32(0))),
                    end: Box::new(Expr::Literal(Literal::U32(5))),
                    inclusive: false,
                },
                Expr::PositionalArgs,
            ],
        })]);
        assert!(
            p.validate_ast(&ast).is_ok(),
            "Multiple diverse safe args in exec() ok"
        );
    }

    #[test]
    fn test_exec_ctx_multiple_args_one_has_backtick() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(Expr::FunctionCall {
            name: "exec".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("safe1".to_string())),
                Expr::Literal(Literal::Str("safe2".to_string())),
                Expr::Literal(Literal::Str("`evil`".to_string())),
            ],
        })]);
        assert!(
            p.validate_ast(&ast).is_err(),
            "Backtick in one arg should block"
        );
    }

    // ── PositionalArgs variant in exec context ──

    #[test]
    fn test_exec_ctx_positional_args_alone() {
        let p = strict_pipeline();
        let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::PositionalArgs))]);
        assert!(
            p.validate_ast(&ast).is_ok(),
            "PositionalArgs in exec() ok"
        );
    }

    // ── Pipes and operators allowed in exec string ──

    #[test]
    fn test_exec_ctx_string_shell_operators_allowed() {
        let p = strict_pipeline();
        let allowed = vec![
            "cat /etc/passwd | grep root > /dev/null",
            "cd /tmp; ls -la; pwd",
            "cat <<EOF\nhello\nEOF",
        ];
        for cmd in allowed {
            let ast = ast_with_body(vec![Stmt::Expr(exec_with_arg(Expr::Literal(
                Literal::Str(cmd.to_string()),
            )))]);
            assert!(p.validate_ast(&ast).is_ok(), "Shell operator allowed in exec(): {}", cmd);
        }
    }
}
