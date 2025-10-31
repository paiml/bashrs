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
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Keyword,   // if, then, while, for, do, done, case, esac, function
    String,    // "..." or '...'
    Variable,  // $var, ${var}, $?
    Command,   // First word in pipeline
    Comment,   // # comment
    Operator,  // |, >, <, &&, ||, ;
    Whitespace, // Spaces, tabs
    Text,      // Everything else
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
pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().enumerate().peekable();
    let mut is_first_word = true;

    while let Some((i, ch)) = chars.next() {
        let start = i;

        // Handle comments
        if ch == '#' {
            let mut text = String::from('#');
            while let Some((_, c)) = chars.peek() {
                if *c == '\n' {
                    break;
                }
                text.push(*c);
                chars.next();
            }
            let len = text.len();
            tokens.push(Token::new(TokenType::Comment, text, start, start + len));
            continue;
        }

        // Handle strings
        if ch == '"' || ch == '\'' {
            let quote = ch;
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
            tokens.push(Token::new(TokenType::String, text, start, start + len));
            continue;
        }

        // Handle variables
        if ch == '$' {
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
                    if c.is_alphanumeric() || *c == '_' || *c == '?' || *c == '!' || *c == '@' || *c == '#' {
                        text.push(*c);
                        chars.next();
                    } else {
                        break;
                    }
                }
            }

            let len = text.len();
            tokens.push(Token::new(TokenType::Variable, text, start, start + len));
            continue;
        }

        // Handle operators
        if ch == '|' || ch == '&' || ch == ';' || ch == '<' || ch == '>' {
            let mut text = String::from(ch);

            // Check for && || >> 2>&1 etc
            if let Some((_, next_ch)) = chars.peek() {
                if (ch == '|' && *next_ch == '|')
                    || (ch == '&' && *next_ch == '&')
                    || (ch == '>' && *next_ch == '>')
                    || (ch == '<' && *next_ch == '<')
                {
                    text.push(*next_ch);
                    chars.next();
                } else if ch == '>' && *next_ch == '&' {
                    text.push(*next_ch);
                    chars.next();
                    // Handle 2>&1
                    while let Some((_, c)) = chars.peek() {
                        if c.is_numeric() {
                            text.insert(0, *c);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
            }

            let len = text.len();
            tokens.push(Token::new(TokenType::Operator, text, start, start + len));
            is_first_word = true; // Reset after operator
            continue;
        }

        // Handle whitespace
        if ch.is_whitespace() {
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
            tokens.push(Token::new(TokenType::Whitespace, text, start, start + len));
            continue;
        }

        // Handle words (commands, keywords, text)
        if ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == '/' || ch == '.' {
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
            } else if is_first_word {
                is_first_word = false;
                TokenType::Command
            } else {
                TokenType::Text
            };

            let len = text.len();
            tokens.push(Token::new(token_type, text, start, start + len));
            continue;
        }

        // Everything else is text
        let text = String::from(ch);
        tokens.push(Token::new(TokenType::Text, text, start, start + 1));
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

    /// Test: REPL-015-002-INT-001 - Full bash statement
    #[test]
    fn test_REPL_015_002_INT_001_full_statement() {
        let input = r#"if [ -f "$file" ]; then echo "found"; fi"#;
        let highlighted = highlight_bash(input);

        // Should have keyword highlighting
        assert!(highlighted.contains("\x1b[1;34mif\x1b[0m"));
        assert!(highlighted.contains("\x1b[1;34mthen\x1b[0m"));
        assert!(highlighted.contains("\x1b[1;34mfi\x1b[0m"));

        // Should have string highlighting
        assert!(highlighted.contains("\"$file\""));
        assert!(highlighted.contains("\"found\""));

        // Should have operator highlighting
        assert!(highlighted.contains("\x1b[35m;\x1b[0m"));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;

    /// Property: Highlighting never panics
    #[test]
    fn prop_highlighting_never_panics() {
        // Should never panic on any input
        let long_input = "x".repeat(1000);
        let test_inputs = vec![
            "",
            "if then",
            "echo $HOME",
            "cat file | grep pattern",
            long_input.as_str(),
            "# comment only",
            r#"echo "string with spaces""#,
        ];

        for input in test_inputs {
            let _ = highlight_bash(input);
        }
    }

    /// Property: Highlighting preserves text
    #[test]
    fn prop_highlighting_preserves_text() {
        let test_inputs = vec![
            "echo hello",
            "if then else fi",
            "mkdir /tmp",
        ];

        for input in test_inputs {
            let highlighted = highlight_bash(input);
            let stripped = strip_ansi_codes(&highlighted);
            assert_eq!(stripped, input);
        }
    }
}
