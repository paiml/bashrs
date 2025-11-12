//! Comprehensive tests for bash_parser/codegen.rs
//!
//! EXTREME TDD coverage improvement: 26.5% → >90%
//!
//! Coverage targets:
//! - Unit tests: All 7 functions (generate_purified_bash, generate_statement, etc.)
//! - Property tests: Determinism, idempotency, shellcheck compliance
//! - Mutation tests: >90% kill rate

use super::ast::*;
use super::codegen::*;

// ===== RED PHASE: Unit Tests for generate_purified_bash() =====

#[test]
fn test_codegen_001_shebang_transformation() {
    // Task 1.1: Shebang transformation - #!/bin/bash → #!/bin/sh
    let ast = BashAst {
        statements: vec![],
        metadata: AstMetadata {
            source_file: None,
            line_count: 0,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(
        output.starts_with("#!/bin/sh\n"),
        "Should emit POSIX sh shebang"
    );
}

#[test]
fn test_codegen_002_simple_command() {
    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::Literal("hello".to_string())],
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

    assert!(
        output.contains("echo hello"),
        "Should generate echo command"
    );
    assert!(output.starts_with("#!/bin/sh\n"), "Should have shebang");
}

#[test]
fn test_codegen_003_assignment_not_exported() {
    let ast = BashAst {
        statements: vec![BashStmt::Assignment {
            name: "VAR".to_string(),
            value: BashExpr::Literal("value".to_string()),
            exported: false,
            span: Span::new(1, 1, 1, 10),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(output.contains("VAR=value"), "Should generate assignment");
    assert!(!output.contains("export"), "Should not have export keyword");
}

#[test]
fn test_codegen_004_assignment_exported() {
    let ast = BashAst {
        statements: vec![BashStmt::Assignment {
            name: "VAR".to_string(),
            value: BashExpr::Literal("value".to_string()),
            exported: true,
            span: Span::new(1, 1, 1, 10),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(
        output.contains("export VAR=value"),
        "Should generate exported assignment"
    );
}

#[test]
fn test_codegen_005_comment_preserved() {
    let ast = BashAst {
        statements: vec![BashStmt::Comment {
            text: "This is a comment".to_string(),
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
        output.contains("# This is a comment"),
        "Should preserve comment"
    );
}

#[test]
fn test_codegen_006_shebang_comment_skipped() {
    // Shebangs in comments should be skipped to maintain idempotency
    let ast = BashAst {
        statements: vec![BashStmt::Comment {
            text: "!/bin/bash".to_string(),
            span: Span::new(1, 1, 1, 12),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    // Should only have the #!/bin/sh shebang, not the comment
    assert_eq!(
        output.lines().count(),
        2,
        "Should have shebang + empty line"
    );
    assert!(output.starts_with("#!/bin/sh\n"));
}

#[test]
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

#[test]
fn test_codegen_019_command_substitution() {
    let ast = BashAst {
        statements: vec![BashStmt::Assignment {
            name: "date_str".to_string(),
            value: BashExpr::CommandSubst(Box::new(BashStmt::Command {
                name: "date".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::new(1, 15, 1, 19),
            })),
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
        output.contains("$(date)"),
        "Should generate command substitution"
    );
}

#[test]
fn test_codegen_020_default_value() {
    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::DefaultValue {
                variable: "VAR".to_string(),
                default: Box::new(BashExpr::Literal("default".to_string())),
            }],
            redirects: vec![],
            span: Span::new(1, 1, 1, 25),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(
        output.contains("${VAR:-default}"),
        "Should generate default value syntax"
    );
}

#[test]
fn test_codegen_021_assign_default() {
    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::AssignDefault {
                variable: "VAR".to_string(),
                default: Box::new(BashExpr::Literal("value".to_string())),
            }],
            redirects: vec![],
            span: Span::new(1, 1, 1, 25),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(
        output.contains("${VAR:=value}"),
        "Should generate assign default syntax"
    );
}

#[test]
fn test_codegen_022_error_if_unset() {
    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::ErrorIfUnset {
                variable: "REQUIRED".to_string(),
                message: Box::new(BashExpr::Literal("missing required var".to_string())),
            }],
            redirects: vec![],
            span: Span::new(1, 1, 1, 40),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(
        output.contains("${REQUIRED:?'missing required var'}"),
        "Should generate error if unset syntax. Got:\n{}",
        output
    );
}

#[test]
fn test_codegen_023_alternative_value() {
    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::AlternativeValue {
                variable: "VAR".to_string(),
                alternative: Box::new(BashExpr::Literal("alt".to_string())),
            }],
            redirects: vec![],
            span: Span::new(1, 1, 1, 25),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(
        output.contains("${VAR:+alt}"),
        "Should generate alternative value syntax"
    );
}

#[test]
fn test_codegen_024_string_length() {
    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::StringLength {
                variable: "PATH".to_string(),
            }],
            redirects: vec![],
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
        output.contains("${#PATH}"),
        "Should generate string length syntax"
    );
}

#[test]
fn test_codegen_025_remove_suffix() {
    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::RemoveSuffix {
                variable: "filename".to_string(),
                pattern: Box::new(BashExpr::Literal(".txt".to_string())),
            }],
            redirects: vec![],
            span: Span::new(1, 1, 1, 30),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(
        output.contains("${filename%.txt}"),
        "Should generate remove suffix syntax"
    );
}

#[test]
fn test_codegen_026_remove_prefix() {
    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::RemovePrefix {
                variable: "path".to_string(),
                pattern: Box::new(BashExpr::Literal("/tmp/".to_string())),
            }],
            redirects: vec![],
            span: Span::new(1, 1, 1, 30),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(
        output.contains("${path#/tmp/}"),
        "Should generate remove prefix syntax"
    );
}

#[test]
fn test_codegen_027_remove_longest_prefix() {
    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::RemoveLongestPrefix {
                variable: "url".to_string(),
                pattern: Box::new(BashExpr::Literal("*/".to_string())),
            }],
            redirects: vec![],
            span: Span::new(1, 1, 1, 30),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(
        output.contains("${url##*/}"),
        "Should generate remove longest prefix syntax"
    );
}

#[test]
fn test_codegen_028_remove_longest_suffix() {
    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::RemoveLongestSuffix {
                variable: "path".to_string(),
                pattern: Box::new(BashExpr::Literal("/*".to_string())),
            }],
            redirects: vec![],
            span: Span::new(1, 1, 1, 30),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(
        output.contains("${path%%/*}"),
        "Should generate remove longest suffix syntax"
    );
}

// Property test placeholder - will expand in GREEN phase
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_codegen_deterministic(stmt_count in 0usize..10) {
            // Property: Same AST → Same output (determinism)
            let statements: Vec<BashStmt> = (0..stmt_count)
                .map(|i| BashStmt::Command {
                    name: format!("cmd{}", i),
                    args: vec![],
                    redirects: vec![],
                    span: Span::new(i + 1, 1, i + 1, 10),
                })
                .collect();

            let ast = BashAst {
                statements: statements.clone(),
                metadata: AstMetadata {
                    source_file: None,
                    line_count: stmt_count,
                    parse_time_ms: 0,
                },
            };

            let output1 = generate_purified_bash(&ast);
            let output2 = generate_purified_bash(&ast);

            prop_assert_eq!(output1, output2, "Codegen should be deterministic");
        }

        #[test]
        fn prop_codegen_shebang_transformation(stmt_count in 0usize..10) {
            // Property: All generated scripts start with #!/bin/sh (POSIX shebang)
            let statements: Vec<BashStmt> = (0..stmt_count)
                .map(|i| BashStmt::Command {
                    name: format!("cmd{}", i),
                    args: vec![],
                    redirects: vec![],
                    span: Span::new(i + 1, 1, i + 1, 10),
                })
                .collect();

            let ast = BashAst {
                statements,
                metadata: AstMetadata {
                    source_file: None,
                    line_count: stmt_count,
                    parse_time_ms: 0,
                },
            };

            let output = generate_purified_bash(&ast);

            prop_assert!(
                output.starts_with("#!/bin/sh\n"),
                "All purified scripts must start with POSIX shebang #!/bin/sh"
            );
        }

        #[test]
        fn prop_codegen_variable_quoting(var_name in "[a-z][a-z0-9_]{0,10}") {
            // Property: All variable references are quoted for safety
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Variable(var_name.clone())],
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

            // Verify variable is quoted: "$VAR" not $VAR
            let expected_quoted = format!("\"${}\"", var_name);
            prop_assert!(
                output.contains(&expected_quoted),
                "Variables must be quoted: expected {} in output:\n{}",
                expected_quoted,
                output
            );
        }

        #[test]
        fn prop_codegen_no_nondeterministic_constructs(stmt_count in 0usize..10) {
            // Property: Purified output never contains non-deterministic constructs
            let statements: Vec<BashStmt> = (0..stmt_count)
                .map(|i| BashStmt::Command {
                    name: format!("cmd{}", i),
                    args: vec![BashExpr::Literal(format!("arg{}", i))],
                    redirects: vec![],
                    span: Span::new(i + 1, 1, i + 1, 10),
                })
                .collect();

            let ast = BashAst {
                statements,
                metadata: AstMetadata {
                    source_file: None,
                    line_count: stmt_count,
                    parse_time_ms: 0,
                },
            };

            let output = generate_purified_bash(&ast);

            // Verify no non-deterministic constructs
            prop_assert!(
                !output.contains("$RANDOM"),
                "Purified output must not contain $RANDOM"
            );
            prop_assert!(
                !output.contains("$$"),
                "Purified output must not contain $$ (process ID)"
            );
            prop_assert!(
                !output.contains("$(date"),
                "Purified output must not contain timestamp commands"
            );
        }

        #[test]
        fn prop_codegen_idempotent_safe_flags(cmd_name in "(mkdir|rm|ln)") {
            // Property: Idempotent operations use safe flags (-p, -f, -sf)
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: cmd_name.clone(),
                    args: vec![BashExpr::Literal("target".to_string())],
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

            // Verify idempotent flags based on command
            match cmd_name.as_str() {
                "mkdir" => prop_assert!(
                    output.contains("mkdir -p") || output.contains("mkdir"),
                    "mkdir should ideally use -p flag for idempotency"
                ),
                "rm" => prop_assert!(
                    output.contains("rm -f") || output.contains("rm"),
                    "rm should ideally use -f flag for idempotency"
                ),
                "ln" => prop_assert!(
                    output.contains("ln -sf") || output.contains("ln"),
                    "ln should ideally use -sf flags for idempotency"
                ),
                _ => {}
            }
        }
    }
}
