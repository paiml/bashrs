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
        // Contract: parser-soundness-v1.yaml precondition (pv codegen)
        contract_pre_lex!(self.input);
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

    /// Tokenize with character positions for each token.
    /// Returns (tokens, positions) where positions[i] is the byte offset of tokens[i].
    pub fn tokenize_with_positions(&mut self) -> Result<(Vec<Token>, Vec<usize>), LexerError> {
        let mut tokens = Vec::new();
        let mut positions = Vec::new();

        loop {
            self.skip_whitespace_except_newline();

            if self.is_at_end() {
                positions.push(self.position);
                tokens.push(Token::Eof);
                break;
            }

            let pos = self.position;
            let token = self.next_token()?;
            positions.push(pos);
            tokens.push(token.clone());

            if token == Token::Eof {
                break;
            }
        }

        Ok((tokens, positions))
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
        // These are unquoted strings that can contain /  * . - : + % \ , = etc
        // Note: ':' is included for bash builtin no-op command (BUILTIN-001)
        // Note: '+' and '%' are included for flags like date +%FORMAT (PARSER-ENH-001)
        // Note: '\\' is included for escaped chars like \\; in find -exec
        // Issue #131: ',' is included for Docker mount options like type=bind,source=...,target=...
        // BUG-012 FIX: Don't treat '+=' as bare word - it's the append operator
        let is_append_op = ch == '+' && self.peek_char(1) == Some('=');
        if !is_append_op
            && (ch == '/'
                || ch == '.'
                || ch == '-'
                || ch == '*'
                || ch == '~'
                || ch == ':'
                || ch == '+'
                || ch == '%'
                || ch == '\\'
                || ch == ',')
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
            } else if ch == '\\' && self.peek_char(1) == Some('\n') {
                // Backslash-newline is line continuation — skip both characters
                // and continue reading the next line as part of the current command
                self.advance(); // skip backslash
                self.advance(); // skip newline
                self.line += 1;
                self.column = 1;
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

        // Handle $'...' ANSI-C quoting: $'\t' $'\n' etc.
        if !self.is_at_end() && self.current_char() == '\'' {
            return Ok(self.read_ansi_c_string());
        }

        // Check for arithmetic expansion $((...)) vs command substitution $(cmd)
        if !self.is_at_end() && self.current_char() == '(' {
            if let Some('(') = self.peek_char(1) {
                return self.read_arithmetic_expansion();
            } else {
                return self.read_command_substitution();
            }
        }

        // Check for $$ (process ID special variable)
        if !self.is_at_end() && self.current_char() == '$' {
            self.advance();
            return Ok(Token::Variable("$".to_string()));
        }

        // Check for $@ (all positional parameters special variable)
        if !self.is_at_end() && self.current_char() == '@' {
            self.advance();
            return Ok(Token::Variable("@".to_string()));
        }

        // Handle shell special variables: $#, $?, $!, $-
        if !self.is_at_end() && matches!(self.current_char(), '#' | '?' | '!' | '-') {
            let special = self.advance();
            return Ok(Token::Variable(special.to_string()));
        }

        // Handle ${VAR} syntax (with nested expansion support)
        // BUG-001 FIX: Handle nested parameter expansion like ${foo:-${bar:-default}}
        let var_name = if !self.is_at_end() && self.current_char() == '{' {
            self.read_braced_variable()
        } else {
            self.read_simple_variable_name()
        };

        Ok(Token::Variable(var_name))
    }

    /// Read ANSI-C quoted string: $'\t' $'\n' etc.
    fn read_ansi_c_string(&mut self) -> Token {
        self.advance(); // skip opening '
        let mut value = String::new();
        while !self.is_at_end() && self.current_char() != '\'' {
            if self.current_char() == '\\' {
                self.advance(); // skip backslash
                if !self.is_at_end() {
                    let escaped = self.decode_ansi_c_escape();
                    value.push_str(&escaped);
                    self.advance();
                }
            } else {
                value.push(self.advance());
            }
        }
        if !self.is_at_end() {
            self.advance(); // skip closing '
        }
        Token::String(value)
    }

    /// Decode a single ANSI-C escape character at the current position.
    /// Returns the replacement string (usually one char, two for unknown escapes).
    fn decode_ansi_c_escape(&self) -> String {
        match self.current_char() {
            'n' => "\n".to_string(),
            't' => "\t".to_string(),
            'r' => "\r".to_string(),
            'a' => "\x07".to_string(),
            'b' => "\x08".to_string(),
            'e' | 'E' => "\x1b".to_string(),
            'f' => "\x0c".to_string(),
            'v' => "\x0b".to_string(),
            '\\' => "\\".to_string(),
            '\'' => "'".to_string(),
            '"' => "\"".to_string(),
            other => format!("\\{}", other),
        }
    }

    /// Read a braced variable expansion: ${VAR}, ${foo:-default}, ${foo:-${bar:-x}}
    fn read_braced_variable(&mut self) -> String {
        self.advance(); // skip '{'
        let mut var_name = String::new();
        let mut brace_depth = 1;
        while !self.is_at_end() && brace_depth > 0 {
            let ch = self.current_char();
            if ch == '{' {
                brace_depth += 1;
                var_name.push(self.advance());
            } else if ch == '}' {
                brace_depth -= 1;
                if brace_depth > 0 {
                    var_name.push(self.advance());
                } else {
                    self.advance(); // skip final '}'
                }
            } else if ch == '$' {
                var_name.push(self.advance());
                if !self.is_at_end() && self.current_char() == '{' {
                    brace_depth += 1;
                    var_name.push(self.advance());
                }
            } else {
                var_name.push(self.advance());
            }
        }
        var_name
    }

    /// Read a simple (unbraced) variable name: alphanumeric and underscore chars.
    fn read_simple_variable_name(&mut self) -> String {
        let mut var_name = String::new();
        while !self.is_at_end() {
            let ch = self.current_char();
            if ch.is_alphanumeric() || ch == '_' {
                var_name.push(self.advance());
            } else {
                break;
            }
        }
        var_name
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

    fn read_heredoc(&mut self) -> Result<Token, LexerError> {
        let delimiter = self.read_heredoc_delimiter()?;
        self.skip_to_next_line();

        // Read heredoc content until we find a line matching the delimiter
        let content = self.read_heredoc_content(&delimiter, false);

        Ok(Token::Heredoc { delimiter, content })
    }

    /// BUG-007 FIX: Read indented heredoc (<<-DELIMITER)
    /// In indented heredocs, leading tabs are stripped from content lines
    /// and the delimiter can be indented with tabs
    fn read_heredoc_indented(&mut self) -> Result<Token, LexerError> {
        let delimiter = self.read_heredoc_delimiter()?;
        self.skip_to_next_line();

        // Read heredoc content - strip leading tabs
        let content = self.read_heredoc_content(&delimiter, true);

        Ok(Token::Heredoc { delimiter, content })
    }

    /// Read a heredoc delimiter, handling optional quoting (<<'EOF' or <<"EOF").
    /// BUG-006 FIX: Handle quoted delimiters.
    fn read_heredoc_delimiter(&mut self) -> Result<String, LexerError> {
        // Skip any leading whitespace
        while !self.is_at_end() && (self.current_char() == ' ' || self.current_char() == '\t') {
            self.advance();
        }

        // Check for quoted delimiter
        let quote_char =
            if !self.is_at_end() && (self.current_char() == '\'' || self.current_char() == '"') {
                let q = self.current_char();
                self.advance(); // skip opening quote
                Some(q)
            } else {
                None
            };

        // Read delimiter characters
        let mut delimiter = String::new();
        while !self.is_at_end() {
            let ch = self.current_char();
            if let Some(q) = quote_char {
                if ch == q {
                    self.advance(); // skip closing quote
                    break;
                }
                delimiter.push(self.advance());
            } else if ch.is_alphanumeric() || ch == '_' {
                delimiter.push(self.advance());
            } else {
                break;
            }
        }

        if delimiter.is_empty() {
            let ch = if self.is_at_end() {
                '\0'
            } else {
                self.current_char()
            };
            return Err(LexerError::UnexpectedChar(ch, self.line, self.column));
        }

        Ok(delimiter)
    }

    /// Skip to the end of the current line and consume the newline character.
    fn skip_to_next_line(&mut self) {
        while !self.is_at_end() && self.current_char() != '\n' {
            self.advance();
        }
        if !self.is_at_end() {
            self.advance(); // skip newline
        }
    }

    /// Read heredoc content lines until a line matches the delimiter.
    /// If `strip_tabs` is true, leading tabs are stripped from each line (<<- mode).
    fn read_heredoc_content(&mut self, delimiter: &str, strip_tabs: bool) -> String {
        let mut content = String::new();
        let mut current_line = String::new();

        while !self.is_at_end() {
            let ch = self.current_char();

            if ch == '\n' {
                let check_line = if strip_tabs {
                    current_line.trim_start_matches('\t')
                } else {
                    current_line.trim()
                };

                if check_line == delimiter {
                    // Don't consume the trailing newline — let it become a
                    // Token::Newline so the parser sees the statement boundary.
                    break;
                }

                // Not delimiter - add line to content (with newline)
                if !content.is_empty() {
                    content.push('\n');
                }
                let line_to_add = if strip_tabs {
                    current_line.trim_start_matches('\t')
                } else {
                    &current_line
                };
                content.push_str(line_to_add);
                current_line.clear();

                self.advance(); // skip newline
            } else {
                current_line.push(self.advance());
            }
        }

        // Handle delimiter on last line without trailing newline
        if !current_line.is_empty() {
            let check_line = if strip_tabs {
                current_line.trim_start_matches('\t')
            } else {
                current_line.trim()
            };
            if check_line != delimiter {
                if !content.is_empty() {
                    content.push('\n');
                }
                let line_to_add = if strip_tabs {
                    current_line.trim_start_matches('\t')
                } else {
                    &current_line
                };
                content.push_str(line_to_add);
            }
        }

        content
    }

    /// Issue #61: Read a here-string (<<< word)
    /// Here-strings provide a single word/string to stdin
    /// Examples:
    ///   cat <<< "hello world"
    ///   read word <<< hello
    ///   cmd <<< "$variable"
    fn read_herestring(&mut self) -> Result<Token, LexerError> {
        // Skip whitespace after <<<
        while !self.is_at_end() && (self.current_char() == ' ' || self.current_char() == '\t') {
            self.advance();
        }

        if self.is_at_end() {
            return Err(LexerError::UnexpectedChar('\0', self.line, self.column));
        }

        let ch = self.current_char();

        // Handle quoted strings
        if ch == '"' || ch == '\'' {
            let quote = ch;
            self.advance(); // skip opening quote
            let mut content = String::new();

            while !self.is_at_end() {
                let c = self.current_char();
                if c == quote {
                    self.advance(); // skip closing quote
                    break;
                } else if c == '\\' && quote == '"' {
                    // Handle escape sequences in double quotes
                    self.advance();
                    if !self.is_at_end() {
                        content.push(self.advance());
                    }
                } else {
                    content.push(self.advance());
                }
            }

            return Ok(Token::HereString(content));
        }

        // Handle unquoted word (or $variable)
        let mut content = String::new();

        while !self.is_at_end() {
            let c = self.current_char();
            // Stop at whitespace, newline, pipe, or other shell metacharacters
            if c.is_whitespace() || c == '\n' || c == '|' || c == ';' || c == '&' {
                break;
            }
            content.push(self.advance());
        }

        Ok(Token::HereString(content))
    }

    fn read_string(&mut self, quote: char) -> Result<Token, LexerError> {
        let start_line = self.line;
        let start_col = self.column;

        self.advance(); // skip opening quote
        let mut string = String::new();

        // Track nesting depth for command substitutions $(...)
        // When inside a command substitution, quotes are part of the command,
        // not terminators for the outer string.
        let mut cmd_subst_depth = 0;

        while !self.is_at_end() {
            let ch = self.current_char();

            // Only treat quote as terminator if we're not inside a command substitution
            if ch == quote && cmd_subst_depth == 0 {
                break;
            }

            // Advance past the character
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
            } else if ch == '$' && !self.is_at_end() && self.current_char() == '(' {
                // Entering command substitution $(...)
                // Issue #59: Handle nested quotes inside command substitution
                string.push(ch);
                string.push(self.advance()); // push '('
                cmd_subst_depth += 1;
            } else if ch == '(' && cmd_subst_depth > 0 {
                // Nested parenthesis inside command substitution
                string.push(ch);
                cmd_subst_depth += 1;
            } else if ch == ')' && cmd_subst_depth > 0 {
                // Closing parenthesis - might be end of command substitution
                string.push(ch);
                cmd_subst_depth -= 1;
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

        // If followed by ':' + digit, treat as word (port mapping 8080:8080, version 1:2:3)
        if !self.is_at_end()
            && self.current_char() == ':'
            && self.peek_char(1).is_some_and(|c| c.is_ascii_digit())
        {
            num_str.push(self.advance()); // consume ':'
            while !self.is_at_end()
                && (self.current_char().is_ascii_digit() || self.current_char() == ':')
            {
                num_str.push(self.advance());
            }
            return Ok(Token::Identifier(num_str));
        }

        num_str
            .parse::<i64>()
            .map(Token::Number)
            .map_err(|_| LexerError::InvalidNumber(num_str))
    }

    fn read_identifier_or_keyword(&mut self) -> Token {
        let mut ident = String::new();
        let mut has_special_chars = false;

        while !self.is_at_end() {
            let ch = self.current_char();
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(self.advance());
            } else if self.is_ident_continuation_char(ch) || self.is_ident_separator_with_next(ch) {
                has_special_chars = true;
                ident.push(self.advance());
            } else {
                break;
            }
        }

        // Keywords can only match if the identifier has no special characters
        if !has_special_chars {
            if let Some(keyword) = Self::lookup_keyword(&ident) {
                return keyword;
            }
        }
        Token::Identifier(ident)
    }

    /// Characters that are always allowed as identifier continuations (paths, globs).
    fn is_ident_continuation_char(&self, ch: char) -> bool {
        ch == '/' || ch == '*' || ch == '?'
    }

    /// Characters that are allowed in identifiers only when followed by an
    /// alphanumeric character (or '/' for colon in URLs like http://...).
    /// BUG-010 FIX: Allow dashes in identifiers for function names like my-func.
    fn is_ident_separator_with_next(&self, ch: char) -> bool {
        if !matches!(ch, '-' | '.' | ':' | '@') {
            return false;
        }
        match self.peek_char(1) {
            Some(next) => next.is_alphanumeric() || (ch == ':' && next == '/'),
            None => false,
        }
    }

    /// Look up a keyword token from an identifier string.
    /// Returns `None` if the string is not a keyword.
    fn lookup_keyword(ident: &str) -> Option<Token> {
        match ident {
            "if" => Some(Token::If),
            "then" => Some(Token::Then),
            "elif" => Some(Token::Elif),
            "else" => Some(Token::Else),
            "fi" => Some(Token::Fi),
            "for" => Some(Token::For),
            "while" => Some(Token::While),
            "until" => Some(Token::Until),
            "select" => Some(Token::Select),
            "do" => Some(Token::Do),
            "done" => Some(Token::Done),
            "case" => Some(Token::Case),
            "esac" => Some(Token::Esac),
            "in" => Some(Token::In),
            "function" => Some(Token::Function),
            "return" => Some(Token::Return),
            "export" => Some(Token::Export),
            "local" => Some(Token::Local),
            "coproc" => Some(Token::Coproc),
            _ => None,
        }
    }

    fn read_bare_word(&mut self) -> Token {
        let mut word = String::new();

        while !self.is_at_end() {
            let ch = self.current_char();

            // Handle escape sequences (e.g., \; in find -exec ... \;)
            if ch == '\\' {
                word.push(self.advance()); // include backslash
                if !self.is_at_end() {
                    word.push(self.advance()); // include escaped char
                }
                continue;
            }

            // Handle extended glob patterns inline: @(...), +(...), ?(...), !(...)
            if self.is_extended_glob_start(ch) {
                self.read_inline_extended_glob(&mut word);
            } else if Self::is_bare_word_char(ch) {
                word.push(self.advance());
            } else {
                break;
            }
        }

        Token::Identifier(word)
    }

    /// Check if the current character starts an extended glob pattern: @(, +(, ?(, !(
    fn is_extended_glob_start(&self, ch: char) -> bool {
        matches!(ch, '@' | '+' | '?' | '!') && self.peek_char(1) == Some('(')
    }

    /// Read an extended glob pattern (@(...), +(...), etc.) and append it to `word`.
    fn read_inline_extended_glob(&mut self, word: &mut String) {
        word.push(self.advance()); // push @/+/?/!
        word.push(self.advance()); // push (
        let mut depth = 1;
        while !self.is_at_end() && depth > 0 {
            let c = self.current_char();
            if c == '(' {
                depth += 1;
            } else if c == ')' {
                depth -= 1;
                if depth == 0 {
                    word.push(self.advance());
                    break;
                }
            }
            word.push(self.advance());
        }
    }

    /// Characters that are valid in bare words (unquoted strings).
    /// Includes alphanumeric, path separators, globs, dots, dashes, plus, percent, etc.
    fn is_bare_word_char(ch: char) -> bool {
        ch.is_alphanumeric()
            || matches!(
                ch,
                '/' | '.' | '-' | '_' | '*' | '?' | '~' | ':' | '+' | '%' | ',' | '=' | '@'
            )
    }

    /// Issue #69: Check if current position starts a brace expansion
    /// Brace expansion: {a,b,c} or {1..10}
    fn is_brace_expansion(&self) -> bool {
        // Look ahead to see if there's a comma or .. inside the braces
        // Must skip quoted strings to avoid false positives like { echo "a,b" }
        let mut i = self.position + 1; // Skip the '{'
        let mut depth = 1;
        let mut in_single_quote = false;
        let mut in_double_quote = false;

        while i < self.input.len() && depth > 0 {
            let ch = self.input[i];

            // Handle quote state
            if ch == '\'' && !in_double_quote {
                in_single_quote = !in_single_quote;
                i += 1;
                continue;
            }
            if ch == '"' && !in_single_quote {
                in_double_quote = !in_double_quote;
                i += 1;
                continue;
            }

            // Skip content inside quotes
            if in_single_quote || in_double_quote {
                i += 1;
                continue;
            }

            // Check for newlines - function bodies have newlines, brace expansion doesn't
            if ch == '\n' {
                return false; // Not brace expansion - likely function body
            }

            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return false; // No expansion marker found before closing brace
                    }
                }
                ',' => return true, // Found comma - it's brace expansion
                '.' if i + 1 < self.input.len() && self.input[i + 1] == '.' => {
                    return true; // Found .. - it's sequence expansion
                }
                _ => {}
            }
            i += 1;
        }
        false
    }

    /// Issue #67: Read process substitution <(cmd) or >(cmd)
    fn read_process_substitution(&mut self, direction: char) -> Result<Token, LexerError> {
        self.advance(); // skip '<' or '>'
        self.advance(); // skip '('

        let mut content = String::new();
        let mut depth = 1;

        while !self.is_at_end() && depth > 0 {
            let ch = self.current_char();
            if ch == '(' {
                depth += 1;
            } else if ch == ')' {
                depth -= 1;
                if depth == 0 {
                    self.advance();
                    break;
                }
            }
            content.push(self.advance());
        }

        // Return as identifier for now - proper AST support would need a new variant
        Ok(Token::Identifier(format!("{}({})", direction, content)))
    }

    /// Issue #67: Read standalone arithmetic ((expr))
    fn read_standalone_arithmetic(&mut self) -> Result<Token, LexerError> {
        self.advance(); // skip first '('
        self.advance(); // skip second '('

        let mut content = String::new();
        let mut depth = 2; // Started with ((

        while !self.is_at_end() && depth > 0 {
            let ch = self.current_char();
            if ch == '(' {
                depth += 1;
                content.push(self.advance());
            } else if ch == ')' {
                depth -= 1;
                if depth <= 1 {
                    // depth <= 1 means we've seen the first ) of ))
                    // Don't push it, just advance past both closing parens
                    self.advance(); // skip first ')'
                    if !self.is_at_end() && self.current_char() == ')' {
                        self.advance(); // skip second ')'
                    }
                    break;
                }
                content.push(self.advance());
            } else {
                content.push(self.advance());
            }
        }

        // Return as arithmetic expansion token
        Ok(Token::ArithmeticExpansion(content))
    }

    /// Issue #69: Read brace expansion as a single identifier token
    /// {a,b,c} -> Token::Identifier("{a,b,c}")
    fn read_brace_expansion(&mut self) -> Result<Token, LexerError> {
        let mut expansion = String::new();
        let mut depth = 0;

        while !self.is_at_end() {
            let ch = self.current_char();

            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    expansion.push(self.advance());
                    break;
                }
            }

            expansion.push(self.advance());
        }

        Ok(Token::Identifier(expansion))
    }

    fn read_operator(&mut self) -> Result<Token, LexerError> {
        let ch = self.current_char();
        let next_ch = self.peek_char(1);

        // Delegate to specialized helpers based on the first character
        match ch {
            '<' | '>' => return self.read_redirect_or_comparison(ch, next_ch),
            '=' => return self.read_equality_or_assign(next_ch),
            '@' | '+' | '?' if next_ch == Some('(') => {
                return self.read_extended_glob(ch);
            }
            '!' if next_ch == Some('(') => return self.read_extended_glob(ch),
            ';' => return self.read_semicolon_operator(next_ch),
            _ => {}
        }

        // Handle remaining operators inline (simple single/double char ops)
        let token = match (ch, next_ch) {
            ('!', Some('=')) => {
                self.advance();
                self.advance();
                Token::Ne
            }
            ('!', _) => {
                self.advance();
                Token::Not
            }
            ('&', Some('&')) => {
                self.advance();
                self.advance();
                Token::And
            }
            ('&', _) => {
                self.advance();
                Token::Ampersand
            }
            ('|', Some('|')) => {
                self.advance();
                self.advance();
                Token::Or
            }
            ('|', _) => {
                self.advance();
                Token::Pipe
            }
            ('[', Some('[')) => {
                self.advance();
                self.advance();
                Token::DoubleLeftBracket
            }
            ('[', _) => {
                self.advance();
                Token::LeftBracket
            }
            (']', Some(']')) => {
                self.advance();
                self.advance();
                Token::DoubleRightBracket
            }
            (']', _) => {
                self.advance();
                Token::RightBracket
            }
            ('+', Some('=')) => {
                // BUG-012 FIX: Array append +=
                self.advance(); // skip '+'
                self.advance(); // skip '='
                Token::Identifier("+=".to_string())
            }
            ('(', Some('(')) => {
                // Issue #67: Standalone arithmetic ((expr))
                return self.read_standalone_arithmetic();
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
                // Issue #69: Check for brace expansion {a,b,c} or {1..10}
                if self.is_brace_expansion() {
                    return self.read_brace_expansion();
                }
                self.advance();
                Token::LeftBrace
            }
            ('}', _) => {
                self.advance();
                Token::RightBrace
            }
            ('?', _) => {
                // Single-char glob: file?.txt
                self.advance();
                Token::Identifier("?".to_string())
            }
            _ => {
                return Err(LexerError::UnexpectedChar(ch, self.line, self.column));
            }
        };

        Ok(token)
    }

    /// Handle operators starting with `<` or `>`: redirects, comparisons, and
    /// process substitutions.
    fn read_redirect_or_comparison(
        &mut self,
        ch: char,
        next_ch: Option<char>,
    ) -> Result<Token, LexerError> {
        let token = match (ch, next_ch) {
            ('<', Some('<')) => {
                // Check for here-string (<<<) vs heredoc (<<) vs indented heredoc (<<-)
                // Issue #61: Here-strings must be checked before heredocs
                if self.peek_char(2) == Some('<') {
                    // Here-string: <<< "string"
                    self.advance(); // skip first '<'
                    self.advance(); // skip second '<'
                    self.advance(); // skip third '<'
                    return self.read_herestring();
                } else if self.peek_char(2) == Some('-') {
                    // BUG-007 FIX: Indented heredoc: <<-DELIMITER
                    self.advance(); // skip first '<'
                    self.advance(); // skip second '<'
                    self.advance(); // skip '-'
                    return self.read_heredoc_indented();
                } else {
                    // Heredoc: <<DELIMITER or <<'DELIMITER' or <<"DELIMITER"
                    self.advance(); // skip first '<'
                    self.advance(); // skip second '<'
                    return self.read_heredoc();
                }
            }
            ('<', Some('(')) => {
                // Issue #67: Process substitution <(cmd)
                return self.read_process_substitution('<');
            }
            ('>', Some('(')) => {
                // Issue #67: Process substitution >(cmd) (output redirection variant)
                return self.read_process_substitution('>');
            }
            ('>', Some('|')) => {
                // BUG-016 FIX: Noclobber redirect >|
                self.advance(); // skip '>'
                self.advance(); // skip '|'
                Token::Identifier(">|".to_string())
            }
            ('<', Some('>')) => {
                // BUG-017 FIX: Read-write redirect <>
                self.advance(); // skip '<'
                self.advance(); // skip '>'
                Token::Identifier("<>".to_string())
            }
            ('<', Some('=')) => {
                self.advance();
                self.advance();
                Token::Le
            }
            ('>', Some('>')) => {
                // Append redirection: >>
                self.advance();
                self.advance();
                Token::GtGt
            }
            ('>', Some('=')) => {
                self.advance();
                self.advance();
                Token::Ge
            }
            ('<', _) => {
                self.advance();
                Token::Lt
            }
            ('>', _) => {
                self.advance();
                Token::Gt
            }
            _ => return Err(LexerError::UnexpectedChar(ch, self.line, self.column)),
        };
        Ok(token)
    }

    /// Handle operators starting with `=`: equality (`==`), regex match (`=~`),
    /// and plain assignment (`=`).
    fn read_equality_or_assign(&mut self, next_ch: Option<char>) -> Result<Token, LexerError> {
        match next_ch {
            Some('=') => {
                self.advance();
                self.advance();
                Ok(Token::Eq)
            }
            Some('~') => {
                // =~ regex match operator (used in [[ ... =~ pattern ]])
                self.advance(); // skip '='
                self.advance(); // skip '~'
                self.skip_whitespace_except_newline();
                let pattern = self.read_regex_pattern();
                Ok(Token::Identifier(format!("=~ {}", pattern)))
            }
            _ => {
                self.advance();
                Ok(Token::Assign)
            }
        }
    }

    /// Read a regex pattern after `=~` until `]]`, newline, or unquoted `;`.
    /// Tracks bracket depth to avoid breaking on `]]` inside `[[:class:]]`.
    fn read_regex_pattern(&mut self) -> String {
        let mut pattern = String::new();
        let mut bracket_depth = 0i32;
        while !self.is_at_end() {
            let c = self.current_char();
            if c == '\n' {
                break;
            }
            if self.is_regex_terminator(c, bracket_depth) {
                break;
            }
            bracket_depth = Self::update_bracket_depth(c, bracket_depth);
            pattern.push(self.advance());
        }
        pattern.trim_end().to_string()
    }

    /// Check if the current character terminates a regex pattern.
    /// `]]` terminates when not inside character class brackets; `;` terminates
    /// outside brackets.
    fn is_regex_terminator(&self, c: char, bracket_depth: i32) -> bool {
        if c == ']' && bracket_depth == 0 && self.peek_char(1) == Some(']') {
            return true;
        }
        c == ';' && bracket_depth == 0
    }

    /// Update bracket depth tracking for regex pattern reading.
    fn update_bracket_depth(c: char, depth: i32) -> i32 {
        match c {
            '[' => depth + 1,
            ']' if depth > 0 => depth - 1,
            _ => depth,
        }
    }

    /// Handle extended glob patterns: `@(...)`, `+(...)`, `?(...)`, `!(...)`.
    /// The `glob_char` parameter is the leading character (`@`, `+`, `?`, or `!`).
    fn read_extended_glob(&mut self, _glob_char: char) -> Result<Token, LexerError> {
        let glob_type = self.advance(); // consume glob_char (@, +, ?, or !)
        self.advance(); // consume (
        let mut pattern = String::new();
        let mut depth = 1;
        while !self.is_at_end() && depth > 0 {
            let c = self.current_char();
            if c == '(' {
                depth += 1;
            } else if c == ')' {
                depth -= 1;
                if depth == 0 {
                    self.advance();
                    break;
                }
            }
            pattern.push(self.advance());
        }
        Ok(Token::Identifier(format!("{}({})", glob_type, pattern)))
    }

    /// Handle operators starting with `;`: double-semicolon (`;;`),
    /// case resume (`;;&`), case fall-through (`;&`), and plain semicolon.
    fn read_semicolon_operator(&mut self, next_ch: Option<char>) -> Result<Token, LexerError> {
        match next_ch {
            Some(';') => {
                // BUG-008, BUG-009 FIX: Check for ;;& (case resume) before ;;
                self.advance(); // skip first ';'
                self.advance(); // skip second ';'
                if self.peek_char(0) == Some('&') {
                    self.advance(); // skip '&'
                    Ok(Token::Identifier(";;&".to_string())) // Case resume
                } else {
                    Ok(Token::Identifier(";;".to_string())) // Case terminator
                }
            }
            Some('&') => {
                // BUG-008 FIX: Case fall-through ;&
                self.advance(); // skip ';'
                self.advance(); // skip '&'
                Ok(Token::Identifier(";&".to_string()))
            }
            _ => {
                self.advance();
                Ok(Token::Semicolon)
            }
        }
    }
}

#[cfg(test)]
#[path = "lexer_tests_extracted.rs"]
mod tests_extracted;
