impl BashParser {

    /// Issue #93: Parse a command used as a condition in if/while statements
    /// Similar to parse_command but stops at `then`, `do`, and doesn't include redirections
    pub(crate) fn parse_condition_command(&mut self) -> ParseResult<BashStmt> {
        let env_prefixes = self.collect_env_prefixes();
        let cmd_name = self.consume_command_name()?;

        // Build the full name with env prefixes: "IFS= read" or "LC_ALL=C sort"
        let name = if env_prefixes.is_empty() {
            cmd_name
        } else {
            let mut full = env_prefixes.join(" ");
            full.push(' ');
            full.push_str(&cmd_name);
            full
        };

        let mut args = Vec::new();
        let mut redirects = Vec::new();

        // Parse arguments until semicolon, newline, then, do, or special tokens
        while !self.at_condition_arg_boundary() {
            if let Some(redir) = self.try_parse_condition_redirect()? {
                redirects.push(redir);
            } else {
                args.push(self.parse_expression()?);
            }
        }

        Ok(BashStmt::Command {
            name,
            args,
            redirects,
            span: Span::new(self.current_line, 0, self.current_line, 0),
        })
    }

    /// Collect env prefix assignments before a command: `IFS= read`, `LC_ALL=C sort`
    fn collect_env_prefixes(&mut self) -> Vec<String> {
        let mut env_prefixes = Vec::new();
        while matches!(self.peek(), Some(Token::Identifier(_)))
            && self.peek_ahead(1) == Some(&Token::Assign)
        {
            let var_name = if let Some(Token::Identifier(n)) = self.peek() {
                n.clone()
            } else {
                break;
            };
            self.advance(); // consume identifier
            let assign_idx = self.position;
            self.advance(); // consume =

            let value = self.consume_adjacent_value(assign_idx);
            if value.is_empty() {
                env_prefixes.push(format!("{var_name}="));
            } else {
                env_prefixes.push(format!("{var_name}={value}"));
            }
        }
        env_prefixes
    }

    /// Consume an adjacent token value for env prefix assignments.
    /// Returns empty string if no adjacent value.
    fn consume_adjacent_value(&mut self, assign_idx: usize) -> String {
        if !self.tokens_adjacent(assign_idx) {
            return String::new();
        }
        match self.peek() {
            Some(Token::Identifier(id)) => {
                let v = id.clone();
                self.advance();
                v
            }
            Some(Token::String(s)) => {
                let v = s.clone();
                self.advance();
                v
            }
            Some(Token::Number(n)) => {
                let v = n.to_string();
                self.advance();
                v
            }
            _ => String::new(),
        }
    }

    /// Consume and return the command name token.
    fn consume_command_name(&mut self) -> ParseResult<String> {
        match self.peek() {
            Some(Token::Identifier(n)) => {
                let cmd = n.clone();
                self.advance();
                Ok(cmd)
            }
            Some(Token::String(s)) => {
                let cmd = s.clone();
                self.advance();
                Ok(cmd)
            }
            Some(Token::Variable(v)) => {
                let cmd = format!("${v}");
                self.advance();
                Ok(cmd)
            }
            _ => Err(self.syntax_error("command name")),
        }
    }

    /// Check if we've reached a boundary token that ends condition command arguments.
    /// Stop at standalone & (background) but NOT &> (combined redirect).
    fn at_condition_arg_boundary(&self) -> bool {
        if self.is_at_end() {
            return true;
        }
        match self.peek() {
            Some(
                Token::Newline
                | Token::Semicolon
                | Token::Then
                | Token::Do
                | Token::Pipe
                | Token::And
                | Token::Or
                | Token::RightParen
                | Token::Comment(_),
            ) => true,
            Some(Token::Ampersand) => !matches!(self.peek_ahead(1), Some(Token::Gt)),
            _ => false,
        }
    }

    /// Try to parse a redirection at the current position.
    /// Returns `Ok(Some(redirect))` if a redirect was parsed,
    /// `Ok(None)` if no redirect pattern matched (caller should parse an argument).
    fn try_parse_condition_redirect(&mut self) -> ParseResult<Option<Redirect>> {
        // fd>& fd duplication: 2>&1
        if matches!(self.peek(), Some(Token::Number(_)))
            && matches!(self.peek_ahead(1), Some(Token::Gt))
            && matches!(self.peek_ahead(2), Some(Token::Ampersand))
            && matches!(self.peek_ahead(3), Some(Token::Number(_)))
        {
            return self.parse_fd_to_fd_redirect().map(Some);
        }
        // fd> redirect: 2>file
        if matches!(self.peek(), Some(Token::Number(_)))
            && matches!(self.peek_ahead(1), Some(Token::Gt))
        {
            self.advance();
            self.advance();
            let target = self.parse_redirect_target()?;
            return Ok(Some(Redirect::Error { target }));
        }
        // &> combined redirect
        if matches!(self.peek(), Some(Token::Ampersand))
            && matches!(self.peek_ahead(1), Some(Token::Gt))
        {
            self.advance();
            self.advance();
            let target = self.parse_redirect_target()?;
            return Ok(Some(Redirect::Combined { target }));
        }
        // >&fd duplication shorthand: >&2
        if matches!(self.peek(), Some(Token::Gt))
            && matches!(self.peek_ahead(1), Some(Token::Ampersand))
            && matches!(self.peek_ahead(2), Some(Token::Number(_)))
        {
            return self.parse_gt_ampersand_fd_redirect().map(Some);
        }
        // Simple > redirect
        if matches!(self.peek(), Some(Token::Gt)) {
            self.advance();
            let target = self.parse_redirect_target()?;
            return Ok(Some(Redirect::Output { target }));
        }
        // >> append redirect
        if matches!(self.peek(), Some(Token::GtGt)) {
            self.advance();
            let target = self.parse_redirect_target()?;
            return Ok(Some(Redirect::Append { target }));
        }
        // < input redirect
        if matches!(self.peek(), Some(Token::Lt)) {
            self.advance();
            let target = self.parse_redirect_target()?;
            return Ok(Some(Redirect::Input { target }));
        }
        Ok(None)
    }

