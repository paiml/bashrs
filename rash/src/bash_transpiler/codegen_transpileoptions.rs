//! Code Generation for Bash-to-Rash Transpiler

use super::patterns::*;
use super::TranspileResult;
use crate::bash_parser::ast::*;

pub struct TranspileOptions {
    pub add_safety_checks: bool,
    pub preserve_comments: bool,
    pub indent_size: usize,
}

impl Default for TranspileOptions {
    fn default() -> Self {
        Self {
            add_safety_checks: true,
            preserve_comments: true,
            indent_size: 4,
        }
    }
}

pub struct BashToRashTranspiler {
    options: TranspileOptions,
    current_indent: usize,
}

impl BashToRashTranspiler {
    pub fn new(options: TranspileOptions) -> Self {
        Self {
            options,
            current_indent: 0,
        }
    }

    pub fn transpile(&mut self, ast: &BashAst) -> TranspileResult<String> {
        let mut output = String::new();

        // Add header comment
        output.push_str("// Transpiled from bash by rash\n\n");

        // Process all statements
        for stmt in &ast.statements {
            let rash_code = self.transpile_statement(stmt)?;
            output.push_str(&self.indent(&rash_code));
            output.push('\n');
        }

        Ok(output)
    }

    fn transpile_statement(&mut self, stmt: &BashStmt) -> TranspileResult<String> {
        match stmt {
            BashStmt::Assignment {
                name,
                value,
                exported,
                ..
            } => self.transpile_assignment(name, value, *exported),

            BashStmt::Command { name, args, .. } => self.transpile_command_stmt(name, args),

            BashStmt::Function { name, body, .. } => self.transpile_function_stmt(name, body),

            BashStmt::If {
                condition,
                then_block,
                elif_blocks,
                else_block,
                ..
            } => self.transpile_if_stmt(condition, then_block, elif_blocks, else_block),

            BashStmt::While {
                condition, body, ..
            } => self.transpile_while_stmt(condition, body),

            BashStmt::Until {
                condition, body, ..
            } => self.transpile_until_stmt(condition, body),

            BashStmt::For {
                variable,
                items,
                body,
                ..
            } => self.transpile_for_stmt(variable, items, body),

            BashStmt::ForCStyle { body, .. } => self.transpile_for_c_style_stmt(body),

            BashStmt::Return { code, .. } => self.transpile_return_stmt(code.as_ref()),

            BashStmt::Comment { text, .. } => self.transpile_comment(text),

            BashStmt::Case { word, arms, .. } => self.transpile_case_stmt(word, arms),

            BashStmt::Pipeline { commands, .. } => self.transpile_pipeline_stmt(commands),

            BashStmt::AndList { left, right, .. } => self.transpile_and_list(left, right),

            BashStmt::OrList { left, right, .. } => self.transpile_or_list(left, right),

            BashStmt::BraceGroup { body, .. } => self.transpile_brace_group(body),

            BashStmt::Coproc { name, body, .. } => {
                self.transpile_coproc_stmt(name.as_deref(), body)
            }

            BashStmt::Select {
                variable,
                items,
                body,
                ..
            } => self.transpile_select_stmt(variable, items, body),

            BashStmt::Negated { command, .. } => self.transpile_negated_stmt(command),
        }
    }

    fn transpile_assignment(
        &mut self,
        name: &str,
        value: &BashExpr,
        exported: bool,
    ) -> TranspileResult<String> {
        let value_rash = self.transpile_expression(value)?;
        let pattern = VariablePattern { exported };
        Ok(pattern.to_rash(name, &value_rash))
    }

    fn transpile_command_stmt(&mut self, name: &str, args: &[BashExpr]) -> TranspileResult<String> {
        let mut rash_args = Vec::new();
        for arg in args {
            rash_args.push(self.transpile_expression(arg)?);
        }
        Ok(CommandPattern::to_rash(name, &rash_args))
    }

    fn transpile_function_stmt(
        &mut self,
        name: &str,
        body: &[BashStmt],
    ) -> TranspileResult<String> {
        self.current_indent += 1;
        let mut body_stmts = Vec::new();
        for stmt in body {
            body_stmts.push(self.transpile_statement(stmt)?);
        }
        let body_rash = body_stmts.join("\n");
        self.current_indent -= 1;

        Ok(FunctionPattern::to_rash(name, &self.indent(&body_rash)))
    }

    fn transpile_if_stmt(
        &mut self,
        condition: &BashExpr,
        then_block: &[BashStmt],
        elif_blocks: &[(BashExpr, Vec<BashStmt>)],
        else_block: &Option<Vec<BashStmt>>,
    ) -> TranspileResult<String> {
        let cond_rash = self.transpile_test_expression(condition)?;

        self.current_indent += 1;
        let then_rash = self.transpile_block(then_block)?;

        let mut elif_rash = Vec::new();
        for (elif_cond, elif_body) in elif_blocks {
            let cond = self.transpile_test_expression(elif_cond)?;
            let body = self.transpile_block(elif_body)?;
            elif_rash.push((cond, body));
        }

        let else_rash = if let Some(else_body) = else_block {
            Some(self.transpile_block(else_body)?)
        } else {
            None
        };

        self.current_indent -= 1;

        Ok(IfPattern::to_rash(
            &cond_rash,
            &then_rash,
            &elif_rash,
            else_rash.as_deref(),
        ))
    }

