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
