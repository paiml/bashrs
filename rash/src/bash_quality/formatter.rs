// formatter.rs - Bash script formatter
// Following ruchy design patterns for code formatting
use crate::bash_parser::ast::{BashAst, BashExpr, BashStmt};
use crate::bash_quality::formatter_config::FormatterConfig;
use anyhow::Result;

pub struct Formatter {
    pub(crate) config: FormatterConfig,
    source: Option<String>,
}

impl Formatter {
    /// Create a new formatter with default configuration
    pub fn new() -> Self {
        Self::with_config(FormatterConfig::default())
    }

    /// Create a new formatter with custom configuration
    pub fn with_config(config: FormatterConfig) -> Self {
        Self {
            config,
            source: None,
        }
    }

    /// Set the original source text for ignore directives
    pub fn set_source(&mut self, source: impl Into<String>) {
        self.source = Some(source.into());
    }

    /// Format a bash AST
    pub fn format(&self, ast: &BashAst) -> Result<String> {
        let mut result = String::new();

        for (i, stmt) in ast.statements.iter().enumerate() {
            if i > 0 {
                result.push('\n');
            }
            result.push_str(&self.format_stmt(stmt, 0));
        }

        Ok(result)
    }

    /// Format a bash script from source text
    pub fn format_source(&mut self, source: &str) -> Result<String> {
        use crate::bash_parser::BashParser;

        self.set_source(source);
        let mut parser = BashParser::new(source)?;
        let ast = parser
            .parse()
            .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

        self.format(&ast)
    }

    /// Format a statement (thin dispatcher)
    pub(crate) fn format_stmt(&self, stmt: &BashStmt, indent: usize) -> String {
        let indent_str = self.make_indent(indent);

        match stmt {
            BashStmt::Comment { text, .. } => {
                format!("{}#{}", indent_str, text)
            }

            BashStmt::Assignment {
                name,
                value,
                exported,
                ..
            } => {
                let export = if *exported { "export " } else { "" };
                format!(
                    "{}{}{}={}",
                    indent_str,
                    export,
                    name,
                    self.format_expr(value)
                )
            }

            BashStmt::Command { name, args, .. } => {
                let mut result = format!("{}{}", indent_str, name);
                for arg in args {
                    result.push(' ');
                    result.push_str(&self.format_expr(arg));
                }
                result
            }

            BashStmt::Return { code, .. } => {
                if let Some(expr) = code {
                    format!("{}return {}", indent_str, self.format_expr(expr))
                } else {
                    format!("{}return", indent_str)
                }
            }

            // Control flow: if/for/while/until/case/select
            BashStmt::If { .. }
            | BashStmt::While { .. }
            | BashStmt::Until { .. }
            | BashStmt::For { .. }
            | BashStmt::ForCStyle { .. }
            | BashStmt::Case { .. }
            | BashStmt::Select { .. } => self.format_control_flow_stmt(stmt, indent, &indent_str),

            // Compound: function/pipeline/and/or/brace/coproc/negated
            BashStmt::Function { .. }
            | BashStmt::Pipeline { .. }
            | BashStmt::AndList { .. }
            | BashStmt::OrList { .. }
            | BashStmt::BraceGroup { .. }
            | BashStmt::Coproc { .. }
            | BashStmt::Negated { .. } => self.format_compound_stmt(stmt, indent, &indent_str),
        }
    }

    /// Format control flow statements: if/for/while/until/case/select
    fn format_control_flow_stmt(&self, stmt: &BashStmt, indent: usize, indent_str: &str) -> String {
        match stmt {
            BashStmt::If {
                condition,
                then_block,
                elif_blocks,
                else_block,
                ..
            } => self.format_if_stmt(
                condition,
                then_block,
                elif_blocks,
                else_block,
                indent,
                indent_str,
            ),

            BashStmt::While {
                condition, body, ..
            } => self.format_loop_stmt("while", condition, body, indent, indent_str),

            BashStmt::Until {
                condition, body, ..
            } => self.format_loop_stmt("until", condition, body, indent, indent_str),

            BashStmt::For {
                variable,
                items,
                body,
                ..
            } => self.format_for_stmt(variable, items, body, indent, indent_str),

            BashStmt::ForCStyle {
                init,
                condition,
                increment,
                body,
                ..
            } => self.format_for_c_style_stmt(init, condition, increment, body, indent, indent_str),

            BashStmt::Case { word, arms, .. } => {
                self.format_case_stmt(word, arms, indent, indent_str)
            }

            BashStmt::Select {
                variable,
                items,
                body,
                ..
            } => self.format_select_stmt(variable, items, body, indent, indent_str),

            // Unreachable: caller only passes control flow variants
            _ => unreachable!(),
        }
    }

