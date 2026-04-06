impl Lexer {
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

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "lexer_tests_tokenize_sim.rs"]
// FIXME(PMAT-238): mod tests_extracted;
