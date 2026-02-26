//! Expression, variable expansion, and test/condition parsing.
//!
//! Extracted from `parser.rs` to reduce per-file complexity.

use super::ast::*;
use super::lexer::Token;
use super::parser::{BashParser, ParseResult};

impl BashParser {
    /// Parse variable expansion patterns like ${VAR:-default}, ${VAR:=default}, etc.
    pub(crate) fn parse_variable_expansion(&self, var_content: &str) -> ParseResult<BashExpr> {
        // Check for parameter expansion patterns
        // ${#VAR} - string length (but NOT $# which is argument count)
        if var_content.starts_with('#') && var_content.len() > 1 && !var_content.contains(':') {
            let variable = var_content[1..].to_string();
            return Ok(BashExpr::StringLength { variable });
        }

        // ${VAR:-default} - use default if unset or null
        if let Some(pos) = var_content.find(":-") {
            let variable = var_content[..pos].to_string();
            let default = var_content[pos + 2..].to_string();
            return Ok(BashExpr::DefaultValue {
                variable,
                default: Box::new(BashExpr::Literal(default)),
            });
        }

        // ${VAR:=default} - assign default if unset or null
        if let Some(pos) = var_content.find(":=") {
            let variable = var_content[..pos].to_string();
            let default = var_content[pos + 2..].to_string();
            return Ok(BashExpr::AssignDefault {
                variable,
                default: Box::new(BashExpr::Literal(default)),
            });
        }

        // ${VAR:+alternative} - use alternative if set and not null
        if let Some(pos) = var_content.find(":+") {
            let variable = var_content[..pos].to_string();
            let alternative = var_content[pos + 2..].to_string();
            return Ok(BashExpr::AlternativeValue {
                variable,
                alternative: Box::new(BashExpr::Literal(alternative)),
            });
        }

        // ${VAR:?error} - error if unset or null
        if let Some(pos) = var_content.find(":?") {
            let variable = var_content[..pos].to_string();
            let message = var_content[pos + 2..].to_string();
            return Ok(BashExpr::ErrorIfUnset {
                variable,
                message: Box::new(BashExpr::Literal(message)),
            });
        }

        // ${VAR##pattern} - remove longest prefix pattern (must check before #)
        if let Some(pos) = var_content.find("##") {
            let variable = var_content[..pos].to_string();
            let pattern = var_content[pos + 2..].to_string();
            return Ok(BashExpr::RemoveLongestPrefix {
                variable,
                pattern: Box::new(BashExpr::Literal(pattern)),
            });
        }

        // ${VAR#pattern} - remove shortest prefix pattern
        if let Some(pos) = var_content.find('#') {
            // Make sure it's not the start (which would be string length)
            if pos > 0 {
                let variable = var_content[..pos].to_string();
                let pattern = var_content[pos + 1..].to_string();
                return Ok(BashExpr::RemovePrefix {
                    variable,
                    pattern: Box::new(BashExpr::Literal(pattern)),
                });
            }
        }

        // ${VAR%%pattern} - remove longest suffix pattern (must check before %)
        if let Some(pos) = var_content.find("%%") {
            let variable = var_content[..pos].to_string();
            let pattern = var_content[pos + 2..].to_string();
            return Ok(BashExpr::RemoveLongestSuffix {
                variable,
                pattern: Box::new(BashExpr::Literal(pattern)),
            });
        }

        // ${VAR%pattern} - remove shortest suffix pattern
        if let Some(pos) = var_content.find('%') {
            let variable = var_content[..pos].to_string();
            let pattern = var_content[pos + 1..].to_string();
            return Ok(BashExpr::RemoveSuffix {
                variable,
                pattern: Box::new(BashExpr::Literal(pattern)),
            });
        }

        // Simple variable: $VAR or ${VAR}
        Ok(BashExpr::Variable(var_content.to_string()))
    }