    /// Format an if/elif/else statement
    fn format_if_stmt(
        &self,
        condition: &BashExpr,
        then_block: &[BashStmt],
        elif_blocks: &[(BashExpr, Vec<BashStmt>)],
        else_block: &Option<Vec<BashStmt>>,
        indent: usize,
        indent_str: &str,
    ) -> String {
        let mut result = format!("{}if ", indent_str);
        result.push_str(&self.format_expr(condition));

        if self.config.inline_then {
            result.push_str("; then");
        } else {
            result.push_str("\nthen");
        }
        result.push('\n');

        for stmt in then_block {
            result.push_str(&self.format_stmt(stmt, indent + 1));
            result.push('\n');
        }

        for (cond, block) in elif_blocks {
            result.push_str(&format!("{}elif ", indent_str));
            result.push_str(&self.format_expr(cond));
            if self.config.inline_then {
                result.push_str("; then\n");
            } else {
                result.push_str("\nthen\n");
            }
            for stmt in block {
                result.push_str(&self.format_stmt(stmt, indent + 1));
                result.push('\n');
            }
        }

        if let Some(else_stmts) = else_block {
            result.push_str(&format!("{}else\n", indent_str));
            for stmt in else_stmts {
                result.push_str(&self.format_stmt(stmt, indent + 1));
                result.push('\n');
            }
        }

        result.push_str(&format!("{}fi", indent_str));
        result
    }

    /// Format while/until loop (shared logic)
    fn format_loop_stmt(
        &self,
        keyword: &str,
        condition: &BashExpr,
        body: &[BashStmt],
        indent: usize,
        indent_str: &str,
    ) -> String {
        let mut result = format!("{}{} ", indent_str, keyword);
        result.push_str(&self.format_expr(condition));
        result.push_str("; do\n");

        for stmt in body {
            result.push_str(&self.format_stmt(stmt, indent + 1));
            result.push('\n');
        }

        result.push_str(&format!("{}done", indent_str));
        result
    }

    /// Format a for-in loop
    fn format_for_stmt(
        &self,
        variable: &str,
        items: &BashExpr,
        body: &[BashStmt],
        indent: usize,
        indent_str: &str,
    ) -> String {
        let mut result = format!("{}for {} in ", indent_str, variable);
        result.push_str(&self.format_expr(items));
        result.push_str("; do\n");

        for stmt in body {
            result.push_str(&self.format_stmt(stmt, indent + 1));
            result.push('\n');
        }

        result.push_str(&format!("{}done", indent_str));
        result
    }

    /// Format a C-style for loop
    fn format_for_c_style_stmt(
        &self,
        init: &str,
        condition: &str,
        increment: &str,
        body: &[BashStmt],
        indent: usize,
        indent_str: &str,
    ) -> String {
        let mut result = format!(
            "{}for (({}; {}; {})); do\n",
            indent_str, init, condition, increment
        );

        for stmt in body {
            result.push_str(&self.format_stmt(stmt, indent + 1));
            result.push('\n');
        }

        result.push_str(&format!("{}done", indent_str));
        result
    }

