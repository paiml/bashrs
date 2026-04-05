// REPL-015-002: Syntax Highlighting in REPL
//
// Provides terminal syntax highlighting for bash code
//
// Quality gates:
// - EXTREME TDD: RED → GREEN → REFACTOR → PROPERTY → MUTATION
// - Unit tests: 8+ scenarios
// - Property tests: 2+ generators
// - Complexity: <10 per function

/// ANSI color codes for terminal highlighting
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD_BLUE: &str = "\x1b[1;34m"; // Keywords
    pub const GREEN: &str = "\x1b[32m"; // Strings
    pub const YELLOW: &str = "\x1b[33m"; // Variables
    pub const CYAN: &str = "\x1b[36m"; // Commands
    pub const GRAY: &str = "\x1b[90m"; // Comments
    pub const MAGENTA: &str = "\x1b[35m"; // Operators
}

/// Token type for syntax highlighting
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    Keyword,    // if, then, while, for, do, done, case, esac, function
    String,     // "..." or '...'
    Variable,   // $var, ${var}, $?
    Command,    // First word in pipeline
    Comment,    // # comment
    Operator,   // |, >, <, &&, ||, ;
    Whitespace, // Spaces, tabs
    Text,       // Everything else
}

/// Token with type and position
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub text: String,
    pub start: usize,
    pub end: usize,
}

impl Token {
    pub fn new(token_type: TokenType, text: String, start: usize, end: usize) -> Self {
        Self {
            token_type,
            text,
            start,
            end,
        }
    }
}

/// Check if word is a bash keyword
///
/// # Examples
///
/// ```
/// use bashrs::repl::highlighting::is_keyword;
///
/// assert!(is_keyword("if"));
/// assert!(is_keyword("then"));
/// assert!(!is_keyword("echo"));
/// ```
pub fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "if" | "then"
            | "else"
            | "elif"
            | "fi"
            | "for"
            | "while"
            | "do"
            | "done"
            | "case"
            | "esac"
            | "in"
            | "function"
            | "select"
            | "until"
    )
}

/// Tokenize bash input into a sequence of tokens
///
/// Parses bash syntax into tokens including keywords, strings, variables,
/// commands, operators, comments, whitespace, and plain text.
///
/// # Arguments
///
/// * `input` - The bash code to tokenize
///
/// # Returns
///
/// A vector of tokens with their types and positions
///
/// # Examples
///
/// ```
/// use bashrs::repl::highlighting::tokenize;
///
/// let tokens = tokenize("echo $HOME");
/// assert_eq!(tokens.len(), 3); // "echo", " ", "$HOME"
/// ```
/// Tokenize a comment starting with #
fn tokenize_comment<I>(chars: &mut std::iter::Peekable<I>, start: usize) -> Token
where
    I: Iterator<Item = (usize, char)>,
{
    let mut text = String::from('#');
    while let Some((_, c)) = chars.peek() {
        if *c == '\n' {
            break;
        }
        text.push(*c);
        chars.next();
    }
    let len = text.len();
    Token::new(TokenType::Comment, text, start, start + len)
}

/// Tokenize a string (single or double quoted)
fn tokenize_string<I>(chars: &mut std::iter::Peekable<I>, quote: char, start: usize) -> Token
where
    I: Iterator<Item = (usize, char)>,
{
    let mut text = String::from(quote);
    let mut escaped = false;

    for (_, c) in chars.by_ref() {
        text.push(c);
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == quote {
            break;
        }
    }

    let len = text.len();
    Token::new(TokenType::String, text, start, start + len)
}

/// Tokenize a variable ($VAR, ${VAR}, $?, etc.)
fn tokenize_variable<I>(chars: &mut std::iter::Peekable<I>, start: usize) -> Token
where
    I: Iterator<Item = (usize, char)>,
{
    let mut text = String::from('$');

    // Check for ${var} or $var
    if let Some((_, '{')) = chars.peek() {
        text.push('{');
        chars.next();
        for (_, c) in chars.by_ref() {
            text.push(c);
            if c == '}' {
                break;
            }
        }
    } else {
        // Simple $var or $? $$ etc
        while let Some((_, c)) = chars.peek() {
            if c.is_alphanumeric() || *c == '_' || *c == '?' || *c == '!' || *c == '@' || *c == '#'
            {
                text.push(*c);
                chars.next();
            } else {
                break;
            }
        }
    }

    let len = text.len();
    Token::new(TokenType::Variable, text, start, start + len)
}