    /// Parse `N>&M` fd-to-fd duplication redirect.
    fn parse_fd_to_fd_redirect(&mut self) -> ParseResult<Redirect> {
        let from_fd = if let Some(Token::Number(n)) = self.peek() {
            *n as i32
        } else {
            unreachable!()
        };
        self.advance(); // number
        self.advance(); // >
        self.advance(); // &
        let to_fd = if let Some(Token::Number(n)) = self.peek() {
            *n as i32
        } else {
            unreachable!()
        };
        self.advance();
        Ok(Redirect::Duplicate { from_fd, to_fd })
    }

    /// Parse `>&N` fd duplication shorthand (stdout to fd N).
    fn parse_gt_ampersand_fd_redirect(&mut self) -> ParseResult<Redirect> {
        self.advance(); // consume '>'
        self.advance(); // consume '&'
        let to_fd = if let Some(Token::Number(n)) = self.peek() {
            *n as i32
        } else {
            unreachable!()
        };
        self.advance();
        Ok(Redirect::Duplicate { from_fd: 1, to_fd })
    }

    pub(crate) fn parse_test_condition(&mut self) -> ParseResult<TestExpr> {
        // Issue #62: Handle negation operator ! at the start of test condition
        if self.check(&Token::Not) {
            self.advance(); // consume '!'
            let inner = self.parse_test_condition()?;
            return Ok(TestExpr::Not(Box::new(inner)));
        }

        // Check for unary test operators first (operators are tokenized as Identifier)
        if let Some(Token::Identifier(op)) = self.peek() {
            let operator = op.clone();

            match operator.as_str() {
                "-n" => {
                    self.advance(); // consume operator
                    let expr = self.parse_expression()?;
                    return Ok(TestExpr::StringNonEmpty(expr));
                }
                "-z" => {
                    self.advance();
                    let expr = self.parse_expression()?;
                    return Ok(TestExpr::StringEmpty(expr));
                }
                "-f" | "-e" | "-s" | "-v" | "-L" | "-h" | "-p" | "-b" | "-c" | "-g" | "-k"
                | "-u" | "-t" | "-O" | "-G" | "-N" => {
                    // File test operators: -f, -e, -s, -L/-h, -p, -b, -c,
                    // -g, -k, -u, -t, -O, -G, -N, -v
                    self.advance();
                    let expr = self.parse_expression()?;
                    return Ok(TestExpr::FileExists(expr));
                }
                "-d" => {
                    self.advance();
                    let expr = self.parse_expression()?;
                    return Ok(TestExpr::FileDirectory(expr));
                }
                "-r" => {
                    self.advance();
                    let expr = self.parse_expression()?;
                    return Ok(TestExpr::FileReadable(expr));
                }
                "-w" => {
                    self.advance();
                    let expr = self.parse_expression()?;
                    return Ok(TestExpr::FileWritable(expr));
                }
                "-x" => {
                    self.advance();
                    let expr = self.parse_expression()?;
                    return Ok(TestExpr::FileExecutable(expr));
                }
                _ => {
                    // Not a unary operator, continue with binary operator parsing
                }
            }
        }

        // Parse left operand for binary operators
        let left = self.parse_expression()?;

        // Check for binary operators
        match self.peek() {
            Some(Token::Assign | Token::Eq) => {
                // Both = (Token::Assign) and == (Token::Eq) are string equality in tests
                self.advance();
                let right = self.parse_expression()?;
                Ok(TestExpr::StringEq(left, right))
            }
            Some(Token::Ne) => {
                self.advance();
                let right = self.parse_expression()?;
                Ok(TestExpr::StringNe(left, right))
            }
            Some(Token::Lt) => {
                self.advance();
                let right = self.parse_expression()?;
                Ok(TestExpr::IntLt(left, right))
            }
            Some(Token::Gt) => {
                self.advance();
                let right = self.parse_expression()?;
                Ok(TestExpr::IntGt(left, right))
            }
            // =~ regex match: [[ str =~ pattern ]] — pattern is embedded in token
            Some(Token::Identifier(op)) if op.starts_with("=~ ") => {
                let pattern = op.strip_prefix("=~ ").unwrap_or("").to_string();
                self.advance();
                // Treat as string equality test with the regex pattern as literal
                // (bash regex semantics can't be fully represented in POSIX)
                Ok(TestExpr::StringEq(left, BashExpr::Literal(pattern)))
            }
            Some(Token::Identifier(op))
                if matches!(op.as_str(), "-eq" | "-ne" | "-lt" | "-le" | "-gt" | "-ge") =>
            {
                let operator = op.clone();
                self.advance();
                let right = self.parse_expression()?;

                match operator.as_str() {
                    "-eq" => Ok(TestExpr::IntEq(left, right)),
                    "-ne" => Ok(TestExpr::IntNe(left, right)),
                    "-lt" => Ok(TestExpr::IntLt(left, right)),
                    "-le" => Ok(TestExpr::IntLe(left, right)),
                    "-gt" => Ok(TestExpr::IntGt(left, right)),
                    "-ge" => Ok(TestExpr::IntGe(left, right)),
                    _ => unreachable!(),
                }
            }
            _ => Ok(TestExpr::StringNonEmpty(left)),
        }
    }
}
