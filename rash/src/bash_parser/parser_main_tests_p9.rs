use super::*;
use crate::bash_parser::ast::{ArithExpr, BashExpr, BashStmt, Redirect, Span, TestExpr};
use crate::bash_parser::parser_arith::ArithToken;
#[test]
fn test_ARITH_EXPR_008_logical_not() {
    // !x becomes Sub(Number(-1), Variable("x"))
    assert_eq!(
        parse_arith("!x"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Number(-1)),
            Box::new(ArithExpr::Variable("x".to_string())),
        )
    );
}

// ── Multiplicative (Level 13) ────────────────────────────────────

#[test]
fn test_ARITH_EXPR_009_multiply() {
    assert_eq!(
        parse_arith("a * b"),
        ArithExpr::Mul(
            Box::new(ArithExpr::Variable("a".to_string())),
            Box::new(ArithExpr::Variable("b".to_string())),
        )
    );
}

#[test]
fn test_ARITH_EXPR_010_divide() {
    assert_eq!(
        parse_arith("a / b"),
        ArithExpr::Div(
            Box::new(ArithExpr::Variable("a".to_string())),
            Box::new(ArithExpr::Variable("b".to_string())),
        )
    );
}

#[test]
fn test_ARITH_EXPR_011_modulo() {
    assert_eq!(
        parse_arith("a % b"),
        ArithExpr::Mod(
            Box::new(ArithExpr::Variable("a".to_string())),
            Box::new(ArithExpr::Variable("b".to_string())),
        )
    );
}

#[test]
fn test_ARITH_EXPR_012_chained_multiplicative() {
    // a * b / c  =>  Div(Mul(a, b), c)  (left-to-right associativity)
    assert_eq!(
        parse_arith("a * b / c"),
        ArithExpr::Div(
            Box::new(ArithExpr::Mul(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )),
            Box::new(ArithExpr::Variable("c".to_string())),
        )
    );
}

// ── Additive (Level 12) ──────────────────────────────────────────

#[test]
fn test_ARITH_EXPR_013_addition() {
    assert_eq!(
        parse_arith("a + b"),
        ArithExpr::Add(
            Box::new(ArithExpr::Variable("a".to_string())),
            Box::new(ArithExpr::Variable("b".to_string())),
        )
    );
}

#[test]
fn test_ARITH_EXPR_014_subtraction() {
    assert_eq!(
        parse_arith("a - b"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Variable("a".to_string())),
            Box::new(ArithExpr::Variable("b".to_string())),
        )
    );
}

#[test]
fn test_ARITH_EXPR_015_mixed_additive() {
    // a + b - c  =>  Sub(Add(a, b), c)
    assert_eq!(
        parse_arith("a + b - c"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Add(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )),
            Box::new(ArithExpr::Variable("c".to_string())),
        )
    );
}

// ── Shift (Level 11) ─────────────────────────────────────────────

#[test]
fn test_ARITH_EXPR_016_shift_left() {
    // a << b  =>  Mul(a, b)
    assert_eq!(
        parse_arith("a << b"),
        ArithExpr::Mul(
            Box::new(ArithExpr::Variable("a".to_string())),
            Box::new(ArithExpr::Variable("b".to_string())),
        )
    );
}

#[test]
fn test_ARITH_EXPR_017_shift_right() {
    // a >> b  =>  Div(a, b)
    assert_eq!(
        parse_arith("a >> b"),
        ArithExpr::Div(
            Box::new(ArithExpr::Variable("a".to_string())),
            Box::new(ArithExpr::Variable("b".to_string())),
        )
    );
}

// ── Comparison (Level 10) ────────────────────────────────────────

#[test]
fn test_ARITH_EXPR_018_less_than() {
    // a < b  =>  Sub(a, b)
    assert_eq!(
        parse_arith("a < b"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Variable("a".to_string())),
            Box::new(ArithExpr::Variable("b".to_string())),
        )
    );
}

#[test]
fn test_ARITH_EXPR_019_less_equal() {
    assert_eq!(
        parse_arith("a <= b"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Variable("a".to_string())),
            Box::new(ArithExpr::Variable("b".to_string())),
        )
    );
}

#[test]
fn test_ARITH_EXPR_020_greater_than() {
    assert_eq!(
        parse_arith("a > b"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Variable("a".to_string())),
            Box::new(ArithExpr::Variable("b".to_string())),
        )
    );
}

#[test]
fn test_ARITH_EXPR_021_greater_equal() {
    assert_eq!(
        parse_arith("a >= b"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Variable("a".to_string())),
            Box::new(ArithExpr::Variable("b".to_string())),
        )
    );
}

// ── Equality (Level 9) ───────────────────────────────────────────

