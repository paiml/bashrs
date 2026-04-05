#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_ASSIGN_COV_007_keyword_while_as_variable_name() {
    let script = "while=value";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "while"));
    assert!(has_assignment, "Should parse 'while' as variable name");
}

#[test]
fn test_ASSIGN_COV_008_keyword_do_as_variable_name() {
    let script = "do=value";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "do"));
    assert!(has_assignment, "Should parse 'do' as variable name");
}

#[test]
fn test_ASSIGN_COV_009_keyword_done_as_variable_name() {
    let script = "done=value";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "done"));
    assert!(has_assignment, "Should parse 'done' as variable name");
}

#[test]
fn test_ASSIGN_COV_010_keyword_case_as_variable_name() {
    let script = "case=value";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "case"));
    assert!(has_assignment, "Should parse 'case' as variable name");
}

#[test]
fn test_ASSIGN_COV_011_keyword_esac_as_variable_name() {
    let script = "esac=value";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "esac"));
    assert!(has_assignment, "Should parse 'esac' as variable name");
}

#[test]
fn test_ASSIGN_COV_012_keyword_in_as_variable_name() {
    let script = "in=value";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "in"));
    assert!(has_assignment, "Should parse 'in' as variable name");
}

#[test]
fn test_ASSIGN_COV_013_keyword_function_as_variable_name() {
    let script = "function=value";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "function"));
    assert!(has_assignment, "Should parse 'function' as variable name");
}

#[test]
fn test_ASSIGN_COV_014_keyword_return_as_variable_name() {
    let script = "return=value";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "return"));
    assert!(has_assignment, "Should parse 'return' as variable name");
}

// ===== parse_assignment coverage: array element assignment =====

#[test]
fn test_ASSIGN_COV_015_array_element_number_index() {
    // arr[0]=value
    let script = "arr[0]=hello";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_indexed_assignment = ast.statements.iter().any(|s| {
        matches!(s, BashStmt::Assignment { name, index: Some(idx), .. } if name == "arr" && idx == "0")
    });
    assert!(
        has_indexed_assignment,
        "Should parse array element assignment with number index"
    );
}

#[test]
fn test_ASSIGN_COV_016_array_element_identifier_index() {
    // arr[key]=value
    let script = "arr[key]=world";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_indexed_assignment = ast.statements.iter().any(|s| {
        matches!(s, BashStmt::Assignment { name, index: Some(idx), .. } if name == "arr" && idx == "key")
    });
    assert!(
        has_indexed_assignment,
        "Should parse array element assignment with identifier index"
    );
}

#[test]
fn test_ASSIGN_COV_017_array_element_string_index() {
    // arr["quoted"]=value
    let script = r#"arr["quoted"]=value"#;
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_indexed_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, index: Some(_), .. } if name == "arr"));
    assert!(
        has_indexed_assignment,
        "Should parse array element assignment with string index"
    );
}

#[test]
fn test_ASSIGN_COV_018_array_element_variable_index() {
    // arr[$i]=value
    let script = "arr[$i]=value";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_indexed_assignment = ast.statements.iter().any(|s| {
        matches!(s, BashStmt::Assignment { name, index: Some(idx), .. } if name == "arr" && idx == "$i")
    });
    assert!(
        has_indexed_assignment,
        "Should parse array element assignment with variable index"
    );
}

// ===== parse_assignment coverage: append operator += =====

#[test]
fn test_ASSIGN_COV_019_append_assignment() {
    // PATH+=/usr/local/bin (append operator)
    let script = "PATH+=/usr/local/bin";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    // Parser should produce an Assignment (or equivalent) for +=
    assert!(
        !ast.statements.is_empty(),
        "Should parse += append assignment"
    );
}

// ===== parse_assignment coverage: empty assignment before pipe/comment =====

#[test]
fn test_ASSIGN_COV_020_empty_assignment_before_pipe() {
    let script = "x= | cat";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    assert!(
        !ast.statements.is_empty(),
        "Should parse empty assignment before pipe"
    );
}

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
