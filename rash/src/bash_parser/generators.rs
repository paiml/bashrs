//! Proptest Generators for Bash Syntax
//!
//! Generates random but valid bash constructs for property-based testing.
//! Also includes purified bash generation for the bash→rust→purified pipeline.

use super::ast::*;
use proptest::prelude::*;
use proptest::strategy::BoxedStrategy;

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
        output.push_str(&generate_statement(stmt));
        output.push('\n');
    }

    output
}

/// Generate a single statement
fn generate_statement(stmt: &BashStmt) -> String {
    match stmt {
        BashStmt::Command { name, args, .. } => generate_stmt_command(name, args),
        BashStmt::Assignment {
            name,
            value,
            exported,
            ..
        } => generate_stmt_assignment(name, value, *exported),
        BashStmt::Comment { text, .. } => format!("# {}", text),
        BashStmt::Function { name, body, .. } => generate_stmt_function(name, body),
        BashStmt::If {
            condition,
            then_block,
            else_block,
            ..
        } => generate_stmt_if(condition, then_block, else_block.as_deref()),
        BashStmt::For {
            variable,
            items,
            body,
            ..
        } => generate_stmt_for(variable, items, body),
        BashStmt::ForCStyle {
            init,
            condition,
            increment,
            body,
            ..
        } => generate_stmt_for_c_style(init, condition, increment, body),
        BashStmt::While {
            condition, body, ..
        } => generate_stmt_while(condition, body),
        BashStmt::Until {
            condition, body, ..
        } => generate_stmt_until(condition, body),
        BashStmt::Return { code, .. } => generate_stmt_return(code.as_ref()),
        BashStmt::Case { word, arms, .. } => generate_stmt_case(word, arms),
        BashStmt::Pipeline { commands, .. } => generate_stmt_pipeline(commands),
        BashStmt::AndList { left, right, .. } => {
            format!(
                "{} && {}",
                generate_statement(left),
                generate_statement(right)
            )
        }
        BashStmt::OrList { left, right, .. } => {
            format!(
                "{} || {}",
                generate_statement(left),
                generate_statement(right)
            )
        }
        BashStmt::BraceGroup { body, .. } => generate_stmt_brace_group(body),
        BashStmt::Coproc { name, body, .. } => generate_stmt_coproc(name.as_deref(), body),
        BashStmt::Select {
            variable,
            items,
            body,
            ..
        } => generate_stmt_select(variable, items, body),
        BashStmt::Negated { command, .. } => format!("! {}", generate_statement(command)),
    }
}

/// Append indented body statements to the output buffer
fn append_indented_body(output: &mut String, body: &[BashStmt]) {
    for stmt in body {
        output.push_str("    ");
        output.push_str(&generate_statement(stmt));
        output.push('\n');
    }
}

/// Generate a command statement: name arg1 arg2 ...
fn generate_stmt_command(name: &str, args: &[BashExpr]) -> String {
    let mut cmd = name.to_string();
    for arg in args {
        cmd.push(' ');
        cmd.push_str(&generate_expr(arg));
    }
    cmd
}

/// Generate an assignment statement: [export] name=value
fn generate_stmt_assignment(name: &str, value: &BashExpr, exported: bool) -> String {
    let mut assign = String::new();
    if exported {
        assign.push_str("export ");
    }
    assign.push_str(name);
    assign.push('=');
    assign.push_str(&generate_expr(value));
    assign
}

/// Generate a function definition: name() { body }

include!("generators_incl2.rs");
