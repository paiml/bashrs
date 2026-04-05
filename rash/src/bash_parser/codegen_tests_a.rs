//! Comprehensive tests for bash_parser/codegen.rs
//!
//! EXTREME TDD coverage improvement: 26.5% → >90%
//!
//! Coverage targets:
//! - Unit tests: All 7 functions (generate_purified_bash, generate_statement, etc.)
//! - Property tests: Determinism, idempotency, shellcheck compliance
//! - Mutation tests: >90% kill rate

#![allow(clippy::expect_used)]

use super::ast::*;
use super::codegen::*;

// ===== RED PHASE: Unit Tests for generate_purified_bash() =====

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

#[test]
fn test_codegen_019_command_substitution() {
    let ast = BashAst {
        statements: vec![BashStmt::Assignment {
            name: "date_str".to_string(),
            index: None,
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

/// Issue #72 - Test that redirects are preserved in codegen
#[test]
fn test_codegen_029_redirect_output() {
    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "sort".to_string(),
            args: vec![],
            redirects: vec![Redirect::Output {
                target: BashExpr::Literal("/tmp/out.txt".to_string()),
            }],
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
        output.contains("> /tmp/out.txt"),
        "Should preserve output redirect. Got: {}",
        output
    );
}

/// Issue #72 - End-to-end test: parse → generate preserves redirects
#[test]
fn test_codegen_030_redirect_roundtrip() {
    use super::parser::BashParser;

    let input = "sort > /tmp/out.txt";
    let mut parser = BashParser::new(input).expect("Failed to create parser");
    let ast = parser.parse().expect("Failed to parse");

    let output = generate_purified_bash(&ast);

    assert!(
        output.contains("> /tmp/out.txt") || output.contains(">/tmp/out.txt"),
        "Redirect should be preserved after parse→generate. Got: {}",
        output
    );
}

/// Issue #72 - Test that input redirects are preserved
#[test]
fn test_codegen_031_redirect_input() {
    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "wc".to_string(),
            args: vec![BashExpr::Literal("-l".to_string())],
            redirects: vec![Redirect::Input {
                target: BashExpr::Literal("/tmp/input.txt".to_string()),
            }],
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
        output.contains("< /tmp/input.txt"),
        "Should preserve input redirect. Got: {}",
        output
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
