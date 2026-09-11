//! String literal and exec-context validators. Extracted from pipeline.rs.
use super::ValidationLevel;
use crate::models::error::{RashError, RashResult};
impl super::pipeline::ValidationPipeline {
    pub(crate) fn check_dangerous_patterns(s: &str) -> RashResult<()> {
        let dangerous_patterns = [
            ("&& ", "AND operator detected in string literal"),
            ("|| ", "OR operator detected in string literal"),
            (
                "'; ",
                "Quote escape with semicolon detected in string literal",
            ),
            (
                "\"; ",
                "Quote escape with semicolon detected in string literal",
            ),
            ("<<", "Here-doc syntax detected in string literal"),
            ("eval ", "eval command detected in string literal"),
            ("exec ", "exec command detected in string literal"),
        ];

        for (pattern, message) in &dangerous_patterns {
            if s.contains(pattern) {
                return Err(RashError::ValidationError(format!(
                    "{}: '{}'",
                    message,
                    s.chars().take(50).collect::<String>()
                )));
            }
        }
        Ok(())
    }

    /// GH-294: `$(` and a backtick are only *live* shell metacharacters when
    /// the literal is emitted somewhere other than a lone single-quoted word
    /// — e.g. raw-concatenated inside a double-quoted `__format_concat`
    /// string, or as an `exec()`/`capture()` argument (checked separately by
    /// [`Self::validate_string_literal_in_exec`]).
    ///
    /// A standalone Rust string literal (`let s = "...";`, a bare
    /// `println!("{}", "...")` argument, an array element, ...) always
    /// lowers to `ShellValue::String` and is emitted by
    /// `escape_shell_string` (`rash/src/emitter/escape.rs`): any byte outside
    /// its small "safe unquoted" set — which does not include `$` or `` ` ``
    /// — forces the whole word into single quotes, where both are inert.
    /// Rejecting it before emission on content alone was a false positive
    /// (GH-294); this check exists for the one caller — `__format_concat`
    /// literal parts — where the emitter genuinely embeds the text raw
    /// inside a double-quoted word (`append_concat_part`,
    /// `rash/src/emitter/posix_emit_value.rs`).
    pub(crate) fn check_substitution_patterns(s: &str) -> RashResult<()> {
        let patterns = [
            ("$(", "Command substitution detected in string literal"),
            (
                "`",
                "Backtick command substitution detected in string literal (SC2006)",
            ),
        ];

        for (pattern, message) in &patterns {
            if s.contains(pattern) {
                return Err(RashError::ValidationError(format!(
                    "{}: '{}'",
                    message,
                    s.chars().take(50).collect::<String>()
                )));
            }
        }
        Ok(())
    }

    /// Check for pipe operator injection, excluding table/formatting strings
    pub(crate) fn check_pipe_injection(s: &str) -> RashResult<()> {
        // Issue #94: Skip pipe check for table/formatting strings
        let is_formatting_string = s.chars().filter(|c| *c == '|').count() > 1
            && !s.contains(';')
            && !s.contains("$(")
            && !s.contains("&&");

        if is_formatting_string || !s.contains("| ") {
            return Ok(());
        }

        let trimmed = s.trim();
        if trimmed.starts_with('|') || trimmed.ends_with('|') {
            return Ok(());
        }

        if let Some(pos) = s.find("| ") {
            let after_pipe = &s[pos + 2..].trim_start();
            if !after_pipe.is_empty()
                && after_pipe.chars().next().is_some_and(|c| c.is_alphabetic())
            {
                return Err(RashError::ValidationError(format!(
                    "Pipe operator detected in string literal: '{}'",
                    s.chars().take(50).collect::<String>()
                )));
            }
        }
        Ok(())
    }

