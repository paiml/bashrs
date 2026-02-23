//! Command parsing: simple commands, redirections, arguments.
//!
//! Extracted from `parser.rs` to reduce per-file complexity.

use super::ast::*;
use super::lexer::Token;
use super::parser::{BashParser, ParseError, ParseResult};

impl BashParser {
    pub(crate) fn parse_command(&mut self) -> ParseResult<BashStmt> {
        let name = match self.peek() {
            Some(Token::Identifier(n)) => {
                let cmd = n.clone();
                self.advance();
                cmd
            }
            Some(Token::String(s)) => {
                let cmd = s.clone();
                self.advance();
                cmd
            }
            // Handle $VAR as command name (e.g., $KUBECTL scale ...)
            Some(Token::Variable(v)) => {
                let cmd = format!("${}", v);
                self.advance();
                cmd
            }
            // Handle keyword tokens as command names (rare but valid bash)
            Some(t) if Self::keyword_as_str(t).is_some() => {
                // SAFETY: keyword_as_str(t).is_some() checked in guard
                #[allow(clippy::expect_used)]
                let cmd = Self::keyword_as_str(t)
                    .expect("checked is_some")
                    .to_string();
                self.advance();
                cmd
            }
            _ => return Err(self.syntax_error("command name")),
        };

        let mut args = Vec::new();
        let mut redirects = Vec::new();

        // Parse arguments and redirections until newline or special token
        while !self.at_command_boundary() {
            // Try redirect first; if not a redirect, parse as argument
            if self.try_parse_redirect(&mut redirects)? {
                continue;
            }
            self.parse_command_argument(&mut args)?;
        }

        Ok(BashStmt::Command {
            name,
            args,
            redirects,
            span: Span::new(self.current_line, 0, self.current_line, 0),
        })
    }

    /// Check if the parser is at a command boundary (end of command arguments/redirects)
    pub(crate) fn at_command_boundary(&self) -> bool {
        // Also stop at comments (BUILTIN-001: colon no-op with comments)
        // Issue #59: Also stop at && and || for logical operator support
        // BUG-008, BUG-009 FIX: Also stop at case terminators
        // BUG-011 FIX: Also stop at RightParen and RightBrace for function/subshell/brace bodies
        self.is_at_end()
            || self.check(&Token::Newline)
            || self.check(&Token::Semicolon)
            || self.check(&Token::Pipe)
            || self.check(&Token::And)
            || self.check(&Token::Or)
            // Stop at standalone & (background) but NOT &> (combined redirect)
            || (self.check(&Token::Ampersand) && !matches!(self.peek_ahead(1), Some(Token::Gt)))
            || self.check(&Token::RightParen)
            || self.check(&Token::RightBrace)
            || matches!(self.peek(), Some(Token::Comment(_)))
            || matches!(self.peek(), Some(Token::Identifier(s)) if s == ";;" || s == ";&" || s == ";;&")
    }

    /// Try to parse a redirect operator from the current position.
    /// Returns Ok(true) if a redirect was consumed, Ok(false) if not a redirect.
    pub(crate) fn try_parse_redirect(&mut self, redirects: &mut Vec<Redirect>) -> ParseResult<bool> {
        // BUG-015 FIX: Check fd-based patterns first (close, dup, error, append-error)
        if let Some(result) = self.try_parse_fd_close_redirect(redirects) {
            return result;
        }
        if let Some(result) = self.try_parse_fd_dup_redirect(redirects) {
            return result;
        }
        if let Some(result) = self.try_parse_fd_redirect(redirects) {
            return result;
        }
        // Heredoc and here-string
        if let Some(Token::Heredoc { content, delimiter }) = self.peek() {
            let content = content.clone();
            let _delimiter = delimiter.clone();
            self.advance();
            redirects.push(Redirect::HereString { content });
            return Ok(true);
        }
        if let Some(Token::HereString(content)) = self.peek() {
            let content = content.clone();
            self.advance();
            redirects.push(Redirect::HereString { content });
            return Ok(true);
        }
        // Simple token-based redirects
        if matches!(self.peek(), Some(Token::Lt)) {
            self.advance();
            let target = self.parse_redirect_target()?;
            redirects.push(Redirect::Input { target });
            return Ok(true);
        }
        if matches!(self.peek(), Some(Token::GtGt)) {
            self.advance();
            let target = self.parse_redirect_target()?;
            redirects.push(Redirect::Append { target });
            return Ok(true);
        }
        // Combined redirection: &> file
        if matches!(self.peek(), Some(Token::Ampersand))
            && matches!(self.peek_ahead(1), Some(Token::Gt))
        {
            self.advance(); // consume '&'
            self.advance(); // consume '>'
            let target = self.parse_redirect_target()?;
            redirects.push(Redirect::Combined { target });
            return Ok(true);
        }
        // F004 FIX: fd dup shorthand >&2 (shorthand for 1>&2)
        if matches!(self.peek(), Some(Token::Gt))
            && matches!(self.peek_ahead(1), Some(Token::Ampersand))
            && matches!(self.peek_ahead(2), Some(Token::Number(_)))
        {
            self.advance(); // consume '>'
            self.advance(); // consume '&'
            let to_fd = self.expect_number_as_fd();
            self.advance();
            redirects.push(Redirect::Duplicate { from_fd: 1, to_fd });
            return Ok(true);
        }
        // Output redirection: > file
        if matches!(self.peek(), Some(Token::Gt)) {
            self.advance();
            let target = self.parse_redirect_target()?;
            redirects.push(Redirect::Output { target });
            return Ok(true);
        }
        // BUG-015, BUG-016, BUG-017 FIX: Special redirect operators as identifiers
        self.try_parse_special_redirect_ident(redirects)
    }

