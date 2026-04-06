
use super::*;
use crate::bash_parser::ast::{AstMetadata, BashExpr, Span};

fn dummy_span() -> Span {
    Span::new(1, 1, 1, 10)
}

// ===== UNIT TESTS (RED PHASE) =====

/// Test: REPL-004-002-001 - Display simple command AST
#[test]
fn test_REPL_004_002_display_simple_command() {
    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::Literal("hello".to_string())],
            redirects: vec![],
            span: dummy_span(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = format_ast(&ast);

    assert!(output.contains("=== AST ==="), "Should have AST header");
    assert!(
        output.contains("Statements: 1"),
        "Should show statement count"
    );
    assert!(output.contains("Command: echo"), "Should show command name");
}

/// Test: REPL-004-002-002 - Display assignment AST
#[test]
fn test_REPL_004_002_display_assignment() {
    let ast = BashAst {
        statements: vec![BashStmt::Assignment {
            name: "VAR".to_string(),
            index: None,
            value: BashExpr::Literal("value".to_string()),
            exported: false,
            span: dummy_span(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = format_ast(&ast);

    assert!(output.contains("Assignment: VAR"), "Should show assignment");
}

/// Test: REPL-004-002-003 - Display if statement AST
#[test]
fn test_REPL_004_002_display_if_statement() {
    let ast = BashAst {
        statements: vec![BashStmt::If {
            condition: BashExpr::Literal("true".to_string()),
            then_block: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![],
                redirects: vec![],
                span: dummy_span(),
            }],
            elif_blocks: vec![],
            else_block: None,
            span: dummy_span(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 3,
            parse_time_ms: 0,
        },
    };

    let output = format_ast(&ast);

    assert!(output.contains("If statement"), "Should show if statement");
    assert!(
        output.contains("then: 1 statements"),
        "Should show then block"
    );
}

/// Test: REPL-004-002-004 - Display empty AST
#[test]
fn test_REPL_004_002_display_empty_ast() {
    let ast = BashAst {
        statements: vec![],
        metadata: AstMetadata {
            source_file: None,
            line_count: 0,
            parse_time_ms: 0,
        },
    };

    let output = format_ast(&ast);

    assert!(
        output.contains("Statements: 0"),
        "Should show zero statements"
    );
}

/// Test: REPL-004-002-005 - Display multiple statements
#[test]
fn test_REPL_004_002_display_multiple_statements() {
    let ast = BashAst {
        statements: vec![
            BashStmt::Command {
                name: "echo".to_string(),
                args: vec![],
                redirects: vec![],
                span: dummy_span(),
            },
            BashStmt::Assignment {
                name: "X".to_string(),
                index: None,
                value: BashExpr::Literal("5".to_string()),
                exported: false,
                span: dummy_span(),
            },
            BashStmt::Command {
                name: "ls".to_string(),
                args: vec![BashExpr::Literal("-la".to_string())],
                redirects: vec![],
                span: dummy_span(),
            },
        ],
        metadata: AstMetadata {
            source_file: None,
            line_count: 3,
            parse_time_ms: 0,
        },
    };

    let output = format_ast(&ast);

    assert!(output.contains("Statements: 3"), "Should show 3 statements");
    assert!(output.contains("[0]"), "Should have index 0");
    assert!(output.contains("[1]"), "Should have index 1");
    assert!(output.contains("[2]"), "Should have index 2");
}

/// Test: REPL-004-002-006 - Display for loop AST
#[test]
fn test_REPL_004_002_display_for_loop() {
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
                span: dummy_span(),
            }],
            span: dummy_span(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 3,
            parse_time_ms: 0,
        },
    };

    let output = format_ast(&ast);

    assert!(
        output.contains("For loop: i"),
        "Should show for loop variable"
    );
    assert!(output.contains("1 statements"), "Should show body size");
}

/// Test: REPL-004-002-007 - Display while loop AST
#[test]
fn test_REPL_004_002_display_while_loop() {
    let ast = BashAst {
        statements: vec![BashStmt::While {
            condition: BashExpr::Literal("true".to_string()),
            body: vec![
                BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: dummy_span(),
                },
                BashStmt::Command {
                    name: "sleep".to_string(),
                    args: vec![BashExpr::Literal("1".to_string())],
                    redirects: vec![],
                    span: dummy_span(),
                },
            ],
            span: dummy_span(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 3,
            parse_time_ms: 0,
        },
    };

    let output = format_ast(&ast);
    assert!(output.contains("While loop (2 statements)"));
}

#[cfg(test)]
mod ast_display_tests_extracted_REPL {
    use super::*;
    include!("ast_display_tests_extracted_REPL.rs");
}
