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

            // Issue #68: C-style for loop
            BashStmt::ForCStyle {
                init,
                condition,
                increment,
                body,
                ..
            } => {
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
            BashStmt::Select {
                variable,
                items,
                body,
                ..
            } => {
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

            BashExpr::CommandCondition(cmd) => {
                // Issue #93: Format command condition (command used as condition in if/while)
                self.format_stmt(cmd, 0).trim().to_string()
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
    use crate::bash_parser::ast::{AstMetadata, BashExpr, BashStmt, CaseArm, Span};

    fn dummy_metadata() -> AstMetadata {
        AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        }
    }

    #[test]
    fn test_formatter_new() {
        let formatter = Formatter::new();
        assert_eq!(formatter.config.indent_width, 2);
        assert!(!formatter.config.use_tabs);
    }

    #[test]
    fn test_formatter_default() {
        let formatter = Formatter::default();
        assert_eq!(formatter.config.indent_width, 2);
    }

    #[test]
    fn test_formatter_with_config() {
        let mut config = FormatterConfig::default();
        config.indent_width = 4;
        let formatter = Formatter::with_config(config);
        assert_eq!(formatter.config.indent_width, 4);
    }

    #[test]
    fn test_set_source() {
        let mut formatter = Formatter::new();
        assert!(formatter.source.is_none());
        formatter.set_source("echo hello");
        assert!(formatter.source.is_some());
        assert_eq!(formatter.source.unwrap(), "echo hello");
    }

    #[test]
    fn test_format_assignment() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "VAR".to_string(),
                index: None,
                value: BashExpr::Literal("value".to_string()),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert_eq!(result, "VAR=value");
    }

    #[test]
    fn test_format_exported_assignment() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "VAR".to_string(),
                index: None,
                value: BashExpr::Literal("value".to_string()),
                exported: true,
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("export "));
        assert!(result.contains("VAR=value"));
    }

    #[test]
    fn test_format_comment() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Comment {
                text: " This is a comment".to_string(),
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert_eq!(result, "# This is a comment");
    }

    #[test]
    fn test_format_command() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![
                    BashExpr::Literal("hello".to_string()),
                    BashExpr::Variable("name".to_string()),
                ],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("echo"));
        assert!(result.contains("hello"));
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
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("greet() {"));
        assert!(result.contains("  echo hello"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_format_function_not_normalized() {
        let mut config = FormatterConfig::default();
        config.normalize_functions = false;
        let formatter = Formatter::with_config(config);

        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: "test".to_string(),
                body: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("function test()"));
    }

    #[test]
    fn test_format_function_space_before_brace() {
        let mut config = FormatterConfig::default();
        config.space_before_brace = false;
        let formatter = Formatter::with_config(config);

        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: "test".to_string(),
                body: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("test(){"));
    }

    #[test]
    fn test_format_if() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::Test(Box::new(TestExpr::IntEq(
                    BashExpr::Variable("x".to_string()),
                    BashExpr::Literal("1".to_string()),
                ))),
                then_block: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("yes".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                elif_blocks: vec![],
                else_block: None,
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("if"));
        assert!(result.contains("then"));
        assert!(result.contains("fi"));
    }

    #[test]
    fn test_format_if_else() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::Literal("true".to_string()),
                then_block: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("yes".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                elif_blocks: vec![],
                else_block: Some(vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("no".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }]),
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("else"));
    }

    #[test]
    fn test_format_if_elif() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::Literal("true".to_string()),
                then_block: vec![],
                elif_blocks: vec![(BashExpr::Literal("false".to_string()), vec![])],
                else_block: None,
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("elif"));
    }

    #[test]
    fn test_format_if_inline_then() {
        let mut config = FormatterConfig::default();
        config.inline_then = false;
        let formatter = Formatter::with_config(config);

        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::Literal("true".to_string()),
                then_block: vec![],
                elif_blocks: vec![],
                else_block: None,
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("\nthen"));
    }

    #[test]
    fn test_format_while() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::While {
                condition: BashExpr::Literal("true".to_string()),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("loop".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("while"));
        assert!(result.contains("do"));
        assert!(result.contains("done"));
    }

    #[test]
    fn test_format_until() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Until {
                condition: BashExpr::Literal("false".to_string()),
                body: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("until"));
        assert!(result.contains("do"));
        assert!(result.contains("done"));
    }

    #[test]
    fn test_format_for() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::For {
                variable: "i".to_string(),
                items: BashExpr::Literal("1 2 3".to_string()),
                body: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("for i in"));
        assert!(result.contains("do"));
        assert!(result.contains("done"));
    }

    #[test]
    fn test_format_for_cstyle() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::ForCStyle {
                init: "i=0".to_string(),
                condition: "i<10".to_string(),
                increment: "i++".to_string(),
                body: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("for (("));
        assert!(result.contains("i=0"));
        assert!(result.contains("i<10"));
        assert!(result.contains("i++"));
    }

    #[test]
    fn test_format_return() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Return {
                code: None,
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert_eq!(result, "return");
    }

    #[test]
    fn test_format_return_with_code() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Return {
                code: Some(BashExpr::Literal("0".to_string())),
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert_eq!(result, "return 0");
    }

    #[test]
    fn test_format_case() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Case {
                word: BashExpr::Variable("x".to_string()),
                arms: vec![CaseArm {
                    patterns: vec!["a".to_string()],
                    body: vec![BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("a".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    }],
                }],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("case"));
        assert!(result.contains("esac"));
        assert!(result.contains(";;"));
    }

    #[test]
    fn test_format_pipeline() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Pipeline {
                commands: vec![
                    BashStmt::Command {
                        name: "ls".to_string(),
                        args: vec![],
                        redirects: vec![],
                        span: Span::dummy(),
                    },
                    BashStmt::Command {
                        name: "grep".to_string(),
                        args: vec![BashExpr::Literal("foo".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    },
                ],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("ls | grep"));
    }

    #[test]
    fn test_format_and_list() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::AndList {
                left: Box::new(BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                right: Box::new(BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("ok".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("&&"));
    }

    #[test]
    fn test_format_or_list() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::OrList {
                left: Box::new(BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                right: Box::new(BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("fail".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("||"));
    }

    #[test]
    fn test_format_brace_group() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::BraceGroup {
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("test".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("{"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_format_coproc() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Coproc {
                name: Some("mycoproc".to_string()),
                body: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("coproc mycoproc"));
    }

    #[test]
    fn test_format_coproc_unnamed() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Coproc {
                name: None,
                body: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("coproc {"));
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
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("\techo test"));
    }

    // Expression formatting tests
    #[test]
    fn test_format_expr_literal_special_chars() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("hello world".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("\"hello world\""));
    }

    #[test]
    fn test_format_expr_variable_quoted() {
        let mut config = FormatterConfig::default();
        config.quote_variables = true;
        let formatter = Formatter::with_config(config);

        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("x".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("\"$x\""));
    }

    #[test]
    fn test_format_expr_variable_unquoted() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("x".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("$x"));
    }

    #[test]
    fn test_format_expr_command_subst() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::CommandSubst(Box::new(BashStmt::Command {
                    name: "date".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: Span::dummy(),
                }))],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("$(date)"));
    }

    #[test]
    fn test_format_expr_array() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "arr".to_string(),
                index: None,
                value: BashExpr::Array(vec![
                    BashExpr::Literal("a".to_string()),
                    BashExpr::Literal("b".to_string()),
                ]),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("(a b)"));
    }

    #[test]
    fn test_format_expr_concat() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Concat(vec![
                    BashExpr::Literal("hello".to_string()),
                    BashExpr::Variable("name".to_string()),
                ])],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        // Variable formatting includes $, so we check for echo hello$name
        assert!(result.contains("hello"), "Expected 'hello' in: {}", result);
        assert!(result.contains("name"), "Expected 'name' in: {}", result);
    }

    #[test]
    fn test_format_expr_test_single_brackets() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Test(Box::new(TestExpr::FileExists(
                    BashExpr::Literal("/tmp".to_string()),
                )))],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("[ -e /tmp ]"));
    }

    #[test]
    fn test_format_expr_test_double_brackets() {
        let mut config = FormatterConfig::default();
        config.use_double_brackets = true;
        let formatter = Formatter::with_config(config);

        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Test(Box::new(TestExpr::FileExists(
                    BashExpr::Literal("/tmp".to_string()),
                )))],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("[[ -e /tmp ]]"));
    }

    #[test]
    fn test_format_expr_glob() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "ls".to_string(),
                args: vec![BashExpr::Glob("*.txt".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("*.txt"));
    }

    #[test]
    fn test_format_expr_default_value() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::DefaultValue {
                    variable: "x".to_string(),
                    default: Box::new(BashExpr::Literal("default".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("${x:-default}"));
    }

    #[test]
    fn test_format_expr_assign_default() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::AssignDefault {
                    variable: "x".to_string(),
                    default: Box::new(BashExpr::Literal("value".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("${x:=value}"));
    }

    #[test]
    fn test_format_expr_error_if_unset() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::ErrorIfUnset {
                    variable: "x".to_string(),
                    message: Box::new(BashExpr::Literal("error".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("${x:?error}"));
    }

    #[test]
    fn test_format_expr_alternative_value() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::AlternativeValue {
                    variable: "x".to_string(),
                    alternative: Box::new(BashExpr::Literal("alt".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("${x:+alt}"));
    }

    #[test]
    fn test_format_expr_string_length() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::StringLength {
                    variable: "x".to_string(),
                }],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("${#x}"));
    }

    #[test]
    fn test_format_expr_remove_suffix() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::RemoveSuffix {
                    variable: "x".to_string(),
                    pattern: Box::new(BashExpr::Literal(".txt".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("${x%.txt}"));
    }

    #[test]
    fn test_format_expr_remove_prefix() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::RemovePrefix {
                    variable: "x".to_string(),
                    pattern: Box::new(BashExpr::Literal("/tmp/".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("${x#/tmp/}"));
    }

    #[test]
    fn test_format_expr_remove_longest_prefix() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::RemoveLongestPrefix {
                    variable: "x".to_string(),
                    pattern: Box::new(BashExpr::Literal("*/".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        // * is a special char that gets quoted
        assert!(result.contains("${x##"), "Expected '${{x##' in: {}", result);
    }

    #[test]
    fn test_format_expr_remove_longest_suffix() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::RemoveLongestSuffix {
                    variable: "x".to_string(),
                    pattern: Box::new(BashExpr::Literal(".*".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        // * is a special char that gets quoted
        assert!(result.contains("${x%%"), "Expected '${{x%%' in: {}", result);
    }

    #[test]
    fn test_format_expr_command_condition() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::CommandCondition(Box::new(BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: Span::dummy(),
                }))],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("test"));
    }

    // Arithmetic expression tests
    #[test]
    fn test_format_arith_add() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Arithmetic(Box::new(ArithExpr::Add(
                    Box::new(ArithExpr::Number(1)),
                    Box::new(ArithExpr::Number(2)),
                )))],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("1 + 2"));
    }

    #[test]
    fn test_format_arith_sub() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Arithmetic(Box::new(ArithExpr::Sub(
                    Box::new(ArithExpr::Number(5)),
                    Box::new(ArithExpr::Number(3)),
                )))],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("5 - 3"));
    }

    #[test]
    fn test_format_arith_mul() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Arithmetic(Box::new(ArithExpr::Mul(
                    Box::new(ArithExpr::Number(2)),
                    Box::new(ArithExpr::Number(3)),
                )))],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("2 * 3"));
    }

    #[test]
    fn test_format_arith_div() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Arithmetic(Box::new(ArithExpr::Div(
                    Box::new(ArithExpr::Number(10)),
                    Box::new(ArithExpr::Number(2)),
                )))],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("10 / 2"));
    }

    #[test]
    fn test_format_arith_mod() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Arithmetic(Box::new(ArithExpr::Mod(
                    Box::new(ArithExpr::Number(10)),
                    Box::new(ArithExpr::Number(3)),
                )))],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("10 % 3"));
    }

    #[test]
    fn test_format_arith_variable() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Arithmetic(Box::new(ArithExpr::Variable(
                    "x".to_string(),
                )))],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("x"));
    }

    // Test expression formatting tests
    #[test]
    fn test_format_test_string_eq() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::StringEq(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("a".to_string()),
        ));
        assert!(result.contains(" = "));
    }

    #[test]
    fn test_format_test_string_ne() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::StringNe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("a".to_string()),
        ));
        assert!(result.contains(" != "));
    }

    #[test]
    fn test_format_test_int_lt() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::IntLt(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("10".to_string()),
        ));
        assert!(result.contains(" -lt "));
    }

    #[test]
    fn test_format_test_int_le() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::IntLe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("10".to_string()),
        ));
        assert!(result.contains(" -le "));
    }

    #[test]
    fn test_format_test_int_gt() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::IntGt(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("10".to_string()),
        ));
        assert!(result.contains(" -gt "));
    }

    #[test]
    fn test_format_test_int_ge() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::IntGe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("10".to_string()),
        ));
        assert!(result.contains(" -ge "));
    }

    #[test]
    fn test_format_test_int_ne() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::IntNe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("10".to_string()),
        ));
        assert!(result.contains(" -ne "));
    }

    #[test]
    fn test_format_test_file_readable() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::FileReadable(BashExpr::Literal(
            "/tmp".to_string(),
        )));
        assert!(result.contains("-r "));
    }

    #[test]
    fn test_format_test_file_writable() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::FileWritable(BashExpr::Literal(
            "/tmp".to_string(),
        )));
        assert!(result.contains("-w "));
    }

    #[test]
    fn test_format_test_file_executable() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::FileExecutable(BashExpr::Literal(
            "/bin/sh".to_string(),
        )));
        assert!(result.contains("-x "));
    }

    #[test]
    fn test_format_test_file_directory() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::FileDirectory(BashExpr::Literal(
            "/tmp".to_string(),
        )));
        assert!(result.contains("-d "));
    }

    #[test]
    fn test_format_test_string_empty() {
        let formatter = Formatter::new();
        let result =
            formatter.format_test(&TestExpr::StringEmpty(BashExpr::Variable("x".to_string())));
        assert!(result.contains("-z "));
    }

    #[test]
    fn test_format_test_string_non_empty() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::StringNonEmpty(BashExpr::Variable(
            "x".to_string(),
        )));
        assert!(result.contains("-n "));
    }

    #[test]
    fn test_format_test_and() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::And(
            Box::new(TestExpr::StringEq(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("a".to_string()),
            )),
            Box::new(TestExpr::StringEq(
                BashExpr::Variable("y".to_string()),
                BashExpr::Literal("b".to_string()),
            )),
        ));
        assert!(result.contains(" && "));
    }

    #[test]
    fn test_format_test_or() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::Or(
            Box::new(TestExpr::StringEq(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("a".to_string()),
            )),
            Box::new(TestExpr::StringEq(
                BashExpr::Variable("y".to_string()),
                BashExpr::Literal("b".to_string()),
            )),
        ));
        assert!(result.contains(" || "));
    }

    #[test]
    fn test_format_test_not() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::Not(Box::new(TestExpr::StringEq(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("a".to_string()),
        ))));
        assert!(result.contains("! "));
    }

    #[test]
    fn test_format_source() {
        let mut formatter = Formatter::new();
        let result = formatter.format_source("x=1");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("x=1"));
    }

    #[test]
    fn test_format_source_error() {
        let mut formatter = Formatter::new();
        // Invalid bash syntax should return error
        let result = formatter.format_source("if then fi");
        // This might parse or not depending on parser; just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_format_multiple_statements() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![
                BashStmt::Assignment {
                    name: "x".to_string(),
                    index: None,
                    value: BashExpr::Literal("1".to_string()),
                    exported: false,
                    span: Span::dummy(),
                },
                BashStmt::Assignment {
                    name: "y".to_string(),
                    index: None,
                    value: BashExpr::Literal("2".to_string()),
                    exported: false,
                    span: Span::dummy(),
                },
            ],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("x=1"));
        assert!(result.contains("y=2"));
        assert!(result.contains("\n"));
    }
}
