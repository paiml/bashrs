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
            self.advance();
            return Ok(Token::Variable("@".to_string()));
        }

        // Handle shell special variables: $#, $?, $!, $-
        if !self.is_at_end() && matches!(self.current_char(), '#' | '?' | '!' | '-') {
            let special = self.advance();
            return Ok(Token::Variable(special.to_string()));
        }

        let mut var_name = String::new();

        // Handle ${VAR} syntax
        // BUG-001 FIX: Handle nested parameter expansion like ${foo:-${bar:-default}}
        if !self.is_at_end() && self.current_char() == '{' {
            self.advance();
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
                } else if ch == '$' && !self.is_at_end() {
                    // Handle nested ${...} or $(...)
                    var_name.push(self.advance());
                    if !self.is_at_end() && self.current_char() == '{' {
                        brace_depth += 1;
                        var_name.push(self.advance());
                    }
                } else {
                    var_name.push(self.advance());
                }
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

    fn read_heredoc(&mut self) -> Result<Token, LexerError> {
        // BUG-006 FIX: Handle quoted delimiters <<'EOF' or <<"EOF"
        // Skip any leading whitespace
        while !self.is_at_end() && (self.current_char() == ' ' || self.current_char() == '\t') {
            self.advance();
        }

        // Check for quoted delimiter
        let mut delimiter = String::new();
        let quote_char =
            if !self.is_at_end() && (self.current_char() == '\'' || self.current_char() == '"') {
                let q = self.current_char();
                self.advance(); // skip opening quote
                Some(q)
            } else {
                None
            };

        // Read delimiter
        while !self.is_at_end() {
            let ch = self.current_char();
            if let Some(q) = quote_char {
                // Quoted delimiter - read until closing quote
                if ch == q {
                    self.advance(); // skip closing quote
                    break;
                }
                delimiter.push(self.advance());
            } else {
                // Unquoted delimiter - alphanumeric and underscore
                if ch.is_alphanumeric() || ch == '_' {
                    delimiter.push(self.advance());
                } else {
                    break;
                }
            }
        }

        if delimiter.is_empty() {
            let ch = if self.is_at_end() { '\0' } else { self.current_char() };
            return Err(LexerError::UnexpectedChar(
                ch,
                self.line,
                self.column,
            ));
        }

        // Skip to end of line (heredoc content starts on next line)
        while !self.is_at_end() && self.current_char() != '\n' {
            self.advance();
        }
        if !self.is_at_end() {
            self.advance(); // skip newline
        }

        // Read heredoc content until we find a line matching the delimiter
        let mut content = String::new();
        let mut current_line = String::new();

        while !self.is_at_end() {
            let ch = self.current_char();

            if ch == '\n' {
                // Check if current_line matches delimiter
                if current_line.trim() == delimiter {
                    // Found delimiter - skip the newline and stop
                    self.advance();
                    break;
                }

                // Not delimiter - add line to content (with newline)
                if !content.is_empty() {
                    content.push('\n');
                }
                content.push_str(&current_line);
                current_line.clear();

                self.advance(); // skip newline
            } else {
                current_line.push(self.advance());
            }
        }

        Ok(Token::Heredoc { delimiter, content })
    }

    /// BUG-007 FIX: Read indented heredoc (<<-DELIMITER)
    /// In indented heredocs, leading tabs are stripped from content lines
    /// and the delimiter can be indented with tabs
    fn read_heredoc_indented(&mut self) -> Result<Token, LexerError> {
        // Skip any leading whitespace
        while !self.is_at_end() && (self.current_char() == ' ' || self.current_char() == '\t') {
            self.advance();
        }

        // Check for quoted delimiter
        let mut delimiter = String::new();
        let quote_char =
            if !self.is_at_end() && (self.current_char() == '\'' || self.current_char() == '"') {
                let q = self.current_char();
                self.advance();
                Some(q)
            } else {
                None
            };

        // Read delimiter
        while !self.is_at_end() {
            let ch = self.current_char();
            if let Some(q) = quote_char {
                if ch == q {
                    self.advance();
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
            let ch = if self.is_at_end() { '\0' } else { self.current_char() };
            return Err(LexerError::UnexpectedChar(
                ch,
                self.line,
                self.column,
            ));
        }

        // Skip to end of line
        while !self.is_at_end() && self.current_char() != '\n' {
            self.advance();
        }
        if !self.is_at_end() {
            self.advance();
        }

        // Read heredoc content - strip leading tabs
        let mut content = String::new();
        let mut current_line = String::new();

        while !self.is_at_end() {
            let ch = self.current_char();

            if ch == '\n' {
                // Strip leading tabs and check for delimiter
                let trimmed = current_line.trim_start_matches('\t');
                if trimmed == delimiter {
                    self.advance();
                    break;
                }

                // Add stripped line to content
                if !content.is_empty() {
                    content.push('\n');
                }
                content.push_str(trimmed);
                current_line.clear();

                self.advance();
            } else {
                current_line.push(self.advance());
            }
        }

        Ok(Token::Heredoc { delimiter, content })
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

        while !self.is_at_end() {
            let ch = self.current_char();
            // BUG-010 FIX: Allow dashes in identifiers for function names like my-func
            // Dashes are allowed if followed by alphanumeric (not at end, not before operator)
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(self.advance());
            } else if ch == '-' || ch == '.' || ch == ':' {
                // Allow dash/dot/colon in identifiers for function names
                // But only if followed by alphanumeric (not operators like -eq)
                // Also allow colon followed by / for URLs (http://...)
                if let Some(next) = self.peek_char(1) {
                    if next.is_alphanumeric() || (ch == ':' && next == '/') {
                        ident.push(self.advance());
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else if ch == '/' || ch == '*' || ch == '?' {
                // Path/glob continuation: dist/*, src/*.rs, path/to/file, etc.
                ident.push(self.advance());
            } else {
                break;
            }
        }

        // Check for keywords (only if no special chars in identifier)
        if !ident.contains('-')
            && !ident.contains('.')
            && !ident.contains(':')
            && !ident.contains('/')
            && !ident.contains('*')
            && !ident.contains('?')
        {
            match ident.as_str() {
                "if" => return Token::If,
                "then" => return Token::Then,
                "elif" => return Token::Elif,
                "else" => return Token::Else,
                "fi" => return Token::Fi,
                "for" => return Token::For,
                "while" => return Token::While,
                "until" => return Token::Until,
                "select" => return Token::Select, // F017: select statement
                "do" => return Token::Do,
                "done" => return Token::Done,
                "case" => return Token::Case,
                "esac" => return Token::Esac,
                "in" => return Token::In,
                "function" => return Token::Function,
                "return" => return Token::Return,
                "export" => return Token::Export,
                "local" => return Token::Local,
                "coproc" => return Token::Coproc, // BUG-018
                _ => {}
            }
        }
        Token::Identifier(ident)
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

            // Bare words can contain alphanumeric, path separators, globs, dots, dashes, plus signs, percent signs
            // Note: '+' and '%' added for date +%FORMAT support (PARSER-ENH-001)
            // Issue #131: ',' added for Docker mount options like type=bind,source=...,target=...
            // Issue #131: '=' added for key=value arguments like --mount type=bind
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
                || ch == ','
                || ch == '='
            {
                word.push(self.advance());
            } else {
                break;
            }
        }

        Token::Identifier(word)
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
            ('+', Some('=')) => {
                // BUG-012 FIX: Array append +=
                self.advance(); // skip '+'
                self.advance(); // skip '='
                Token::Identifier("+=".to_string())
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
            ('!', Some('(')) => {
                // BUG-020 FIX: Extended glob: !(...)
                self.advance(); // consume !
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
                Token::Identifier(format!("!({})", pattern))
            }
            ('!', _) => {
                self.advance();
                Token::Not
            }
            ('|', _) => {
                self.advance();
                Token::Pipe
            }
            (';', Some(';')) => {
                // BUG-008, BUG-009 FIX: Check for ;;& (case resume) before ;;
                self.advance(); // skip first ';'
                self.advance(); // skip second ';'
                if self.peek_char(0) == Some('&') {
                    self.advance(); // skip '&'
                    Token::Identifier(";;&".to_string()) // Case resume
                } else {
                    Token::Identifier(";;".to_string()) // Case terminator
                }
            }
            (';', Some('&')) => {
                // BUG-008 FIX: Case fall-through ;&
                self.advance(); // skip ';'
                self.advance(); // skip '&'
                Token::Identifier(";&".to_string())
            }
            (';', _) => {
                self.advance();
                Token::Semicolon
            }
            ('&', _) => {
                self.advance();
                Token::Ampersand
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
            ('[', _) => {
                self.advance();
                Token::LeftBracket
            }
            (']', _) => {
                self.advance();
                Token::RightBracket
            }
            // BUG-019, BUG-020, BUG-021 FIX: Extended globs and glob patterns
            // @(pattern|pattern), !(pattern), +(pattern), *(pattern), ?(pattern)
            // and ? as single-char glob
            ('@', Some('(')) | ('+', Some('(')) => {
                // Extended glob: @(...) or +(...)
                let glob_type = self.advance(); // consume @ or +
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
                Token::Identifier(format!("{}({})", glob_type, pattern))
            }
            ('?', Some('(')) => {
                // Extended glob: ?(...)
                self.advance(); // consume ?
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
                Token::Identifier(format!("?({})", pattern))
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

    // ============================================================================
    // Token Display Tests
    // ============================================================================

    #[test]
    fn test_token_display_if() {
        assert_eq!(format!("{}", Token::If), "if");
    }

    #[test]
    fn test_token_display_then() {
        assert_eq!(format!("{}", Token::Then), "then");
    }

    #[test]
    fn test_token_display_identifier() {
        assert_eq!(
            format!("{}", Token::Identifier("foo".to_string())),
            "Identifier(foo)"
        );
    }

    #[test]
    fn test_token_display_string() {
        assert_eq!(
            format!("{}", Token::String("hello".to_string())),
            "String(hello)"
        );
    }

    #[test]
    fn test_token_display_number() {
        assert_eq!(format!("{}", Token::Number(42)), "Number(42)");
    }

    #[test]
    fn test_token_display_variable() {
        assert_eq!(format!("{}", Token::Variable("x".to_string())), "$x");
    }

    #[test]
    fn test_token_display_arithmetic() {
        assert_eq!(
            format!("{}", Token::ArithmeticExpansion("1+2".to_string())),
            "$((1+2)"
        );
    }

    #[test]
    fn test_token_display_command_sub() {
        assert_eq!(
            format!("{}", Token::CommandSubstitution("ls".to_string())),
            "$(ls)"
        );
    }

    #[test]
    fn test_token_display_comment() {
        assert_eq!(format!("{}", Token::Comment("test".to_string())), "#test");
    }

    #[test]
    fn test_token_display_eof() {
        assert_eq!(format!("{}", Token::Eof), "EOF");
    }

    #[test]
    fn test_token_display_other() {
        // Other tokens use Debug format
        let output = format!("{}", Token::Semicolon);
        assert!(output.contains("Semicolon"));
    }

    // ============================================================================
    // LexerError Tests
    // ============================================================================

    #[test]
    fn test_lexer_error_unexpected_char() {
        let err = LexerError::UnexpectedChar('$', 1, 5);
        assert!(err.to_string().contains("'$'"));
        assert!(err.to_string().contains("line 1"));
    }

    #[test]
    fn test_lexer_error_unterminated_string() {
        let err = LexerError::UnterminatedString(2, 10);
        assert!(err.to_string().contains("Unterminated"));
        assert!(err.to_string().contains("line 2"));
    }

    #[test]
    fn test_lexer_error_invalid_number() {
        let err = LexerError::InvalidNumber("abc123".to_string());
        assert!(err.to_string().contains("Invalid"));
    }

    // ============================================================================
    // Lexer Method Tests
    // ============================================================================

    #[test]
    fn test_lexer_new() {
        let lexer = Lexer::new("echo hello");
        assert_eq!(lexer.position, 0);
        assert_eq!(lexer.line, 1);
        assert_eq!(lexer.column, 1);
    }

    #[test]
    fn test_lexer_empty_input() {
        let mut lexer = Lexer::new("");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Eof);
    }

    #[test]
    fn test_lexer_whitespace_only() {
        let mut lexer = Lexer::new("   \t   ");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Eof);
    }

    #[test]
    fn test_lexer_newline() {
        let mut lexer = Lexer::new("\n");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.iter().any(|t| matches!(t, Token::Newline)));
    }

    #[test]
    fn test_lexer_multiple_newlines() {
        let mut lexer = Lexer::new("\n\n\n");
        let tokens = lexer.tokenize().unwrap();
        assert!(
            tokens
                .iter()
                .filter(|t| matches!(t, Token::Newline))
                .count()
                >= 1
        );
    }

    #[test]
    fn test_lexer_variable_simple() {
        let mut lexer = Lexer::new("$FOO");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Variable("FOO".to_string()));
    }

    #[test]
    fn test_lexer_variable_braces() {
        let mut lexer = Lexer::new("${FOO}");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0], Token::Variable(_)));
    }

    #[test]
    fn test_lexer_variable_special() {
        let mut lexer = Lexer::new("$?");
        let tokens = lexer.tokenize().unwrap();
        // $? is tokenized as Variable - content may vary by implementation
        assert!(matches!(tokens[0], Token::Variable(_)));
    }

    #[test]
    fn test_lexer_command_substitution() {
        let mut lexer = Lexer::new("$(echo hello)");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0], Token::CommandSubstitution(_)));
    }

    #[test]
    fn test_lexer_keywords() {
        let keywords = vec![
            ("if", Token::If),
            ("then", Token::Then),
            ("elif", Token::Elif),
            ("else", Token::Else),
            ("fi", Token::Fi),
            ("for", Token::For),
            ("while", Token::While),
            ("until", Token::Until),
            ("do", Token::Do),
            ("done", Token::Done),
            ("case", Token::Case),
            ("esac", Token::Esac),
            ("in", Token::In),
            ("function", Token::Function),
            ("return", Token::Return),
            ("export", Token::Export),
            ("local", Token::Local),
            ("coproc", Token::Coproc),
        ];

        for (input, expected) in keywords {
            let mut lexer = Lexer::new(input);
            let tokens = lexer.tokenize().unwrap();
            assert_eq!(tokens[0], expected, "Failed for keyword: {}", input);
        }
    }

    #[test]
    fn test_lexer_operators() {
        let mut lexer = Lexer::new("= == != < <= > >= && || !");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.contains(&Token::Assign));
        assert!(tokens.contains(&Token::Eq));
        assert!(tokens.contains(&Token::Ne));
    }

    #[test]
    fn test_lexer_pipe() {
        let mut lexer = Lexer::new("echo hello | grep h");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.contains(&Token::Pipe));
    }

    #[test]
    fn test_lexer_semicolon() {
        let mut lexer = Lexer::new("echo a; echo b");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.contains(&Token::Semicolon));
    }

    #[test]
    fn test_lexer_ampersand() {
        let mut lexer = Lexer::new("sleep 1 &");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.contains(&Token::Ampersand));
    }

    #[test]
    fn test_lexer_parentheses() {
        let mut lexer = Lexer::new("(echo hello)");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.contains(&Token::LeftParen));
        assert!(tokens.contains(&Token::RightParen));
    }

    #[test]
    fn test_lexer_braces() {
        let mut lexer = Lexer::new("{ echo hello; }");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.contains(&Token::LeftBrace));
        assert!(tokens.contains(&Token::RightBrace));
    }

    #[test]
    fn test_lexer_brackets() {
        let mut lexer = Lexer::new("[ $x ]");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.contains(&Token::LeftBracket));
        assert!(tokens.contains(&Token::RightBracket));
    }

    #[test]
    fn test_lexer_double_brackets() {
        let mut lexer = Lexer::new("[[ $x ]]");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.contains(&Token::DoubleLeftBracket));
        assert!(tokens.contains(&Token::DoubleRightBracket));
    }

    #[test]
    fn test_lexer_single_quoted_string() {
        let mut lexer = Lexer::new("'hello world'");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::String("hello world".to_string()));
    }

    #[test]
    fn test_lexer_double_quoted_string() {
        let mut lexer = Lexer::new("\"hello world\"");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::String("hello world".to_string()));
    }

    #[test]
    fn test_lexer_number() {
        let mut lexer = Lexer::new("42");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Number(42));
    }

    #[test]
    fn test_lexer_negative_number() {
        let mut lexer = Lexer::new("x=-5");
        let tokens = lexer.tokenize().unwrap();
        // -5 may be parsed as identifier or number depending on context
        assert!(tokens.len() >= 3);
    }

    #[test]
    fn test_lexer_herestring() {
        let mut lexer = Lexer::new("cat <<< 'hello'");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.iter().any(|t| matches!(t, Token::HereString(_))));
    }

    #[test]
    fn test_lexer_heredoc() {
        let mut lexer = Lexer::new("cat <<EOF\nhello\nEOF");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.iter().any(|t| matches!(t, Token::Heredoc { .. })));
    }

    #[test]
    fn test_lexer_append_redirect() {
        let mut lexer = Lexer::new("echo hello >> file");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.contains(&Token::GtGt));
    }

    #[test]
    fn test_lexer_for_loop() {
        let mut lexer = Lexer::new("for i in 1 2 3; do echo $i; done");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.contains(&Token::For));
        assert!(tokens.contains(&Token::In));
        assert!(tokens.contains(&Token::Do));
        assert!(tokens.contains(&Token::Done));
    }

    #[test]
    fn test_lexer_while_loop() {
        let mut lexer = Lexer::new("while true; do echo loop; done");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.contains(&Token::While));
        assert!(tokens.contains(&Token::Do));
        assert!(tokens.contains(&Token::Done));
    }

    #[test]
    fn test_lexer_case_statement() {
        let mut lexer = Lexer::new("case $x in a) echo a;; esac");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.contains(&Token::Case));
        assert!(tokens.contains(&Token::In));
        assert!(tokens.contains(&Token::Esac));
    }

    #[test]
    fn test_lexer_function_definition() {
        let mut lexer = Lexer::new("function foo { echo hello; }");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.contains(&Token::Function));
    }

    #[test]
    fn test_lexer_export() {
        let mut lexer = Lexer::new("export FOO=bar");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.contains(&Token::Export));
    }

    #[test]
    fn test_lexer_local() {
        let mut lexer = Lexer::new("local x=5");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.contains(&Token::Local));
    }

    #[test]
    fn test_lexer_return() {
        let mut lexer = Lexer::new("return 0");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.contains(&Token::Return));
    }

    #[test]
    fn test_token_clone() {
        let tokens = vec![
            Token::If,
            Token::Then,
            Token::Identifier("x".to_string()),
            Token::String("hello".to_string()),
            Token::Number(42),
            Token::Variable("x".to_string()),
            Token::Eof,
        ];
        for token in tokens {
            let _ = token.clone();
        }
    }

    #[test]
    fn test_token_eq() {
        assert_eq!(Token::If, Token::If);
        assert_ne!(Token::If, Token::Then);
        assert_eq!(Token::Number(42), Token::Number(42));
        assert_ne!(Token::Number(42), Token::Number(43));
    }

    #[test]
    fn test_lexer_error_debug() {
        let err = LexerError::UnexpectedChar('x', 1, 1);
        let debug = format!("{:?}", err);
        assert!(debug.contains("UnexpectedChar"));
    }

    #[test]
    fn test_lexer_complex_script() {
        let input = r#"
#!/bin/bash
# Comment
FOO=bar
if [ "$FOO" == "bar" ]; then
    echo "Hello $FOO"
fi
"#;
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        assert!(result.is_ok());
    }

    #[test]
    fn test_lexer_escape_in_string() {
        let mut lexer = Lexer::new(r#""hello\nworld""#);
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0], Token::String(_)));
    }

    #[test]
    fn test_lexer_dollar_sign_context() {
        // $ followed by space might be handled differently
        let mut lexer = Lexer::new("echo $FOO");
        let tokens = lexer.tokenize().unwrap();
        // Should have a variable token
        assert!(tokens.iter().any(|t| matches!(t, Token::Variable(_))));
    }

    // ============================================================================
    // Coverage Tests - read_operator (LEX_OP_COV_001-020)
    // ============================================================================

    /// Helper: tokenize and return the token types
    fn lex(input: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(input);
        lexer.tokenize().unwrap_or_default()
    }

    #[test]
    fn test_LEX_OP_COV_001_ne_operator() {
        let tokens = lex("[ a != b ]");
        assert!(tokens.iter().any(|t| matches!(t, Token::Ne)));
    }

    #[test]
    fn test_LEX_OP_COV_002_le_operator() {
        let tokens = lex("[[ a <= b ]]");
        assert!(tokens.iter().any(|t| matches!(t, Token::Le)));
    }

    #[test]
    fn test_LEX_OP_COV_003_ge_operator() {
        let tokens = lex("[[ a >= b ]]");
        assert!(tokens.iter().any(|t| matches!(t, Token::Ge)));
    }

    #[test]
    fn test_LEX_OP_COV_004_append_redirect() {
        let tokens = lex("echo hi >> file");
        assert!(tokens.iter().any(|t| matches!(t, Token::GtGt)));
    }

    #[test]
    fn test_LEX_OP_COV_005_and_operator() {
        let tokens = lex("true && false");
        assert!(tokens.iter().any(|t| matches!(t, Token::And)));
    }

    #[test]
    fn test_LEX_OP_COV_006_or_operator() {
        let tokens = lex("true || false");
        assert!(tokens.iter().any(|t| matches!(t, Token::Or)));
    }

    #[test]
    fn test_LEX_OP_COV_007_double_brackets() {
        let tokens = lex("[[ x == y ]]");
        assert!(tokens.iter().any(|t| matches!(t, Token::DoubleLeftBracket)));
        assert!(tokens
            .iter()
            .any(|t| matches!(t, Token::DoubleRightBracket)));
    }

    #[test]
    fn test_LEX_OP_COV_008_plus_equals() {
        let tokens = lex("arr+=(val)");
        assert!(tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == "+=")));
    }

    #[test]
    fn test_LEX_OP_COV_009_not_operator() {
        let tokens = lex("! true");
        assert!(tokens.iter().any(|t| matches!(t, Token::Not)));
    }

    #[test]
    fn test_LEX_OP_COV_010_pipe() {
        let tokens = lex("ls | grep foo");
        assert!(tokens.iter().any(|t| matches!(t, Token::Pipe)));
    }

    #[test]
    fn test_LEX_OP_COV_011_case_double_semicolon() {
        let tokens = lex(";;");
        assert!(tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == ";;")));
    }

    #[test]
    fn test_LEX_OP_COV_012_case_semicolon_ampersand() {
        let tokens = lex(";&");
        assert!(tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == ";&")));
    }

    #[test]
    fn test_LEX_OP_COV_013_ampersand_background() {
        let tokens = lex("sleep 1 &");
        assert!(tokens.iter().any(|t| matches!(t, Token::Ampersand)));
    }

    #[test]
    fn test_LEX_OP_COV_014_parens() {
        let tokens = lex("(echo hi)");
        assert!(tokens.iter().any(|t| matches!(t, Token::LeftParen)));
        assert!(tokens.iter().any(|t| matches!(t, Token::RightParen)));
    }

    #[test]
    fn test_LEX_OP_COV_015_braces() {
        let tokens = lex("{ echo hi; }");
        assert!(tokens.iter().any(|t| matches!(t, Token::LeftBrace)));
        assert!(tokens.iter().any(|t| matches!(t, Token::RightBrace)));
    }

    #[test]
    fn test_LEX_OP_COV_016_brackets() {
        let tokens = lex("[ -f file ]");
        assert!(tokens.iter().any(|t| matches!(t, Token::LeftBracket)));
        assert!(tokens.iter().any(|t| matches!(t, Token::RightBracket)));
    }

    #[test]
    fn test_LEX_OP_COV_017_noclobber_redirect() {
        let tokens = lex("echo hi >| file");
        assert!(tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == ">|")));
    }

    #[test]
    fn test_LEX_OP_COV_018_readwrite_redirect() {
        let tokens = lex("exec 3<> file");
        assert!(tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == "<>")));
    }

    #[test]
    fn test_LEX_OP_COV_019_question_glob() {
        let tokens = lex("echo file?.txt");
        // The ? should be tokenized somewhere in the output
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_LEX_OP_COV_020_case_resume_double_semi_ampersand() {
        let tokens = lex(";;&");
        assert!(tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == ";;&")));
    }

    #[test]
    fn test_LEX_OP_COV_021_herestring() {
        let tokens = lex("cat <<< 'hello'");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::HereString(s) if s == "hello")),
            "Expected HereString(\"hello\"), got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_022_heredoc_indented() {
        let tokens = lex("cat <<-EOF\n\t\tline1\n\tEOF\n");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Heredoc { delimiter, .. } if delimiter == "EOF")),
            "Expected Heredoc with delimiter EOF, got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_023_process_substitution_input() {
        let tokens = lex("diff <(ls dir1) file2");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s.starts_with("<("))),
            "Expected process substitution <(...), got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_024_process_substitution_output() {
        let tokens = lex("tee >(grep foo)");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s.starts_with(">("))),
            "Expected process substitution >(...), got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_025_case_fall_through_semicolon_ampersand() {
        let tokens = lex(";&");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s == ";&")),
            "Expected ;& fall-through operator, got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_026_extended_glob_negation() {
        let tokens = lex("!(foo|bar)");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s == "!(foo|bar)")),
            "Expected extended glob !(foo|bar), got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_027_eq_in_double_bracket() {
        let tokens = lex("[[ $x == y ]]");
        assert!(tokens.iter().any(|t| matches!(t, Token::DoubleLeftBracket)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Eq)));
        assert!(tokens
            .iter()
            .any(|t| matches!(t, Token::DoubleRightBracket)));
    }

    #[test]
    fn test_LEX_OP_COV_028_heredoc_basic_delimiter() {
        let tokens = lex("cat <<END\nhello world\nEND\n");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Heredoc { delimiter, content }
                    if delimiter == "END" && content == "hello world")),
            "Expected Heredoc with delimiter END and content 'hello world', got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_029_multiple_operators_and_or_sequence() {
        let tokens = lex("a && b || c");
        assert!(tokens.iter().any(|t| matches!(t, Token::And)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Or)));
    }

    #[test]
    fn test_LEX_OP_COV_030_fd_number_before_append_redirect() {
        let tokens = lex("cmd 2>>file");
        assert!(
            tokens.iter().any(|t| matches!(t, Token::GtGt)),
            "Expected >> append redirect, got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_031_noclobber_after_fd_number() {
        let tokens = lex("cmd 1>| file");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s == ">|")),
            "Expected >| noclobber redirect, got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_032_readwrite_redirect_after_fd() {
        let tokens = lex("exec 3<> /dev/tty");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s == "<>")),
            "Expected <> read-write redirect, got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_033_double_semi_vs_semi_amp_disambiguation() {
        // ;; is case terminator
        let tokens_dsemi = lex(";;");
        assert!(
            tokens_dsemi
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s == ";;")),
            "Expected ;; case terminator, got: {:?}",
            tokens_dsemi
        );

        // ;& is case fall-through
        let tokens_samp = lex(";&");
        assert!(
            tokens_samp
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s == ";&")),
            "Expected ;& fall-through, got: {:?}",
            tokens_samp
        );

        // ;;& is case resume
        let tokens_dsamp = lex(";;&");
        assert!(
            tokens_dsamp
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s == ";;&")),
            "Expected ;;& case resume, got: {:?}",
            tokens_dsamp
        );
    }

    #[test]
    fn test_LEX_OP_COV_034_plus_equals_different_lhs() {
        // Array append
        let tokens = lex("myarr+=(newval)");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s == "+=")),
            "Expected += operator, got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_035_nested_extended_glob_with_inner_parens() {
        let tokens = lex("!(a|(b|c))");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s == "!(a|(b|c))")),
            "Expected nested extended glob !(a|(b|c)), got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_036_not_before_command() {
        let tokens = lex("! grep foo file");
        assert!(
            tokens.iter().any(|t| matches!(t, Token::Not)),
            "Expected ! (Not) token, got: {:?}",
            tokens
        );
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s == "grep")),
            "Expected command identifier 'grep', got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_037_pipe_in_pipeline() {
        let tokens = lex("ls -la | sort | head -5");
        let pipe_count = tokens.iter().filter(|t| matches!(t, Token::Pipe)).count();
        assert_eq!(
            pipe_count, 2,
            "Expected 2 pipe tokens in pipeline, got {}: {:?}",
            pipe_count, tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_038_semicolon_in_different_contexts() {
        // Semicolon as command separator
        let tokens = lex("echo a; echo b");
        let semi_count = tokens
            .iter()
            .filter(|t| matches!(t, Token::Semicolon))
            .count();
        assert_eq!(
            semi_count, 1,
            "Expected 1 semicolon, got {}: {:?}",
            semi_count, tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_039_append_redirect_in_pipeline() {
        let tokens = lex("cmd1 | cmd2 >> outfile");
        assert!(
            tokens.iter().any(|t| matches!(t, Token::Pipe)),
            "Expected pipe, got: {:?}",
            tokens
        );
        assert!(
            tokens.iter().any(|t| matches!(t, Token::GtGt)),
            "Expected >> append redirect, got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_040_mixed_operators_conditional_and_or() {
        let tokens = lex("[[ $x == y ]] && echo yes || echo no");
        assert!(tokens.iter().any(|t| matches!(t, Token::DoubleLeftBracket)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Eq)));
        assert!(tokens
            .iter()
            .any(|t| matches!(t, Token::DoubleRightBracket)));
        assert!(tokens.iter().any(|t| matches!(t, Token::And)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Or)));
    }
}
