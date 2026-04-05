//! Bash Parser
//!
//! Parses token stream from lexer into bash AST.
//! Implements recursive descent parsing for bash syntax.

use super::ast::*;
use super::lexer::{Lexer, LexerError, Token};
use thiserror::Error;

// Re-export error display functions that were extracted to parser_error_display.rs
pub use super::parser_error_display::format_parse_diagnostic;

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

impl ParseError {
    /// Extract line number from any parse error variant
    pub fn line(&self) -> Option<usize> {
        match self {
            Self::UnexpectedToken { line, .. } => Some(*line),
            Self::LexerError(LexerError::UnexpectedChar(_, line, _)) => Some(*line),
            Self::LexerError(LexerError::UnterminatedString(line, _)) => Some(*line),
            _ => None,
        }
    }

    /// Extract column number from any parse error variant
    pub fn column(&self) -> Option<usize> {
        match self {
            Self::LexerError(LexerError::UnexpectedChar(_, _, col)) => Some(*col),
            Self::LexerError(LexerError::UnterminatedString(_, col)) => Some(*col),
            _ => None,
        }
    }
}

pub struct BashParser {
    pub(crate) tokens: Vec<Token>,
    /// Character positions of each token in the source string
    pub(crate) token_positions: Vec<usize>,
    pub(crate) position: usize,
    pub(crate) current_line: usize,
    pub(crate) tracer: Option<crate::tracing::TraceManager>,
    /// Original source code, stored for error diagnostics
    pub(crate) source: String,
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
        let (tokens, token_positions) = lexer.tokenize_with_positions()?;

