//! Issue #136: Regression tests for `if ! command -v` pattern.
//!
//! The root cause was fixed in 7489a908d (issue #133) which added
//! `BashStmt::Negated` + `BashExpr::CommandCondition` support to
//! `parse_test_expression`. These tests prevent recurrence.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]

use crate::bash_parser::ast::*;
use crate::bash_parser::codegen::generate_purified_bash;
use crate::bash_parser::parser::BashParser;

// =============================================================================
// Parser-level: AST structure verification
// =============================================================================

/// Exact reproduction of issue #136: `if ! command -v apr >/dev/null 2>&1; then`
#[test]
fn test_issue_136_if_not_command_v_with_redirects() {
    let script = "if ! command -v apr >/dev/null 2>&1; then\n    echo \"ERROR: apr not found.\"\n    exit 1\nfi";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().unwrap_or_else(|e| {
        panic!("Issue #136 REGRESSION: `if ! command -v` must parse. Error: {e:?}")
    });
    assert_eq!(ast.statements.len(), 1, "Should produce one If statement");
    match &ast.statements[0] {
        BashStmt::If {
            condition,
            then_block,
            ..
        } => {
            // Condition: CommandCondition(Negated { command: Command { name: "command", .. } })
            match condition {
                BashExpr::CommandCondition(cmd) => {
                    assert!(
                        matches!(cmd.as_ref(), BashStmt::Negated { .. }),
                        "Condition should be Negated, got: {cmd:?}"
                    );
                }
                other => panic!("Expected CommandCondition, got: {other:?}"),
            }
            assert_eq!(then_block.len(), 2, "then block: echo + exit");
        }
        other => panic!("Expected If, got: {other:?}"),
    }
}

/// Variant: no redirects
#[test]
fn test_issue_136_if_not_command_v_no_redirects() {
    let script = "if ! command -v foo; then echo missing; fi";
    let mut parser = BashParser::new(script).expect("Lexer");
    let ast = parser
        .parse()
        .unwrap_or_else(|e| panic!("Issue #136: no-redirect variant must parse. Error: {e:?}"));
    assert_eq!(ast.statements.len(), 1);
    assert!(matches!(&ast.statements[0], BashStmt::If { .. }));
}

/// Variant: variable argument `"$bin"`
#[test]
fn test_issue_136_if_not_command_v_with_variable() {
    let script = "if ! command -v \"$bin\" >/dev/null 2>&1; then\n    exit 1\nfi";
    let mut parser = BashParser::new(script).expect("Lexer");
    let ast = parser
        .parse()
        .unwrap_or_else(|e| panic!("Issue #136: variable arg variant must parse. Error: {e:?}"));
    assert_eq!(ast.statements.len(), 1);
    assert!(matches!(&ast.statements[0], BashStmt::If { .. }));
}

/// Variant: while loop with negated command
#[test]
fn test_issue_136_while_not_command_v() {
    let script = "while ! command -v foo >/dev/null 2>&1; do sleep 1; done";
    let mut parser = BashParser::new(script).expect("Lexer");
    let ast = parser
        .parse()
        .unwrap_or_else(|e| panic!("Issue #136: while variant must parse. Error: {e:?}"));
    assert_eq!(ast.statements.len(), 1);
    assert!(matches!(&ast.statements[0], BashStmt::While { .. }));
}

/// Variant: inside a function body
#[test]
fn test_issue_136_negated_command_v_in_function() {
    let script = "qualify() {\n    if ! command -v \"$1\" >/dev/null 2>&1; then\n        return 1\n    fi\n}";
    let mut parser = BashParser::new(script).expect("Lexer");
    let ast = parser
        .parse()
        .unwrap_or_else(|e| panic!("Issue #136: function-body variant must parse. Error: {e:?}"));
    assert!(!ast.statements.is_empty());
}

// =============================================================================
// End-to-end: parse -> codegen round-trip
// =============================================================================

/// Full round-trip: parse `if ! command -v` and regenerate purified POSIX output.
#[test]
fn test_issue_136_roundtrip_if_not_command_v() {
    let input = "#!/bin/bash\nif ! command -v apr >/dev/null 2>&1; then\n    echo 'not found'\n    exit 1\nfi";
    let mut parser = BashParser::new(input).expect("Lex");
    let ast = parser.parse().expect("Issue #136: parse must succeed");
    let output = generate_purified_bash(&ast);
    assert!(
        output.contains("if ! command -v apr"),
        "negated command preserved: {output}"
    );
    assert!(output.contains("then"), "then keyword: {output}");
    assert!(output.contains("fi"), "fi keyword: {output}");
}

/// Round-trip with variable argument
#[test]
fn test_issue_136_roundtrip_negated_command_v_variable() {
    let input = "#!/bin/bash\nif ! command -v \"$bin\" >/dev/null 2>&1; then\n    exit 1\nfi";
    let mut parser = BashParser::new(input).expect("Lex");
    let ast = parser.parse().expect("Issue #136: parse must succeed");
    let output = generate_purified_bash(&ast);
    assert!(
        output.contains("! command"),
        "negated command preserved: {output}"
    );
}