    fn transpile_while_stmt(
        &mut self,
        condition: &BashExpr,
        body: &[BashStmt],
    ) -> TranspileResult<String> {
        let cond_rash = self.transpile_test_expression(condition)?;

        self.current_indent += 1;
        let body_rash = self.transpile_block(body)?;
        self.current_indent -= 1;

        Ok(WhilePattern::to_rash(&cond_rash, &body_rash))
    }

    fn transpile_until_stmt(
        &mut self,
        condition: &BashExpr,
        body: &[BashStmt],
    ) -> TranspileResult<String> {
        // Until loop transpiles to while with negated condition
        let cond_rash = self.transpile_test_expression(condition)?;
        let negated_cond = format!("!({})", cond_rash);

        self.current_indent += 1;
        let body_rash = self.transpile_block(body)?;
        self.current_indent -= 1;

        Ok(WhilePattern::to_rash(&negated_cond, &body_rash))
    }

    fn transpile_for_stmt(
        &mut self,
        variable: &str,
        items: &BashExpr,
        body: &[BashStmt],
    ) -> TranspileResult<String> {
        let items_rash = self.transpile_expression(items)?;

        self.current_indent += 1;
        let body_rash = self.transpile_block(body)?;
        self.current_indent -= 1;

        Ok(ForPattern::to_rash(variable, &items_rash, &body_rash))
    }

    fn transpile_for_c_style_stmt(&mut self, body: &[BashStmt]) -> TranspileResult<String> {
        // Issue #68: C-style for loop (transpile to Rust while loop)
        // For now, transpile C-style loops as a comment + body
        // Full conversion would need parsing C arithmetic to Rust
        self.current_indent += 1;
        let body_rash = self.transpile_block(body)?;
        self.current_indent -= 1;

        Ok(format!(
            "// C-style for loop (not yet fully transpiled)\n{}",
            body_rash
        ))
    }

    fn transpile_return_stmt(&mut self, code: Option<&BashExpr>) -> TranspileResult<String> {
        if let Some(expr) = code {
            let val = self.transpile_expression(expr)?;
            Ok(format!("return {};", val))
        } else {
            Ok("return;".to_string())
        }
    }

    fn transpile_comment(&self, text: &str) -> TranspileResult<String> {
        if self.options.preserve_comments {
            Ok(format!("//{}", text))
        } else {
            Ok(String::new())
        }
    }

    fn transpile_case_stmt(
        &mut self,
        word: &BashExpr,
        arms: &[CaseArm],
    ) -> TranspileResult<String> {
        let word_rash = self.transpile_expression(word)?;
        let mut result = format!("match {} {{\n", word_rash);

        self.current_indent += 1;

        for arm in arms {
            let pattern_str = arm.patterns.join(" | ");
            result.push_str(&self.indent(&format!("{} => {{\n", pattern_str)));

            self.current_indent += 1;
            for stmt in &arm.body {
                let stmt_rash = self.transpile_statement(stmt)?;
                result.push_str(&self.indent(&stmt_rash));
                result.push('\n');
            }
            self.current_indent -= 1;

            result.push_str(&self.indent("}\n"));
        }

        self.current_indent -= 1;
        result.push_str(&self.indent("}"));

        Ok(result)
    }

    fn transpile_pipeline_stmt(&mut self, commands: &[BashStmt]) -> TranspileResult<String> {
        // TODO: Full pipeline transpilation not implemented yet
        // For now, transpile each command separately
        let mut result = String::new();
        for cmd in commands {
            result.push_str(&self.transpile_statement(cmd)?);
            result.push_str(" | ");
        }
        // Remove trailing " | "
        if result.ends_with(" | ") {
            result.truncate(result.len() - 3);
        }
        Ok(result)
    }

    fn transpile_and_list(&mut self, left: &BashStmt, right: &BashStmt) -> TranspileResult<String> {
        // Transpile AND list: left && right
        let left_str = self.transpile_statement(left)?;
        let right_str = self.transpile_statement(right)?;
        Ok(format!("{} && {}", left_str, right_str))
    }

    fn transpile_or_list(&mut self, left: &BashStmt, right: &BashStmt) -> TranspileResult<String> {
        // Transpile OR list: left || right
        let left_str = self.transpile_statement(left)?;
        let right_str = self.transpile_statement(right)?;
        Ok(format!("{} || {}", left_str, right_str))
    }

    fn transpile_brace_group(&mut self, body: &[BashStmt]) -> TranspileResult<String> {
        // Transpile brace group as a block
        self.current_indent += 1;
        let body_rash = self.transpile_block(body)?;
        self.current_indent -= 1;
        Ok(format!("{{\n{}\n}}", body_rash))
    }