        Ok(Self {
            tokens,
            token_positions,
            position: 0,
            current_line: 1,
            tracer: None,
            source: input.to_string(),
        })
    }

    /// Get the original source code (for error diagnostics)
    pub fn source(&self) -> &str {
        &self.source
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
        // Contract: parser-soundness-v1.yaml precondition (pv codegen)
        contract_pre_parse!(self.tokens);
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
                // Skip newlines and semicolons between statements
                while self.check(&Token::Newline) || self.check(&Token::Semicolon) {
                    self.advance();
                }
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
                // Skip newlines and semicolons after statement
                while self.check(&Token::Newline) || self.check(&Token::Semicolon) {
                    self.advance();
                }
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

    pub(crate) fn parse_statement(&mut self) -> ParseResult<BashStmt> {
        // Skip comments and collect them
        if let Some(Token::Comment(text)) = self.peek() {
            let comment = text.clone();
            self.advance();
            return Ok(BashStmt::Comment {
                text: comment,
                span: Span::new(self.current_line, 0, self.current_line, 0),
            });
        }

        // Parse first statement (could be part of pipeline)
        let first_stmt = match self.peek() {
            // Bash allows keywords as variable names (e.g., fi=1, for=2, while=3)
            // Check for assignment pattern first before treating as control structure
            Some(t) if Self::is_keyword_token(t) && self.peek_ahead(1) == Some(&Token::Assign) => {
                self.parse_assignment(false)
            }
            // Control flow statements (if/for/while/until/case/select)
            Some(Token::If) => self.parse_if(),
            Some(Token::While) => self.parse_while(),
            Some(Token::Until) => self.parse_until(),
            Some(Token::For) => self.parse_for(),
            Some(Token::Select) => self.parse_select(), // F017: select statement
            Some(Token::Case) => self.parse_case(),
            // Declaration statements (function/return/export/local/coproc)
            Some(Token::Function) => self.parse_function(),
            Some(Token::Return) => self.parse_return(),
            Some(Token::Export) => self.parse_export(),
            Some(Token::Local) => self.parse_local(),
            Some(Token::Coproc) => self.parse_coproc(), // BUG-018
            // Identifiers: assignment, function def shorthand, or command
            Some(Token::Identifier(_)) => self.parse_identifier_statement(),
            // Issue #67: Handle standalone arithmetic ((expr)) as a command
            Some(Token::ArithmeticExpansion(_)) => self.parse_standalone_arithmetic(),
            // Compound commands: brace group, subshell, test, extended test
            Some(Token::LeftBrace) => self.parse_brace_group(),
            Some(Token::LeftParen) => self.parse_subshell(),
            Some(Token::LeftBracket) => self.parse_test_command(),
            Some(Token::DoubleLeftBracket) => self.parse_extended_test_command(),
            _ => self.parse_command(),
        }?;

        // Handle pipeline, logical operators, and background
        self.parse_statement_tail(first_stmt)
    }

    /// Check if a token is a keyword that can also serve as a variable name in assignments
    fn is_keyword_token(token: &Token) -> bool {
        matches!(
            token,
            Token::If
                | Token::Then
                | Token::Elif
                | Token::Else
                | Token::Fi
                | Token::While
                | Token::Until
                | Token::For
                | Token::Do
                | Token::Done
                | Token::Case
                | Token::Esac
                | Token::In
                | Token::Function
                | Token::Return
        )
    }

    /// Parse an identifier that could be an assignment, function definition, or command
    fn parse_identifier_statement(&mut self) -> ParseResult<BashStmt> {
        // Could be assignment, function, or command
        // BUG-012 FIX: Also handle += for array append
        // F019 FIX: Also handle array element assignment: name[index]=value
        if self.peek_ahead(1) == Some(&Token::Assign)
            || matches!(self.peek_ahead(1), Some(Token::Identifier(s)) if s == "+=")
        {
            self.parse_assignment(false)
        } else if self.peek_ahead(1) == Some(&Token::LeftBracket)
            && self.peek_ahead(3) == Some(&Token::RightBracket)
            && self.peek_ahead(4) == Some(&Token::Assign)
        {
            // F019: Array element assignment: hash[key]=value
            // Must have pattern: name[index]=value (with ] followed by =)
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

    /// Issue #67: Handle standalone arithmetic ((expr)) as a command
    fn parse_standalone_arithmetic(&mut self) -> ParseResult<BashStmt> {
        let arith_expr = match self.peek() {
            Some(Token::ArithmeticExpansion(expr)) => expr.clone(),
            _ => return Err(self.syntax_error("arithmetic expansion")),
        };
        self.advance();
        Ok(BashStmt::Command {
            name: ":".to_string(),
            args: vec![BashExpr::Literal(format!("$(({}))", arith_expr))],
            redirects: vec![],
            span: Span::new(self.current_line, 0, self.current_line, 0),
        })
    }

    /// Parse pipeline, logical operators (&&, ||), and background (&) after the first statement
    fn parse_statement_tail(&mut self, first_stmt: BashStmt) -> ParseResult<BashStmt> {
        // Check for pipeline: cmd1 | cmd2 | cmd3
        let stmt = if self.check(&Token::Pipe) {
            let mut commands = vec![first_stmt];

            // Collect all piped commands
            while self.check(&Token::Pipe) {
                self.advance(); // consume '|'

                // Skip newlines after pipe
                self.skip_newlines();

                // Parse next command in pipeline — compound commands
                // are valid on the right side of a pipe:
                //   cmd | while read line; do ...; done
                //   cmd | if ...; then ...; fi
                //   cmd | { cmd1; cmd2; }
                let next_cmd = self.parse_pipeline_rhs()?;
                commands.push(next_cmd);
            }

            // Return pipeline with all collected commands
            BashStmt::Pipeline {
                commands,
                span: Span::new(self.current_line, 0, self.current_line, 0),
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
                span: Span::new(self.current_line, 0, self.current_line, 0),
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
                span: Span::new(self.current_line, 0, self.current_line, 0),
            });
        }

        // Consume trailing & (background operator) — acts as statement terminator
        if self.check(&Token::Ampersand) {
            self.advance();
        }

        // Not a pipeline or logical list, return the statement
        Ok(stmt)
    }
}
