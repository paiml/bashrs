#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_parse_basic_pipeline() {
    let script = "echo hello | grep hello";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    assert_eq!(ast.statements.len(), 1);

    // RED PHASE: This will fail - Pipeline variant doesn't exist yet
    if let BashStmt::Pipeline { commands, span: _ } = &ast.statements[0] {
        assert_eq!(commands.len(), 2, "Expected 2 commands in pipeline");

        // First command: echo hello
        if let BashStmt::Command {
            name: name1,
            args: args1,
            ..
        } = &commands[0]
        {
            assert_eq!(name1, "echo");
            assert_eq!(args1.len(), 1);
            if let BashExpr::Literal(arg) = &args1[0] {
                assert_eq!(arg, "hello");
            } else {
                panic!("Expected literal argument 'hello'");
            }
        } else {
            panic!("Expected Command statement for first command");
        }

        // Second command: grep hello
        if let BashStmt::Command {
            name: name2,
            args: args2,
            ..
        } = &commands[1]
        {
            assert_eq!(name2, "grep");
            assert_eq!(args2.len(), 1);
            if let BashExpr::Literal(arg) = &args2[0] {
                assert_eq!(arg, "hello");
            } else {
                panic!("Expected literal argument 'hello'");
            }
        } else {
            panic!("Expected Command statement for second command");
        }
    } else {
        panic!("Expected Pipeline statement");
    }
}

/// Issue #59: Test parsing nested quotes in command substitution
/// INPUT: OUTPUT="$(echo "test" 2>&1)"
/// BUG: Gets mangled to: OUTPUT='$(echo ' test ' 2>&1)'
/// EXPECTED: String contains command substitution, preserves inner quotes