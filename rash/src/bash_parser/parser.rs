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

pub struct BashParser {
    tokens: Vec<Token>,
    position: usize,
    current_line: usize,
}

impl BashParser {
    pub fn new(input: &str) -> ParseResult<Self> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;

        Ok(Self {
            tokens,
            position: 0,
            current_line: 1,
        })
    }

    pub fn parse(&mut self) -> ParseResult<BashAst> {
        let start_time = std::time::Instant::now();
        let mut statements = Vec::new();

        while !self.is_at_end() {
            self.skip_newlines();
            if self.is_at_end() {
                break;
            }

            let stmt = self.parse_statement()?;
            statements.push(stmt);

            self.skip_newlines();
        }

        let parse_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(BashAst {
            statements,
            metadata: AstMetadata {
                source_file: None,
                line_count: self.current_line,
                parse_time_ms,
            },
        })
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
            Some(Token::If) => self.parse_if(),
            Some(Token::While) => self.parse_while(),
            Some(Token::For) => self.parse_for(),
            Some(Token::Function) => self.parse_function(),
            Some(Token::Return) => self.parse_return(),
            Some(Token::Export) => self.parse_export(),
            Some(Token::Local) => self.parse_local(),
            Some(Token::Identifier(_)) => {
                // Could be assignment or command
                if self.peek_ahead(1) == Some(&Token::Assign) {
                    self.parse_assignment(false)
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
        if let Some(Token::Identifier(word)) = self.peek() {
            if word == "in" {
                self.advance();
            } else {
                return Err(ParseError::InvalidSyntax(
                    "Expected 'in' after for variable".to_string(),
                ));
            }
        }

        let items = self.parse_expression()?;

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
                span: Span::dummy(),
            })
        }
    }

    fn parse_assignment(&mut self, exported: bool) -> ParseResult<BashStmt> {
        let name = if let Some(Token::Identifier(n)) = self.peek() {
            let var_name = n.clone();
            self.advance();
            var_name
        } else {
            return Err(ParseError::InvalidSyntax(
                "Expected variable name in assignment".to_string(),
            ));
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

        // Parse arguments until newline or special token
        // Also stop at comments (BUILTIN-001: colon no-op with comments)
        while !self.is_at_end()
            && !self.check(&Token::Newline)
            && !self.check(&Token::Semicolon)
            && !self.check(&Token::Pipe)
            && !matches!(self.peek(), Some(Token::Comment(_)))
        {
            args.push(self.parse_expression()?);
        }

        Ok(BashStmt::Command {
            name,
            args,
            span: Span::dummy(),
        })
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
            Some(Token::Identifier(op)) if matches!(op.as_str(), "-eq" | "-ne" | "-lt" | "-le" | "-gt" | "-ge") => {
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
}