    /// Extract the current token as an i32 file descriptor number.
    /// Caller must ensure `self.peek()` is `Token::Number`.
    fn expect_number_as_fd(&self) -> i32 {
        if let Some(Token::Number(n)) = self.peek() {
            *n as i32
        } else {
            unreachable!("caller must verify Token::Number")
        }
    }

    /// Try to parse close-fd redirect: `3>&-`
    /// Lexer tokenizes as Number(3) + Gt + Ampersand + Identifier("-")
    /// Returns `None` if the pattern doesn't match.
    fn try_parse_fd_close_redirect(
        &mut self,
        redirects: &mut Vec<Redirect>,
    ) -> Option<ParseResult<bool>> {
        if matches!(self.peek(), Some(Token::Number(_)))
            && matches!(self.peek_ahead(1), Some(Token::Gt))
            && matches!(self.peek_ahead(2), Some(Token::Ampersand))
            && matches!(self.peek_ahead(3), Some(Token::Identifier(s)) if s == "-" || s.starts_with('-'))
        {
            let from_fd = self.expect_number_as_fd();
            self.advance(); // consume fd number
            self.advance(); // consume '>'
            self.advance(); // consume '&'
            self.advance(); // consume '-'
            redirects.push(Redirect::Duplicate { from_fd, to_fd: -1 });
            return Some(Ok(true));
        }
        None
    }

    /// Try to parse fd duplication redirect: `2>&1`
    /// Lexer tokenizes as Number(2) + Gt + Ampersand + Number(1)
    /// Must check BEFORE error redirection since it's a longer pattern.
    /// Returns `None` if the pattern doesn't match.
    fn try_parse_fd_dup_redirect(
        &mut self,
        redirects: &mut Vec<Redirect>,
    ) -> Option<ParseResult<bool>> {
        if matches!(self.peek(), Some(Token::Number(_)))
            && matches!(self.peek_ahead(1), Some(Token::Gt))
            && matches!(self.peek_ahead(2), Some(Token::Ampersand))
            && matches!(self.peek_ahead(3), Some(Token::Number(_)))
        {
            let from_fd = self.expect_number_as_fd();
            self.advance(); // consume from_fd
            self.advance(); // consume '>'
            self.advance(); // consume '&'
            let to_fd = self.expect_number_as_fd();
            self.advance(); // consume to_fd
            redirects.push(Redirect::Duplicate { from_fd, to_fd });
            return Some(Ok(true));
        }
        None
    }

    /// Try to parse fd-based redirects: `2>file` or `2>>file`
    /// Returns `None` if the pattern doesn't match.
    fn try_parse_fd_redirect(
        &mut self,
        redirects: &mut Vec<Redirect>,
    ) -> Option<ParseResult<bool>> {
        if matches!(self.peek(), Some(Token::Number(_)))
            && matches!(self.peek_ahead(1), Some(Token::Gt))
        {
            // Error redirection: 2> file
            self.advance(); // consume number (file descriptor)
            self.advance(); // consume '>'
            let target = match self.parse_redirect_target() {
                Ok(t) => t,
                Err(e) => return Some(Err(e)),
            };
            redirects.push(Redirect::Error { target });
            return Some(Ok(true));
        }
        if matches!(self.peek(), Some(Token::Number(_)))
            && matches!(self.peek_ahead(1), Some(Token::GtGt))
        {
            // Append error redirection: 2>> file
            self.advance(); // consume number (file descriptor)
            self.advance(); // consume '>>'
            let target = match self.parse_redirect_target() {
                Ok(t) => t,
                Err(e) => return Some(Err(e)),
            };
            redirects.push(Redirect::AppendError { target });
            return Some(Ok(true));
        }
        None
    }

    /// Try to parse special redirect operators tokenized as identifiers (>|, <>).
    /// BUG-015, BUG-016, BUG-017 FIX.
    fn try_parse_special_redirect_ident(
        &mut self,
        redirects: &mut Vec<Redirect>,
    ) -> ParseResult<bool> {
        if let Some(Token::Identifier(s)) = self.peek() {
            match s.as_str() {
                ">|" => {
                    self.advance();
                    let target = self.parse_redirect_target()?;
                    redirects.push(Redirect::Output { target });
                    return Ok(true);
                }
                "<>" => {
                    self.advance();
                    let target = self.parse_redirect_target()?;
                    redirects.push(Redirect::Input { target });
                    return Ok(true);
                }
                _ => {}
            }
        }
        Ok(false)
    }

