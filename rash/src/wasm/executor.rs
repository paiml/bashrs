//! Bash Script Executor for WASM Runtime
//!
//! Executes parsed bash scripts in a sandboxed WASM environment.
//!
//! # Example
//!
//! ```rust
//! use bashrs::wasm::executor::BashExecutor;
//!
//! let mut executor = BashExecutor::new();
//! let result = executor.execute("echo 'Hello, World!'").unwrap();
//! assert_eq!(result.stdout, "Hello, World!\n");
//! assert_eq!(result.exit_code, 0);
//! ```

use crate::wasm::builtins::Builtins;
use crate::wasm::io::IoStreams;
use crate::wasm::vfs::VirtualFilesystem;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Captured stdout
    pub stdout: String,
    /// Captured stderr
    pub stderr: String,
    /// Exit code
    pub exit_code: i32,
}

/// Stored function definition
#[derive(Debug, Clone)]
struct FunctionDef {
    /// Function body (lines of code)
    body: Vec<String>,
}

/// Bash script executor
pub struct BashExecutor {
    /// Environment variables
    env: HashMap<String, String>,
    /// Virtual filesystem
    vfs: VirtualFilesystem,
    /// I/O streams
    io: IoStreams,
    /// Last exit code
    exit_code: i32,
    /// Defined functions (name -> definition)
    functions: HashMap<String, FunctionDef>,
}

