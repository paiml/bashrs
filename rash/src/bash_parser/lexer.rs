//! Bash Lexer
//!
//! Tokenizes bash scripts into a stream of tokens for parsing.
//! Handles shell-specific quirks like variable expansion, quoting, etc.

use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    // Keywords
    If,
    Then,
    Elif,
    Else,
    Fi,
    For,
    While,
    Do,
    Done,
    Case,
    Esac,
    In,
    Function,
    Return,
    Export,
    Local,

    // Identifiers and literals
    Identifier(String),
    String(String),
    Number(i64),

    // Operators
    Assign,             // =
    Eq,                 // ==
    Ne,                 // !=
    Lt,                 // <
    Le,                 // <=
    Gt,                 // >
    Ge,                 // >=
    And,                // &&
    Or,                 // ||
    Not,                // !
    Pipe,               // |
    Semicolon,          // ;
    Ampersand,          // &
    Dollar,             // $
    LeftParen,          // (
    RightParen,         // )
    LeftBrace,          // {
    RightBrace,         // }
    LeftBracket,        // [
    RightBracket,       // ]
    DoubleLeftBracket,  // [[
    DoubleRightBracket, // ]]

    // Special
    Variable(String),            // $VAR
    ArithmeticExpansion(String), // $((expr))
    CommandSubstitution(String), // $(command)
    Comment(String),
    Newline,
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::If => write!(f, "if"),
            Token::Then => write!(f, "then"),
            Token::Identifier(s) => write!(f, "Identifier({})", s),
            Token::String(s) => write!(f, "String({})", s),
            Token::Number(n) => write!(f, "Number({})", n),
            Token::Variable(v) => write!(f, "${}", v),
            Token::ArithmeticExpansion(e) => write!(f, "$(({})", e),
            Token::CommandSubstitution(c) => write!(f, "$({})", c),
            Token::Comment(c) => write!(f, "#{}", c),
            Token::Eof => write!(f, "EOF"),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("Unexpected character '{0}' at line {1}, column {2}")]
    UnexpectedChar(char, usize, usize),

    #[error("Unterminated string at line {0}, column {1}")]
    UnterminatedString(usize, usize),

    #[error("Invalid number format: {0}")]
    InvalidNumber(String),
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace_except_newline();

            if self.is_at_end() {
                tokens.push(Token::Eof);
                break;
            }

            let token = self.next_token()?;
            tokens.push(token.clone());

            if token == Token::Eof {
                break;
            }
        }

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, LexerError> {
        if self.is_at_end() {
            return Ok(Token::Eof);
        }

        let ch = self.current_char();

        // Comments
        if ch == '#' {
            return Ok(self.read_comment());
        }

        // Newlines
        if ch == '\n' {
            self.advance();
            return Ok(Token::Newline);
        }

        // Variables
        if ch == '$' {
            return self.read_variable();
        }

        // Strings
        if ch == '"' || ch == '\'' {
            return self.read_string(ch);
        }

        // Numbers
        if ch.is_ascii_digit() {
            return self.read_number();
        }

        // Identifiers and keywords
        if ch.is_alphabetic() || ch == '_' {
            return Ok(self.read_identifier_or_keyword());
        }

        // Bare words (paths, globs, etc) - must come before operators
        // These are unquoted strings that can contain /  * . - : + % etc
        // Note: ':' is included for bash builtin no-op command (BUILTIN-001)
        // Note: '+' and '%' are included for flags like date +%FORMAT (PARSER-ENH-001)
        if ch == '/'
            || ch == '.'
            || ch == '-'
            || ch == '*'
            || ch == '~'
            || ch == ':'
            || ch == '+'
            || ch == '%'
        {
            return Ok(self.read_bare_word());
        }

        // Operators and symbols
        self.read_operator()
    }

    fn current_char(&self) -> char {
        self.input[self.position]
    }

    fn peek_char(&self, offset: usize) -> Option<char> {
        self.input.get(self.position + offset).copied()
    }

    fn advance(&mut self) -> char {
        let ch = self.current_char();
        self.position += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        ch
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    fn skip_whitespace_except_newline(&mut self) {
        while !self.is_at_end() {
            let ch = self.current_char();
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_comment(&mut self) -> Token {
        self.advance(); // skip '#'
        let mut comment = String::new();
        while !self.is_at_end() && self.current_char() != '\n' {
            comment.push(self.advance());
        }
        Token::Comment(comment)
    }

    fn read_variable(&mut self) -> Result<Token, LexerError> {
        self.advance(); // skip '$'

        // Check for arithmetic expansion $((...)) vs command substitution $(cmd)
        if !self.is_at_end() && self.current_char() == '(' {
            if let Some('(') = self.peek_char(1) {
                // Double paren: $((...)) = arithmetic expansion
                return self.read_arithmetic_expansion();
            } else {
                // Single paren: $(cmd) = command substitution
                return self.read_command_substitution();
            }
        }

        // Check for $$ (process ID special variable)
        if !self.is_at_end() && self.current_char() == '$' {
            self.advance(); // skip second '$'
                            // Return special variable name for process ID
                            // Using "$" as the variable name to represent $$
            return Ok(Token::Variable("$".to_string()));
        }

        // Check for $@ (all positional parameters special variable)
        if !self.is_at_end() && self.current_char() == '@' {
            self.advance(); // skip '@'
                            // Return special variable name for all positional parameters
                            // Using "@" as the variable name to represent $@
            return Ok(Token::Variable("@".to_string()));
        }

        let mut var_name = String::new();

        // Handle ${VAR} syntax
        if !self.is_at_end() && self.current_char() == '{' {
            self.advance();
            while !self.is_at_end() && self.current_char() != '}' {
                var_name.push(self.advance());
            }
            if !self.is_at_end() {
                self.advance(); // skip '}'
            }
        } else {
            // Handle $VAR syntax
            while !self.is_at_end() {
                let ch = self.current_char();
                if ch.is_alphanumeric() || ch == '_' {
                    var_name.push(self.advance());
                } else {
                    break;
                }
            }
        }

        Ok(Token::Variable(var_name))
    }

    fn read_arithmetic_expansion(&mut self) -> Result<Token, LexerError> {
        // Skip '(('
        self.advance(); // skip first '('
        self.advance(); // skip second '('

        let mut expr = String::new();
        let mut paren_depth = 0;

        while !self.is_at_end() {
            let ch = self.current_char();

            // Handle nested parentheses
            if ch == '(' {
                paren_depth += 1;
                expr.push(self.advance());
            } else if ch == ')' {
                // Check if this closes the arithmetic expansion
                if paren_depth == 0 && self.peek_char(1) == Some(')') {
                    self.advance(); // skip first ')'
                    self.advance(); // skip second ')'
                    break;
                } else {
                    paren_depth -= 1;
                    expr.push(self.advance());
                }
            } else {
                expr.push(self.advance());
            }
        }

        Ok(Token::ArithmeticExpansion(expr))
    }

    fn read_command_substitution(&mut self) -> Result<Token, LexerError> {
        // Skip '('
        self.advance(); // skip '('

        let mut command = String::new();
        let mut paren_depth = 0;

        while !self.is_at_end() {
            let ch = self.current_char();

            // Handle nested command substitutions: $(outer $(inner))
            if ch == '(' {
                paren_depth += 1;
                command.push(self.advance());
            } else if ch == ')' {
                // Check if this closes the command substitution
                if paren_depth == 0 {
                    self.advance(); // skip closing ')'
                    break;
                } else {
                    paren_depth -= 1;
                    command.push(self.advance());
                }
            } else {
                command.push(self.advance());
            }
        }

        Ok(Token::CommandSubstitution(command))
    }

    fn read_string(&mut self, quote: char) -> Result<Token, LexerError> {
        let start_line = self.line;
        let start_col = self.column;

        self.advance(); // skip opening quote
        let mut string = String::new();

        while !self.is_at_end() && self.current_char() != quote {
            let ch = self.advance();
            if ch == '\\' && !self.is_at_end() {
                // Handle escape sequences
                let escaped = self.advance();
                match escaped {
                    'n' => string.push('\n'),
                    't' => string.push('\t'),
                    'r' => string.push('\r'),
                    '\\' => string.push('\\'),
                    _ => {
                        string.push('\\');
                        string.push(escaped);
                    }
                }
            } else {
                string.push(ch);
            }
        }

        if self.is_at_end() {
            return Err(LexerError::UnterminatedString(start_line, start_col));
        }

        self.advance(); // skip closing quote
        Ok(Token::String(string))
    }

    fn read_number(&mut self) -> Result<Token, LexerError> {
        let mut num_str = String::new();

        while !self.is_at_end() && self.current_char().is_ascii_digit() {
            num_str.push(self.advance());
        }

        num_str
            .parse::<i64>()
            .map(Token::Number)
            .map_err(|_| LexerError::InvalidNumber(num_str))
    }

    fn read_identifier_or_keyword(&mut self) -> Token {
        let mut ident = String::new();

        while !self.is_at_end() {
            let ch = self.current_char();
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(self.advance());
            } else {
                break;
            }
        }

        // Check for keywords
        match ident.as_str() {
            "if" => Token::If,
            "then" => Token::Then,
            "elif" => Token::Elif,
            "else" => Token::Else,
            "fi" => Token::Fi,
            "for" => Token::For,
            "while" => Token::While,
            "do" => Token::Do,
            "done" => Token::Done,
            "case" => Token::Case,
            "esac" => Token::Esac,
            "in" => Token::In,
            "function" => Token::Function,
            "return" => Token::Return,
            "export" => Token::Export,
            "local" => Token::Local,
            _ => Token::Identifier(ident),
        }
    }

    fn read_bare_word(&mut self) -> Token {
        let mut word = String::new();

        while !self.is_at_end() {
            let ch = self.current_char();
            // Bare words can contain alphanumeric, path separators, globs, dots, dashes, plus signs, percent signs
            // Note: '+' and '%' added for date +%FORMAT support (PARSER-ENH-001)
            if ch.is_alphanumeric()
                || ch == '/'
                || ch == '.'
                || ch == '-'
                || ch == '_'
                || ch == '*'
                || ch == '?'
                || ch == '~'
                || ch == ':'
                || ch == '+'
                || ch == '%'
            {
                word.push(self.advance());
            } else {
                break;
            }
        }

        Token::Identifier(word)
    }

    fn read_operator(&mut self) -> Result<Token, LexerError> {
        let ch = self.current_char();
        let next_ch = self.peek_char(1);

        let token = match (ch, next_ch) {
            ('=', Some('=')) => {
                self.advance();
                self.advance();
                Token::Eq
            }
            ('!', Some('=')) => {
                self.advance();
                self.advance();
                Token::Ne
            }
            ('<', Some('=')) => {
                self.advance();
                self.advance();
                Token::Le
            }
            ('>', Some('=')) => {
                self.advance();
                self.advance();
                Token::Ge
            }
            ('&', Some('&')) => {
                self.advance();
                self.advance();
                Token::And
            }
            ('|', Some('|')) => {
                self.advance();
                self.advance();
                Token::Or
            }
            ('[', Some('[')) => {
                self.advance();
                self.advance();
                Token::DoubleLeftBracket
            }
            (']', Some(']')) => {
                self.advance();
                self.advance();
                Token::DoubleRightBracket
            }
            ('=', _) => {
                self.advance();
                Token::Assign
            }
            ('<', _) => {
                self.advance();
                Token::Lt
            }
            ('>', _) => {
                self.advance();
                Token::Gt
            }
            ('!', _) => {
                self.advance();
                Token::Not
            }
            ('|', _) => {
                self.advance();
                Token::Pipe
            }
            (';', _) => {
                self.advance();
                Token::Semicolon
            }
            ('&', _) => {
                self.advance();
                Token::Ampersand
            }
            ('(', _) => {
                self.advance();
                Token::LeftParen
            }
            (')', _) => {
                self.advance();
                Token::RightParen
            }
            ('{', _) => {
                self.advance();
                Token::LeftBrace
            }
            ('}', _) => {
                self.advance();
                Token::RightBrace
            }
            ('[', _) => {
                self.advance();
                Token::LeftBracket
            }
            (']', _) => {
                self.advance();
                Token::RightBracket
            }
            _ => {
                return Err(LexerError::UnexpectedChar(ch, self.line, self.column));
            }
        };

        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_assignment() {
        let mut lexer = Lexer::new("FOO=bar");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::Identifier("FOO".to_string()));
        assert_eq!(tokens[1], Token::Assign);
        assert_eq!(tokens[2], Token::Identifier("bar".to_string()));
    }

    #[test]
    fn test_tokenize_if_statement() {
        let mut lexer = Lexer::new("if [ $x == 1 ]; then");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::If);
        assert_eq!(tokens[1], Token::LeftBracket);
        assert!(matches!(tokens[2], Token::Variable(_)));
    }

    #[test]
    fn test_tokenize_string() {
        let mut lexer = Lexer::new(r#""hello world""#);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::String("hello world".to_string()));
    }

    #[test]
    fn test_tokenize_comment() {
        let mut lexer = Lexer::new("# This is a comment");
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0], Token::Comment(_)));
    }

    // EXTREME TDD - RED Phase: Test for date +FORMAT support
    // This test is EXPECTED TO FAIL until lexer enhancement is implemented
    #[test]
    fn test_lexer_plus_in_command_args() {
        let input = "date +%s";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        // Expected tokens: [Identifier("date"), Identifier("+%s"), Eof]
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Identifier("date".to_string()));
        assert_eq!(tokens[1], Token::Identifier("+%s".to_string()));
        assert_eq!(tokens[2], Token::Eof);
    }

    #[test]
    fn test_lexer_date_format_quoted() {
        let input = r#"date '+%Y-%m-%d %H:%M:%S'"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        // Expected tokens: [Identifier("date"), String("+%Y-%m-%d %H:%M:%S"), Eof]
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Identifier("date".to_string()));
        assert_eq!(tokens[1], Token::String("+%Y-%m-%d %H:%M:%S".to_string()));
        assert_eq!(tokens[2], Token::Eof);
    }

    #[test]
    fn test_lexer_plus_in_various_contexts() {
        // Test +%Y%m%d%H%M%S format
        let input = "date +%Y%m%d%H%M%S";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[1], Token::Identifier("+%Y%m%d%H%M%S".to_string()));

        // Test bare +x flag
        let input2 = "some_cmd +x";
        let mut lexer2 = Lexer::new(input2);
        let tokens2 = lexer2.tokenize().unwrap();
        assert_eq!(tokens2[1], Token::Identifier("+x".to_string()));
    }

    #[test]
    fn test_lexer_arithmetic_expansion_basic() {
        let input = "y=$((x + 1))";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        // Expected: [Identifier("y"), Assign, ArithmeticExpansion("x + 1"), Eof]
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::Identifier("y".to_string()));
        assert_eq!(tokens[1], Token::Assign);
        assert_eq!(tokens[2], Token::ArithmeticExpansion("x + 1".to_string()));
        assert_eq!(tokens[3], Token::Eof);
    }

    #[test]
    fn test_lexer_arithmetic_expansion_complex() {
        let input = "sum=$((a + b))";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[2], Token::ArithmeticExpansion("a + b".to_string()));

        let input2 = "diff=$((a - b))";
        let mut lexer2 = Lexer::new(input2);
        let tokens2 = lexer2.tokenize().unwrap();

        assert_eq!(tokens2[2], Token::ArithmeticExpansion("a - b".to_string()));
    }

    #[test]
    fn test_lexer_arithmetic_expansion_nested_parens() {
        let input = "result=$(((a + b) * c))";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens[2],
            Token::ArithmeticExpansion("(a + b) * c".to_string())
        );
    }
}
