fn test_fn_context_match_with_should_echo() {
    let ast = make_with_fn(
        "cls",
        vec![Parameter {
            name: "n".into(),
            param_type: Type::U32,
        }],
        Type::Str,
        vec![Stmt::Match {
            scrutinee: Expr::Variable("n".into()),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Literal(Literal::U32(0)),
                    guard: None,
                    body: vec![Stmt::Expr(Expr::Literal(Literal::Str("zero".into())))],
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: vec![Stmt::Expr(Expr::Literal(Literal::Str("other".into())))],
                },
            ],
        }],
        vec![Stmt::Expr(Expr::FunctionCall {
            name: "cls".into(),
            args: vec![Expr::Literal(Literal::U32(1))],
        })],
    );
    assert_seq(&from_ast(&ast).unwrap());
}

// ============================================================================
// convert_expr dispatch: exec(), __format_concat, non-fn expressions
// ============================================================================

#[test]
fn test_exec_and_format_concat_and_noop() {
    // exec() -> eval
    let ast = make_main(vec![Stmt::Expr(Expr::FunctionCall {
        name: "exec".into(),
        args: vec![Expr::Literal(Literal::Str("ls".into()))],
    })]);
    assert_seq(&from_ast(&ast).unwrap());

    // __format_concat at expr level -> noop
    let ast2 = make_main(vec![Stmt::Expr(Expr::FunctionCall {
        name: "__format_concat".into(),
        args: vec![
            Expr::Literal(Literal::Str("hi ".into())),
            Expr::Variable("n".into()),
        ],
    })]);
    assert_seq(&from_ast(&ast2).unwrap());

    // Variable at stmt level -> noop
    let ast3 = make_main(vec![Stmt::Expr(Expr::Variable("x".into()))]);
    assert_seq(&from_ast(&ast3).unwrap());
}

// ============================================================================
// analyze_command_effects
// ============================================================================

#[test]
fn test_effect_analysis() {
    let mk = |name: &str| {
        make_main(vec![Stmt::Expr(Expr::FunctionCall {
            name: name.into(),
            args: vec![Expr::Literal(Literal::Str("x".into()))],
        })])
    };

    let ir = from_ast(&mk("curl")).unwrap();
    assert!(ir.effects().contains(&Effect::NetworkAccess));

    let ir = from_ast(&mk("echo")).unwrap();
    assert!(ir.effects().contains(&Effect::FileWrite));

    let ir = from_ast(&mk("custom_func")).unwrap();
    assert!(!ir.effects().contains(&Effect::NetworkAccess));
    assert!(!ir.effects().contains(&Effect::FileWrite));
}

// ============================================================================
// convert_index_to_value branches
// ============================================================================

#[test]
fn test_index_dynamic_and_literal() {
    // Dynamic: arr[i]
    let ast = make_main(vec![
        Stmt::Let {
            name: "arr".into(),
            value: Expr::Array(vec![
                Expr::Literal(Literal::U32(10)),
                Expr::Literal(Literal::U32(20)),
            ]),
            declaration: true,
        },
        Stmt::Let {
            name: "i".into(),
            value: Expr::Literal(Literal::U32(0)),
            declaration: true,
        },
        Stmt::Let {
            name: "v".into(),
            value: Expr::Index {
                object: Box::new(Expr::Variable("arr".into())),
                index: Box::new(Expr::Variable("i".into())),
            },
            declaration: true,
        },
    ]);
    assert_seq(&from_ast(&ast).unwrap());

    // Literal: arr[2]
    let ast2 = make_main(vec![
        Stmt::Let {
            name: "a".into(),
            value: Expr::Array(vec![
                Expr::Literal(Literal::U32(1)),
                Expr::Literal(Literal::U32(2)),
                Expr::Literal(Literal::U32(3)),
            ]),
            declaration: true,
        },
        Stmt::Let {
            name: "v".into(),
            value: Expr::Index {
                object: Box::new(Expr::Variable("a".into())),
                index: Box::new(Expr::Literal(Literal::U32(2))),
            },
            declaration: true,
        },
    ]);
    assert_seq(&from_ast(&ast2).unwrap());
}

// ============================================================================
// convert_let_block: multi-stmt block as value
// ============================================================================