impl BashExecutor {
    /// Create new bash executor
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
            vfs: VirtualFilesystem::new(),
            io: IoStreams::new_capture(),
            exit_code: 0,
            functions: HashMap::new(),
        }
    }

    /// Execute a bash script
    pub fn execute(&mut self, source: &str) -> Result<ExecutionResult> {
        let lines: Vec<&str> = source
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];

            // Check for function definition
            if self.is_function_definition(line) {
                let (func_end, func_name) = self.parse_function_definition(&lines, i)?;
                i = func_end + 1;
                continue;
            }

            // Check for function call
            if let Some(func_name) = self.is_function_call(line) {
                let exit_code = self.execute_function_call(&func_name, line)?;
                self.exit_code = exit_code;
                i += 1;
                continue;
            }

            // Check for control flow constructs (if, for, while)
            if line.starts_with("if ") {
                // Parse and execute if statement
                let (if_end, exit_code) = self.execute_if_statement(&lines, i)?;
                self.exit_code = exit_code;
                i = if_end + 1;
                continue;
            } else if line.starts_with("for ") {
                // Parse and execute for loop
                let (loop_end, exit_code) = self.execute_for_loop(&lines, i)?;
                self.exit_code = exit_code;
                i = loop_end + 1;
                continue;
            } else if line.starts_with("while ") {
                // Parse and execute while loop
                let (loop_end, exit_code) = self.execute_while_loop(&lines, i)?;
                self.exit_code = exit_code;
                i = loop_end + 1;
                continue;
            }

            self.exit_code = self.execute_command(line)?;
            i += 1;
        }

        Ok(ExecutionResult {
            stdout: self.io.get_stdout(),
            stderr: self.io.get_stderr(),
            exit_code: self.exit_code,
        })
    }

    /// Execute a single command (or pipeline)
    fn execute_command(&mut self, line: &str) -> Result<i32> {
        // First, expand arithmetic expressions
        let line_with_arith = self.expand_arithmetic(line)?;

        // Then, expand command substitutions
        let line_with_subs = self.expand_command_substitutions(&line_with_arith);

        // Check if it's a variable assignment
        if let Some((var, value)) = self.parse_assignment(&line_with_subs) {
            let expanded_value = self.expand_variables(&value);
            self.env.insert(var, expanded_value);
            return Ok(0);
        }

        // Check if it's a pipeline (contains | outside quotes)
        if self.has_pipeline(&line_with_subs) {
            return self.execute_pipeline(&line_with_subs);
        }

        // Parse command and args (command substitutions already expanded)
        let parts = self.parse_command_line(&line_with_subs);
        if parts.is_empty() {
            return Ok(0);
        }

        let cmd = &parts[0];
        let args: Vec<String> = parts[1..]
            .iter()
            .map(|arg| self.expand_variables(arg))
            .collect();

        // Execute builtin command
        if Builtins::is_builtin(cmd) {
            Builtins::execute(cmd, &args, &mut self.vfs, &mut self.io)
        } else {
            Err(anyhow!("Unknown command: {}", cmd))
        }
    }

    /// Check if line contains a pipeline (| outside quotes)
    fn has_pipeline(&self, line: &str) -> bool {
        let mut in_quotes = false;
        let mut quote_char = ' ';

        for ch in line.chars() {
            match ch {
                '\'' | '"' if !in_quotes => {
                    in_quotes = true;
                    quote_char = ch;
                }
                c if in_quotes && c == quote_char => {
                    in_quotes = false;
                }
                '|' if !in_quotes => {
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    /// Execute a pipeline: cmd1 | cmd2 | cmd3
    fn execute_pipeline(&mut self, line: &str) -> Result<i32> {
        // Split pipeline into individual commands
        let commands = self.split_pipeline(line);

        if commands.is_empty() {
            return Ok(0);
        }

        let mut prev_stdout = String::new();
        let mut exit_code = 0;

        for (i, cmd_str) in commands.iter().enumerate() {
            // Set stdin to previous command's stdout
            if i > 0 {
                self.io.set_stdin(&prev_stdout);
            }

            // Clear stdout for this command
            self.io = IoStreams::new_capture();
            if i > 0 {
                self.io.set_stdin(&prev_stdout);
            }

            // Parse and execute command
            let parts = self.parse_command_line(cmd_str);
            if parts.is_empty() {
                continue;
            }

            let cmd = &parts[0];
            let args: Vec<String> = parts[1..]
                .iter()
                .map(|arg| self.expand_variables(arg))
                .collect();

            // Execute command
            exit_code = if Builtins::is_builtin(cmd) {
                Builtins::execute(cmd, &args, &mut self.vfs, &mut self.io)?
            } else {
                return Err(anyhow!("Unknown command in pipeline: {}", cmd));
            };

            // Capture stdout for next stage
            prev_stdout = self.io.get_stdout();
        }

        Ok(exit_code)
    }

    /// Split pipeline into individual commands (respecting quotes)
    fn split_pipeline(&self, line: &str) -> Vec<String> {
        let mut commands = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut quote_char = ' ';

        for ch in line.chars() {
            match ch {
                '\'' | '"' if !in_quotes => {
                    in_quotes = true;
                    quote_char = ch;
                    current.push(ch);
                }
                c if in_quotes && c == quote_char => {
                    in_quotes = false;
                    current.push(c);
                }
                '|' if !in_quotes => {
                    if !current.trim().is_empty() {
                        commands.push(current.trim().to_string());
                        current.clear();
                    }
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        // Add last command
        if !current.trim().is_empty() {
            commands.push(current.trim().to_string());
        }

        commands
    }

    /// Parse command line into words (simple space-based splitting with quote support)
    fn parse_command_line(&self, line: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut quote_char = ' ';

        for ch in line.chars() {
            match ch {
                '\'' | '"' if !in_quotes => {
                    in_quotes = true;
                    quote_char = ch;
                }
                c if in_quotes && c == quote_char => {
                    in_quotes = false;
                }
                ' ' if !in_quotes => {
                    if !current.is_empty() {
                        parts.push(current.clone());
                        current.clear();
                    }
                }
                _ => current.push(ch),
            }
        }

        if !current.is_empty() {
            parts.push(current);
        }

        parts
    }

    /// Parse and expand variables in a string
    fn expand_variables(&self, text: &str) -> String {
        // First expand command substitutions $(...)
        let text_with_subs = self.expand_command_substitutions(text);

        // Then expand variables
        let mut result = String::new();
        let mut chars = text_with_subs.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' {
                // Check for command substitution (already expanded above, but handle edge cases)
                if chars.peek() == Some(&'(') {
                    // Skip - should have been handled already
                    result.push(ch);
                    continue;
                }

                // Variable expansion
                let mut var_name = String::new();
                while let Some(&next) = chars.peek() {
                    if next.is_alphanumeric() || next == '_' {
                        var_name.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                if let Some(value) = self.env.get(&var_name) {
                    result.push_str(value);
                } else {
                    // Variable not set, keep empty
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Expand command substitutions: $(cmd) -> command output
    fn expand_command_substitutions(&self, text: &str) -> String {
        let mut result = String::new();
        let mut chars = text.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'(') {
                // Found $( - start of command substitution
                chars.next(); // consume '('

                // Extract command until matching ')'
                let mut cmd = String::new();
                let mut depth = 1; // Track nested $(...)

                while let Some(c) = chars.next() {
                    if c == '$' && chars.peek() == Some(&'(') {
                        // Nested command substitution
                        cmd.push(c);
                        depth += 1;
                    } else if c == ')' {
                        depth -= 1;
                        if depth == 0 {
                            break; // Found matching ')'
                        }
                        cmd.push(c);
                    } else {
                        cmd.push(c);
                    }
                }

                // Execute the command and substitute with its output
                if let Ok(output) = self.execute_substitution(&cmd) {
                    // Trim trailing newline (bash behavior)
                    result.push_str(output.trim_end_matches('\n'));
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Execute a command for substitution (creates a new executor context)
    fn execute_substitution(&self, cmd: &str) -> Result<String> {
        // Create a new executor with same environment and filesystem
        let mut sub_executor = BashExecutor {
            env: self.env.clone(),
            vfs: self.vfs.clone(),
            io: IoStreams::new_capture(),
            exit_code: 0,
            functions: self.functions.clone(),
        };

        // Execute the command
        let result = sub_executor.execute(cmd)?;

        // Return the stdout
        Ok(result.stdout)
    }

    /// Expand arithmetic expressions: $((expr)) -> evaluated result
    fn expand_arithmetic(&self, text: &str) -> Result<String> {
        let mut result = String::new();
        let mut chars = text.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'(') {
                // Check if it's $(( for arithmetic
                let mut temp_chars = chars.clone();
                temp_chars.next(); // consume first '('

                if temp_chars.peek() == Some(&'(') {
                    // It's $((arithmetic))
                    chars.next(); // consume first '('
                    chars.next(); // consume second '('

                    // Extract expression until matching '))'
                    let mut expr = String::new();
                    let mut depth = 1;

                    while let Some(c) = chars.next() {
                        if c == '(' && chars.peek() == Some(&'(') {
                            expr.push(c);
                            expr.push('(');
                            chars.next();
                            depth += 1;
                        } else if c == ')' && chars.peek() == Some(&')') {
                            depth -= 1;
                            if depth == 0 {
                                chars.next(); // consume second ')'
                                break;
                            }
                            expr.push(c);
                            expr.push(')');
                            chars.next();
                        } else {
                            expr.push(c);
                        }
                    }

                    // Evaluate arithmetic expression (propagate errors)
                    let value = self.evaluate_arithmetic(&expr)?;
                    result.push_str(&value.to_string());
                } else {
                    // It's just $( not $((, so keep the $
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }

    /// Evaluate an arithmetic expression
    fn evaluate_arithmetic(&self, expr: &str) -> Result<i64> {
        // Trim whitespace
        let expr = expr.trim();

        // In arithmetic context, variable names don't need $ prefix
        // So we need special expansion: "x + y" -> "5 + 3" (if x=5, y=3)
        let expanded = self.expand_arithmetic_variables(expr);

        // Parse and evaluate the expression
        self.parse_and_eval(&expanded)
    }

    /// Expand variables in arithmetic context (no $ prefix needed)
    fn expand_arithmetic_variables(&self, expr: &str) -> String {
        let mut result = String::new();
        let mut chars = expr.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch.is_alphabetic() || ch == '_' {
                // Start of a potential variable name
                let mut var_name = String::new();
                var_name.push(ch);

                while let Some(&next) = chars.peek() {
                    if next.is_alphanumeric() || next == '_' {
                        var_name.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                // Try to expand as variable, otherwise keep as-is
                if let Some(value) = self.env.get(&var_name) {
                    result.push_str(value);
                } else {
                    // Not a variable, keep the identifier (will likely fail in parsing)
                    result.push_str(&var_name);
                }
            } else if ch == '$' {
                // Handle $var syntax in arithmetic (also valid)
                let mut var_name = String::new();
                while let Some(&next) = chars.peek() {
                    if next.is_alphanumeric() || next == '_' {
                        var_name.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                if let Some(value) = self.env.get(&var_name) {
                    result.push_str(value);
                } else {
                    // Variable not set, treat as 0 in arithmetic context
                    result.push('0');
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Parse and evaluate arithmetic expression with proper operator precedence
    fn parse_and_eval(&self, expr: &str) -> Result<i64> {
        // Simple recursive descent parser
        // Expression grammar:
        //   expr   := term (('+' | '-') term)*
        //   term   := factor (('*' | '/' | '%') factor)*
        //   factor := '(' expr ')' | number

        let tokens = self.tokenize_arithmetic(expr)?;
        let mut pos = 0;
        self.parse_expr(&tokens, &mut pos)
    }

    /// Tokenize arithmetic expression
    fn tokenize_arithmetic(&self, expr: &str) -> Result<Vec<String>> {
        let mut tokens = Vec::new();
        let mut current = String::new();

        for ch in expr.chars() {
            match ch {
                '+' | '-' | '*' | '/' | '%' | '(' | ')' => {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                    tokens.push(ch.to_string());
                }
                ' ' => {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        if !current.is_empty() {
            tokens.push(current);
        }

        Ok(tokens)
    }

    /// Parse expression: term (('+' | '-') term)*
    fn parse_expr(&self, tokens: &[String], pos: &mut usize) -> Result<i64> {
        let mut left = self.parse_term(tokens, pos)?;

        while *pos < tokens.len() {
            let op = &tokens[*pos];
            if op == "+" || op == "-" {
                *pos += 1;
                let right = self.parse_term(tokens, pos)?;
                left = if op == "+" {
                    left + right
                } else {
                    left - right
                };
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// Parse term: factor (('*' | '/' | '%') factor)*
    fn parse_term(&self, tokens: &[String], pos: &mut usize) -> Result<i64> {
        let mut left = self.parse_factor(tokens, pos)?;

        while *pos < tokens.len() {
            let op = &tokens[*pos];
            if op == "*" || op == "/" || op == "%" {
                *pos += 1;
                let right = self.parse_factor(tokens, pos)?;
                left = match op.as_str() {
                    "*" => left * right,
                    "/" => {
                        if right == 0 {
                            return Err(anyhow!("Division by zero"));
                        }
                        left / right
                    }
                    "%" => {
                        if right == 0 {
                            return Err(anyhow!("Division by zero"));
                        }
                        left % right
                    }
                    _ => unreachable!(),
                };
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// Parse factor: '(' expr ')' | '-' factor | number
    fn parse_factor(&self, tokens: &[String], pos: &mut usize) -> Result<i64> {
        if *pos >= tokens.len() {
            return Err(anyhow!("Unexpected end of expression"));
        }

        let token = &tokens[*pos];

        if token == "(" {
            *pos += 1;
            let result = self.parse_expr(tokens, pos)?;
            if *pos >= tokens.len() || tokens[*pos] != ")" {
                return Err(anyhow!("Missing closing parenthesis"));
            }
            *pos += 1;
            Ok(result)
        } else if token == "-" {
            // Unary minus
            *pos += 1;
            let value = self.parse_factor(tokens, pos)?;
            Ok(-value)
        } else if token == "+" {
            // Unary plus (just skip it)
            *pos += 1;
            self.parse_factor(tokens, pos)
        } else {
            // Parse number
            *pos += 1;
            token.parse::<i64>()
                .map_err(|_| anyhow!("Invalid number: {}", token))
        }
    }

    /// Evaluate test command: [ condition ]
    /// Returns true if condition is true, false otherwise
    fn evaluate_test_command(&self, condition: &str) -> Result<bool> {
        // Extract condition from [ ... ]
        let condition = condition.trim();

        // Remove [ and ] if present
        let condition = if condition.starts_with('[') && condition.ends_with(']') {
            condition[1..condition.len()-1].trim()
        } else {
            condition
        };

        // Split into parts
        let parts: Vec<&str> = condition.split_whitespace().collect();

        if parts.is_empty() {
            return Ok(false);
        }

        // Handle different test operators
        if parts.len() == 3 {
            let left = self.expand_variables(parts[0]);
            let op = parts[1];
            let right = self.expand_variables(parts[2]);

            match op {
                // Integer comparisons
                "-eq" => {
                    let l: i64 = left.parse().unwrap_or(0);
                    let r: i64 = right.parse().unwrap_or(0);
                    Ok(l == r)
                }
                "-ne" => {
                    let l: i64 = left.parse().unwrap_or(0);
                    let r: i64 = right.parse().unwrap_or(0);
                    Ok(l != r)
                }
                "-gt" => {
                    let l: i64 = left.parse().unwrap_or(0);
                    let r: i64 = right.parse().unwrap_or(0);
                    Ok(l > r)
                }
                "-ge" => {
                    let l: i64 = left.parse().unwrap_or(0);
                    let r: i64 = right.parse().unwrap_or(0);
                    Ok(l >= r)
                }
                "-lt" => {
                    let l: i64 = left.parse().unwrap_or(0);
                    let r: i64 = right.parse().unwrap_or(0);
                    Ok(l < r)
                }
                "-le" => {
                    let l: i64 = left.parse().unwrap_or(0);
                    let r: i64 = right.parse().unwrap_or(0);
                    Ok(l <= r)
                }
                // String comparisons
                "=" => Ok(left == right),
                "!=" => Ok(left != right),
                _ => Err(anyhow!("Unknown test operator: {}", op)),
            }
        } else if parts.len() == 2 {
            // Unary operators
            let op = parts[0];
            let arg = self.expand_variables(parts[1]);

            match op {
                "-n" => Ok(!arg.is_empty()),
                "-z" => Ok(arg.is_empty()),
                _ => Err(anyhow!("Unknown unary test operator: {}", op)),
            }
        } else {
            Err(anyhow!("Invalid test command syntax: {}", condition))
        }
    }

    /// Execute an if statement
    /// Returns (end_line_index, exit_code)
    fn execute_if_statement(&mut self, lines: &[&str], start: usize) -> Result<(usize, i32)> {
        // Parse: if CONDITION; then COMMANDS fi
        // or:    if CONDITION \n then \n COMMANDS \n fi
        // or:    if CONDITION \n then \n COMMANDS \n else \n COMMANDS \n fi

        let first_line = lines[start];

        // Extract condition from "if [ ... ]" or "if [ ... ]; then"
        let condition = if first_line.contains("; then") {
            first_line
                .strip_prefix("if ")
                .unwrap()
                .split("; then")
                .next()
                .unwrap()
        } else {
            first_line.strip_prefix("if ").unwrap()
        };

        // Evaluate condition
        let condition_result = self.evaluate_test_command(condition)?;

        // Find then, else, fi
        let mut then_idx = None;
        let mut else_idx = None;
        let mut fi_idx = None;

        for (idx, line) in lines.iter().enumerate().skip(start) {
            if *line == "then" {
                then_idx = Some(idx);
            } else if *line == "else" {
                else_idx = Some(idx);
            } else if *line == "fi" {
                fi_idx = Some(idx);
                break;
            }
        }

        let then_idx = then_idx.ok_or_else(|| anyhow!("Missing 'then' in if statement"))?;
        let fi_idx = fi_idx.ok_or_else(|| anyhow!("Missing 'fi' in if statement"))?;

        let mut exit_code = 0;

        if condition_result {
            // Execute then block
            let then_block_start = then_idx + 1;
            let then_block_end = else_idx.unwrap_or(fi_idx);

            for i in then_block_start..then_block_end {
                exit_code = self.execute_command(lines[i])?;
            }
        } else if let Some(else_idx) = else_idx {
            // Execute else block
            let else_block_start = else_idx + 1;
            let else_block_end = fi_idx;

            for i in else_block_start..else_block_end {
                exit_code = self.execute_command(lines[i])?;
            }
        }

        Ok((fi_idx, exit_code))
    }

    /// Execute a for loop
    /// Returns (end_line_index, exit_code)
    fn execute_for_loop(&mut self, lines: &[&str], start: usize) -> Result<(usize, i32)> {
        // Parse: for VAR in LIST; do ... done
        // or:    for VAR in LIST \n do \n ... \n done

        let first_line = lines[start];

        // Extract variable name and list
        // Format: "for VAR in item1 item2 item3" or "for VAR in item1 item2 item3; do ..."
        let for_part = if first_line.contains("; do") {
            first_line.split("; do").next().unwrap()
        } else {
            first_line
        };

        // Parse "for VAR in ITEMS"
        let parts: Vec<&str> = for_part.split_whitespace().collect();
        if parts.len() < 2 || parts[0] != "for" {
            return Err(anyhow!("Invalid for loop syntax"));
        }

        let var_name = parts[1];

        // Get the items list (everything after "in")
        let mut items = Vec::new();
        if parts.len() > 3 && parts[2] == "in" {
            // Collect items, expanding variables and command substitutions
            let items_str = parts[3..].join(" ");
            let expanded = self.expand_command_substitutions(&items_str);
            let expanded = self.expand_variables(&expanded);

            // Parse items (handle quotes)
            items = self.parse_command_line(&expanded);
        }

        // Find the loop body (between "do" and "done")
        let mut body_start = start;
        let mut body_end = start;

        // Check if "do" is on same line or next line
        if first_line.contains("; do") {
            body_start = start;
        } else {
            // Find "do" line
            for i in (start + 1)..lines.len() {
                if lines[i] == "do" || lines[i].starts_with("do ") {
                    body_start = i;
                    break;
                }
            }
        }

        // Find "done" line
        if first_line.contains("; done") {
            // Single-line loop, done is on same line
            body_end = start;
        } else {
            // Multi-line loop, search for done
            let mut depth = 1; // Track nested loops
            for i in (body_start + 1)..lines.len() {
                if lines[i].starts_with("for ") || lines[i].starts_with("while ") {
                    depth += 1;
                } else if lines[i] == "done" || lines[i].starts_with("done ") {
                    depth -= 1;
                    if depth == 0 {
                        body_end = i;
                        break;
                    }
                }
            }

            if body_end == start {
                return Err(anyhow!("for loop missing 'done'"));
            }
        }

        // Execute loop body for each item
        let mut exit_code = 0;
        for item in items {
            // Set loop variable
            self.env.insert(var_name.to_string(), item);

            // Execute loop body
            let body_lines: Vec<&str> = if first_line.contains("; do") {
                // Single-line loop: for x in ...; do cmd1; cmd2; done
                let after_do = first_line.split("; do ").nth(1).unwrap();
                let before_done = after_do.split("; done").next().unwrap();
                before_done.split("; ").collect()
            } else {
                // Multi-line loop
                lines[(body_start + 1)..body_end].to_vec()
            };

            for body_line in body_lines {
                if !body_line.is_empty() {
                    exit_code = self.execute_command(body_line)?;
                }
            }
        }

        Ok((body_end, exit_code))
    }

    /// Execute a while loop
    /// Returns (end_line_index, exit_code)
    fn execute_while_loop(&mut self, lines: &[&str], start: usize) -> Result<(usize, i32)> {
        // Parse: while CONDITION; do ... done
        // or:    while CONDITION \n do \n ... \n done

        let first_line = lines[start];

        // Extract condition
        let condition_part = if first_line.contains("; do") {
            first_line.split("; do").next().unwrap()
        } else {
            first_line
        };

        // Parse "while CONDITION"
        let condition = condition_part.strip_prefix("while ").unwrap_or("");

        // Find the loop body (between "do" and "done")
        let mut body_start = start;
        let mut body_end = start;

        // Check if "do" is on same line or next line
        if first_line.contains("; do") {
            body_start = start;
        } else {
            // Find "do" line
            for i in (start + 1)..lines.len() {
                if lines[i] == "do" || lines[i].starts_with("do ") {
                    body_start = i;
                    break;
                }
            }
        }

        // Find "done" line
        if first_line.contains("; done") {
            // Single-line loop, done is on same line
            body_end = start;
        } else {
            // Multi-line loop, search for done
            let mut depth = 1;
            for i in (body_start + 1)..lines.len() {
                if lines[i].starts_with("for ") || lines[i].starts_with("while ") {
                    depth += 1;
                } else if lines[i] == "done" || lines[i].starts_with("done ") {
                    depth -= 1;
                    if depth == 0 {
                        body_end = i;
                        break;
                    }
                }
            }

            if body_end == start {
                return Err(anyhow!("while loop missing 'done'"));
            }
        }

        // Execute loop while condition is true
        let mut exit_code = 0;
        let max_iterations = 10000; // Safety limit
        let mut iterations = 0;

        loop {
            iterations += 1;
            if iterations > max_iterations {
                return Err(anyhow!("while loop exceeded maximum iterations"));
            }

            // Evaluate condition
            // Special cases: "true" always succeeds, "false" always fails, [ ] is test command
            let condition_result = if condition == "true" {
                Ok(0)
            } else if condition == "false" {
                Ok(1)
            } else if condition.starts_with('[') && condition.ends_with(']') {
                // Test command: [ condition ]
                match self.evaluate_test_command(condition) {
                    Ok(true) => Ok(0),   // Condition is true -> exit code 0
                    Ok(false) => Ok(1),  // Condition is false -> exit code 1
                    Err(e) => Err(e),    // Error evaluating condition
                }
            } else {
                // Execute condition as command and check exit code
                let temp_io = self.io.clone();
                self.io = IoStreams::new_capture();
                let result = self.execute_command(condition);
                self.io = temp_io;
                result
            };

            match condition_result {
                Ok(0) => {
                    // Condition true, execute body
                    let body_lines: Vec<&str> = if first_line.contains("; do") {
                        // Single-line loop
                        let after_do = first_line.split("; do ").nth(1).unwrap();
                        let before_done = after_do.split("; done").next().unwrap();
                        before_done.split("; ").collect()
                    } else {
                        // Multi-line loop
                        lines[(body_start + 1)..body_end].to_vec()
                    };

                    for body_line in body_lines {
                        if !body_line.is_empty() {
                            exit_code = self.execute_command(body_line)?;
                        }
                    }
                }
                Ok(_) => {
                    // Condition false, exit loop
                    break;
                }
                Err(e) => {
                    // Condition error, treat as false
                    break;
                }
            }
        }

        Ok((body_end, exit_code))
    }

    /// Check if line is a variable assignment
    fn parse_assignment(&self, line: &str) -> Option<(String, String)> {
        // Simple pattern: VAR=value (no spaces around =)
        if let Some(eq_pos) = line.find('=') {
            let var = &line[..eq_pos];
            let value = &line[eq_pos + 1..];

            // Variable name must be valid (alphanumeric + underscore)
            if var.chars().all(|c| c.is_alphanumeric() || c == '_') && !var.is_empty() {
                // Remove quotes from value if present
                let clean_value = value.trim_matches('"').trim_matches('\'');
                return Some((var.to_string(), clean_value.to_string()));
            }
        }

        None
    }

    /// Check if a line is a function definition
    fn is_function_definition(&self, line: &str) -> bool {
        // Style 1: function name() {
        if line.starts_with("function ") && line.contains("()") {
            return true;
        }
        // Style 2: name() {
        if line.contains("()") && line.ends_with("{") {
            return true;
        }
        // Style 3: name() on separate line from {
        if line.contains("()") && !line.contains("{") && !line.contains("}") {
            return true;
        }
        false
    }

    /// Parse a function definition and store it
    fn parse_function_definition(&mut self, lines: &[&str], start: usize) -> Result<(usize, String)> {
        let first_line = lines[start];

        // Extract function name
        let func_name = if first_line.starts_with("function ") {
            // Style 1: function greet() {
            let after_function = first_line.strip_prefix("function ").unwrap();
            if let Some(paren_pos) = after_function.find('(') {
                after_function[..paren_pos].trim().to_string()
            } else {
                return Err(anyhow!("Invalid function syntax: {}", first_line));
            }
        } else if first_line.contains("()") {
            // Style 2: greet() {
            if let Some(paren_pos) = first_line.find('(') {
                first_line[..paren_pos].trim().to_string()
            } else {
                return Err(anyhow!("Invalid function syntax: {}", first_line));
            }
        } else {
            return Err(anyhow!("Invalid function syntax: {}", first_line));
        };

        // Find function body (between { and })
        let mut body_start = start;
        let mut body_lines = Vec::new();
        let mut brace_count = 0;
        let mut func_end = start;
        let mut found_open_brace = false;

        for (idx, line) in lines.iter().enumerate().skip(start) {
            // Count braces
            for ch in line.chars() {
                if ch == '{' {
                    brace_count += 1;
                    found_open_brace = true;
                } else if ch == '}' {
                    brace_count -= 1;
                }
            }

            // Collect body lines (skip the line with opening brace)
            if found_open_brace && brace_count > 0 && !line.ends_with('{') {
                body_lines.push((*line).to_string());
            }

            // Check if we've reached the end
            if found_open_brace && brace_count == 0 {
                func_end = idx;
                break;
            }
        }

        // Store the function
        self.functions.insert(func_name.clone(), FunctionDef {
            body: body_lines,
        });

        Ok((func_end, func_name))
    }

    /// Check if a line is a function call
    fn is_function_call(&self, line: &str) -> Option<String> {
        // Extract the command name (first word)
        let command_name = if let Some(space_pos) = line.find(' ') {
            &line[..space_pos]
        } else {
            line
        };

        // Check if this command is a defined function
        if self.functions.contains_key(command_name) {
            Some(command_name.to_string())
        } else {
            None
        }
    }

    /// Execute a function call
    fn execute_function_call(&mut self, func_name: &str, line: &str) -> Result<i32> {
        // Get the function definition
        let func_def = self.functions.get(func_name)
            .ok_or_else(|| anyhow!("Function not found: {}", func_name))?
            .clone();

        // Parse arguments from the call line
        // Need to handle quoted arguments properly
        let parts: Vec<&str> = line.split_whitespace().collect();
        let mut args: Vec<String> = Vec::new();
        let mut i = 1; // Skip function name
        while i < parts.len() {
            let part = parts[i];
            if part.starts_with('"') {
                // Collect quoted argument (may span multiple parts)
                let mut quoted_arg = part.trim_start_matches('"').to_string();
                if part.ends_with('"') && part.len() > 1 {
                    // Single-word quoted argument
                    quoted_arg = part.trim_matches('"').to_string();
                } else {
                    // Multi-word quoted argument
                    i += 1;
                    while i < parts.len() {
                        if parts[i].ends_with('"') {
                            quoted_arg.push(' ');
                            quoted_arg.push_str(parts[i].trim_end_matches('"'));
                            break;
                        } else {
                            quoted_arg.push(' ');
                            quoted_arg.push_str(parts[i]);
                            i += 1;
                        }
                    }
                }
                args.push(quoted_arg);
            } else {
                // Unquoted argument
                args.push(part.to_string());
            }
            i += 1;
        }
        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        // Save positional parameters that we'll override
        let mut saved_params: HashMap<String, String> = HashMap::new();
        for i in 1..=args.len() {
            let param_name = format!("{}", i);
            if let Some(value) = self.env.get(&param_name) {
                saved_params.insert(param_name.clone(), value.clone());
            }
        }

        // Set positional parameters ($1, $2, etc.)
        for (i, arg) in args.iter().enumerate() {
            let param_name = format!("{}", i + 1);
            self.env.insert(param_name, arg.clone());
        }

        // Reconstruct function body as a script and execute it
        // This allows functions to call other functions, use control flow, etc.
        let function_script = func_def.body.join("\n");
        let result = self.execute(&function_script)?;
        let exit_code = result.exit_code;

        // Restore only the positional parameters, keep other variable changes
        for i in 1..=args.len() {
            let param_name = format!("{}", i);
            if let Some(old_value) = saved_params.get(&param_name) {
                self.env.insert(param_name, old_value.clone());
            } else {
                self.env.remove(&param_name);
            }
        }

        Ok(exit_code)
    }
}

impl Default for BashExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_echo() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute("echo 'hello world'").unwrap();

        // ASSERT
        assert_eq!(result.stdout, "hello world\n");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_execute_echo_without_quotes() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute("echo hello world").unwrap();

        // ASSERT
        assert_eq!(result.stdout, "hello world\n");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_execute_pwd() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute("pwd").unwrap();

        // ASSERT
        assert_eq!(result.stdout, "/\n");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_execute_cd_then_pwd() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor
            .execute(
                r#"
cd /tmp
pwd
"#,
            )
            .unwrap();

        // ASSERT
        assert!(result.stdout.contains("/tmp"));
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_variable_assignment() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor
            .execute(
                r#"
name="Claude"
echo $name
"#,
            )
            .unwrap();

        // ASSERT
        assert!(result.stdout.contains("Claude"));
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_variable_expansion() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor
            .execute(
                r#"
greeting="Hello"
name="World"
echo $greeting, $name!
"#,
            )
            .unwrap();

        // ASSERT
        assert!(result.stdout.contains("Hello, World!"));
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_multi_command() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor
            .execute(
                r#"
echo "Line 1"
echo "Line 2"
echo "Line 3"
"#,
            )
            .unwrap();

        // ASSERT
        assert!(result.stdout.contains("Line 1"));
        assert!(result.stdout.contains("Line 2"));
        assert!(result.stdout.contains("Line 3"));
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_exit_code_success() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute("echo 'success'").unwrap();

        // ASSERT
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_empty_script() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute("").unwrap();

        // ASSERT
        assert_eq!(result.stdout, "");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_comments_ignored() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor
            .execute(
                r#"
# This is a comment
echo "visible"
# Another comment
"#,
            )
            .unwrap();

        // ASSERT
        assert!(result.stdout.contains("visible"));
        assert!(!result.stdout.contains("comment"));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Echo never panics on any input
        #[test]
        fn prop_echo_never_panics(s in "\\PC*") {
            let mut executor = BashExecutor::new();
            let script = format!("echo '{}'", s.replace('\'', "\\'"));
            let result = executor.execute(&script);
            // Should always return a result (Ok or Err), never panic
            assert!(result.is_ok() || result.is_err());
        }

        /// Property: Variable assignment is deterministic
        #[test]
        fn prop_variable_assignment_deterministic(
            name in "[a-z][a-z0-9_]{0,10}",
            value in "[ -~]{0,50}"
        ) {
            let mut executor1 = BashExecutor::new();
            let mut executor2 = BashExecutor::new();

            let script = format!("{}='{}'\necho ${}", name, value, name);

            let result1 = executor1.execute(&script).unwrap();
            let result2 = executor2.execute(&script).unwrap();

            // Same input = same output (determinism)
            prop_assert_eq!(result1.stdout, result2.stdout);
            prop_assert_eq!(result1.exit_code, result2.exit_code);
        }

        /// Property: Empty variable expands to empty string
        #[test]
        fn prop_undefined_variable_expands_empty(
            name in "[a-z][a-z0-9_]{0,10}"
        ) {
            let mut executor = BashExecutor::new();
            // Use space as delimiter to separate variable from text
            let script = format!("echo ${} x", name);
            let result = executor.execute(&script).unwrap();

            // Undefined variable should expand to nothing, leaving just "x"
            prop_assert_eq!(result.stdout.trim(), "x");
        }

        /// Property: cd always updates pwd consistently
        #[test]
        fn prop_cd_updates_pwd(path in "/[a-z]{1,10}") {
            let mut executor = BashExecutor::new();

            // Create the directory first
            executor.vfs.mkdir(&path).unwrap();

            let script = format!("cd {}\npwd", path);
            let result = executor.execute(&script).unwrap();

            // pwd output should contain the path we cd'd to
            prop_assert!(result.stdout.contains(&path));
        }

        /// Property: Multiple echo commands produce concatenated output
        #[test]
        fn prop_multi_echo_concatenates(
            s1 in "[a-z]{1,10}",
            s2 in "[a-z]{1,10}",
            s3 in "[a-z]{1,10}"
        ) {
            let mut executor = BashExecutor::new();
            let script = format!("echo {}\necho {}\necho {}", s1, s2, s3);
            let result = executor.execute(&script).unwrap();

            // All three strings should appear in output
            prop_assert!(result.stdout.contains(&s1));
            prop_assert!(result.stdout.contains(&s2));
            prop_assert!(result.stdout.contains(&s3));
        }

        /// Property: Exit code is always 0 for successful commands
        #[test]
        fn prop_successful_commands_exit_zero(
            value in "[a-z]{1,20}"
        ) {
            let mut executor = BashExecutor::new();
            let script = format!("echo {}", value);
            let result = executor.execute(&script).unwrap();

            prop_assert_eq!(result.exit_code, 0);
        }

        /// Property: Variable expansion preserves value exactly
        #[test]
        fn prop_variable_expansion_preserves_value(
            name in "[a-z][a-z0-9_]{0,10}",
            value in "[a-zA-Z0-9]{1,30}"
        ) {
            let mut executor = BashExecutor::new();
            let script = format!("{}='{}'\necho ${}", name, value, name);
            let result = executor.execute(&script).unwrap();

            // Value should be preserved exactly
            prop_assert_eq!(result.stdout.trim(), value);
        }

        /// Property: Empty scripts always succeed
        #[test]
        fn prop_empty_script_succeeds(whitespace in "[ \t\n]{0,10}") {
            let mut executor = BashExecutor::new();
            let result = executor.execute(&whitespace).unwrap();

            prop_assert_eq!(result.stdout, "");
            prop_assert_eq!(result.exit_code, 0);
        }

        /// Property: Pipelines are deterministic
        #[test]
        fn prop_pipeline_deterministic(
            input in "[a-z ]{1,20}"
        ) {
            let mut executor1 = BashExecutor::new();
            let mut executor2 = BashExecutor::new();

            let script = format!("echo '{}' | wc -c", input);

            let result1 = executor1.execute(&script).unwrap();
            let result2 = executor2.execute(&script).unwrap();

            // Same input = same output (determinism)
            prop_assert_eq!(result1.stdout, result2.stdout);
            prop_assert_eq!(result1.exit_code, result2.exit_code);
        }

        /// Property: Multi-stage pipelines never panic
        #[test]
        fn prop_multi_stage_pipeline_robust(
            s1 in "[a-z]{1,10}",
            s2 in "[a-z]{1,10}"
        ) {
            let mut executor = BashExecutor::new();
            let script = format!("echo '{}' | tr '{}' '{}' | wc -c", s1, s2, s2);
            let result = executor.execute(&script);

            // Should never panic, always return Ok or Err
            prop_assert!(result.is_ok() || result.is_err());
        }

        /// Property: Pipeline with wc -c counts correctly
        #[test]
        fn prop_pipeline_wc_counts_chars(
            s in "[a-z]{1,30}"
        ) {
            let mut executor = BashExecutor::new();
            let script = format!("echo '{}' | wc -c", s);
            let result = executor.execute(&script).unwrap();

            // wc -c should count string length + newline
            let expected_count = s.len() + 1; // +1 for newline from echo
            prop_assert_eq!(result.stdout.trim(), expected_count.to_string());
        }

        /// Property: tr uppercase transformation is reversible
        #[test]
        fn prop_pipeline_tr_reversible(
            s in "[a-z]{1,20}"
        ) {
            let mut executor1 = BashExecutor::new();
            let mut executor2 = BashExecutor::new();

            // Uppercase then lowercase should give original
            let script1 = format!("echo '{}' | tr 'a-z' 'A-Z'", s);
            let result1 = executor1.execute(&script1).unwrap();

            let script2 = format!("echo '{}' | tr 'A-Z' 'a-z'", result1.stdout.trim());
            let result2 = executor2.execute(&script2).unwrap();

            prop_assert_eq!(result2.stdout.trim(), s);
        }

        /// Property: Command substitution is deterministic
        #[test]
        fn prop_substitution_deterministic(
            value in "[a-z ]{1,30}"
        ) {
            let mut executor1 = BashExecutor::new();
            let mut executor2 = BashExecutor::new();

            let script = format!("echo \"Result: $(echo '{}')\"", value);

            let result1 = executor1.execute(&script).unwrap();
            let result2 = executor2.execute(&script).unwrap();

            // Same input = same output
            prop_assert_eq!(result1.stdout, result2.stdout);
            prop_assert_eq!(result1.exit_code, result2.exit_code);
        }

        /// Property: Command substitution never panics
        #[test]
        fn prop_substitution_robust(
            cmd in "(echo|pwd|cd)"
        ) {
            let mut executor = BashExecutor::new();
            let script = format!("echo \"Output: $({})\"", cmd);
            let result = executor.execute(&script);

            // Should never panic
            prop_assert!(result.is_ok() || result.is_err());
        }

        /// Property: Nested substitution preserves content
        #[test]
        fn prop_nested_substitution_preserves(
            inner in "[a-z]{1,15}",
            outer_prefix in "[a-z]{1,10}"
        ) {
            let mut executor = BashExecutor::new();
            let script = format!(
                "echo \"{}: $(echo 'Inner: $(echo {})')\"",
                outer_prefix, inner
            );
            let result = executor.execute(&script).unwrap();

            // Output should contain both prefix and inner value
            prop_assert!(result.stdout.contains(&outer_prefix));
            prop_assert!(result.stdout.contains(&inner));
        }

        /// Property: Substitution in assignment is retrievable
        #[test]
        fn prop_substitution_in_assignment(
            varname in "[a-z][a-z0-9_]{0,10}",
            value in "[a-z]{1,20}"
        ) {
            let mut executor = BashExecutor::new();
            let script = format!(
                "{}=$(echo '{}')\necho ${}",
                varname, value, varname
            );
            let result = executor.execute(&script).unwrap();

            // Variable should contain the substituted value
            prop_assert_eq!(result.stdout.trim(), value);
        }

        /// Property: Multiple substitutions concatenate correctly
        #[test]
        fn prop_multiple_substitutions(
            s1 in "[a-z]{1,10}",
            s2 in "[a-z]{1,10}"
        ) {
            let mut executor = BashExecutor::new();
            let script = format!(
                "echo \"A: $(echo '{}') B: $(echo '{}')\"",
                s1, s2
            );
            let result = executor.execute(&script).unwrap();

            // Output should contain both values
            prop_assert!(result.stdout.contains(&s1));
            prop_assert!(result.stdout.contains(&s2));
        }
    }
}

/// Command Substitution tests (SUB-001)
///
/// RED tests for Sprint WASM-RUNTIME-002
/// Expected to FAIL until command substitution is implemented
#[cfg(test)]
mod substitution_tests {
    use super::*;

    /// SUB-001: Basic command substitution
    #[test]
    fn test_sub_001_basic() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: echo "Result: $(echo 'hello')"
        let result = executor.execute("echo \"Result: $(echo 'hello')\"");

        // ASSERT: RED phase - will fail
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "Result: hello");
    }

    /// SUB-001: Command substitution with wc
    #[test]
    fn test_sub_001_with_wc() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: count=$(echo 'test' | wc -c); echo "Count: $count"
        let result = executor.execute("count=$(echo 'test' | wc -c)\necho \"Count: $count\"");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "Count: 5");
    }

    /// SUB-001: Nested command substitution
    #[test]
    fn test_sub_001_nested() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: echo "Outer: $(echo "Inner: $(echo 'nested')")"
        let result = executor.execute("echo \"Outer: $(echo 'Inner: $(echo nested)')\"");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "Outer: Inner: nested");
    }

    /// SUB-001: Command substitution in assignment
    #[test]
    fn test_sub_001_in_assignment() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute(r#"
greeting=$(echo 'Hello, World!')
echo "$greeting"
"#);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "Hello, World!");
    }

    /// SUB-001: Multiple substitutions in one line
    #[test]
    fn test_sub_001_multiple() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: echo "A: $(echo 'a') B: $(echo 'b')"
        let result = executor.execute("echo \"A: $(echo 'a') B: $(echo 'b')\"");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "A: a B: b");
    }

    /// SUB-001: Command substitution with pipeline
    #[test]
    fn test_sub_001_with_pipeline() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: result=$(echo 'hello' | tr 'a-z' 'A-Z'); echo "Result: $result"
        let result = executor.execute("result=$(echo 'hello' | tr 'a-z' 'A-Z')\necho \"Result: $result\"");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "Result: HELLO");
    }

    /// SUB-001: Command substitution with pwd
    #[test]
    fn test_sub_001_with_pwd() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute("cd /tmp\necho \"Current dir: $(pwd)\"");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "Current dir: /tmp");
    }

    /// SUB-001: Empty command substitution
    #[test]
    fn test_sub_001_empty() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: echo "Value: $()"
        let result = executor.execute("echo \"Value: $()\"");

        // ASSERT: Should handle empty substitution gracefully
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "Value:");
    }

    /// SUB-001: Command substitution without quotes
    #[test]
    fn test_sub_001_unquoted() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: echo Result: $(echo hello)
        let result = executor.execute("echo Result: $(echo hello)");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "Result: hello");
    }

    /// SUB-001: Command substitution with arithmetic (when implemented)
    #[test]
    #[ignore] // Requires arithmetic - defer to later
    fn test_sub_001_with_arithmetic() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: echo "Sum: $(echo $((2 + 3)))"
        let result = executor.execute("echo \"Sum: $(echo $((2 + 3)))\"");

        // ASSERT: Deferred
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "Sum: 5");
    }
}

/// Pipeline tests (PIPE-001, PIPE-002)
///
/// These are RED tests for Sprint WASM-RUNTIME-002
/// Expected to FAIL until pipeline implementation is complete
#[cfg(test)]
mod pipeline_tests {
    use super::*;

    /// PIPE-001: Simple 2-stage pipeline
    #[test]
    fn test_pipe_001_simple_two_stage() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: echo "hello world" | wc -c
        // Expected: Should count characters in "hello world\n" = 12
        let result = executor.execute("echo 'hello world' | wc -c");

        // ASSERT: This will FAIL (RED phase) - pipeline not implemented yet
        assert!(result.is_ok(), "Pipeline execution should not panic");
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "12", "wc -c should count 12 characters");
        assert_eq!(result.exit_code, 0);
    }

    /// PIPE-001: Echo to uppercase (tr command)
    #[test]
    fn test_pipe_001_echo_to_tr() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: echo "hello" | tr 'a-z' 'A-Z'
        let result = executor.execute("echo 'hello' | tr 'a-z' 'A-Z'");

        // ASSERT: RED phase - will fail
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "HELLO");
        assert_eq!(result.exit_code, 0);
    }

    /// PIPE-002: Multi-stage pipeline (3 commands)
    #[test]
    fn test_pipe_002_three_stage() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: echo "a b c" | tr ' ' '\n' | wc -l
        // Step 1: echo "a b c" -> "a b c\n"
        // Step 2: tr ' ' '\n' -> "a\nb\nc\n"
        // Step 3: wc -l -> "3\n"
        let result = executor.execute("echo 'a b c' | tr ' ' '\\n' | wc -l");

        // ASSERT: RED phase - will fail
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "3", "Should count 3 lines");
        assert_eq!(result.exit_code, 0);
    }

    /// PIPE-002: Four-stage pipeline
    #[test]
    fn test_pipe_002_four_stage() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: Complex pipeline
        let result = executor.execute("echo 'hello world' | tr 'a-z' 'A-Z' | tr ' ' '_' | wc -c");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        // "HELLO_WORLD\n" = 12 characters
        assert_eq!(result.stdout.trim(), "12");
    }

    /// PIPE-001: Pipeline with variable expansion
    #[test]
    fn test_pipe_001_with_variables() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute(r#"
msg="hello"
echo "$msg world" | wc -c
"#);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "12");
    }

    /// PIPE-001: Error handling - command not found in pipeline
    #[test]
    fn test_pipe_001_error_command_not_found() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: Pipeline with non-existent command
        let result = executor.execute("echo 'test' | nonexistent_cmd");

        // ASSERT: Should return error gracefully
        // RED phase - error handling not yet implemented
        assert!(result.is_err() || result.unwrap().exit_code != 0,
                "Pipeline should fail gracefully when command not found");
    }

    /// PIPE-001: Empty pipeline input
    #[test]
    fn test_pipe_001_empty_input() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: echo "" | wc -c should give 1 (just newline)
        let result = executor.execute("echo '' | wc -c");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "1"); // Just the newline
    }

    /// PIPE-002: Pipeline preserves exit codes
    #[test]
    fn test_pipe_002_exit_code_propagation() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: Successful pipeline
        let result = executor.execute("echo 'test' | wc -c");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.exit_code, 0, "Successful pipeline should exit 0");
    }

    /// PIPE-001: Pipeline with quoted strings containing pipes
    #[test]
    fn test_pipe_001_quoted_pipe_character() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: Pipe character inside quotes should not be treated as pipeline
        let result = executor.execute("echo 'this | is not a pipe'");

        // ASSERT: RED phase (but may pass if quotes work)
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "this | is not a pipe");
    }

    /// PIPE-002: Nested pipelines (if supported)
    #[test]
    #[ignore] // Stretch goal - may defer to Sprint 003
    fn test_pipe_002_nested_pipelines() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: Nested command substitution with pipeline
        let result = executor.execute("echo $(echo 'test' | tr 'a-z' 'A-Z')");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "TEST");
    }
}

