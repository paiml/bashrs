//! Tests extracted from generators.rs for file health compliance.
#![allow(clippy::unwrap_used)]

use crate::bash_parser::generators::*;
use proptest::strategy::ValueTree;

#[test]
fn test_generate_purified_bash_and_list() {
    let ast = BashAst {
        statements: vec![BashStmt::AndList {
            left: Box::new(BashStmt::Command {
                name: "true".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }),
            right: Box::new(BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("ok".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }),
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };
    let output = generate_purified_bash(&ast);
    assert!(output.contains("true && echo ok"));
}

#[test]
fn test_generate_purified_bash_or_list() {
    let ast = BashAst {
        statements: vec![BashStmt::OrList {
            left: Box::new(BashStmt::Command {
                name: "false".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }),
            right: Box::new(BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("failed".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }),
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };
    let output = generate_purified_bash(&ast);
    assert!(output.contains("false || echo failed"));
}

#[test]
fn test_generate_purified_bash_brace_group() {
    let ast = BashAst {
        statements: vec![BashStmt::BraceGroup {
            body: vec![
                BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("a".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                },
                BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("b".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                },
            ],
            subshell: false,
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };
    let output = generate_purified_bash(&ast);
    assert!(output.contains("{"));
    assert!(output.contains("}"));
}

#[test]
fn test_generate_purified_bash_coproc_with_name() {
    let ast = BashAst {
        statements: vec![BashStmt::Coproc {
            name: Some("mycoproc".to_string()),
            body: vec![BashStmt::Command {
                name: "cat".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };
    let output = generate_purified_bash(&ast);
    assert!(output.contains("coproc mycoproc"));
}

#[test]
fn test_generate_purified_bash_coproc_without_name() {
    let ast = BashAst {
        statements: vec![BashStmt::Coproc {
            name: None,
            body: vec![BashStmt::Command {
                name: "cat".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };
    let output = generate_purified_bash(&ast);
    assert!(output.contains("coproc { cat; }"));
}

// ============== generate_expr tests ==============

#[test]
fn test_generate_expr_literal_simple() {
    let expr = BashExpr::Literal("hello".to_string());
    let output = generate_expr(&expr);
    assert_eq!(output, "hello");
}

#[test]
fn test_generate_expr_literal_with_space() {
    let expr = BashExpr::Literal("hello world".to_string());
    let output = generate_expr(&expr);
    assert_eq!(output, "'hello world'");
}

#[test]
fn test_generate_expr_literal_with_dollar() {
    let expr = BashExpr::Literal("$HOME".to_string());
    let output = generate_expr(&expr);
    assert_eq!(output, "'$HOME'");
}

#[test]
fn test_generate_expr_variable() {
    let expr = BashExpr::Variable("FOO".to_string());
    let output = generate_expr(&expr);
    assert_eq!(output, "\"$FOO\"");
}

#[test]
fn test_generate_expr_array() {
    let expr = BashExpr::Array(vec![
        BashExpr::Literal("a".to_string()),
        BashExpr::Literal("b".to_string()),
    ]);
    let output = generate_expr(&expr);
    assert_eq!(output, "a b");
}

#[test]
fn test_generate_expr_arithmetic() {
    let expr = BashExpr::Arithmetic(Box::new(ArithExpr::Add(
        Box::new(ArithExpr::Number(1)),
        Box::new(ArithExpr::Number(2)),
    )));
    let output = generate_expr(&expr);
    assert_eq!(output, "$((1 + 2))");
}

#[test]
fn test_generate_expr_command_subst() {
    let expr = BashExpr::CommandSubst(Box::new(BashStmt::Command {
        name: "date".to_string(),
        args: vec![],
        redirects: vec![],
        span: Span::dummy(),
    }));
    let output = generate_expr(&expr);
    assert_eq!(output, "$(date)");
}

#[test]
fn test_generate_expr_concat() {
    let expr = BashExpr::Concat(vec![
        BashExpr::Literal("prefix_".to_string()),
        BashExpr::Variable("VAR".to_string()),
    ]);
    let output = generate_expr(&expr);
    assert!(output.contains("prefix_"));
    assert!(output.contains("\"$VAR\""));
}

#[test]
fn test_generate_expr_glob() {
    let expr = BashExpr::Glob("*.txt".to_string());
    let output = generate_expr(&expr);
    assert_eq!(output, "*.txt");
}

#[test]
fn test_generate_expr_default_value() {
    let expr = BashExpr::DefaultValue {
        variable: "FOO".to_string(),
        default: Box::new(BashExpr::Literal("default".to_string())),
    };
    let output = generate_expr(&expr);
    assert!(output.contains("${FOO:-default}"));
}

#[test]
fn test_generate_expr_assign_default() {
    let expr = BashExpr::AssignDefault {
        variable: "FOO".to_string(),
        default: Box::new(BashExpr::Literal("default".to_string())),
    };
    let output = generate_expr(&expr);
    assert!(output.contains("${FOO:=default}"));
}

#[test]
fn test_generate_expr_error_if_unset() {
    let expr = BashExpr::ErrorIfUnset {
        variable: "FOO".to_string(),
        message: Box::new(BashExpr::Literal("error".to_string())),
    };
    let output = generate_expr(&expr);
    assert!(output.contains("${FOO:?error}"));
}

#[test]
fn test_generate_expr_alternative_value() {
    let expr = BashExpr::AlternativeValue {
        variable: "FOO".to_string(),
        alternative: Box::new(BashExpr::Literal("alt".to_string())),
    };
    let output = generate_expr(&expr);
    assert!(output.contains("${FOO:+alt}"));
}

#[test]
fn test_generate_expr_string_length() {
    let expr = BashExpr::StringLength {
        variable: "FOO".to_string(),
    };
    let output = generate_expr(&expr);
    assert!(output.contains("${#FOO}"));
}

#[test]
fn test_generate_expr_remove_suffix() {
    let expr = BashExpr::RemoveSuffix {
        variable: "FILE".to_string(),
        pattern: Box::new(BashExpr::Literal(".txt".to_string())),
    };
    let output = generate_expr(&expr);
    assert!(output.contains("${FILE%.txt}"));
}

#[test]
fn test_generate_expr_remove_prefix() {
    let expr = BashExpr::RemovePrefix {
        variable: "PATH".to_string(),
        pattern: Box::new(BashExpr::Literal("*/".to_string())),
    };
    let output = generate_expr(&expr);
    assert!(output.contains("${PATH#*/}"));
}

#[test]
fn test_generate_expr_remove_longest_prefix() {
    let expr = BashExpr::RemoveLongestPrefix {
        variable: "PATH".to_string(),
        pattern: Box::new(BashExpr::Literal("*/".to_string())),
    };
    let output = generate_expr(&expr);
    assert!(output.contains("${PATH##*/}"));
}

#[test]
fn test_generate_expr_remove_longest_suffix() {
    let expr = BashExpr::RemoveLongestSuffix {
        variable: "FILE".to_string(),
        pattern: Box::new(BashExpr::Literal(".*".to_string())),
    };
    let output = generate_expr(&expr);
    assert!(output.contains("${FILE%%.*}"));
}

#[test]
fn test_generate_expr_command_condition() {
    let expr = BashExpr::CommandCondition(Box::new(BashStmt::Command {
        name: "test".to_string(),
        args: vec![
            BashExpr::Literal("-f".to_string()),
            BashExpr::Literal("file".to_string()),
        ],
        redirects: vec![],
        span: Span::dummy(),
    }));
    let output = generate_expr(&expr);
    assert!(output.contains("test -f file"));
}

// ============== generate_arith_expr tests ==============

#[test]
fn test_generate_arith_expr_number() {
    let expr = ArithExpr::Number(42);
    let output = generate_arith_expr(&expr);
    assert_eq!(output, "42");
}

#[test]
fn test_generate_arith_expr_variable() {
    let expr = ArithExpr::Variable("x".to_string());
    let output = generate_arith_expr(&expr);
    assert_eq!(output, "x");
}

#[test]
fn test_generate_arith_expr_add() {
    let expr = ArithExpr::Add(
        Box::new(ArithExpr::Number(1)),
        Box::new(ArithExpr::Number(2)),
    );
    let output = generate_arith_expr(&expr);
    assert_eq!(output, "1 + 2");
}

#[test]
fn test_generate_arith_expr_sub() {
    let expr = ArithExpr::Sub(
        Box::new(ArithExpr::Number(5)),
        Box::new(ArithExpr::Number(3)),
    );
    let output = generate_arith_expr(&expr);
    assert_eq!(output, "5 - 3");
}

#[test]
fn test_generate_arith_expr_mul() {
    let expr = ArithExpr::Mul(
        Box::new(ArithExpr::Number(2)),
        Box::new(ArithExpr::Number(3)),
    );
    let output = generate_arith_expr(&expr);
    assert_eq!(output, "2 * 3");
}

#[test]
fn test_generate_arith_expr_div() {
    let expr = ArithExpr::Div(
        Box::new(ArithExpr::Number(6)),
        Box::new(ArithExpr::Number(2)),
    );
    let output = generate_arith_expr(&expr);
    assert_eq!(output, "6 / 2");
}

#[test]
fn test_generate_arith_expr_mod() {
    let expr = ArithExpr::Mod(
        Box::new(ArithExpr::Number(7)),
        Box::new(ArithExpr::Number(3)),
    );
    let output = generate_arith_expr(&expr);
    assert_eq!(output, "7 % 3");
}

// ============== generate_test_expr tests ==============

#[test]
fn test_generate_test_expr_string_eq() {
    let expr = TestExpr::StringEq(
        BashExpr::Variable("x".to_string()),
        BashExpr::Literal("y".to_string()),
    );
    let output = generate_test_expr(&expr);
    assert!(output.contains("= y"));
}

#[test]
fn test_generate_test_expr_string_ne() {
    let expr = TestExpr::StringNe(
        BashExpr::Variable("x".to_string()),
        BashExpr::Literal("y".to_string()),
    );
    let output = generate_test_expr(&expr);
    assert!(output.contains("!= y"));
}

#[test]
fn test_generate_test_expr_int_eq() {
    let expr = TestExpr::IntEq(
        BashExpr::Variable("x".to_string()),
        BashExpr::Literal("5".to_string()),
    );
    let output = generate_test_expr(&expr);
    assert!(output.contains("-eq 5"));
}

#[test]
fn test_generate_test_expr_int_ne() {
    let expr = TestExpr::IntNe(
        BashExpr::Variable("x".to_string()),
        BashExpr::Literal("5".to_string()),
    );
    let output = generate_test_expr(&expr);
    assert!(output.contains("-ne 5"));
}

#[test]
fn test_generate_test_expr_int_lt() {
    let expr = TestExpr::IntLt(
        BashExpr::Variable("x".to_string()),
        BashExpr::Literal("5".to_string()),
    );
    let output = generate_test_expr(&expr);
    assert!(output.contains("-lt 5"));
}

#[test]
fn test_generate_test_expr_int_le() {
    let expr = TestExpr::IntLe(
        BashExpr::Variable("x".to_string()),
        BashExpr::Literal("5".to_string()),
    );
    let output = generate_test_expr(&expr);
    assert!(output.contains("-le 5"));
}

#[test]
fn test_generate_test_expr_int_gt() {
    let expr = TestExpr::IntGt(
        BashExpr::Variable("x".to_string()),
        BashExpr::Literal("5".to_string()),
    );
    let output = generate_test_expr(&expr);
    assert!(output.contains("-gt 5"));
}

#[test]
fn test_generate_test_expr_int_ge() {
    let expr = TestExpr::IntGe(
        BashExpr::Variable("x".to_string()),
        BashExpr::Literal("5".to_string()),
    );
    let output = generate_test_expr(&expr);
    assert!(output.contains("-ge 5"));
}

#[test]
fn test_generate_test_expr_file_exists() {
    let expr = TestExpr::FileExists(BashExpr::Literal("/tmp".to_string()));
    let output = generate_test_expr(&expr);
    assert!(output.contains("-e /tmp"));
}

#[test]
fn test_generate_test_expr_file_readable() {
    let expr = TestExpr::FileReadable(BashExpr::Literal("/tmp".to_string()));
    let output = generate_test_expr(&expr);
    assert!(output.contains("-r /tmp"));
}

#[test]
fn test_generate_test_expr_file_writable() {
    let expr = TestExpr::FileWritable(BashExpr::Literal("/tmp".to_string()));
    let output = generate_test_expr(&expr);
    assert!(output.contains("-w /tmp"));
}

#[test]
fn test_generate_test_expr_file_executable() {
    let expr = TestExpr::FileExecutable(BashExpr::Literal("/tmp".to_string()));
    let output = generate_test_expr(&expr);
    assert!(output.contains("-x /tmp"));
}
