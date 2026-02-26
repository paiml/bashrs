//! Control flow parsing: if/while/until/for/case/select/brace/subshell/coproc.
//!
//! Extracted from `parser.rs` to reduce per-file complexity.

use super::ast::*;
use super::lexer::Token;
use super::parser::{BashParser, ParseError, ParseResult};

impl BashParser {
    pub(crate) fn parse_if(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::If)?;

        let condition = self.parse_test_expression()?;

        // Skip redirections on test expressions: `if [ cond ] 2>/dev/null; then`
        self.skip_condition_redirects();

        // Skip optional semicolon before then
        if self.check(&Token::Semicolon) {
            self.advance();
        }

        self.skip_newlines();
        self.expect(Token::Then)?;
        self.skip_newlines();

        let then_block = self.parse_block_until(&[Token::Elif, Token::Else, Token::Fi])?;

        let mut elif_blocks = Vec::new();
        while self.check(&Token::Elif) {
            self.advance();
            let elif_condition = self.parse_test_expression()?;

            // Skip redirections on test expressions: `elif [ cond ] 2>/dev/null; then`
            self.skip_condition_redirects();

            // Skip optional semicolon before then
            if self.check(&Token::Semicolon) {
                self.advance();
            }

            self.skip_newlines();
            self.expect(Token::Then)?;
            self.skip_newlines();
            let elif_body = self.parse_block_until(&[Token::Elif, Token::Else, Token::Fi])?;
            elif_blocks.push((elif_condition, elif_body));
        }

        let else_block = if self.check(&Token::Else) {
            self.advance();
            self.skip_newlines();
            Some(self.parse_block_until(&[Token::Fi])?)
        } else {
            None
        };

        self.expect(Token::Fi)?;

        // Handle trailing redirects: `fi > log` or `fi 2>/dev/null`
        self.skip_compound_redirects();

