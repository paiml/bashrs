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
        if self.input.is_empty() {
            return Ok(vec![Token::Eof]);
        }
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

}

include!("lexer_read_string.rs");
