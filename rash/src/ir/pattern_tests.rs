//! Coverage tests for `rash/src/ir/pattern.rs`.
//!
//! Tests `convert_match_pattern`, `lower_let_match`, `convert_match_arm_for_let`,
//! `lower_let_if`, `has_range_patterns`, `convert_range_match`, `lower_let_if_expr`,
//! `lower_return_if_expr`, and related helpers.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

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
}

