impl BashParser {

    /// F017: Parse select statement: select VAR in WORDS; do COMMANDS; done
    /// Interactive menu selection loop (bash-specific)
    /// Presents numbered menu from WORDS, user selects, VAR is set to selection, COMMANDS run
    pub(crate) fn parse_select(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::Select)?;

        let variable = if let Some(Token::Identifier(name)) = self.peek() {
            let var = name.clone();
            self.advance();
            var
        } else {
            return Err(self.syntax_error("variable name after 'select'"));
        };

        // Expect 'in'
        self.expect(Token::In)?;

        // Parse items (same pattern as for loop)
        let mut item_list = vec![];
        loop {
            let item = self.parse_expression()?;
            item_list.push(item);

            if self.check(&Token::Semicolon)
                || self.check(&Token::Do)
                || self.check(&Token::Newline)
            {
                break;
            }
        }

        let items = if item_list.len() > 1 {
            BashExpr::Array(item_list)
        } else {
            item_list
                .into_iter()
                .next()
                .expect("item_list guaranteed non-empty: else branch requires len == 1")
        };

        // Skip optional semicolon before do
        if self.check(&Token::Semicolon) {
            self.advance();
        }

        self.skip_newlines();
        self.expect(Token::Do)?;
        self.skip_newlines();

        let body = self.parse_block_until(&[Token::Done])?;
        self.expect(Token::Done)?;

