#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_ASSIGN_COV_021_empty_assignment_before_comment() {
    let script = "x= # comment";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "x"));
    assert!(
        has_assignment,
        "Should parse empty assignment before comment"
    );
}

#[test]
fn test_ASSIGN_COV_022_empty_assignment_before_and() {
    let script = "x= && echo ok";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    assert!(
        !ast.statements.is_empty(),
        "Should parse empty assignment before &&"
    );
}

#[test]
fn test_ASSIGN_COV_023_empty_assignment_before_or() {
    let script = "x= || echo fail";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    assert!(
        !ast.statements.is_empty(),
        "Should parse empty assignment before ||"
    );
}

// ===== parse_assignment coverage: exported keyword-as-variable =====

#[test]
fn test_ASSIGN_COV_024_exported_assignment() {
    let script = "export MY_VAR=hello";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    assert!(
        !ast.statements.is_empty(),
        "Should parse exported assignment"
    );
}
