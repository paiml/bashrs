//! Makefile code emitter
//!
//! Converts a RestrictedAst (Rust DSL) into a valid Makefile using conventions:
//! - `let` bindings with string values -> Makefile variables (SIMPLE `:=`)
//! - `target("name", &["deps"], &["recipes"])` function calls -> Make targets
//! - `phony_target("name", ...)` -> `.PHONY` targets
//!
//! The generated Makefile is passed through the existing Makefile generator
//! infrastructure for consistent formatting.

use crate::ast::restricted::{BinaryOp, Function, Literal};
use crate::ast::{Expr, RestrictedAst, Stmt};
use crate::make_parser::ast::{MakeAst, MakeItem, MakeMetadata, Span, VarFlavor};
use crate::make_parser::generators::generate_purified_makefile;
use crate::models::{Error, Result};

/// Convert a RestrictedAst (Rust DSL) to a Makefile string.
///
/// The DSL conventions:
/// ```ignore
/// fn main() {
///     // Variables become Makefile variables
///     let cc = "gcc";
///     let cflags = "-O2 -Wall";
///
///     // target(name, deps, recipes) -> Make target
///     target("build", &["src/main.c"], &["$(CC) -o build src/main.c"]);
///
///     // phony_target(name, deps, recipes) -> .PHONY target
///     phony_target("clean", &[], &["rm -f build"]);
/// }
/// ```
pub fn emit_makefile(ast: &RestrictedAst) -> Result<String> {
    let mut converter = MakefileConverter::new();

    // Detect if the entry uses exec()/println! for raw output
    let entry_fn = ast
        .functions
        .iter()
        .find(|f| f.name == ast.entry_point)
        .ok_or_else(|| Error::IrGeneration("Entry point not found".to_string()))?;

    let has_raw_output = entry_fn.body.iter().any(|stmt| {
        matches!(stmt, Stmt::Expr(Expr::FunctionCall { name, .. })
            if name == "exec" || name == "rash_println" || name == "println"
                || name == "rash_print" || name == "print")
    });

    if has_raw_output {
        // Raw output mode: collect resolved lines from exec/println
        let raw = converter.emit_raw_lines(entry_fn)?;
        if !raw.trim().is_empty() {
            return Ok(raw);
        }
    }

    // DSL mode: use target()/phony_target()/let bindings → MakeAst
    if !has_raw_output {
        let make_ast = converter.convert(ast)?;
        let dsl_output = generate_purified_makefile(&make_ast);
        if !dsl_output.trim().is_empty() {
            return Ok(dsl_output);
        }
    }

    // Fallback: transpile to bash, wrap in Makefile all: target
    emit_bash_as_makefile(ast)
}

/// Fallback: transpile Rust to bash shell code and wrap in a Makefile `all:` target.
fn emit_bash_as_makefile(ast: &RestrictedAst) -> Result<String> {
    let ir = crate::ir::from_ast(ast)?;
    let config = crate::models::Config::default();
    let shell = crate::emitter::emit(&ir, &config)?;
    wrap_shell_in_makefile(&shell)
}

/// Wrap shell script lines into a Makefile `all:` target recipe.
/// Each line is prefixed with a tab and shell continuation where needed.
fn wrap_shell_in_makefile(shell: &str) -> Result<String> {
    let mut out = String::from(".PHONY: all\nall:\n");
    for line in shell.lines() {
        let trimmed = line.trim();
        // Skip shebang and shell config lines
        if trimmed.starts_with("#!") || trimmed.starts_with("set -") {
            continue;
        }
        if trimmed.is_empty() {
            continue;
        }
        out.push('\t');
        out.push_str(trimmed);
        out.push('\n');
    }
    Ok(out)
}

struct MakefileConverter {
    line: usize,
    /// Track variable bindings for println! format string resolution
    vars: std::collections::HashMap<String, String>,
}

impl MakefileConverter {
    fn new() -> Self {
        Self {
            line: 1,
            vars: std::collections::HashMap::new(),
        }
    }

    fn next_span(&mut self) -> Span {
        let span = Span {
            start: self.line,
            end: self.line,
            line: self.line,
        };
        self.line += 1;
        span
    }

