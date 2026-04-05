//! Formatter expression methods — extracted for file health.

use super::formatter::Formatter;
use crate::bash_parser::ast::{ArithExpr, BashExpr, TestExpr};

impl Formatter {
    /// Format an expression
    pub(crate) fn format_expr(&self, expr: &BashExpr) -> String {
        match expr {
            BashExpr::Literal(s) => {
                // Quote literals if they contain special characters
                if s.contains(' ') || s.contains('$') || s.contains('*') {
                    format!("\"{}\"", s)
                } else {
                    s.clone()
                }
            }

            BashExpr::Variable(name) => {
                if self.config.quote_variables {
                    format!("\"${}\"", name)
                } else {
                    format!("${}", name)
                }
            }

            BashExpr::CommandSubst(cmd) => {
                format!("$({})", self.format_stmt(cmd, 0).trim())
            }

            BashExpr::Arithmetic(arith) => {
                format!("$(({})", self.format_arith(arith))
            }

            BashExpr::Array(items) => {
                let formatted_items: Vec<String> =
                    items.iter().map(|item| self.format_expr(item)).collect();
                format!("({})", formatted_items.join(" "))
            }

            BashExpr::Concat(exprs) => exprs
                .iter()
                .map(|e| self.format_expr(e))
                .collect::<String>(),

            BashExpr::Test(test) => {
                if self.config.use_double_brackets {
                    format!("[[ {} ]]", self.format_test(test))
                } else {
                    format!("[ {} ]", self.format_test(test))
                }
            }

            BashExpr::Glob(pattern) => pattern.clone(),

            BashExpr::DefaultValue { variable, default } => {
                format!("${{{}:-{}}}", variable, self.format_expr(default))
            }

            BashExpr::AssignDefault { variable, default } => {
                format!("${{{}:={}}}", variable, self.format_expr(default))
            }

            BashExpr::ErrorIfUnset { variable, message } => {
                format!("${{{}:?{}}}", variable, self.format_expr(message))
            }

            BashExpr::AlternativeValue {
                variable,
                alternative,
            } => {
                format!("${{{}:+{}}}", variable, self.format_expr(alternative))
            }

            BashExpr::StringLength { variable } => {
                format!("${{#{}}}", variable)
            }

            BashExpr::RemoveSuffix { variable, pattern } => {
                format!("${{{}%{}}}", variable, self.format_expr(pattern))
            }

            BashExpr::RemovePrefix { variable, pattern } => {
                format!("${{{}#{}}}", variable, self.format_expr(pattern))
            }

            BashExpr::RemoveLongestPrefix { variable, pattern } => {
                format!("${{{}##{}}}", variable, self.format_expr(pattern))
            }

            BashExpr::RemoveLongestSuffix { variable, pattern } => {
                format!("${{{}%%{}}}", variable, self.format_expr(pattern))
            }

            BashExpr::CommandCondition(cmd) => {
                // Issue #93: Format command condition (command used as condition in if/while)
                self.format_stmt(cmd, 0).trim().to_string()
            }
        }
    }

    /// Format arithmetic expression
    pub(crate) fn format_arith(&self, arith: &ArithExpr) -> String {
        match arith {
            ArithExpr::Number(n) => n.to_string(),
            ArithExpr::Variable(v) => v.clone(),
            ArithExpr::Add(left, right) => {
                format!("{} + {}", self.format_arith(left), self.format_arith(right))
            }
            ArithExpr::Sub(left, right) => {
                format!("{} - {}", self.format_arith(left), self.format_arith(right))
            }
            ArithExpr::Mul(left, right) => {
                format!("{} * {}", self.format_arith(left), self.format_arith(right))
            }
            ArithExpr::Div(left, right) => {
                format!("{} / {}", self.format_arith(left), self.format_arith(right))
            }
            ArithExpr::Mod(left, right) => {
                format!("{} % {}", self.format_arith(left), self.format_arith(right))
            }
        }
    }

    /// Format test expression
    pub(crate) fn format_test(&self, test: &TestExpr) -> String {
        match test {
            TestExpr::StringEq(left, right) => {
                format!("{} = {}", self.format_expr(left), self.format_expr(right))
            }
            TestExpr::StringNe(left, right) => {
                format!("{} != {}", self.format_expr(left), self.format_expr(right))
            }
            TestExpr::IntEq(left, right) => {
                format!("{} -eq {}", self.format_expr(left), self.format_expr(right))
            }
            TestExpr::IntNe(left, right) => {
                format!("{} -ne {}", self.format_expr(left), self.format_expr(right))
            }
            TestExpr::IntLt(left, right) => {
                format!("{} -lt {}", self.format_expr(left), self.format_expr(right))
            }
            TestExpr::IntLe(left, right) => {
                format!("{} -le {}", self.format_expr(left), self.format_expr(right))
            }
            TestExpr::IntGt(left, right) => {
                format!("{} -gt {}", self.format_expr(left), self.format_expr(right))
            }
            TestExpr::IntGe(left, right) => {
                format!("{} -ge {}", self.format_expr(left), self.format_expr(right))
            }
            TestExpr::FileExists(expr) => {
                format!("-e {}", self.format_expr(expr))
            }
            TestExpr::FileReadable(expr) => {
                format!("-r {}", self.format_expr(expr))
            }
            TestExpr::FileWritable(expr) => {
                format!("-w {}", self.format_expr(expr))
            }
            TestExpr::FileExecutable(expr) => {
                format!("-x {}", self.format_expr(expr))
            }
            TestExpr::FileDirectory(expr) => {
                format!("-d {}", self.format_expr(expr))
            }
            TestExpr::StringEmpty(expr) => {
                format!("-z {}", self.format_expr(expr))
            }
            TestExpr::StringNonEmpty(expr) => {
                format!("-n {}", self.format_expr(expr))
            }
            TestExpr::And(left, right) => {
                format!("{} && {}", self.format_test(left), self.format_test(right))
            }
            TestExpr::Or(left, right) => {
                format!("{} || {}", self.format_test(left), self.format_test(right))
            }
            TestExpr::Not(test) => {
                format!("! {}", self.format_test(test))
            }
        }
    }

    /// Make indentation string
    pub(crate) fn make_indent(&self, indent: usize) -> String {
        if self.config.use_tabs {
            "\t".repeat(indent)
        } else {
            " ".repeat(indent * self.config.indent_width)
        }
    }
}
