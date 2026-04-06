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
    let shell = crate::emitter::emit(&ir)?;
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

include!("makefile_makefileconvert.rs");
