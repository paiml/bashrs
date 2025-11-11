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

            BashStmt::Until {
                condition, body, ..
            } => {
                // Until loop transpiles to while with negated condition
                let cond_rash = self.transpile_test_expression(condition)?;
                let negated_cond = format!("!({})", cond_rash);

                self.current_indent += 1;
                let body_rash = self.transpile_block(body)?;
                self.current_indent -= 1;

                Ok(WhilePattern::to_rash(&negated_cond, &body_rash))
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

            BashStmt::Case { word, arms, .. } => {
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

            BashStmt::Pipeline { commands, .. } => {
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

            BashExpr::RemovePrefix { variable, pattern } => {
                // ${VAR#pattern} → var.strip_prefix(pattern).unwrap_or(&var)
                let pattern_rash = self.transpile_expression(pattern)?;
                Ok(format!(
                    "{}.strip_prefix({}).unwrap_or(&{})",
                    variable, pattern_rash, variable
                ))
            }

            BashExpr::RemoveLongestPrefix { variable, pattern } => {
                // ${VAR##pattern} → var.rsplit_once(pattern).map_or(&var, |(_, suffix)| suffix)
                // For greedy prefix removal, find last occurrence of pattern
                let pattern_rash = self.transpile_expression(pattern)?;
                Ok(format!(
                    "{}.rsplit_once({}).map_or(&{}, |(_, suffix)| suffix)",
                    variable, pattern_rash, variable
                ))
            }

            BashExpr::RemoveLongestSuffix { variable, pattern } => {
                // ${VAR%%pattern} → var.split_once(pattern).map_or(&var, |(prefix, _)| prefix)
                // For greedy suffix removal, find first occurrence of pattern
                let pattern_rash = self.transpile_expression(pattern)?;
                Ok(format!(
                    "{}.split_once({}).map_or(&{}, |(prefix, _)| prefix)",
                    variable, pattern_rash, variable
                ))
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