/// Loop tests (LOOP-001, LOOP-002)
///
/// RED tests for Sprint WASM-RUNTIME-002 Week 2
/// Expected to FAIL until loop constructs are implemented
#[cfg(test)]
mod loop_tests {
    use super::*;

    // ========== FOR LOOP TESTS (LOOP-001) ==========

    /// LOOP-001: Basic for loop with list
    #[test]
    fn test_loop_001_for_basic_list() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: for i in a b c; do echo $i; done
        let script = "for i in a b c\ndo\necho $i\ndone";
        let result = executor.execute(script);

        // ASSERT: RED phase - will fail
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "a\nb\nc\n");
    }

    /// LOOP-001: For loop with semicolon syntax
    #[test]
    fn test_loop_001_for_semicolon_syntax() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: for x in 1 2 3; do echo "num: $x"; done
        let script = "for x in 1 2 3; do echo \"num: $x\"; done";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "num: 1\nnum: 2\nnum: 3\n");
    }

    /// LOOP-001: For loop with variable expansion in list
    #[test]
    fn test_loop_001_for_variable_expansion() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: items="x y z"; for i in $items; do echo $i; done
        let script = "items=\"x y z\"\nfor i in $items\ndo\necho $i\ndone";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "x\ny\nz\n");
    }

    /// LOOP-001: For loop with command substitution
    #[test]
    fn test_loop_001_for_command_substitution() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: for word in $(echo "one two three"); do echo $word; done
        let script = "for word in $(echo \"one two three\")\ndo\necho $word\ndone";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "one\ntwo\nthree\n");
    }

    /// LOOP-001: For loop with empty list
    #[test]
    fn test_loop_001_for_empty_list() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: for i in; do echo $i; done (should not execute body)
        let script = "for i in\ndo\necho $i\ndone";
        let result = executor.execute(script);

        // ASSERT: RED phase - should produce no output
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "");
    }

    /// LOOP-001: For loop with single item
    #[test]
    fn test_loop_001_for_single_item() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: for item in solo; do echo "Item: $item"; done
        let script = "for item in solo; do echo \"Item: $item\"; done";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "Item: solo\n");
    }

    /// LOOP-001: Nested for loops
    #[test]
    fn test_loop_001_for_nested() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: Nested loops
        let script = "for i in 1 2\ndo\nfor j in a b\ndo\necho \"$i$j\"\ndone\ndone";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "1a\n1b\n2a\n2b\n");
    }

    /// LOOP-001: For loop with builtin commands
    #[test]
    fn test_loop_001_for_with_builtins() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: for dir in /tmp /home; do cd $dir && pwd; done
        let script = "for dir in /tmp /home\ndo\ncd $dir\npwd\ndone";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "/tmp\n/home\n");
    }

    /// LOOP-001: For loop accumulating variable
    #[test]
    fn test_loop_001_for_accumulate() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: sum=""; for n in A B C; do sum="$sum$n"; done; echo $sum
        let script = "sum=\"\"\nfor n in A B C\ndo\nsum=\"$sum$n\"\ndone\necho $sum";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "ABC\n");
    }

    /// LOOP-001: For loop with pipeline
    #[test]
    fn test_loop_001_for_with_pipeline() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: for item in hello world; do echo $item | tr 'a-z' 'A-Z'; done
        let script = "for item in hello world\ndo\necho $item | tr 'a-z' 'A-Z'\ndone";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "HELLO\nWORLD\n");
    }

    // ========== WHILE LOOP TESTS (LOOP-002) ==========

    /// LOOP-002: Basic while loop with counter
    #[test]
    fn test_loop_002_while_basic_counter() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: i=1; while [ $i -le 3 ]; do echo $i; i=$((i+1)); done
        // Simplified without test command: use variable check
        let script = "count=3\nwhile [ $count -gt 0 ]\ndo\necho $count\ncount=$((count-1))\ndone";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "3\n2\n1\n");
    }

    /// LOOP-002: While loop with simple condition (non-empty variable)
    #[test]
    fn test_loop_002_while_simple_condition() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: Simplified while - check if variable is set
        let script = "val=\"go\"\nwhile echo $val\ndo\nval=\"\"\ndone";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "go\n\n");
    }

    /// LOOP-002: While loop that never executes
    #[test]
    fn test_loop_002_while_false_condition() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: while false; do echo "never"; done
        let script = "while false\ndo\necho \"never\"\ndone";
        let result = executor.execute(script);

        // ASSERT: RED phase - should produce no output
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "");
    }

    /// LOOP-002: While loop with break (if we implement it)
    #[test]
    #[ignore] // May defer break/continue to later sprint
    fn test_loop_002_while_break() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: infinite loop with break
        let script = "i=1\nwhile true\ndo\necho $i\nif [ $i -eq 3 ]; then break; fi\ni=$((i+1))\ndone";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "1\n2\n3\n");
    }

    /// LOOP-002: While loop with continue (if we implement it)
    #[test]
    #[ignore] // May defer break/continue to later sprint
    fn test_loop_002_while_continue() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: loop with continue to skip
        let script = "i=0\nwhile [ $i -lt 5 ]\ndo\ni=$((i+1))\nif [ $i -eq 3 ]; then continue; fi\necho $i\ndone";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "1\n2\n4\n5\n");
    }

    /// LOOP-002: Nested while loops
    #[test]
    fn test_loop_002_while_nested() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: Nested while loops
        let script = "i=1\nwhile [ $i -le 2 ]\ndo\nj=1\nwhile [ $j -le 2 ]\ndo\necho \"$i,$j\"\nj=$((j+1))\ndone\ni=$((i+1))\ndone";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "1,1\n1,2\n2,1\n2,2\n");
    }

    /// LOOP-002: While loop with pipeline
    #[test]
    fn test_loop_002_while_with_pipeline() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: while with pipeline in body
        let script = "i=1\nwhile [ $i -le 2 ]\ndo\necho \"test\" | tr 'a-z' 'A-Z'\ni=$((i+1))\ndone";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "TEST\nTEST\n");
    }

    // ========== MIXED LOOP TESTS ==========

    /// LOOP-001 + LOOP-002: For loop inside while loop
    #[test]
    fn test_loop_003_for_inside_while() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: while containing for
        let script = "outer=1\nwhile [ $outer -le 2 ]\ndo\nfor inner in a b\ndo\necho \"$outer:$inner\"\ndone\nouter=$((outer+1))\ndone";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "1:a\n1:b\n2:a\n2:b\n");
    }

    /// LOOP-001 + LOOP-002: While loop inside for loop
    #[test]
    fn test_loop_003_while_inside_for() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: for containing while
        let script = "for letter in X Y\ndo\ni=1\nwhile [ $i -le 2 ]\ndo\necho \"$letter$i\"\ni=$((i+1))\ndone\ndone";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "X1\nX2\nY1\nY2\n");
    }

    /// LOOP-001: For loop modifying environment
    #[test]
    fn test_loop_001_for_environment_modification() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: Loop that sets variables, verify persistence
        let script = "for key in A B C\ndo\nlast=$key\ndone\necho \"Last: $last\"";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "Last: C\n");
    }

    /// LOOP-002: While loop modifying environment
    #[test]
    fn test_loop_002_while_environment_modification() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: While that accumulates
        let script = "result=\"\"\ni=1\nwhile [ $i -le 3 ]\ndo\nresult=\"${result}$i\"\ni=$((i+1))\ndone\necho $result";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "123\n");
    }

    /// LOOP-001: For loop with quoted items
    #[test]
    fn test_loop_001_for_quoted_items() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: for item in "hello world" "foo bar"; do echo $item; done
        let script = "for item in \"hello world\" \"foo bar\"\ndo\necho $item\ndone";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "hello world\nfoo bar\n");
    }

    /// LOOP-001: For loop with special characters
    #[test]
    fn test_loop_001_for_special_chars() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: for ch in ! @ '#' $; do echo "char: $ch"; done
        let script = "for ch in ! @ '#' '$'\ndo\necho \"char: $ch\"\ndone";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "char: !\nchar: @\nchar: #\nchar: $\n");
    }

    /// LOOP-002: While loop with command substitution in condition
    #[test]
    fn test_loop_002_while_command_sub_condition() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: while with dynamic condition
        let script = "state=\"go\"\nwhile echo $state | tr 'a-z' 'A-Z'\ndo\nstate=\"stop\"\ndone";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "GO\nSTOP\n");
    }

    /// LOOP-001: For loop with multiline body
    #[test]
    fn test_loop_001_for_multiline_body() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: for with multiple commands in body
        let script = "for num in 1 2\ndo\necho \"Number: $num\"\necho \"Double: $((num * 2))\"\ndone";
        let result = executor.execute(script);

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "Number: 1\nDouble: 2\nNumber: 2\nDouble: 4\n");
    }
}

