
    #[test]
    fn test_lower_let_if_with_else() {
        // let x = if cond { "yes" } else { "no" }
        let ast = make_main(vec![
            Stmt::Let {
                name: "x".to_string(),
                value: Expr::Block(vec![Stmt::If {
                    condition: Expr::Literal(Literal::Bool(true)),
                    then_block: vec![Stmt::Expr(Expr::Literal(Literal::Str("yes".to_string())))],
                    else_block: Some(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                        "no".to_string(),
                    )))]),
                }]),
                declaration: true,
            },
        ]);
        let ir = from_ast(&ast).expect("let-if with else should work");
        assert_ir_is_sequence(&ir);
    }

    #[test]
    fn test_lower_let_if_without_else() {
        // let x = if cond { "yes" }  (no else)
        let ast = make_main(vec![
            Stmt::Let {
                name: "x".to_string(),
                value: Expr::Block(vec![Stmt::If {
                    condition: Expr::Literal(Literal::Bool(false)),
                    then_block: vec![Stmt::Expr(Expr::Literal(Literal::Str("yes".to_string())))],
                    else_block: None,
                }]),
                declaration: true,
            },
        ]);
        let ir = from_ast(&ast).expect("let-if without else should work");
        assert_ir_is_sequence(&ir);
    }

    // ── convert_range_match: range patterns → if-elif-else chain ──

    #[test]
    fn test_range_match_single_range_with_wildcard() {
        // match n { 0..10 => "low", _ => "high" }
        let ast = make_main(vec![Stmt::Match {
            scrutinee: Expr::Variable("n".to_string()),
            arms: vec![
                range_arm(
                    Literal::I32(0),
                    Literal::I32(10),
                    false,
                    vec![Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("low".to_string()))],
                    })],
                ),
                wildcard_arm(vec![Stmt::Expr(Expr::FunctionCall {
                    name: "echo".to_string(),
                    args: vec![Expr::Literal(Literal::Str("high".to_string()))],
                })]),
            ],
        }]);
        let ir = from_ast(&ast).expect("Range match should convert to if-chain");
        assert_ir_is_sequence(&ir);
    }

    #[test]
    fn test_range_match_multiple_ranges() {
        // match n { 0..5 => "low", 5..=10 => "mid", _ => "high" }
        let ast = make_main(vec![Stmt::Match {
            scrutinee: Expr::Variable("n".to_string()),
            arms: vec![
                range_arm(
                    Literal::I32(0),
                    Literal::I32(5),
                    false,
                    vec![Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("low".to_string()))],
                    })],
                ),
                range_arm(
                    Literal::I32(5),
                    Literal::I32(10),
                    true,
                    vec![Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("mid".to_string()))],
                    })],
                ),
                wildcard_arm(vec![Stmt::Expr(Expr::FunctionCall {
                    name: "echo".to_string(),
                    args: vec![Expr::Literal(Literal::Str("high".to_string()))],
                })]),
            ],
        }]);
        let ir = from_ast(&ast).expect("Multi-range match should convert");
        assert_ir_is_sequence(&ir);
    }

    #[test]
    fn test_range_match_only_wildcard_produces_else_body() {
        // All arms are wildcards → all become else bodies
        let ast = make_main(vec![Stmt::Match {
            scrutinee: Expr::Variable("n".to_string()),
            arms: vec![wildcard_arm(vec![Stmt::Expr(Expr::FunctionCall {
                name: "echo".to_string(),
                args: vec![Expr::Literal(Literal::Str("any".to_string()))],
            })])],
        }]);
        let ir = from_ast(&ast).expect("Wildcard-only match should work");
        assert_ir_is_sequence(&ir);
    }

    // ── convert_range_match_fn: range patterns in function context ──

    #[test]
    fn test_range_match_in_function_context() {
        // Non-void function with range match
        let ast = RestrictedAst {
            functions: vec![Function {
                name: "classify".to_string(),
                params: vec![Parameter {
                    name: "n".to_string(),
                    param_type: Type::U32,
                }],
                return_type: Type::Str,
                body: vec![Stmt::Match {
                    scrutinee: Expr::Variable("n".to_string()),
                    arms: vec![
                        range_arm(
                            Literal::I32(0),
                            Literal::I32(10),
                            false,
                            vec![Stmt::Return(Some(Expr::Literal(Literal::Str(
                                "low".to_string(),
                            ))))],
                        ),
                        wildcard_arm(vec![Stmt::Return(Some(Expr::Literal(Literal::Str(
                            "high".to_string(),
                        ))))]),
                    ],
                }],
            },
            Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Expr(Expr::FunctionCall {
                    name: "classify".to_string(),
                    args: vec![Expr::Literal(Literal::I32(5))],
                })],
            }],
            entry_point: "main".to_string(),
        };
        let ir = from_ast(&ast).expect("Range match in function should convert");
        assert_ir_is_sequence(&ir);
    }

    // ── lower_let_if_expr: __if_expr lowering ──

    #[test]
    fn test_lower_let_if_expr_simple() {
        // let x = __if_expr(cond, "yes", "no")
        let ast = make_main(vec![Stmt::Let {
            name: "x".to_string(),
            value: Expr::FunctionCall {
                name: "__if_expr".to_string(),
                args: vec![
                    Expr::Literal(Literal::Bool(true)),
                    Expr::Literal(Literal::Str("yes".to_string())),
                    Expr::Literal(Literal::Str("no".to_string())),
                ],
            },
            declaration: true,
        }]);
        let ir = from_ast(&ast).expect("__if_expr should lower to If IR");
        assert_ir_is_sequence(&ir);
    }

    #[test]
    fn test_lower_let_if_expr_nested_then() {
        // let x = __if_expr(c1, __if_expr(c2, "a", "b"), "c")
        let ast = make_main(vec![Stmt::Let {
            name: "x".to_string(),
            value: Expr::FunctionCall {
                name: "__if_expr".to_string(),
                args: vec![
                    Expr::Literal(Literal::Bool(true)),
                    Expr::FunctionCall {
                        name: "__if_expr".to_string(),
                        args: vec![
                            Expr::Literal(Literal::Bool(false)),
                            Expr::Literal(Literal::Str("a".to_string())),
                            Expr::Literal(Literal::Str("b".to_string())),
                        ],
                    },
                    Expr::Literal(Literal::Str("c".to_string())),
                ],
            },
            declaration: true,
        }]);
        let ir = from_ast(&ast).expect("Nested __if_expr in then should work");
        assert_ir_is_sequence(&ir);
    }

    #[test]
    fn test_lower_let_if_expr_nested_else() {
        // let x = __if_expr(c1, "a", __if_expr(c2, "b", "c"))
        let ast = make_main(vec![Stmt::Let {
            name: "x".to_string(),
            value: Expr::FunctionCall {
                name: "__if_expr".to_string(),
                args: vec![
                    Expr::Literal(Literal::Bool(true)),
                    Expr::Literal(Literal::Str("a".to_string())),
                    Expr::FunctionCall {
                        name: "__if_expr".to_string(),
                        args: vec![
                            Expr::Literal(Literal::Bool(false)),
                            Expr::Literal(Literal::Str("b".to_string())),
                            Expr::Literal(Literal::Str("c".to_string())),
                        ],
                    },
                ],
            },
            declaration: true,
        }]);
        let ir = from_ast(&ast).expect("Nested __if_expr in else (elif chain) should work");
        assert_ir_is_sequence(&ir);
    }

    #[test]
    fn test_lower_let_if_expr_non_if_fn_in_then() {
        // let x = __if_expr(c1, other_fn(args), "c")  — non-__if_expr fn in then
        let ast = make_main(vec![Stmt::Let {
            name: "x".to_string(),
            value: Expr::FunctionCall {
                name: "__if_expr".to_string(),
                args: vec![
                    Expr::Literal(Literal::Bool(true)),
                    Expr::FunctionCall {
                        name: "other_fn".to_string(),
                        args: vec![Expr::Literal(Literal::Str("arg".to_string()))],
                    },
                    Expr::Literal(Literal::Str("fallback".to_string())),
                ],
            },
            declaration: true,
        }]);
        let ir = from_ast(&ast).expect("Non-__if_expr fn in then branch should work");
        assert_ir_is_sequence(&ir);
    }

    #[test]
    fn test_lower_let_if_expr_non_if_fn_in_else() {
        // let x = __if_expr(c1, "a", other_fn(args))  — non-__if_expr fn in else
        let ast = make_main(vec![Stmt::Let {
            name: "x".to_string(),
            value: Expr::FunctionCall {
                name: "__if_expr".to_string(),
                args: vec![
                    Expr::Literal(Literal::Bool(true)),
                    Expr::Literal(Literal::Str("a".to_string())),
                    Expr::FunctionCall {
                        name: "other_fn".to_string(),
                        args: vec![Expr::Literal(Literal::Str("arg".to_string()))],
                    },
                ],
            },
            declaration: true,
        }]);
        let ir = from_ast(&ast).expect("Non-__if_expr fn in else branch should work");
        assert_ir_is_sequence(&ir);
    }

    // ── lower_return_if_expr ──

    #[test]
    fn test_lower_return_if_expr_simple() {
        // return __if_expr(cond, 1, 0)
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
                        Expr::Literal(Literal::U32(0)),
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
        let ir = from_ast(&ast).expect("return __if_expr should lower to If IR");
        assert_ir_is_sequence(&ir);
    }

    #[test]
    fn test_lower_return_if_expr_nested_then() {
        // return __if_expr(c1, __if_expr(c2, 1, 2), 3)
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
                            name: "__if_expr".to_string(),
                            args: vec![
                                Expr::Literal(Literal::Bool(false)),
                                Expr::Literal(Literal::U32(1)),
                                Expr::Literal(Literal::U32(2)),
                            ],
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
        let ir = from_ast(&ast).expect("Nested return __if_expr in then should work");
        assert_ir_is_sequence(&ir);
    }

    #[test]
    fn test_lower_return_if_expr_nested_else() {
        // return __if_expr(c1, 1, __if_expr(c2, 2, 3))
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
                            name: "__if_expr".to_string(),
                            args: vec![
                                Expr::Literal(Literal::Bool(false)),
                                Expr::Literal(Literal::U32(2)),
                                Expr::Literal(Literal::U32(3)),
                            ],
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
        let ir = from_ast(&ast).expect("Nested return __if_expr in else (elif) should work");
        assert_ir_is_sequence(&ir);
    }

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
