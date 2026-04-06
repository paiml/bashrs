impl BashToRashTranspiler {

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

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "codegen_tests_transpile_op.rs"]
// FIXME(PMAT-238): mod tests_extracted;