        Ok(BashStmt::Select {
            variable,
            items,
            body,
            span: Span::new(self.current_line, 0, self.current_line, 0),
        })
    }

    /// Issue #68: Parse C-style for loop: for ((init; cond; incr)); do BODY; done
    /// This is a bash-specific construct that will be purified to a POSIX while loop.
    pub(crate) fn parse_for_c_style(&mut self) -> ParseResult<BashStmt> {
        // Consume '(('
        self.expect(Token::LeftParen)?;
        self.expect(Token::LeftParen)?;

        // Read the entire arithmetic expression content until '))'
        // The content is: init; condition; increment
        let mut content = String::new();
        let mut paren_depth = 0;

        while !self.is_at_end() {
            // Check for closing '))'
            if paren_depth == 0
                && self.check(&Token::RightParen)
                && self.peek_ahead(1) == Some(&Token::RightParen)
            {
                break;
            }

            // Handle nested parentheses
            if self.check(&Token::LeftParen) {
                paren_depth += 1;
                content.push('(');
                self.advance();
            } else if self.check(&Token::RightParen) {
                paren_depth -= 1;
                content.push(')');
                self.advance();
            } else {
                // Append token content
                match self.peek() {
                    Some(Token::Identifier(s)) => {
                        content.push_str(s);
                        self.advance();
                    }
                    Some(Token::Number(n)) => {
                        content.push_str(&n.to_string());
                        self.advance();
                    }
                    Some(Token::Semicolon) => {
                        content.push(';');
                        self.advance();
                    }
                    Some(Token::Assign) => {
                        content.push('=');
                        self.advance();
                    }
                    Some(Token::Lt) => {
                        content.push('<');
                        self.advance();
                    }
                    Some(Token::Gt) => {
                        content.push('>');
                        self.advance();
                    }
                    Some(Token::Le) => {
                        content.push_str("<=");
                        self.advance();
                    }
                    Some(Token::Ge) => {
                        content.push_str(">=");
                        self.advance();
                    }
                    Some(Token::Eq) => {
                        content.push_str("==");
                        self.advance();
                    }
                    Some(Token::Ne) => {
                        content.push_str("!=");
                        self.advance();
                    }
                    Some(Token::Variable(v)) => {
                        content.push('$');
                        content.push_str(v);
                        self.advance();
                    }
                    _ => {
                        // Skip unknown tokens with a space
                        content.push(' ');
                        self.advance();
                    }
                }
            }
        }

        // Consume '))'
        self.expect(Token::RightParen)?;
        self.expect(Token::RightParen)?;

        // Parse the three parts: init; condition; increment
        let parts: Vec<&str> = content.split(';').collect();
        let (init, condition, increment) = if parts.len() >= 3 {
            (
                parts[0].trim().to_string(),
                parts[1].trim().to_string(),
                parts[2].trim().to_string(),
            )
        } else {
            // Malformed, use empty strings
            (String::new(), String::new(), String::new())
        };

        // Skip optional semicolon before do
        if self.check(&Token::Semicolon) {
            self.advance();
        }

        self.skip_newlines();
        self.expect(Token::Do)?;
        self.skip_newlines();

        let body = self.parse_block_until(&[Token::Done])?;
        self.expect(Token::Done)?;

        Ok(BashStmt::ForCStyle {
            init,
            condition,
            increment,
            body,
            span: Span::new(self.current_line, 0, self.current_line, 0),
        })
    }

    /// Parse C-style for loop from pre-parsed content string
    /// Called when the lexer has already combined ((init; cond; incr)) into ArithmeticExpansion token
    pub(crate) fn parse_for_c_style_from_content(
        &mut self,
        content: &str,
    ) -> ParseResult<BashStmt> {
        // Parse the three parts: init; condition; increment
        let parts: Vec<&str> = content.split(';').collect();
        let (init, condition, increment) = if parts.len() >= 3 {
            (
                parts[0].trim().to_string(),
                parts[1].trim().to_string(),
                parts[2].trim().to_string(),
            )
        } else {
            // Malformed, use empty strings
            (String::new(), String::new(), String::new())
        };

        // Skip optional semicolon before do
        if self.check(&Token::Semicolon) {
            self.advance();
        }

        self.skip_newlines();
        self.expect(Token::Do)?;
        self.skip_newlines();

        let body = self.parse_block_until(&[Token::Done])?;
        self.expect(Token::Done)?;

        Ok(BashStmt::ForCStyle {
            init,
            condition,
            increment,
            body,
            span: Span::new(self.current_line, 0, self.current_line, 0),
        })
    }

    pub(crate) fn parse_case(&mut self) -> ParseResult<BashStmt> {
        use crate::bash_parser::ast::CaseArm;

        self.expect(Token::Case)?;

        // Parse the word to match against
        let word = self.parse_expression()?;

        self.skip_newlines();
        self.expect(Token::In)?;
        self.skip_newlines();

        let mut arms = Vec::new();

        // Parse case arms until esac
        while !self.check(&Token::Esac) {
            if self.is_at_end() {
                return Err(ParseError::InvalidSyntax(
                    "Expected 'esac' to close case statement".to_string(),
                ));
            }

            let patterns = self.parse_case_patterns()?;

            // Expect )
            if self.check(&Token::RightParen) {
                self.advance();
            }

            self.skip_newlines();

            let body = self.parse_case_arm_body()?;

            self.consume_case_terminator();

            self.skip_newlines();

            arms.push(CaseArm { patterns, body });
        }

        self.expect(Token::Esac)?;

        Ok(BashStmt::Case {
            word,
            arms,
            span: Span::new(self.current_line, 0, self.current_line, 0),
        })
    }

    /// Parse a single case pattern by concatenating consecutive tokens.
    ///
    /// Case patterns can contain dots, globs, etc. (e.g., `server.host`, `db.*`, `\#*`).
    /// These may be tokenized as multiple consecutive tokens that need concatenation.
    /// Also handles Variable tokens for patterns like `$VAR)` and Number for `1|2|3)`.
    fn parse_case_single_pattern(&mut self) -> String {
        let mut pattern = String::new();

        // Concatenate consecutive tokens that form a single pattern
        // Stop at: RightParen, Pipe, Semicolon, Newline, Esac, Eof
        while !self.is_at_end()
            && !self.check(&Token::RightParen)
            && !self.check(&Token::Pipe)
            && !self.check(&Token::Semicolon)
            && !self.check(&Token::Newline)
            && !self.check(&Token::Esac)
        {
            match self.peek() {
                Some(Token::Identifier(s)) if s == ";;" || s == ";&" || s == ";;&" => break,
                Some(Token::Identifier(s)) => {
                    pattern.push_str(s);
                    self.advance();
                }
                Some(Token::String(s)) => {
                    pattern.push_str(s);
                    self.advance();
                }
                Some(Token::Variable(v)) => {
                    pattern.push('$');
                    pattern.push_str(v);
                    self.advance();
                }
                Some(Token::Number(n)) => {
                    pattern.push_str(&n.to_string());
                    self.advance();
                }
                Some(Token::LeftBracket) => self.parse_case_bracket_class(&mut pattern),
                Some(Token::DoubleLeftBracket) => self.parse_case_posix_class(&mut pattern),
                _ => break,
            }
        }

        pattern
    }

    /// Parse bracket character class inside a case pattern: `[0-9]`, `[a-z]*`, `[!abc]`.
    fn parse_case_bracket_class(&mut self, pattern: &mut String) {
        pattern.push('[');
        self.advance();
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
    }

    /// Parse POSIX character class inside a case pattern: `[[:space:]]`, `[[:alpha:]]`.
    ///
    /// The lexer tokenizes `[[` as `DoubleLeftBracket`, but in case context
    /// it's part of a `[[:class:]]` pattern.
    fn parse_case_posix_class(&mut self, pattern: &mut String) {
        pattern.push_str("[[");
        self.advance();
        // Read chars until ]] which closes the POSIX class
        while !self.is_at_end() && !self.check(&Token::DoubleRightBracket) {
            match self.peek() {
                Some(Token::Identifier(s)) => {
                    pattern.push_str(s);
                    self.advance();
                }
                _ => break,
            }
        }
        if self.check(&Token::DoubleRightBracket) {
            pattern.push_str("]]");
            self.advance();
        }
    }

    /// Parse multiple case patterns separated by `|` (e.g., `pattern1|pattern2`).
    fn parse_case_patterns(&mut self) -> ParseResult<Vec<String>> {
        let mut patterns = Vec::new();
        loop {
            let pattern = self.parse_case_single_pattern();

            if !pattern.is_empty() {
                patterns.push(pattern);
            }

            // Check for | (alternative pattern)
            if self.check(&Token::Pipe) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(patterns)
    }

    /// Parse the body of a case arm until a case terminator (`;;`, `;&`, `;;&`) or `esac`.
    fn parse_case_arm_body(&mut self) -> ParseResult<Vec<BashStmt>> {
        let mut body = Vec::new();
        while !self.is_at_end() && !self.check(&Token::Esac) {
            // Check for case terminators (lexed as single identifier token)
            if let Some(Token::Identifier(s)) = self.peek() {
                if s == ";;" || s == ";&" || s == ";;&" {
                    break;
                }
            }
            // Check for ;; as two Semicolon tokens
            if self.check(&Token::Semicolon) {
                if self.peek_ahead(1) == Some(&Token::Semicolon) {
                    // This is ;; - arm terminator, break
                    break;
                }
                // Single ; is a statement separator within the arm
                self.advance(); // consume ;
                self.skip_newlines();
                continue;
            }
            body.push(self.parse_statement()?);
            self.skip_newlines();
        }
        Ok(body)
    }

    /// Consume a case arm terminator: `;;`, `;&`, or `;;&`.
    ///
    /// BUG-008, BUG-009 FIX: Handle all case terminators.
    /// `;;` = stop, `;&` = fall-through, `;;&` = resume pattern matching.
    fn consume_case_terminator(&mut self) {
        if let Some(Token::Identifier(s)) = self.peek() {
            if s == ";;" || s == ";&" || s == ";;&" {
                self.advance(); // consume the terminator
            }
        } else if self.check(&Token::Semicolon) {
            self.advance();
            if self.check(&Token::Semicolon) {
                self.advance();
            }
        }
    }
}