        Ok(BashStmt::If {
            condition,
            then_block,
            elif_blocks,
            else_block,
            span: Span::new(self.current_line, 0, self.current_line, 0),
        })
    }

    pub(crate) fn parse_while(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::While)?;

        let condition = self.parse_test_expression()?;

        // Skip redirections on test expressions: `while [ cond ] 2>/dev/null; do`
        self.skip_condition_redirects();
        self.skip_newlines();

        // PARSER-ENH-003: Optionally consume semicolon before 'do'
        // Both `while [ cond ]; do` and `while [ cond ]\ndo` are valid bash syntax
        if self.check(&Token::Semicolon) {
            self.advance();
        }

        self.expect(Token::Do)?;
        self.skip_newlines();

        let body = self.parse_block_until(&[Token::Done])?;
        self.expect(Token::Done)?;

        // Handle trailing redirects on compound commands:
        // `done < <(cmd)` or `done < file` or `done <<< "string"`
        // Process substitution is a bash-ism; purified output drops it (not POSIX).
        self.skip_compound_redirects();

        Ok(BashStmt::While {
            condition,
            body,
            span: Span::new(self.current_line, 0, self.current_line, 0),
        })
    }

    pub(crate) fn parse_until(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::Until)?;

        let condition = self.parse_test_expression()?;

        // Skip redirections on test expressions
        self.skip_condition_redirects();
        self.skip_newlines();

        // Optionally consume semicolon before 'do'
        if self.check(&Token::Semicolon) {
            self.advance();
        }

        self.expect(Token::Do)?;
        self.skip_newlines();

        let body = self.parse_block_until(&[Token::Done])?;
        self.expect(Token::Done)?;

        Ok(BashStmt::Until {
            condition,
            body,
            span: Span::new(self.current_line, 0, self.current_line, 0),
        })
    }

    /// Parse a brace group: { cmd1; cmd2; }
    /// Issue #60: Brace groups are compound commands that can appear after || and &&
    pub(crate) fn parse_brace_group(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::LeftBrace)?;
        self.skip_newlines();

        // Parse statements until we hit the closing brace
        let body = self.parse_block_until(&[Token::RightBrace])?;

        self.expect(Token::RightBrace)?;

        // Handle trailing redirects: `{ cmd; } > out 2> err`
        self.skip_compound_redirects();

        Ok(BashStmt::BraceGroup {
            body,
            subshell: false,
            span: Span::new(self.current_line, 0, self.current_line, 0),
        })
    }

    pub(crate) fn parse_subshell(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::LeftParen)?;
        self.skip_newlines();

        let body = self.parse_block_until(&[Token::RightParen])?;

        self.expect(Token::RightParen)?;

        // Handle trailing redirects: `( cmd ) > out 2> err`
        self.skip_compound_redirects();

        Ok(BashStmt::BraceGroup {
            body,
            subshell: true,
            span: Span::new(self.current_line, 0, self.current_line, 0),
        })
    }

    /// BUG-018: Parse coproc: coproc NAME { COMMAND; } or coproc { COMMAND; }
    pub(crate) fn parse_coproc(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::Coproc)?;
        self.skip_newlines();

        // Check if there's a name before the brace
        let name = if !self.check(&Token::LeftBrace) {
            // Named coproc: coproc NAME { ... }
            if let Some(Token::Identifier(n)) = self.peek() {
                let coproc_name = n.clone();
                self.advance();
                self.skip_newlines();
                Some(coproc_name)
            } else {
                None
            }
        } else {
            None
        };

        // Parse the body
        self.expect(Token::LeftBrace)?;
        self.skip_newlines();

        let body = self.parse_block_until(&[Token::RightBrace])?;

        self.expect(Token::RightBrace)?;

        Ok(BashStmt::Coproc {
            name,
            body,
            span: Span::new(self.current_line, 0, self.current_line, 0),
        })
    }

    /// Parse standalone [ ] test command
    /// Used as a command that returns 0 (true) or 1 (false)
    /// Example: [ -d /tmp ] && echo "exists"
    pub(crate) fn parse_test_command(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::LeftBracket)?;
        let mut test_expr = self.parse_test_condition()?;
        // Handle -a (AND) and -o (OR) inside [ ]: [ cond1 -a cond2 -o cond3 ]
        while matches!(self.peek(), Some(Token::Identifier(s)) if s == "-a" || s == "-o") {
            let is_and = matches!(self.peek(), Some(Token::Identifier(s)) if s == "-a");
            self.advance();
            let right = self.parse_test_condition()?;
            if is_and {
                test_expr = TestExpr::And(Box::new(test_expr), Box::new(right));
            } else {
                test_expr = TestExpr::Or(Box::new(test_expr), Box::new(right));
            }
        }
        self.expect(Token::RightBracket)?;

        // Return as a Command with name "[" containing the test as an argument
        Ok(BashStmt::Command {
            name: "[".to_string(),
            args: vec![BashExpr::Test(Box::new(test_expr))],
            redirects: vec![],
            span: Span::new(self.current_line, 0, self.current_line, 0),
        })
    }

    /// Issue #62: Parse standalone [[ ]] extended test command
    /// Used as a command that returns 0 (true) or 1 (false)
    /// Example: [[ -d /tmp ]] && echo "exists"
    pub(crate) fn parse_extended_test_command(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::DoubleLeftBracket)?;
        let mut test_expr = self.parse_test_condition()?;
        // Handle && and || inside [[ ]]: [[ cond1 && cond2 || cond3 ]]
        while self.check(&Token::And) || self.check(&Token::Or) {
            let is_and = self.check(&Token::And);
            self.advance();
            let right = self.parse_test_condition()?;
            if is_and {
                test_expr = TestExpr::And(Box::new(test_expr), Box::new(right));
            } else {
                test_expr = TestExpr::Or(Box::new(test_expr), Box::new(right));
            }
        }
        self.expect(Token::DoubleRightBracket)?;

        // Return as a Command with name "[[" containing the test as an argument
        Ok(BashStmt::Command {
            name: "[[".to_string(),
            args: vec![BashExpr::Test(Box::new(test_expr))],
            redirects: vec![],
            span: Span::new(self.current_line, 0, self.current_line, 0),
        })
    }

    pub(crate) fn parse_for(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::For)?;

        // Issue #68: Check for C-style for loop: for ((init; cond; incr))
        // The lexer reads ((expr)) as ArithmeticExpansion token
        if let Some(Token::ArithmeticExpansion(content)) = self.peek() {
            let content = content.clone();
            self.advance();
            return self.parse_for_c_style_from_content(&content);
        }

        // Also handle case where lexer produces two LeftParens
        if self.check(&Token::LeftParen) && self.peek_ahead(1) == Some(&Token::LeftParen) {
            return self.parse_for_c_style();
        }

        let variable = if let Some(Token::Identifier(name)) = self.peek() {
            let var = name.clone();
            self.advance();
            var
        } else {
            return Err(self.syntax_error("variable name after 'for'"));
        };

        // Expect 'in'
        self.expect(Token::In)?;

        // PARSER-ENH-002: Parse multiple items (for i in 1 2 3; do...)
        // Bug fix: Parser previously only handled single item after 'in'
        // Now collects multiple expressions until semicolon or 'do' keyword
        let mut item_list = vec![];
        loop {
            // Parse one item
            let item = self.parse_expression()?;
            item_list.push(item);

            // Check if we've reached the end of the item list
            // Break on semicolon, do keyword, or newline
            if self.check(&Token::Semicolon)
                || self.check(&Token::Do)
                || self.check(&Token::Newline)
            {
                break;
            }
        }

        // If we have multiple items, wrap in Array. Otherwise, use single item.
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

        Ok(BashStmt::For {
            variable,
            items,
            body,
            span: Span::new(self.current_line, 0, self.current_line, 0),
        })
    }

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