    /// Emit raw makefile lines from exec()/println!() calls.
    ///
    /// First pass: collect variable bindings for format resolution.
    /// Second pass: resolve and emit each exec/println line.
    fn emit_raw_lines(&mut self, entry_fn: &Function) -> Result<String> {
        // First pass: collect variable bindings
        self.collect_variable_bindings(entry_fn);

        // Second pass: collect output lines
        let mut output = String::new();
        for stmt in &entry_fn.body {
            self.emit_raw_output_stmt(stmt, &mut output);
        }

        Ok(output)
    }

    /// Collect variable bindings from let statements
    fn collect_variable_bindings(&mut self, entry_fn: &Function) {
        for stmt in &entry_fn.body {
            if let Stmt::Let {
                name,
                value: Expr::Literal(Literal::Str(s)),
                ..
            } = stmt
            {
                self.vars.insert(name.to_string(), s.clone());
            }
        }
    }

    /// Emit a single raw output statement (exec/println/print) into the output buffer
    fn emit_raw_output_stmt(&self, stmt: &Stmt, output: &mut String) {
        if let Stmt::Expr(Expr::FunctionCall { name, args }) = stmt {
            let is_output = matches!(
                name.as_str(),
                "exec" | "rash_println" | "println" | "rash_print" | "print"
            );
            if is_output {
                if let Some(first_arg) = args.first() {
                    let resolved = self.resolve_concat_expr(first_arg);
                    if !resolved.is_empty() {
                        output.push_str(&resolved);
                        if name != "rash_print" && name != "print" {
                            output.push('\n');
                        }
                    }
                }
            }
        }
    }

    fn convert(&self, ast: &RestrictedAst) -> Result<MakeAst> {
        let mut converter = MakefileConverter::new();
        let mut items = Vec::new();

        // Find the entry point function
        let entry_fn = ast
            .functions
            .iter()
            .find(|f| f.name == ast.entry_point)
            .ok_or_else(|| Error::IrGeneration("Entry point not found".to_string()))?;

        // Convert each statement in the entry function
        for stmt in &entry_fn.body {
            if let Some(item) = converter.convert_stmt(stmt)? {
                items.push(item);
            }
        }

        // Convert non-main functions as potential helper targets
        for function in &ast.functions {
            if function.name != ast.entry_point {
                if let Some(target) = Self::convert_helper_function(function, &mut converter) {
                    items.push(target);
                }
            }
        }

        Ok(MakeAst {
            items,
            metadata: MakeMetadata::new(),
        })
    }

    /// Convert a non-main function into a Makefile target, if it has echo/println statements
    fn convert_helper_function(
        function: &Function,
        converter: &mut MakefileConverter,
    ) -> Option<MakeItem> {
        let mut recipes = Vec::new();
        for stmt in &function.body {
            if let Stmt::Expr(Expr::FunctionCall { name, args }) = stmt {
                if name == "echo" || name == "println" {
                    if let Some(Expr::Literal(Literal::Str(s))) = args.first() {
                        recipes.push(format!("@echo '{}'", s));
                    }
                }
            }
        }

        if recipes.is_empty() {
            return None;
        }

        let params: Vec<String> = function.params.iter().map(|p| p.name.clone()).collect();
        Some(MakeItem::Target {
            name: function.name.clone(),
            prerequisites: params,
            recipe: recipes,
            phony: true,
            recipe_metadata: None,
            span: converter.next_span(),
        })
    }

    fn convert_stmt(&mut self, stmt: &Stmt) -> Result<Option<MakeItem>> {
        match stmt {
            Stmt::Let { name, value, .. } => self.convert_let(name, value),
            Stmt::Expr(expr) => self.convert_expr(expr),
            _ => Ok(None),
        }
    }

