// Control flow purification for Bash scripts
//
// Handles: If, While, Until, For, ForCStyle, Case, Select

use crate::bash_parser::ast::*;
use super::{PurificationResult, Purifier};

impl Purifier {
    /// Purify control flow statements: If, While, Until, For, ForCStyle, Case, Select
    pub(super) fn purify_control_flow(&mut self, stmt: &BashStmt) -> PurificationResult<BashStmt> {
        match stmt {
            BashStmt::If {
                condition,
                then_block,
                elif_blocks,
                else_block,
                span,
            } => {
                let purified_condition = self.purify_expression(condition)?;

                let purified_then = self.purify_body(then_block)?;

                let mut purified_elif = Vec::new();
                for (cond, body) in elif_blocks {
                    let p_cond = self.purify_expression(cond)?;
                    let p_body = self.purify_body(body)?;
                    purified_elif.push((p_cond, p_body));
                }

                let purified_else = if let Some(else_body) = else_block {
                    Some(self.purify_body(else_body)?)
                } else {
                    None
                };

                Ok(BashStmt::If {
                    condition: purified_condition,
                    then_block: purified_then,
                    elif_blocks: purified_elif,
                    else_block: purified_else,
                    span: *span,
                })
            }

            BashStmt::While {
                condition,
                body,
                span,
            } => {
                let purified_condition = self.purify_expression(condition)?;
                let purified_body = self.purify_body(body)?;

                Ok(BashStmt::While {
                    condition: purified_condition,
                    body: purified_body,
                    span: *span,
                })
            }

            BashStmt::Until {
                condition,
                body,
                span,
            } => {
                let purified_condition = self.purify_expression(condition)?;
                let purified_body = self.purify_body(body)?;

                Ok(BashStmt::Until {
                    condition: purified_condition,
                    body: purified_body,
                    span: *span,
                })
            }

            BashStmt::For {
                variable,
                items,
                body,
                span,
            } => {
                let purified_items = self.purify_expression(items)?;
                let purified_body = self.purify_body(body)?;

                Ok(BashStmt::For {
                    variable: variable.clone(),
                    items: purified_items,
                    body: purified_body,
                    span: *span,
                })
            }

            // Issue #68: Purify C-style for loop (already handled by codegen)
            BashStmt::ForCStyle {
                init,
                condition,
                increment,
                body,
                span,
            } => {
                // Purify the body statements
                let purified_body = self.purify_body(body)?;

                // Return the purified C-style for loop as-is
                // The codegen will convert it to POSIX while loop
                Ok(BashStmt::ForCStyle {
                    init: init.clone(),
                    condition: condition.clone(),
                    increment: increment.clone(),
                    body: purified_body,
                    span: *span,
                })
            }

            BashStmt::Case { word, arms, span } => {
                let purified_word = self.purify_expression(word)?;

                let mut purified_arms = Vec::new();
                for arm in arms {
                    let purified_body = self.purify_body(&arm.body)?;
                    purified_arms.push(crate::bash_parser::ast::CaseArm {
                        patterns: arm.patterns.clone(),
                        body: purified_body,
                    });
                }

                Ok(BashStmt::Case {
                    word: purified_word,
                    arms: purified_arms,
                    span: *span,
                })
            }

            BashStmt::Select {
                variable,
                items,
                body,
                span,
            } => {
                // F017: Purify select statement
                let purified_items = self.purify_expression(items)?;
                let purified_body = self.purify_body(body)?;

                Ok(BashStmt::Select {
                    variable: variable.clone(),
                    items: purified_items,
                    body: purified_body,
                    span: *span,
                })
            }

            _ => Ok(stmt.clone()),
        }
    }
}
