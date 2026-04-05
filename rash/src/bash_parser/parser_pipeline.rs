//! Parser pipeline methods — extracted from parser.rs for file health.

use super::ast::*;
use super::lexer::Token;
use super::parser::ParseError;
use super::parser::{BashParser, ParseResult};
use super::parser_error_display::{expected_display, suggest_fix, token_display};

impl BashParser {
    /// Parse the right-hand side of a pipeline (compound commands are valid)
    pub(crate) fn parse_pipeline_rhs(&mut self) -> ParseResult<BashStmt> {
        match self.peek() {
            Some(Token::While) => self.parse_while(),
            Some(Token::Until) => self.parse_until(),
            Some(Token::For) => self.parse_for(),
            Some(Token::If) => self.parse_if(),
            Some(Token::Case) => self.parse_case(),
            Some(Token::LeftBrace) => self.parse_brace_group(),
            Some(Token::LeftParen) => self.parse_subshell(),
            Some(Token::Select) => self.parse_select(),
            _ => self.parse_command(),
        }
    }

    pub(crate) fn parse_block_until(
        &mut self,
        terminators: &[Token],
    ) -> ParseResult<Vec<BashStmt>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            // Skip newlines, semicolons, and background operators between statements
            // Issue #60: Brace groups use semicolons as statement separators
            // & (ampersand) is a statement terminator that backgrounds the command
            while self.check(&Token::Newline)
                || self.check(&Token::Semicolon)
                || self.check(&Token::Ampersand)
            {
                self.advance();
            }

            if terminators.iter().any(|t| self.check(t)) {
                break;
            }

            if self.is_at_end() {
                break;
            }

            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    // Helper methods
    pub(crate) fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    pub(crate) fn peek_ahead(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.position + offset)
    }

    pub(crate) fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.position += 1;
        }
        self.tokens.get(self.position - 1)
    }

    pub(crate) fn is_at_end(&self) -> bool {
        matches!(self.peek(), Some(Token::Eof) | None)
    }

    pub(crate) fn check(&self, token: &Token) -> bool {
        if let Some(current) = self.peek() {
            std::mem::discriminant(current) == std::mem::discriminant(token)
        } else {
            false
        }
    }

    pub(crate) fn expect(&mut self, expected: Token) -> ParseResult<()> {
        if self.check(&expected) {
            self.advance();
            Ok(())
        } else {
            let found_display = match self.peek() {
                Some(tok) => token_display(tok),
                None => "end of file".to_string(),
            };
            let expected_display = expected_display(&expected);
            let suggestion = suggest_fix(&expected, self.peek());
            let mut msg = format!("{expected_display}, found {found_display}");
            if let Some(hint) = suggestion {
                msg.push_str(&format!(" ({hint})"));
            }
            Err(ParseError::UnexpectedToken {
                expected: expected_display.to_string(),
                found: found_display,
                line: self.current_line,
            })
        }
    }

    /// Create a rich syntax error with current location context
    pub(crate) fn syntax_error(&self, msg: &str) -> ParseError {
        let found_display = match self.peek() {
            Some(tok) => token_display(tok),
            None => "end of file".to_string(),
        };
        ParseError::UnexpectedToken {
            expected: msg.to_string(),
            found: found_display,
            line: self.current_line,
        }
    }

    pub(crate) fn skip_newlines(&mut self) {
        while self.check(&Token::Newline) {
            self.advance();
            self.current_line += 1;
        }
    }

    /// Check if the token at the given index ends immediately before the next token
    /// (no whitespace between them). Used to distinguish `VAR=VALUE` from `VAR= VALUE`.
    pub(crate) fn tokens_adjacent(&self, token_index: usize) -> bool {
        if token_index + 1 >= self.token_positions.len() {
            return false;
        }
        let current_pos = self.token_positions[token_index];
        let next_pos = self.token_positions[token_index + 1];
        // The current token's end position = start + length of the token text
        // For Token::Assign (=), length is 1
        let current_end = match &self.tokens[token_index] {
            Token::Assign => current_pos + 1,
            Token::Identifier(s) | Token::String(s) | Token::Variable(s) => {
                // Approximate: identifier length = string length
                // (may not be exact for strings with quotes, but close enough)
                current_pos + s.len()
            }
            _ => current_pos + 1, // fallback
        };
        current_end == next_pos
    }

    /// Skip trailing redirects on compound commands and test expressions.
    /// Handles all redirect patterns:
    /// - `N>file`, `N>&M`, `N>&-` (fd-prefixed)
    /// - `>file`, `>>file`, `<file` (bare redirects)
    /// - `< <(cmd)`, `> >(cmd)` (process substitution targets)
    /// - `<<< "str"` (here-strings)
    pub(crate) fn skip_condition_redirects(&mut self) {
        loop {
            // Heredoc: <<DELIMITER ... DELIMITER
            if matches!(self.peek(), Some(Token::Heredoc { .. })) {
                self.advance();
                continue;
            }

            // Here-string: <<< "string"
            if matches!(self.peek(), Some(Token::HereString(_))) {
                self.advance();
                continue;
            }

            // fd-prefixed redirect: 2>/dev/null, 2>&1, 2>&-
            if matches!(self.peek(), Some(Token::Number(_)))
                && matches!(
                    self.peek_ahead(1),
                    Some(Token::Gt | Token::GtGt | Token::Lt)
                )
            {
                self.advance(); // consume fd number
                self.advance(); // consume redirect operator
                                // Handle >&N or >&- (fd duplication / close)
                if self.check(&Token::Ampersand) {
                    self.advance(); // consume &
                }
                // Consume redirect target (process sub <(cmd) is tokenized as Identifier)
                match self.peek() {
                    Some(
                        Token::Identifier(_)
                        | Token::String(_)
                        | Token::Variable(_)
                        | Token::Number(_),
                    ) => {
                        self.advance();
                    }
                    _ => break,
                }
                continue;
            }

            // bare redirect: >/dev/null, >>file, <file, < <(cmd), >&2, >&-
            if matches!(self.peek(), Some(Token::Gt | Token::GtGt | Token::Lt)) {
                self.advance(); // consume redirect operator
                                // Handle >&N (fd duplication) and >&- (fd close)
                if self.check(&Token::Ampersand) {
                    self.advance(); // consume &
                }
                match self.peek() {
                    Some(
                        Token::Identifier(_)
                        | Token::String(_)
                        | Token::Variable(_)
                        | Token::Number(_),
                    ) => {
                        self.advance();
                    }
                    _ => break,
                }
                continue;
            }

            break;
        }
    }

    /// Skip trailing redirects on compound commands (while/for/if/brace/subshell).
    /// Handles: `done < file`, `} > out 2> err`, `done < <(cmd)`, `fi 2>/dev/null`
    pub(crate) fn skip_compound_redirects(&mut self) {
        // Reuse skip_condition_redirects since it handles all redirect patterns
        self.skip_condition_redirects();
    }
}
