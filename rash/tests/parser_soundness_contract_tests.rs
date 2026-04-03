#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]
//! Provable Contract Tests: parser-soundness-v1.yaml
//!
//! Each test attempts to FALSIFY a parser soundness claim.
//! The parser must correctly handle valid bash, preserve AST
//! constructs, and be deterministic.
//!
//! Reference: GH-183 (KZ-11: Missing provable contracts)

use bashrs::bash_parser::{BashParser, BashStmt};

/// Helper: parse bash source and return the AST statements
fn parse_ok(source: &str) -> Vec<BashStmt> {
    let mut parser = BashParser::new(source).expect("lexer should succeed");
    let ast = parser.parse().expect("parse should succeed");
    ast.statements
}

/// Helper: check if a specific statement variant exists
fn has_stmt<F>(stmts: &[BashStmt], pred: F) -> bool
where
    F: Fn(&BashStmt) -> bool,
{
    stmts.iter().any(&pred)
}

// ============================================================================
// F-PARSE-001..006: Valid bash parses correctly
// ============================================================================

/// F-PARSE-001: echo command
#[test]
fn falsify_PARSE_001_echo_command() {
    let stmts = parse_ok("echo \"hello world\"");
    assert!(
        has_stmt(&stmts, |s| matches!(s, BashStmt::Command { name, .. } if name == "echo")),
        "F-PARSE-001: echo must parse to Command {{ name: 'echo' }}"
    );
}

/// F-PARSE-002: variable assignment
#[test]
fn falsify_PARSE_002_assignment() {
    let stmts = parse_ok("x=42");
    assert!(
        has_stmt(&stmts, |s| matches!(s, BashStmt::Assignment { name, .. } if name == "x")),
        "F-PARSE-002: x=42 must parse to Assignment {{ name: 'x' }}"
    );
}

/// F-PARSE-003: if-then-else
#[test]
fn falsify_PARSE_003_if_else() {
    let stmts = parse_ok("if [ \"$x\" = \"1\" ]; then echo yes; else echo no; fi");
    assert!(
        has_stmt(&stmts, |s| matches!(s, BashStmt::If { else_block: Some(_), .. })),
        "F-PARSE-003: if/else must parse to If with else_block"
    );
}

/// F-PARSE-004: for loop
#[test]
fn falsify_PARSE_004_for_loop() {
    let stmts = parse_ok("for f in *.txt; do echo \"$f\"; done");
    assert!(
        has_stmt(&stmts, |s| matches!(s, BashStmt::For { variable, .. } if variable == "f")),
        "F-PARSE-004: for loop must parse to For {{ variable: 'f' }}"
    );
}

/// F-PARSE-005: function definition
#[test]
fn falsify_PARSE_005_function_def() {
    let stmts = parse_ok("greet() { echo hello; }");
    assert!(
        has_stmt(&stmts, |s| matches!(s, BashStmt::Function { name, .. } if name == "greet")),
        "F-PARSE-005: function must parse to Function {{ name: 'greet' }}"
    );
}

/// F-PARSE-006: pipeline
#[test]
fn falsify_PARSE_006_pipeline() {
    let stmts = parse_ok("ls -la | grep foo | wc -l");
    assert!(
        has_stmt(&stmts, |s| matches!(s, BashStmt::Pipeline { .. })),
        "F-PARSE-006: pipeline must parse to Pipeline node, got: {:?}",
        stmts.iter().map(std::mem::discriminant).collect::<Vec<_>>()
    );
}

// ============================================================================
// F-PARSE-007: Determinism
// ============================================================================

/// F-PARSE-007: same input → same AST
#[test]
fn falsify_PARSE_007_determinism() {
    let input = "x=1\nif [ \"$x\" ]; then echo yes; fi";
    let stmts1 = parse_ok(input);
    let stmts2 = parse_ok(input);
    assert_eq!(
        stmts1, stmts2,
        "F-PARSE-007: parse(input) must equal parse(input)"
    );
}

/// F-PARSE-007 variant: complex script
#[test]
fn falsify_PARSE_007_determinism_complex() {
    let input = "#!/bin/bash\nfor f in *.sh; do\n  echo \"$f\"\ndone\ngreet() { echo hi; }";
    let stmts1 = parse_ok(input);
    let stmts2 = parse_ok(input);
    assert_eq!(stmts1, stmts2, "F-PARSE-007: complex parse must be deterministic");
}

// ============================================================================
// F-PARSE-008..009: Edge cases
// ============================================================================

/// F-PARSE-008: empty input → empty AST
#[test]
fn falsify_PARSE_008_empty_input() {
    let stmts = parse_ok("");
    assert!(
        stmts.is_empty(),
        "F-PARSE-008: empty input must produce empty AST, got {} statements",
        stmts.len()
    );
}

/// F-PARSE-009: comment-only input
#[test]
fn falsify_PARSE_009_comment_only() {
    // Should parse without error — comments are valid bash
    let mut parser = BashParser::new("# this is a comment").expect("lexer ok");
    let result = parser.parse();
    assert!(
        result.is_ok(),
        "F-PARSE-009: comment-only input must parse successfully"
    );
}

// ============================================================================
// F-PARSE-010: Statement count preservation
// ============================================================================

/// F-PARSE-010: multi-line preserves statement count
#[test]
fn falsify_PARSE_010_statement_count() {
    let stmts = parse_ok("x=1\ny=2\necho $x $y");
    assert_eq!(
        stmts.len(),
        3,
        "F-PARSE-010: 3 statements must produce 3 AST nodes, got {}",
        stmts.len()
    );
}

// ============================================================================
// F-PARSE-011..012: Construct preservation
// ============================================================================

/// F-PARSE-011: while loop
#[test]
fn falsify_PARSE_011_while_loop() {
    let stmts = parse_ok("while true; do sleep 1; done");
    assert!(
        has_stmt(&stmts, |s| matches!(s, BashStmt::While { .. })),
        "F-PARSE-011: while loop must parse to While node"
    );
}

/// F-PARSE-012: case statement
#[test]
fn falsify_PARSE_012_case_statement() {
    let stmts = parse_ok("case \"$1\" in start) echo go;; stop) echo halt;; esac");
    let has_case = has_stmt(&stmts, |s| {
        if let BashStmt::Case { arms, .. } = s {
            arms.len() == 2
        } else {
            false
        }
    });
    assert!(
        has_case,
        "F-PARSE-012: case must parse to Case with 2 arms"
    );
}
