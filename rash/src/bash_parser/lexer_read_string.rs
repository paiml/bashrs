impl Lexer {
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

}

include!("lexer_read_operators.rs");
