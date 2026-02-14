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

            // Issue #68: C-style for loop (transpile to Rust while loop)
            BashStmt::ForCStyle { body, .. } => {
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

            BashStmt::AndList { left, right, .. } => {
                // Transpile AND list: left && right
                let left_str = self.transpile_statement(left)?;
                let right_str = self.transpile_statement(right)?;
                Ok(format!("{} && {}", left_str, right_str))
            }

            BashStmt::OrList { left, right, .. } => {
                // Transpile OR list: left || right
                let left_str = self.transpile_statement(left)?;
                let right_str = self.transpile_statement(right)?;
                Ok(format!("{} || {}", left_str, right_str))
            }

            BashStmt::BraceGroup { body, .. } => {
                // Transpile brace group as a block
                self.current_indent += 1;
                let body_rash = self.transpile_block(body)?;
                self.current_indent -= 1;
                Ok(format!("{{\n{}\n}}", body_rash))
            }

            BashStmt::Coproc { name, body, .. } => {
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
            BashStmt::Select {
                variable,
                items,
                body,
                ..
            } => {
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

            BashStmt::Negated { command, .. } => {
                // Issue #133: Negated command - transpile inner and negate
                let inner = self.transpile_statement(command)?;
                Ok(format!("// negated: ! {}", inner))
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

            BashExpr::CommandCondition(cmd) => {
                // Issue #93: Command condition - transpile as command that returns exit code
                let cmd_rash = self.transpile_statement(cmd)?;
                Ok(format!("{{ {} }}.success()", cmd_rash))
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
    use crate::bash_parser::ast::AstMetadata;
    use crate::bash_parser::parser::BashParser;

    // TranspileOptions tests
    #[test]
    fn test_transpile_options_default() {
        let opts = TranspileOptions::default();
        assert!(opts.add_safety_checks);
        assert!(opts.preserve_comments);
        assert_eq!(opts.indent_size, 4);
    }

    #[test]
    fn test_transpile_options_custom() {
        let opts = TranspileOptions {
            add_safety_checks: false,
            preserve_comments: false,
            indent_size: 2,
        };
        assert!(!opts.add_safety_checks);
        assert!(!opts.preserve_comments);
        assert_eq!(opts.indent_size, 2);
    }

    #[test]
    fn test_transpiler_new() {
        let opts = TranspileOptions::default();
        let transpiler = BashToRashTranspiler::new(opts);
        assert_eq!(transpiler.current_indent, 0);
    }

    // Assignment tests
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
    fn test_transpile_exported_assignment() {
        let bash_code = "export FOO=bar";
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("env::set_var"));
    }

    #[test]
    fn test_transpile_numeric_assignment() {
        let bash_code = "COUNT=42";
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("42"));
    }

    // Function tests
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
    fn test_transpile_function_with_body() {
        let bash_code = r#"
foo() {
    x=1
    echo $x
}
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("fn foo()"));
    }

    // If statement tests
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

    #[test]
    fn test_transpile_if_else() {
        let bash_code = r#"
if [ $x -eq 1 ]; then
    echo "one"
else
    echo "other"
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("if"));
        assert!(rash_code.contains("else"));
    }

    // While loop tests
    #[test]
    fn test_transpile_while_loop() {
        let bash_code = r#"
while [ $x -lt 10 ]; do
    echo $x
done
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("while"));
    }

    // Until loop tests - test using AST directly since parser may not support all operators
    #[test]
    fn test_transpile_until_loop() {
        // Build until loop AST directly
        let until_stmt = BashStmt::Until {
            condition: BashExpr::Test(Box::new(TestExpr::IntGe(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("10".to_string()),
            ))),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("x".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };

        let ast = BashAst {
            statements: vec![until_stmt],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        // Until becomes while with negated condition
        assert!(rash_code.contains("while"));
        assert!(rash_code.contains("!"));
    }

    // For loop tests
    #[test]
    fn test_transpile_for_loop() {
        let bash_code = r#"
for i in 1 2 3; do
    echo $i
done
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("for"));
    }

    // Comment tests
    #[test]
    fn test_transpile_comment_preserved() {
        let bash_code = "# This is a comment";
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let opts = TranspileOptions {
            preserve_comments: true,
            ..TranspileOptions::default()
        };
        let mut transpiler = BashToRashTranspiler::new(opts);
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("//"));
    }

    #[test]
    fn test_transpile_comment_discarded() {
        let bash_code = "# This is a comment\nx=1";
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let opts = TranspileOptions {
            preserve_comments: false,
            ..TranspileOptions::default()
        };
        let mut transpiler = BashToRashTranspiler::new(opts);
        let rash_code = transpiler.transpile(&ast).unwrap();

        // Comment line should be empty, not contain //
        assert!(rash_code.contains("let x"));
    }

    // Return statement tests
    #[test]
    fn test_transpile_return_no_value() {
        let bash_code = r#"
foo() {
    return
}
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("return;"));
    }

    #[test]
    fn test_transpile_return_with_value() {
        let bash_code = r#"
foo() {
    return 0
}
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("return"));
        assert!(rash_code.contains("0"));
    }

    // Expression tests
    #[test]
    fn test_transpile_literal_string() {
        let bash_code = "echo hello";
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("hello"));
    }

    #[test]
    fn test_transpile_variable() {
        let bash_code = "echo $x";
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("x"));
    }

    // Test expression tests
    #[test]
    fn test_transpile_string_eq() {
        let bash_code = r#"
if [ "$x" == "foo" ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("=="));
    }

    #[test]
    fn test_transpile_string_ne() {
        let bash_code = r#"
if [ "$x" != "foo" ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("!="));
    }

    #[test]
    fn test_transpile_int_lt() {
        let bash_code = r#"
if [ $x -lt 10 ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("<"));
    }

    #[test]
    fn test_transpile_int_gt() {
        let bash_code = r#"
if [ $x -gt 10 ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains(">"));
    }

    #[test]
    fn test_transpile_file_exists() {
        let bash_code = r#"
if [ -e /tmp/file ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("exists"));
    }

    #[test]
    fn test_transpile_file_directory() {
        let bash_code = r#"
if [ -d /tmp ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("is_dir"));
    }

    #[test]
    fn test_transpile_string_empty() {
        let bash_code = r#"
if [ -z "$x" ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("is_empty"));
    }

    #[test]
    fn test_transpile_string_non_empty() {
        let bash_code = r#"
if [ -n "$x" ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("!"));
        assert!(rash_code.contains("is_empty"));
    }

    // Indent tests
    #[test]
    fn test_indent_empty_lines() {
        let opts = TranspileOptions::default();
        let transpiler = BashToRashTranspiler::new(opts);

        let result = transpiler.indent("line1\n\nline2");
        assert!(result.contains("line1"));
        assert!(result.contains("line2"));
    }

    #[test]
    fn test_indent_with_level() {
        let opts = TranspileOptions {
            indent_size: 2,
            ..TranspileOptions::default()
        };
        let mut transpiler = BashToRashTranspiler::new(opts);
        transpiler.current_indent = 1;

        let result = transpiler.indent("code");
        assert!(result.starts_with("  ")); // 2 spaces for indent level 1
    }

    // Header test
    #[test]
    fn test_transpile_header() {
        let bash_code = "x=1";
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("// Transpiled from bash by rash"));
    }

    // Arithmetic tests via expressions
    #[test]
    fn test_transpile_arithmetic_add() {
        // We test arithmetic through the AST directly
        let arith = ArithExpr::Add(
            Box::new(ArithExpr::Number(1)),
            Box::new(ArithExpr::Number(2)),
        );
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_arithmetic(&arith).unwrap();
        assert!(result.contains("+"));
        assert!(result.contains("1"));
        assert!(result.contains("2"));
    }

    #[test]
    fn test_transpile_arithmetic_sub() {
        let arith = ArithExpr::Sub(
            Box::new(ArithExpr::Number(5)),
            Box::new(ArithExpr::Number(3)),
        );
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_arithmetic(&arith).unwrap();
        assert!(result.contains("-"));
    }

    #[test]
    fn test_transpile_arithmetic_mul() {
        let arith = ArithExpr::Mul(
            Box::new(ArithExpr::Number(2)),
            Box::new(ArithExpr::Number(3)),
        );
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_arithmetic(&arith).unwrap();
        assert!(result.contains("*"));
    }

    #[test]
    fn test_transpile_arithmetic_div() {
        let arith = ArithExpr::Div(
            Box::new(ArithExpr::Number(10)),
            Box::new(ArithExpr::Number(2)),
        );
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_arithmetic(&arith).unwrap();
        assert!(result.contains("/"));
    }

    #[test]
    fn test_transpile_arithmetic_mod() {
        let arith = ArithExpr::Mod(
            Box::new(ArithExpr::Number(10)),
            Box::new(ArithExpr::Number(3)),
        );
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_arithmetic(&arith).unwrap();
        assert!(result.contains("%"));
    }

    #[test]
    fn test_transpile_arithmetic_variable() {
        let arith = ArithExpr::Variable("x".to_string());
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_arithmetic(&arith).unwrap();
        assert_eq!(result, "x");
    }

    #[test]
    fn test_transpile_arithmetic_number() {
        let arith = ArithExpr::Number(42);
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_arithmetic(&arith).unwrap();
        assert_eq!(result, "42");
    }

    // Test expression direct tests
    #[test]
    fn test_transpile_test_int_le() {
        let test = TestExpr::IntLe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("10".to_string()),
        );
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test(&test).unwrap();
        assert!(result.contains("<="));
    }

    #[test]
    fn test_transpile_test_int_ge() {
        let test = TestExpr::IntGe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("10".to_string()),
        );
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test(&test).unwrap();
        assert!(result.contains(">="));
    }

    #[test]
    fn test_transpile_test_int_ne() {
        let test = TestExpr::IntNe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("10".to_string()),
        );
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test(&test).unwrap();
        assert!(result.contains("!="));
    }

    #[test]
    fn test_transpile_test_and() {
        let test = TestExpr::And(
            Box::new(TestExpr::StringEq(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("a".to_string()),
            )),
            Box::new(TestExpr::StringEq(
                BashExpr::Variable("y".to_string()),
                BashExpr::Literal("b".to_string()),
            )),
        );
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test(&test).unwrap();
        assert!(result.contains("&&"));
    }

    #[test]
    fn test_transpile_test_or() {
        let test = TestExpr::Or(
            Box::new(TestExpr::StringEq(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("a".to_string()),
            )),
            Box::new(TestExpr::StringEq(
                BashExpr::Variable("y".to_string()),
                BashExpr::Literal("b".to_string()),
            )),
        );
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test(&test).unwrap();
        assert!(result.contains("||"));
    }

    #[test]
    fn test_transpile_test_not() {
        let test = TestExpr::Not(Box::new(TestExpr::StringEq(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("a".to_string()),
        )));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test(&test).unwrap();
        assert!(result.contains("!("));
    }

    #[test]
    fn test_transpile_test_file_readable() {
        let test = TestExpr::FileReadable(BashExpr::Literal("/tmp/file".to_string()));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test(&test).unwrap();
        assert!(result.contains("metadata"));
        assert!(result.contains("readonly"));
    }

    #[test]
    fn test_transpile_test_file_writable() {
        let test = TestExpr::FileWritable(BashExpr::Literal("/tmp/file".to_string()));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test(&test).unwrap();
        assert!(result.contains("metadata"));
        assert!(result.contains("readonly"));
    }

    #[test]
    fn test_transpile_test_file_executable() {
        let test = TestExpr::FileExecutable(BashExpr::Literal("/tmp/file".to_string()));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test(&test).unwrap();
        assert!(result.contains("is_executable"));
    }

    // Expression direct tests
    #[test]
    fn test_transpile_expr_glob() {
        let expr = BashExpr::Glob("*.txt".to_string());
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("glob"));
        assert!(result.contains("*.txt"));
    }

    #[test]
    fn test_transpile_expr_default_value() {
        let expr = BashExpr::DefaultValue {
            variable: "x".to_string(),
            default: Box::new(BashExpr::Literal("default".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("unwrap_or"));
    }

    #[test]
    fn test_transpile_expr_assign_default() {
        let expr = BashExpr::AssignDefault {
            variable: "x".to_string(),
            default: Box::new(BashExpr::Literal("default".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("get_or_insert"));
    }

    #[test]
    fn test_transpile_expr_error_if_unset() {
        let expr = BashExpr::ErrorIfUnset {
            variable: "x".to_string(),
            message: Box::new(BashExpr::Literal("error".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("expect"));
    }

    #[test]
    fn test_transpile_expr_alternative_value() {
        let expr = BashExpr::AlternativeValue {
            variable: "x".to_string(),
            alternative: Box::new(BashExpr::Literal("alt".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("as_ref"));
        assert!(result.contains("map"));
    }

    #[test]
    fn test_transpile_expr_string_length() {
        let expr = BashExpr::StringLength {
            variable: "x".to_string(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains(".len()"));
    }

    #[test]
    fn test_transpile_expr_remove_suffix() {
        let expr = BashExpr::RemoveSuffix {
            variable: "x".to_string(),
            pattern: Box::new(BashExpr::Literal(".txt".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("strip_suffix"));
    }

    #[test]
    fn test_transpile_expr_remove_prefix() {
        let expr = BashExpr::RemovePrefix {
            variable: "x".to_string(),
            pattern: Box::new(BashExpr::Literal("/tmp/".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("strip_prefix"));
    }

    #[test]
    fn test_transpile_expr_remove_longest_prefix() {
        let expr = BashExpr::RemoveLongestPrefix {
            variable: "x".to_string(),
            pattern: Box::new(BashExpr::Literal("/*/".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("rsplit_once"));
    }

    #[test]
    fn test_transpile_expr_remove_longest_suffix() {
        let expr = BashExpr::RemoveLongestSuffix {
            variable: "x".to_string(),
            pattern: Box::new(BashExpr::Literal(".*".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("split_once"));
    }

    #[test]
    fn test_transpile_expr_array() {
        let expr = BashExpr::Array(vec![
            BashExpr::Literal("a".to_string()),
            BashExpr::Literal("b".to_string()),
        ]);
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("vec!"));
    }

    #[test]
    fn test_transpile_expr_concat() {
        let expr = BashExpr::Concat(vec![
            BashExpr::Literal("hello".to_string()),
            BashExpr::Variable("name".to_string()),
        ]);
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("format!"));
    }

    #[test]
    fn test_transpile_expr_command_subst() {
        let stmt = BashStmt::Command {
            name: "ls".to_string(),
            args: vec![],
            redirects: vec![],
            span: Span::dummy(),
        };
        let expr = BashExpr::CommandSubst(Box::new(stmt));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("{"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_transpile_expr_command_condition() {
        let stmt = BashStmt::Command {
            name: "test".to_string(),
            args: vec![],
            redirects: vec![],
            span: Span::dummy(),
        };
        let expr = BashExpr::CommandCondition(Box::new(stmt));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("success"));
    }

    #[test]
    fn test_CODEGEN_COV_001_if_with_elif() {
        let stmt = BashStmt::If {
            condition: BashExpr::Test(Box::new(TestExpr::StringEq(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("1".to_string()),
            ))),
            then_block: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("one".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            elif_blocks: vec![(
                BashExpr::Test(Box::new(TestExpr::StringEq(
                    BashExpr::Variable("x".to_string()),
                    BashExpr::Literal("2".to_string()),
                ))),
                vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("two".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
            )],
            else_block: None,
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("else if"));
    }

    #[test]
    fn test_CODEGEN_COV_002_for_c_style() {
        let stmt = BashStmt::ForCStyle {
            init: "i=0".to_string(),
            condition: "i<10".to_string(),
            increment: "i++".to_string(),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("i".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("C-style for loop"));
    }

    #[test]
    fn test_CODEGEN_COV_003_case_statement() {
        let stmt = BashStmt::Case {
            word: BashExpr::Variable("opt".to_string()),
            arms: vec![
                CaseArm {
                    patterns: vec!["start".to_string()],
                    body: vec![BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("starting".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    }],
                },
                CaseArm {
                    patterns: vec!["*".to_string()],
                    body: vec![BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("default".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    }],
                },
            ],
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("match"));
        assert!(result.contains("start"));
    }

    #[test]
    fn test_CODEGEN_COV_004_pipeline() {
        let stmt = BashStmt::Pipeline {
            commands: vec![
                BashStmt::Command {
                    name: "cat".to_string(),
                    args: vec![BashExpr::Literal("file.txt".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                },
                BashStmt::Command {
                    name: "grep".to_string(),
                    args: vec![BashExpr::Literal("pattern".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                },
            ],
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("|"));
    }

    #[test]
    fn test_CODEGEN_COV_005_and_list() {
        let stmt = BashStmt::AndList {
            left: Box::new(BashStmt::Command {
                name: "test".to_string(),
                args: vec![BashExpr::Literal("-f".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }),
            right: Box::new(BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("exists".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }),
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("&&"));
    }

    #[test]
    fn test_CODEGEN_COV_006_or_list() {
        let stmt = BashStmt::OrList {
            left: Box::new(BashStmt::Command {
                name: "test".to_string(),
                args: vec![BashExpr::Literal("-f".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }),
            right: Box::new(BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("missing".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }),
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("||"));
    }

    #[test]
    fn test_CODEGEN_COV_007_brace_group() {
        let stmt = BashStmt::BraceGroup {
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("inside".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            subshell: false,
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("{"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_CODEGEN_COV_008_coproc_named() {
        let stmt = BashStmt::Coproc {
            name: Some("myproc".to_string()),
            body: vec![BashStmt::Command {
                name: "cat".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("coproc myproc"));
    }

    #[test]
    fn test_CODEGEN_COV_009_coproc_unnamed() {
        let stmt = BashStmt::Coproc {
            name: None,
            body: vec![BashStmt::Command {
                name: "cat".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("coproc -"));
    }

    #[test]
    fn test_CODEGEN_COV_010_select() {
        let stmt = BashStmt::Select {
            variable: "opt".to_string(),
            items: BashExpr::Array(vec![
                BashExpr::Literal("a".to_string()),
                BashExpr::Literal("b".to_string()),
            ]),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("opt".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("select opt"));
    }

    #[test]
    fn test_CODEGEN_COV_011_expr_arithmetic() {
        let expr = BashExpr::Arithmetic(Box::new(ArithExpr::Add(
            Box::new(ArithExpr::Number(5)),
            Box::new(ArithExpr::Number(3)),
        )));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("5") && result.contains("3"));
    }

    #[test]
    fn test_CODEGEN_COV_012_expr_test() {
        let expr = BashExpr::Test(Box::new(TestExpr::StringEq(
            BashExpr::Literal("a".to_string()),
            BashExpr::Literal("b".to_string()),
        )));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("=="));
    }

    #[test]
    fn test_CODEGEN_COV_013_test_expression_fallback() {
        // Non-Test expr in test position falls through to transpile_expression
        let expr = BashExpr::Literal("true".to_string());
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test_expression(&expr).unwrap();
        assert!(result.contains("true"));
    }
}
