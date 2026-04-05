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
}

include!("parser_control_methods.rs");