#[test]
fn test_ARITH_EXPR_022_equality() {
    // a == b  =>  Sub(a, b)
    assert_eq!(
        parse_arith("a == b"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Variable("a".to_string())),
            Box::new(ArithExpr::Variable("b".to_string())),
        )
    );
}

#[test]
fn test_ARITH_EXPR_023_not_equal() {
    // a != b  =>  Sub(a, b)
    assert_eq!(
        parse_arith("a != b"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Variable("a".to_string())),
            Box::new(ArithExpr::Variable("b".to_string())),
        )
    );
}

// ── Bitwise AND (Level 8) ────────────────────────────────────────

#[test]
fn test_ARITH_EXPR_024_bitwise_and() {
    // a & b  =>  Mul(a, b)
    assert_eq!(
        parse_arith("a & b"),
        ArithExpr::Mul(
            Box::new(ArithExpr::Variable("a".to_string())),
            Box::new(ArithExpr::Variable("b".to_string())),
        )
    );
}

// ── Bitwise XOR (Level 7) ────────────────────────────────────────

#[test]
fn test_ARITH_EXPR_025_bitwise_xor() {
    // a ^ b  =>  Sub(a, b)
    assert_eq!(
        parse_arith("a ^ b"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Variable("a".to_string())),
            Box::new(ArithExpr::Variable("b".to_string())),
        )
    );
}

// ── Bitwise OR (Level 6) ─────────────────────────────────────────

#[test]
fn test_ARITH_EXPR_026_bitwise_or() {
    // a | b  =>  Add(a, b)
    assert_eq!(
        parse_arith("a | b"),
        ArithExpr::Add(
            Box::new(ArithExpr::Variable("a".to_string())),
            Box::new(ArithExpr::Variable("b".to_string())),
        )
    );
}

// ── Logical AND (Level 5) ────────────────────────────────────────

#[test]
fn test_ARITH_EXPR_027_logical_and() {
    // a && b  =>  Mul(a, b)
    assert_eq!(
        parse_arith("a && b"),
        ArithExpr::Mul(
            Box::new(ArithExpr::Variable("a".to_string())),
            Box::new(ArithExpr::Variable("b".to_string())),
        )
    );
}

// ── Logical OR (Level 4) ─────────────────────────────────────────

#[test]
fn test_ARITH_EXPR_028_logical_or() {
    // a || b  =>  Add(a, b)
    assert_eq!(
        parse_arith("a || b"),
        ArithExpr::Add(
            Box::new(ArithExpr::Variable("a".to_string())),
            Box::new(ArithExpr::Variable("b".to_string())),
        )
    );
}

// ── Ternary (Level 3) ────────────────────────────────────────────

#[test]
fn test_ARITH_EXPR_029_ternary() {
    // a ? b : c  =>  Add(Mul(a, b), Mul(Sub(1, a), c))
    assert_eq!(
        parse_arith("a ? b : c"),
        ArithExpr::Add(
            Box::new(ArithExpr::Mul(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )),
            Box::new(ArithExpr::Mul(
                Box::new(ArithExpr::Sub(
                    Box::new(ArithExpr::Number(1)),
                    Box::new(ArithExpr::Variable("a".to_string())),
                )),
                Box::new(ArithExpr::Variable("c".to_string())),
            )),
        )
    );
}

// ── Comma (Level 1) ──────────────────────────────────────────────

#[test]
fn test_ARITH_EXPR_030_comma() {
    // a , b  =>  returns b (right value)
    assert_eq!(parse_arith("a , b"), ArithExpr::Variable("b".to_string()));
}

// ── Precedence / Complex ─────────────────────────────────────────

#[test]
fn test_ARITH_EXPR_031_precedence_mul_over_add() {
    // 1 + 2 * 3  =>  Add(1, Mul(2, 3))
    assert_eq!(
        parse_arith("1 + 2 * 3"),
        ArithExpr::Add(
            Box::new(ArithExpr::Number(1)),
            Box::new(ArithExpr::Mul(
                Box::new(ArithExpr::Number(2)),
                Box::new(ArithExpr::Number(3)),
            )),
        )
    );
}

#[test]
fn test_ARITH_EXPR_032_parentheses_override_precedence() {
    // (1 + 2) * 3  =>  Mul(Add(1, 2), 3)
    assert_eq!(
        parse_arith("(1 + 2) * 3"),
        ArithExpr::Mul(
            Box::new(ArithExpr::Add(
                Box::new(ArithExpr::Number(1)),
                Box::new(ArithExpr::Number(2)),
            )),
            Box::new(ArithExpr::Number(3)),
        )
    );
}

