//! Coverage tests for bash_parser/parser_control.rs uncovered branches.
//!
//! Targets: if/elif/else with redirects, while/until with semicolons and
//! redirects, brace group/subshell with redirects, coproc named/unnamed,
//! standalone [ ] and [[ ]] test commands with combinators, for loops
//! (C-style, single/multi item, newline terminator), select statement,
//! case parsing (patterns, alternates, body semicolons, terminators).
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::ast::{BashExpr, BashStmt};
use super::parser::BashParser;

/// Helper: parse input and return the AST, panicking on failure.
fn parse_ok(input: &str) -> super::ast::BashAst {
    let mut p = BashParser::new(input).unwrap();
    p.parse().unwrap()
}

/// Helper: parse input, accepting either Ok or Err (no panic).
fn parse_no_panic(input: &str) {
    let _ = BashParser::new(input).and_then(|mut p| p.parse());
}

// ---------------------------------------------------------------------------
// parse_if — elif, else, redirect branches
// ---------------------------------------------------------------------------

#[test]
fn test_if_with_elif_redirect_suppression() {
    let input = "if [ -f /a ]; then\n  echo a\nelif [ -f /b ] 2>/dev/null; then\n  echo b\nfi";
    assert!(BashParser::new(input).and_then(|mut p| p.parse()).is_ok());
}

#[test]
fn test_if_with_else_block() {
    let ast = parse_ok("if [ -f /a ]; then\n  echo yes\nelse\n  echo no\nfi");
    if let BashStmt::If { else_block, .. } = &ast.statements[0] {
        assert!(else_block.is_some());
    }
}

#[test]
fn test_if_trailing_redirect() {
    parse_no_panic("if true; then echo hi; fi > /tmp/log");
}

#[test]
fn test_if_semicolon_before_then() {
    let ast = parse_ok("if [ 1 = 1 ] ; then echo ok ; fi");
    assert!(ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::If { .. })));
}

#[test]
fn test_if_multiple_elif_blocks() {
    let input = "if [ $x = 1 ]; then\n  echo one\nelif [ $x = 2 ]; then\n  echo two\nelif [ $x = 3 ]; then\n  echo three\nelse\n  echo other\nfi";
    let ast = parse_ok(input);
    if let BashStmt::If { elif_blocks, .. } = &ast.statements[0] {
        assert_eq!(elif_blocks.len(), 2);
    }
}

// ---------------------------------------------------------------------------
// parse_while — semicolons, redirects
// ---------------------------------------------------------------------------

#[test]
fn test_while_variants() {
    assert!(BashParser::new("while [ $i -lt 10 ]; do echo $i; done")
        .and_then(|mut p| p.parse())
        .is_ok());
    assert!(BashParser::new("while [ $i -lt 5 ]\ndo\n  echo $i\ndone")
        .and_then(|mut p| p.parse())
        .is_ok());
    parse_no_panic("while read line; do echo $line; done < /tmp/in");
    parse_no_panic("while [ -f /tmp/lock ] 2>/dev/null; do sleep 1; done");
}

// ---------------------------------------------------------------------------
// parse_until — semicolons, redirects
// ---------------------------------------------------------------------------

#[test]
fn test_until_variants() {
    assert!(BashParser::new("until [ $done = yes ]; do echo w; done")
        .and_then(|mut p| p.parse())
        .is_ok());
    assert!(
        BashParser::new("until [ -f /tmp/ready ]\ndo\n  sleep 1\ndone")
            .and_then(|mut p| p.parse())
            .is_ok()
    );
    parse_no_panic("until [ -f /tmp/done ] 2>/dev/null; do sleep 1; done");
}

// ---------------------------------------------------------------------------
// parse_brace_group and parse_subshell — trailing redirects
// ---------------------------------------------------------------------------

#[test]
fn test_brace_group_redirects() {
    parse_no_panic("{ echo a; echo b; } > /tmp/out");
    parse_no_panic("{ echo a; echo b; } 2>/dev/null");
    parse_no_panic("{ echo out; echo err >&2; } > /tmp/out 2>/dev/null");
}

#[test]
fn test_subshell_redirects() {
    parse_no_panic("(echo a; echo b) > /tmp/out");
    parse_no_panic("(echo a; echo b) 2>/dev/null");
    parse_no_panic("(echo l1; echo l2) >> /tmp/log");
}

// ---------------------------------------------------------------------------
// parse_coproc — named and unnamed
// ---------------------------------------------------------------------------

