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
            if c.is_alphanumeric() || *c == '_' || *c == '?' || *c == '!' || *c == '@' || *c == '#' {
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
fn tokenize_word<I>(chars: &mut std::iter::Peekable<I>, ch: char, start: usize, is_first: bool) -> Token
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

    /// Property: Tokenization preserves all input characters
    #[test]
    fn prop_tokenize_preserves_chars() {
        let test_inputs = vec![
            "echo hello",
            "if then else fi",
            "# comment",
            "echo \"string\"",
            "echo $VAR",
            "cat file | grep pattern",
            "cmd1 && cmd2",
            "cmd1 || cmd2",
            "echo test > file",
            "cat < file",
            "cmd1; cmd2",
            "function foo { echo bar; }",
            "${VAR:-default}",
            "echo 'single quotes'",
            "ls -la /tmp",
            "echo test # comment",
            "#!/bin/bash",
            "",
        ];

        for input in test_inputs {
            let tokens = tokenize(input);
            let reconstructed: String = tokens.iter().map(|t| t.text.as_str()).collect();
            assert_eq!(
                reconstructed, input,
                "Tokenization must preserve all characters for input: {:?}",
                input
            );
        }
    }

    /// Property: Token positions are contiguous and cover entire input
    #[test]
    fn prop_tokenize_positions_contiguous() {
        let test_inputs = vec![
            "echo hello world",
            "if [ -f file ]; then echo yes; fi",
            "cat file | grep pattern | wc -l",
        ];

        for input in test_inputs {
            let tokens = tokenize(input);
            let mut expected_start = 0;

            for token in &tokens {
                assert_eq!(
                    token.start, expected_start,
                    "Token positions must be contiguous at {:?} in input {:?}",
                    token, input
                );
                expected_start = token.end;
            }

            assert_eq!(
                expected_start,
                input.len(),
                "Tokens must cover entire input for {:?}",
                input
            );
        }
    }

    /// Property: Tokenization is deterministic
    #[test]
    fn prop_tokenize_deterministic() {
        let test_inputs = vec![
            "echo hello",
            "if then fi",
            "# comment",
            "echo $VAR",
        ];

        for input in test_inputs {
            let tokens1 = tokenize(input);
            let tokens2 = tokenize(input);

            assert_eq!(tokens1.len(), tokens2.len());
            for (t1, t2) in tokens1.iter().zip(tokens2.iter()) {
                assert_eq!(t1.token_type, t2.token_type);
                assert_eq!(t1.text, t2.text);
                assert_eq!(t1.start, t2.start);
                assert_eq!(t1.end, t2.end);
            }
        }
    }

    /// Property: Comments always start with # and consume to end of line
    #[test]
    fn prop_tokenize_comments() {
        let test_inputs = vec![
            ("# comment", vec![TokenType::Comment]),
            ("echo test # comment", vec![TokenType::Command, TokenType::Whitespace, TokenType::Text, TokenType::Whitespace, TokenType::Comment]),
            ("# only comment", vec![TokenType::Comment]),
        ];

        for (input, expected_types) in test_inputs {
            let tokens = tokenize(input);
            let types: Vec<TokenType> = tokens.iter().map(|t| t.token_type).collect();
            assert_eq!(types, expected_types, "Failed for input: {:?}", input);

            // Verify comment token starts with #
            for token in &tokens {
                if token.token_type == TokenType::Comment {
                    assert!(token.text.starts_with('#'));
                }
            }
        }
    }

    /// Property: String tokens always contain matching quotes
    #[test]
    fn prop_tokenize_strings() {
        let test_inputs = vec![
            r#"echo "hello""#,
            r#"echo 'world'"#,
            r#"echo "escaped \" quote""#,
            r#"echo 'single'"#,
        ];

        for input in test_inputs {
            let tokens = tokenize(input);

            for token in &tokens {
                if token.token_type == TokenType::String {
                    // String must start and end with same quote type
                    let first = token.text.chars().next().unwrap();
                    let last = token.text.chars().last().unwrap();
                    assert!(first == '"' || first == '\'');
                    // Note: may not end with quote if unclosed string
                    if token.text.len() > 1 && !token.text.ends_with('\\') {
                        // If closed, should have matching quote
                        if token.text.len() > 1 && (last == '"' || last == '\'') {
                            assert_eq!(first, last, "String quotes must match in {:?}", token.text);
                        }
                    }
                }
            }
        }
    }

    /// Property: Variable tokens always start with $
    #[test]
    fn prop_tokenize_variables() {
        let test_inputs = vec![
            "echo $VAR",
            "echo ${VAR}",
            "echo $HOME",
            "echo $$",
            "echo $?",
            "echo $!",
            "echo $@",
            "echo $#",
        ];

        for input in test_inputs {
            let tokens = tokenize(input);

            for token in &tokens {
                if token.token_type == TokenType::Variable {
                    assert!(token.text.starts_with('$'), "Variable must start with $ in {:?}", token.text);
                }
            }
        }
    }

    /// Property: Operators are recognized correctly
    #[test]
    fn prop_tokenize_operators() {
        let test_inputs = vec![
            ("cmd1 | cmd2", TokenType::Operator),
            ("cmd1 && cmd2", TokenType::Operator),
            ("cmd1 || cmd2", TokenType::Operator),
            ("cmd1 ; cmd2", TokenType::Operator),
            ("echo > file", TokenType::Operator),
            ("cat < file", TokenType::Operator),
            ("echo >> file", TokenType::Operator),
        ];

        for (input, expected_operator_type) in test_inputs {
            let tokens = tokenize(input);
            let has_operator = tokens.iter().any(|t| t.token_type == expected_operator_type);
            assert!(has_operator, "Expected operator in: {:?}", input);
        }
    }

    /// Property: Keywords are recognized correctly
    #[test]
    fn prop_tokenize_keywords() {
        let keywords = vec!["if", "then", "else", "elif", "fi", "for", "while", "do", "done", "case", "esac", "function"];

        for keyword in keywords {
            let tokens = tokenize(keyword);
            assert_eq!(tokens.len(), 1);
            assert_eq!(tokens[0].token_type, TokenType::Keyword);
            assert_eq!(tokens[0].text, keyword);
        }
    }

    /// Property: First word is command (unless keyword)
    #[test]
    fn prop_tokenize_first_word_command() {
        let test_inputs = vec![
            ("echo hello", TokenType::Command, "echo"),
            ("ls -la", TokenType::Command, "ls"),
            ("mkdir /tmp", TokenType::Command, "mkdir"),
        ];

        for (input, expected_type, expected_text) in test_inputs {
            let tokens = tokenize(input);
            let first_word = tokens.iter().find(|t| !matches!(t.token_type, TokenType::Whitespace));

            assert!(first_word.is_some());
            let first_word = first_word.unwrap();
            assert_eq!(first_word.token_type, expected_type);
            assert_eq!(first_word.text, expected_text);
        }
    }
}