/// Check if two characters form a double operator (&&, ||, >>, <<)
fn is_double_operator(ch: char, next_ch: char) -> bool {
    (ch == '|' && next_ch == '|')
        || (ch == '&' && next_ch == '&')
        || (ch == '>' && next_ch == '>')
        || (ch == '<' && next_ch == '<')
}

/// Handle redirection with file descriptor (2>&1, etc.)
fn handle_redirection<I>(chars: &mut std::iter::Peekable<I>, text: &mut String)
where
    I: Iterator<Item = (usize, char)>,
{
    while let Some((_, c)) = chars.peek() {
        if c.is_numeric() {
            text.insert(0, *c);
            chars.next();
        } else {
            break;
        }
    }
}

/// Tokenize an operator (|, &, ;, <, >, &&, ||, >>, etc.)
fn tokenize_operator<I>(chars: &mut std::iter::Peekable<I>, ch: char, start: usize) -> Token
where
    I: Iterator<Item = (usize, char)>,
{
    let mut text = String::from(ch);

    if let Some((_, next_ch)) = chars.peek() {
        if is_double_operator(ch, *next_ch) {
            text.push(*next_ch);
            chars.next();
        } else if ch == '>' && *next_ch == '&' {
            text.push(*next_ch);
            chars.next();
            handle_redirection(chars, &mut text);
        }
    }

    let len = text.len();
    Token::new(TokenType::Operator, text, start, start + len)
}

/// Tokenize whitespace
fn tokenize_whitespace<I>(chars: &mut std::iter::Peekable<I>, ch: char, start: usize) -> Token
where
    I: Iterator<Item = (usize, char)>,
{
    let mut text = String::from(ch);
    while let Some((_, c)) = chars.peek() {
        if c.is_whitespace() {
            text.push(*c);
            chars.next();
        } else {
            break;
        }
    }
    let len = text.len();
    Token::new(TokenType::Whitespace, text, start, start + len)
}

/// Tokenize a word (command, keyword, or text)
fn tokenize_word<I>(
    chars: &mut std::iter::Peekable<I>,
    ch: char,
    start: usize,
    is_first: bool,
) -> Token
where
    I: Iterator<Item = (usize, char)>,
{
    let mut text = String::from(ch);
    while let Some((_, c)) = chars.peek() {
        if c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '/' || *c == '.' {
            text.push(*c);
            chars.next();
        } else {
            break;
        }
    }

    // Determine token type
    let token_type = if is_keyword(&text) {
        TokenType::Keyword
    } else if is_first {
        TokenType::Command
    } else {
        TokenType::Text
    };

    let len = text.len();
    Token::new(token_type, text, start, start + len)
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().enumerate().peekable();
    let mut is_first_word = true;

    while let Some((i, ch)) = chars.next() {
        let start = i;

        let token = match ch {
            '#' => tokenize_comment(&mut chars, start),
            '"' | '\'' => tokenize_string(&mut chars, ch, start),
            '$' => tokenize_variable(&mut chars, start),
            '|' | '&' | ';' | '<' | '>' => {
                let tok = tokenize_operator(&mut chars, ch, start);
                is_first_word = true; // Reset after operator
                tok
            }
            c if c.is_whitespace() => tokenize_whitespace(&mut chars, ch, start),
            c if c.is_alphanumeric() || c == '_' || c == '-' || c == '/' || c == '.' => {
                let tok = tokenize_word(&mut chars, ch, start, is_first_word);
                if !is_keyword(&tok.text) && is_first_word {
                    is_first_word = false;
                }
                tok
            }
            // Everything else is text
            _ => Token::new(TokenType::Text, String::from(ch), start, start + 1),
        };

        tokens.push(token);
    }

    tokens
}

/// Highlight a single token with ANSI color codes
///
/// Wraps the token text with appropriate ANSI escape sequences based on
/// the token type.
///
/// # Arguments
///
/// * `token` - The token to highlight
///
/// # Returns
///
/// The token text wrapped in ANSI color codes
pub fn highlight_token(token: &Token) -> String {
    use colors::*;

    match token.token_type {
        TokenType::Keyword => format!("{}{}{}", BOLD_BLUE, token.text, RESET),
        TokenType::String => format!("{}{}{}", GREEN, token.text, RESET),
        TokenType::Variable => format!("{}{}{}", YELLOW, token.text, RESET),
        TokenType::Command => format!("{}{}{}", CYAN, token.text, RESET),
        TokenType::Comment => format!("{}{}{}", GRAY, token.text, RESET),
        TokenType::Operator => format!("{}{}{}", MAGENTA, token.text, RESET),
        TokenType::Whitespace | TokenType::Text => token.text.clone(),
    }
}