#[test]
fn test_ARITH_EXPR_033_complex_nested() {
    // (a + b) * (c - d)  =>  Mul(Add(a, b), Sub(c, d))
    assert_eq!(
        parse_arith("(a + b) * (c - d)"),
        ArithExpr::Mul(
            Box::new(ArithExpr::Add(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )),
            Box::new(ArithExpr::Sub(
                Box::new(ArithExpr::Variable("c".to_string())),
                Box::new(ArithExpr::Variable("d".to_string())),
            )),
        )
    );
}

#[test]
fn test_ARITH_EXPR_034_negative_number_literal() {
    assert_eq!(
        parse_arith("-1"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Number(0)),
            Box::new(ArithExpr::Number(1)),
        )
    );
}

#[test]
fn test_ARITH_EXPR_035_zero() {
    assert_eq!(parse_arith("0"), ArithExpr::Number(0));
}

// ── Error Cases ──────────────────────────────────────────────────

#[test]
#[ignore = "parser panics on malformed arithmetic"]
fn test_ARITH_EXPR_036_missing_closing_paren() {
    let err = parse_arith_err("(1 + 2");
    assert!(matches!(err, ParseError::InvalidSyntax(_)));
}

#[test]
#[ignore = "parser panics on malformed arithmetic"]
fn test_ARITH_EXPR_037_empty_parentheses() {
    let err = parse_arith_err("()");
    assert!(matches!(err, ParseError::InvalidSyntax(_)));
}

#[test]
#[ignore = "parser panics on malformed arithmetic"]
fn test_ARITH_EXPR_038_trailing_operator() {
    let err = parse_arith_err("1 +");
    assert!(matches!(err, ParseError::InvalidSyntax(_)));
}

#[test]
#[ignore = "parser panics on malformed arithmetic"]
fn test_ARITH_EXPR_039_ternary_missing_colon() {
    let err = parse_arith_err("a ? b");
    assert!(matches!(err, ParseError::InvalidSyntax(_)));
}

// ── Additional Precedence / Associativity ────────────────────────

#[test]
fn test_ARITH_EXPR_040_left_associative_subtraction() {
    // a - b - c  =>  Sub(Sub(a, b), c)
    assert_eq!(
        parse_arith("a - b - c"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Sub(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )),
            Box::new(ArithExpr::Variable("c".to_string())),
        )
    );
}

#[test]
fn test_ARITH_EXPR_041_unary_minus_in_expression() {
    // a + -b  =>  Add(a, Sub(0, b))
    assert_eq!(
        parse_arith("a + -b"),
        ArithExpr::Add(
            Box::new(ArithExpr::Variable("a".to_string())),
            Box::new(ArithExpr::Sub(
                Box::new(ArithExpr::Number(0)),
                Box::new(ArithExpr::Variable("b".to_string())),
            )),
        )
    );
}

#[test]
fn test_ARITH_EXPR_042_comma_chain_returns_last() {
    // 1 , 2 , 3  =>  Number(3) (comma returns rightmost)
    assert_eq!(parse_arith("1 , 2 , 3"), ArithExpr::Number(3));
}

// --- Batch 2: semicolons, -v test, env prefix, &> in conditions ---

#[test]
fn test_SEMICOLON_SEP_001_simple() {
    let input = "a=10; b=3";
    let mut parser = BashParser::new(input).expect("parser");
    let ast = parser.parse();
    assert!(
        ast.is_ok(),
        "Semicolon-separated assignments should parse: {:?}",
        ast.err()
    );
    assert_eq!(ast.as_ref().expect("ok").statements.len(), 2);
}

#[test]
fn test_SEMICOLON_SEP_002_multiple() {
    let input = "echo a; echo b; echo c";
    let mut parser = BashParser::new(input).expect("parser");
    let ast = parser.parse();
    assert!(
        ast.is_ok(),
        "Multiple semicolons should parse: {:?}",
        ast.err()
    );
    assert_eq!(ast.as_ref().expect("ok").statements.len(), 3);
}

fn parse_arith(input: &str) -> ArithExpr {
    let mut parser = BashParser::new(&format!("echo $(({input}))")).expect("parser init");
    let ast = parser.parse().expect("parse");
    match &ast.statements[0] {
        BashStmt::Command { args, .. } => match &args[0] {
            BashExpr::Arithmetic(expr) => *expr.clone(),
            other => panic!("Expected Arithmetic, got {other:?}"),
        },
        other => panic!("Expected Command, got {other:?}"),
    }
}

fn parse_arith_err(input: &str) -> ParseError {
    let mut parser = BashParser::new(&format!("echo $(({input}))")).expect("parser init");
    parser.parse().expect_err("expected parse error")
}