#[cfg(test)]
mod proptest_generative {
    use super::*;
    use proptest::prelude::*;

    // Strategy for bash commands
    fn bash_command() -> impl Strategy<Value = String> {
        prop::sample::select(vec![
            "echo", "ls", "cat", "grep", "sed", "awk", "mkdir", "rm", "cp", "mv",
            "find", "chmod", "chown", "pwd", "cd", "pushd", "popd", "test",
        ])
        .prop_map(|s| s.to_string())
    }

    // Strategy for bash keywords
    fn bash_keyword() -> impl Strategy<Value = String> {
        prop::sample::select(vec![
            "if", "then", "else", "elif", "fi", "for", "while", "do", "done",
            "case", "esac", "function",
        ])
        .prop_map(|s| s.to_string())
    }

    // Strategy for variable names
    fn var_name() -> impl Strategy<Value = String> {
        "[A-Z_][A-Z0-9_]{0,10}".prop_map(|s| s.to_string())
    }

    // Strategy for simple strings (no quotes inside)
    fn simple_string() -> impl Strategy<Value = String> {
        "[a-z0-9/.-]{1,20}".prop_map(|s| s.to_string())
    }

    // Strategy for operators
    fn operator() -> impl Strategy<Value = String> {
        prop::sample::select(vec!["|", "&&", "||", ";", ">", "<", ">>", "<<", "2>&1"])
            .prop_map(|s| s.to_string())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: Tokenization always preserves input length
        #[test]
        fn prop_gen_tokenize_preserves_length(cmd in bash_command()) {
            let tokens = tokenize(&cmd);
            let reconstructed: String = tokens.iter().map(|t| t.text.as_str()).collect();
            prop_assert_eq!(reconstructed.len(), cmd.len());
        }

        /// Property: Tokenization never panics on any command
        #[test]
        fn prop_gen_tokenize_never_panics(cmd in bash_command(), arg in simple_string()) {
            let input = format!("{} {}", cmd, arg);
            let _ = tokenize(&input);
            // If we get here, no panic occurred
            prop_assert!(true);
        }

        /// Property: Commands are always recognized as Command type
        #[test]
        fn prop_gen_commands_recognized(cmd in bash_command()) {
            let tokens = tokenize(&cmd);
            prop_assert!(!tokens.is_empty());
            prop_assert_eq!(tokens[0].token_type, TokenType::Command);
            prop_assert_eq!(&tokens[0].text, &cmd);
        }

        /// Property: Keywords are always recognized as Keyword type
        #[test]
        fn prop_gen_keywords_recognized(kw in bash_keyword()) {
            let tokens = tokenize(&kw);
            prop_assert!(!tokens.is_empty());
            prop_assert_eq!(tokens[0].token_type, TokenType::Keyword);
            prop_assert_eq!(&tokens[0].text, &kw);
        }

        /// Property: Variables always start with $
        #[test]
        fn prop_gen_variables_start_dollar(var in var_name()) {
            let input = format!("${}", var);
            let tokens = tokenize(&input);
            let var_token = tokens.iter().find(|t| t.token_type == TokenType::Variable);
            prop_assert!(var_token.is_some());
            prop_assert!(var_token.unwrap().text.starts_with('$'));
        }

        /// Property: Braced variables are complete
        #[test]
        fn prop_gen_braced_variables(var in var_name()) {
            let input = format!("${{{}}}", var);  // Creates ${VAR}
            let tokens = tokenize(&input);
            let var_token = tokens.iter().find(|t| t.token_type == TokenType::Variable);
            prop_assert!(var_token.is_some());
            let text = &var_token.unwrap().text;
            // Check format: must start with $, contain var name, and be braced
            prop_assert!(text.len() >= 3);  // At least ${X}
            prop_assert_eq!(text.chars().next(), Some('$'));
            let has_braces = text.contains(|c| c == '{') && text.contains(|c| c == '}');
            prop_assert!(has_braces);
        }

        /// Property: Operators are recognized
        #[test]
        fn prop_gen_operators_recognized(op in operator()) {
            let input = format!("cmd1 {} cmd2", op);
            let tokens = tokenize(&input);
            let op_token = tokens.iter().find(|t| t.token_type == TokenType::Operator);
            prop_assert!(op_token.is_some());
        }

        /// Property: Double quoted strings contain quotes
        #[test]
        fn prop_gen_double_quoted_strings(s in simple_string()) {
            let input = format!("\"{}\"", s);
            let tokens = tokenize(&input);
            let str_token = tokens.iter().find(|t| t.token_type == TokenType::String);
            prop_assert!(str_token.is_some());
            let text = &str_token.unwrap().text;
            prop_assert!(text.starts_with('"'));
            prop_assert!(text.ends_with('"'));
        }

        /// Property: Single quoted strings contain quotes
        #[test]
        fn prop_gen_single_quoted_strings(s in simple_string()) {
            let input = format!("'{}'", s);
            let tokens = tokenize(&input);
            let str_token = tokens.iter().find(|t| t.token_type == TokenType::String);
            prop_assert!(str_token.is_some());
            let text = &str_token.unwrap().text;
            prop_assert!(text.starts_with('\''));
            prop_assert!(text.ends_with('\''));
        }

        /// Property: Comments always start with #
        #[test]
        fn prop_gen_comments_start_hash(s in simple_string()) {
            let input = format!("# {}", s);
            let tokens = tokenize(&input);
            prop_assert!(!tokens.is_empty());
            prop_assert_eq!(tokens[0].token_type, TokenType::Comment);
            prop_assert!(tokens[0].text.starts_with('#'));
        }

        /// Property: Simple command with arg preserves text
        #[test]
        fn prop_gen_command_with_arg(cmd in bash_command(), arg in simple_string()) {
            let input = format!("{} {}", cmd, arg);
            let tokens = tokenize(&input);
            let reconstructed: String = tokens.iter().map(|t| t.text.as_str()).collect();
            prop_assert_eq!(reconstructed, input);
        }

        /// Property: Pipeline preserves structure
        #[test]
        fn prop_gen_pipeline(cmd1 in bash_command(), cmd2 in bash_command()) {
            let input = format!("{} | {}", cmd1, cmd2);
            let tokens = tokenize(&input);
            let has_pipe = tokens.iter().any(|t| t.token_type == TokenType::Operator && t.text == "|");
            prop_assert!(has_pipe);
        }

        /// Property: Redirection preserves operator
        #[test]
        fn prop_gen_redirection(cmd in bash_command(), file in simple_string()) {
            let input = format!("{} > {}", cmd, file);
            let tokens = tokenize(&input);
            let has_redirect = tokens.iter().any(|t| t.token_type == TokenType::Operator && t.text == ">");
            prop_assert!(has_redirect);
        }

        /// Property: Command sequence with semicolon
        #[test]
        fn prop_gen_semicolon_sequence(cmd1 in bash_command(), cmd2 in bash_command()) {
            let input = format!("{}; {}", cmd1, cmd2);
            let tokens = tokenize(&input);
            let has_semi = tokens.iter().any(|t| t.token_type == TokenType::Operator && t.text == ";");
            prop_assert!(has_semi);
        }

        /// Property: AND operator recognized
        #[test]
        fn prop_gen_and_operator(cmd1 in bash_command(), cmd2 in bash_command()) {
            let input = format!("{} && {}", cmd1, cmd2);
            let tokens = tokenize(&input);
            let has_and = tokens.iter().any(|t| t.token_type == TokenType::Operator && t.text == "&&");
            prop_assert!(has_and);
        }

        /// Property: OR operator recognized
        #[test]
        fn prop_gen_or_operator(cmd1 in bash_command(), cmd2 in bash_command()) {
            let input = format!("{} || {}", cmd1, cmd2);
            let tokens = tokenize(&input);
            let has_or = tokens.iter().any(|t| t.token_type == TokenType::Operator && t.text == "||");
            prop_assert!(has_or);
        }

        /// Property: Token positions never overlap
        #[test]
        fn prop_gen_positions_no_overlap(cmd in bash_command(), arg in simple_string()) {
            let input = format!("{} {}", cmd, arg);
            let tokens = tokenize(&input);
            for i in 0..tokens.len().saturating_sub(1) {
                prop_assert_eq!(tokens[i].end, tokens[i + 1].start);
            }
        }

        /// Property: Token positions cover entire input
        #[test]
        fn prop_gen_positions_complete(cmd in bash_command()) {
            let tokens = tokenize(&cmd);
            if !tokens.is_empty() {
                prop_assert_eq!(tokens[0].start, 0);
                prop_assert_eq!(tokens.last().unwrap().end, cmd.len());
            }
        }

        /// Property: Tokenization is idempotent
        #[test]
        fn prop_gen_tokenize_idempotent(cmd in bash_command(), arg in simple_string()) {
            let input = format!("{} {}", cmd, arg);
            let tokens1 = tokenize(&input);
            let tokens2 = tokenize(&input);
            prop_assert_eq!(tokens1.len(), tokens2.len());
            for (t1, t2) in tokens1.iter().zip(tokens2.iter()) {
                prop_assert_eq!(t1.token_type, t2.token_type);
                prop_assert_eq!(&t1.text, &t2.text);
                prop_assert_eq!(t1.start, t2.start);
                prop_assert_eq!(t1.end, t2.end);
            }
        }

        /// Property: Empty input produces empty tokens
        #[test]
        fn prop_gen_empty_input_empty_tokens(_x in 0u8..10u8) {
            let tokens = tokenize("");
            prop_assert!(tokens.is_empty());
        }

        /// Property: Whitespace-only input produces only whitespace tokens
        #[test]
        fn prop_gen_whitespace_only(n in 1usize..20usize) {
            let input = " ".repeat(n);
            let tokens = tokenize(&input);
            for token in &tokens {
                prop_assert_eq!(token.token_type, TokenType::Whitespace);
            }
        }
    }
}

