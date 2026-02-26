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

impl BashParser {
    pub(crate) fn parse_arithmetic_expr(&mut self, input: &str) -> ParseResult<ArithExpr> {
        let tokens = self.tokenize_arithmetic(input)?;
        let mut pos = 0;
        arith_prec::parse_comma(&tokens, &mut pos)
    }

    /// Tokenize arithmetic expression string
    /// BUG-002, BUG-003, BUG-004, BUG-014 FIX: Extended arithmetic tokenizer
    pub(crate) fn tokenize_arithmetic(&self, input: &str) -> ParseResult<Vec<ArithToken>> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(&ch) = chars.peek() {
            match ch {
                ' ' | '\t' | '\n' => {
                    chars.next();
                }
                // Operators and punctuation
                '+' | '-' | '*' | '/' | '%' | '(' | ')' | '<' | '>' | '=' | '!' | '?' | ':'
                | '&' | '|' | '^' | '~' | ',' => {
                    Self::tokenize_arith_operator(ch, &mut chars, &mut tokens);
                }
                // Numeric literals (decimal, hex, octal, base#value)
                '0'..='9' => {
                    Self::tokenize_arith_number(ch, &mut chars, &mut tokens)?;
                }
                // Variables (including $var references) and bare identifiers
                '$' | 'a'..='z' | 'A'..='Z' | '_' => {
                    Self::tokenize_arith_variable(ch, &mut chars, &mut tokens);
                }
                _ => {
                    return Err(ParseError::InvalidSyntax(format!(
                        "Invalid character in arithmetic: {}",
                        ch
                    )));
                }
            }
        }

