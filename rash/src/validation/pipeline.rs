use super::{ValidationError, ValidationLevel};
use crate::ast::RestrictedAst;
use crate::ir::{shell_ir::CaseArm, ShellIR, ShellValue};
use crate::models::config::Config;
use crate::models::error::{RashError, RashResult};

pub struct ValidationPipeline {
    pub(crate) level: ValidationLevel,
    pub(crate) strict_mode: bool,
}

impl ValidationPipeline {
    pub fn new(config: &Config) -> Self {
        Self {
            level: config.validation_level.unwrap_or_default(),
            strict_mode: config.strict_mode,
        }
    }

    pub fn validate_ast(&self, ast: &RestrictedAst) -> RashResult<()> {
        if self.level == ValidationLevel::None {
            return Ok(());
        }

        // Validate all functions
        for function in &ast.functions {
            // Validate function name is not a shell builtin
            self.validate_function_name(&function.name)?;

            // Validate function body
            for stmt in &function.body {
                self.validate_stmt(stmt)?;
            }
        }
        Ok(())
    }

    pub fn validate_ir(&self, ir: &ShellIR) -> RashResult<()> {
        if self.level == ValidationLevel::None {
            return Ok(());
        }

        self.validate_ir_recursive(ir)
    }

    pub fn validate_output(&self, _shell_script: &str) -> RashResult<()> {
        if self.level == ValidationLevel::None {
            return Ok(());
        }

        #[cfg(debug_assertions)]
        self.verify_with_embedded_rules(_shell_script)?;

        Ok(())
    }