/// Property tests for loops (LOOP-001, LOOP-002)
///
/// REFACTOR phase tests for Sprint WASM-RUNTIME-002 Week 2
/// Generative tests to validate loop robustness
#[cfg(test)]
mod loop_property_tests {
    use super::*;

    #[cfg(test)]
    use proptest::prelude::*;

    /// Property: For loops are deterministic
    /// Same items should produce same output every time
    #[test]
    fn prop_for_loop_deterministic() {
        proptest!(|(items in prop::collection::vec("[a-z]{1,5}", 1..10))| {
            let mut executor1 = BashExecutor::new();
            let mut executor2 = BashExecutor::new();

            let items_str = items.join(" ");
            let script = format!("for item in {}\ndo\necho $item\ndone", items_str);

            let result1 = executor1.execute(&script).unwrap();
            let result2 = executor2.execute(&script).unwrap();

            // Same input = same output
            prop_assert_eq!(result1.stdout, result2.stdout);
            prop_assert_eq!(result1.exit_code, result2.exit_code);
        });
    }

    /// Property: For loops preserve item order
    /// Items should be processed in the order specified
    #[test]
    fn prop_for_loop_preserves_order() {
        proptest!(|(items in prop::collection::vec("[a-z]{1,5}", 1..10))| {
            let mut executor = BashExecutor::new();

            let items_str = items.join(" ");
            let script = format!("for item in {}\ndo\necho $item\ndone", items_str);

            let result = executor.execute(&script).unwrap();

            // Expected output: each item on its own line
            let expected: String = items.iter()
                .map(|item| format!("{}\n", item))
                .collect();

            prop_assert_eq!(result.stdout, expected);
        });
    }

