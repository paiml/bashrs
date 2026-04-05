//! Tests extracted from generators.rs for file health compliance.
#![allow(clippy::unwrap_used)]

use crate::bash_parser::generators::*;
use proptest::strategy::ValueTree;

proptest! {
    #[test]
    fn test_generates_valid_identifiers(id in bash_identifier()) {
        // Should start with letter or underscore
        assert!(id.chars().next().unwrap().is_alphabetic() || id.starts_with('_'));
        // Should be reasonable length
        assert!(id.len() <= 16);
    }

    #[test]
    fn test_generates_valid_expressions(expr in bash_expr(2)) {
        // All expressions should be constructible
        match expr {
            BashExpr::Literal(s) => assert!(!s.is_empty() || s.is_empty()),
            BashExpr::Variable(v) => assert!(!v.is_empty()),
            BashExpr::Array(items) => assert!(items.len() <= 3),
            BashExpr::Arithmetic(_) => {},
            _ => {}
        }
    }

    #[test]
    fn test_generates_valid_statements(stmt in bash_stmt(2)) {
        // All statements should be constructible
        match stmt {
            BashStmt::Assignment { name, .. } => assert!(!name.is_empty()),
            BashStmt::Command { name, .. } => assert!(!name.is_empty()),
            BashStmt::Function { name, body, .. } => {
                assert!(!name.is_empty());
                assert!(!body.is_empty());
            }
            _ => {}
        }
    }

    #[test]
    fn test_generates_valid_scripts(script in bash_script()) {
        // Scripts should have at least one statement
        assert!(!script.statements.is_empty());
        assert!(script.statements.len() <= 10);
    }

    /// 🔴 RED: Property test for unique function names
    /// TICKET-6002: bash_script() should generate scripts with unique function names
    #[test]
    fn test_generated_scripts_have_unique_function_names(script in bash_script()) {
        use std::collections::HashSet;

        // Collect all function names
        let mut function_names = HashSet::new();
        let mut duplicate_found = false;
        let mut duplicate_name = String::new();

        for stmt in &script.statements {
            if let BashStmt::Function { name, .. } = stmt {
                if !function_names.insert(name.clone()) {
                    // Duplicate found!
                    duplicate_found = true;
                    duplicate_name = name.clone();
                    break;
                }
            }
        }

        prop_assert!(
            !duplicate_found,
            "Generated script has duplicate function name: '{}'. \
            All function names in a script must be unique. \
            Function names found: {:?}",
            duplicate_name,
            function_names
        );
    }
}

// ============== generate_purified_bash tests ==============

#[test]
fn test_generate_purified_bash_empty() {
    let ast = BashAst {
        statements: vec![],
        metadata: AstMetadata {
            source_file: None,
            line_count: 0,
            parse_time_ms: 0,
        },
    };
    let output = generate_purified_bash(&ast);
    assert!(output.starts_with("#!/bin/sh\n"));
}

#[test]
fn test_generate_purified_bash_command() {
    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::Literal("hello".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };
    let output = generate_purified_bash(&ast);
    assert!(output.contains("echo hello"));
}

