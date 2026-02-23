//! Declaration parsing: functions, return, export, local, assignment.
//!
//! Extracted from `parser.rs` to reduce per-file complexity.

use super::ast::*;
use super::lexer::Token;
use super::parser::{BashParser, ParseError, ParseResult};

impl BashParser {
    pub(crate) fn parse_function(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::Function)?;

        let name = if let Some(Token::Identifier(n)) = self.peek() {
            let fn_name = n.clone();
            self.advance();
            fn_name
        } else {
            return Err(self.syntax_error("function name after 'function'"));
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
            span: Span::new(self.current_line, 0, self.current_line, 0),
        })
    }

    pub(crate) fn parse_function_shorthand(&mut self) -> ParseResult<BashStmt> {
        // Parse name() { ... } or name() ( ... ) syntax without 'function' keyword
        let name = if let Some(Token::Identifier(n)) = self.peek() {
            let fn_name = n.clone();
            self.advance();
            fn_name
        } else {
            return Err(self.syntax_error("function name"));
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
                span: Span::new(self.current_line, 0, self.current_line, 0),
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
                span: Span::new(self.current_line, 0, self.current_line, 0),
            })
        }
    }

    pub(crate) fn parse_return(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::Return)?;

        let code = if self.check(&Token::Newline) || self.is_at_end() {
            None
        } else {
            Some(self.parse_expression()?)
        };

        Ok(BashStmt::Return {
            code,
            span: Span::new(self.current_line, 0, self.current_line, 0),
        })
    }

    pub(crate) fn parse_export(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::Export)?;
        self.parse_assignment(true)
    }

    pub(crate) fn parse_local(&mut self) -> ParseResult<BashStmt> {
        self.expect(Token::Local)?;

        // Skip flags like -i, -r, -a, -A (bash-specific, dropped for POSIX)
        while !self.is_at_end() {
            if let Some(Token::Identifier(s)) = self.peek() {
                if s.starts_with('-') && s.len() > 1 && s[1..].chars().all(|c| c.is_alphabetic()) {
                    self.advance(); // skip flag like "-i", "-r"
                    continue;
                }
            }
            break;
        }

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
                    span: Span::new(self.current_line, 0, self.current_line, 0),
                })
            }
        } else {
            // Just "local" by itself - treat as command
            Ok(BashStmt::Command {
                name: "local".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::new(self.current_line, 0, self.current_line, 0),
            })
        }
    }

    pub(crate) fn parse_assignment(&mut self, exported: bool) -> ParseResult<BashStmt> {
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

        // F019 FIX: Handle array element assignment: name[index]=value
        let index = if self.check(&Token::LeftBracket) {
            self.advance(); // consume '['
            let idx = match self.peek() {
                Some(Token::Identifier(s)) => {
                    let idx_str = s.clone();
                    self.advance();
                    idx_str
                }
                Some(Token::Number(n)) => {
                    let idx_str = n.to_string();
                    self.advance();
                    idx_str
                }
                Some(Token::String(s)) => {
                    let idx_str = s.clone();
                    self.advance();
                    idx_str
                }
                Some(Token::Variable(v)) => {
                    let idx_str = format!("${}", v);
                    self.advance();
                    idx_str
                }
                _ => {
                    return Err(ParseError::InvalidSyntax(
                        "Expected array index".to_string(),
                    ))
                }
            };
            self.expect(Token::RightBracket)?;
            Some(idx)
        } else {
            None
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
            index,
            value,
            exported,
            span: Span::new(self.current_line, 0, self.current_line, 0),
        })
    }
}
