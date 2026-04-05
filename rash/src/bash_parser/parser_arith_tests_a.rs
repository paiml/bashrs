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
#[test]
fn test_arith_subexpression_in_both_sides() {
    // (a+b)*(c-d) => Mul(Add(a,b), Sub(c,d))
    assert_eq!(
        parse_arith("(a+b)*(c-d)"),
        ArithExpr::Mul(
            Box::new(ArithExpr::Add(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )),
            Box::new(ArithExpr::Sub(
                Box::new(ArithExpr::Variable("c".to_string())),
                Box::new(ArithExpr::Variable("d".to_string())),
            )),
        ),
    );
}

// ===========================================================================
// Edge cases
// ===========================================================================

#[test]
fn test_arith_empty() {
    // Empty string yields an error from the parser (no primary token)
    let err = parse_arith_err("");
    assert!(
        matches!(err, super::parser::ParseError::InvalidSyntax(_)),
        "Empty input should produce InvalidSyntax, got: {err:?}",
    );
}

#[test]
fn test_arith_single_zero() {
    assert_eq!(parse_arith("0"), ArithExpr::Number(0));
}

#[test]
fn test_arith_large_number() {
    assert_eq!(parse_arith("999999"), ArithExpr::Number(999_999));
}

#[test]
fn test_arith_multi_digit() {
    assert_eq!(
        parse_arith("123+456"),
        ArithExpr::Add(
            Box::new(ArithExpr::Number(123)),
            Box::new(ArithExpr::Number(456)),
        ),
    );
}

#[test]
fn test_arith_hex_literal() {
    // 0xFF => 255
    assert_eq!(parse_arith("0xFF"), ArithExpr::Number(255));
}

#[test]
fn test_arith_octal_literal() {
    // 010 => 8  (octal)
    assert_eq!(parse_arith("010"), ArithExpr::Number(8));
}

// ===========================================================================
// Tokenizer-specific tests
// ===========================================================================

#[test]
fn test_tokenize_power_operator() {
    let tokens = tokenize("2**3");
    assert_eq!(
        tokens,
        vec![
            ArithToken::Number(2),
            ArithToken::Power,
            ArithToken::Number(3),
        ],
    );
}

#[test]
fn test_tokenize_ternary_full() {
    let tokens = tokenize("x?1:0");
    assert_eq!(
        tokens,
        vec![
            ArithToken::Variable("x".to_string()),
            ArithToken::Question,
            ArithToken::Number(1),
            ArithToken::Colon,
            ArithToken::Number(0),
        ],
    );
}

#[test]
fn test_tokenize_comma_separated_expressions() {
    let tokens = tokenize("a=1,b=2");
    assert_eq!(
        tokens,
        vec![
            ArithToken::Variable("a".to_string()),
            ArithToken::Assign,
            ArithToken::Number(1),
            ArithToken::Comma,
            ArithToken::Variable("b".to_string()),
            ArithToken::Assign,
            ArithToken::Number(2),
        ],
    );
}

#[test]
fn test_tokenize_dollar_variable() {
    let tokens = tokenize("$count + 1");
    assert_eq!(
        tokens,
        vec![
            ArithToken::Variable("count".to_string()),
            ArithToken::Plus,
            ArithToken::Number(1),
        ],
    );
}

#[test]
fn test_tokenize_bitwise_not() {
    let tokens = tokenize("~x");
    assert_eq!(
        tokens,
        vec![ArithToken::BitNot, ArithToken::Variable("x".to_string())],
    );
}

#[test]
fn test_tokenize_underscore_variable() {
    let tokens = tokenize("_my_var");
    assert_eq!(tokens, vec![ArithToken::Variable("_my_var".to_string())],);
}

// ===========================================================================
// Error handling
// ===========================================================================

#[test]
fn test_arith_missing_closing_paren() {
    let err = parse_arith_err("(1+2");
    assert!(
        matches!(err, super::parser::ParseError::InvalidSyntax(_)),
        "Missing paren should produce InvalidSyntax, got: {err:?}",
    );
}

#[test]
fn test_arith_trailing_operator() {
    let err = parse_arith_err("1+");
    assert!(
        matches!(err, super::parser::ParseError::InvalidSyntax(_)),
        "Trailing operator should produce InvalidSyntax, got: {err:?}",
    );
}

#[test]
fn test_arith_ternary_missing_colon() {
    let err = parse_arith_err("x?1");
    assert!(
        matches!(err, super::parser::ParseError::InvalidSyntax(_)),
        "Ternary missing colon should produce InvalidSyntax, got: {err:?}",
    );
}

#[test]
fn test_arith_empty_parens() {
    let err = parse_arith_err("()");
    assert!(
        matches!(err, super::parser::ParseError::InvalidSyntax(_)),
        "Empty parens should produce InvalidSyntax, got: {err:?}",
    );
}

#[test]
fn test_arith_unary_plus() {
    // Unary +5 passes through to Number(5)
    assert_eq!(parse_arith("+5"), ArithExpr::Number(5));
}

#[test]
fn test_arith_bitwise_not_tilde() {
    // ~x => Sub(-1, x)
    assert_eq!(
        parse_arith("~x"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Number(-1)),
            Box::new(ArithExpr::Variable("x".to_string())),
        ),
    );
}
