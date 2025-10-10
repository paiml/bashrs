//! Code Generation for Bash-to-Rash Transpiler

use super::patterns::*;
use super::{TranspileError, TranspileResult};
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
            } => {
                let value_rash = self.transpile_expression(value)?;
                let pattern = VariablePattern {
                    exported: *exported,
                };
                Ok(pattern.to_rash(name, &value_rash))
            }

            BashStmt::Command { name, args, .. } => {
                let mut rash_args = Vec::new();
                for arg in args {
                    rash_args.push(self.transpile_expression(arg)?);
                }
                Ok(CommandPattern::to_rash(name, &rash_args))
            }

            BashStmt::Function { name, body, .. } => {
                self.current_indent += 1;
                let mut body_stmts = Vec::new();
                for stmt in body {
                    body_stmts.push(self.transpile_statement(stmt)?);
                }
                let body_rash = body_stmts.join("\n");
                self.current_indent -= 1;

                Ok(FunctionPattern::to_rash(name, &self.indent(&body_rash)))
            }

            BashStmt::If {
                condition,
                then_block,
                elif_blocks,
                else_block,
                ..
            } => {
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

            BashStmt::While {
                condition, body, ..
            } => {
                let cond_rash = self.transpile_test_expression(condition)?;

                self.current_indent += 1;
                let body_rash = self.transpile_block(body)?;
                self.current_indent -= 1;

                Ok(WhilePattern::to_rash(&cond_rash, &body_rash))
            }

            BashStmt::For {
                variable,
                items,
                body,
                ..
            } => {
                let items_rash = self.transpile_expression(items)?;

                self.current_indent += 1;
                let body_rash = self.transpile_block(body)?;
                self.current_indent -= 1;

                Ok(ForPattern::to_rash(variable, &items_rash, &body_rash))
            }

            BashStmt::Return { code, .. } => {
                if let Some(expr) = code {
                    let val = self.transpile_expression(expr)?;
                    Ok(format!("return {};", val))
                } else {
                    Ok("return;".to_string())
                }
            }

            BashStmt::Comment { text, .. } => {
                if self.options.preserve_comments {
                    Ok(format!("//{}", text))
                } else {
                    Ok(String::new())
                }
            }
        }
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
        }
    }

    fn transpile_test_expression(&mut self, expr: &BashExpr) -> TranspileResult<String> {
        match expr {
            BashExpr::Test(test) => self.transpile_test(test),
            _ => self.transpile_expression(expr),
        }
    }

    fn transpile_test(&mut self, test: &TestExpr) -> TranspileResult<String> {
        match test {
            TestExpr::StringEq(a, b) => {
                let a_rash = self.transpile_expression(a)?;
                let b_rash = self.transpile_expression(b)?;
                Ok(format!("{} == {}", a_rash, b_rash))
            }

            TestExpr::StringNe(a, b) => {
                let a_rash = self.transpile_expression(a)?;
                let b_rash = self.transpile_expression(b)?;
                Ok(format!("{} != {}", a_rash, b_rash))
            }

            TestExpr::IntEq(a, b) => {
                let a_rash = self.transpile_expression(a)?;
                let b_rash = self.transpile_expression(b)?;
                Ok(format!("{} == {}", a_rash, b_rash))
            }

            TestExpr::IntNe(a, b) => {
                let a_rash = self.transpile_expression(a)?;
                let b_rash = self.transpile_expression(b)?;
                Ok(format!("{} != {}", a_rash, b_rash))
            }

            TestExpr::IntLt(a, b) => {
                let a_rash = self.transpile_expression(a)?;
                let b_rash = self.transpile_expression(b)?;
                Ok(format!("{} < {}", a_rash, b_rash))
            }

            TestExpr::IntLe(a, b) => {
                let a_rash = self.transpile_expression(a)?;
                let b_rash = self.transpile_expression(b)?;
                Ok(format!("{} <= {}", a_rash, b_rash))
            }

            TestExpr::IntGt(a, b) => {
                let a_rash = self.transpile_expression(a)?;
                let b_rash = self.transpile_expression(b)?;
                Ok(format!("{} > {}", a_rash, b_rash))
            }

            TestExpr::IntGe(a, b) => {
                let a_rash = self.transpile_expression(a)?;
                let b_rash = self.transpile_expression(b)?;
                Ok(format!("{} >= {}", a_rash, b_rash))
            }

            TestExpr::FileExists(path) => {
                let path_rash = self.transpile_expression(path)?;
                Ok(format!("std::path::Path::new(&{}).exists()", path_rash))
            }

            TestExpr::FileReadable(path) => {
                let path_rash = self.transpile_expression(path)?;
                Ok(format!(
                    "std::fs::metadata(&{}).map(|m| m.permissions().readonly()).unwrap_or(false)",
                    path_rash
                ))
            }

            TestExpr::FileWritable(path) => {
                let path_rash = self.transpile_expression(path)?;
                Ok(format!(
                    "std::fs::metadata(&{}).map(|m| !m.permissions().readonly()).unwrap_or(false)",
                    path_rash
                ))
            }

            TestExpr::FileExecutable(path) => {
                let path_rash = self.transpile_expression(path)?;
                Ok(format!("is_executable(&{})", path_rash))
            }

            TestExpr::FileDirectory(path) => {
                let path_rash = self.transpile_expression(path)?;
                Ok(format!("std::path::Path::new(&{}).is_dir()", path_rash))
            }

            TestExpr::StringEmpty(s) => {
                let s_rash = self.transpile_expression(s)?;
                Ok(format!("{}.is_empty()", s_rash))
            }

            TestExpr::StringNonEmpty(s) => {
                let s_rash = self.transpile_expression(s)?;
                Ok(format!("!{}.is_empty()", s_rash))
            }

            TestExpr::And(a, b) => {
                let a_rash = self.transpile_test(a)?;
                let b_rash = self.transpile_test(b)?;
                Ok(format!("({}) && ({})", a_rash, b_rash))
            }

            TestExpr::Or(a, b) => {
                let a_rash = self.transpile_test(a)?;
                let b_rash = self.transpile_test(b)?;
                Ok(format!("({}) || ({})", a_rash, b_rash))
            }

            TestExpr::Not(t) => {
                let t_rash = self.transpile_test(t)?;
                Ok(format!("!({})", t_rash))
            }
        }
    }

    fn transpile_arithmetic(&mut self, arith: &ArithExpr) -> TranspileResult<String> {
        match arith {
            ArithExpr::Number(n) => Ok(n.to_string()),
            ArithExpr::Variable(v) => Ok(v.clone()),
            ArithExpr::Add(a, b) => {
                let a_rash = self.transpile_arithmetic(a)?;
                let b_rash = self.transpile_arithmetic(b)?;
                Ok(format!("({} + {})", a_rash, b_rash))
            }
            ArithExpr::Sub(a, b) => {
                let a_rash = self.transpile_arithmetic(a)?;
                let b_rash = self.transpile_arithmetic(b)?;
                Ok(format!("({} - {})", a_rash, b_rash))
            }
            ArithExpr::Mul(a, b) => {
                let a_rash = self.transpile_arithmetic(a)?;
                let b_rash = self.transpile_arithmetic(b)?;
                Ok(format!("({} * {})", a_rash, b_rash))
            }
            ArithExpr::Div(a, b) => {
                let a_rash = self.transpile_arithmetic(a)?;
                let b_rash = self.transpile_arithmetic(b)?;
                Ok(format!("({} / {})", a_rash, b_rash))
            }
            ArithExpr::Mod(a, b) => {
                let a_rash = self.transpile_arithmetic(a)?;
                let b_rash = self.transpile_arithmetic(b)?;
                Ok(format!("({} % {})", a_rash, b_rash))
            }
        }
    }

    fn indent(&self, code: &str) -> String {
        let indent_str = " ".repeat(self.current_indent * self.options.indent_size);
        code.lines()
            .map(|line| {
                if line.trim().is_empty() {
                    String::new()
                } else {
                    format!("{}{}", indent_str, line)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bash_parser::parser::BashParser;

    #[test]
    fn test_transpile_simple_assignment() {
        let bash_code = "FOO=bar";
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("let FOO"));
        assert!(rash_code.contains("bar"));
    }

    #[test]
    fn test_transpile_function() {
        let bash_code = r#"
function greet() {
    echo "hello"
}
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("fn greet()"));
    }

    #[test]
    fn test_transpile_if_statement() {
        let bash_code = r#"
if [ $x == 1 ]; then
    echo "one"
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("if x == 1"));
    }
}
