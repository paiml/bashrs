//! Makefile code emitter
//!
//! Converts a RestrictedAst (Rust DSL) into a valid Makefile using conventions:
//! - `let` bindings with string values -> Makefile variables (SIMPLE `:=`)
//! - `target("name", &["deps"], &["recipes"])` function calls -> Make targets
//! - `phony_target("name", ...)` -> `.PHONY` targets
//!
//! The generated Makefile is passed through the existing Makefile generator
//! infrastructure for consistent formatting.

use crate::ast::restricted::{BinaryOp, Literal};
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
                    let params: Vec<String> =
                        function.params.iter().map(|p| p.name.clone()).collect();

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
            Stmt::Let { name, value, .. } => self.convert_let(name, value),
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
        let target_name = match args.first().expect("verified args.len() >= 2") {
            Expr::Literal(Literal::Str(s)) => s.clone(),
            _ => {
                return Err(Error::Validation(format!(
                    "{}() first argument must be a string literal (target name)",
                    func_name
                )))
            }
        };

        // Extract dependencies (from array literal) - safe: verified args.len() >= 2
        let deps_arg = args.get(1).expect("verified args.len() >= 2");
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
                Expr::Array(vec![
                    Expr::Literal(Literal::Str("src/main.c".to_string())),
                ]),
                Expr::Array(vec![
                    Expr::Literal(Literal::Str("$(CC) -o build src/main.c".to_string())),
                ]),
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
                Expr::Array(vec![
                    Expr::Literal(Literal::Str("rm -f build".to_string())),
                ]),
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
            args: vec![
                Expr::Literal(Literal::I32(42)),
                Expr::Array(vec![]),
            ],
        })]);

        let err = emit_makefile(&ast).unwrap_err();
        assert!(
            format!("{err}").contains("string literal"),
            "Error: {err}"
        );
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
                Expr::Array(vec![
                    Expr::Literal(Literal::Str("gcc -o build main.c".to_string())),
                ]),
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
                Expr::Array(vec![
                    Expr::Literal(Literal::Str("rm -f build".to_string())),
                ]),
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
                        args: vec![Expr::Literal(Literal::Str(
                            "Usage: make build".to_string(),
                        ))],
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
                Expr::Array(vec![
                    Expr::Literal(Literal::Str("gcc -o build".to_string())),
                ]),
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
                Expr::Array(vec![
                    Expr::Literal(Literal::Str("gcc -o build".to_string())),
                ]),
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
        assert!(result.is_ok(), "Array should convert to Makefile value: {:?}", result);
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
                    body: vec![
                        Stmt::Let {
                            name: "x".to_string(),
                            value: Expr::Literal(Literal::Str("val".to_string())),
                            declaration: true,
                        },
                    ],
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
                    Expr::Array(vec![
                        Expr::Literal(Literal::Str("build".to_string())),
                    ]),
                    Expr::Array(vec![]),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "target".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("build".to_string())),
                    Expr::Array(vec![
                        Expr::Literal(Literal::Str("main.c".to_string())),
                    ]),
                    Expr::Array(vec![
                        Expr::Literal(Literal::Str("$(CC) $(CFLAGS) -o build main.c".to_string())),
                    ]),
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