#[test]
fn test_let_block_multi_stmt() {
    let ast = make_main(vec![Stmt::Let {
        name: "x".into(),
        value: Expr::Block(vec![
            Stmt::Let {
                name: "tmp".into(),
                value: Expr::Literal(Literal::U32(1)),
                declaration: true,
            },
            Stmt::Expr(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Variable("tmp".into())),
                right: Box::new(Expr::Literal(Literal::U32(2))),
            }),
        ]),
        declaration: true,
    }]);
    assert_seq(&from_ast(&ast).unwrap());
}

// ============================================================================
// Break, Continue, Return at top level
// ============================================================================

#[test]
fn test_break_continue_return_top_level() {
    let ast = make_main(vec![Stmt::While {
        condition: Expr::Literal(Literal::Bool(true)),
        body: vec![Stmt::Break],
        max_iterations: Some(10000),
    }]);
    assert_seq(&from_ast(&ast).unwrap());

    let ast2 = make_main(vec![Stmt::For {
        pattern: Pattern::Variable("i".into()),
        iter: Expr::Range {
            start: Box::new(Expr::Literal(Literal::U32(0))),
            end: Box::new(Expr::Literal(Literal::U32(5))),
            inclusive: false,
        },
        body: vec![Stmt::Continue],
        max_iterations: Some(1000),
    }]);
    assert_seq(&from_ast(&ast2).unwrap());

    assert_seq(
        &from_ast(&make_main(vec![Stmt::Return(Some(Expr::Literal(
            Literal::U32(42),
        )))]))
        .unwrap(),
    );
    assert_seq(&from_ast(&make_main(vec![Stmt::Return(None)])).unwrap());
}

// ============================================================================
// for over various iterables
// ============================================================================

#[test]
fn test_for_over_iterables() {
    // Array literal
    let ast = make_main(vec![Stmt::For {
        pattern: Pattern::Variable("it".into()),
        iter: Expr::Array(vec![
            Expr::Literal(Literal::Str("a".into())),
            Expr::Literal(Literal::Str("b".into())),
        ]),
        body: vec![Stmt::Expr(Expr::FunctionCall {
            name: "echo".into(),
            args: vec![Expr::Variable("it".into())],
        })],
        max_iterations: Some(1000),
    }]);
    assert_seq(&from_ast(&ast).unwrap());

    // Tracked array variable
    let ast2 = make_main(vec![
        Stmt::Let {
            name: "arr".into(),
            value: Expr::Array(vec![
                Expr::Literal(Literal::U32(1)),
                Expr::Literal(Literal::U32(2)),
            ]),
            declaration: true,
        },
        Stmt::For {
            pattern: Pattern::Variable("x".into()),
            iter: Expr::Variable("arr".into()),
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "echo".into(),
                args: vec![Expr::Variable("x".into())],
            })],
            max_iterations: Some(1000),
        },
    ]);
    assert_seq(&from_ast(&ast2).unwrap());

    // Untracked variable
    let ast3 = make_main(vec![Stmt::For {
        pattern: Pattern::Variable("x".into()),
        iter: Expr::Variable("items".into()),
        body: vec![Stmt::Expr(Expr::FunctionCall {
            name: "echo".into(),
            args: vec![Expr::Variable("x".into())],
        })],
        max_iterations: Some(1000),
    }]);
    assert_seq(&from_ast(&ast3).unwrap());

    // Generic expression
    let ast4 = make_main(vec![Stmt::For {
        pattern: Pattern::Variable("x".into()),
        iter: Expr::FunctionCall {
            name: "get".into(),
            args: vec![],
        },
        body: vec![Stmt::Expr(Expr::FunctionCall {
            name: "echo".into(),
            args: vec![Expr::Variable("x".into())],
        })],
        max_iterations: Some(1000),
    }]);
    assert_seq(&from_ast(&ast4).unwrap());
}

// ============================================================================
// Entry point not found error
// ============================================================================

#[test]
fn test_entry_point_not_found_error() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "helper".into(),
            params: vec![],
            return_type: Type::Void,
            body: vec![],
        }],
        entry_point: "main".into(),
    };
    assert!(from_ast(&ast).is_err());
}

// ============================================================================
// Empty array and PositionalArgs
// ============================================================================

#[test]
fn test_empty_array_and_positional_args() {
    let ast = make_main(vec![Stmt::Let {
        name: "e".into(),
        value: Expr::Array(vec![]),
        declaration: true,
    }]);
    assert_seq(&from_ast(&ast).unwrap());

    let ast2 = make_main(vec![Stmt::Let {
        name: "a".into(),
        value: Expr::PositionalArgs,
        declaration: true,
    }]);
    assert_seq(&from_ast(&ast2).unwrap());
}
