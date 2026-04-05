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
    Until,
    Do,
    Done,
    Case,
    Esac,
    In,
    Function,
    Return,
    Export,
    Local,
    Coproc, // BUG-018: coproc keyword
    Select, // F017: select keyword for select-in-do-done loops

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
    GtGt,               // >> (append redirection)
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
    Variable(String),                               // $VAR
    ArithmeticExpansion(String),                    // $((expr))
    CommandSubstitution(String),                    // $(command)
    Heredoc { delimiter: String, content: String }, // <<DELIMITER
    HereString(String),                             // <<< string (Issue #61)
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

include!("lexer_methods.rs");