    /// Parse a single command argument (identifier with name=value, glob bracket, assign, or expression)
    pub(crate) fn parse_command_argument(&mut self, args: &mut Vec<BashExpr>) -> ParseResult<()> {
        if let Some(Token::Identifier(s)) = self.peek() {
            if self.peek_ahead(1) == Some(&Token::Assign) {
                let var_name = s.clone();
                return self.parse_name_value_arg(args, &var_name);
            }
        }

        if self.check(&Token::LeftBracket) {
            return self.parse_glob_bracket_arg(args);
        }

        if self.check(&Token::Assign) {
            // Standalone '=' in argument position (edge case)
            self.advance();
            self.push_assign_value_arg(args, "=")?;
            return Ok(());
        }

        // Regular argument
        args.push(self.parse_expression()?);
        Ok(())
    }

    /// Parse a `name=value` argument pattern.
    /// e.g., `docker ps --filter name=myapp`, `env VAR=value cmd`
    fn parse_name_value_arg(
        &mut self,
        args: &mut Vec<BashExpr>,
        var_name: &str,
    ) -> ParseResult<()> {
        self.advance(); // consume name
        self.advance(); // consume '='
        let prefix = format!("{}=", var_name);
        self.push_assign_value_arg(args, &prefix)
    }

    /// After consuming a `prefix` (like `"name="` or `"="`), parse the optional value
    /// and push the combined argument(s) onto `args`.
    fn push_assign_value_arg(
        &mut self,
        args: &mut Vec<BashExpr>,
        prefix: &str,
    ) -> ParseResult<()> {
        if self.is_at_end()
            || self.check(&Token::Newline)
            || self.check(&Token::Semicolon)
            || matches!(self.peek(), Some(Token::Comment(_)))
        {
            args.push(BashExpr::Literal(prefix.to_string()));
        } else {
            let val = self.parse_expression()?;
            match val {
                BashExpr::Literal(v) => {
                    args.push(BashExpr::Literal(format!("{}{}", prefix, v)));
                }
                other => {
                    args.push(BashExpr::Literal(prefix.to_string()));
                    args.push(other);
                }
            }
        }
        Ok(())
    }

    /// Parse a glob bracket argument: `[abc]`, `[a-z]`, `[!abc]`, `[^abc]`, etc.
    fn parse_glob_bracket_arg(&mut self, args: &mut Vec<BashExpr>) -> ParseResult<()> {
        let mut pattern = String::from("[");
        self.advance(); // consume '['

        // Collect characters until ']'
        while !self.is_at_end() && !self.check(&Token::RightBracket) {
            match self.peek() {
                Some(Token::Identifier(s)) => {
                    pattern.push_str(s);
                    self.advance();
                }
                Some(Token::Number(n)) => {
                    pattern.push_str(&n.to_string());
                    self.advance();
                }
                Some(Token::Not) => {
                    pattern.push('!');
                    self.advance();
                }
                _ => break,
            }
        }

        if self.check(&Token::RightBracket) {
            pattern.push(']');
            self.advance();
        }

        // If followed by more identifier parts, append them (.txt, etc.)
        while let Some(Token::Identifier(s)) = self.peek() {
            if s == ";" || s == ";;" || s == ";&" || s == ";;&" {
                break;
            }
            pattern.push_str(s);
            self.advance();
        }

        args.push(BashExpr::Literal(pattern));
        Ok(())
    }

    /// Parse redirect target (filename)
    ///
    /// Handles filenames like "output.txt" which are tokenized as multiple tokens:
    /// - "output" (Identifier)
    /// - ".txt" (Identifier from bareword)
    ///
    /// Concatenates consecutive identifier tokens until hitting a delimiter
    pub(crate) fn parse_redirect_target(&mut self) -> ParseResult<BashExpr> {
        let mut filename = String::new();

        // Consume consecutive identifier/bareword tokens
        while !self.is_at_end()
            && !self.check(&Token::Newline)
            && !self.check(&Token::Semicolon)
            && !self.check(&Token::Pipe)
            && !self.check(&Token::Gt)
            && !matches!(self.peek(), Some(Token::Comment(_)))
        {
            match self.peek() {
                Some(Token::Identifier(s)) => {
                    filename.push_str(s);
                    self.advance();
                }
                Some(Token::String(s)) => {
                    filename.push_str(s);
                    self.advance();
                    break; // Quoted strings are complete filenames
                }
                Some(Token::Variable(name)) => {
                    // Variables in redirect targets need special handling
                    // For now, return what we have
                    if filename.is_empty() {
                        return Ok(BashExpr::Variable(name.clone()));
                    }
                    break;
                }
                _ => break,
            }
        }

        if filename.is_empty() {
            return Err(ParseError::InvalidSyntax(
                "Expected filename after redirect operator".to_string(),
            ));
        }

        Ok(BashExpr::Literal(filename))
    }
}