#[test]
fn test_coproc_unnamed() {
    let result = BashParser::new("coproc { cat; }").and_then(|mut p| p.parse());
    if let Ok(ast) = &result {
        if let BashStmt::Coproc { name, .. } = &ast.statements[0] {
            assert!(name.is_none());
        }
    }
}

#[test]
fn test_coproc_named() {
    let result = BashParser::new("coproc mycat { cat; }").and_then(|mut p| p.parse());
    if let Ok(ast) = &result {
        if let BashStmt::Coproc { name, .. } = &ast.statements[0] {
            assert_eq!(name.as_deref(), Some("mycat"));
        }
    }
}

#[test]
fn test_coproc_with_newlines() {
    parse_no_panic("coproc\n{\n  cat\n}");
}

// ---------------------------------------------------------------------------
// Standalone [ ] and [[ ]] test commands with combinators
// ---------------------------------------------------------------------------

#[test]
fn test_standalone_test_commands() {
    parse_no_panic("[ -f /tmp/test ] && echo exists");
    parse_no_panic("[ -f /a -a -d /b ] && echo both");
    parse_no_panic("[ -f /a -o -f /b ] && echo one");
}

#[test]
fn test_standalone_extended_test_commands() {
    parse_no_panic("[[ -d /tmp ]] && echo dir");
    parse_no_panic("[[ -f /a && -d /b ]] && echo both");
    parse_no_panic("[[ -f /a || -d /b ]] && echo one");
}

// ---------------------------------------------------------------------------
// parse_for — single/multi items, newline, C-style
// ---------------------------------------------------------------------------

#[test]
fn test_for_single_item() {
    let ast = parse_ok("for x in items; do echo $x; done");
    if let BashStmt::For { items, .. } = &ast.statements[0] {
        assert!(!matches!(items, BashExpr::Array(_)));
    }
}

#[test]
fn test_for_multiple_items() {
    let ast = parse_ok("for x in a b c d; do echo $x; done");
    if let BashStmt::For { items, .. } = &ast.statements[0] {
        assert!(matches!(items, BashExpr::Array(_)));
    }
}

#[test]
fn test_for_items_newline_terminated() {
    assert!(BashParser::new("for x in a b c\ndo\n  echo $x\ndone")
        .and_then(|mut p| p.parse())
        .is_ok());
}

#[test]
fn test_for_with_variable_and_cmd_subst() {
    assert!(BashParser::new("for f in $FILES; do echo $f; done")
        .and_then(|mut p| p.parse())
        .is_ok());
    parse_no_panic("for f in $(ls); do echo $f; done");
}

#[test]
fn test_for_c_style_from_arithmetic_token() {
    parse_no_panic("for ((i=0; i<10; i++)); do echo $i; done");
}

#[test]
fn test_for_c_style_parts_parsing() {
    let result =
        BashParser::new("for ((x=1; x<=5; x++)); do echo $x; done").and_then(|mut p| p.parse());
    if let Ok(ast) = &result {
        if let BashStmt::ForCStyle {
            init,
            condition,
            increment,
            ..
        } = &ast.statements[0]
        {
            assert!(!init.is_empty());
            assert!(!condition.is_empty());
            assert!(!increment.is_empty());
        }
    }
}

#[test]
fn test_for_c_style_operators() {
    // Various operator tokens inside (( )): <=, >=, ==, !=, $var
    parse_no_panic("for ((i=0; i<=10; i++)); do echo $i; done");
    parse_no_panic("for ((i=10; i>=0; i--)); do echo $i; done");
    parse_no_panic("for ((i=0; i==0; i++)); do echo once; done");
    parse_no_panic("for ((i=0; i!=5; i++)); do echo $i; done");
    parse_no_panic("for ((i=0; i<$MAX; i++)); do echo $i; done");
}

#[test]
fn test_for_c_style_malformed() {
    parse_no_panic("for ((i=0)); do echo $i; done");
}

#[test]
fn test_for_error_missing_variable() {
    parse_no_panic("for in a b; do echo nope; done");
}

// ---------------------------------------------------------------------------
// parse_select — interactive menu
// ---------------------------------------------------------------------------

#[test]
fn test_select_single_item() {
    assert!(BashParser::new("select opt in options; do echo $opt; done")
        .and_then(|mut p| p.parse())
        .is_ok());
}

#[test]
fn test_select_multiple_items() {
    let ast = parse_ok("select opt in a b c d; do echo $opt; break; done");
    if let BashStmt::Select {
        variable, items, ..
    } = &ast.statements[0]
    {
        assert_eq!(variable, "opt");
        assert!(matches!(items, BashExpr::Array(_)));
    }
}

