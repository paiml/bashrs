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
}

include!("codegen_transpile_until.rs");