    pub(crate) fn parse_expression(&mut self) -> ParseResult<BashExpr> {
        match self.peek() {
            Some(Token::String(s)) => {
                let str = s.clone();
                self.advance();
                Ok(BashExpr::Literal(str))
            }
            Some(Token::Number(n)) => {
                let num = *n;
                self.advance();
                Ok(BashExpr::Literal(num.to_string()))
            }
            Some(Token::Variable(v)) => {
                let var = v.clone();
                self.advance();
                self.parse_variable_expansion(&var)
            }
            Some(Token::Identifier(s)) => {
                let ident = s.clone();
                self.advance();
                // Unquoted identifiers with glob characters should be Glob, not Literal
                if ident.contains('*') || ident.contains('?') {
                    Ok(BashExpr::Glob(ident))
                } else {
                    Ok(BashExpr::Literal(ident))
                }
            }
            // BUG-012, BUG-013 FIX: Array literals (value1 value2) or ([0]=a [5]=b)
            Some(Token::LeftParen) => self.parse_array_literal(),
            Some(Token::ArithmeticExpansion(expr)) => {
                let expr_str = expr.clone();
                self.advance();
                let arith_expr = self.parse_arithmetic_expr(&expr_str)?;
                Ok(BashExpr::Arithmetic(Box::new(arith_expr)))
            }
            Some(Token::CommandSubstitution(cmd)) => {
                let cmd_str = cmd.clone();
                self.advance();
                Ok(BashExpr::CommandSubst(Box::new(BashStmt::Command {
                    name: cmd_str.clone(),
                    args: vec![],
                    redirects: vec![],
                    span: Span {
                        start_line: 0,
                        start_col: 0,
                        end_line: 0,
                        end_col: 0,
                    },
                })))
            }
            Some(Token::Heredoc {
                delimiter: _,
                content,
            }) => {
                let content_str = content.clone();
                self.advance();
                Ok(BashExpr::Literal(content_str))
            }
            // Glob bracket pattern: [0-9], [a-z], [!abc], etc.
            Some(Token::LeftBracket) => self.parse_glob_bracket_pattern(),
            // {} as literal in argument context (e.g., find -exec cmd {} \;)
            Some(Token::LeftBrace) if self.peek_ahead(1) == Some(&Token::RightBrace) => {
                self.advance(); // consume {
                self.advance(); // consume }
                Ok(BashExpr::Literal("{}".to_string()))
            }
            // Keyword tokens used as literal strings in argument context
            // e.g., `echo done`, `echo fi`, `echo then`
            Some(t) if Self::keyword_as_str(t).is_some() => {
                // SAFETY: keyword_as_str(t).is_some() checked in guard
                #[allow(clippy::expect_used)]
                let kw = Self::keyword_as_str(t).expect("checked is_some");
                self.advance();
                Ok(BashExpr::Literal(kw.to_string()))
            }
            _ => Err(self.syntax_error("expression")),
        }
    }

    /// Parse an array literal: (value1 value2) or ([0]=a [5]=b)
    fn parse_array_literal(&mut self) -> ParseResult<BashExpr> {
        self.advance(); // consume '('
        let mut elements = Vec::new();
        while !self.is_at_end() && !self.check(&Token::RightParen) {
            if self.check(&Token::LeftBracket) {
                elements.push(self.parse_sparse_array_element()?);
            } else if self.check(&Token::Newline) {
                self.advance();
            } else {
                elements.push(self.parse_expression()?);
            }
        }
        self.expect(Token::RightParen)?;
        Ok(BashExpr::Array(elements))
    }

    /// Parse a sparse array element: [index]=value
    fn parse_sparse_array_element(&mut self) -> ParseResult<BashExpr> {
        self.advance(); // skip '['
        let index = self.collect_bracket_index();
        if self.check(&Token::RightBracket) {
            self.advance(); // skip ']'
        }
        if self.check(&Token::Assign) {
            self.advance(); // skip '='
        }
        // Parse the value
        if self.is_at_end() || self.check(&Token::RightParen) {
            return Ok(BashExpr::Literal(format!("[{index}]=")));
        }
        let value = self.parse_expression()?;
        let value_str = match &value {
            BashExpr::Literal(s) => s.clone(),
            BashExpr::Variable(v) => format!("${v}"),
            _ => "?".to_string(),
        };
        Ok(BashExpr::Literal(format!("[{index}]={value_str}")))
    }

    /// Collect tokens inside brackets to form an index string.
    fn collect_bracket_index(&mut self) -> String {
        let mut index = String::new();
        while !self.is_at_end() && !self.check(&Token::RightBracket) {
            match self.peek() {
                Some(Token::Identifier(s) | Token::String(s)) => {
                    index.push_str(s);
                    self.advance();
                }
                Some(Token::Number(n)) => {
                    index.push_str(&n.to_string());
                    self.advance();
                }
                _ => break,
            }
        }
        index
    }

    /// Parse a glob bracket pattern: [0-9], [a-z], [!abc], etc.
    fn parse_glob_bracket_pattern(&mut self) -> ParseResult<BashExpr> {
        let mut pattern = String::from("[");
        self.advance(); // consume '['
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
        // Absorb trailing glob/identifier parts: [0-9]*.sql → "[0-9]*.sql"
        while let Some(Token::Identifier(s)) = self.peek() {
            pattern.push_str(s);
            self.advance();
        }
        Ok(BashExpr::Glob(pattern))
    }

    /// Convert a keyword token to its string representation.
    /// Returns None for non-keyword tokens.
    pub(crate) fn keyword_as_str(token: &Token) -> Option<&'static str> {
        match token {
            Token::If => Some("if"),
            Token::Then => Some("then"),
            Token::Elif => Some("elif"),
            Token::Else => Some("else"),
            Token::Fi => Some("fi"),
            Token::For => Some("for"),
            Token::While => Some("while"),
            Token::Until => Some("until"),
            Token::Do => Some("do"),
            Token::Done => Some("done"),
            Token::Case => Some("case"),
            Token::Esac => Some("esac"),
            Token::In => Some("in"),
            Token::Function => Some("function"),
            Token::Return => Some("return"),
            Token::Export => Some("export"),
            Token::Local => Some("local"),
            Token::Coproc => Some("coproc"),
            Token::Select => Some("select"),
            _ => None,
        }
    }

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