    fn validate_stmt(&self, stmt: &crate::ast::Stmt) -> RashResult<()> {
        use crate::ast::Stmt;

        match stmt {
            Stmt::Let { value, .. } => {
                self.validate_expr(value)?;
            }
            Stmt::Expr(expr) => {
                self.validate_expr(expr)?;
            }
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
                self.validate_expr(condition)?;
                for stmt in then_block {
                    self.validate_stmt(stmt)?;
                }
                if let Some(else_stmts) = else_block {
                    for stmt in else_stmts {
                        self.validate_stmt(stmt)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_expr(&self, expr: &crate::ast::Expr) -> RashResult<()> {
        use crate::ast::Expr;

        match expr {
            Expr::Literal(lit) => self.validate_literal(lit),
            Expr::Variable(name) => self.validate_variable_name(name),
            Expr::Binary { left, right, .. } => self.validate_binary_expr(left, right),
            Expr::Unary { operand, .. } => self.validate_expr(operand),
            Expr::FunctionCall { name, args } => self.validate_function_call(name, args),
            Expr::MethodCall {
                receiver,
                method,
                args,
            } => self.validate_method_call(receiver, method, args),
            Expr::Array(items) => self.validate_array_items(items),
            Expr::Index { object, index } => self.validate_index_expr(object, index),
            Expr::Try { expr } => self.validate_expr(expr),
            Expr::Block(stmts) => self.validate_block_statements(stmts),
            Expr::Range { start, end, .. } => {
                self.validate_expr(start)?;
                self.validate_expr(end)
            }
            Expr::PositionalArgs => {
                // Positional arguments are valid - will be handled by emitter
                Ok(())
            }
        }
    }

    // Helper methods to reduce complexity
    fn validate_literal(&self, lit: &crate::ast::restricted::Literal) -> RashResult<()> {
        use crate::ast::restricted::Literal;

        match lit {
            Literal::Str(s) => self.validate_string_literal(s),
            Literal::Bool(_) | Literal::U16(_) | Literal::U32(_) | Literal::I32(_) => Ok(()),
        }
    }

    fn validate_string_literal(&self, s: &str) -> RashResult<()> {
        if self.level == ValidationLevel::None {
            return Ok(());
        }

        // Special handling for shellshock-style attacks
        if s.contains("() { :; }") {
            return Err(RashError::ValidationError(
                "Shellshock-style function definition detected in string literal".to_string(),
            ));
        }

        // Check for command injection patterns
        Self::check_dangerous_patterns(s)?;

        // Issue #94: Only check for pipe operator if not a formatting string
        Self::check_pipe_injection(s)?;

        // Check for newlines/carriage returns followed by dangerous commands
        Self::check_newline_injection(s)?;

        Ok(())
    }

    /// Check for known dangerous command injection patterns in string literals
    fn check_dangerous_patterns(s: &str) -> RashResult<()> {
        let dangerous_patterns = [
            ("$(", "Command substitution detected in string literal"),
            (
                "`",
                "Backtick command substitution detected in string literal (SC2006)",
            ),
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

    /// Check for pipe operator injection, excluding table/formatting strings
    fn check_pipe_injection(s: &str) -> RashResult<()> {
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
    fn check_newline_injection(s: &str) -> RashResult<()> {
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

    fn validate_function_name(&self, name: &str) -> RashResult<()> {
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

    fn validate_variable_name(&self, name: &str) -> RashResult<()> {
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

    fn validate_binary_expr(
        &self,
        left: &crate::ast::Expr,
        right: &crate::ast::Expr,
    ) -> RashResult<()> {
        self.validate_expr(left)?;
        self.validate_expr(right)?;
        Ok(())
    }

    fn validate_function_call(&self, name: &str, args: &[crate::ast::Expr]) -> RashResult<()> {
        if name.is_empty() {
            return Err(RashError::ValidationError(
                "Empty function name".to_string(),
            ));
        }

        // Issue #95: exec() arguments ARE meant to be shell commands
        // Skip shell operator validation (|, &&, ||) for exec() but keep shellshock protection
        let is_exec_context = name == "exec";

        for arg in args {
            if is_exec_context {
                self.validate_expr_in_exec_context(arg)?;
            } else {
                self.validate_expr(arg)?;
            }
        }
        Ok(())
    }

    /// Validate expression in exec() context - allows shell operators but blocks shellshock
    fn validate_expr_in_exec_context(&self, expr: &crate::ast::Expr) -> RashResult<()> {
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

    fn validate_literal_in_exec_context(
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
    fn validate_string_literal_in_exec(&self, s: &str) -> RashResult<()> {
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

    fn validate_method_call(
        &self,
        receiver: &crate::ast::Expr,
        method: &str,
        args: &[crate::ast::Expr],
    ) -> RashResult<()> {
        if method.is_empty() {
            return Err(RashError::ValidationError("Empty method name".to_string()));
        }
        self.validate_expr(receiver)?;
        for arg in args {
            self.validate_expr(arg)?;
        }
        Ok(())
    }

    fn validate_array_items(&self, items: &[crate::ast::Expr]) -> RashResult<()> {
        for item in items {
            self.validate_expr(item)?;
        }
        Ok(())
    }

    fn validate_index_expr(
        &self,
        object: &crate::ast::Expr,
        index: &crate::ast::Expr,
    ) -> RashResult<()> {
        self.validate_expr(object)?;
        self.validate_expr(index)?;
        Ok(())
    }

    fn validate_block_statements(&self, stmts: &[crate::ast::Stmt]) -> RashResult<()> {
        for stmt in stmts {
            self.validate_stmt(stmt)?;
        }
        Ok(())
    }

    fn validate_ir_recursive(&self, ir: &ShellIR) -> RashResult<()> {
        match ir {
            ShellIR::Let { value, .. } => self.validate_shell_value(value),
            ShellIR::Exec { cmd, .. } => self.validate_exec_args(&cmd.args),
            ShellIR::If {
                test,
                then_branch,
                else_branch,
            } => self.validate_ir_if(test, then_branch, else_branch),
            ShellIR::Sequence(irs) => self.validate_ir_sequence(irs),
            ShellIR::Function { body, .. } => self.validate_ir_recursive(body),
            ShellIR::Exit { .. } | ShellIR::Noop => Ok(()),
            ShellIR::Echo { value } => self.validate_shell_value(value),
            ShellIR::For {
                var,
                start,
                end,
                body,
            } => self.validate_ir_for(var, start, end, body),
            ShellIR::While { condition, body } => {
                self.validate_shell_value(condition)?;
                self.validate_ir_recursive(body)
            }
            ShellIR::Case { scrutinee, arms } => self.validate_ir_case(scrutinee, arms),
            ShellIR::ForIn { var, items, body } => self.validate_ir_for_in(var, items, body),
            ShellIR::Break | ShellIR::Continue => Ok(()),
            ShellIR::Return { value } => {
                if let Some(val) = value {
                    self.validate_shell_value(val)?;
                }
                Ok(())
            }
        }
    }

    fn validate_exec_args(&self, args: &[ShellValue]) -> RashResult<()> {
        for arg in args {
            self.validate_shell_value(arg)?;
        }
        Ok(())
    }

    fn validate_ir_if(
        &self,
        test: &ShellValue,
        then_branch: &ShellIR,
        else_branch: &Option<Box<ShellIR>>,
    ) -> RashResult<()> {
        self.validate_shell_value(test)?;
        self.validate_ir_recursive(then_branch)?;
        if let Some(else_b) = else_branch {
            self.validate_ir_recursive(else_b)?;
        }
        Ok(())
    }

    fn validate_ir_sequence(&self, irs: &[ShellIR]) -> RashResult<()> {
        for ir in irs {
            self.validate_ir_recursive(ir)?;
        }
        Ok(())
    }

    fn validate_ir_for(
        &self,
        var: &str,
        start: &ShellValue,
        end: &ShellValue,
        body: &ShellIR,
    ) -> RashResult<()> {
        if var.is_empty() {
            return Err(RashError::ValidationError(
                "For loop variable name cannot be empty".to_string(),
            ));
        }
        self.validate_shell_value(start)?;
        self.validate_shell_value(end)?;
        self.validate_ir_recursive(body)
    }

    fn validate_ir_case(&self, scrutinee: &ShellValue, arms: &[CaseArm]) -> RashResult<()> {
        self.validate_shell_value(scrutinee)?;
        for arm in arms {
            if let Some(guard) = &arm.guard {
                self.validate_shell_value(guard)?;
            }
            self.validate_ir_recursive(&arm.body)?;
        }
        if arms.is_empty() {
            return Err(RashError::ValidationError(
                "Match expression must have at least one arm".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_ir_for_in(
        &self,
        var: &str,
        items: &[ShellValue],
        body: &ShellIR,
    ) -> RashResult<()> {
        if var.is_empty() {
            return Err(RashError::ValidationError(
                "For-in loop variable name cannot be empty".to_string(),
            ));
        }
        for item in items {
            self.validate_shell_value(item)?;
        }
        self.validate_ir_recursive(body)
    }

    pub(crate) fn validate_shell_value(&self, value: &crate::ir::ShellValue) -> RashResult<()> {
        use crate::ir::ShellValue;

        match value {
            ShellValue::Variable(_name) => {
                if self.level >= ValidationLevel::Minimal {
                    // Variables should generally be quoted in shell
                    // This is a simplified check - real implementation would check context
                }
            }
            ShellValue::CommandSubst(cmd) => {
                if cmd.program.contains('`') && self.level >= ValidationLevel::Minimal {
                    return Err(RashError::ValidationError(
                        "Use $(...) instead of backticks (SC2006)".to_string(),
                    ));
                }
            }
            ShellValue::Concat(parts) => {
                for part in parts {
                    self.validate_shell_value(part)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn validate_expression(&self, expr: &crate::ir::ShellExpression) -> RashResult<()> {
        use crate::ir::ShellExpression;

        match expr {
            ShellExpression::Variable(name, quoted) => {
                if !quoted && self.level >= ValidationLevel::Minimal {
                    return Err(RashError::ValidationError(format!(
                        "Unquoted variable ${name} (SC2086)"
                    )));
                }
            }
            ShellExpression::Command(cmd) => {
                if cmd.contains('`') && self.level >= ValidationLevel::Minimal {
                    return Err(RashError::ValidationError(
                        "Use $(...) instead of backticks (SC2006)".to_string(),
                    ));
                }
            }
            ShellExpression::String(_) => {}
            ShellExpression::Arithmetic(_) => {}
        }
        Ok(())
    }

    #[cfg(debug_assertions)]
    fn verify_with_embedded_rules(&self, script: &str) -> RashResult<()> {
        super::rules::validate_all(script)
    }

    pub fn report_error(&self, error: &ValidationError) -> String {
        if self.strict_mode && error.severity == super::Severity::Error {
            format!("ERROR: {error}")
        } else {
            format!("{error}")
        }
    }

    pub fn should_fail(&self, errors: &[ValidationError]) -> bool {
        if self.strict_mode {
            !errors.is_empty()
        } else {
            errors.iter().any(|e| e.severity == super::Severity::Error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pipeline() -> ValidationPipeline {
        ValidationPipeline {
            level: ValidationLevel::Strict,
            strict_mode: true,
        }
    }

    // Issue #94: Table formatting strings should not be flagged
    #[test]
    fn test_issue_94_table_formatting_ok() {
        let pipeline = create_test_pipeline();
        // Table formatting with multiple pipes is NOT a command injection
        let result =
            pipeline.validate_string_literal("|     whisper.apr      |     whisper.cpp      |");
        assert!(
            result.is_ok(),
            "Table formatting with pipes should NOT be flagged: {:?}",
            result
        );
    }

    #[test]
    fn test_issue_94_pipe_border_ok() {
        let pipeline = create_test_pipeline();
        // Table borders are OK
        let result = pipeline.validate_string_literal("| col1 | col2 | col3 |");
        assert!(result.is_ok(), "Table border should not be flagged");
    }

    #[test]
    fn test_issue_94_single_pipe_border_ok() {
        let pipeline = create_test_pipeline();
        // Single pipe at start or end is OK (table border)
        let result = pipeline.validate_string_literal("| some content");
        assert!(result.is_ok(), "Leading pipe should not be flagged");
    }

    #[test]
    fn test_issue_94_pipe_to_command_flagged() {
        let pipeline = create_test_pipeline();
        // Actual command pipe should still be flagged
        let result = pipeline.validate_string_literal("cat file | rm -rf /");
        assert!(result.is_err(), "Actual command pipe should be flagged");
    }

    #[test]
    fn test_issue_94_semicolon_in_quoted_string_allowed() {
        let pipeline = create_test_pipeline();
        // Bare semicolons inside double-quoted strings are safe (echo "a; b" is not injection)
        let result = pipeline.validate_string_literal("cmd1; cmd2");
        assert!(
            result.is_ok(),
            "Bare semicolons in double-quoted strings are not injection: {:?}",
            result,
        );
    }

    #[test]
    fn test_issue_94_quote_escape_semicolon_flagged() {
        let pipeline = create_test_pipeline();
        // Quote-escape + semicolon IS injection (breaks out of quotes)
        let result = pipeline.validate_string_literal("value'; rm -rf /");
        assert!(result.is_err(), "Quote-escape semicolon should be flagged");
    }

    #[test]
    fn test_issue_94_command_substitution_still_flagged() {
        let pipeline = create_test_pipeline();
        // Command substitution should still be flagged
        let result = pipeline.validate_string_literal("$(dangerous_command)");
        assert!(result.is_err(), "Command substitution should be flagged");
    }

    // Issue #95: exec() arguments should allow shell operators
    #[test]
    fn test_issue_95_exec_with_pipe_allowed() {
        let pipeline = create_test_pipeline();
        // Pipes in exec() are expected - this is the whole point of exec()
        let result = pipeline.validate_string_literal_in_exec("ldd /usr/bin/foo | grep cuda");
        assert!(
            result.is_ok(),
            "Pipe in exec() should be allowed: {:?}",
            result
        );
    }

    #[test]
    fn test_issue_95_exec_with_and_allowed() {
        let pipeline = create_test_pipeline();
        // AND operator in exec() is expected
        let result = pipeline.validate_string_literal_in_exec("cmd1 && cmd2");
        assert!(
            result.is_ok(),
            "AND operator in exec() should be allowed: {:?}",
            result
        );
    }

    #[test]
    fn test_issue_95_exec_with_or_allowed() {
        let pipeline = create_test_pipeline();
        // OR operator in exec() is expected
        let result = pipeline.validate_string_literal_in_exec("cmd1 || cmd2");
        assert!(
            result.is_ok(),
            "OR operator in exec() should be allowed: {:?}",
            result
        );
    }

    #[test]
    fn test_issue_95_exec_with_semicolon_allowed() {
        let pipeline = create_test_pipeline();
        // Semicolon in exec() is expected
        let result = pipeline.validate_string_literal_in_exec("cmd1; cmd2");
        assert!(
            result.is_ok(),
            "Semicolon in exec() should be allowed: {:?}",
            result
        );
    }

    #[test]
    fn test_issue_95_exec_shellshock_still_blocked() {
        let pipeline = create_test_pipeline();
        // Shellshock attacks should STILL be blocked even in exec()
        let result = pipeline.validate_string_literal_in_exec("() { :; }; echo pwned");
        assert!(
            result.is_err(),
            "Shellshock in exec() should still be blocked"
        );
    }

    #[test]
    fn test_issue_95_exec_command_substitution_blocked() {
        let pipeline = create_test_pipeline();
        // Command substitution in exec() is blocked (potential injection vector)
        let result = pipeline.validate_string_literal_in_exec("echo $(whoami)");
        assert!(
            result.is_err(),
            "Command substitution in exec() should be blocked"
        );
    }

    #[test]
    fn test_issue_95_non_exec_pipe_still_flagged() {
        let pipeline = create_test_pipeline();
        // Non-exec strings with pipes should still be flagged
        let result = pipeline.validate_string_literal("cat file | rm -rf /");
        assert!(result.is_err(), "Pipe in non-exec string should be flagged");
    }

    #[test]
    fn test_issue_95_complex_exec_command() {
        let pipeline = create_test_pipeline();
        // Complex shell commands should work in exec()
        let result = pipeline.validate_string_literal_in_exec(
            "ldd /home/noah/.local/bin/main 2>/dev/null | grep -i blas | head -1",
        );
        assert!(
            result.is_ok(),
            "Complex pipeline in exec() should be allowed: {:?}",
            result
        );
    }
}