/// Apply syntax highlighting to bash input
///
/// Tokenizes the input and applies ANSI color codes to each token based on
/// its type (keywords, strings, variables, commands, etc.).
///
/// # Arguments
///
/// * `input` - The bash code to highlight
///
/// # Returns
///
/// The highlighted bash code with ANSI color codes
///
/// # Examples
///
/// ```
/// use bashrs::repl::highlighting::highlight_bash;
///
/// let highlighted = highlight_bash("echo $HOME");
/// // Output contains ANSI codes for syntax highlighting
/// ```
pub fn highlight_bash(input: &str) -> String {
    let tokens = tokenize(input);
    tokens.iter().map(highlight_token).collect()
}

/// Strip ANSI escape codes from a string
///
/// Removes all ANSI color codes from the input, returning plain text.
/// Useful for testing that highlighting preserves the original text.
///
/// # Arguments
///
/// * `input` - The string with ANSI codes
///
/// # Returns
///
/// The string with all ANSI codes removed
pub fn strip_ansi_codes(input: &str) -> String {
    // Simple regex-free approach: skip escape sequences
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Skip until 'm' (end of ANSI sequence)
            for c in chars.by_ref() {
                if c == 'm' {
                    break;
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== UNIT TESTS (RED PHASE) =====

    /// Test: REPL-015-002-001 - Highlight keywords
    #[test]
    fn test_REPL_015_002_001_highlight_keywords() {
        let input = "if then else fi";
        let highlighted = highlight_bash(input);

        assert!(highlighted.contains("\x1b[1;34mif\x1b[0m"));
        assert!(highlighted.contains("\x1b[1;34mthen\x1b[0m"));
        assert!(highlighted.contains("\x1b[1;34melse\x1b[0m"));
        assert!(highlighted.contains("\x1b[1;34mfi\x1b[0m"));
    }

    /// Test: REPL-015-002-002 - Highlight strings
    #[test]
    fn test_REPL_015_002_002_highlight_strings() {
        let input = r#"echo "hello world" 'single'"#;
        let highlighted = highlight_bash(input);

        assert!(highlighted.contains("\x1b[32m\"hello world\"\x1b[0m"));
        assert!(highlighted.contains("\x1b[32m'single'\x1b[0m"));
    }

    /// Test: REPL-015-002-003 - Highlight variables
    #[test]
    fn test_REPL_015_002_003_highlight_variables() {
        let input = "echo $HOME ${USER} $?";
        let highlighted = highlight_bash(input);

        assert!(highlighted.contains("\x1b[33m$HOME\x1b[0m"));
        assert!(highlighted.contains("\x1b[33m${USER}\x1b[0m"));
        assert!(highlighted.contains("\x1b[33m$?\x1b[0m"));
    }

    /// Test: REPL-015-002-004 - Highlight commands
    #[test]
    fn test_REPL_015_002_004_highlight_commands() {
        let input = "mkdir -p /tmp";
        let highlighted = highlight_bash(input);

        // First word should be highlighted as command
        assert!(highlighted.contains("\x1b[36mmkdir\x1b[0m"));
    }

    /// Test: REPL-015-002-005 - Highlight comments
    #[test]
    fn test_REPL_015_002_005_highlight_comments() {
        let input = "echo hello # this is a comment";
        let highlighted = highlight_bash(input);

        assert!(highlighted.contains("\x1b[90m# this is a comment\x1b[0m"));
    }

    /// Test: REPL-015-002-006 - Highlight operators
    #[test]
    fn test_REPL_015_002_006_highlight_operators() {
        let input = "cat file | grep pattern && echo done";
        let highlighted = highlight_bash(input);

        assert!(highlighted.contains("\x1b[35m|\x1b[0m"));
        assert!(highlighted.contains("\x1b[35m&&\x1b[0m"));
    }

    /// Test: REPL-015-002-007 - is_keyword function
    #[test]
    fn test_REPL_015_002_007_is_keyword() {
        assert!(is_keyword("if"));
        assert!(is_keyword("then"));
        assert!(is_keyword("while"));
        assert!(is_keyword("for"));
        assert!(is_keyword("do"));
        assert!(is_keyword("done"));

        assert!(!is_keyword("echo"));
        assert!(!is_keyword("test"));
        assert!(!is_keyword("hello"));
    }


        include!("highlighting_part2_incl2.rs");