#[test]
fn test_select_newline_and_semicolon() {
    assert!(
        BashParser::new("select x in a b c\ndo\n  echo $x\n  break\ndone")
            .and_then(|mut p| p.parse())
            .is_ok()
    );
    assert!(
        BashParser::new("select color in red green blue; do echo $color; break; done")
            .and_then(|mut p| p.parse())
            .is_ok()
    );
}

#[test]
fn test_select_error_missing_variable() {
    parse_no_panic("select in a b; do echo nope; done");
}

// ---------------------------------------------------------------------------
// parse_case — patterns, alternates, body, terminators
// ---------------------------------------------------------------------------

#[test]
fn test_case_basic() {
    let ast = parse_ok("case $x in\n  a) echo a ;;\n  b) echo b ;;\nesac");
    if let BashStmt::Case { arms, .. } = &ast.statements[0] {
        assert_eq!(arms.len(), 2);
    }
}

#[test]
fn test_case_with_pipe_alternatives() {
    let ast = parse_ok("case $x in\n  a|b|c) echo abc ;;\n  *) echo other ;;\nesac");
    if let BashStmt::Case { arms, .. } = &ast.statements[0] {
        assert!(arms[0].patterns.len() >= 2);
    }
}

#[test]
fn test_case_pattern_types() {
    // Variable, number, glob, string patterns
    assert!(BashParser::new("case $x in\n  $E) echo m ;;\nesac")
        .and_then(|mut p| p.parse())
        .is_ok());
    assert!(
        BashParser::new("case $x in\n  1) echo one ;;\n  2) echo two ;;\nesac")
            .and_then(|mut p| p.parse())
            .is_ok()
    );
    assert!(
        BashParser::new("case $f in\n  *.txt) echo t ;;\n  *) echo o ;;\nesac")
            .and_then(|mut p| p.parse())
            .is_ok()
    );
    parse_no_panic("case $x in\n  \"hello\") echo g ;;\nesac");
}

#[test]
fn test_case_bracket_class_pattern() {
    parse_no_panic("case $x in\n  [0-9]*) echo d ;;\n  [a-z]*) echo a ;;\nesac");
}

#[test]
fn test_case_arm_body_variants() {
    // Multiple stmts, empty body, semicolon-separated stmts
    assert!(
        BashParser::new("case $x in\n  a) echo a; echo again ;;\nesac")
            .and_then(|mut p| p.parse())
            .is_ok()
    );
    assert!(
        BashParser::new("case $x in\n  skip) ;;\n  *) echo d ;;\nesac")
            .and_then(|mut p| p.parse())
            .is_ok()
    );
    assert!(
        BashParser::new("case $x in\n  a) echo one; echo two ;;\nesac")
            .and_then(|mut p| p.parse())
            .is_ok()
    );
}

#[test]
fn test_case_terminators() {
    // ;& and ;;& terminators
    parse_no_panic("case $x in\n  a) echo a ;& \n  b) echo b ;;\nesac");
    parse_no_panic("case $x in\n  a) echo a ;;& \n  b) echo b ;;\nesac");
}

#[test]
fn test_case_double_semicolon_tokens() {
    // Two consecutive Semicolon tokens as ;; (vs single identifier)
    assert!(
        BashParser::new("case $x in\na) echo a\n;;\nb) echo b\n;;\nesac")
            .and_then(|mut p| p.parse())
            .is_ok()
    );
}

#[test]
fn test_case_missing_esac_error() {
    let result = BashParser::new("case $x in\n  a) echo a ;;\n").and_then(|mut p| p.parse());
    assert!(result.is_err());
}

#[test]
fn test_case_no_terminator_before_esac() {
    parse_no_panic("case $x in\n  *) echo default\nesac");
}

#[test]
fn test_case_word_is_variable() {
    let ast = parse_ok("case $CMD in\n  start) echo s ;;\n  stop) echo t ;;\nesac");
    if let BashStmt::Case { word, .. } = &ast.statements[0] {
        assert!(matches!(word, BashExpr::Variable(_)));
    }
}

// ---------------------------------------------------------------------------
// Compound command nesting
// ---------------------------------------------------------------------------

#[test]
fn test_nested_control_flow() {
    parse_no_panic("while true; do\n  if [ $x = 5 ]; then break; fi\n  continue\ndone");
    assert!(BashParser::new(
        "for x in 1 2 3; do\n  if [ $x = 2 ]; then\n    echo found\n  fi\ndone"
    )
    .and_then(|mut p| p.parse())
    .is_ok());
    parse_no_panic(
        "while read cmd; do\n  case $cmd in\n    quit) break ;;\n    *) echo u ;;\n  esac\ndone",
    );
}
