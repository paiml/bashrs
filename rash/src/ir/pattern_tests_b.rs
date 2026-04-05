//! Coverage tests for `rash/src/ir/pattern.rs`.
//!
//! Tests `convert_match_pattern`, `lower_let_match`, `convert_match_arm_for_let`,
//! `lower_let_if`, `has_range_patterns`, `convert_range_match`, `lower_let_if_expr`,
//! `lower_return_if_expr`, and related helpers.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

#[cfg(test)]
    #[test]
    fn test_lower_return_if_expr_non_if_fn_in_then() {
        // return __if_expr(c1, other_fn(), 3)  — non-__if_expr in then
        let ast = RestrictedAst {
            functions: vec![Function {
                name: "get_val".to_string(),
                params: vec![],
                return_type: Type::U32,
                body: vec![Stmt::Return(Some(Expr::FunctionCall {
                    name: "__if_expr".to_string(),
                    args: vec![
                        Expr::Literal(Literal::Bool(true)),
                        Expr::FunctionCall {
                            name: "compute".to_string(),
                            args: vec![],
                        },
                        Expr::Literal(Literal::U32(3)),
                    ],
                }))],
            },
            Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Expr(Expr::FunctionCall {
                    name: "get_val".to_string(),
                    args: vec![],
                })],
            }],
            entry_point: "main".to_string(),
        };
        let ir = from_ast(&ast).expect("Non-__if_expr fn in return then should work");
        assert_ir_is_sequence(&ir);
    }

    #[test]
    fn test_lower_return_if_expr_non_if_fn_in_else() {
        // return __if_expr(c1, 1, other_fn())  — non-__if_expr in else
        let ast = RestrictedAst {
            functions: vec![Function {
                name: "get_val".to_string(),
                params: vec![],
                return_type: Type::U32,
                body: vec![Stmt::Return(Some(Expr::FunctionCall {
                    name: "__if_expr".to_string(),
                    args: vec![
                        Expr::Literal(Literal::Bool(true)),
                        Expr::Literal(Literal::U32(1)),
                        Expr::FunctionCall {
                            name: "compute".to_string(),
                            args: vec![],
                        },
                    ],
                }))],
            },
            Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Expr(Expr::FunctionCall {
                    name: "get_val".to_string(),
                    args: vec![],
                })],
            }],
            entry_point: "main".to_string(),
        };
        let ir = from_ast(&ast).expect("Non-__if_expr fn in return else should work");
        assert_ir_is_sequence(&ir);
    }

    // ── convert_range_match_for_let: range patterns in let binding ──

    #[test]
    fn test_range_match_for_let() {
        // let grade = match score { 90..=100 => "A", 80..=89 => "B", _ => "C" }
        let arms = vec![
            range_arm(
                Literal::I32(90),
                Literal::I32(100),
                true,
                vec![Stmt::Expr(Expr::Literal(Literal::Str("A".to_string())))],
            ),
            range_arm(
                Literal::I32(80),
                Literal::I32(89),
                true,
                vec![Stmt::Expr(Expr::Literal(Literal::Str("B".to_string())))],
            ),
            wildcard_arm(vec![Stmt::Expr(Expr::Literal(Literal::Str("C".to_string())))]),
        ];
        let ast = make_main(vec![
            Stmt::Let {
                name: "score".to_string(),
                value: Expr::Literal(Literal::I32(85)),
                declaration: true,
            },
            let_match("grade", Expr::Variable("score".to_string()), arms),
        ]);
        let ir = from_ast(&ast).expect("Range match for let should convert");
        assert_ir_is_sequence(&ir);
    }

    // ── Assertion helpers ──

    fn assert_ir_is_sequence(ir: &ShellIR) {
        match ir {
            ShellIR::Sequence(_) => {}
            other => panic!("Expected ShellIR::Sequence, got {:?}", other),
        }
    }
}

