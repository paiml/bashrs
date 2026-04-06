//! Coverage tests for `validate_expr_in_exec_context` exercised via the public API.
//!
//! Since `validate_expr_in_exec_context` is private, we exercise it by calling
//! `validate_expr` on a `FunctionCall { name: "exec", args: [...] }` expression,
//! which internally dispatches to `validate_expr_in_exec_context` for each arg.
//! We also test via `validate_ast` and `validate_ir`.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

#[cfg(test)]
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
