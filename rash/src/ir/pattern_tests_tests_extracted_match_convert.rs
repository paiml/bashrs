
    #[test]
    fn test_convert_match_arm_for_let_single_if() {
        let outer_arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::I32(1)),
                guard: None,
                // Arm body is an if statement
                body: vec![Stmt::If {
                    condition: Expr::Literal(Literal::Bool(true)),
                    then_block: vec![Stmt::Expr(Expr::Literal(Literal::Str("yes".to_string())))],
                    else_block: Some(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                        "no".to_string(),
                    )))]),
                }],
            },
            wildcard_arm(vec![Stmt::Expr(Expr::Literal(Literal::Str("other".to_string())))]),
        ];
        let ast = make_main(vec![
            Stmt::Let {
                name: "n".to_string(),
                value: Expr::Literal(Literal::I32(1)),
                declaration: true,
            },
            let_match("r", Expr::Variable("n".to_string()), outer_arms),
        ]);
        let ir = from_ast(&ast).expect("If arm body should work");
        assert_ir_is_sequence(&ir);
    }

    // ── convert_match_arm_for_let: multi-statement body ──

    #[test]
    fn test_convert_match_arm_for_let_multi_stmt_expr_last() {
        // Multi-stmt arm body ending with Expr
        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::I32(1)),
                guard: None,
                body: vec![
                    Stmt::Let {
                        name: "tmp".to_string(),
                        value: Expr::Literal(Literal::Str("temp".to_string())),
                        declaration: true,
                    },
                    Stmt::Expr(Expr::Literal(Literal::Str("result".to_string()))),
                ],
            },
            wildcard_arm(vec![Stmt::Expr(Expr::Literal(Literal::Str("other".to_string())))]),
        ];
        let ast = make_main(vec![
            Stmt::Let {
                name: "n".to_string(),
                value: Expr::Literal(Literal::I32(1)),
                declaration: true,
            },
            let_match("r", Expr::Variable("n".to_string()), arms),
        ]);
        let ir = from_ast(&ast).expect("Multi-stmt arm ending with Expr should work");
        assert_ir_is_sequence(&ir);
    }

    #[test]
    fn test_convert_match_arm_for_let_multi_stmt_return_last() {
        // Multi-stmt arm body ending with Return
        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::I32(1)),
                guard: None,
                body: vec![
                    Stmt::Let {
                        name: "tmp".to_string(),
                        value: Expr::Literal(Literal::Str("x".to_string())),
                        declaration: true,
                    },
                    Stmt::Return(Some(Expr::Literal(Literal::Str("done".to_string())))),
                ],
            },
            wildcard_arm(vec![Stmt::Return(Some(Expr::Literal(Literal::Str(
                "default".to_string(),
            ))))]),
        ];
        let ast = make_main(vec![
            Stmt::Let {
                name: "n".to_string(),
                value: Expr::Literal(Literal::I32(1)),
                declaration: true,
            },
            let_match("r", Expr::Variable("n".to_string()), arms),
        ]);
        let ir = from_ast(&ast).expect("Multi-stmt ending with Return should work");
        assert_ir_is_sequence(&ir);
    }

    #[test]
    fn test_convert_match_arm_for_let_multi_stmt_match_last() {
        // Multi-stmt arm body ending with a Match
        let inner_arms = vec![wildcard_arm(vec![Stmt::Expr(Expr::Literal(Literal::Str(
            "inner".to_string(),
        )))])];
        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::I32(1)),
                guard: None,
                body: vec![
                    Stmt::Let {
                        name: "tmp".to_string(),
                        value: Expr::Literal(Literal::Str("x".to_string())),
                        declaration: true,
                    },
                    Stmt::Match {
                        scrutinee: Expr::Variable("tmp".to_string()),
                        arms: inner_arms,
                    },
                ],
            },
            wildcard_arm(vec![Stmt::Expr(Expr::Literal(Literal::Str("other".to_string())))]),
        ];
        let ast = make_main(vec![
            Stmt::Let {
                name: "n".to_string(),
                value: Expr::Literal(Literal::I32(1)),
                declaration: true,
            },
            Stmt::Let {
                name: "tmp".to_string(),
                value: Expr::Literal(Literal::Str("x".to_string())),
                declaration: true,
            },
            let_match("r", Expr::Variable("n".to_string()), arms),
        ]);
        let ir = from_ast(&ast).expect("Multi-stmt ending with Match should work");
        assert_ir_is_sequence(&ir);
    }

    #[test]
    fn test_convert_match_arm_for_let_multi_stmt_if_last() {
        // Multi-stmt arm body ending with If
        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::I32(1)),
                guard: None,
                body: vec![
                    Stmt::Let {
                        name: "tmp".to_string(),
                        value: Expr::Literal(Literal::Str("x".to_string())),
                        declaration: true,
                    },
                    Stmt::If {
                        condition: Expr::Literal(Literal::Bool(true)),
                        then_block: vec![Stmt::Expr(Expr::Literal(Literal::Str(
                            "t".to_string(),
                        )))],
                        else_block: Some(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                            "f".to_string(),
                        )))]),
                    },
                ],
            },
            wildcard_arm(vec![Stmt::Expr(Expr::Literal(Literal::Str("other".to_string())))]),
        ];
        let ast = make_main(vec![
            Stmt::Let {
                name: "n".to_string(),
                value: Expr::Literal(Literal::I32(1)),
                declaration: true,
            },
            let_match("r", Expr::Variable("n".to_string()), arms),
        ]);
        let ir = from_ast(&ast).expect("Multi-stmt ending with If should work");
        assert_ir_is_sequence(&ir);
    }

    #[test]
    fn test_convert_match_arm_for_let_multi_stmt_other_last() {
        // Multi-stmt arm body ending with a non-special stmt (e.g., Break)
        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::I32(1)),
                guard: None,
                body: vec![
                    Stmt::Expr(Expr::Literal(Literal::Str("first".to_string()))),
                    Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("done".to_string()))],
                    }),
                ],
            },
            wildcard_arm(vec![Stmt::Expr(Expr::Literal(Literal::Str("other".to_string())))]),
        ];
        let ast = make_main(vec![
            Stmt::Let {
                name: "n".to_string(),
                value: Expr::Literal(Literal::I32(1)),
                declaration: true,
            },
            let_match("r", Expr::Variable("n".to_string()), arms),
        ]);
        let ir = from_ast(&ast).expect("Multi-stmt other-last should work");
        assert_ir_is_sequence(&ir);
    }

    // ── lower_let_if: with and without else ──

include!("pattern_tests_tests_extracted_match_convert_lower.rs");