        Ok(tokens)
    }

    /// Resolve a two-character operator given the first char and an optional peeked second char.
    ///
    /// Returns `(token, consume_second)` where `consume_second` indicates whether the
    /// caller should advance past the second character.
    fn resolve_two_char_op(first: char, second: Option<&char>) -> (ArithToken, bool) {
        match (first, second) {
            // Two-char operators: consume both characters
            ('*', Some(&'*')) => (ArithToken::Power, true),
            ('<', Some(&'=')) => (ArithToken::Le, true),
            ('<', Some(&'<')) => (ArithToken::ShiftLeft, true),
            ('>', Some(&'=')) => (ArithToken::Ge, true),
            ('>', Some(&'>')) => (ArithToken::ShiftRight, true),
            ('=', Some(&'=')) => (ArithToken::Eq, true),
            ('!', Some(&'=')) => (ArithToken::Ne, true),
            ('&', Some(&'&')) => (ArithToken::LogicalAnd, true),
            ('|', Some(&'|')) => (ArithToken::LogicalOr, true),
            // Single-char fallbacks for the multi-char group
            ('*', _) => (ArithToken::Multiply, false),
            ('<', _) => (ArithToken::Lt, false),
            ('>', _) => (ArithToken::Gt, false),
            ('=', _) => (ArithToken::Assign, false),
            ('!', _) => (ArithToken::LogicalNot, false),
            ('&', _) => (ArithToken::BitAnd, false),
            ('|', _) => (ArithToken::BitOr, false),
            // Should not be reached (only called for the multi-char group)
            _ => (ArithToken::Plus, false),
        }
    }

    /// Tokenize a single arithmetic operator character (possibly multi-char like **, <=, &&, etc.)
    fn tokenize_arith_operator(
        ch: char,
        chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
        tokens: &mut Vec<ArithToken>,
    ) {
        chars.next(); // consume the first character
        let token = match ch {
            // Simple single-character operators
            '+' => ArithToken::Plus,
            '-' => ArithToken::Minus,
            '/' => ArithToken::Divide,
            '%' => ArithToken::Modulo,
            '(' => ArithToken::LeftParen,
            ')' => ArithToken::RightParen,
            '?' => ArithToken::Question,
            ':' => ArithToken::Colon,
            '^' => ArithToken::BitXor,
            '~' => ArithToken::BitNot,
            ',' => ArithToken::Comma,
            // Multi-character operators: peek ahead and possibly consume second char
            '*' | '<' | '>' | '=' | '!' | '&' | '|' => {
                let (tok, consume) = Self::resolve_two_char_op(ch, chars.peek());
                if consume {
                    chars.next();
                }
                tok
            }
            _ => return, // unreachable when called from tokenize_arithmetic
        };
        tokens.push(token);
    }

    /// Collect contiguous characters matching a predicate into a string.
    fn collect_digits(
        chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
        pred: fn(char) -> bool,
    ) -> String {
        let mut buf = String::new();
        while let Some(&c) = chars.peek() {
            if pred(c) {
                buf.push(c);
                chars.next();
            } else {
                break;
            }
        }
        buf
    }

    /// Parse a hex literal after the leading "0x"/"0X" has been detected.
    /// `num_str` already contains "0" and the 'x'/'X' has been peeked but not consumed.
    fn parse_hex_literal(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> ParseResult<i64> {
        // Consume the 'x'/'X' prefix character
        chars.next();
        let hex_digits = Self::collect_digits(chars, |c| c.is_ascii_hexdigit());
        i64::from_str_radix(&hex_digits, 16)
            .map_err(|_| ParseError::InvalidSyntax(format!("Invalid hex number: 0x{}", hex_digits)))
    }

    /// Parse an octal literal or bare zero. `num_str` already contains "0" and the
    /// leading '0' has been consumed from `chars`.
    fn parse_octal_or_zero(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> i64 {
        let extra = Self::collect_digits(chars, |c| c.is_ascii_digit());
        if extra.is_empty() {
            return 0;
        }
        // Build the full string: "0" + extra digits
        let mut full = String::with_capacity(1 + extra.len());
        full.push('0');
        full.push_str(&extra);
        // Parse as octal; fall back to decimal, then 0
        i64::from_str_radix(&full, 8).unwrap_or_else(|_| full.parse::<i64>().unwrap_or(0))
    }

    /// Parse a decimal literal or base#value notation.
    /// `chars` is positioned at the first digit (non-zero).
    fn parse_decimal_or_base(
        chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    ) -> ParseResult<i64> {
        let digits = Self::collect_digits(chars, |c| c.is_ascii_digit());
        // Handle base#value notation: 16#FF, 8#77, 2#1010
        if chars.peek() == Some(&'#') {
            chars.next(); // consume '#'
            let base = digits.parse::<u32>().unwrap_or(10);
            let value_str = Self::collect_digits(chars, |c| c.is_ascii_alphanumeric() || c == '_');
            Ok(i64::from_str_radix(&value_str, base).unwrap_or(0))
        } else {
            digits
                .parse::<i64>()
                .map_err(|_| ParseError::InvalidSyntax(format!("Invalid number: {}", digits)))
        }
    }

    /// Tokenize a numeric literal (decimal, hex 0x, octal 0nnn, or base#value notation)
    fn tokenize_arith_number(
        ch: char,
        chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
        tokens: &mut Vec<ArithToken>,
    ) -> ParseResult<()> {
        let num = if ch == '0' {
            chars.next(); // consume the leading '0'
            if matches!(chars.peek(), Some(&'x' | &'X')) {
                Self::parse_hex_literal(chars)?
            } else {
                Self::parse_octal_or_zero(chars)
            }
        } else {
            Self::parse_decimal_or_base(chars)?
        };
        tokens.push(ArithToken::Number(num));
        Ok(())
    }

    /// Tokenize a variable reference ($var or bare identifier)
    fn tokenize_arith_variable(
        ch: char,
        chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
        tokens: &mut Vec<ArithToken>,
    ) {
        if ch == '$' {
            chars.next();
            let mut ident = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_alphanumeric() || c == '_' {
                    ident.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(ArithToken::Variable(ident));
        } else {
            // 'a'..='z' | 'A'..='Z' | '_'
            let mut ident = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_alphanumeric() || c == '_' {
                    ident.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(ArithToken::Variable(ident));
        }
    }
}