    /// Check for newlines followed by dangerous shell commands
    pub(crate) fn check_newline_injection(s: &str) -> RashResult<()> {
        if !s.contains('\n') && !s.contains('\r') {
            return Ok(());
        }

        let dangerous_starts = ["rm ", "curl ", "wget ", "eval ", "exec ", "bash ", "sh "];
        for line in s.split(&['\n', '\r'][..]) {
            let trimmed = line.trim();
            for start in &dangerous_starts {
                if trimmed.starts_with(start) {
                    return Err(RashError::ValidationError(format!(
                        "Newline followed by dangerous command detected: '{}'",
                        trimmed.chars().take(50).collect::<String>()
                    )));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn validate_function_name(&self, name: &str) -> RashResult<()> {
        if name.is_empty() {
            return Err(RashError::ValidationError(
                "Empty function name".to_string(),
            ));
        }

        // Check for reserved shell builtins that cannot be redefined
        // These are POSIX special builtins and common builtins that cause errors
        let reserved_builtins = [
            "break", "continue", "exit", "return", "shift", "trap", "unset", "export", "readonly",
            "set", "times", "exec", "eval", ".", ":", "true", "false", "test", "[",
        ];

        if reserved_builtins.contains(&name) {
            return Err(RashError::ValidationError(format!(
                "Function name '{}' is a reserved shell builtin and cannot be redefined",
                name
            )));
        }

        Ok(())
    }

    pub(crate) fn validate_variable_name(&self, name: &str) -> RashResult<()> {
        if name.is_empty() {
            return Err(RashError::ValidationError(
                "Empty variable name".to_string(),
            ));
        }
        if name.contains(char::is_whitespace) {
            return Err(RashError::ValidationError(format!(
                "Variable name '{name}' contains whitespace"
            )));
        }
        Ok(())
    }

    pub(crate) fn validate_binary_expr(
        &self,
        left: &crate::ast::Expr,
        right: &crate::ast::Expr,
    ) -> RashResult<()> {
        self.validate_expr(left)?;
        self.validate_expr(right)?;
        Ok(())
    }

    pub(crate) fn validate_function_call(
        &self,
        name: &str,
        args: &[crate::ast::Expr],
    ) -> RashResult<()> {
        if name.is_empty() {
            return Err(RashError::ValidationError(
                "Empty function name".to_string(),
            ));
        }

        // Issue #95: exec() arguments ARE meant to be shell commands
        // GH-148: capture() arguments are also shell commands (may contain pipes)
        // Skip shell operator validation (|, &&, ||) for these but keep shellshock protection
        let is_exec_context = name == "exec" || name == "capture";
        // GH-294: `__format_concat` parts are raw-concatenated inside one
        // double-quoted word (see `check_substitution_patterns`), so a
        // literal part there must still be checked for `$(`/backtick.
        let is_concat_context = name == "__format_concat";

        for arg in args {
            if is_exec_context {
                self.validate_expr_in_exec_context(arg)?;
            } else if is_concat_context {
                self.validate_expr_in_concat_context(arg)?;
            } else {
                self.validate_expr(arg)?;
            }
        }
        Ok(())
    }

    /// GH-294: validate a `__format_concat` argument. A literal part is
    /// checked with [`Self::check_substitution_patterns`] in addition to the
    /// normal literal checks, because it is emitted raw inside a
    /// double-quoted word. Anything else recurses normally — its own
    /// emission already decides its quoting.
    pub(crate) fn validate_expr_in_concat_context(
        &self,
        expr: &crate::ast::Expr,
    ) -> RashResult<()> {
        use crate::ast::{restricted::Literal, Expr};

        match expr {
            Expr::Literal(Literal::Str(s)) => self.validate_string_literal_concat(s),
            _ => self.validate_expr(expr),
        }
    }

    /// A `__format_concat` literal part: the normal standalone checks, plus
    /// the `$(`/backtick check that a standalone literal is exempt from.
    pub(crate) fn validate_string_literal_concat(&self, s: &str) -> RashResult<()> {
        self.validate_string_literal(s)?;
        Self::check_substitution_patterns(s)
    }

    /// Validate expression in exec() context - allows shell operators but blocks shellshock
    pub(crate) fn validate_expr_in_exec_context(&self, expr: &crate::ast::Expr) -> RashResult<()> {
        use crate::ast::Expr;

        match expr {
            Expr::Literal(lit) => self.validate_literal_in_exec_context(lit),
            Expr::Variable(name) => self.validate_variable_name(name),
            Expr::Binary { left, right, .. } => {
                self.validate_expr_in_exec_context(left)?;
                self.validate_expr_in_exec_context(right)
            }
            Expr::Unary { operand, .. } => self.validate_expr_in_exec_context(operand),
            Expr::FunctionCall { name, args } => self.validate_function_call(name, args),
            Expr::MethodCall {
                receiver,
                method,
                args,
            } => {
                self.validate_expr_in_exec_context(receiver)?;
                if method.is_empty() {
                    return Err(RashError::ValidationError("Empty method name".to_string()));
                }
                for arg in args {
                    self.validate_expr_in_exec_context(arg)?;
                }
                Ok(())
            }
            Expr::Array(items) => {
                for item in items {
                    self.validate_expr_in_exec_context(item)?;
                }
                Ok(())
            }
            Expr::Index { object, index } => {
                self.validate_expr_in_exec_context(object)?;
                self.validate_expr_in_exec_context(index)
            }
            Expr::Try { expr } => self.validate_expr_in_exec_context(expr),
            Expr::Block(stmts) => self.validate_block_statements(stmts),
            Expr::Range { start, end, .. } => {
                self.validate_expr_in_exec_context(start)?;
                self.validate_expr_in_exec_context(end)
            }
            Expr::PositionalArgs => Ok(()),
        }
    }

    pub(crate) fn validate_literal_in_exec_context(
        &self,
        lit: &crate::ast::restricted::Literal,
    ) -> RashResult<()> {
        use crate::ast::restricted::Literal;

        match lit {
            Literal::Str(s) => self.validate_string_literal_in_exec(s),
            Literal::Bool(_) | Literal::U16(_) | Literal::U32(_) | Literal::I32(_) => Ok(()),
        }
    }

    /// Validate string literal in exec() context - only check for shellshock, allow pipes/operators
    pub(crate) fn validate_string_literal_in_exec(&self, s: &str) -> RashResult<()> {
        if self.level == ValidationLevel::None {
            return Ok(());
        }

        // Still block shellshock-style attacks even in exec context
        if s.contains("() { :; }") {
            return Err(RashError::ValidationError(
                "Shellshock-style function definition detected in exec command".to_string(),
            ));
        }

        // Block command substitution in exec strings (could be injection vector)
        if s.contains("$(") {
            return Err(RashError::ValidationError(format!(
                "Command substitution detected in exec command: '{}'",
                s.chars().take(50).collect::<String>()
            )));
        }

        // Block backtick substitution
        if s.contains('`') {
            return Err(RashError::ValidationError(
                "Backtick command substitution detected in exec command (SC2006)".to_string(),
            ));
        }

        Ok(())
    }
}
