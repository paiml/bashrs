//! Comprehensive tests for the `parser_arith` module.
//!
//! Covers tokenization, precedence-climbing parsing, operator lowering,
//! edge cases, and error handling for bash arithmetic expressions.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::ast::ArithExpr;
use super::parser::BashParser;
use super::parser_arith::ArithToken;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse an arithmetic expression string into an `ArithExpr`.
fn parse_arith(input: &str) -> ArithExpr {
    let mut parser = BashParser::new("echo x").unwrap();
    parser.parse_arithmetic_expr(input).unwrap()
}

/// Tokenize an arithmetic expression string.
fn tokenize(input: &str) -> Vec<ArithToken> {
    let parser = BashParser::new("echo x").unwrap();
    parser.tokenize_arithmetic(input).unwrap()
}

/// Parse expecting an error.
fn parse_arith_err(input: &str) -> super::parser::ParseError {
    let mut parser = BashParser::new("echo x").unwrap();
    parser.parse_arithmetic_expr(input).unwrap_err()
}

// ===========================================================================
// Basic arithmetic: number and variable literals
// ===========================================================================

#[test]
fn test_arith_number() {
    assert_eq!(parse_arith("42"), ArithExpr::Number(42));
}

#[test]
fn test_arith_variable() {
    assert_eq!(parse_arith("x"), ArithExpr::Variable("x".to_string()),);
}

#[test]
fn test_arith_addition() {
    assert_eq!(
        parse_arith("1+2"),
        ArithExpr::Add(
            Box::new(ArithExpr::Number(1)),
            Box::new(ArithExpr::Number(2)),
        ),
    );
}

#[test]
fn test_arith_subtraction() {
    assert_eq!(
        parse_arith("5-3"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Number(5)),
            Box::new(ArithExpr::Number(3)),
        ),
    );
}

#[test]
fn test_arith_multiplication() {
    assert_eq!(
        parse_arith("3*4"),
        ArithExpr::Mul(
            Box::new(ArithExpr::Number(3)),
            Box::new(ArithExpr::Number(4)),
        ),
    );
}

#[test]
fn test_arith_division() {
    assert_eq!(
        parse_arith("10/2"),
        ArithExpr::Div(
            Box::new(ArithExpr::Number(10)),
            Box::new(ArithExpr::Number(2)),
        ),
    );
}

#[test]
fn test_arith_modulo() {
    assert_eq!(
        parse_arith("7%3"),
        ArithExpr::Mod(
            Box::new(ArithExpr::Number(7)),
            Box::new(ArithExpr::Number(3)),
        ),
    );
}

// ===========================================================================
// Operator precedence
// ===========================================================================

#[test]
fn test_arith_precedence_mul_over_add() {
    // 1+2*3 => Add(1, Mul(2, 3))  -- multiplication binds tighter
    assert_eq!(
        parse_arith("1+2*3"),
        ArithExpr::Add(
            Box::new(ArithExpr::Number(1)),
            Box::new(ArithExpr::Mul(
                Box::new(ArithExpr::Number(2)),
                Box::new(ArithExpr::Number(3)),
            )),
        ),
    );
}

#[test]
fn test_arith_precedence_parens() {
    // (1+2)*3 => Mul(Add(1, 2), 3)  -- parentheses override
    assert_eq!(
        parse_arith("(1+2)*3"),
        ArithExpr::Mul(
            Box::new(ArithExpr::Add(
                Box::new(ArithExpr::Number(1)),
                Box::new(ArithExpr::Number(2)),
            )),
            Box::new(ArithExpr::Number(3)),
        ),
    );
}

#[test]
fn test_arith_left_assoc() {
    // 1-2-3 => Sub(Sub(1, 2), 3)  -- left-to-right associativity
    assert_eq!(
        parse_arith("1-2-3"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Sub(
                Box::new(ArithExpr::Number(1)),
                Box::new(ArithExpr::Number(2)),
            )),
            Box::new(ArithExpr::Number(3)),
        ),
    );
}

// ===========================================================================
// Comparison operators (all lowered to Sub in the AST)
// ===========================================================================

#[test]
fn test_arith_lt() {
    // x < 10 => Sub(Variable("x"), Number(10))
    assert_eq!(
        parse_arith("x<10"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Variable("x".to_string())),
            Box::new(ArithExpr::Number(10)),
        ),
    );
}

#[test]
fn test_arith_le() {
    assert_eq!(
        parse_arith("x<=10"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Variable("x".to_string())),
            Box::new(ArithExpr::Number(10)),
        ),
    );
}

#[test]
fn test_arith_gt() {
    assert_eq!(
        parse_arith("x>0"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Variable("x".to_string())),
            Box::new(ArithExpr::Number(0)),
        ),
    );
}

