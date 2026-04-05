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
}

include!("parser_expr_methods.rs");