    fn transpile_coproc_stmt(
        &mut self,
        name: Option<&str>,
        body: &[BashStmt],
    ) -> TranspileResult<String> {
        // Coproc is bash-specific, transpile as async block
        // Note: This is a placeholder - coproc has no direct Rust equivalent
        self.current_indent += 1;
        let body_rash = self.transpile_block(body)?;
        self.current_indent -= 1;
        if let Some(n) = name {
            Ok(format!(
                "// coproc {} - async subprocess\n{{\n{}\n}}",
                n, body_rash
            ))
        } else {
            Ok(format!(
                "// coproc - async subprocess\n{{\n{}\n}}",
                body_rash
            ))
        }
    }

    fn transpile_select_stmt(
        &mut self,
        variable: &str,
        items: &BashExpr,
        body: &[BashStmt],
    ) -> TranspileResult<String> {
        // F017: Select is bash-specific, transpile as loop with menu
        // Note: No direct Rust equivalent, generate a comment placeholder
        self.current_indent += 1;
        let body_rash = self.transpile_block(body)?;
        self.current_indent -= 1;
        let items_rash = self.transpile_expression(items)?;
        Ok(format!(
            "// select {} in {} - interactive menu loop\nfor {} in {} {{\n{}\n}}",
            variable, items_rash, variable, items_rash, body_rash
        ))
    }

    fn transpile_negated_stmt(&mut self, command: &BashStmt) -> TranspileResult<String> {
        // Issue #133: Negated command - transpile inner and negate
        let inner = self.transpile_statement(command)?;
        Ok(format!("// negated: ! {}", inner))
    }

    fn transpile_block(&mut self, stmts: &[BashStmt]) -> TranspileResult<String> {
        let mut result = Vec::new();
        for stmt in stmts {
            result.push(self.transpile_statement(stmt)?);
        }
        Ok(self.indent(&result.join("\n")))
    }

    fn transpile_expression(&mut self, expr: &BashExpr) -> TranspileResult<String> {
        match expr {
            BashExpr::Literal(s) => {
                // Quote strings for Rust
                if s.parse::<i64>().is_ok() || s.parse::<bool>().is_ok() {
                    Ok(s.clone())
                } else {
                    Ok(format!("\"{}\"", s.escape_default()))
                }
            }

            BashExpr::Variable(name) => Ok(name.clone()),

            BashExpr::Array(items) => {
                let mut rash_items = Vec::new();
                for item in items {
                    rash_items.push(self.transpile_expression(item)?);
                }
                Ok(format!("vec![{}]", rash_items.join(", ")))
            }

            BashExpr::Concat(parts) => {
                let mut rash_parts = Vec::new();
                for part in parts {
                    rash_parts.push(self.transpile_expression(part)?);
                }
                Ok(format!("format!(\"{{}}\", {})", rash_parts.join(" + ")))
            }

            BashExpr::CommandSubst(cmd) => {
                // Command substitution becomes a function call
                let cmd_rash = self.transpile_statement(cmd)?;
                Ok(format!("{{ {} }}", cmd_rash))
            }

            BashExpr::Arithmetic(arith) => self.transpile_arithmetic(arith),

            BashExpr::Test(test) => self.transpile_test(test),

            BashExpr::Glob(pattern) => {
                // Convert glob to Rust code that returns matching paths
                Ok(format!("glob(\"{}\")", pattern.escape_default()))
            }

            BashExpr::DefaultValue { variable, default } => {
                // ${VAR:-default} → var.unwrap_or("default")
                let default_rash = self.transpile_expression(default)?;
                Ok(format!("{}.unwrap_or({})", variable, default_rash))
            }

            BashExpr::AssignDefault { variable, default } => {
                // ${VAR:=default} → var.get_or_insert("default")
                // or: if var.is_none() { var = Some("default"); } var.unwrap()
                let default_rash = self.transpile_expression(default)?;
                Ok(format!("{}.get_or_insert({})", variable, default_rash))
            }

            BashExpr::ErrorIfUnset { variable, message } => {
                // ${VAR:?message} → var.expect("message")
                let message_rash = self.transpile_expression(message)?;
                Ok(format!("{}.expect({})", variable, message_rash))
            }

            BashExpr::AlternativeValue {
                variable,
                alternative,
            } => {
                // ${VAR:+alt} → var.as_ref().map(|_| alt).unwrap_or("")
                // or: if var.is_some() { alt } else { "" }
                let alt_rash = self.transpile_expression(alternative)?;
                Ok(format!(
                    "{}.as_ref().map(|_| {}).unwrap_or(\"\")",
                    variable, alt_rash
                ))
            }

            BashExpr::StringLength { variable } => {
                // ${#VAR} → var.len()
                Ok(format!("{}.len()", variable))
            }

            BashExpr::RemoveSuffix { variable, pattern } => {
                // ${VAR%pattern} → var.strip_suffix(pattern).unwrap_or(&var)
                let pattern_rash = self.transpile_expression(pattern)?;
                Ok(format!(
                    "{}.strip_suffix({}).unwrap_or(&{})",
                    variable, pattern_rash, variable
                ))
            }


}
}
}

                    include!("codegen_part2_incl2.rs");