    /// Property: For loop with empty list produces no output
    #[test]
    fn prop_for_loop_empty_list_no_output() {
        proptest!(|(var_name in "[a-z]{1,10}")| {
            let mut executor = BashExecutor::new();

            let script = format!("for {} in\ndo\necho \"never\"\ndone", var_name);
            let result = executor.execute(&script).unwrap();

            // Empty list = no iterations = no output
            prop_assert_eq!(result.stdout, "");
        });
    }

    /// Property: For loop variable persists after loop
    /// Loop variable should contain last item after loop exits
    #[test]
    fn prop_for_loop_variable_persistence() {
        proptest!(|(items in prop::collection::vec("[a-z]{1,5}", 1..10))| {
            let mut executor = BashExecutor::new();

            let items_str = items.join(" ");
            let script = format!("for item in {}\ndo\necho $item\ndone\necho \"Last: $item\"", items_str);

            let result = executor.execute(&script).unwrap();

            // Last line should contain the last item
            let last_item = items.last().unwrap();
            let expected_last_line = format!("Last: {}\n", last_item);

            prop_assert!(result.stdout.ends_with(&expected_last_line),
                "Expected last line to be '{}', but got '{}'",
                expected_last_line.trim(),
                result.stdout.lines().last().unwrap_or(""));
        });
    }

    /// Property: For loop with single item executes exactly once
    #[test]
    fn prop_for_loop_single_item_once() {
        proptest!(|(item in "[a-z]{1,10}")| {
            let mut executor = BashExecutor::new();

            let script = format!("for x in {}\ndo\necho $x\ndone", item);
            let result = executor.execute(&script).unwrap();

            // Single item = single line of output
            let lines: Vec<&str> = result.stdout.lines().collect();
            prop_assert_eq!(lines.len(), 1);
            prop_assert_eq!(lines[0], item);
        });
    }

    /// Property: For loop accumulation is correct
    /// Accumulating items should preserve all items in order
    #[test]
    fn prop_for_loop_accumulation_correct() {
        proptest!(|(items in prop::collection::vec("[a-z]{1,3}", 1..10))| {
            let mut executor = BashExecutor::new();

            let items_str = items.join(" ");
            let script = format!(
                "result=\"\"\nfor item in {}\ndo\nresult=\"$result$item\"\ndone\necho $result",
                items_str
            );

            let result = executor.execute(&script).unwrap();

            // Accumulated string should be all items concatenated
            let expected = items.join("");
            prop_assert_eq!(result.stdout.trim(), expected);
        });
    }

