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
    // BUG-003 FIX: Comparison operators for ternary
    Lt,       // <
    Le,       // <=
    Gt,       // >
    Ge,       // >=
    Eq,       // ==
    Ne,       // !=
    Question, // ?
    Colon,    // :
    // BUG-004 FIX: Bitwise operators
    BitAnd,     // &
    BitOr,      // |
    BitXor,     // ^
    BitNot,     // ~
    ShiftLeft,  // <<
    ShiftRight, // >>
    // Assignment in arithmetic
    Assign, // =
    // Comma operator (BUG-014)
    Comma, // ,
    // Logical operators
    LogicalAnd, // &&
    LogicalOr,  // ||
    LogicalNot, // !
}

pub struct BashParser {
    tokens: Vec<Token>,
    position: usize,
    current_line: usize,
    tracer: Option<crate::tracing::TraceManager>,
}

impl BashParser {
    /// Create a new Bash parser from source code.
    ///
    /// Tokenizes the input and prepares the parser for AST generation.
    ///
    /// # Arguments
    ///
    /// * `input` - Bash script source code as a string
    ///
    /// # Returns
    ///
    /// * `Ok(BashParser)` - Ready to parse the script
    /// * `Err(ParseError)` - Lexer error (invalid tokens)
    ///
    /// # Examples
    ///
    /// ## Basic usage
    ///
    /// ```
    /// use bashrs::bash_parser::BashParser;
    ///
    /// let script = "echo hello";
    /// let parser = BashParser::new(script).unwrap();
    /// // Parser is ready to call parse()
    /// ```
    ///
    /// ## Variable assignment
    ///
    /// ```
    /// use bashrs::bash_parser::BashParser;
    ///
    /// let script = "x=42\ny=hello";
    /// let mut parser = BashParser::new(script).unwrap();
    /// let ast = parser.parse().unwrap();
    /// assert_eq!(ast.statements.len(), 2);
    /// ```
    ///
    /// ## Invalid syntax
    ///
    /// ```
    /// use bashrs::bash_parser::BashParser;
    ///
    /// // Unclosed string
    /// let script = r#"echo "unclosed"#;
    /// let result = BashParser::new(script);
    /// // Lexer detects the error
    /// assert!(result.is_err());
    /// ```
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

    /// Parse the tokenized script into a Bash AST.
    ///
    /// Performs recursive descent parsing to build an abstract syntax tree
    /// representing the structure of the bash script.
    ///
    /// # Returns
    ///
    /// * `Ok(BashAst)` - Successfully parsed AST
    /// * `Err(ParseError)` - Syntax error or unexpected token
    ///
    /// # Examples
    ///
    /// ## Simple command
    ///
    /// ```
    /// use bashrs::bash_parser::BashParser;
    ///
    /// let script = "echo hello";
    /// let mut parser = BashParser::new(script).unwrap();
    /// let ast = parser.parse().unwrap();
    /// assert_eq!(ast.statements.len(), 1);
    /// ```
    ///
    /// ## Multiple statements
    ///
    /// ```
    /// use bashrs::bash_parser::BashParser;
    ///
    /// let script = r#"
    /// x=10
    /// echo $x
    /// y=20
    /// "#;
    /// let mut parser = BashParser::new(script).unwrap();
    /// let ast = parser.parse().unwrap();
    /// assert_eq!(ast.statements.len(), 3);
    /// ```
    ///
    /// ## Control flow
    ///
    /// ```
    /// use bashrs::bash_parser::BashParser;
    ///
    /// let script = r#"
    /// if [ "$x" = "5" ]; then
    ///     echo "five"
    /// fi
    /// "#;
    /// let mut parser = BashParser::new(script).unwrap();
    /// let ast = parser.parse().unwrap();
    /// // Should parse the if statement
    /// assert!(!ast.statements.is_empty());
    /// ```
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

        // Issue #67: Handle standalone arithmetic ((expr)) as a command
        if let Some(Token::ArithmeticExpansion(expr)) = self.peek() {
            let arith_expr = expr.clone();
            self.advance();
            // Emit as a literal since we can't fully parse all bash arithmetic
            // The user can review and adjust if needed
            return Ok(BashStmt::Command {
                name: ":".to_string(), // POSIX no-op
                args: vec![BashExpr::Literal(format!("$(({}))", arith_expr))],
                redirects: vec![],
                span: Span::dummy(),
            });
        }

