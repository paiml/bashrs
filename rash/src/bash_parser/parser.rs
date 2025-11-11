//! Bash Parser
//!
//! Parses token stream from lexer into bash AST.
//! Implements recursive descent parsing for bash syntax.

use super::ast::*;
use super::lexer::{Lexer, LexerError, Token};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Lexer error: {0}")]
    LexerError(#[from] LexerError),

    #[error("Unexpected token: expected {expected}, found {found} at line {line}")]
    UnexpectedToken {
        expected: String,
        found: String,
        line: usize,
    },

    #[error("Unexpected end of file")]
    UnexpectedEof,

    #[error("Invalid syntax: {0}")]
    InvalidSyntax(String),
}

pub type ParseResult<T> = Result<T, ParseError>;

/// Internal tokens for arithmetic expression parsing
#[derive(Debug, Clone, PartialEq)]
enum ArithToken {
    Number(i64),
    Variable(String),
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    LeftParen,
    RightParen,
}

pub struct BashParser {
    tokens: Vec<Token>,
    position: usize,
    current_line: usize,
    tracer: Option<crate::tracing::TraceManager>,
}

impl BashParser {
    pub fn new(input: &str) -> ParseResult<Self> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;

        Ok(Self {
            tokens,
            position: 0,
            current_line: 1,
            tracer: None,
        })
    }

    /// Enable tracing for this parser
    ///
    /// Allows instrumentation of parsing events for debugging and analysis.
    /// Zero overhead when not called (tracer remains None).
    pub fn with_tracer(mut self, tracer: crate::tracing::TraceManager) -> Self {
        self.tracer = Some(tracer);
        self
    }

    pub fn parse(&mut self) -> ParseResult<BashAst> {
        let start_time = std::time::Instant::now();

        // Emit ParseStart trace event
        if let Some(ref tracer) = self.tracer {
            tracer.emit_parse(crate::tracing::ParseEvent::ParseStart {
                source: String::from("<input>"),
                line: 1,
                col: 1,
            });
        }

        let mut statements = Vec::new();
        let parse_result = (|| -> ParseResult<BashAst> {
            while !self.is_at_end() {
                self.skip_newlines();
                if self.is_at_end() {
                    break;
                }

                let stmt = self.parse_statement()?;

                // Emit ParseNode trace event for each statement
                if let Some(ref tracer) = self.tracer {
                    tracer.emit_parse(crate::tracing::ParseEvent::ParseNode {
                        node_type: stmt.node_type().to_string(),
                        span: stmt.span(),
                    });
                }

                statements.push(stmt);
                self.skip_newlines();
            }

            let duration = start_time.elapsed();
            let parse_time_ms = duration.as_millis() as u64;

            // Emit ParseComplete trace event
            if let Some(ref tracer) = self.tracer {
                tracer.emit_parse(crate::tracing::ParseEvent::ParseComplete {
                    node_count: statements.len(),
                    duration,
                });
            }

            Ok(BashAst {
                statements,
                metadata: AstMetadata {
                    source_file: None,
                    line_count: self.current_line,
                    parse_time_ms,
                },
            })
        })();

        // Emit ParseError trace event if parsing failed
        if let Err(ref err) = parse_result {
            if let Some(ref tracer) = self.tracer {
                tracer.emit_parse(crate::tracing::ParseEvent::ParseError {
                    error: err.to_string(),
                    span: crate::tracing::Span::single_line(self.current_line, 1, 1),
                });
            }
        }

        parse_result
    }

    fn parse_statement(&mut self) -> ParseResult<BashStmt> {
        // Skip comments and collect them
        if let Some(Token::Comment(text)) = self.peek() {
            let comment = text.clone();
            self.advance();
            return Ok(BashStmt::Comment {
                text: comment,
                span: Span::dummy(),
            });
        }

        match self.peek() {
            // Bash allows keywords as variable names (e.g., fi=1, for=2, while=3)
            // Check for assignment pattern first before treating as control structure
            Some(Token::If) if self.peek_ahead(1) == Some(&Token::Assign) => {
                self.parse_assignment(false)
            }
            Some(Token::Then) if self.peek_ahead(1) == Some(&Token::Assign) => {
                self.parse_assignment(false)
            }
            Some(Token::Elif) if self.peek_ahead(1) == Some(&Token::Assign) => {
                self.parse_assignment(false)
            }
            Some(Token::Else) if self.peek_ahead(1) == Some(&Token::Assign) => {
                self.parse_assignment(false)
            }
            Some(Token::Fi) if self.peek_ahead(1) == Some(&Token::Assign) => {
                self.parse_assignment(false)
            }
            Some(Token::While) if self.peek_ahead(1) == Some(&Token::Assign) => {
                self.parse_assignment(false)
            }
            Some(Token::For) if self.peek_ahead(1) == Some(&Token::Assign) => {
                self.parse_assignment(false)
            }
            Some(Token::Do) if self.peek_ahead(1) == Some(&Token::Assign) => {
                self.parse_assignment(false)
            }
            Some(Token::Done) if self.peek_ahead(1) == Some(&Token::Assign) => {
                self.parse_assignment(false)
            }
            Some(Token::Case) if self.peek_ahead(1) == Some(&Token::Assign) => {
                self.parse_assignment(false)
            }
            Some(Token::Esac) if self.peek_ahead(1) == Some(&Token::Assign) => {
                self.parse_assignment(false)
            }
            Some(Token::In) if self.peek_ahead(1) == Some(&Token::Assign) => {
                self.parse_assignment(false)
            }
            Some(Token::Function) if self.peek_ahead(1) == Some(&Token::Assign) => {
                self.parse_assignment(false)
            }
            Some(Token::Return) if self.peek_ahead(1) == Some(&Token::Assign) => {
                self.parse_assignment(false)
            }
            // Now handle keywords as control structures (only if not assignments)
            Some(Token::If) => self.parse_if(),
            Some(Token::While) => self.parse_while(),
            Some(Token::For) => self.parse_for(),
            Some(Token::Case) => self.parse_case(),
            Some(Token::Function) => self.parse_function(),
            Some(Token::Return) => self.parse_return(),
            Some(Token::Export) => self.parse_export(),
            Some(Token::Local) => self.parse_local(),
            Some(Token::Identifier(_)) => {
                // Could be assignment, function, or command
                if self.peek_ahead(1) == Some(&Token::Assign) {
                    self.parse_assignment(false)
                } else if self.peek_ahead(1) == Some(&Token::LeftParen)
                    && self.peek_ahead(2) == Some(&Token::RightParen)
                {
                    // This is a function definition: name() { ... }
                    self.parse_function_shorthand()
                } else {
                    self.parse_command()
                }
            }
            _ => self.parse_command(),
        }
    }

    fn parse_if(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::If)?;

        let condition = self.parse_test_expression()?;

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

        Ok(BashStmt::If {
            condition,
            then_block,
            elif_blocks,
            else_block,
            span: Span::dummy(),
        })
    }

    fn parse_while(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::While)?;

        let condition = self.parse_test_expression()?;
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

        Ok(BashStmt::While {
            condition,
            body,
            span: Span::dummy(),
        })
    }

    fn parse_for(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::For)?;

        let variable = if let Some(Token::Identifier(name)) = self.peek() {
            let var = name.clone();
            self.advance();
            var
        } else {
            return Err(ParseError::InvalidSyntax(
                "Expected identifier after 'for'".to_string(),
            ));
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
            item_list.into_iter().next().unwrap() // Safe: we have at least one item
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
            span: Span::dummy(),
        })
    }

    fn parse_case(&mut self) -> ParseResult<BashStmt> {
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

            // Parse patterns (can be multiple patterns separated by |)
            let mut patterns = Vec::new();
            while let Some(Token::Identifier(pat)) | Some(Token::String(pat)) = self.peek() {
                patterns.push(pat.clone());
                self.advance();

                // Check for | (alternative pattern)
                if !self.check(&Token::Pipe) {
                    break;
                }
                self.advance();
            }

            // Expect )
            if self.check(&Token::RightParen) {
                self.advance();
            }

            self.skip_newlines();

            // Parse body until ;;
            let mut body = Vec::new();
            while !self.check(&Token::Semicolon) && !self.check(&Token::Esac) {
                if self.is_at_end() {
                    break;
                }
                body.push(self.parse_statement()?);
                self.skip_newlines();
            }

            // Expect ;;
            if self.check(&Token::Semicolon) {
                self.advance();
                if self.check(&Token::Semicolon) {
                    self.advance();
                }
            }

            self.skip_newlines();

            arms.push(CaseArm { patterns, body });
        }

        self.expect(Token::Esac)?;

        Ok(BashStmt::Case {
            word,
            arms,
            span: Span::dummy(),
        })
    }

    fn parse_function(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::Function)?;

        let name = if let Some(Token::Identifier(n)) = self.peek() {
            let fn_name = n.clone();
            self.advance();
            fn_name
        } else {
            return Err(ParseError::InvalidSyntax(
                "Expected function name".to_string(),
            ));
        };

        // Optional () after function name
        if self.check(&Token::LeftParen) {
            self.advance();
            self.expect(Token::RightParen)?;
        }

        self.skip_newlines();
        self.expect(Token::LeftBrace)?;
        self.skip_newlines();

        let body = self.parse_block_until(&[Token::RightBrace])?;
        self.expect(Token::RightBrace)?;

        Ok(BashStmt::Function {
            name,
            body,
            span: Span::dummy(),
        })
    }

    fn parse_function_shorthand(&mut self) -> ParseResult<BashStmt> {
        // Parse name() { ... } syntax without 'function' keyword
        let name = if let Some(Token::Identifier(n)) = self.peek() {
            let fn_name = n.clone();
            self.advance();
            fn_name
        } else {
            return Err(ParseError::InvalidSyntax(
                "Expected function name".to_string(),
            ));
        };

        // Expect ()
        self.expect(Token::LeftParen)?;
        self.expect(Token::RightParen)?;

        self.skip_newlines();
        self.expect(Token::LeftBrace)?;
        self.skip_newlines();

        let body = self.parse_block_until(&[Token::RightBrace])?;
        self.expect(Token::RightBrace)?;

        Ok(BashStmt::Function {
            name,
            body,
            span: Span::dummy(),
        })
    }

    fn parse_return(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::Return)?;

        let code = if self.check(&Token::Newline) || self.is_at_end() {
            None
        } else {
            Some(self.parse_expression()?)
        };

        Ok(BashStmt::Return {
            code,
            span: Span::dummy(),
        })
    }

    fn parse_export(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::Export)?;
        self.parse_assignment(true)
    }

    fn parse_local(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::Local)?;

        // Check if there's an assignment after local
        if !self.is_at_end() && !self.check(&Token::Newline) && !self.check(&Token::Semicolon) {
            // Parse as assignment with local scoping
            self.parse_assignment(false)
        } else {
            // Just "local" by itself - treat as command
            Ok(BashStmt::Command {
                name: "local".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            })
        }
    }

    fn parse_assignment(&mut self, exported: bool) -> ParseResult<BashStmt> {
        // In bash, keywords can be used as variable names (e.g., fi=1, done=2)
        let name = match self.peek() {
            Some(Token::Identifier(n)) => {
                let var_name = n.clone();
                self.advance();
                var_name
            }
            // Allow bash keywords as variable names
            Some(Token::If) => {
                self.advance();
                "if".to_string()
            }
            Some(Token::Then) => {
                self.advance();
                "then".to_string()
            }
            Some(Token::Elif) => {
                self.advance();
                "elif".to_string()
            }
            Some(Token::Else) => {
                self.advance();
                "else".to_string()
            }
            Some(Token::Fi) => {
                self.advance();
                "fi".to_string()
            }
            Some(Token::For) => {
                self.advance();
                "for".to_string()
            }
            Some(Token::While) => {
                self.advance();
                "while".to_string()
            }
            Some(Token::Do) => {
                self.advance();
                "do".to_string()
            }
            Some(Token::Done) => {
                self.advance();
                "done".to_string()
            }
            Some(Token::Case) => {
                self.advance();
                "case".to_string()
            }
            Some(Token::Esac) => {
                self.advance();
                "esac".to_string()
            }
            Some(Token::In) => {
                self.advance();
                "in".to_string()
            }
            Some(Token::Function) => {
                self.advance();
                "function".to_string()
            }
            Some(Token::Return) => {
                self.advance();
                "return".to_string()
            }
            _ => {
                return Err(ParseError::InvalidSyntax(
                    "Expected variable name in assignment".to_string(),
                ))
            }
        };

        self.expect(Token::Assign)?;
        let value = self.parse_expression()?;

        Ok(BashStmt::Assignment {
            name,
            value,
            exported,
            span: Span::dummy(),
        })
    }

    fn parse_command(&mut self) -> ParseResult<BashStmt> {
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
            _ => {
                return Err(ParseError::InvalidSyntax(
                    "Expected command name".to_string(),
                ))
            }
        };

        let mut args = Vec::new();
        let mut redirects = Vec::new();

        // Parse arguments and redirections until newline or special token
        // Also stop at comments (BUILTIN-001: colon no-op with comments)
        while !self.is_at_end()
            && !self.check(&Token::Newline)
            && !self.check(&Token::Semicolon)
            && !self.check(&Token::Pipe)
            && !matches!(self.peek(), Some(Token::Comment(_)))
        {
            // Check for error redirection: 2> file
            // Lexer tokenizes "2>" as Number(2) followed by Gt
            if matches!(self.peek(), Some(Token::Number(_)))
                && matches!(self.peek_ahead(1), Some(Token::Gt))
            {
                // Error redirection: 2> file
                self.advance(); // consume number (file descriptor)
                self.advance(); // consume '>'
                let target = self.parse_redirect_target()?;
                redirects.push(Redirect::Error { target });
            } else if matches!(self.peek(), Some(Token::Lt)) {
                // Input redirection: < file
                self.advance(); // consume '<'
                let target = self.parse_redirect_target()?;
                redirects.push(Redirect::Input { target });
            } else if matches!(self.peek(), Some(Token::GtGt)) {
                // Append redirection: >> file
                self.advance(); // consume '>>'
                let target = self.parse_redirect_target()?;
                redirects.push(Redirect::Append { target });
            } else if matches!(self.peek(), Some(Token::Gt)) {
                // Output redirection: > file
                self.advance(); // consume '>'
                let target = self.parse_redirect_target()?;
                redirects.push(Redirect::Output { target });
            } else {
                // Regular argument
                args.push(self.parse_expression()?);
            }
        }

        Ok(BashStmt::Command {
            name,
            args,
            redirects,
            span: Span::dummy(),
        })
    }

    /// Parse redirect target (filename)
    ///
    /// Handles filenames like "output.txt" which are tokenized as multiple tokens:
    /// - "output" (Identifier)
    /// - ".txt" (Identifier from bareword)
    ///
    /// Concatenates consecutive identifier tokens until hitting a delimiter
    fn parse_redirect_target(&mut self) -> ParseResult<BashExpr> {
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

    /// Parse arithmetic expression with operator precedence
    /// Grammar:
    ///   expr     := term (('+' | '-') term)*
    ///   term     := factor (('*' | '/' | '%') factor)*
    ///   factor   := number | variable | '(' expr ')'
    fn parse_arithmetic_expr(&mut self, input: &str) -> ParseResult<ArithExpr> {
        // Create a mini-lexer for arithmetic expressions
        let tokens = self.tokenize_arithmetic(input)?;
        let mut pos = 0;

        fn parse_expr(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
            let mut left = parse_term(tokens, pos)?;

            while *pos < tokens.len() {
                match &tokens[*pos] {
                    ArithToken::Plus => {
                        *pos += 1;
                        let right = parse_term(tokens, pos)?;
                        left = ArithExpr::Add(Box::new(left), Box::new(right));
                    }
                    ArithToken::Minus => {
                        *pos += 1;
                        let right = parse_term(tokens, pos)?;
                        left = ArithExpr::Sub(Box::new(left), Box::new(right));
                    }
                    _ => break,
                }
            }

            Ok(left)
        }

        fn parse_term(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
            let mut left = parse_factor(tokens, pos)?;

            while *pos < tokens.len() {
                match &tokens[*pos] {
                    ArithToken::Multiply => {
                        *pos += 1;
                        let right = parse_factor(tokens, pos)?;
                        left = ArithExpr::Mul(Box::new(left), Box::new(right));
                    }
                    ArithToken::Divide => {
                        *pos += 1;
                        let right = parse_factor(tokens, pos)?;
                        left = ArithExpr::Div(Box::new(left), Box::new(right));
                    }
                    ArithToken::Modulo => {
                        *pos += 1;
                        let right = parse_factor(tokens, pos)?;
                        left = ArithExpr::Mod(Box::new(left), Box::new(right));
                    }
                    _ => break,
                }
            }

            Ok(left)
        }

        fn parse_factor(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
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
                    let expr = parse_expr(tokens, pos)?;
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

        parse_expr(&tokens, &mut pos)
    }

    /// Tokenize arithmetic expression string
    fn tokenize_arithmetic(&self, input: &str) -> ParseResult<Vec<ArithToken>> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(&ch) = chars.peek() {
            match ch {
                ' ' | '\t' => {
                    chars.next();
                }
                '+' => {
                    chars.next();
                    tokens.push(ArithToken::Plus);
                }
                '-' => {
                    chars.next();
                    tokens.push(ArithToken::Minus);
                }
                '*' => {
                    chars.next();
                    tokens.push(ArithToken::Multiply);
                }
                '/' => {
                    chars.next();
                    tokens.push(ArithToken::Divide);
                }
                '%' => {
                    chars.next();
                    tokens.push(ArithToken::Modulo);
                }
                '(' => {
                    chars.next();
                    tokens.push(ArithToken::LeftParen);
                }
                ')' => {
                    chars.next();
                    tokens.push(ArithToken::RightParen);
                }
                '0'..='9' => {
                    let mut num_str = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch.is_ascii_digit() {
                            num_str.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    let num = num_str.parse::<i64>().map_err(|_| {
                        ParseError::InvalidSyntax(format!("Invalid number: {}", num_str))
                    })?;
                    tokens.push(ArithToken::Number(num));
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut ident = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch.is_alphanumeric() || ch == '_' {
                            ident.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(ArithToken::Variable(ident));
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

    fn parse_expression(&mut self) -> ParseResult<BashExpr> {
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
                Ok(BashExpr::Variable(var))
            }
            Some(Token::Identifier(s)) => {
                let ident = s.clone();
                self.advance();
                Ok(BashExpr::Literal(ident))
            }
            Some(Token::ArithmeticExpansion(expr)) => {
                let expr_str = expr.clone();
                self.advance();
                let arith_expr = self.parse_arithmetic_expr(&expr_str)?;
                Ok(BashExpr::Arithmetic(Box::new(arith_expr)))
            }
            Some(Token::CommandSubstitution(cmd)) => {
                let cmd_str = cmd.clone();
                self.advance();
                // For now, parse the command string as a simple command
                // This creates a placeholder AST node that accepts $(command) syntax
                // Full command parsing can be enhanced later
                let placeholder_stmt = BashStmt::Command {
                    name: cmd_str.clone(),
                    args: vec![],
                    redirects: vec![],
                    span: Span {
                        start_line: 0,
                        start_col: 0,
                        end_line: 0,
                        end_col: 0,
                    },
                };
                Ok(BashExpr::CommandSubst(Box::new(placeholder_stmt)))
            }
            Some(Token::Heredoc {
                delimiter: _,
                content,
            }) => {
                // Parse heredoc - treat content as a literal for now
                let content_str = content.clone();
                self.advance();
                Ok(BashExpr::Literal(content_str))
            }
            _ => Err(ParseError::InvalidSyntax("Expected expression".to_string())),
        }
    }

    fn parse_test_expression(&mut self) -> ParseResult<BashExpr> {
        // Handle [ ... ] test syntax
        if self.check(&Token::LeftBracket) {
            self.advance();
            let expr = self.parse_test_condition()?;
            self.expect(Token::RightBracket)?;
            return Ok(BashExpr::Test(Box::new(expr)));
        }

        // Handle [[ ... ]] test syntax
        if self.check(&Token::DoubleLeftBracket) {
            self.advance();
            let expr = self.parse_test_condition()?;
            self.expect(Token::DoubleRightBracket)?;
            return Ok(BashExpr::Test(Box::new(expr)));
        }

        // Fallback to regular expression
        self.parse_expression()
    }

    fn parse_test_condition(&mut self) -> ParseResult<TestExpr> {
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
                "-f" | "-e" => {
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
            Some(Token::Assign) | Some(Token::Eq) => {
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

    fn parse_block_until(&mut self, terminators: &[Token]) -> ParseResult<Vec<BashStmt>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            self.skip_newlines();

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
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn peek_ahead(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.position + offset)
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.position += 1;
        }
        self.tokens.get(self.position - 1)
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Some(Token::Eof) | None)
    }

    fn check(&self, token: &Token) -> bool {
        if let Some(current) = self.peek() {
            std::mem::discriminant(current) == std::mem::discriminant(token)
        } else {
            false
        }
    }

    fn expect(&mut self, expected: Token) -> ParseResult<()> {
        if self.check(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", expected),
                found: format!("{:?}", self.peek()),
                line: self.current_line,
            })
        }
    }

    fn skip_newlines(&mut self) {
        while self.check(&Token::Newline) {
            self.advance();
            self.current_line += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_assignment() {
        let mut parser = BashParser::new("FOO=bar").unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 1);
        assert!(matches!(ast.statements[0], BashStmt::Assignment { .. }));
    }

    #[test]
    fn test_parse_if_statement() {
        let input = r#"
if [ $x == 1 ]; then
    echo "one"
fi
"#;
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();

        assert!(ast
            .statements
            .iter()
            .any(|s| matches!(s, BashStmt::If { .. })));
    }

    #[test]
    fn test_parse_function() {
        let input = r#"
function greet() {
    echo "Hello"
}
"#;
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();

        assert!(ast
            .statements
            .iter()
            .any(|s| matches!(s, BashStmt::Function { .. })));
    }

    // RED PHASE: Arithmetic expansion tests
    #[test]
    fn test_parse_arithmetic_basic() {
        let input = "y=$((x + 1))";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            BashStmt::Assignment { name, value, .. } => {
                assert_eq!(name, "y");
                match value {
                    BashExpr::Arithmetic(arith) => match arith.as_ref() {
                        ArithExpr::Add(left, right) => {
                            assert!(matches!(left.as_ref(), ArithExpr::Variable(v) if v == "x"));
                            assert!(matches!(right.as_ref(), ArithExpr::Number(1)));
                        }
                        _ => panic!("Expected Add expression"),
                    },
                    _ => panic!("Expected Arithmetic expression, got {:?}", value),
                }
            }
            _ => panic!("Expected Assignment statement"),
        }
    }

    #[test]
    fn test_parse_arithmetic_complex() {
        let input = "result=$(((a + b) * c))";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            BashStmt::Assignment { name, value, .. } => {
                assert_eq!(name, "result");
                match value {
                    BashExpr::Arithmetic(arith) => {
                        // Should be: Mul(Add(a, b), c)
                        match arith.as_ref() {
                            ArithExpr::Mul(left, right) => {
                                assert!(matches!(left.as_ref(), ArithExpr::Add(_, _)));
                                assert!(
                                    matches!(right.as_ref(), ArithExpr::Variable(v) if v == "c")
                                );
                            }
                            _ => panic!("Expected Mul expression at top level"),
                        }
                    }
                    _ => panic!("Expected Arithmetic expression"),
                }
            }
            _ => panic!("Expected Assignment statement"),
        }
    }

    #[test]
    fn test_parse_arithmetic_precedence() {
        let input = "z=$((a + b * c))";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            BashStmt::Assignment { name, value, .. } => {
                assert_eq!(name, "z");
                match value {
                    BashExpr::Arithmetic(arith) => {
                        // Should be: Add(a, Mul(b, c)) - multiplication has higher precedence
                        match arith.as_ref() {
                            ArithExpr::Add(left, right) => {
                                assert!(
                                    matches!(left.as_ref(), ArithExpr::Variable(v) if v == "a")
                                );
                                assert!(matches!(right.as_ref(), ArithExpr::Mul(_, _)));
                            }
                            _ => panic!("Expected Add expression at top level"),
                        }
                    }
                    _ => panic!("Expected Arithmetic expression"),
                }
            }
            _ => panic!("Expected Assignment statement"),
        }
    }
}