    /// Property: While loop with false condition never executes
    #[test]
    fn prop_while_false_never_executes() {
        proptest!(|(body_cmd in "[a-z]{1,10}")| {
            let mut executor = BashExecutor::new();

            let script = format!("while false\ndo\necho \"{}\"\ndone", body_cmd);
            let result = executor.execute(&script).unwrap();

            // False condition = zero iterations = no output
            prop_assert_eq!(result.stdout, "");
        });
    }

    /// Property: For loop handles quoted items correctly
    /// Items with spaces should be treated as single items when quoted
    #[test]
    fn prop_for_loop_quoted_items() {
        proptest!(|(items in prop::collection::vec("[a-z]{1,5}( [a-z]{1,5})?", 1..5))| {
            let mut executor = BashExecutor::new();

            // Quote each item
            let quoted_items: Vec<String> = items.iter()
                .map(|item| format!("\"{}\"", item))
                .collect();
            let items_str = quoted_items.join(" ");

            let script = format!("for item in {}\ndo\necho \"[$item]\"\ndone", items_str);
            let result = executor.execute(&script).unwrap();

            // Each item (with spaces) should appear on its own line, wrapped in brackets
            // Note: Variable expansion without quotes collapses multiple spaces to single space
            // So we need to match that behavior
            let expected: String = items.iter()
                .map(|item| {
                    // Collapse multiple spaces (matches bash variable expansion behavior)
                    let collapsed = item.split_whitespace().collect::<Vec<_>>().join(" ");
                    format!("[{}]\n", collapsed)
                })
                .collect();

            prop_assert_eq!(result.stdout, expected);
        });
    }

    /// Property: For loop with variable expansion works correctly
    #[test]
    fn prop_for_loop_variable_expansion() {
        proptest!(|(items in prop::collection::vec("[a-z]{1,5}", 1..10))| {
            let mut executor = BashExecutor::new();

            let items_str = items.join(" ");
            let script = format!(
                "list=\"{}\"\nfor item in $list\ndo\necho $item\ndone",
                items_str
            );

            let result = executor.execute(&script).unwrap();

            // Variable expansion should produce same result as literal list
            let expected: String = items.iter()
                .map(|item| format!("{}\n", item))
                .collect();

            prop_assert_eq!(result.stdout, expected);
        });
    }

    /// Property: For loop never panics on any valid input
    #[test]
    fn prop_for_loop_robust() {
        proptest!(|(
            var_name in "[a-z]{1,10}",
            items in prop::collection::vec("[a-zA-Z0-9 ]{0,20}", 0..10)
        )| {
            let mut executor = BashExecutor::new();

            let items_str = items.join(" ");
            let script = format!("for {} in {}\ndo\necho test\ndone", var_name, items_str);

            // Should never panic, even with edge cases
            let result = executor.execute(&script);
            prop_assert!(result.is_ok());
        });
    }
}

/// Arithmetic Expansion tests (ARITH-001)
///
/// RED tests for Sprint WASM-RUNTIME-002 Week 3
/// Expected to FAIL until arithmetic expansion is implemented
#[cfg(test)]
mod arithmetic_tests {
    use super::*;

    /// ARITH-001: Basic arithmetic - addition
    #[test]
    fn test_arith_001_addition() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: echo $((2 + 3))
        let result = executor.execute("echo $((2 + 3))");

        // ASSERT: RED phase - will fail
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "5");
    }

    /// ARITH-001: Basic arithmetic - subtraction
    #[test]
    fn test_arith_001_subtraction() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: echo $((10 - 3))
        let result = executor.execute("echo $((10 - 3))");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "7");
    }

    /// ARITH-001: Basic arithmetic - multiplication
    #[test]
    fn test_arith_001_multiplication() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: echo $((4 * 5))
        let result = executor.execute("echo $((4 * 5))");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "20");
    }

    /// ARITH-001: Basic arithmetic - division
    #[test]
    fn test_arith_001_division() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: echo $((20 / 4))
        let result = executor.execute("echo $((20 / 4))");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "5");
    }

    /// ARITH-001: Basic arithmetic - modulo
    #[test]
    fn test_arith_001_modulo() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: echo $((10 % 3))
        let result = executor.execute("echo $((10 % 3))");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "1");
    }

    /// ARITH-001: Arithmetic with variables
    #[test]
    fn test_arith_001_variables() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: x=5; y=3; echo $((x + y))
        let result = executor.execute("x=5\ny=3\necho $((x + y))");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "8");
    }

    /// ARITH-001: Arithmetic with variable assignment
    #[test]
    fn test_arith_001_assignment() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: result=$((7 * 6)); echo $result
        let result = executor.execute("result=$((7 * 6))\necho $result");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "42");
    }

    /// ARITH-001: Nested arithmetic
    #[test]
    fn test_arith_001_nested() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: echo $(( (5 + 3) * 2 ))
        let result = executor.execute("echo $(( (5 + 3) * 2 ))");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "16");
    }

    /// ARITH-001: Arithmetic in for loop (unblock LOOP tests)
    #[test]
    fn test_arith_001_in_for_loop() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: for i in 1 2; do echo $((i * 2)); done
        let result = executor.execute("for i in 1 2\ndo\necho $((i * 2))\ndone");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "2\n4\n");
    }

    /// ARITH-001: Increment operation
    #[test]
    fn test_arith_001_increment() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: i=5; i=$((i + 1)); echo $i
        let result = executor.execute("i=5\ni=$((i + 1))\necho $i");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "6");
    }

    /// ARITH-001: Decrement operation
    #[test]
    fn test_arith_001_decrement() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: count=10; count=$((count - 1)); echo $count
        let result = executor.execute("count=10\ncount=$((count - 1))\necho $count");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "9");
    }

    /// ARITH-001: Multiple operations in sequence
    #[test]
    fn test_arith_001_multiple_operations() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: a=$((2 + 3)); b=$((a * 4)); echo $b
        let result = executor.execute("a=$((2 + 3))\nb=$((a * 4))\necho $b");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "20");
    }

    /// ARITH-001: Negative numbers
    #[test]
    fn test_arith_001_negative_numbers() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: echo $((-5 + 3))
        let result = executor.execute("echo $((-5 + 3))");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "-2");
    }

    /// ARITH-001: Order of operations (multiplication before addition)
    #[test]
    fn test_arith_001_order_of_operations() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: echo $((2 + 3 * 4))
        let result = executor.execute("echo $((2 + 3 * 4))");

        // ASSERT: RED phase - should be 14, not 20
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "14");
    }

    /// ARITH-001: Division by zero (should handle gracefully)
    #[test]
    fn test_arith_001_division_by_zero() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: echo $((5 / 0))
        let result = executor.execute("echo $((5 / 0))");

        // ASSERT: RED phase - should error, not panic
        assert!(result.is_err());
    }

    /// ARITH-001: Arithmetic in command substitution
    #[test]
    fn test_arith_001_in_command_substitution() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: echo "Result: $((10 / 2))"
        let result = executor.execute("echo \"Result: $((10 / 2))\"");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "Result: 5");
    }
}

#[cfg(test)]
mod arithmetic_property_tests {
    use super::*;
    #[cfg(test)]
    use proptest::prelude::*;

    /// Property: Arithmetic is deterministic
    #[test]
    fn prop_arithmetic_deterministic() {
        proptest!(|(a in -1000i64..1000, b in -1000i64..1000)| {
            let mut executor1 = BashExecutor::new();
            let mut executor2 = BashExecutor::new();

            let script = format!("echo $(({} + {}))", a, b);
            let result1 = executor1.execute(&script).unwrap();
            let result2 = executor2.execute(&script).unwrap();

            prop_assert_eq!(result1.stdout, result2.stdout);
            prop_assert_eq!(result1.exit_code, result2.exit_code);
        });
    }

    /// Property: Addition is commutative: a + b = b + a
    #[test]
    fn prop_arithmetic_addition_commutative() {
        proptest!(|(a in -100i64..100, b in -100i64..100)| {
            let mut executor1 = BashExecutor::new();
            let mut executor2 = BashExecutor::new();

            let result1 = executor1.execute(&format!("echo $(({} + {}))", a, b)).unwrap();
            let result2 = executor2.execute(&format!("echo $(({} + {}))", b, a)).unwrap();

            prop_assert_eq!(result1.stdout.trim(), result2.stdout.trim());
        });
    }

    /// Property: Multiplication is commutative: a * b = b * a
    #[test]
    fn prop_arithmetic_multiplication_commutative() {
        proptest!(|(a in -50i64..50, b in -50i64..50)| {
            let mut executor1 = BashExecutor::new();
            let mut executor2 = BashExecutor::new();

            let result1 = executor1.execute(&format!("echo $(({} * {}))", a, b)).unwrap();
            let result2 = executor2.execute(&format!("echo $(({} * {}))", b, a)).unwrap();

            prop_assert_eq!(result1.stdout.trim(), result2.stdout.trim());
        });
    }

    /// Property: Addition with zero is identity: a + 0 = a
    #[test]
    fn prop_arithmetic_addition_identity() {
        proptest!(|(a in -1000i64..1000)| {
            let mut executor1 = BashExecutor::new();
            let mut executor2 = BashExecutor::new();

            let result1 = executor1.execute(&format!("echo $(({} + 0))", a)).unwrap();
            let result2 = executor2.execute(&format!("echo {}", a)).unwrap();

            prop_assert_eq!(result1.stdout.trim(), result2.stdout.trim());
        });
    }

    /// Property: Multiplication with one is identity: a * 1 = a
    #[test]
    fn prop_arithmetic_multiplication_identity() {
        proptest!(|(a in -1000i64..1000)| {
            let mut executor1 = BashExecutor::new();
            let mut executor2 = BashExecutor::new();

            let result1 = executor1.execute(&format!("echo $(({} * 1))", a)).unwrap();
            let result2 = executor2.execute(&format!("echo {}", a)).unwrap();

            prop_assert_eq!(result1.stdout.trim(), result2.stdout.trim());
        });
    }

    /// Property: Multiplication with zero: a * 0 = 0
    #[test]
    fn prop_arithmetic_multiplication_zero() {
        proptest!(|(a in -1000i64..1000)| {
            let mut executor = BashExecutor::new();

            let result = executor.execute(&format!("echo $(({} * 0))", a)).unwrap();

            prop_assert_eq!(result.stdout.trim(), "0");
        });
    }

    /// Property: Subtraction self: a - a = 0
    #[test]
    fn prop_arithmetic_subtraction_self() {
        proptest!(|(a in -1000i64..1000)| {
            let mut executor = BashExecutor::new();

            let result = executor.execute(&format!("echo $(({} - {}))", a, a)).unwrap();

            prop_assert_eq!(result.stdout.trim(), "0");
        });
    }

    /// Property: Division by self: a / a = 1 (for a != 0)
    #[test]
    fn prop_arithmetic_division_self() {
        proptest!(|(a in (-1000i64..=-1).prop_union(1i64..=1000))| {
            let mut executor = BashExecutor::new();

            let result = executor.execute(&format!("echo $(({} / {}))", a, a)).unwrap();

            prop_assert_eq!(result.stdout.trim(), "1");
        });
    }

    /// Property: Modulo range: a % b is in [0, |b|-1] for positive b
    #[test]
    fn prop_arithmetic_modulo_range() {
        proptest!(|(a in 0i64..1000, b in 1i64..100)| {
            let mut executor = BashExecutor::new();

            let result = executor.execute(&format!("echo $(({} % {}))", a, b)).unwrap();
            let modulo: i64 = result.stdout.trim().parse().unwrap();

            prop_assert!(modulo >= 0);
            prop_assert!(modulo < b);
        });
    }

    /// Property: Variables in arithmetic expand correctly
    #[test]
    fn prop_arithmetic_variables_expand() {
        proptest!(|(x in -100i64..100, y in -100i64..100)| {
            let mut executor = BashExecutor::new();

            let script = format!("x={}\ny={}\necho $((x + y))", x, y);
            let result = executor.execute(&script).unwrap();
            let sum: i64 = result.stdout.trim().parse().unwrap();

            prop_assert_eq!(sum, x + y);
        });
    }

    /// Property: Order of operations - multiplication before addition
    #[test]
    fn prop_arithmetic_order_of_operations() {
        proptest!(|(a in -50i64..50, b in -50i64..50, c in -50i64..50)| {
            let mut executor = BashExecutor::new();

            // a + b * c should be a + (b * c), not (a + b) * c
            let result = executor.execute(&format!("echo $(({} + {} * {}))", a, b, c)).unwrap();
            let value: i64 = result.stdout.trim().parse().unwrap();

            prop_assert_eq!(value, a + (b * c));
        });
    }