    fn convert_let(&mut self, name: &str, value: &Expr) -> Result<Option<MakeItem>> {
        // let cc = "gcc"; -> CC := gcc
        let var_value = self.expr_to_string(value)?;
        let var_name = name.to_uppercase();

        // Track binding for println! format resolution
        if let Expr::Literal(Literal::Str(s)) = value {
            self.vars.insert(name.to_string(), s.clone());
        }

        Ok(Some(MakeItem::Variable {
            name: var_name,
            value: var_value,
            flavor: VarFlavor::Simple, // := (immediate assignment)
            span: self.next_span(),
        }))
    }

    fn convert_expr(&mut self, expr: &Expr) -> Result<Option<MakeItem>> {
        match expr {
            Expr::FunctionCall { name, args } => {
                if name == "target" || name == "phony_target" {
                    self.convert_target_call(name, args)
                } else if (name == "println"
                    || name == "rash_println"
                    || name == "rash_print"
                    || name == "rash_eprintln")
                    && !args.is_empty()
                {
                    self.convert_println(args)
                } else if name == "exec" && !args.is_empty() {
                    self.convert_exec(args)
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    /// Convert a println!() call into a MakeItem by resolving its argument.
    ///
    /// The parser converts `println!("fmt", args...)` into:
    ///   `FunctionCall { name: "rash_println", args: [resolved_expr] }`
    /// where resolved_expr is either a literal string or a `__format_concat` call.
    ///
    /// The resolved line is analyzed:
    /// - Contains `:` with no `=` → Target rule (name: deps)
    /// - Contains `:=` or `=` → Variable assignment
    /// - Otherwise → Comment (raw line)
    fn convert_println(&mut self, args: &[Expr]) -> Result<Option<MakeItem>> {
        let resolved = match args.first() {
            Some(expr) => self.resolve_concat_expr(expr),
            None => return Ok(None),
        };
        if resolved.is_empty() {
            return Ok(None);
        }
        self.analyze_makefile_line(&resolved)
    }

    /// Analyze a raw makefile line and convert it to the appropriate MakeItem.
    ///
    /// Detection rules (in priority order):
    /// 1. `.PHONY:` → Target with phony=true
    /// 2. Tab-prefixed → Comment (recipe line, will be associated with preceding target)
    /// 3. `NAME :=` → Variable (Simple flavor)
    /// 4. `name:` with no `=` → Target rule
    /// 5. `NAME =` → Variable (Recursive flavor)
    /// 6. Otherwise → Comment
    fn analyze_makefile_line(&mut self, line: &str) -> Result<Option<MakeItem>> {
        // .PHONY line (check before general colon detection)
        if line.starts_with(".PHONY") {
            if let Some(colon_pos) = line.find(':') {
                let _targets: Vec<String> = line[colon_pos + 1..]
                    .split_whitespace()
                    .map(String::from)
                    .collect();
                // Emit a .PHONY comment — the targets will be marked phony individually
                return Ok(Some(MakeItem::Comment {
                    text: line.to_string(),
                    span: self.next_span(),
                }));
            }
        }

        // Tab-prefixed lines are recipe lines (emit as comment to preserve content)
        if line.starts_with('\t') {
            return Ok(Some(MakeItem::Comment {
                text: line.to_string(),
                span: self.next_span(),
            }));
        }

        // Detect variable assignment: "NAME := value" (Simple)
        if let Some(eq_idx) = line.find(":=") {
            let var_name = line[..eq_idx].trim().to_string();
            let var_val = line[eq_idx + 2..].trim().to_string();
            if !var_name.is_empty() && !var_name.contains(' ') {
                return Ok(Some(MakeItem::Variable {
                    name: var_name,
                    value: var_val,
                    flavor: VarFlavor::Simple,
                    span: self.next_span(),
                }));
            }
        }

        // Detect target rule: "name: deps" (but not after := which was handled above)
        if let Some(colon_idx) = line.find(':') {
            // Make sure this isn't a := (already handled)
            let after_colon = line.get(colon_idx + 1..colon_idx + 2).unwrap_or("");
            if after_colon != "=" {
                let target_name = line[..colon_idx].trim().to_string();
                let deps_str = line[colon_idx + 1..].trim();
                if !target_name.is_empty()
                    && !target_name.contains(' ')
                    && !target_name.starts_with('#')
                {
                    let prerequisites: Vec<String> = if deps_str.is_empty() {
                        vec![]
                    } else {
                        deps_str.split_whitespace().map(String::from).collect()
                    };
                    return Ok(Some(MakeItem::Target {
                        name: target_name,
                        prerequisites,
                        recipe: vec![],
                        phony: false,
                        recipe_metadata: None,
                        span: self.next_span(),
                    }));
                }
            }
        }

        // Detect recursive variable assignment: "NAME = value"
        if let Some(eq_idx) = line.find('=') {
            let before = line[..eq_idx].trim();
            if !before.is_empty()
                && !before.contains(' ')
                && !before.starts_with('\t')
                && !before.starts_with('.')
                && !before.starts_with('#')
            {
                let var_val = line[eq_idx + 1..].trim().to_string();
                return Ok(Some(MakeItem::Variable {
                    name: before.to_string(),
                    value: var_val,
                    flavor: VarFlavor::Recursive,
                    span: self.next_span(),
                }));
            }
        }

        // Default: emit as comment
        Ok(Some(MakeItem::Comment {
            text: line.to_string(),
            span: self.next_span(),
        }))
    }

    /// Resolve an expression (possibly a `__format_concat` call) to its string value.
    ///
    /// The parser converts `println!("{}: {}", a, b)` to:
    ///   `FunctionCall { name: "__format_concat", args: [Variable("a"), Literal(": "), Variable("b")] }`
    fn resolve_concat_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(Literal::Str(s)) => s.clone(),
            Expr::Literal(Literal::I32(n)) => n.to_string(),
            Expr::Literal(Literal::U16(n)) => n.to_string(),
            Expr::Literal(Literal::U32(n)) => n.to_string(),
            Expr::Literal(Literal::Bool(b)) => b.to_string(),
            Expr::Variable(name) => {
                // Try to resolve from tracked bindings, fall back to $(NAME)
                if let Some(val) = self.vars.get(name.as_str()) {
                    val.clone()
                } else {
                    format!("$({})", name.to_uppercase())
                }
            }
            Expr::FunctionCall { name, args } if name == "__format_concat" => {
                // Concatenate all parts
                args.iter().map(|a| self.resolve_concat_expr(a)).collect()
            }
            _ => String::new(),
        }
    }

    /// Convert an exec() call to a MakeItem.
    ///
    /// `exec("CC := gcc")` → Variable { name: "CC", value: "gcc", flavor: Simple }
    /// `exec("build: main.o")` → Target { name: "build", deps: ["main.o"] }
    /// `exec("\tcargo build")` → appended as recipe to previous target
    ///
    /// This reuses the same analysis logic as convert_println.
    fn convert_exec(&mut self, args: &[Expr]) -> Result<Option<MakeItem>> {
        let resolved = match args.first() {
            Some(expr) => self.resolve_concat_expr(expr),
            None => return Ok(None),
        };
        if resolved.is_empty() {
            return Ok(None);
        }
        self.analyze_makefile_line(&resolved)
    }

    fn convert_target_call(&mut self, func_name: &str, args: &[Expr]) -> Result<Option<MakeItem>> {
        // target("name", &["dep1", "dep2"], &["recipe1", "recipe2"])
        // target("name", &["dep1", "dep2"]) -- deps only, no recipes
        // phony_target("name", &["dep1"], &["recipe1"])
        if args.len() < 2 {
            return Err(Error::Validation(format!(
                "{}() requires at least 2 arguments: name, dependencies (recipes optional)",
                func_name
            )));
        }

        // Extract target name (safe indexing: we verified args.len() >= 2 above)
        let Some(first_arg) = args.first() else {
            return Err(Error::Validation(format!(
                "{}() requires at least 2 arguments",
                func_name
            )));
        };
        let target_name = match first_arg {
            Expr::Literal(Literal::Str(s)) => s.clone(),
            _ => {
                return Err(Error::Validation(format!(
                    "{}() first argument must be a string literal (target name)",
                    func_name
                )))
            }
        };

        // Extract dependencies (from array literal) - safe: verified args.len() >= 2
        let Some(deps_arg) = args.get(1) else {
            return Err(Error::Validation(format!(
                "{}() requires at least 2 arguments",
                func_name
            )));
        };
        let deps = self.extract_string_array(deps_arg)?;

        // Extract recipes (optional third argument)
        let recipes = if let Some(recipes_arg) = args.get(2) {
            self.extract_string_array(recipes_arg)?
        } else {
            Vec::new()
        };

        let is_phony = func_name == "phony_target";

        Ok(Some(MakeItem::Target {
            name: target_name,
            prerequisites: deps,
            recipe: recipes,
            phony: is_phony,
            recipe_metadata: None,
            span: self.next_span(),
        }))
    }

    fn extract_string_array(&self, expr: &Expr) -> Result<Vec<String>> {
        match expr {
            // Handle Array expression: &["dep1", "dep2"]
            Expr::Array(items) => {
                let mut result = Vec::new();
                for item in items {
                    result.push(self.expr_to_string(item)?);
                }
                Ok(result)
            }
            // Handle array/slice literals represented as function calls in our AST
            Expr::FunctionCall { name, args } if name == "__array__" => {
                let mut result = Vec::new();
                for arg in args {
                    result.push(self.expr_to_string(arg)?);
                }
                Ok(result)
            }
            // Handle empty array
            Expr::Literal(Literal::Str(s)) if s.is_empty() => Ok(Vec::new()),
            // Handle single string as array of one
            Expr::Literal(Literal::Str(s)) => Ok(vec![s.clone()]),
            _ => {
                // Try to extract as a string
                Ok(vec![self.expr_to_string(expr)?])
            }
        }
    }

    fn expr_to_string(&self, expr: &Expr) -> Result<String> {
        match expr {
            Expr::Literal(Literal::Str(s)) => Ok(s.clone()),
            Expr::Literal(Literal::U16(n)) => Ok(n.to_string()),
            Expr::Literal(Literal::U32(n)) => Ok(n.to_string()),
            Expr::Literal(Literal::I32(n)) => Ok(n.to_string()),
            Expr::Literal(Literal::Bool(b)) => Ok(b.to_string()),
            Expr::Variable(name) => Ok(format!("$({})", name.to_uppercase())),
            Expr::Array(items) => {
                // Arrays become space-separated lists in Makefiles
                let parts: Vec<String> = items
                    .iter()
                    .map(|item| self.expr_to_string(item))
                    .collect::<Result<_>>()?;
                Ok(parts.join(" "))
            }
            Expr::Binary { left, op, right } => {
                let l = self.expr_to_string(left)?;
                let r = self.expr_to_string(right)?;
                let op_str = match op {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    _ => return Ok(format!("{} {}", l, r)),
                };
                Ok(format!("$(shell echo $$(({} {} {})))", l, op_str, r))
            }
            Expr::Index { object, index } => {
                let arr_str = self.expr_to_string(object)?;
                let idx_str = self.expr_to_string(index)?;
                Ok(format!("$(word {},{})", idx_str, arr_str))
            }
            _ => Err(Error::Validation(
                "Cannot convert expression to Makefile value".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::ast::restricted::{Function, Type};

    fn make_simple_ast(stmts: Vec<Stmt>) -> RestrictedAst {
        RestrictedAst {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: stmts,
            }],
            entry_point: "main".to_string(),
        }
    }

    #[test]
    fn test_MAKE_BUILD_001_basic_variable() {
        let ast = make_simple_ast(vec![Stmt::Let {
            name: "cc".to_string(),
            value: Expr::Literal(Literal::Str("gcc".to_string())),
            declaration: true,
        }]);

        let result = emit_makefile(&ast).unwrap();
        assert!(
            result.contains("CC := gcc"),
            "Expected 'CC := gcc' in: {}",
            result
        );
    }

    #[test]
    fn test_MAKE_BUILD_002_multiple_variables() {
        let ast = make_simple_ast(vec![
            Stmt::Let {
                name: "cc".to_string(),
                value: Expr::Literal(Literal::Str("gcc".to_string())),
                declaration: true,
            },
            Stmt::Let {
                name: "cflags".to_string(),
                value: Expr::Literal(Literal::Str("-O2 -Wall".to_string())),
                declaration: true,
            },
        ]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("CC := gcc"));
        assert!(result.contains("CFLAGS := -O2 -Wall"));
    }

    #[test]
    fn test_MAKE_BUILD_003_target_with_deps_and_recipes() {
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "target".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("build".to_string())),
                Expr::Array(vec![Expr::Literal(Literal::Str("src/main.c".to_string()))]),
                Expr::Array(vec![Expr::Literal(Literal::Str(
                    "$(CC) -o build src/main.c".to_string(),
                ))]),
            ],
        })]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("build"), "Target name 'build' in: {result}");
        assert!(result.contains("src/main.c"), "Dep in: {result}");
    }

    #[test]
    fn test_MAKE_BUILD_004_phony_target() {
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "phony_target".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("clean".to_string())),
                Expr::Array(vec![]),
                Expr::Array(vec![Expr::Literal(Literal::Str("rm -f build".to_string()))]),
            ],
        })]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("clean"), "Phony target in: {result}");
        assert!(result.contains(".PHONY"), ".PHONY in: {result}");
    }

    #[test]
    fn test_MAKE_BUILD_005_target_deps_only() {
        // target with 2 args (no recipes)
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "target".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("all".to_string())),
                Expr::Array(vec![
                    Expr::Literal(Literal::Str("build".to_string())),
                    Expr::Literal(Literal::Str("test".to_string())),
                ]),
            ],
        })]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("all"), "Target 'all' in: {result}");
    }

    #[test]
    fn test_MAKE_BUILD_006_target_too_few_args() {
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "target".to_string(),
            args: vec![Expr::Literal(Literal::Str("build".to_string()))],
        })]);

        let err = emit_makefile(&ast).unwrap_err();
        assert!(
            format!("{err}").contains("at least 2 arguments"),
            "Error: {err}"
        );
    }

    #[test]
    fn test_MAKE_BUILD_007_target_non_string_name() {
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "target".to_string(),
            args: vec![Expr::Literal(Literal::I32(42)), Expr::Array(vec![])],
        })]);

        let err = emit_makefile(&ast).unwrap_err();
        assert!(format!("{err}").contains("string literal"), "Error: {err}");
    }

    #[test]
    fn test_MAKE_BUILD_008_variable_expr_to_string() {
        let ast = make_simple_ast(vec![Stmt::Let {
            name: "output".to_string(),
            value: Expr::Variable("cc".to_string()),
            declaration: true,
        }]);

        let result = emit_makefile(&ast).unwrap();
        assert!(
            result.contains("$(CC)"),
            "Variable ref should be $(CC) in: {result}"
        );
    }

    #[test]
    fn test_MAKE_BUILD_009_numeric_literals() {
        let ast = make_simple_ast(vec![
            Stmt::Let {
                name: "port".to_string(),
                value: Expr::Literal(Literal::U16(8080)),
                declaration: true,
            },
            Stmt::Let {
                name: "count".to_string(),
                value: Expr::Literal(Literal::I32(42)),
                declaration: true,
            },
            Stmt::Let {
                name: "size".to_string(),
                value: Expr::Literal(Literal::U32(1024)),
                declaration: true,
            },
            Stmt::Let {
                name: "verbose".to_string(),
                value: Expr::Literal(Literal::Bool(true)),
                declaration: true,
            },
        ]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("8080"), "U16 in: {result}");
        assert!(result.contains("42"), "I32 in: {result}");
        assert!(result.contains("1024"), "U32 in: {result}");
        assert!(result.contains("true"), "Bool in: {result}");
    }

    #[test]
    fn test_MAKE_BUILD_010_extract_string_array_single() {
        // Single string treated as array of one
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "target".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("build".to_string())),
                Expr::Literal(Literal::Str("main.c".to_string())),
                Expr::Array(vec![Expr::Literal(Literal::Str(
                    "gcc -o build main.c".to_string(),
                ))]),
            ],
        })]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("build"), "Target in: {result}");
    }

    #[test]
    fn test_MAKE_BUILD_011_extract_string_array_empty() {
        // Empty string treated as empty deps
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "target".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("clean".to_string())),
                Expr::Literal(Literal::Str("".to_string())),
                Expr::Array(vec![Expr::Literal(Literal::Str("rm -f build".to_string()))]),
            ],
        })]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("clean"), "Target in: {result}");
    }

    #[test]
    fn test_MAKE_BUILD_012_non_main_function_as_target() {
        // Non-main functions with echo/println become helper targets
        let ast = RestrictedAst {
            functions: vec![
                Function {
                    name: "main".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Let {
                        name: "cc".to_string(),
                        value: Expr::Literal(Literal::Str("gcc".to_string())),
                        declaration: true,
                    }],
                },
                Function {
                    name: "help".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("Usage: make build".to_string()))],
                    })],
                },
            ],
            entry_point: "main".to_string(),
        };

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("CC := gcc"), "Main var in: {result}");
        assert!(result.contains("help"), "Helper target in: {result}");
    }

    #[test]
    fn test_MAKE_BUILD_013_ignored_stmt_types() {
        // Non-Let, non-Expr statements should be ignored
        let ast = make_simple_ast(vec![
            Stmt::Let {
                name: "cc".to_string(),
                value: Expr::Literal(Literal::Str("gcc".to_string())),
                declaration: true,
            },
            Stmt::Expr(Expr::FunctionCall {
                name: "unknown_func".to_string(),
                args: vec![],
            }),
        ]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("CC := gcc"));
    }

    // --- Coverage tests for uncovered lines ---

    #[test]
    fn test_MAKE_COV_001_convert_stmt_return_catchall() {
        // Line 118: _ => Ok(None) in convert_stmt — Stmt::Return is not Let or Expr
        let ast = make_simple_ast(vec![
            Stmt::Let {
                name: "cc".to_string(),
                value: Expr::Literal(Literal::Str("gcc".to_string())),
                declaration: true,
            },
            Stmt::Return(None),
        ]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("CC := gcc"));
    }

    #[test]
    fn test_MAKE_COV_002_convert_expr_non_functioncall() {
        // Line 144: _ => Ok(None) in convert_expr — Expr::Variable is not FunctionCall
        let ast = make_simple_ast(vec![
            Stmt::Let {
                name: "cc".to_string(),
                value: Expr::Literal(Literal::Str("gcc".to_string())),
                declaration: true,
            },
            Stmt::Expr(Expr::Variable("some_var".to_string())),
        ]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("CC := gcc"));
    }

    #[test]
    fn test_MAKE_COV_003_non_main_function_non_echo_body() {
        // Lines 87-88: Function body stmt is FunctionCall but NOT echo/println
        let ast = RestrictedAst {
            functions: vec![
                Function {
                    name: "main".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Let {
                        name: "cc".to_string(),
                        value: Expr::Literal(Literal::Str("gcc".to_string())),
                        declaration: true,
                    }],
                },
                Function {
                    name: "setup".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Expr(Expr::FunctionCall {
                        name: "run_cmd".to_string(),
                        args: vec![Expr::Literal(Literal::Str("init".to_string()))],
                    })],
                },
            ],
            entry_point: "main".to_string(),
        };

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("CC := gcc"));
        // "setup" function has no echo/println so no target is generated
        assert!(!result.contains("setup:"));
    }

    #[test]
    fn test_MAKE_COV_004_non_main_function_non_expr_stmt() {
        // Lines 87-88: Function body stmt is NOT Stmt::Expr(FunctionCall{..})
        let ast = RestrictedAst {
            functions: vec![
                Function {
                    name: "main".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Let {
                        name: "cc".to_string(),
                        value: Expr::Literal(Literal::Str("gcc".to_string())),
                        declaration: true,
                    }],
                },
                Function {
                    name: "init".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Return(None)],
                },
            ],
            entry_point: "main".to_string(),
        };

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("CC := gcc"));
    }

    #[test]
    fn test_MAKE_COV_005_extract_string_array_dunder_array() {
        // Lines 204-209: __array__ function call in extract_string_array
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "target".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("build".to_string())),
                Expr::FunctionCall {
                    name: "__array__".to_string(),
                    args: vec![
                        Expr::Literal(Literal::Str("dep1".to_string())),
                        Expr::Literal(Literal::Str("dep2".to_string())),
                    ],
                },
                Expr::Array(vec![Expr::Literal(Literal::Str(
                    "gcc -o build".to_string(),
                ))]),
            ],
        })]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("build"), "Target in: {result}");
    }

    #[test]
    fn test_MAKE_COV_006_extract_string_array_fallback_expr() {
        // Line 217: catch-all _ => in extract_string_array — non-string expr
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "target".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("build".to_string())),
                Expr::Variable("deps_var".to_string()),
                Expr::Array(vec![Expr::Literal(Literal::Str(
                    "gcc -o build".to_string(),
                ))]),
            ],
        })]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("$(DEPS_VAR)"), "Variable ref in: {result}");
    }

    #[test]
    fn test_MAKE_COV_007_expr_to_string_array_succeeds() {
        // Arrays are now converted to space-separated Makefile values
        let ast = make_simple_ast(vec![Stmt::Let {
            name: "data".to_string(),
            value: Expr::Array(vec![
                Expr::Literal(Literal::Str("a".to_string())),
                Expr::Literal(Literal::Str("b".to_string())),
            ]),
            declaration: true,
        }]);

        let result = emit_makefile(&ast);
        assert!(
            result.is_ok(),
            "Array should convert to Makefile value: {:?}",
            result
        );
        let output = result.expect("verified ok above");
        assert!(output.contains("DATA := a b"), "Output: {}", output);
    }

    #[test]
    fn test_MAKE_COV_007b_expr_to_string_unsupported() {
        // Truly unsupported expression types still fail
        let ast = make_simple_ast(vec![Stmt::Let {
            name: "data".to_string(),
            value: Expr::Block(vec![]),
            declaration: true,
        }]);

        let err = emit_makefile(&ast).unwrap_err();
        assert!(
            format!("{err}").contains("Cannot convert expression"),
            "Error: {err}"
        );
    }

    #[test]
    fn test_MAKE_COV_008_non_main_function_empty_recipes() {
        // Line 104: !recipes.is_empty() is false — function with no echo
        let ast = RestrictedAst {
            functions: vec![
                Function {
                    name: "main".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Let {
                        name: "cc".to_string(),
                        value: Expr::Literal(Literal::Str("gcc".to_string())),
                        declaration: true,
                    }],
                },
                Function {
                    name: "check".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Let {
                        name: "x".to_string(),
                        value: Expr::Literal(Literal::Str("val".to_string())),
                        declaration: true,
                    }],
                },
            ],
            entry_point: "main".to_string(),
        };

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("CC := gcc"));
        // check function has no echo → no target
        assert!(!result.contains("check:"));
    }

    #[test]
    fn test_MAKE_BUILD_014_combined_vars_and_targets() {
        let ast = make_simple_ast(vec![
            Stmt::Let {
                name: "cc".to_string(),
                value: Expr::Literal(Literal::Str("gcc".to_string())),
                declaration: true,
            },
            Stmt::Let {
                name: "cflags".to_string(),
                value: Expr::Literal(Literal::Str("-O2".to_string())),
                declaration: true,
            },
            Stmt::Expr(Expr::FunctionCall {
                name: "phony_target".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("all".to_string())),
                    Expr::Array(vec![Expr::Literal(Literal::Str("build".to_string()))]),
                    Expr::Array(vec![]),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "target".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("build".to_string())),
                    Expr::Array(vec![Expr::Literal(Literal::Str("main.c".to_string()))]),
                    Expr::Array(vec![Expr::Literal(Literal::Str(
                        "$(CC) $(CFLAGS) -o build main.c".to_string(),
                    ))]),
                ],
            }),
        ]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("CC := gcc"));
        assert!(result.contains("CFLAGS := -O2"));
        assert!(result.contains("all"), "all target in: {result}");
        assert!(result.contains("build"), "build target in: {result}");
    }
}
