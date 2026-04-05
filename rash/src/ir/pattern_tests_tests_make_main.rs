#[cfg(test)]
mod tests {
    use crate::ast::restricted::{BinaryOp, Function, Literal, MatchArm, Parameter, Pattern, Type};
    use crate::ast::{Expr, RestrictedAst, Stmt};
    use crate::ir::{from_ast, shell_ir::ShellIR};
    #[allow(unused_imports)]
    use crate::ir::{EffectSet, ShellValue};

    // ── Helpers ──

    fn make_main(body: Vec<Stmt>) -> RestrictedAst {
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

    // (make_main_with_return is unused; removed)

    /// Build a `let x = match scrutinee { arms }` statement.
    fn let_match(target: &str, scrutinee: Expr, arms: Vec<MatchArm>) -> Stmt {
        // We encode this as a Let whose value triggers lower_let_match.
        // In the AST, let-match is represented as a Block containing a Match stmt.
        Stmt::Let {
            name: target.to_string(),
            value: Expr::Block(vec![Stmt::Match { scrutinee, arms }]),
            declaration: true,
        }
    }

    fn wildcard_arm(body: Vec<Stmt>) -> MatchArm {
        MatchArm {
            pattern: Pattern::Wildcard,
            guard: None,
            body,
        }
    }

    fn lit_arm(lit: Literal, body: Vec<Stmt>) -> MatchArm {
        MatchArm {
            pattern: Pattern::Literal(lit),
            guard: None,
            body,
        }
    }

    fn range_arm(start: Literal, end: Literal, inclusive: bool, body: Vec<Stmt>) -> MatchArm {
        MatchArm {
            pattern: Pattern::Range {
                start,
                end,
                inclusive,
            },
            guard: None,
            body,
        }
    }

    // ── has_range_patterns (tested indirectly via from_ast) ──
    // IrConverter is private; has_range_patterns is exercised by the range match tests below.

    // ── convert_match_pattern: Literal variants ──

    #[test]
    fn test_match_literal_bool_true() {
        // Match on bool true → Case with Literal("true") pattern
        let ast = make_main(vec![Stmt::Match {
            scrutinee: Expr::Variable("x".to_string()),
            arms: vec![
                lit_arm(
                    Literal::Bool(true),
                    vec![Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("yes".to_string()))],
                    })],
                ),
                wildcard_arm(vec![Stmt::Expr(Expr::FunctionCall {
                    name: "echo".to_string(),
                    args: vec![Expr::Literal(Literal::Str("no".to_string()))],
                })]),
            ],
        }]);
        let ir = from_ast(&ast).expect("Should convert bool match");
        assert_ir_is_sequence(&ir);
    }

    #[test]
    fn test_match_literal_bool_false() {
        let ast = make_main(vec![Stmt::Match {
            scrutinee: Expr::Variable("x".to_string()),
            arms: vec![
                lit_arm(
                    Literal::Bool(false),
                    vec![Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("false branch".to_string()))],
                    })],
                ),
                wildcard_arm(vec![]),
            ],
        }]);
        let ir = from_ast(&ast).expect("Should convert bool-false match");
        assert_ir_is_sequence(&ir);
    }

    #[test]
    fn test_match_literal_u16() {
        let ast = make_main(vec![Stmt::Match {
            scrutinee: Expr::Variable("n".to_string()),
            arms: vec![
                lit_arm(
                    Literal::U16(0),
                    vec![Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("zero".to_string()))],
                    })],
                ),
                wildcard_arm(vec![]),
            ],
        }]);
        let ir = from_ast(&ast).expect("Should convert u16 match");
        assert_ir_is_sequence(&ir);
    }

    #[test]
    fn test_match_literal_u32() {
        let ast = make_main(vec![Stmt::Match {
            scrutinee: Expr::Variable("n".to_string()),
            arms: vec![
                lit_arm(
                    Literal::U32(42),
                    vec![Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("forty-two".to_string()))],
                    })],
                ),
                wildcard_arm(vec![]),
            ],
        }]);
        let ir = from_ast(&ast).expect("Should convert u32 match");
        assert_ir_is_sequence(&ir);
    }

    #[test]
    fn test_match_literal_i32() {
        let ast = make_main(vec![Stmt::Match {
            scrutinee: Expr::Variable("n".to_string()),
            arms: vec![
                lit_arm(
                    Literal::I32(-1),
                    vec![Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("neg".to_string()))],
                    })],
                ),
                wildcard_arm(vec![]),
            ],
        }]);
        let ir = from_ast(&ast).expect("Should convert i32 match");
        assert_ir_is_sequence(&ir);
    }

    #[test]
    fn test_match_literal_str() {
        let ast = make_main(vec![Stmt::Match {
            scrutinee: Expr::Variable("s".to_string()),
            arms: vec![
                lit_arm(
                    Literal::Str("hello".to_string()),
                    vec![Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("got hello".to_string()))],
                    })],
                ),
                wildcard_arm(vec![]),
            ],
        }]);
        let ir = from_ast(&ast).expect("Should convert string match");
        assert_ir_is_sequence(&ir);
    }

    // ── convert_match_pattern: Wildcard and Variable ──

    #[test]
    fn test_match_wildcard_pattern() {
        let ast = make_main(vec![Stmt::Match {
            scrutinee: Expr::Variable("x".to_string()),
            arms: vec![wildcard_arm(vec![Stmt::Expr(Expr::FunctionCall {
                name: "echo".to_string(),
                args: vec![Expr::Literal(Literal::Str("any".to_string()))],
            })])],
        }]);
        let ir = from_ast(&ast).expect("Should convert wildcard match");
        assert_ir_is_sequence(&ir);
    }

include!("pattern_tests_tests_extracted_match.rs");