#[test]
fn test_generate_purified_bash_assignment() {
    let ast = BashAst {
        statements: vec![BashStmt::Assignment {
            name: "FOO".to_string(),
            index: None,
            value: BashExpr::Literal("bar".to_string()),
            exported: false,
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };
    let output = generate_purified_bash(&ast);
    assert!(output.contains("FOO=bar"));
}

#[test]
fn test_generate_purified_bash_exported_assignment() {
    let ast = BashAst {
        statements: vec![BashStmt::Assignment {
            name: "PATH".to_string(),
            index: None,
            value: BashExpr::Literal("/usr/bin".to_string()),
            exported: true,
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };
    let output = generate_purified_bash(&ast);
    assert!(output.contains("export PATH=/usr/bin"));
}

#[test]
fn test_generate_purified_bash_comment() {
    let ast = BashAst {
        statements: vec![BashStmt::Comment {
            text: "This is a comment".to_string(),
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };
    let output = generate_purified_bash(&ast);
    assert!(output.contains("# This is a comment"));
}

#[test]
fn test_generate_purified_bash_function() {
    let ast = BashAst {
        statements: vec![BashStmt::Function {
            name: "my_func".to_string(),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("hello".to_string())],
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
    assert!(output.contains("my_func() {"));
    assert!(output.contains("echo hello"));
    assert!(output.contains("}"));
}

#[test]
fn test_generate_purified_bash_if_statement() {
    let ast = BashAst {
        statements: vec![BashStmt::If {
            condition: BashExpr::Test(Box::new(TestExpr::StringNonEmpty(BashExpr::Variable(
                "x".to_string(),
            )))),
            then_block: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("yes".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            elif_blocks: vec![],
            else_block: None,
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };
    let output = generate_purified_bash(&ast);
    assert!(output.contains("if"));
    assert!(output.contains("then"));
    assert!(output.contains("fi"));
}

#[test]
fn test_generate_purified_bash_if_with_else() {
    let ast = BashAst {
        statements: vec![BashStmt::If {
            condition: BashExpr::Test(Box::new(TestExpr::StringNonEmpty(BashExpr::Variable(
                "x".to_string(),
            )))),
            then_block: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("yes".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            elif_blocks: vec![],
            else_block: Some(vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("no".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }]),
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };
    let output = generate_purified_bash(&ast);
    assert!(output.contains("else"));
}

#[test]
fn test_generate_purified_bash_for_loop() {
    let ast = BashAst {
        statements: vec![BashStmt::For {
            variable: "i".to_string(),
            items: BashExpr::Array(vec![
                BashExpr::Literal("1".to_string()),
                BashExpr::Literal("2".to_string()),
            ]),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("i".to_string())],
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
    assert!(output.contains("for i in"));
    assert!(output.contains("do"));
    assert!(output.contains("done"));
}

#[test]
fn test_generate_purified_bash_for_c_style() {
    let ast = BashAst {
        statements: vec![BashStmt::ForCStyle {
            init: "i=0".to_string(),
            condition: "i<10".to_string(),
            increment: "i++".to_string(),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("i".to_string())],
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
    assert!(output.contains("for ((i=0; i<10; i++))"));
    assert!(output.contains("do"));
    assert!(output.contains("done"));
}

#[test]
fn test_generate_purified_bash_while_loop() {
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
    assert!(output.contains("while"));
    assert!(output.contains("do"));
    assert!(output.contains("done"));
}

#[test]
fn test_generate_purified_bash_until_loop() {
    let ast = BashAst {
        statements: vec![BashStmt::Until {
            condition: BashExpr::Test(Box::new(TestExpr::IntGe(
                BashExpr::Variable("i".to_string()),
                BashExpr::Literal("10".to_string()),
            ))),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("i".to_string())],
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
    // Until is transformed to while with negated condition
    assert!(output.contains("while"));
    assert!(output.contains("!"));
}

#[test]
fn test_generate_purified_bash_return() {
    let ast = BashAst {
        statements: vec![BashStmt::Return {
            code: Some(BashExpr::Literal("0".to_string())),
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };
    let output = generate_purified_bash(&ast);
    assert!(output.contains("return 0"));
}

#[test]
fn test_generate_purified_bash_return_without_code() {
    let ast = BashAst {
        statements: vec![BashStmt::Return {
            code: None,
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };
    let output = generate_purified_bash(&ast);
    assert!(output.contains("return"));
}

#[test]
fn test_generate_purified_bash_case() {
    let ast = BashAst {
        statements: vec![BashStmt::Case {
            word: BashExpr::Variable("x".to_string()),
            arms: vec![
                CaseArm {
                    patterns: vec!["a".to_string()],
                    body: vec![BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("A".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    }],
                },
                CaseArm {
                    patterns: vec!["b".to_string(), "c".to_string()],
                    body: vec![BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("B or C".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    }],
                },
            ],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };
    let output = generate_purified_bash(&ast);
    assert!(output.contains("case"));
    assert!(output.contains("esac"));
    assert!(output.contains(";;"));
    assert!(output.contains("b|c"));
}

#[test]
fn test_generate_purified_bash_pipeline() {
    let ast = BashAst {
        statements: vec![BashStmt::Pipeline {
            commands: vec![
                BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("hello".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                },
                BashStmt::Command {
                    name: "grep".to_string(),
                    args: vec![BashExpr::Literal("h".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                },
            ],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };
    let output = generate_purified_bash(&ast);
    assert!(output.contains("echo hello | grep h"));
}
