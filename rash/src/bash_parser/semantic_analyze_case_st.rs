impl SemanticAnalyzer {

    /// Analyze a case statement with pattern arms.
    fn analyze_case_stmt(
        &mut self,
        word: &BashExpr,
        arms: &[CaseArm],
        scope: &mut ScopeInfo,
    ) -> SemanticResult<()> {
        self.analyze_expression(word, scope)?;

        for arm in arms {
            self.analyze_body(&arm.body, scope)?;
        }
        Ok(())
    }

    /// Analyze a select statement, registering the iteration variable.
    fn analyze_select_stmt(
        &mut self,
        variable: &str,
        body: &[BashStmt],
        scope: &mut ScopeInfo,
    ) -> SemanticResult<()> {
        // F017: Analyze select statement - variable is assigned in each iteration
        scope.variables.insert(
            variable.to_string(),
            VarInfo {
                name: variable.to_string(),
                exported: false,
                assigned: true,
                used: false,
                inferred_type: InferredType::String, // User selection is string
            },
        );
        self.analyze_body(body, scope)
    }

    /// Analyze a sequence of statements (loop body, block, etc.).
    fn analyze_body(&mut self, body: &[BashStmt], scope: &mut ScopeInfo) -> SemanticResult<()> {
        for stmt in body {
            self.analyze_statement(stmt, scope)?;
        }
        Ok(())
    }

    fn analyze_expression(&mut self, expr: &BashExpr, scope: &mut ScopeInfo) -> SemanticResult<()> {
        match expr {
            BashExpr::Variable(name) => {
                // Mark variable as used
                // Note: We don't error on undefined variables in bash
                // since they can come from environment
                Self::mark_var_used(scope, name);
            }

            BashExpr::CommandSubst(cmd) | BashExpr::CommandCondition(cmd) => {
                self.analyze_statement(cmd, scope)?;
            }

            BashExpr::Array(items) | BashExpr::Concat(items) => {
                for item in items {
                    self.analyze_expression(item, scope)?;
                }
            }

            BashExpr::Test(test_expr) => {
                self.analyze_test_expr(test_expr, scope)?;
            }

            BashExpr::Literal(_) | BashExpr::Glob(_) => {}

            BashExpr::Arithmetic(arith) => {
                self.analyze_arithmetic(arith, scope)?;
            }

            BashExpr::DefaultValue { variable, default }
            | BashExpr::ErrorIfUnset {
                variable,
                message: default,
            } => {
                Self::mark_var_used(scope, variable);
                self.analyze_expression(default, scope)?;
            }

            BashExpr::AlternativeValue {
                variable,
                alternative,
            } => {
                Self::mark_var_used(scope, variable);
                self.analyze_expression(alternative, scope)?;
            }

            BashExpr::AssignDefault { variable, default } => {
                self.analyze_assign_default(variable, default, scope)?;
            }

            BashExpr::StringLength { variable } => {
                Self::mark_var_used(scope, variable);
            }

            BashExpr::RemoveSuffix { variable, pattern }
            | BashExpr::RemovePrefix { variable, pattern }
            | BashExpr::RemoveLongestPrefix { variable, pattern }
            | BashExpr::RemoveLongestSuffix { variable, pattern } => {
                Self::mark_var_used(scope, variable);
                self.analyze_expression(pattern, scope)?;
            }
        }

        Ok(())
    }

    /// Mark a variable as used in the given scope.
    fn mark_var_used(scope: &mut ScopeInfo, name: &str) {
        if let Some(var) = scope.variables.get_mut(name) {
            var.used = true;
        }
    }

    /// Analyze ${VAR:=default} — assigns to VAR if unset.
    fn analyze_assign_default(
        &mut self,
        variable: &str,
        default: &BashExpr,
        scope: &mut ScopeInfo,
    ) -> SemanticResult<()> {
        if let Some(var) = scope.variables.get_mut(variable) {
            var.used = true;
            var.assigned = true;
        } else {
            scope.variables.insert(
                variable.to_string(),
                VarInfo {
                    name: variable.to_string(),
                    exported: false,
                    assigned: true,
                    used: true,
                    inferred_type: InferredType::Unknown,
                },
            );
        }
        self.analyze_expression(default, scope)
    }

    fn analyze_test_expr(&mut self, test: &TestExpr, scope: &mut ScopeInfo) -> SemanticResult<()> {
        match test {
            TestExpr::StringEq(a, b)
            | TestExpr::StringNe(a, b)
            | TestExpr::IntEq(a, b)
            | TestExpr::IntNe(a, b)
            | TestExpr::IntLt(a, b)
            | TestExpr::IntLe(a, b)
            | TestExpr::IntGt(a, b)
            | TestExpr::IntGe(a, b) => {
                self.analyze_expression(a, scope)?;
                self.analyze_expression(b, scope)?;
            }

            TestExpr::FileExists(path)
            | TestExpr::FileReadable(path)
            | TestExpr::FileWritable(path)
            | TestExpr::FileExecutable(path)
            | TestExpr::FileDirectory(path) => {
                self.analyze_expression(path, scope)?;
                // File tests imply file reads
                if let BashExpr::Literal(p) = path {
                    self.effects.file_reads.insert(p.clone());
                }
            }

            TestExpr::StringEmpty(s) | TestExpr::StringNonEmpty(s) => {
                self.analyze_expression(s, scope)?;
            }

            TestExpr::And(a, b) | TestExpr::Or(a, b) => {
                self.analyze_test_expr(a, scope)?;
                self.analyze_test_expr(b, scope)?;
            }

            TestExpr::Not(t) => {
                self.analyze_test_expr(t, scope)?;
            }
        }

        Ok(())
    }

    fn analyze_arithmetic(
        &mut self,
        arith: &ArithExpr,
        scope: &mut ScopeInfo,
    ) -> SemanticResult<()> {
        match arith {
            ArithExpr::Variable(name) => {
                if let Some(var) = scope.variables.get_mut(name) {
                    var.used = true;
                }
            }
            ArithExpr::Add(a, b)
            | ArithExpr::Sub(a, b)
            | ArithExpr::Mul(a, b)
            | ArithExpr::Div(a, b)
            | ArithExpr::Mod(a, b) => {
                self.analyze_arithmetic(a, scope)?;
                self.analyze_arithmetic(b, scope)?;
            }
            ArithExpr::Number(_) => {}
        }
        Ok(())
    }

    fn infer_type(&self, expr: &BashExpr) -> InferredType {
        match expr {
            BashExpr::Literal(s) => {
                if s.parse::<i64>().is_ok() {
                    InferredType::Integer
                } else {
                    InferredType::String
                }
            }
            BashExpr::Array(_) => InferredType::Array,
            BashExpr::Arithmetic(_) => InferredType::Integer,
            _ => InferredType::Unknown,
        }
    }

    fn track_command_effects(&mut self, command: &str) {
        // Track known commands with side effects
        match command {
            "curl" | "wget" | "nc" | "telnet" | "ssh" => {
                self.effects.network_access = true;
            }
            "rm" | "mv" | "cp" | "touch" | "mkdir" | "rmdir" => {
                // File modification commands
                self.effects.file_writes.insert(command.to_string());
            }
            "cat" | "less" | "more" | "head" | "tail" | "grep" => {
                // File reading commands
                self.effects.file_reads.insert(command.to_string());
            }
            _ => {}
        }
    }
}










include!("semantic_default.rs");
