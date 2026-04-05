//! Bash Code Generation
//!
//! Generates purified bash scripts from BashAst.
//! Used by the `bashrs purify` command to emit safe, deterministic bash.

use super::ast::*;

/// Generate purified bash from BashAst
///
/// This function transforms a BashAst into purified POSIX sh:
/// - Transforms #!/bin/bash → #!/bin/sh
/// - Ensures deterministic output (no $RANDOM, timestamps)
/// - Ensures idempotent operations (mkdir -p, rm -f)
/// - Quotes all variables for injection safety
///
/// Task 1.1: Shebang Transformation
pub fn generate_purified_bash(ast: &BashAst) -> String {
    let mut output = String::new();

    // Always start with POSIX sh shebang
    output.push_str("#!/bin/sh\n");

    // Generate statements
    for stmt in &ast.statements {
        output.push_str(&generate_stmt(stmt, 0));
        output.push('\n');
    }

    output
}

/// Generate a single statement (top-level, no indentation)
fn generate_statement(stmt: &BashStmt) -> String {
    generate_stmt(stmt, 0)
}

/// Generate a statement with proper indentation at the given nesting level.
/// Each level adds 4 spaces of indentation.
fn generate_stmt(stmt: &BashStmt, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    match stmt {
        BashStmt::Command {
            name,
            args,
            redirects,
            ..
        } => generate_command_stmt(&pad, name, args, redirects),
        BashStmt::Assignment {
            name,
            value,
            exported,
            ..
        } => generate_assignment_stmt(&pad, name, value, *exported),
        BashStmt::Comment { text, .. } => generate_comment_stmt(&pad, text),
        BashStmt::Function { name, body, .. } => generate_function_stmt(&pad, name, body, indent),
        BashStmt::If {
            condition,
            then_block,
            elif_blocks,
            else_block,
            ..
        } => generate_if_stmt(&pad, condition, then_block, elif_blocks, else_block, indent),
        BashStmt::For {
            variable,
            items,
            body,
            ..
        } => generate_loop_body(
            &format!("{}for {} in {}; do", pad, variable, generate_expr(items)),
            &pad,
            body,
            indent,
        ),
        BashStmt::ForCStyle {
            init,
            condition,
            increment,
            body,
            ..
        } => {
            let inner_pad = "    ".repeat(indent + 1);
            generate_for_c_style(&pad, &inner_pad, init, condition, increment, body, indent)
        }
        BashStmt::While {
            condition, body, ..
        } => generate_loop_body(
            &format!("{}while {}; do", pad, generate_condition(condition)),
            &pad,
            body,
            indent,
        ),
        BashStmt::Until {
            condition, body, ..
        } => generate_loop_body(
            &format!("{}while {}; do", pad, negate_condition(condition)),
            &pad,
            body,
            indent,
        ),
        BashStmt::Return { code, .. } => code.as_ref().map_or_else(
            || format!("{}return", pad),
            |c| format!("{}return {}", pad, generate_expr(c)),
        ),
        BashStmt::Case { word, arms, .. } => generate_case_stmt(&pad, word, arms, indent),
        BashStmt::Pipeline { commands, .. } => generate_pipeline(&pad, commands),
        BashStmt::AndList { left, right, .. } => {
            format!(
                "{}{} && {}",
                pad,
                generate_statement(left),
                generate_statement(right)
            )
        }
        BashStmt::OrList { left, right, .. } => {
            format!(
                "{}{} || {}",
                pad,
                generate_statement(left),
                generate_statement(right)
            )
        }
        BashStmt::BraceGroup { body, subshell, .. } => {
            generate_brace_group(&pad, body, *subshell, indent)
        }
        BashStmt::Coproc { name, body, .. } => generate_coproc(&pad, name, body),
        BashStmt::Select {
            variable,
            items,
            body,
            ..
        } => generate_loop_body(
            &format!("{}select {} in {}; do", pad, variable, generate_expr(items)),
            &pad,
            body,
            indent,
        ),
        BashStmt::Negated { command, .. } => {
            format!("{}! {}", pad, generate_statement(command))
        }
    }
}

/// Generate a command statement (including declare/typeset POSIX conversion)
fn generate_command_stmt(
    pad: &str,
    name: &str,
    args: &[BashExpr],
    redirects: &[Redirect],
) -> String {
    if name == "declare" || name == "typeset" {
        return format!("{}{}", pad, generate_declare_posix(args, redirects));
    }
    let mut cmd = format!("{}{}", pad, name);
    for arg in args {
        cmd.push(' ');
        cmd.push_str(&generate_expr(arg));
    }
    for redirect in redirects {
        cmd.push(' ');
        cmd.push_str(&generate_redirect(redirect));
    }
    cmd
}

/// Generate an assignment statement
fn generate_assignment_stmt(pad: &str, name: &str, value: &BashExpr, exported: bool) -> String {
    let mut assign = pad.to_string();
    if exported {
        assign.push_str("export ");
    }
    assign.push_str(name);
    assign.push('=');
    assign.push_str(&generate_expr(value));
    assign
}

/// Generate a comment statement (skipping shebangs)
fn generate_comment_stmt(pad: &str, text: &str) -> String {
    if text.starts_with("!/bin/") || text.starts_with(" !/bin/") {
        return String::new();
    }
    format!("{}# {}", pad, text)
}

/// Generate a function definition

include!("codegen_incl2.rs");
