
    #[test]
    fn test_match_variable_pattern_treated_as_wildcard() {
        // Variable patterns are treated as wildcards in current implementation
        let ast = make_main(vec![Stmt::Match {
            scrutinee: Expr::Variable("x".to_string()),
            arms: vec![MatchArm {
                pattern: Pattern::Variable("v".to_string()),
                guard: None,
                body: vec![Stmt::Expr(Expr::FunctionCall {
                    name: "echo".to_string(),
                    args: vec![Expr::Variable("v".to_string())],
                })],
            }],
        }]);
        let ir = from_ast(&ast).expect("Variable pattern should be ok (treated as wildcard)");
        assert_ir_is_sequence(&ir);
    }

    // ── convert_match_pattern: Range ──

    #[test]
    fn test_match_range_pattern_exclusive() {
        let ast = make_main(vec![Stmt::Match {
            scrutinee: Expr::Variable("n".to_string()),
            arms: vec![
                range_arm(
                    Literal::I32(0),
                    Literal::I32(10),
                    false,
                    vec![Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("0..10".to_string()))],
                    })],
                ),
                wildcard_arm(vec![]),
            ],
        }]);
        let ir = from_ast(&ast).expect("Range pattern (exclusive) should convert");
        assert_ir_is_sequence(&ir);
    }

    #[test]
    fn test_match_range_pattern_inclusive() {
        let ast = make_main(vec![Stmt::Match {
            scrutinee: Expr::Variable("n".to_string()),
            arms: vec![
                range_arm(
                    Literal::I32(1),
                    Literal::I32(5),
                    true,
                    vec![Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("1..=5".to_string()))],
                    })],
                ),
                wildcard_arm(vec![]),
            ],
        }]);
        let ir = from_ast(&ast).expect("Range pattern (inclusive) should convert");
        assert_ir_is_sequence(&ir);
    }

    // ── convert_match_pattern: Struct/Tuple → error ──

    #[test]
    fn test_match_struct_pattern_returns_error() {
        let ast = make_main(vec![Stmt::Match {
            scrutinee: Expr::Variable("x".to_string()),
            arms: vec![MatchArm {
                pattern: Pattern::Struct {
                    name: "Foo".to_string(),
                    fields: vec![("a".to_string(), Pattern::Wildcard)],
                },
                guard: None,
                body: vec![],
            }],
        }]);
        let result = from_ast(&ast);
        assert!(result.is_err(), "Struct pattern should produce error");
    }

    #[test]
    fn test_match_tuple_pattern_returns_error() {
        let ast = make_main(vec![Stmt::Match {
            scrutinee: Expr::Variable("x".to_string()),
            arms: vec![MatchArm {
                pattern: Pattern::Tuple(vec![Pattern::Wildcard]),
                guard: None,
                body: vec![],
            }],
        }]);
        let result = from_ast(&ast);
        assert!(result.is_err(), "Tuple pattern should produce error");
    }

    // ── lower_let_match: match arm with guard ──

    #[test]
    fn test_lower_let_match_with_guard() {
        // Build: let result = match x { v if v > 0 => "pos", _ => "other" }
        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::I32(1)),
                guard: Some(Expr::Binary {
                    op: BinaryOp::Gt,
                    left: Box::new(Expr::Variable("x".to_string())),
                    right: Box::new(Expr::Literal(Literal::I32(0))),
                }),
                body: vec![Stmt::Expr(Expr::Literal(Literal::Str("pos".to_string())))],
            },
            wildcard_arm(vec![Stmt::Expr(Expr::Literal(Literal::Str("other".to_string())))]),
        ];
        let ast = make_main(vec![
            Stmt::Let {
                name: "x".to_string(),
                value: Expr::Literal(Literal::I32(5)),
                declaration: true,
            },
            let_match("result", Expr::Variable("x".to_string()), arms),
        ]);
        let ir = from_ast(&ast).expect("let-match with guard should convert");
        assert_ir_is_sequence(&ir);
    }

    // ── convert_match_arm_for_let: empty body ──

    #[test]
    fn test_convert_match_arm_for_let_empty_body() {
        // An arm with no statements → default let target = "0"
        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::I32(1)),
                guard: None,
                body: vec![], // empty body
            },
            wildcard_arm(vec![Stmt::Expr(Expr::Literal(Literal::Str("x".to_string())))]),
        ];
        let ast = make_main(vec![
            Stmt::Let {
                name: "n".to_string(),
                value: Expr::Literal(Literal::I32(1)),
                declaration: true,
            },
            let_match("r", Expr::Variable("n".to_string()), arms),
        ]);
        let ir = from_ast(&ast).expect("Empty arm body should give default");
        assert_ir_is_sequence(&ir);
    }

    // ── convert_match_arm_for_let: single Return ──

    #[test]
    fn test_convert_match_arm_for_let_single_return() {
        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::I32(1)),
                guard: None,
                body: vec![Stmt::Return(Some(Expr::Literal(Literal::Str(
                    "one".to_string(),
                ))))],
            },
            wildcard_arm(vec![Stmt::Return(Some(Expr::Literal(Literal::Str(
                "other".to_string(),
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
        let ir = from_ast(&ast).expect("Return arm body should work");
        assert_ir_is_sequence(&ir);
    }

    // ── convert_match_arm_for_let: single nested Match ──

    #[test]
    fn test_convert_match_arm_for_let_nested_match() {
        // Outer match: arm body is another match stmt
        let inner_arms = vec![
            lit_arm(
                Literal::I32(0),
                vec![Stmt::Expr(Expr::Literal(Literal::Str("zero".to_string())))],
            ),
            wildcard_arm(vec![Stmt::Expr(Expr::Literal(Literal::Str("pos".to_string())))]),
        ];
        let outer_arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::I32(1)),
                guard: None,
                body: vec![Stmt::Match {
                    scrutinee: Expr::Variable("y".to_string()),
                    arms: inner_arms,
                }],
            },
            wildcard_arm(vec![Stmt::Expr(Expr::Literal(Literal::Str("big".to_string())))]),
        ];
        let ast = make_main(vec![
            Stmt::Let {
                name: "x".to_string(),
                value: Expr::Literal(Literal::I32(1)),
                declaration: true,
            },
            Stmt::Let {
                name: "y".to_string(),
                value: Expr::Literal(Literal::I32(0)),
                declaration: true,
            },
            let_match("r", Expr::Variable("x".to_string()), outer_arms),
        ]);
        let ir = from_ast(&ast).expect("Nested match arm body should work");
        assert_ir_is_sequence(&ir);
    }

    // ── convert_match_arm_for_let: single If ──

include!("pattern_tests_tests_extracted_match_convert.rs");
