// REPL AST Display Module
//
// Task: REPL-004-002 - AST display mode
// Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//
// Quality targets:
// - Unit tests: 6+ scenarios
// - Integration tests: 2+ CLI tests
// - Complexity: <10 per function

use crate::bash_parser::{BashAst, BashStmt};

/// Format AST for display in REPL
///
/// # Examples
///
/// ```
/// use bashrs::repl::ast_display::format_ast;
/// use bashrs::bash_parser::{BashAst, ast::AstMetadata};
///
/// let ast = BashAst {
///     statements: vec![],
///     metadata: AstMetadata {
///         source_file: None,
///         line_count: 0,
///         parse_time_ms: 0,
///     },
/// };
/// let output = format_ast(&ast);
/// assert!(output.contains("AST"));
/// ```
pub fn format_ast(ast: &BashAst) -> String {
    let mut output = String::new();
    output.push_str("=== AST ===\n");
    output.push_str(&format!("Statements: {}\n", ast.statements.len()));

    for (i, stmt) in ast.statements.iter().enumerate() {
        output.push_str(&format!("\n[{}] {}\n", i, format_statement(stmt, 0)));
    }

    output
}

/// Format a single statement with indentation
fn format_statement(stmt: &BashStmt, indent: usize) -> String {
    let indent_str = "  ".repeat(indent);

    match stmt {
        BashStmt::Command { name, args, .. } => {
            if args.is_empty() {
                format!("{}Command: {}", indent_str, name)
            } else {
                format!("{}Command: {} (args: {})", indent_str, name, args.len())
            }
        }
        BashStmt::Assignment { name, .. } => {
            format!("{}Assignment: {}", indent_str, name)
        }
        BashStmt::If {
            then_block,
            elif_blocks,
            else_block,
            ..
        } => {
            let mut s = format!("{}If statement", indent_str);
            s.push_str(&format!(
                "\n{}  then: {} statements",
                indent_str,
                then_block.len()
            ));
            if !elif_blocks.is_empty() {
                s.push_str(&format!(
                    "\n{}  elif: {} branches",
                    indent_str,
                    elif_blocks.len()
                ));
            }
            if else_block.is_some() {
                s.push_str(&format!("\n{}  else: present", indent_str));
            }
            s
        }
        BashStmt::While { body, .. } => {
            format!("{}While loop ({} statements)", indent_str, body.len())
        }
        BashStmt::For { variable, body, .. } => {
            format!(
                "{}For loop: {} ({} statements)",
                indent_str,
                variable,
                body.len()
            )
        }
        BashStmt::ForCStyle {
            init,
            condition,
            increment,
            body,
            ..
        } => {
            format!(
                "{}C-style for loop: {}; {}; {} ({} statements)",
                indent_str,
                init,
                condition,
                increment,
                body.len()
            )
        }
        BashStmt::Function { name, body, .. } => {
            format!(
                "{}Function: {} ({} statements)",
                indent_str,
                name,
                body.len()
            )
        }
        BashStmt::Case { arms, .. } => {
            format!("{}Case statement ({} arms)", indent_str, arms.len())
        }
        BashStmt::Until { body, .. } => {
            format!("{}Until loop ({} statements)", indent_str, body.len())
        }
        BashStmt::Return { .. } => {
            format!("{}Return statement", indent_str)
        }
        BashStmt::Comment { text, .. } => {
            format!(
                "{}Comment: {}",
                indent_str,
                text.lines().next().unwrap_or("")
            )
        }
        BashStmt::Pipeline { commands, .. } => {
            format!("{}Pipeline ({} commands)", indent_str, commands.len())
        }
        BashStmt::AndList { .. } => {
            format!("{}AndList (&&)", indent_str)
        }
        BashStmt::OrList { .. } => {
            format!("{}OrList (||)", indent_str)
        }
        BashStmt::BraceGroup { body, .. } => {
            format!("{}BraceGroup ({} statements)", indent_str, body.len())
        }
        BashStmt::Coproc { name, body, .. } => {
            if let Some(n) = name {
                format!("{}Coproc: {} ({} statements)", indent_str, n, body.len())
            } else {
                format!("{}Coproc ({} statements)", indent_str, body.len())
            }
        }
        BashStmt::Select { variable, body, .. } => {
            format!(
                "{}Select: {} ({} statements)",
                indent_str,
                variable,
                body.len()
            )
        }

        BashStmt::Negated { command, .. } => {
            format!(
                "{}Negated: {}",
                indent_str,
                format_statement(command, indent + 1)
            )
        }
    }
}

#[cfg(test)]
#[path = "ast_display_tests_dummy_span.rs"]
mod tests_extracted;
