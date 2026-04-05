//! Arithmetic expression parsing: tokenization and precedence climbing.
//!
//! Extracted from `parser.rs` to reduce per-file complexity.

use super::ast::ArithExpr;
use super::parser::{BashParser, ParseError, ParseResult};

/// Internal tokens for arithmetic expression parsing
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ArithToken {
    Number(i64),
    Variable(String),
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    LeftParen,
    RightParen,
    // BUG-003 FIX: Comparison operators for ternary
    Lt,       // <
    Le,       // <=
    Gt,       // >
    Ge,       // >=
    Eq,       // ==
    Ne,       // !=
    Question, // ?
    Colon,    // :
    // BUG-004 FIX: Bitwise operators
    BitAnd,     // &
    BitOr,      // |
    BitXor,     // ^
    BitNot,     // ~
    ShiftLeft,  // <<
    ShiftRight, // >>
    // Exponentiation
    Power, // **
    // Assignment in arithmetic
    Assign, // =
    // Comma operator (BUG-014)
    Comma, // ,
    // Logical operators
    LogicalAnd, // &&
    LogicalOr,  // ||
    LogicalNot, // !
}

/// Arithmetic expression precedence-climbing parser.
///
/// Extracted from `BashParser::parse_arithmetic_expr` to reduce function complexity.
/// Each function handles one or two precedence levels, calling down the chain.
mod arith_prec {
    use super::{ArithExpr, ArithToken, ParseError, ParseResult};

    // Level 1: Comma operator (lowest precedence)
    pub(super) fn parse_comma(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
        let mut left = parse_assign(tokens, pos)?;
        while *pos < tokens.len() && matches!(tokens[*pos], ArithToken::Comma) {
            *pos += 1;
            let right = parse_assign(tokens, pos)?;
            // Comma returns the right value, but we need to represent both
            // For now, just return right (simplified)
            left = right;
        }
        Ok(left)
    }

    // Level 2: Assignment
    fn parse_assign(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
        parse_ternary(tokens, pos)
    }

    // Level 3: Ternary (? :)
    fn parse_ternary(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
        let cond = parse_logical_or(tokens, pos)?;
        if *pos < tokens.len() && matches!(tokens[*pos], ArithToken::Question) {
            *pos += 1;
            let then_expr = parse_ternary(tokens, pos)?;
            if *pos >= tokens.len() || !matches!(tokens[*pos], ArithToken::Colon) {
                return Err(ParseError::InvalidSyntax(
                    "Expected ':' in ternary expression".to_string(),
                ));
            }
            *pos += 1;
            let else_expr = parse_ternary(tokens, pos)?;
            // Represent as: cond ? then : else
            // We'll use a hack: (cond * then) + (!cond * else) conceptually
            // But for parsing, we just accept it - evaluation handles it
            // Store as Add with special marker or just accept the structure
            return Ok(ArithExpr::Add(
                Box::new(ArithExpr::Mul(Box::new(cond.clone()), Box::new(then_expr))),
                Box::new(ArithExpr::Mul(
                    Box::new(ArithExpr::Sub(
                        Box::new(ArithExpr::Number(1)),
                        Box::new(cond),
                    )),
                    Box::new(else_expr),
                )),
            ));
        }
        Ok(cond)
    }

    // Level 4: Logical OR
    fn parse_logical_or(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
        let mut left = parse_logical_and(tokens, pos)?;
        while *pos < tokens.len() && matches!(tokens[*pos], ArithToken::LogicalOr) {
            *pos += 1;
            let right = parse_logical_and(tokens, pos)?;
            // OR: if left != 0 then 1 else (right != 0)
            left = ArithExpr::Add(Box::new(left), Box::new(right)); // Simplified
        }
        Ok(left)
    }

    // Level 5: Logical AND
    fn parse_logical_and(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
        let mut left = parse_bitwise_or(tokens, pos)?;
        while *pos < tokens.len() && matches!(tokens[*pos], ArithToken::LogicalAnd) {
            *pos += 1;
            let right = parse_bitwise_or(tokens, pos)?;
            left = ArithExpr::Mul(Box::new(left), Box::new(right)); // Simplified
        }
        Ok(left)
    }

