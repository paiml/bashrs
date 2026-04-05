impl BashParser {

    pub(crate) fn parse_test_expression(&mut self) -> ParseResult<BashExpr> {
        // Handle [ ... ] test syntax
        if self.check(&Token::LeftBracket) {
            return self.parse_single_bracket_test();
        }

        // Handle [[ ... ]] test syntax
        if self.check(&Token::DoubleLeftBracket) {
            return self.parse_double_bracket_test();
        }

        // Issue #133: Handle negated command/pipeline condition
        let negated = self.check(&Token::Not);
        if negated {
            self.advance(); // consume !
        }

        // Handle assignment-as-condition: `if pid=$(check_pid); then`
        if self.is_assignment_condition() {
            return self.parse_assignment_condition(negated);
        }

        // Handle subshell as condition: `if ( cmd1; cmd2 ); then`
        if self.check(&Token::LeftParen) {
            return self.parse_subshell_condition(negated);
        }

        // Issue #93, #133: Handle bare command / pipeline as condition
        if self.is_command_condition_start() {
            return self.parse_bare_command_condition(negated);
        }

        // If we consumed ! but didn't find a command, handle negated test expressions
        if negated {
            return self.parse_negated_test_fallback();
        }

        // Fallback to regular expression (for backwards compatibility)
        self.parse_expression()
    }

    /// Parse `[ cond1 -a cond2 ]` single-bracket test expression.
    fn parse_single_bracket_test(&mut self) -> ParseResult<BashExpr> {
        self.advance(); // consume [
        let mut expr = self.parse_test_condition()?;
        // Handle -a (AND) and -o (OR) inside [ ]
        while matches!(self.peek(), Some(Token::Identifier(s)) if s == "-a" || s == "-o") {
            let is_and = matches!(self.peek(), Some(Token::Identifier(s)) if s == "-a");
            self.advance();
            let right = self.parse_test_condition()?;
            expr = if is_and {
                TestExpr::And(Box::new(expr), Box::new(right))
            } else {
                TestExpr::Or(Box::new(expr), Box::new(right))
            };
        }
        self.expect(Token::RightBracket)?;
        self.parse_compound_test(BashExpr::Test(Box::new(expr)))
    }

    /// Parse `[[ cond1 && cond2 ]]` double-bracket test expression.
    fn parse_double_bracket_test(&mut self) -> ParseResult<BashExpr> {
        self.advance(); // consume [[
        let mut expr = self.parse_test_condition()?;
        // Handle && and || inside [[ ]]
        while self.check(&Token::And) || self.check(&Token::Or) {
            let is_and = self.check(&Token::And);
            self.advance();
            let right = self.parse_test_condition()?;
            expr = if is_and {
                TestExpr::And(Box::new(expr), Box::new(right))
            } else {
                TestExpr::Or(Box::new(expr), Box::new(right))
            };
        }
        self.expect(Token::DoubleRightBracket)?;
        self.parse_compound_test(BashExpr::Test(Box::new(expr)))
    }

    /// Check if current position looks like an assignment-as-condition.
    /// Detect: Identifier + Assign + (CommandSubstitution|Variable|String) + (not Identifier)
    fn is_assignment_condition(&self) -> bool {
        matches!(self.peek(), Some(Token::Identifier(_)))
            && self.peek_ahead(1) == Some(&Token::Assign)
            && matches!(
                self.peek_ahead(2),
                Some(Token::CommandSubstitution(_) | Token::Variable(_) | Token::String(_))
            )
            && !matches!(self.peek_ahead(3), Some(Token::Identifier(_)))
    }

    /// Parse assignment-as-condition: `if pid=$(check_pid); then`
    fn parse_assignment_condition(&mut self, negated: bool) -> ParseResult<BashExpr> {
        let var_name = if let Some(Token::Identifier(n)) = self.peek() {
            n.clone()
        } else {
            unreachable!()
        };
        self.advance(); // consume variable name
        self.advance(); // consume =
        let value = self.parse_expression()?;
        let assign_stmt = BashStmt::Assignment {
            name: var_name,
            index: None,
            value,
            exported: false,
            span: Span::new(self.current_line, 0, self.current_line, 0),
        };
        let final_stmt = self.maybe_negate(assign_stmt, negated);
        self.parse_compound_test(BashExpr::CommandCondition(Box::new(final_stmt)))
    }

    /// Parse subshell as condition: `if ( cmd1; cmd2 ); then`
    fn parse_subshell_condition(&mut self, negated: bool) -> ParseResult<BashExpr> {
        let subshell = self.parse_subshell()?;
        let final_stmt = self.maybe_negate(subshell, negated);
        self.parse_compound_test(BashExpr::CommandCondition(Box::new(final_stmt)))
    }

    /// Check if current token starts a bare command condition.
    fn is_command_condition_start(&self) -> bool {
        match self.peek() {
            Some(Token::Identifier(name)) => !name.starts_with('-'),
            Some(Token::Variable(_)) => true,
            _ => false,
        }
    }

    /// Parse bare command or pipeline as condition.
    fn parse_bare_command_condition(&mut self, negated: bool) -> ParseResult<BashExpr> {
        let cmd = self.parse_condition_command()?;
        // Issue #133: If next token is Pipe, build a pipeline
        let stmt = if self.check(&Token::Pipe) {
            self.parse_pipeline_from(cmd)?
        } else {
            cmd
        };
        let final_stmt = self.maybe_negate(stmt, negated);
        self.parse_compound_test(BashExpr::CommandCondition(Box::new(final_stmt)))
    }

    /// Build a pipeline from an initial command.
    fn parse_pipeline_from(&mut self, first: BashStmt) -> ParseResult<BashStmt> {
        let mut commands = vec![first];
        while self.check(&Token::Pipe) {
            self.advance(); // consume |
            commands.push(self.parse_condition_command()?);
        }
        Ok(BashStmt::Pipeline {
            commands,
            span: Span::new(self.current_line, 0, self.current_line, 0),
        })
    }

    /// Wrap a statement in `Negated` if the negated flag is set.
    fn maybe_negate(&self, stmt: BashStmt, negated: bool) -> BashStmt {
        if negated {
            BashStmt::Negated {
                command: Box::new(stmt),
                span: Span::new(self.current_line, 0, self.current_line, 0),
            }
        } else {
            stmt
        }
    }

    /// Handle `! <test_expression>` when no command was found after `!`.
    fn parse_negated_test_fallback(&mut self) -> ParseResult<BashExpr> {
        let inner = self.parse_test_expression()?;
        match inner {
            BashExpr::Test(test_expr) => Ok(BashExpr::Test(Box::new(TestExpr::Not(test_expr)))),
            other => Ok(BashExpr::Test(Box::new(TestExpr::Not(Box::new(
                TestExpr::StringNonEmpty(other),
            ))))),
        }
    }

    /// Handle compound test conditions: `[ cond1 ] && [ cond2 ]` or `[ cond1 ] || [ cond2 ]`
    pub(crate) fn parse_compound_test(&mut self, left: BashExpr) -> ParseResult<BashExpr> {
        // Helper to extract TestExpr from BashExpr::Test, or wrap in StringNonEmpty
        fn unwrap_test(expr: BashExpr) -> TestExpr {
            match expr {
                BashExpr::Test(inner) => *inner,
                other => TestExpr::StringNonEmpty(other),
            }
        }

        if self.check(&Token::And) {
            self.advance(); // consume &&
            let right = self.parse_test_expression()?;
            Ok(BashExpr::Test(Box::new(TestExpr::And(
                Box::new(unwrap_test(left)),
                Box::new(unwrap_test(right)),
            ))))
        } else if self.check(&Token::Or) {
            self.advance(); // consume ||
            let right = self.parse_test_expression()?;
            Ok(BashExpr::Test(Box::new(TestExpr::Or(
                Box::new(unwrap_test(left)),
                Box::new(unwrap_test(right)),
            ))))
        } else {
            Ok(left)
        }
    }
}

include!("parser_expr_methods_more.rs");