#[test]
fn test_arith_ge() {
    assert_eq!(
        parse_arith("x>=5"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Variable("x".to_string())),
            Box::new(ArithExpr::Number(5)),
        ),
    );
}

#[test]
fn test_arith_eq() {
    // x == 0 => Sub(Variable("x"), Number(0))
    assert_eq!(
        parse_arith("x==0"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Variable("x".to_string())),
            Box::new(ArithExpr::Number(0)),
        ),
    );
}

#[test]
fn test_arith_ne() {
    assert_eq!(
        parse_arith("x!=0"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Variable("x".to_string())),
            Box::new(ArithExpr::Number(0)),
        ),
    );
}

// ===========================================================================
// Bitwise operators
// ===========================================================================

#[test]
fn test_arith_bitand() {
    // x & 0xFF => Mul(Variable("x"), Number(255))  -- bitwise AND lowered to Mul
    // 0xFF is hex 255
    assert_eq!(
        parse_arith("x&0xFF"),
        ArithExpr::Mul(
            Box::new(ArithExpr::Variable("x".to_string())),
            Box::new(ArithExpr::Number(255)),
        ),
    );
}

#[test]
fn test_arith_bitor() {
    // x | 1 => Add(Variable("x"), Number(1))  -- bitwise OR lowered to Add
    assert_eq!(
        parse_arith("x|1"),
        ArithExpr::Add(
            Box::new(ArithExpr::Variable("x".to_string())),
            Box::new(ArithExpr::Number(1)),
        ),
    );
}

#[test]
fn test_arith_bitxor() {
    // x ^ y => Sub(Variable("x"), Variable("y"))  -- bitwise XOR lowered to Sub
    assert_eq!(
        parse_arith("x^y"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Variable("x".to_string())),
            Box::new(ArithExpr::Variable("y".to_string())),
        ),
    );
}

#[test]
fn test_arith_shl() {
    // 1 << 4 => Mul(Number(1), Number(4))  -- shift left lowered to Mul
    assert_eq!(
        parse_arith("1<<4"),
        ArithExpr::Mul(
            Box::new(ArithExpr::Number(1)),
            Box::new(ArithExpr::Number(4)),
        ),
    );
}

#[test]
fn test_arith_shr() {
    // x >> 1 => Div(Variable("x"), Number(1))  -- shift right lowered to Div
    assert_eq!(
        parse_arith("x>>1"),
        ArithExpr::Div(
            Box::new(ArithExpr::Variable("x".to_string())),
            Box::new(ArithExpr::Number(1)),
        ),
    );
}

// ===========================================================================
// Logical operators
// ===========================================================================

#[test]
fn test_arith_logical_and() {
    // x && y => Mul(Variable("x"), Variable("y"))  -- logical AND lowered to Mul
    assert_eq!(
        parse_arith("x&&y"),
        ArithExpr::Mul(
            Box::new(ArithExpr::Variable("x".to_string())),
            Box::new(ArithExpr::Variable("y".to_string())),
        ),
    );
}

#[test]
fn test_arith_logical_or() {
    // x || y => Add(Variable("x"), Variable("y"))  -- logical OR lowered to Add
    assert_eq!(
        parse_arith("x||y"),
        ArithExpr::Add(
            Box::new(ArithExpr::Variable("x".to_string())),
            Box::new(ArithExpr::Variable("y".to_string())),
        ),
    );
}

#[test]
fn test_arith_logical_not() {
    // !x => Sub(Number(-1), Variable("x"))  -- logical NOT lowered like bitwise NOT
    assert_eq!(
        parse_arith("!x"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Number(-1)),
            Box::new(ArithExpr::Variable("x".to_string())),
        ),
    );
}

// ===========================================================================
// Ternary operator
// ===========================================================================

#[test]
fn test_arith_ternary() {
    // x ? 1 : 0 => Add(Mul(x, 1), Mul(Sub(1, x), 0))
    assert_eq!(
        parse_arith("x?1:0"),
        ArithExpr::Add(
            Box::new(ArithExpr::Mul(
                Box::new(ArithExpr::Variable("x".to_string())),
                Box::new(ArithExpr::Number(1)),
            )),
            Box::new(ArithExpr::Mul(
                Box::new(ArithExpr::Sub(
                    Box::new(ArithExpr::Number(1)),
                    Box::new(ArithExpr::Variable("x".to_string())),
                )),
                Box::new(ArithExpr::Number(0)),
            )),
        ),
    );
}

// ===========================================================================
// Exponentiation
// ===========================================================================

#[test]

include!("parser_arith_tests_incl2.rs");
