// formatter.rs - Bash script formatter
// Following ruchy design patterns for code formatting
use crate::bash_parser::ast::{ArithExpr, BashAst, BashExpr, BashStmt, TestExpr};
use crate::bash_quality::formatter_config::FormatterConfig;
use anyhow::Result;

pub struct Formatter {
    config: FormatterConfig,
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

    /// Format a statement
    fn format_stmt(&self, stmt: &BashStmt, indent: usize) -> String {
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

            BashStmt::If {
                condition,
                then_block,
                elif_blocks,
                else_block,
                ..
            } => {
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

            BashStmt::While {
                condition, body, ..
            } => {
                let mut result = format!("{}while ", indent_str);
                result.push_str(&self.format_expr(condition));
                result.push_str("; do\n");

                for stmt in body {
                    result.push_str(&self.format_stmt(stmt, indent + 1));
                    result.push('\n');
                }

                result.push_str(&format!("{}done", indent_str));
                result
            }

            BashStmt::Until {
                condition, body, ..
            } => {
                let mut result = format!("{}until ", indent_str);
                result.push_str(&self.format_expr(condition));
                result.push_str("; do\n");

                for stmt in body {
                    result.push_str(&self.format_stmt(stmt, indent + 1));
                    result.push('\n');
                }

                result.push_str(&format!("{}done", indent_str));
                result
            }

            BashStmt::For {
                variable,
                items,
                body,
                ..
            } => {
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

            BashStmt::Return { code, .. } => {
                if let Some(expr) = code {
                    format!("{}return {}", indent_str, self.format_expr(expr))
                } else {
                    format!("{}return", indent_str)
                }
            }

            BashStmt::Case { word, arms, .. } => {
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

            BashStmt::Pipeline { commands, .. } => {
                // Format pipeline with proper spacing: cmd1 | cmd2 | cmd3
                let formatted_cmds: Vec<String> = commands
                    .iter()
                    .map(|cmd| self.format_stmt(cmd, 0).trim().to_string())
                    .collect();
                format!("{}{}", indent_str, formatted_cmds.join(" | "))
            }
        }
    }

    /// Format an expression
    fn format_expr(&self, expr: &BashExpr) -> String {
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
                .collect::<Vec<_>>()
                .join(""),

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
        }
    }

    /// Format arithmetic expression
    fn format_arith(&self, arith: &ArithExpr) -> String {
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
    fn format_test(&self, test: &TestExpr) -> String {
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
    fn make_indent(&self, indent: usize) -> String {
        if self.config.use_tabs {
            "\t".repeat(indent)
        } else {
            " ".repeat(indent * self.config.indent_width)
        }
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bash_parser::ast::{AstMetadata, BashExpr, BashStmt, Span};

    #[test]
    fn test_formatter_new() {
        let formatter = Formatter::new();
        assert_eq!(formatter.config.indent_width, 2);
        assert!(!formatter.config.use_tabs);
    }

    #[test]
    fn test_format_assignment() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "VAR".to_string(),
                value: BashExpr::Literal("value".to_string()),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let result = formatter.format(&ast).unwrap();
        assert_eq!(result, "VAR=value");
    }

    #[test]
    fn test_format_function() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: "greet".to_string(),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("hello".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 3,
                parse_time_ms: 0,
            },
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("greet() {"));
        assert!(result.contains("  echo hello"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_format_with_tabs() {
        let mut config = FormatterConfig::default();
        config.use_tabs = true;
        let formatter = Formatter::with_config(config);

        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: "test".to_string(),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("test".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 3,
                parse_time_ms: 0,
            },
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("\techo test"));
    }
}