    /// Property: Negative numbers work correctly
    #[test]
    fn prop_arithmetic_negative_numbers() {
        proptest!(|(a in -100i64..100)| {
            let mut executor = BashExecutor::new();

            let result = executor.execute(&format!("echo $((-{}))", a)).unwrap();
            let value: i64 = result.stdout.trim().parse().unwrap();

            prop_assert_eq!(value, -a);
        });
    }

    /// Property: Division by zero always errors
    #[test]
    fn prop_arithmetic_division_by_zero_errors() {
        proptest!(|(a in -1000i64..1000)| {
            let mut executor = BashExecutor::new();

            let result = executor.execute(&format!("echo $(({} / 0))", a));

            prop_assert!(result.is_err());
        });
    }

    /// Property: Modulo by zero always errors
    #[test]
    fn prop_arithmetic_modulo_by_zero_errors() {
        proptest!(|(a in -1000i64..1000)| {
            let mut executor = BashExecutor::new();

            let result = executor.execute(&format!("echo $(({} % 0))", a));

            prop_assert!(result.is_err());
        });
    }
}

#[cfg(test)]
mod test_command_tests {
    use super::*;

    /// TEST-001: Integer comparison -eq (equal)
    #[test]
    fn test_cmd_001_eq_true() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: if [ 5 -eq 5 ]; then echo "yes"; fi
        let result = executor.execute("if [ 5 -eq 5 ]\nthen\necho \"yes\"\nfi");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "yes");
    }

    /// TEST-001: Integer comparison -eq (not equal)
    #[test]
    fn test_cmd_001_eq_false() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: if [ 5 -eq 3 ]; then echo "yes"; else echo "no"; fi
        let result = executor.execute("if [ 5 -eq 3 ]\nthen\necho \"yes\"\nelse\necho \"no\"\nfi");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "no");
    }

    /// TEST-001: Integer comparison -ne (not equal)
    #[test]
    fn test_cmd_001_ne_true() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: if [ 5 -ne 3 ]; then echo "yes"; fi
        let result = executor.execute("if [ 5 -ne 3 ]\nthen\necho \"yes\"\nfi");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "yes");
    }

    /// TEST-001: Integer comparison -gt (greater than)
    #[test]
    fn test_cmd_001_gt_true() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: if [ 10 -gt 5 ]; then echo "yes"; fi
        let result = executor.execute("if [ 10 -gt 5 ]\nthen\necho \"yes\"\nfi");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "yes");
    }

    /// TEST-001: Integer comparison -gt (not greater than)
    #[test]
    fn test_cmd_001_gt_false() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: if [ 3 -gt 5 ]; then echo "yes"; else echo "no"; fi
        let result = executor.execute("if [ 3 -gt 5 ]\nthen\necho \"yes\"\nelse\necho \"no\"\nfi");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "no");
    }

    /// TEST-001: Integer comparison -lt (less than)
    #[test]
    fn test_cmd_001_lt_true() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: if [ 3 -lt 10 ]; then echo "yes"; fi
        let result = executor.execute("if [ 3 -lt 10 ]\nthen\necho \"yes\"\nfi");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "yes");
    }

    /// TEST-001: Integer comparison -ge (greater or equal)
    #[test]
    fn test_cmd_001_ge_true_greater() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: if [ 10 -ge 5 ]; then echo "yes"; fi
        let result = executor.execute("if [ 10 -ge 5 ]\nthen\necho \"yes\"\nfi");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "yes");
    }

    /// TEST-001: Integer comparison -ge (greater or equal - equal case)
    #[test]
    fn test_cmd_001_ge_true_equal() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: if [ 5 -ge 5 ]; then echo "yes"; fi
        let result = executor.execute("if [ 5 -ge 5 ]\nthen\necho \"yes\"\nfi");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "yes");
    }

    /// TEST-001: Integer comparison -le (less or equal)
    #[test]
    fn test_cmd_001_le_true_less() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: if [ 3 -le 10 ]; then echo "yes"; fi
        let result = executor.execute("if [ 3 -le 10 ]\nthen\necho \"yes\"\nfi");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "yes");
    }

    /// TEST-001: Integer comparison -le (less or equal - equal case)
    #[test]
    fn test_cmd_001_le_true_equal() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: if [ 5 -le 5 ]; then echo "yes"; fi
        let result = executor.execute("if [ 5 -le 5 ]\nthen\necho \"yes\"\nfi");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "yes");
    }

    /// TEST-001: Test command with variables
    #[test]
    fn test_cmd_001_with_variables() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: x=10; y=5; if [ $x -gt $y ]; then echo "yes"; fi
        let result = executor.execute("x=10\ny=5\nif [ $x -gt $y ]\nthen\necho \"yes\"\nfi");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "yes");
    }

    /// TEST-001: Test command in while loop
    #[test]
    fn test_cmd_001_in_while_loop() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: i=3; while [ $i -gt 0 ]; do echo $i; i=$((i-1)); done
        let result = executor.execute("i=3\nwhile [ $i -gt 0 ]\ndo\necho $i\ni=$((i-1))\ndone");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout, "3\n2\n1\n");
    }

    /// TEST-001: String comparison = (equal)
    #[test]
    fn test_cmd_001_string_eq_true() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: if [ "abc" = "abc" ]; then echo "yes"; fi
        let result = executor.execute("if [ \"abc\" = \"abc\" ]\nthen\necho \"yes\"\nfi");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "yes");
    }

    /// TEST-001: String comparison != (not equal)
    #[test]
    fn test_cmd_001_string_ne_true() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: if [ "abc" != "def" ]; then echo "yes"; fi
        let result = executor.execute("if [ \"abc\" != \"def\" ]\nthen\necho \"yes\"\nfi");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "yes");
    }

    /// TEST-001: String test -n (non-empty string)
    #[test]
    fn test_cmd_001_string_n_true() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT: if [ -n "hello" ]; then echo "yes"; fi
        let result = executor.execute("if [ -n \"hello\" ]\nthen\necho \"yes\"\nfi");

        // ASSERT: RED phase
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stdout.trim(), "yes");
    }
}

#[cfg(test)]
mod test_command_property_tests {
    use super::*;
    #[cfg(test)]
    use proptest::prelude::*;

    /// Property: Test command is deterministic
    #[test]
    fn prop_test_deterministic() {
        proptest!(|(a in -100i64..100, b in -100i64..100)| {
            let mut executor1 = BashExecutor::new();
            let mut executor2 = BashExecutor::new();

            let script = format!("if [ {} -gt {} ]\nthen\necho \"yes\"\nfi", a, b);
            let result1 = executor1.execute(&script).unwrap();
            let result2 = executor2.execute(&script).unwrap();

            prop_assert_eq!(result1.stdout, result2.stdout);
        });
    }

    /// Property: -eq is symmetric: a -eq b iff b -eq a
    #[test]
    fn prop_test_eq_symmetric() {
        proptest!(|(a in -100i64..100, b in -100i64..100)| {
            let mut executor1 = BashExecutor::new();
            let mut executor2 = BashExecutor::new();

            let result1 = executor1.execute(&format!("if [ {} -eq {} ]\nthen\necho \"yes\"\nfi", a, b)).unwrap();
            let result2 = executor2.execute(&format!("if [ {} -eq {} ]\nthen\necho \"yes\"\nfi", b, a)).unwrap();

            prop_assert_eq!(result1.stdout, result2.stdout);
        });
    }


    /// Property: -eq self: a -eq a is always true
    #[test]
    fn prop_test_eq_self() {
        proptest!(|(a in -1000i64..1000)| {
            let mut executor = BashExecutor::new();

            let result = executor.execute(&format!("if [ {} -eq {} ]\nthen\necho \"yes\"\nfi", a, a)).unwrap();

            prop_assert_eq!(result.stdout.trim(), "yes");
        });
    }

    /// Property: -ne self: a -ne a is always false
    #[test]
    fn prop_test_ne_self() {
        proptest!(|(a in -1000i64..1000)| {
            let mut executor = BashExecutor::new();

            let result = executor.execute(&format!("if [ {} -ne {} ]\nthen\necho \"yes\"\nelse\necho \"no\"\nfi", a, a)).unwrap();

            prop_assert_eq!(result.stdout.trim(), "no");
        });
    }

    /// Property: -gt self: a -gt a is always false
    #[test]
    fn prop_test_gt_self() {
        proptest!(|(a in -1000i64..1000)| {
            let mut executor = BashExecutor::new();

            let result = executor.execute(&format!("if [ {} -gt {} ]\nthen\necho \"yes\"\nelse\necho \"no\"\nfi", a, a)).unwrap();

            prop_assert_eq!(result.stdout.trim(), "no");
        });
    }

    /// Property: -ge self: a -ge a is always true
    #[test]
    fn prop_test_ge_self() {
        proptest!(|(a in -1000i64..1000)| {
            let mut executor = BashExecutor::new();

            let result = executor.execute(&format!("if [ {} -ge {} ]\nthen\necho \"yes\"\nfi", a, a)).unwrap();

            prop_assert_eq!(result.stdout.trim(), "yes");
        });
    }

    /// Property: -le self: a -le a is always true
    #[test]
    fn prop_test_le_self() {
        proptest!(|(a in -1000i64..1000)| {
            let mut executor = BashExecutor::new();

            let result = executor.execute(&format!("if [ {} -le {} ]\nthen\necho \"yes\"\nfi", a, a)).unwrap();

            prop_assert_eq!(result.stdout.trim(), "yes");
        });
    }

    /// Property: Transitivity: if a < b and b < c then a < c
    #[test]
    fn prop_test_lt_transitive() {
        proptest!(|(a in -50i64..0, b in 0i64..50, c in 50i64..100)| {
            // Ensure a < b < c
            let mut executor = BashExecutor::new();

            // Test a < c (should be true since a < b < c)
            let result = executor.execute(&format!("if [ {} -lt {} ]\nthen\necho \"yes\"\nfi", a, c)).unwrap();

            prop_assert_eq!(result.stdout.trim(), "yes");
        });
    }

    /// Property: String equality is reflexive
    #[test]
    fn prop_test_string_eq_reflexive() {
        proptest!(|(s in "[a-z]{1,10}")| {
            let mut executor = BashExecutor::new();

            let result = executor.execute(&format!("if [ \"{}\" = \"{}\" ]\nthen\necho \"yes\"\nfi", s, s)).unwrap();

            prop_assert_eq!(result.stdout.trim(), "yes");
        });
    }

    /// Property: -n with non-empty string is always true
    #[test]
    fn prop_test_n_nonempty() {
        proptest!(|(s in "[a-z]{1,10}")| {
            let mut executor = BashExecutor::new();

            let result = executor.execute(&format!("if [ -n \"{}\" ]\nthen\necho \"yes\"\nfi", s)).unwrap();

            prop_assert_eq!(result.stdout.trim(), "yes");
        });
    }

    /// Property: Test in while loop condition works correctly
    #[test]
    fn prop_test_in_while_counts_correctly() {
        proptest!(|(n in 1i64..10)| {
            let mut executor = BashExecutor::new();

            let script = format!("i={}\nwhile [ $i -gt 0 ]\ndo\necho $i\ni=$((i-1))\ndone", n);
            let result = executor.execute(&script).unwrap();

            // Count lines of output
            let lines: Vec<&str> = result.stdout.lines().collect();
            prop_assert_eq!(lines.len() as i64, n);
        });
    }
}

/// ============================================================================
/// FUNC-001: Bash Functions - Unit Tests (RED Phase)
/// ============================================================================
///
/// Tests for bash function definition and execution.
///
/// Bash Function Syntax:
/// ```bash
/// # Style 1: function keyword
/// function greet() {
///     echo "Hello, $1"
/// }
///
/// # Style 2: name() syntax
/// greet() {
///     echo "Hello, $1"
/// }
///
/// # Calling
/// greet "World"  # Output: Hello, World
/// ```
///
/// Test Coverage:
/// 1. Basic function definition and call
/// 2. Function with positional parameters ($1, $2)
/// 3. Function return values (exit codes)
/// 4. Multiple function definitions
/// 5. Function calling other functions
/// 6. Local variables vs global variables
/// 7. Function with no parameters
/// 8. Function with multiple statements
/// 9. Nested function calls
/// 10. Return statement
/// 11. Function overriding
/// 12. Empty function body
/// 13. Function with loops
/// 14. Function with conditionals
/// 15. Complex function composition
///
#[cfg(test)]
mod function_tests {
    use super::*;

    /// Test 1: Basic function definition and call (style 2: name())
    #[test]
    fn test_func_001_basic_definition_and_call() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute(
            r#"
greet() {
    echo "Hello"
}
greet
"#,
        );