        // Parse first statement (could be part of pipeline)
        let first_stmt = match self.peek() {
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
            Some(Token::Coproc) => self.parse_coproc(), // BUG-018
            Some(Token::Identifier(_)) => {
                // Could be assignment, function, or command
                // BUG-012 FIX: Also handle += for array append
                if self.peek_ahead(1) == Some(&Token::Assign)
                    || matches!(self.peek_ahead(1), Some(Token::Identifier(s)) if s == "+=")
                {
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
            // Issue #60: Brace group { cmd1; cmd2; } - compound command
            Some(Token::LeftBrace) => self.parse_brace_group(),
            // Issue #62: Standalone [[ ]] extended test as command
            Some(Token::DoubleLeftBracket) => self.parse_extended_test_command(),
            _ => self.parse_command(),
        }?;

        // Check for pipeline: cmd1 | cmd2 | cmd3
        let stmt = if self.check(&Token::Pipe) {
            let mut commands = vec![first_stmt];

            // Collect all piped commands
            while self.check(&Token::Pipe) {
                self.advance(); // consume '|'

                // Skip newlines after pipe
                self.skip_newlines();

                // Parse next command in pipeline
                let next_cmd = self.parse_command()?;
                commands.push(next_cmd);
            }

            // Return pipeline with all collected commands
            BashStmt::Pipeline {
                commands,
                span: Span::dummy(),
            }
        } else {
            first_stmt
        };

        // Issue #59: Check for logical AND (&&) or OR (||) operators
        // These have lower precedence than pipes, so we check after pipeline handling
        // cmd1 | cmd2 && cmd3 parses as (cmd1 | cmd2) && cmd3
        if self.check(&Token::And) {
            self.advance(); // consume '&&'
            self.skip_newlines(); // allow newlines after &&

            // Parse right side (which could itself be a pipeline or logical list)
            let right = self.parse_statement()?;

            return Ok(BashStmt::AndList {
                left: Box::new(stmt),
                right: Box::new(right),
                span: Span::dummy(),
            });
        }

        if self.check(&Token::Or) {
            self.advance(); // consume '||'
            self.skip_newlines(); // allow newlines after ||

            // Parse right side (which could itself be a pipeline or logical list)
            let right = self.parse_statement()?;

            return Ok(BashStmt::OrList {
                left: Box::new(stmt),
                right: Box::new(right),
                span: Span::dummy(),
            });
        }

        // Not a pipeline or logical list, return the statement
        Ok(stmt)
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

    /// Parse a brace group: { cmd1; cmd2; }
    /// Issue #60: Brace groups are compound commands that can appear after || and &&
    fn parse_brace_group(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::LeftBrace)?;
        self.skip_newlines();

        // Parse statements until we hit the closing brace
        let body = self.parse_block_until(&[Token::RightBrace])?;

        self.expect(Token::RightBrace)?;

        Ok(BashStmt::BraceGroup {
            body,
            span: Span::dummy(),
        })
    }

    /// BUG-018: Parse coproc: coproc NAME { COMMAND; } or coproc { COMMAND; }
    fn parse_coproc(&mut self) -> ParseResult<BashStmt> {
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
            span: Span::dummy(),
        })
    }

    /// Issue #62: Parse standalone [[ ]] extended test command
    /// Used as a command that returns 0 (true) or 1 (false)
    /// Example: [[ -d /tmp ]] && echo "exists"
    fn parse_extended_test_command(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::DoubleLeftBracket)?;
        let test_expr = self.parse_test_condition()?;
        self.expect(Token::DoubleRightBracket)?;

        // Return as a Command with name "[[" containing the test as an argument
        Ok(BashStmt::Command {
            name: "[[".to_string(),
            args: vec![BashExpr::Test(Box::new(test_expr))],
            redirects: vec![],
            span: Span::dummy(),
        })
    }

    fn parse_for(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::For)?;

        // Issue #68: Check for C-style for loop: for ((init; cond; incr))
        if self.check(&Token::LeftParen) && self.peek_ahead(1) == Some(&Token::LeftParen) {
            return self.parse_for_c_style();
        }

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

    /// Issue #68: Parse C-style for loop: for ((init; cond; incr)); do BODY; done
    /// This is a bash-specific construct that will be purified to a POSIX while loop.
    fn parse_for_c_style(&mut self) -> ParseResult<BashStmt> {
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
                // BUG-008, BUG-009 FIX: Skip case terminators when parsing patterns
                if pat == ";;" || pat == ";&" || pat == ";;&" {
                    break;
                }
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

            // Parse body until case terminator (;;, ;&, ;;&) or esac
            let mut body = Vec::new();
            while !self.is_at_end() && !self.check(&Token::Esac) {
                // Check for case terminators
                if let Some(Token::Identifier(s)) = self.peek() {
                    if s == ";;" || s == ";&" || s == ";;&" {
                        break;
                    }
                }
                if self.check(&Token::Semicolon) {
                    // Check if this is start of ;; or ;& or ;;&
                    break;
                }
                body.push(self.parse_statement()?);
                self.skip_newlines();
            }

            // BUG-008, BUG-009 FIX: Handle all case terminators
            // ;; = stop, ;& = fall-through, ;;& = resume pattern matching
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
        // Parse name() { ... } or name() ( ... ) syntax without 'function' keyword
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

        // BUG-011 FIX: Allow subshell body: myfunc() ( ... )
        // Check if body starts with { (brace group) or ( (subshell)
        if self.check(&Token::LeftParen) {
            self.advance(); // consume '('
            self.skip_newlines();

            // Parse body until closing ')'
            let body = self.parse_block_until(&[Token::RightParen])?;
            self.expect(Token::RightParen)?;

            Ok(BashStmt::Function {
                name,
                body,
                span: Span::dummy(),
            })
        } else {
            // Standard brace body: myfunc() { ... }
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

        // Check if there's content after local
        if !self.is_at_end() && !self.check(&Token::Newline) && !self.check(&Token::Semicolon) {
            // Check if it's an assignment (identifier followed by =) or just declaration
            // `local x=1` vs `local x y z` vs `local x`
            if self.peek_ahead(1) == Some(&Token::Assign) {
                // It's an assignment: local x=1
                self.parse_assignment(false)
            } else {
                // It's a declaration without value: local x y z
                // Collect all variable names as Literal expressions
                let mut args = Vec::new();
                while !self.is_at_end()
                    && !self.check(&Token::Newline)
                    && !self.check(&Token::Semicolon)
                {
                    match self.peek() {
                        Some(Token::Identifier(name)) => {
                            args.push(BashExpr::Literal(name.clone()));
                            self.advance();
                        }
                        _ => break,
                    }
                }
                Ok(BashStmt::Command {
                    name: "local".to_string(),
                    args,
                    redirects: vec![],
                    span: Span::dummy(),
                })
            }
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

        // BUG-012 FIX: Handle both = and += assignment operators
        let is_append = matches!(self.peek(), Some(Token::Identifier(s)) if s == "+=");
        if is_append {
            self.advance(); // consume '+='
        } else {
            self.expect(Token::Assign)?;
        }

        // BUG-005 FIX: Allow empty variable assignment (x=)
        // Check if we're at end of statement (newline, semicolon, EOF, pipe, etc.)
        let value = if self.is_at_end()
            || self.check(&Token::Newline)
            || self.check(&Token::Semicolon)
            || self.check(&Token::Pipe)
            || self.check(&Token::And)
            || self.check(&Token::Or)
            || matches!(self.peek(), Some(Token::Comment(_)))
        {
            // Empty assignment: x=
            BashExpr::Literal(String::new())
        } else {
            self.parse_expression()?
        };

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
        // Issue #59: Also stop at && and || for logical operator support
        // BUG-008, BUG-009 FIX: Also stop at case terminators
        // BUG-011 FIX: Also stop at RightParen and RightBrace for function/subshell/brace bodies
        while !self.is_at_end()
            && !self.check(&Token::Newline)
            && !self.check(&Token::Semicolon)
            && !self.check(&Token::Pipe)
            && !self.check(&Token::And)
            && !self.check(&Token::Or)
            && !self.check(&Token::RightParen)
            && !self.check(&Token::RightBrace)
            && !matches!(self.peek(), Some(Token::Comment(_)))
            && !matches!(self.peek(), Some(Token::Identifier(s)) if s == ";;" || s == ";&" || s == ";;&")
        {
            // BUG-015 FIX: Check for close fd syntax FIRST: 3>&-
            // Lexer tokenizes "3>&-" as Number(3) + Gt + Ampersand + Identifier("-")
            if matches!(self.peek(), Some(Token::Number(_)))
                && matches!(self.peek_ahead(1), Some(Token::Gt))
                && matches!(self.peek_ahead(2), Some(Token::Ampersand))
                && matches!(self.peek_ahead(3), Some(Token::Identifier(s)) if s == "-" || s.starts_with('-'))
            {
                // Close file descriptor: 3>&-
                let from_fd = if let Some(Token::Number(n)) = self.peek() {
                    *n as i32
                } else {
                    unreachable!()
                };
                self.advance(); // consume fd number
                self.advance(); // consume '>'
                self.advance(); // consume '&'
                self.advance(); // consume '-'
                                // Represent close fd as duplicate to -1
                redirects.push(Redirect::Duplicate { from_fd, to_fd: -1 });
            }
            // Check for file descriptor duplication: 2>&1
            // Lexer tokenizes "2>&1" as Number(2) + Gt + Ampersand + Number(1)
            // Must check this BEFORE error redirection since it's a longer pattern
            else if matches!(self.peek(), Some(Token::Number(_)))
                && matches!(self.peek_ahead(1), Some(Token::Gt))
                && matches!(self.peek_ahead(2), Some(Token::Ampersand))
                && matches!(self.peek_ahead(3), Some(Token::Number(_)))
            {
                // File descriptor duplication: 2>&1
                let from_fd = if let Some(Token::Number(n)) = self.peek() {
                    *n as i32
                } else {
                    unreachable!()
                };
                self.advance(); // consume from_fd number
                self.advance(); // consume '>'
                self.advance(); // consume '&'
                let to_fd = if let Some(Token::Number(n)) = self.peek() {
                    *n as i32
                } else {
                    unreachable!()
                };
                self.advance(); // consume to_fd number
                redirects.push(Redirect::Duplicate { from_fd, to_fd });
            } else if matches!(self.peek(), Some(Token::Number(_)))
                && matches!(self.peek_ahead(1), Some(Token::Gt))
            {
                // Error redirection: 2> file
                self.advance(); // consume number (file descriptor)
                self.advance(); // consume '>'
                let target = self.parse_redirect_target()?;
                redirects.push(Redirect::Error { target });
            } else if matches!(self.peek(), Some(Token::Number(_)))
                && matches!(self.peek_ahead(1), Some(Token::GtGt))
            {
                // Append error redirection: 2>> file
                self.advance(); // consume number (file descriptor)
                self.advance(); // consume '>>'
                let target = self.parse_redirect_target()?;
                redirects.push(Redirect::AppendError { target });
            } else if let Some(Token::HereString(content)) = self.peek() {
                // Issue #61: Here-string: <<< "string"
                let content = content.clone();
                self.advance(); // consume HereString token
                redirects.push(Redirect::HereString { content });
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
            } else if matches!(self.peek(), Some(Token::Ampersand))
                && matches!(self.peek_ahead(1), Some(Token::Gt))
            {
                // Combined redirection: &> file (redirects both stdout and stderr)
                self.advance(); // consume '&'
                self.advance(); // consume '>'
                let target = self.parse_redirect_target()?;
                redirects.push(Redirect::Combined { target });
            } else if matches!(self.peek(), Some(Token::Gt)) {
                // Output redirection: > file
                self.advance(); // consume '>'
                let target = self.parse_redirect_target()?;
                redirects.push(Redirect::Output { target });
            } else if let Some(Token::Identifier(s)) = self.peek() {
                // BUG-015, BUG-016, BUG-017 FIX: Handle special redirect operators
                match s.as_str() {
                    ">|" => {
                        // Noclobber redirect: >| file
                        self.advance(); // consume '>|'
                        let target = self.parse_redirect_target()?;
                        redirects.push(Redirect::Output { target });
                    }
                    "<>" => {
                        // Read-write redirect: <> file
                        self.advance(); // consume '<>'
                        let target = self.parse_redirect_target()?;
                        redirects.push(Redirect::Input { target }); // Treat as input for now
                    }
                    _ => {
                        // Regular argument
                        args.push(self.parse_expression()?);
                    }
                }
            } else if self.check(&Token::LeftBracket) {
                // Glob bracket pattern: [abc], [a-z], [!abc], [^abc], etc.
                // Collect the entire bracket expression as a literal
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
                            // [!abc] negation pattern
                            pattern.push('!');
                            self.advance();
                        }
                        _ => break,
                    }
                }

                if self.check(&Token::RightBracket) {
                    pattern.push(']');
                    self.advance(); // consume ']'
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
    /// BUG-002, BUG-003, BUG-004 FIX: Full arithmetic expression support
    ///
    /// Precedence (lowest to highest):
    ///   1. comma (,)
    ///   2. assignment (=)
    ///   3. ternary (? :)
    ///   4. logical or (||)
    ///   5. logical and (&&)
    ///   6. bitwise or (|)
    ///   7. bitwise xor (^)
    ///   8. bitwise and (&)
    ///   9. equality (== !=)
    ///   10. comparison (< <= > >=)
    ///   11. shift (<< >>)
    ///   12. additive (+ -)
    ///   13. multiplicative (* / %)
    ///   14. unary (- ~ !)
    ///   15. primary (number, variable, parentheses)
    fn parse_arithmetic_expr(&mut self, input: &str) -> ParseResult<ArithExpr> {
        let tokens = self.tokenize_arithmetic(input)?;
        let mut pos = 0;

        // Level 1: Comma operator (lowest precedence)
        fn parse_comma(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
            let mut left = parse_assign(tokens, pos)?;
            while *pos < tokens.len() && matches!(tokens[*pos], ArithToken::Comma) {
                *pos += 1;
                let right = parse_assign(tokens, pos)?;
                // Comma returns the right value, but we need to represent both
                // For now, just return right (simplified)
                left = right;
            }
            Ok(left)
        }

        // Level 2: Assignment
        fn parse_assign(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
            parse_ternary(tokens, pos)
        }

        // Level 3: Ternary (? :)
        fn parse_ternary(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
            let cond = parse_logical_or(tokens, pos)?;
            if *pos < tokens.len() && matches!(tokens[*pos], ArithToken::Question) {
                *pos += 1;
                let then_expr = parse_ternary(tokens, pos)?;
                if *pos >= tokens.len() || !matches!(tokens[*pos], ArithToken::Colon) {
                    return Err(ParseError::InvalidSyntax(
                        "Expected ':' in ternary expression".to_string(),
                    ));
                }
                *pos += 1;
                let else_expr = parse_ternary(tokens, pos)?;
                // Represent as: cond ? then : else
                // We'll use a hack: (cond * then) + (!cond * else) conceptually
                // But for parsing, we just accept it - evaluation handles it
                // Store as Add with special marker or just accept the structure
                return Ok(ArithExpr::Add(
                    Box::new(ArithExpr::Mul(Box::new(cond.clone()), Box::new(then_expr))),
                    Box::new(ArithExpr::Mul(
                        Box::new(ArithExpr::Sub(
                            Box::new(ArithExpr::Number(1)),
                            Box::new(cond),
                        )),
                        Box::new(else_expr),
                    )),
                ));
            }
            Ok(cond)
        }

        // Level 4: Logical OR
        fn parse_logical_or(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
            let mut left = parse_logical_and(tokens, pos)?;
            while *pos < tokens.len() && matches!(tokens[*pos], ArithToken::LogicalOr) {
                *pos += 1;
                let right = parse_logical_and(tokens, pos)?;
                // OR: if left != 0 then 1 else (right != 0)
                left = ArithExpr::Add(Box::new(left), Box::new(right)); // Simplified
            }
            Ok(left)
        }

        // Level 5: Logical AND
        fn parse_logical_and(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
            let mut left = parse_bitwise_or(tokens, pos)?;
            while *pos < tokens.len() && matches!(tokens[*pos], ArithToken::LogicalAnd) {
                *pos += 1;
                let right = parse_bitwise_or(tokens, pos)?;
                left = ArithExpr::Mul(Box::new(left), Box::new(right)); // Simplified
            }
            Ok(left)
        }

        // Level 6: Bitwise OR
        fn parse_bitwise_or(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
            let mut left = parse_bitwise_xor(tokens, pos)?;
            while *pos < tokens.len() && matches!(tokens[*pos], ArithToken::BitOr) {
                *pos += 1;
                let right = parse_bitwise_xor(tokens, pos)?;
                // Represent bitwise OR - for now store as add (semantic loss)
                left = ArithExpr::Add(Box::new(left), Box::new(right));
            }
            Ok(left)
        }

        // Level 7: Bitwise XOR
        fn parse_bitwise_xor(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
            let mut left = parse_bitwise_and(tokens, pos)?;
            while *pos < tokens.len() && matches!(tokens[*pos], ArithToken::BitXor) {
                *pos += 1;
                let right = parse_bitwise_and(tokens, pos)?;
                left = ArithExpr::Sub(Box::new(left), Box::new(right)); // Placeholder
            }
            Ok(left)
        }

        // Level 8: Bitwise AND
        fn parse_bitwise_and(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
            let mut left = parse_equality(tokens, pos)?;
            while *pos < tokens.len() && matches!(tokens[*pos], ArithToken::BitAnd) {
                *pos += 1;
                let right = parse_equality(tokens, pos)?;
                left = ArithExpr::Mul(Box::new(left), Box::new(right)); // Placeholder
            }
            Ok(left)
        }

        // Level 9: Equality (== !=)
        fn parse_equality(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
            let mut left = parse_comparison(tokens, pos)?;
            while *pos < tokens.len() {
                match &tokens[*pos] {
                    ArithToken::Eq | ArithToken::Ne => {
                        *pos += 1;
                        let right = parse_comparison(tokens, pos)?;
                        // Represent as subtraction (0 if equal)
                        left = ArithExpr::Sub(Box::new(left), Box::new(right));
                    }
                    _ => break,
                }
            }
            Ok(left)
        }

        // Level 10: Comparison (< <= > >=)
        fn parse_comparison(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
            let mut left = parse_shift(tokens, pos)?;
            while *pos < tokens.len() {
                match &tokens[*pos] {
                    ArithToken::Lt | ArithToken::Le | ArithToken::Gt | ArithToken::Ge => {
                        *pos += 1;
                        let right = parse_shift(tokens, pos)?;
                        left = ArithExpr::Sub(Box::new(left), Box::new(right));
                    }
                    _ => break,
                }
            }
            Ok(left)
        }

        // Level 11: Shift (<< >>)
        fn parse_shift(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
            let mut left = parse_additive(tokens, pos)?;
            while *pos < tokens.len() {
                match &tokens[*pos] {
                    ArithToken::ShiftLeft => {
                        *pos += 1;
                        let right = parse_additive(tokens, pos)?;
                        left = ArithExpr::Mul(Box::new(left), Box::new(right));
                    }
                    ArithToken::ShiftRight => {
                        *pos += 1;
                        let right = parse_additive(tokens, pos)?;
                        left = ArithExpr::Div(Box::new(left), Box::new(right));
                    }
                    _ => break,
                }
            }
            Ok(left)
        }

        // Level 12: Additive (+ -)
        fn parse_additive(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
            let mut left = parse_multiplicative(tokens, pos)?;
            while *pos < tokens.len() {
                match &tokens[*pos] {
                    ArithToken::Plus => {
                        *pos += 1;
                        let right = parse_multiplicative(tokens, pos)?;
                        left = ArithExpr::Add(Box::new(left), Box::new(right));
                    }
                    ArithToken::Minus => {
                        *pos += 1;
                        let right = parse_multiplicative(tokens, pos)?;
                        left = ArithExpr::Sub(Box::new(left), Box::new(right));
                    }
                    _ => break,
                }
            }
            Ok(left)
        }

        // Level 13: Multiplicative (* / %)
        fn parse_multiplicative(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
            let mut left = parse_unary(tokens, pos)?;
            while *pos < tokens.len() {
                match &tokens[*pos] {
                    ArithToken::Multiply => {
                        *pos += 1;
                        let right = parse_unary(tokens, pos)?;
                        left = ArithExpr::Mul(Box::new(left), Box::new(right));
                    }
                    ArithToken::Divide => {
                        *pos += 1;
                        let right = parse_unary(tokens, pos)?;
                        left = ArithExpr::Div(Box::new(left), Box::new(right));
                    }
                    ArithToken::Modulo => {
                        *pos += 1;
                        let right = parse_unary(tokens, pos)?;
                        left = ArithExpr::Mod(Box::new(left), Box::new(right));
                    }
                    _ => break,
                }
            }
            Ok(left)
        }

        // Level 14: Unary (- ~ !)
        fn parse_unary(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
            if *pos >= tokens.len() {
                return Err(ParseError::InvalidSyntax(
                    "Unexpected end of arithmetic expression".to_string(),
                ));
            }
            match &tokens[*pos] {
                ArithToken::Minus => {
                    *pos += 1;
                    let operand = parse_unary(tokens, pos)?;
                    Ok(ArithExpr::Sub(
                        Box::new(ArithExpr::Number(0)),
                        Box::new(operand),
                    ))
                }
                ArithToken::BitNot | ArithToken::LogicalNot => {
                    *pos += 1;
                    let operand = parse_unary(tokens, pos)?;
                    // Represent as -1 - x for bitwise not (approximation)
                    Ok(ArithExpr::Sub(
                        Box::new(ArithExpr::Number(-1)),
                        Box::new(operand),
                    ))
                }
                ArithToken::Plus => {
                    *pos += 1;
                    parse_unary(tokens, pos)
                }
                _ => parse_primary(tokens, pos),
            }
        }

        // Level 15: Primary (number, variable, parentheses)
        fn parse_primary(tokens: &[ArithToken], pos: &mut usize) -> ParseResult<ArithExpr> {
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
                    let expr = parse_comma(tokens, pos)?;
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

        parse_comma(&tokens, &mut pos)
    }

    /// Tokenize arithmetic expression string
    /// BUG-002, BUG-003, BUG-004, BUG-014 FIX: Extended arithmetic tokenizer
    fn tokenize_arithmetic(&self, input: &str) -> ParseResult<Vec<ArithToken>> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(&ch) = chars.peek() {
            match ch {
                ' ' | '\t' | '\n' => {
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
                // BUG-003 FIX: Comparison operators
                '<' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(ArithToken::Le);
                    } else if chars.peek() == Some(&'<') {
                        chars.next();
                        tokens.push(ArithToken::ShiftLeft);
                    } else {
                        tokens.push(ArithToken::Lt);
                    }
                }
                '>' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(ArithToken::Ge);
                    } else if chars.peek() == Some(&'>') {
                        chars.next();
                        tokens.push(ArithToken::ShiftRight);
                    } else {
                        tokens.push(ArithToken::Gt);
                    }
                }
                '=' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(ArithToken::Eq);
                    } else {
                        tokens.push(ArithToken::Assign);
                    }
                }
                '!' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(ArithToken::Ne);
                    } else {
                        tokens.push(ArithToken::LogicalNot);
                    }
                }
                '?' => {
                    chars.next();
                    tokens.push(ArithToken::Question);
                }
                ':' => {
                    chars.next();
                    tokens.push(ArithToken::Colon);
                }
                // BUG-004 FIX: Bitwise operators
                '&' => {
                    chars.next();
                    if chars.peek() == Some(&'&') {
                        chars.next();
                        tokens.push(ArithToken::LogicalAnd);
                    } else {
                        tokens.push(ArithToken::BitAnd);
                    }
                }
                '|' => {
                    chars.next();
                    if chars.peek() == Some(&'|') {
                        chars.next();
                        tokens.push(ArithToken::LogicalOr);
                    } else {
                        tokens.push(ArithToken::BitOr);
                    }
                }
                '^' => {
                    chars.next();
                    tokens.push(ArithToken::BitXor);
                }
                '~' => {
                    chars.next();
                    tokens.push(ArithToken::BitNot);
                }
                // BUG-014 FIX: Comma operator
                ',' => {
                    chars.next();
                    tokens.push(ArithToken::Comma);
                }
                '0'..='9' => {
                    let mut num_str = String::new();
                    // Check for hex (0x) or octal (0) prefix
                    if ch == '0' {
                        num_str.push(ch);
                        chars.next();
                        if chars.peek() == Some(&'x') || chars.peek() == Some(&'X') {
                            // Hex number - we just verified peek() so next() is guaranteed
                            if let Some(x_char) = chars.next() {
                                num_str.push(x_char);
                            }
                            while let Some(&c) = chars.peek() {
                                if c.is_ascii_hexdigit() {
                                    num_str.push(c);
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            let num = i64::from_str_radix(&num_str[2..], 16).map_err(|_| {
                                ParseError::InvalidSyntax(format!(
                                    "Invalid hex number: {}",
                                    num_str
                                ))
                            })?;
                            tokens.push(ArithToken::Number(num));
                            continue;
                        }
                        // Check if it's octal (starts with 0 and has more digits)
                        let mut is_octal = false;
                        while let Some(&c) = chars.peek() {
                            if c.is_ascii_digit() {
                                num_str.push(c);
                                chars.next();
                                is_octal = true;
                            } else {
                                break;
                            }
                        }
                        if is_octal && num_str.len() > 1 {
                            // Parse as octal
                            let num = i64::from_str_radix(&num_str, 8).unwrap_or_else(|_| {
                                // Fall back to decimal if not valid octal
                                num_str.parse::<i64>().unwrap_or(0)
                            });
                            tokens.push(ArithToken::Number(num));
                        } else {
                            tokens.push(ArithToken::Number(0));
                        }
                    } else {
                        while let Some(&c) = chars.peek() {
                            if c.is_ascii_digit() {
                                num_str.push(c);
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
                }
                // Variables (including $var references)
                '$' => {
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
                }
                'a'..='z' | 'A'..='Z' | '_' => {
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
            // BUG-012, BUG-013 FIX: Array literals (value1 value2) or ([0]=a [5]=b)
            Some(Token::LeftParen) => {
                self.advance(); // consume '('
                let mut elements = Vec::new();
                while !self.is_at_end() && !self.check(&Token::RightParen) {
                    // Handle sparse array [index]=value or regular value
                    if self.check(&Token::LeftBracket) {
                        self.advance(); // skip '['
                                        // Read index
                        let mut index = String::new();
                        while !self.is_at_end() && !self.check(&Token::RightBracket) {
                            match self.peek() {
                                Some(Token::Identifier(s)) | Some(Token::String(s)) => {
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
                        if self.check(&Token::RightBracket) {
                            self.advance(); // skip ']'
                        }
                        if self.check(&Token::Assign) {
                            self.advance(); // skip '='
                        }
                        // Parse the value
                        if !self.is_at_end() && !self.check(&Token::RightParen) {
                            let value = self.parse_expression()?;
                            // Store as [index]=value literal for now
                            elements.push(BashExpr::Literal(format!(
                                "[{}]={}",
                                index,
                                match &value {
                                    BashExpr::Literal(s) => s.clone(),
                                    BashExpr::Variable(v) => format!("${}", v),
                                    _ => "?".to_string(),
                                }
                            )));
                        }
                    } else if self.check(&Token::Newline) {
                        self.advance();
                    } else {
                        elements.push(self.parse_expression()?);
                    }
                }
                self.expect(Token::RightParen)?;
                Ok(BashExpr::Array(elements))
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

        // Issue #93: Handle bare command as condition
        // Example: `if grep -q pattern file; then` - the command's exit code is the condition
        // Check if we have a command identifier (not a unary test operator)
        if let Some(Token::Identifier(name)) = self.peek() {
            // Don't treat test operators as commands
            if !name.starts_with('-') {
                let cmd = self.parse_condition_command()?;
                return Ok(BashExpr::CommandCondition(Box::new(cmd)));
            }
        }

        // Fallback to regular expression (for backwards compatibility)
        self.parse_expression()
    }

    /// Issue #93: Parse a command used as a condition in if/while statements
    /// Similar to parse_command but stops at `then`, `do`, and doesn't include redirections
    fn parse_condition_command(&mut self) -> ParseResult<BashStmt> {
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

        // Parse arguments until semicolon, newline, then, do, or special tokens
        while !self.is_at_end()
            && !self.check(&Token::Newline)
            && !self.check(&Token::Semicolon)
            && !self.check(&Token::Then)
            && !self.check(&Token::Do)
            && !self.check(&Token::Pipe)
            && !self.check(&Token::And)
            && !self.check(&Token::Or)
            && !matches!(self.peek(), Some(Token::Comment(_)))
        {
            // Handle redirections (same as parse_command)
            if matches!(self.peek(), Some(Token::Number(_)))
                && matches!(self.peek_ahead(1), Some(Token::Gt))
                && matches!(self.peek_ahead(2), Some(Token::Ampersand))
                && matches!(self.peek_ahead(3), Some(Token::Number(_)))
            {
                let from_fd = if let Some(Token::Number(n)) = self.peek() {
                    *n as i32
                } else {
                    unreachable!()
                };
                self.advance();
                self.advance();
                self.advance();
                let to_fd = if let Some(Token::Number(n)) = self.peek() {
                    *n as i32
                } else {
                    unreachable!()
                };
                self.advance();
                redirects.push(Redirect::Duplicate { from_fd, to_fd });
            } else if matches!(self.peek(), Some(Token::Number(_)))
                && matches!(self.peek_ahead(1), Some(Token::Gt))
            {
                self.advance();
                self.advance();
                let target = self.parse_redirect_target()?;
                redirects.push(Redirect::Error { target });
            } else if matches!(self.peek(), Some(Token::Gt)) {
                self.advance();
                let target = self.parse_redirect_target()?;
                redirects.push(Redirect::Output { target });
            } else if matches!(self.peek(), Some(Token::GtGt)) {
                self.advance();
                let target = self.parse_redirect_target()?;
                redirects.push(Redirect::Append { target });
            } else if matches!(self.peek(), Some(Token::Lt)) {
                self.advance();
                let target = self.parse_redirect_target()?;
                redirects.push(Redirect::Input { target });
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

    fn parse_test_condition(&mut self) -> ParseResult<TestExpr> {
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
                "-f" | "-e" | "-s" => {
                    // -f: file exists and is regular file
                    // -e: file exists (any type)
                    // -s: file exists and has size > 0
                    // Issue #62: Added -s support
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
            // Skip newlines and semicolons between statements
            // Issue #60: Brace groups use semicolons as statement separators
            while self.check(&Token::Newline) || self.check(&Token::Semicolon) {
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

    // Issue #93: Test inline if/then/else/fi with command condition
    #[test]
    fn test_issue_93_inline_if_with_command_condition() {
        // This is the exact pattern from issue #93 that was failing
        let input = r#"if grep -q "pattern" "$file"; then echo "found"; else echo "not found"; fi"#;
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast.statements.len(),
            1,
            "Should parse single inline if statement"
        );
        match &ast.statements[0] {
            BashStmt::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                // The condition should be a CommandCondition
                assert!(
                    matches!(condition, BashExpr::CommandCondition(_)),
                    "Condition should be CommandCondition, got {:?}",
                    condition
                );

                // Should have then block
                assert!(!then_block.is_empty(), "Should have then block");

                // Should have else block
                assert!(else_block.is_some(), "Should have else block");
            }
            _ => panic!("Expected If statement, got {:?}", ast.statements[0]),
        }
    }

    // Issue #93: Test inline if with grep -q pattern
    #[test]
    fn test_issue_93_inline_if_grep_pattern() {
        let input = r#"if grep -q "MAX_QUEUE_DEPTH.*=.*3" "$BRIDGE"; then pass "1: found"; else fail "1: not found"; fi"#;
        let mut parser = BashParser::new(input).unwrap();
        let result = parser.parse();

        // This should NOT fail with "expected Then, found Identifier"
        assert!(
            result.is_ok(),
            "Parser should handle inline if/grep pattern, got: {:?}",
            result
        );
    }

    // Issue #93: Test while loop with command condition (simple case)
    #[test]
    fn test_issue_93_while_with_command_condition() {
        // Use a simpler while condition that doesn't have redirects
        let input = r#"
while grep -q "pattern" file.txt; do
    echo "found"
done
"#;
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();

        assert!(
            ast.statements
                .iter()
                .any(|s| matches!(s, BashStmt::While { .. })),
            "Should parse while with command condition"
        );
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

    // BUG-011: Function with subshell body
    #[test]
    fn test_parse_function_subshell_body() {
        let input = "myfunc() ( echo subshell )";

        let mut parser = BashParser::new(input).unwrap();
        let ast = parser
            .parse()
            .expect("Should parse function with subshell body");
        assert!(
            ast.statements
                .iter()
                .any(|s| matches!(s, BashStmt::Function { .. })),
            "Should find function statement"
        );
    }

    #[test]
    fn test_glob_bracket_pattern() {
        // Basic bracket glob
        let input = "echo [abc].txt";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().expect("Should parse [abc].txt");
        assert!(matches!(&ast.statements[0], BashStmt::Command { args, .. } if !args.is_empty()));

        // Negated bracket glob [!abc]
        let input2 = "echo [!abc].txt";
        let mut parser2 = BashParser::new(input2).unwrap();
        parser2.parse().expect("Should parse [!abc].txt");
    }

    // BUG-018: Test coproc syntax
    #[test]
    fn test_parse_coproc() {
        // Named coproc
        let input = "coproc myproc { cat; }";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().expect("Should parse named coproc");
        assert!(matches!(
            &ast.statements[0],
            BashStmt::Coproc {
                name: Some(n),
                ..
            } if n == "myproc"
        ));

        // Anonymous coproc
        let input2 = "coproc { cat; }";
        let mut parser2 = BashParser::new(input2).unwrap();
        let ast2 = parser2.parse().expect("Should parse anonymous coproc");
        assert!(matches!(
            &ast2.statements[0],
            BashStmt::Coproc { name: None, .. }
        ));
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
