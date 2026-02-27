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
    assert_eq!(
        parse_arith("x"),
        ArithExpr::Variable("x".to_string()),
    );
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
fn test_arith_power() {
    // 2 ** 10 => Mul(Number(2), Number(10))  -- power lowered to Mul
    assert_eq!(
        parse_arith("2**10"),
        ArithExpr::Mul(
            Box::new(ArithExpr::Number(2)),
            Box::new(ArithExpr::Number(10)),
        ),
    );
}

#[test]
fn test_arith_power_right_associative() {
    // 2 ** 3 ** 2 => Mul(2, Mul(3, 2))  -- right-associative
    assert_eq!(
        parse_arith("2**3**2"),
        ArithExpr::Mul(
            Box::new(ArithExpr::Number(2)),
            Box::new(ArithExpr::Mul(
                Box::new(ArithExpr::Number(3)),
                Box::new(ArithExpr::Number(2)),
            )),
        ),
    );
}

// ===========================================================================
// Comma operator
// ===========================================================================

#[test]
fn test_arith_comma() {
    // x , y => returns y (comma evaluates both but returns the rightmost)
    // With variables this is simplified: comma discards left, returns right
    assert_eq!(
        parse_arith("1,2"),
        ArithExpr::Number(2),
    );
}

#[test]
fn test_arith_comma_chain() {
    // 1 , 2 , 3 => Number(3)
    assert_eq!(
        parse_arith("1,2,3"),
        ArithExpr::Number(3),
    );
}

// ===========================================================================
// Complex expressions
// ===========================================================================

#[test]
fn test_arith_nested_parens() {
    // ((1+2))*3 => Mul(Add(1, 2), 3)
    assert_eq!(
        parse_arith("((1+2))*3"),
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
fn test_arith_variable_expression() {
    // x+y*z => Add(x, Mul(y, z))  -- precedence: * before +
    assert_eq!(
        parse_arith("x+y*z"),
        ArithExpr::Add(
            Box::new(ArithExpr::Variable("x".to_string())),
            Box::new(ArithExpr::Mul(
                Box::new(ArithExpr::Variable("y".to_string())),
                Box::new(ArithExpr::Variable("z".to_string())),
            )),
        ),
    );
}

#[test]
fn test_arith_negative() {
    // Unary minus: -1 => Sub(0, 1)
    assert_eq!(
        parse_arith("-1"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Number(0)),
            Box::new(ArithExpr::Number(1)),
        ),
    );
}

#[test]
fn test_arith_unary_minus_in_expression() {
    // a + -b => Add(a, Sub(0, b))
    assert_eq!(
        parse_arith("a+-b"),
        ArithExpr::Add(
            Box::new(ArithExpr::Variable("a".to_string())),
            Box::new(ArithExpr::Sub(
                Box::new(ArithExpr::Number(0)),
                Box::new(ArithExpr::Variable("b".to_string())),
            )),
        ),
    );
}

#[test]
fn test_arith_spaces() {
    // "1 + 2 * 3" with spaces should parse identically to "1+2*3"
    assert_eq!(
        parse_arith("1 + 2 * 3"),
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
fn test_arith_complex() {
    // a+b*c-d/e => Sub(Add(a, Mul(b, c)), Div(d, e))
    assert_eq!(
        parse_arith("a+b*c-d/e"),
        ArithExpr::Sub(
            Box::new(ArithExpr::Add(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Mul(
                    Box::new(ArithExpr::Variable("b".to_string())),
                    Box::new(ArithExpr::Variable("c".to_string())),
                )),
            )),
            Box::new(ArithExpr::Div(
                Box::new(ArithExpr::Variable("d".to_string())),
                Box::new(ArithExpr::Variable("e".to_string())),
            )),
        ),
    );
}

#[test]
fn test_arith_chained_multiplication_left_assoc() {
    // a*b/c => Div(Mul(a, b), c)
    assert_eq!(
        parse_arith("a*b/c"),
        ArithExpr::Div(
            Box::new(ArithExpr::Mul(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )),
            Box::new(ArithExpr::Variable("c".to_string())),
        ),
    );
}

#[test]
fn test_arith_deeply_nested_parens() {
    // (((42))) => Number(42)
    assert_eq!(parse_arith("(((42)))"), ArithExpr::Number(42));
}

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
    assert_eq!(
        tokens,
        vec![ArithToken::Variable("_my_var".to_string())],
    );
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