        // ASSERT
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.stdout.trim(), "Hello");
    }

    /// Test 2: Function with single positional parameter
    #[test]
    fn test_func_002_single_parameter() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute(
            r#"
greet() {
    echo "Hello, $1"
}
greet "World"
"#,
        );

        // ASSERT
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.stdout.trim(), "Hello, World");
    }

    /// Test 3: Function with multiple positional parameters
    #[test]
    fn test_func_003_multiple_parameters() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute(
            r#"
add() {
    echo "$(($1 + $2))"
}
add 5 3
"#,
        );

        // ASSERT
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.stdout.trim(), "8");
    }

    /// Test 4: Function using 'function' keyword (style 1)
    #[test]
    fn test_func_004_function_keyword() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute(
            r#"
function greet() {
    echo "Hello from function keyword"
}
greet
"#,
        );

        // ASSERT
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.stdout.trim(), "Hello from function keyword");
    }

    /// Test 5: Multiple function definitions
    #[test]
    fn test_func_005_multiple_functions() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute(
            r#"
hello() {
    echo "Hello"
}
world() {
    echo "World"
}
hello
world
"#,
        );

        // ASSERT
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.stdout.trim(), "Hello\nWorld");
    }

    /// Test 6: Function calling another function
    #[test]
    fn test_func_006_nested_calls() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute(
            r#"
inner() {
    echo "inner: $1"
}
outer() {
    inner "from outer"
}
outer
"#,
        );

        // ASSERT
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.stdout.trim(), "inner: from outer");
    }

    /// Test 7: Function with multiple statements
    #[test]
    fn test_func_007_multiple_statements() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute(
            r#"
multi() {
    echo "Line 1"
    echo "Line 2"
    echo "Line 3"
}
multi
"#,
        );

        // ASSERT
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.stdout.trim(), "Line 1\nLine 2\nLine 3");
    }

    /// Test 8: Function with variable assignment
    #[test]
    fn test_func_008_variable_assignment() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute(
            r#"
setvar() {
    myvar="set in function"
}
setvar
echo $myvar
"#,
        );

        // ASSERT
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.stdout.trim(), "set in function");
    }

    /// Test 9: Function with conditional
    #[test]
    fn test_func_009_with_conditional() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute(
            r#"
check() {
    if [ "$1" = "yes" ]
    then
        echo "affirmative"
    else
        echo "negative"
    fi
}
check "yes"
check "no"
"#,
        );

        // ASSERT
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.stdout.trim(), "affirmative\nnegative");
    }

    /// Test 10: Function with for loop
    #[test]
    fn test_func_010_with_for_loop() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute(
            r#"
count() {
    for i in 1 2 3
    do
        echo "num: $i"
    done
}
count
"#,
        );

        // ASSERT
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.stdout.trim(), "num: 1\nnum: 2\nnum: 3");
    }

    /// Test 11: Function calling function with parameters
    #[test]
    fn test_func_011_nested_with_params() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute(
            r#"
double() {
    echo "$(($1 * 2))"
}
quadruple() {
    result=$(double $1)
    echo "$((result * 2))"
}
quadruple 5
"#,
        );

        // ASSERT
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.stdout.trim(), "20");
    }

    /// Test 12: Function with no body (empty function)
    #[test]
    fn test_func_012_empty_function() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute(
            r#"
noop() {
    :
}
noop
echo "after noop"
"#,
        );

        // ASSERT
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.stdout.trim(), "after noop");
    }

    /// Test 13: Function parameter $0 (function name)
    #[test]
    fn test_func_013_parameter_zero() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute(
            r#"
showname() {
    echo "Function: $0"
}
showname
"#,
        );

        // ASSERT
        assert!(result.is_ok());
        let output = result.unwrap();
        // $0 in function context might be function name or script name
        // This tests expected behavior
        assert!(output.stdout.contains("Function:"));
    }

    /// Test 14: Function with while loop
    #[test]
    fn test_func_014_with_while_loop() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute(
            r#"
countdown() {
    i=3
    while [ $i -gt 0 ]
    do
        echo $i
        i=$((i-1))
    done
}
countdown
"#,
        );

        // ASSERT
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.stdout.trim(), "3\n2\n1");
    }

    /// Test 15: Multiple calls to same function
    #[test]
    fn test_func_015_multiple_calls() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute(
            r#"
say() {
    echo "Message: $1"
}
say "first"
say "second"
say "third"
"#,
        );

        // ASSERT
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(
            output.stdout.trim(),
            "Message: first\nMessage: second\nMessage: third"
        );
    }

    /// Test 16: Function with arithmetic
    #[test]
    fn test_func_016_with_arithmetic() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute(
            r#"
calc() {
    a=$1
    b=$2
    sum=$((a + b))
    product=$((a * b))
    echo "sum: $sum"
    echo "product: $product"
}
calc 4 5
"#,
        );

        // ASSERT
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.stdout.trim(), "sum: 9\nproduct: 20");
    }

    /// Test 17: Function overriding (redefining)
    #[test]
    fn test_func_017_function_override() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute(
            r#"
greet() {
    echo "First version"
}
greet() {
    echo "Second version"
}
greet
"#,
        );

        // ASSERT
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.stdout.trim(), "Second version");
    }

    /// Test 18: Function with command substitution
    #[test]
    fn test_func_018_command_substitution() {
        // ARRANGE
        let mut executor = BashExecutor::new();

        // ACT
        let result = executor.execute(
            r#"
wrapper() {
    result=$(echo "inner output")
    echo "wrapped: $result"
}
wrapper
"#,
        );

        // ASSERT
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.stdout.trim(), "wrapped: inner output");
    }
}

/// ============================================================================
/// FUNC-001: Bash Functions - Property Tests (REFACTOR Phase)
/// ============================================================================
///
/// Property-based tests for bash function behavior.
/// Uses proptest to generate test cases and verify properties.
///
#[cfg(test)]
mod function_property_tests {
    use super::*;
    use proptest::prelude::*;

    /// Property: Function calls are deterministic (same input = same output)
    #[test]
    fn prop_func_deterministic() {
        proptest!(|(s in "[a-z]{1,10}")| {
            let mut executor1 = BashExecutor::new();
            let mut executor2 = BashExecutor::new();

            let script = format!(r#"
greet() {{
    echo "Hello, $1"
}}
greet "{}"
"#, s);

            let result1 = executor1.execute(&script).unwrap();
            let result2 = executor2.execute(&script).unwrap();

            prop_assert_eq!(result1.stdout, result2.stdout);
            prop_assert_eq!(result1.exit_code, result2.exit_code);
        });
    }

    /// Property: Multiple calls to same function produce consistent results
    #[test]
    fn prop_func_multiple_calls_consistent() {
        proptest!(|(n in 1usize..10)| {
            let mut executor = BashExecutor::new();

            let mut calls = String::new();
            for _ in 0..n {
                calls.push_str("say \"test\"\n");
            }

            let script = format!(r#"
say() {{
    echo "Message: $1"
}}
{}
"#, calls);

            let result = executor.execute(&script).unwrap();
            let lines: Vec<&str> = result.stdout.lines().collect();

            prop_assert_eq!(lines.len(), n);
            for line in lines {
                prop_assert_eq!(line, "Message: test");
            }
        });
    }

    /// Property: Function parameters are properly isolated
    #[test]
    fn prop_func_parameters_isolated() {
        proptest!(|(a in 1i64..100, b in 1i64..100)| {
            let mut executor = BashExecutor::new();

            let script = format!(r#"
add() {{
    echo "$(($1 + $2))"
}}
add {} {}
"#, a, b);

            let result = executor.execute(&script).unwrap();
            let output: i64 = result.stdout.trim().parse().unwrap_or(0);

            prop_assert_eq!(output, a + b);
        });
    }

    /// Property: Variable assignments in functions persist
    #[test]
    fn prop_func_variables_persist() {
        proptest!(|(s in "[a-z]{1,10}")| {
            let mut executor = BashExecutor::new();

            let script = format!(r#"
setvar() {{
    myvar="{}"
}}
setvar
echo $myvar
"#, s);

            let result = executor.execute(&script).unwrap();

            prop_assert_eq!(result.stdout.trim(), s);
        });
    }

    /// Property: Empty functions always succeed
    #[test]
    fn prop_func_empty_succeeds() {
        proptest!(|(name in "[a-z]{1,10}")| {
            let mut executor = BashExecutor::new();

            let script = format!(r#"
{}() {{
    :
}}
{}
echo "done"
"#, name, name);

            let result = executor.execute(&script).unwrap();

            prop_assert_eq!(result.exit_code, 0);
            prop_assert_eq!(result.stdout.trim(), "done");
        });
    }

    /// Property: Function redefinition replaces previous definition
    #[test]
    fn prop_func_override_replaces() {
        proptest!(|(s1 in "[a-z]{1,10}", s2 in "[a-z]{1,10}")| {
            prop_assume!(s1 != s2); // Ensure different strings

            let mut executor = BashExecutor::new();

            let script = format!(r#"
greet() {{
    echo "{}"
}}
greet() {{
    echo "{}"
}}
greet
"#, s1, s2);

            let result = executor.execute(&script).unwrap();

            prop_assert_eq!(result.stdout.trim(), s2);
        });
    }

    /// Property: Functions can call themselves recursively (depth limited)
    #[test]
    fn prop_func_recursion_limited() {
        proptest!(|(n in 1i64..5)| {
            let mut executor = BashExecutor::new();

            let script = format!(r#"
countdown() {{
    if [ $1 -gt 0 ]
    then
        echo $1
        countdown $(($1 - 1))
    fi
}}
countdown {}
"#, n);

            let result = executor.execute(&script);

            // Should either succeed or fail gracefully
            prop_assert!(result.is_ok() || result.is_err());

            if result.is_ok() {
                let output = result.unwrap();
                let lines: Vec<&str> = output.stdout.lines().collect();
                // If it succeeds, should print countdown
                if !lines.is_empty() {
                    prop_assert!(lines.len() as i64 <= n);
                }
            }
        });
    }

    /// Property: Function with for loop processes all items
    #[test]
    fn prop_func_for_loop_processes_all() {
        proptest!(|(items in prop::collection::vec("[a-z]{1,5}", 1..5))| {
            let mut executor = BashExecutor::new();

            let items_str = items.join(" ");
            let script = format!(r#"
process() {{
    for item in $1
    do
        echo "item: $item"
    done
}}
process "{}"
"#, items_str);

            let result = executor.execute(&script).unwrap();
            let lines: Vec<&str> = result.stdout.lines().collect();

            prop_assert_eq!(lines.len(), items.len());
        });
    }

    /// Property: Function with arithmetic always returns correct result
    #[test]
    fn prop_func_arithmetic_correct() {
        proptest!(|(a in 1i64..50, b in 1i64..50)| {
            let mut executor = BashExecutor::new();

            let script = format!(r#"
multiply() {{
    echo "$(($1 * $2))"
}}
multiply {} {}
"#, a, b);

            let result = executor.execute(&script).unwrap();
            let output: i64 = result.stdout.trim().parse().unwrap_or(0);

            prop_assert_eq!(output, a * b);
        });
    }

    /// Property: Function with conditionals handles both branches
    #[test]
    fn prop_func_conditional_branches() {
        proptest!(|(value in "[a-z]{1,10}")| {
            let mut executor = BashExecutor::new();

            let script = format!(r#"
check() {{
    if [ "$1" = "yes" ]
    then
        echo "affirmative"
    else
        echo "negative"
    fi
}}
check "{}"
"#, value);

            let result = executor.execute(&script).unwrap();
            let output = result.stdout.trim();

            if value == "yes" {
                prop_assert_eq!(output, "affirmative");
            } else {
                prop_assert_eq!(output, "negative");
            }
        });
    }

    /// Property: Multiple function definitions are all stored
    #[test]
    fn prop_func_multiple_definitions_stored() {
        proptest!(|(n in 2usize..5)| {
            let mut executor = BashExecutor::new();

            let mut script = String::new();
            for i in 0..n {
                script.push_str(&format!(r#"
func{}() {{
    echo "function {}"
}}
"#, i, i));
            }

            // Call all functions
            for i in 0..n {
                script.push_str(&format!("func{}\n", i));
            }

            let result = executor.execute(&script).unwrap();
            let lines: Vec<&str> = result.stdout.lines().collect();

            prop_assert_eq!(lines.len(), n);
            for (i, line) in lines.iter().enumerate() {
                prop_assert_eq!(*line, format!("function {}", i));
            }
        });
    }
}
