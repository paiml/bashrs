impl TypeChecker {
    pub fn new() -> Self {
        Self {
            ctx: TypeContext::new(),
            diagnostics: Vec::new(),
            pending_annotations: Vec::new(),
            annotation_hints: HashMap::new(),
        }
    }

    /// Type-check a complete AST, returning diagnostics
    pub fn check_ast(&mut self, ast: &BashAst) -> Vec<TypeDiagnostic> {
        for stmt in &ast.statements {
            self.check_statement(stmt);
        }
        self.diagnostics.clone()
    }

    /// Check a single statement
    pub fn check_statement(&mut self, stmt: &BashStmt) {
        match stmt {
            BashStmt::Comment { text, .. } => {
                if let Some(annotation) = parse_type_annotation(text) {
                    self.pending_annotations.push(annotation);
                }
            }

            BashStmt::Assignment {
                name, value, span, ..
            } => self.check_assignment(name, value, *span),

            BashStmt::Command {
                name, args, span, ..
            } => self.check_command(name, args, *span),

            BashStmt::Function { name, body, .. } => self.check_function(name, body),

            BashStmt::If {
                condition,
                then_block,
                elif_blocks,
                else_block,
                ..
            } => self.check_if(condition, then_block, elif_blocks, else_block),

            BashStmt::While {
                condition, body, ..
            }
            | BashStmt::Until {
                condition, body, ..
            } => {
                self.infer_expr(condition);
                self.check_body(body);
            }

            BashStmt::For { body, items, .. } | BashStmt::Select { body, items, .. } => {
                self.infer_expr(items);
                self.check_body(body);
            }

            BashStmt::ForCStyle { body, .. }
            | BashStmt::BraceGroup { body, .. }
            | BashStmt::Coproc { body, .. } => self.check_body(body),

            BashStmt::Case { word, arms, .. } => {
                self.infer_expr(word);
                for arm in arms {
                    self.check_body(&arm.body);
                }
            }

            BashStmt::Pipeline { commands, .. } => {
                for cmd in commands {
                    self.check_statement(cmd);
                }
            }

            BashStmt::AndList { left, right, .. } | BashStmt::OrList { left, right, .. } => {
                self.check_statement(left);
                self.check_statement(right);
            }

            BashStmt::Negated { command, .. } => self.check_statement(command),

            BashStmt::Return { code, .. } => {
                if let Some(expr) = code {
                    self.infer_expr(expr);
                }
            }
        }
    }

    /// Check a variable assignment with optional type annotation
    fn check_assignment(&mut self, name: &str, value: &BashExpr, span: Span) {
        let annotated_type = self.consume_annotation(name);
        let inferred = self.infer_expr(value);
        let expected_type = annotated_type.or_else(|| self.ctx.lookup(name).cloned());

        if let Some(ref exp_ty) = expected_type {
            self.ctx.set_type(name, exp_ty.clone());
            self.check_type_compatibility(name, exp_ty, &inferred, span);
        } else if let Some(inf_ty) = inferred {
            self.ctx.set_type(name, inf_ty);
        }
    }

    /// Check type compatibility between expected and inferred types
    fn check_type_compatibility(
        &mut self,
        name: &str,
        expected: &ShellType,
        inferred: &Option<ShellType>,
        span: Span,
    ) {
        if let Some(ref inf_ty) = inferred {
            if !expected.is_compatible(inf_ty) && !is_gradual_compatible(expected, inf_ty) {
                self.diagnostics.push(TypeDiagnostic {
                    span,
                    kind: DiagnosticKind::TypeMismatch {
                        expected: expected.clone(),
                        actual: inf_ty.clone(),
                    },
                    severity: Severity::Warning,
                    message: format!(
                        "variable '{}' annotated as {} but assigned {}",
                        name,
                        expected.display(),
                        inf_ty.display()
                    ),
                });
            }
        }
    }

    /// Check a command statement (declare/typeset/local and arguments)
    fn check_command(&mut self, name: &str, args: &[BashExpr], span: Span) {
        if name == "declare" || name == "typeset" || name == "local" {
            self.check_declare(args, span);
        }
        for arg in args {
            self.infer_expr(arg);
        }
    }

    /// Check a function definition with optional type annotations
    fn check_function(&mut self, name: &str, body: &[BashStmt]) {
        let sig = self.collect_function_sig();
        if sig.is_some() {
            self.ctx.set_function_sig(
                name,
                sig.clone().unwrap_or(FunctionSig {
                    params: Vec::new(),
                    return_type: None,
                }),
            );
        }

        self.ctx.push_scope();
        if let Some(ref sig) = sig {
            for (param_name, param_type) in &sig.params {
                self.ctx.set_type(param_name, param_type.clone());
            }
        }
        self.check_body(body);
        self.ctx.pop_scope();
    }

    /// Check an if/elif/else chain
    fn check_if(
        &mut self,
        condition: &BashExpr,
        then_block: &[BashStmt],
        elif_blocks: &[(BashExpr, Vec<BashStmt>)],
        else_block: &Option<Vec<BashStmt>>,
    ) {
        self.infer_expr(condition);
        self.check_body(then_block);
        for (cond, block) in elif_blocks {
            self.infer_expr(cond);
            self.check_body(block);
        }
        if let Some(else_body) = else_block {
            self.check_body(else_body);
        }
    }

    /// Check all statements in a block body
    fn check_body(&mut self, body: &[BashStmt]) {
        for stmt in body {
            self.check_statement(stmt);
        }
    }

    /// Infer the type of an expression
    pub fn infer_expr(&mut self, expr: &BashExpr) -> Option<ShellType> {
        match expr {
            BashExpr::Literal(s) => {
                // Try to detect integer literals
                if s.chars().all(|c| c.is_ascii_digit() || c == '-') && !s.is_empty() && s != "-" {
                    Some(ShellType::Integer)
                } else if s == "true" || s == "false" {
                    Some(ShellType::Boolean)
                } else {
                    Some(ShellType::String)
                }
            }

            BashExpr::Variable(name) => self.ctx.lookup(name).cloned(),

            BashExpr::CommandSubst(_) => {
                // Command substitution always returns a string
                Some(ShellType::String)
            }

            BashExpr::Arithmetic(arith) => {
                // Check variables used in arithmetic context
                self.check_arithmetic_variables(arith);
                Some(ShellType::Integer)
            }

            BashExpr::Array(elements) => {
                // Infer element types
                for elem in elements {
                    self.infer_expr(elem);
                }
                Some(ShellType::Array(Box::new(ShellType::String)))
            }

            BashExpr::Concat(parts) => {
                // String concatenation always produces a string
                for part in parts {
                    self.infer_expr(part);
                }
                Some(ShellType::String)
            }

            BashExpr::Test(_) => Some(ShellType::Boolean),

            BashExpr::Glob(_) => Some(ShellType::String),

            BashExpr::CommandCondition(_) => Some(ShellType::ExitCode),

            BashExpr::DefaultValue { variable, default } => {
                self.infer_expr(default);
                // Type is the variable's type if known, else default's type
                self.ctx.lookup(variable).cloned()
            }

            BashExpr::AssignDefault { variable, default } => {
                self.infer_expr(default);
                self.ctx.lookup(variable).cloned()
            }

            BashExpr::ErrorIfUnset { variable, message } => {
                self.infer_expr(message);
                self.ctx.lookup(variable).cloned()
            }

            BashExpr::AlternativeValue {
                variable,
                alternative,
            } => {
                self.infer_expr(alternative);
                self.ctx.lookup(variable).cloned()
            }

            BashExpr::StringLength { .. } => Some(ShellType::Integer),

            BashExpr::RemoveSuffix { pattern, .. }
            | BashExpr::RemovePrefix { pattern, .. }
            | BashExpr::RemoveLongestPrefix { pattern, .. }
            | BashExpr::RemoveLongestSuffix { pattern, .. } => {
                self.infer_expr(pattern);
                Some(ShellType::String)
            }
        }
    }

    /// Infer the type of an arithmetic expression (always Integer)
    pub fn infer_arithmetic(&self, _arith: &ArithExpr) -> ShellType {
        ShellType::Integer
    }

    /// Infer the type of a test expression (always Boolean)
    pub fn infer_test(&self, _test: &TestExpr) -> ShellType {
        ShellType::Boolean
    }

    /// Check variables used in arithmetic context for type mismatches
    fn check_arithmetic_variables(&mut self, arith: &ArithExpr) {
        match arith {
            ArithExpr::Variable(name) => {
                if let Some(ty) = self.ctx.lookup(name) {
                    if matches!(ty, ShellType::String) {
                        self.diagnostics.push(TypeDiagnostic {
                            span: Span::dummy(),
                            kind: DiagnosticKind::StringInArithmetic {
                                variable: name.clone(),
                            },
                            severity: Severity::Warning,
                            message: format!(
                                "variable '{}' used in arithmetic but typed as string",
                                name
                            ),
                        });
                    }
                }
            }
            ArithExpr::Number(_) => {}
            ArithExpr::Add(l, r)
            | ArithExpr::Sub(l, r)
            | ArithExpr::Mul(l, r)
            | ArithExpr::Div(l, r)
            | ArithExpr::Mod(l, r) => {
                self.check_arithmetic_variables(l);
                self.check_arithmetic_variables(r);
            }
        }
    }

    /// Get collected diagnostics
    pub fn diagnostics(&self) -> &[TypeDiagnostic] {
        &self.diagnostics
    }

    /// Get the type context (for inspection/testing)
    pub fn context(&self) -> &TypeContext {
        &self.ctx
    }

    /// Consume a pending type annotation matching the given variable name
    fn consume_annotation(&mut self, name: &str) -> Option<ShellType> {
        let pos = self
            .pending_annotations
            .iter()
            .position(|a| a.name == name && !a.is_return && !a.is_param)?;
        let annotation = self.pending_annotations.remove(pos);
        self.annotation_hints
            .insert(name.to_string(), annotation.type_hint.clone());
        Some(annotation.shell_type)
    }

    /// Get the original annotation type name for a variable (e.g., "path", "int")
    pub fn annotation_hint(&self, name: &str) -> Option<&str> {
        self.annotation_hints.get(name).map(|s| s.as_str())
    }

    /// Collect pending param/return annotations into a function signature
    fn collect_function_sig(&mut self) -> Option<FunctionSig> {
        let params: Vec<_> = self
            .pending_annotations
            .iter()
            .filter(|a| a.is_param)
            .map(|a| (a.name.clone(), a.shell_type.clone()))
            .collect();

        let return_type = self
            .pending_annotations
            .iter()
            .find(|a| a.is_return)
            .map(|a| a.shell_type.clone());

        if params.is_empty() && return_type.is_none() {
            return None;
        }

        // Remove consumed annotations
        self.pending_annotations
            .retain(|a| !a.is_param && !a.is_return);

        Some(FunctionSig {
            params,
            return_type,
        })
    }

    /// Handle declare/typeset/local with type flags
    fn check_declare(&mut self, args: &[BashExpr], _span: Span) {
        let mut current_type: Option<ShellType> = None;

        for arg in args {
            if let BashExpr::Literal(s) = arg {
                if let Some(ty) = parse_declare_flag(s) {
                    current_type = Some(ty);
                } else {
                    self.register_declare_var(s, &current_type);
                }
            }
        }
    }

    /// Register a variable from a declare argument (name or name=value)
    fn register_declare_var(&mut self, s: &str, current_type: &Option<ShellType>) {
        let var_name = if let Some(eq_pos) = s.find('=') {
            Some(&s[..eq_pos])
        } else if !s.starts_with('-') {
            Some(s)
        } else {
            None
        };
        if let (Some(name), Some(ty)) = (var_name, current_type) {
            self.ctx.set_type(name, ty.clone());
        }
    }
}


include!("type_check_default.rs");
