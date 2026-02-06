//! Makefile code emitter
//!
//! Converts a RestrictedAst (Rust DSL) into a valid Makefile using conventions:
//! - `let` bindings with string values -> Makefile variables (SIMPLE `:=`)
//! - `target("name", &["deps"], &["recipes"])` function calls -> Make targets
//! - `phony_target("name", ...)` -> `.PHONY` targets
//!
//! The generated Makefile is passed through the existing Makefile generator
//! infrastructure for consistent formatting.

use crate::ast::restricted::Literal;
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
    let converter = MakefileConverter::new();
    let make_ast = converter.convert(ast)?;
    Ok(generate_purified_makefile(&make_ast))
}

struct MakefileConverter {
    line: usize,
}

impl MakefileConverter {
    fn new() -> Self {
        Self { line: 1 }
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

    fn convert(&self, ast: &RestrictedAst) -> Result<MakeAst> {
        let mut converter = MakefileConverter::new();
        let mut items = Vec::new();

        // Find the entry point function
        let entry_fn = ast
            .functions
            .iter()
            .find(|f| f.name == ast.entry_point)
            .ok_or_else(|| Error::IrGeneration("Entry point not found".to_string()))?;

        // Convert each statement
        for stmt in &entry_fn.body {
            if let Some(item) = converter.convert_stmt(stmt)? {
                items.push(item);
            }
        }

        // Also convert non-main functions as potential helper targets
        for function in &ast.functions {
            if function.name != ast.entry_point {
                // Convert function definitions to targets with empty prerequisites
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

                // Only generate target if function has meaningful body
                if !recipes.is_empty() {
                    let params: Vec<String> = function
                        .params
                        .iter()
                        .map(|p| p.name.clone())
                        .collect();

                    items.push(MakeItem::Target {
                        name: function.name.clone(),
                        prerequisites: params,
                        recipe: recipes,
                        phony: true,
                        recipe_metadata: None,
                        span: converter.next_span(),
                    });
                }
            }
        }

        Ok(MakeAst {
            items,
            metadata: MakeMetadata::new(),
        })
    }

    fn convert_stmt(&mut self, stmt: &Stmt) -> Result<Option<MakeItem>> {
        match stmt {
            Stmt::Let { name, value } => self.convert_let(name, value),
            Stmt::Expr(expr) => self.convert_expr(expr),
            _ => Ok(None),
        }
    }

    fn convert_let(&mut self, name: &str, value: &Expr) -> Result<Option<MakeItem>> {
        // let cc = "gcc"; -> CC := gcc
        let var_value = self.expr_to_string(value)?;
        let var_name = name.to_uppercase();

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
                } else {
                    Ok(None) // Ignore other function calls
                }
            }
            _ => Ok(None),
        }
    }

    fn convert_target_call(
        &mut self,
        func_name: &str,
        args: &[Expr],
    ) -> Result<Option<MakeItem>> {
        // target("name", &["dep1", "dep2"], &["recipe1", "recipe2"])
        // phony_target("name", &["dep1"], &["recipe1"])
        if args.len() < 3 {
            return Err(Error::Validation(format!(
                "{}() requires 3 arguments: name, dependencies, recipes",
                func_name
            )));
        }

        // Extract target name (safe indexing: we verified args.len() >= 3 above)
        let target_name = match args.first().expect("verified args.len() >= 3") {
            Expr::Literal(Literal::Str(s)) => s.clone(),
            _ => {
                return Err(Error::Validation(format!(
                    "{}() first argument must be a string literal (target name)",
                    func_name
                )))
            }
        };

        // Extract dependencies (from array literal) - safe: verified args.len() >= 3
        let deps_arg = args.get(1).expect("verified args.len() >= 3");
        let deps = self.extract_string_array(deps_arg)?;

        // Extract recipes (from array literal) - safe: verified args.len() >= 3
        let recipes_arg = args.get(2).expect("verified args.len() >= 3");
        let recipes = self.extract_string_array(recipes_arg)?;

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
        }]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("CC := gcc"), "Expected 'CC := gcc' in: {}", result);
    }

    #[test]
    fn test_MAKE_BUILD_002_multiple_variables() {
        let ast = make_simple_ast(vec![
            Stmt::Let {
                name: "cc".to_string(),
                value: Expr::Literal(Literal::Str("gcc".to_string())),
            },
            Stmt::Let {
                name: "cflags".to_string(),
                value: Expr::Literal(Literal::Str("-O2 -Wall".to_string())),
            },
        ]);

        let result = emit_makefile(&ast).unwrap();
        assert!(result.contains("CC := gcc"));
        assert!(result.contains("CFLAGS := -O2 -Wall"));
    }
}