    /// Format a case statement
    fn format_case_stmt(
        &self,
        word: &BashExpr,
        arms: &[crate::bash_parser::ast::CaseArm],
        indent: usize,
        indent_str: &str,
    ) -> String {
        let mut result = format!("{}case {} in", indent_str, self.format_expr(word));
        result.push('\n');

        for arm in arms {
            // Format pattern(s)
            let pattern_str = arm.patterns.join("|");
            result.push_str(&format!("{}  {})", indent_str, pattern_str));
            result.push('\n');

            // Format body
            for stmt in &arm.body {
                result.push_str(&self.format_stmt(stmt, indent + 2));
                result.push('\n');
            }

            // Add ;;
            result.push_str(&format!("{}    ;;", indent_str));
            result.push('\n');
        }

        result.push_str(&format!("{}esac", indent_str));
        result
    }

    /// Format a select statement
    fn format_select_stmt(
        &self,
        variable: &str,
        items: &BashExpr,
        body: &[BashStmt],
        indent: usize,
        indent_str: &str,
    ) -> String {
        // F017: Format select statement
        let items_str = self.format_expr(items);
        let body_stmts: Vec<String> = body
            .iter()
            .map(|s| self.format_stmt(s, indent + 1))
            .collect();
        format!(
            "{}select {} in {}; do\n{}\n{}done",
            indent_str,
            variable,
            items_str,
            body_stmts.join("\n"),
            indent_str
        )
    }

    /// Format compound statements: function/pipeline/and/or/brace/coproc/negated
    fn format_compound_stmt(&self, stmt: &BashStmt, indent: usize, indent_str: &str) -> String {
        match stmt {
            BashStmt::Function { name, body, .. } => {
                let brace_space = if self.config.space_before_brace {
                    " "
                } else {
                    ""
                };
                let mut result = if self.config.normalize_functions {
                    format!("{}{}(){}{{", indent_str, name, brace_space)
                } else {
                    format!("{}function {}(){}{{", indent_str, name, brace_space)
                };
                result.push('\n');

                for stmt in body {
                    result.push_str(&self.format_stmt(stmt, indent + 1));
                    result.push('\n');
                }

                result.push_str(&format!("{}}}", indent_str));
                result
            }

            BashStmt::Pipeline { commands, .. } => {
                // Format pipeline with proper spacing: cmd1 | cmd2 | cmd3
                let formatted_cmds: Vec<String> = commands
                    .iter()
                    .map(|cmd| self.format_stmt(cmd, 0).trim().to_string())
                    .collect();
                format!("{}{}", indent_str, formatted_cmds.join(" | "))
            }

            BashStmt::AndList { left, right, .. } => {
                // Format AND list: cmd1 && cmd2
                let left_fmt = self.format_stmt(left, 0).trim().to_string();
                let right_fmt = self.format_stmt(right, 0).trim().to_string();
                format!("{}{} && {}", indent_str, left_fmt, right_fmt)
            }

            BashStmt::OrList { left, right, .. } => {
                // Format OR list: cmd1 || cmd2
                let left_fmt = self.format_stmt(left, 0).trim().to_string();
                let right_fmt = self.format_stmt(right, 0).trim().to_string();
                format!("{}{} || {}", indent_str, left_fmt, right_fmt)
            }

            BashStmt::BraceGroup { body, .. } => {
                // Format brace group: { cmd1; cmd2; }
                let stmts: Vec<String> = body
                    .iter()
                    .map(|s| self.format_stmt(s, 0).trim().to_string())
                    .collect();
                format!("{}{{ {}; }}", indent_str, stmts.join("; "))
            }

            BashStmt::Coproc { name, body, .. } => {
                // Format coproc: coproc NAME { cmd; }
                let stmts: Vec<String> = body
                    .iter()
                    .map(|s| self.format_stmt(s, 0).trim().to_string())
                    .collect();
                if let Some(n) = name {
                    format!("{}coproc {} {{ {}; }}", indent_str, n, stmts.join("; "))
                } else {
                    format!("{}coproc {{ {}; }}", indent_str, stmts.join("; "))
                }
            }

            BashStmt::Negated { command, .. } => {
                // Issue #133: Format negated command: ! cmd
                format!("{}! {}", indent_str, self.format_stmt(command, 0).trim())
            }

            // Unreachable: caller only passes compound variants
            _ => unreachable!(),
        }
    }
}
impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "formatter_tests_dummy_metada.rs"]
mod tests_ext;
