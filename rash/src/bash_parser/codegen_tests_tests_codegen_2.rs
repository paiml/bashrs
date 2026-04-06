fn test_codegen_007_function_definition() {
    let ast = BashAst {
        statements: vec![BashStmt::Function {
            name: "greet".to_string(),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("Hello".to_string())],
                redirects: vec![],
                span: Span::new(2, 5, 2, 15),
            }],
            span: Span::new(1, 1, 3, 1),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 3,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(
        output.contains("greet()"),
        "Should have function declaration"
    );
    assert!(output.contains("echo Hello"), "Should have function body");
    assert!(output.contains("}"), "Should close function");
}

#[test]
fn test_codegen_008_if_statement_no_else() {
    let ast = BashAst {
        statements: vec![BashStmt::If {
            condition: BashExpr::Test(Box::new(TestExpr::IntEq(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("1".to_string()),
            ))),
            then_block: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("one".to_string())],
                redirects: vec![],
                span: Span::new(2, 5, 2, 15),
            }],
            elif_blocks: vec![],
            else_block: None,
            span: Span::new(1, 1, 3, 2),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 3,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(output.contains("if"), "Should have if keyword");
    assert!(output.contains("then"), "Should have then keyword");
    assert!(output.contains("fi"), "Should close with fi");
    assert!(!output.contains("else"), "Should not have else");
}

#[test]
fn test_codegen_009_if_statement_with_else() {
    let ast = BashAst {
        statements: vec![BashStmt::If {
            condition: BashExpr::Test(Box::new(TestExpr::IntEq(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("1".to_string()),
            ))),
            then_block: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("one".to_string())],
                redirects: vec![],
                span: Span::new(2, 5, 2, 15),
            }],
            elif_blocks: vec![],
            else_block: Some(vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("other".to_string())],
                redirects: vec![],
                span: Span::new(4, 5, 4, 16),
            }]),
            span: Span::new(1, 1, 5, 2),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 5,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(output.contains("if"), "Should have if keyword");
    assert!(output.contains("else"), "Should have else keyword");
    assert!(output.contains("fi"), "Should close with fi");
}

#[test]
fn test_codegen_010_for_loop() {
    let ast = BashAst {
        statements: vec![BashStmt::For {
            variable: "i".to_string(),
            items: BashExpr::Array(vec![
                BashExpr::Literal("1".to_string()),
                BashExpr::Literal("2".to_string()),
                BashExpr::Literal("3".to_string()),
            ]),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("i".to_string())],
                redirects: vec![],
                span: Span::new(2, 5, 2, 13),
            }],
            span: Span::new(1, 1, 3, 4),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 3,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(output.contains("for i in"), "Should have for loop");
    assert!(output.contains("do"), "Should have do keyword");
    assert!(output.contains("done"), "Should have done keyword");
}

#[test]
fn test_codegen_011_while_loop() {
    let ast = BashAst {
        statements: vec![BashStmt::While {
            condition: BashExpr::Test(Box::new(TestExpr::IntLt(
                BashExpr::Variable("i".to_string()),
                BashExpr::Literal("10".to_string()),
            ))),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("i".to_string())],
                redirects: vec![],
                span: Span::new(2, 5, 2, 13),
            }],
            span: Span::new(1, 1, 3, 4),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 3,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(output.contains("while"), "Should have while keyword");
    assert!(output.contains("do"), "Should have do keyword");
    assert!(output.contains("done"), "Should have done keyword");
}

#[test]
fn test_codegen_012_until_loop_negated() {
    // Until loops should be transformed to while loops with negated condition
    let ast = BashAst {
        statements: vec![BashStmt::Until {
            condition: BashExpr::Test(Box::new(TestExpr::IntGt(
                BashExpr::Variable("i".to_string()),
                BashExpr::Literal("5".to_string()),
            ))),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("i".to_string())],
                redirects: vec![],
                span: Span::new(2, 5, 2, 13),
            }],
            span: Span::new(1, 1, 3, 4),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 3,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    // Until should be transformed to while with negation
    assert!(output.contains("while"), "Should transform until to while");
    assert!(output.contains("!"), "Should negate condition");
    assert!(
        !output.contains("until"),
        "Should not contain until keyword"
    );
}

#[test]
fn test_codegen_013_return_with_code() {
    let ast = BashAst {
        statements: vec![BashStmt::Return {
            code: Some(BashExpr::Literal("0".to_string())),
            span: Span::new(1, 1, 1, 9),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(
        output.contains("return 0"),
        "Should generate return with code"
    );
}

#[test]
fn test_codegen_014_return_without_code() {
    let ast = BashAst {
        statements: vec![BashStmt::Return {
            code: None,
            span: Span::new(1, 1, 1, 7),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(
        output.contains("return\n"),
        "Should generate return without code"
    );
}

#[test]
fn test_codegen_015_case_statement() {
    let ast = BashAst {
        statements: vec![BashStmt::Case {
            word: BashExpr::Variable("x".to_string()),
            arms: vec![
                CaseArm {
                    patterns: vec!["1".to_string()],
                    body: vec![BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("one".to_string())],
                        redirects: vec![],
                        span: Span::new(3, 9, 3, 18),
                    }],
                },
                CaseArm {
                    patterns: vec!["2".to_string()],
                    body: vec![BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("two".to_string())],
                        redirects: vec![],
                        span: Span::new(6, 9, 6, 18),
                    }],
                },
            ],
            span: Span::new(1, 1, 9, 4),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 9,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(output.contains("case"), "Should have case keyword");
    assert!(output.contains("esac"), "Should have esac keyword");
    assert!(output.contains(";;"), "Should have pattern terminators");
}

#[test]
fn test_codegen_016_pipeline() {
    let ast = BashAst {
        statements: vec![BashStmt::Pipeline {
            commands: vec![
                BashStmt::Command {
                    name: "cat".to_string(),
                    args: vec![BashExpr::Literal("file.txt".to_string())],
                    redirects: vec![],
                    span: Span::new(1, 1, 1, 13),
                },
                BashStmt::Command {
                    name: "grep".to_string(),
                    args: vec![BashExpr::Literal("pattern".to_string())],
                    redirects: vec![],
                    span: Span::new(1, 17, 1, 30),
                },
            ],
            span: Span::new(1, 1, 1, 30),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(output.contains("|"), "Should have pipe operator");
    assert!(output.contains("cat"), "Should have cat command");
    assert!(output.contains("grep"), "Should have grep command");
}

// ===== Expression Generation Tests =====

#[test]
fn test_codegen_017_variable_quoted() {
    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::Variable("VAR".to_string())],
            redirects: vec![],
            span: Span::new(1, 1, 1, 10),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    // Variables should always be quoted for safety
    assert!(output.contains("\"$VAR\""), "Variables should be quoted");
}

#[test]
fn test_codegen_018_arithmetic_expression() {
    let ast = BashAst {
        statements: vec![BashStmt::Assignment {
            name: "result".to_string(),
            index: None,
            value: BashExpr::Arithmetic(Box::new(ArithExpr::Add(
                Box::new(ArithExpr::Number(5)),
                Box::new(ArithExpr::Number(3)),
            ))),
            exported: false,
            span: Span::new(1, 1, 1, 20),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(
        output.contains("$((5 + 3))"),
        "Should generate arithmetic expansion"
    );
}

include!("codegen_tests_tests_codegen.rs");
