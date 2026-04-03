use super::trace::{DecisionTrace, TranspilerDecision};
use crate::ir::{ShellIR, ShellValue};
use crate::models::Result;
use std::cell::RefCell;
use std::fmt::Write;
/// KAIZEN-083: Pre-computed indent strings to avoid `"    ".repeat(n)` allocation per emit call.
/// Covers depths 0-8 (all practical shell scripts). Falls back to heap allocation for deeper nesting.
const INDENT_CACHE: [&str; 9] = [
    "",
    "    ",
    "        ",
    "            ",
    "                ",
    "                    ",
    "                        ",
    "                            ",
    "                                ",
];
/// Return a cached indent string for the given depth, or allocate for rare deep nesting.
pub(crate) fn get_indent(depth: usize) -> std::borrow::Cow<'static, str> {
    if depth < INDENT_CACHE.len() {
        std::borrow::Cow::Borrowed(INDENT_CACHE[depth])
    } else {
        std::borrow::Cow::Owned("    ".repeat(depth))
    }
}
/// Returns the shell operator string for an ArithmeticOp
pub(crate) fn arithmetic_op_str(op: &crate::ir::shell_ir::ArithmeticOp) -> &'static str {
    use crate::ir::shell_ir::ArithmeticOp;
    match op {
        ArithmeticOp::Add => "+",
        ArithmeticOp::Sub => "-",
        ArithmeticOp::Mul => "*",
        ArithmeticOp::Div => "/",
        ArithmeticOp::Mod => "%",
        ArithmeticOp::BitAnd => "&",
        ArithmeticOp::BitOr => "|",
        ArithmeticOp::BitXor => "^",
        ArithmeticOp::Shl => "<<",
        ArithmeticOp::Shr => ">>",
    }
}
/// Returns precedence level for arithmetic operators (higher = binds tighter).
/// Follows POSIX shell / C operator precedence.
pub(crate) fn arithmetic_precedence(op: &crate::ir::shell_ir::ArithmeticOp) -> u8 {
    use crate::ir::shell_ir::ArithmeticOp;
    match op {
        ArithmeticOp::BitOr => 0,
        ArithmeticOp::BitXor => 1,
        ArithmeticOp::BitAnd => 2,
        ArithmeticOp::Shl | ArithmeticOp::Shr => 3,
        ArithmeticOp::Add | ArithmeticOp::Sub => 4,
        ArithmeticOp::Mul | ArithmeticOp::Div | ArithmeticOp::Mod => 5,
    }
}
/// Try to constant-fold a boolean expression at compile time.
/// Returns `Some(bool)` if all operands are literals, `None` if any are runtime values.
pub(crate) fn try_fold_logical(value: &ShellValue) -> Option<bool> {
    match value {
        ShellValue::Bool(b) => Some(*b),
        ShellValue::LogicalNot { operand } => try_fold_logical(operand).map(|b| !b),
        ShellValue::LogicalAnd { left, right } => {
            Some(try_fold_logical(left)? && try_fold_logical(right)?)
        }
        ShellValue::LogicalOr { left, right } => {
            Some(try_fold_logical(left)? || try_fold_logical(right)?)
        }
        _ => None,
    }
}
/// Unwrap a Sequence containing a single If statement.
/// The parser wraps `else if` branches in `Sequence([If { ... }])`.
pub(crate) fn unwrap_single_if(ir: &ShellIR) -> &ShellIR {
    if let ShellIR::Sequence(items) = ir {
        if items.len() == 1 && matches!(&items[0], ShellIR::If { .. }) {
            return &items[0];
        }
    }
    ir
}
/// Classify an if-statement structure for decision tracing.
pub(crate) fn classify_if_structure(else_branch: Option<&ShellIR>) -> &'static str {
    match else_branch {
        None => "simple_if",
        Some(ir) => {
            let unwrapped = unwrap_single_if(ir);
            if matches!(unwrapped, ShellIR::If { .. }) {
                "elif_chain"
            } else {
                "if_else"
            }
        }
    }
}
/// Classify a test expression for decision tracing.
pub(crate) fn classify_test_expression(test: &ShellValue) -> &'static str {
    match test {
        ShellValue::Bool(_) => "bool_literal",
        ShellValue::Variable(_) => "variable_test",
        ShellValue::String(_) => "string_check",
        ShellValue::Comparison { .. } => "bracket_test",
        ShellValue::LogicalAnd { .. }
        | ShellValue::LogicalOr { .. }
        | ShellValue::LogicalNot { .. } => "logical_op",
        ShellValue::CommandSubst(_) => "cmd_subst_test",
        _ => "other_test",
    }
}
/// Determine whether a child arithmetic expression needs parenthesization
/// relative to its parent operator. Returns true if parens are required.
pub(crate) fn needs_arithmetic_parens(
    child_op: &crate::ir::shell_ir::ArithmeticOp,
    parent_op: Option<&crate::ir::shell_ir::ArithmeticOp>,
    is_right: bool,
) -> bool {
    if let Some(parent) = parent_op {
        let child_prec = arithmetic_precedence(child_op);
        let parent_prec = arithmetic_precedence(parent);
        child_prec < parent_prec || (child_prec == parent_prec && is_right)
    } else {
        false
    }
}
pub struct PosixEmitter {
    trace: RefCell<Vec<TranspilerDecision>>,
    tracing: bool,
}
impl Default for PosixEmitter {
    fn default() -> Self {
        Self {
            trace: RefCell::new(Vec::new()),
            tracing: false,
        }
    }
}
impl PosixEmitter {
    pub fn new() -> Self {
        Self::default()
    }
    /// Create an emitter with decision tracing enabled.
    pub fn new_with_tracing() -> Self {
        Self {
            trace: RefCell::new(Vec::new()),
            tracing: true,
        }
    }
    /// Record a decision made during emission.
    /// No-op when tracing is disabled (the default for normal emit).
    #[inline]
    pub(crate) fn record_decision(&self, decision_type: &str, choice: &str, ir_node: &str) {
        if !self.tracing {
            return;
        }
        self.trace.borrow_mut().push(TranspilerDecision {
            decision_type: decision_type.to_string(),
            choice: choice.to_string(),
            ir_node: ir_node.to_string(),
        });
    }
    /// Drain and return the accumulated decision trace.
    pub fn take_trace(&self) -> DecisionTrace {
        self.trace.borrow_mut().drain(..).collect()
    }
    pub fn emit(&self, ir: &ShellIR) -> Result<String> {
        // Contract: encoder-roundtrip-v1.yaml precondition (pv codegen)
        contract_pre_configuration!(ir);
        let mut output = String::new();
        // Collect used functions for selective runtime emission
        let used_functions = ir.collect_used_functions();
        // Write the POSIX shell header (without main wrapper yet)
        self.write_header_without_main(&mut output, &used_functions)?;
        // Separate helper functions from main body
        let (helper_functions, main_body) = self.separate_functions(ir);
        // Emit helper functions at global scope
        for func_ir in &helper_functions {
            self.emit_ir(&mut output, func_ir, 0)?;
            writeln!(&mut output)?;
        }
        // Now open main() and emit its body
        writeln!(&mut output, "# Main script begins")?;
        writeln!(&mut output, "main() {{")?;
        if main_body.is_empty() {
            // Empty main needs a no-op for valid shell syntax
            writeln!(&mut output, "    :")?;
        } else {
            for stmt_ir in &main_body {
                self.emit_ir(&mut output, stmt_ir, 1)?;
            }
        }
        // Write the footer (closes main and adds execution)
        self.write_footer(&mut output)?;
        Ok(output)
    }
    /// KAIZEN-076: return references instead of cloning IR items.
    /// emit_ir takes &ShellIR, so cloning was unnecessary (~107K clones per corpus run).
    pub(crate) fn separate_functions<'a>(
        &self,
        ir: &'a ShellIR,
    ) -> (Vec<&'a ShellIR>, Vec<&'a ShellIR>) {
        let mut functions = Vec::new();
        let mut main_body = Vec::new();
        if let ShellIR::Sequence(items) = ir {
            for item in items {
                match item {
                    ShellIR::Function { .. } => functions.push(item),
                    _ => main_body.push(item),
                }
            }
        } else {
            main_body.push(ir);
        }
        (functions, main_body)
    }
    pub(crate) fn write_header_without_main(
        &self,
        output: &mut String,
        used_functions: &std::collections::HashSet<&str>,
    ) -> Result<()> {
        writeln!(output, "#!/bin/sh")?;
        writeln!(output, "# Generated by Rash v{}", env!("CARGO_PKG_VERSION"))?;
        writeln!(output, "# POSIX-compliant shell script")?;
        writeln!(output)?;
        // Set strict error handling
        writeln!(output, "set -euf")?;
        writeln!(output, "IFS=' \t\n'")?; // POSIX-compatible IFS setting
        writeln!(output, "export LC_ALL=C")?;
        writeln!(output)?;
        // Include only the runtime functions actually used by the IR
        if self.needs_runtime(used_functions) {
            self.write_selective_runtime(output, used_functions)?;
        }
        Ok(())
    }
    pub(crate) fn write_footer(&self, output: &mut String) -> Result<()> {
        writeln!(output, "}}")?;
        writeln!(output)?;
        writeln!(output, "# Cleanup on exit")?;
        writeln!(output, "trap 'rm -rf \"${{TMPDIR:-/tmp}}/rash.$$\"' EXIT")?;
        writeln!(output)?;
        writeln!(output, "# Execute main function")?;
        writeln!(output, "main \"$@\"")?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "posix_tests.rs"]
mod tests;