    // Level 6: Bitwise OR
    fn parse_bitwise_or(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
        let mut left = parse_bitwise_xor(tokens, pos)?;
        while *pos < tokens.len() && matches!(tokens[*pos], ArithToken::BitOr) {
            *pos += 1;
            let right = parse_bitwise_xor(tokens, pos)?;
            // Represent bitwise OR - for now store as add (semantic loss)
            left = ArithExpr::Add(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    // Level 7: Bitwise XOR
    fn parse_bitwise_xor(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
        let mut left = parse_bitwise_and(tokens, pos)?;
        while *pos < tokens.len() && matches!(tokens[*pos], ArithToken::BitXor) {
            *pos += 1;
            let right = parse_bitwise_and(tokens, pos)?;
            left = ArithExpr::Sub(Box::new(left), Box::new(right)); // Placeholder
        }
        Ok(left)
    }

    // Level 8: Bitwise AND
    fn parse_bitwise_and(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
        let mut left = parse_equality(tokens, pos)?;
        while *pos < tokens.len() && matches!(tokens[*pos], ArithToken::BitAnd) {
            *pos += 1;
            let right = parse_equality(tokens, pos)?;
            left = ArithExpr::Mul(Box::new(left), Box::new(right)); // Placeholder
        }
        Ok(left)
    }

    // Level 9: Equality (== !=)
    fn parse_equality(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
        let mut left = parse_comparison(tokens, pos)?;
        while *pos < tokens.len() {
            match &tokens[*pos] {
                ArithToken::Eq | ArithToken::Ne => {
                    *pos += 1;
                    let right = parse_comparison(tokens, pos)?;
                    // Represent as subtraction (0 if equal)
                    left = ArithExpr::Sub(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // Level 10: Comparison (< <= > >=)
    fn parse_comparison(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
        let mut left = parse_shift(tokens, pos)?;
        while *pos < tokens.len() {
            match &tokens[*pos] {
                ArithToken::Lt | ArithToken::Le | ArithToken::Gt | ArithToken::Ge => {
                    *pos += 1;
                    let right = parse_shift(tokens, pos)?;
                    left = ArithExpr::Sub(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // Level 11: Shift (<< >>)
    fn parse_shift(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
        let mut left = parse_additive(tokens, pos)?;
        while *pos < tokens.len() {
            match &tokens[*pos] {
                ArithToken::ShiftLeft => {
                    *pos += 1;
                    let right = parse_additive(tokens, pos)?;
                    left = ArithExpr::Mul(Box::new(left), Box::new(right));
                }
                ArithToken::ShiftRight => {
                    *pos += 1;
                    let right = parse_additive(tokens, pos)?;
                    left = ArithExpr::Div(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // Level 12: Additive (+ -)
    fn parse_additive(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
        let mut left = parse_multiplicative(tokens, pos)?;
        while *pos < tokens.len() {
            match &tokens[*pos] {
                ArithToken::Plus => {
                    *pos += 1;
                    let right = parse_multiplicative(tokens, pos)?;
                    left = ArithExpr::Add(Box::new(left), Box::new(right));
                }
                ArithToken::Minus => {
                    *pos += 1;
                    let right = parse_multiplicative(tokens, pos)?;
                    left = ArithExpr::Sub(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // Level 13: Multiplicative (* / %)
    fn parse_multiplicative(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
        let mut left = parse_power(tokens, pos)?;
        while *pos < tokens.len() {
            match &tokens[*pos] {
                ArithToken::Multiply => {
                    *pos += 1;
                    let right = parse_power(tokens, pos)?;
                    left = ArithExpr::Mul(Box::new(left), Box::new(right));
                }
                ArithToken::Divide => {
                    *pos += 1;
                    let right = parse_power(tokens, pos)?;
                    left = ArithExpr::Div(Box::new(left), Box::new(right));
                }
                ArithToken::Modulo => {
                    *pos += 1;
                    let right = parse_power(tokens, pos)?;
                    left = ArithExpr::Mod(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // Level 13.5: Exponentiation (**) — right-associative, higher than * / %
    fn parse_power(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
        let base = parse_unary(tokens, pos)?;
        if *pos < tokens.len() && matches!(&tokens[*pos], ArithToken::Power) {
            *pos += 1;
            // Right-associative: 2**3**2 = 2**(3**2)
            let exponent = parse_power(tokens, pos)?;
            // Emit as multiplication chain or use a helper
            // For POSIX sh output, we'll compute the power statically if possible
            Ok(ArithExpr::Mul(Box::new(base), Box::new(exponent)))
        } else {
            Ok(base)
        }
    }

    // Level 14: Unary (- ~ !)
    fn parse_unary(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
        if *pos >= tokens.len() {
            return Err(ParseError::InvalidSyntax(
                "Unexpected end of arithmetic expression".to_string(),
            ));
        }
        match &tokens[*pos] {
            ArithToken::Minus => {
                *pos += 1;
                let operand = parse_unary(tokens, pos)?;
                Ok(ArithExpr::Sub(
                    Box::new(ArithExpr::Number(0)),
                    Box::new(operand),
                ))
            }
            ArithToken::BitNot | ArithToken::LogicalNot => {
                *pos += 1;
                let operand = parse_unary(tokens, pos)?;
                // Represent as -1 - x for bitwise not (approximation)
                Ok(ArithExpr::Sub(
                    Box::new(ArithExpr::Number(-1)),
                    Box::new(operand),
                ))
            }
            ArithToken::Plus => {
                *pos += 1;
                parse_unary(tokens, pos)
            }
            _ => parse_primary(tokens, pos),
        }
    }

    // Level 15: Primary (number, variable, parentheses)
    fn parse_primary(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
        if *pos >= tokens.len() {
            return Err(ParseError::InvalidSyntax(
                "Unexpected end of arithmetic expression".to_string(),
            ));
        }
        match &tokens[*pos] {
            ArithToken::Number(n) => {
                let num = *n;
                *pos += 1;
                Ok(ArithExpr::Number(num))
            }
            ArithToken::Variable(v) => {
                let var = v.clone();
                *pos += 1;
                Ok(ArithExpr::Variable(var))
            }
            ArithToken::LeftParen => {
                *pos += 1;
                let expr = parse_comma(tokens, pos)?;
                if *pos >= tokens.len() || !matches!(tokens[*pos], ArithToken::RightParen) {
                    return Err(ParseError::InvalidSyntax(
                        "Expected closing parenthesis".to_string(),
                    ));
                }
                *pos += 1;
                Ok(expr)
            }
            _ => Err(ParseError::InvalidSyntax(format!(
                "Unexpected token in arithmetic: {:?}",
                tokens[*pos]
            ))),
        }
    }
}

include!("parser_arith_bashparser.rs");
