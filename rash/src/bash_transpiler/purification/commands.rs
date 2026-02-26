// Command purification for Bash scripts
//
// Handles: Command, Pipeline, AndList, OrList, BraceGroup, Coproc
// Also handles making commands idempotent (mkdir -p, rm -f, etc.)

use super::{PurificationResult, Purifier};
use crate::bash_parser::ast::*;

impl Purifier {
    /// Purify command-related statements: Command, Pipeline, AndList, OrList, BraceGroup, Coproc
    pub(super) fn purify_command_stmt(&mut self, stmt: &BashStmt) -> PurificationResult<BashStmt> {
        match stmt {
            BashStmt::Command {
                name,
                args,
                redirects,
                span,
            } => {
                // Detect and transform non-idempotent operations
                // Issue #72: Pass redirects through to preserve them
                let (purified_cmds, idempotent_wrapper) =
                    self.make_command_idempotent(name, args, redirects, *span)?;

                if let Some(wrapper) = idempotent_wrapper {
                    self.report.idempotency_fixes.push(wrapper);
                }

                // If multiple statements were generated (e.g., permission check + command),
                // we need to handle this specially
                if purified_cmds.len() == 1 {
                    // SAFETY: We verified length is 1, so next() will return Some
                    Ok(purified_cmds.into_iter().next().unwrap_or_else(|| {
                        // This should never happen given len check above
                        BashStmt::Comment {
                            text: "ERROR: empty purified_cmds".to_string(),
                            span: *span,
                        }
                    }))
                } else {
                    // For now, we'll return a Pipeline to group multiple statements
                    // This ensures they're executed together
                    Ok(BashStmt::Pipeline {
                        commands: purified_cmds,
                        span: *span,
                    })
                }
            }

            BashStmt::Pipeline { commands, span } => {
                let purified_commands = self.purify_body(commands)?;

                Ok(BashStmt::Pipeline {
                    commands: purified_commands,
                    span: *span,
                })
            }

            BashStmt::AndList { left, right, span } => {
                let purified_left = self.purify_statement(left)?;
                let purified_right = self.purify_statement(right)?;

                Ok(BashStmt::AndList {
                    left: Box::new(purified_left),
                    right: Box::new(purified_right),
                    span: *span,
                })
            }

            BashStmt::OrList { left, right, span } => {
                let purified_left = self.purify_statement(left)?;
                let purified_right = self.purify_statement(right)?;

                Ok(BashStmt::OrList {
                    left: Box::new(purified_left),
                    right: Box::new(purified_right),
                    span: *span,
                })
            }

            BashStmt::BraceGroup {
                body,
                subshell,
                span,
            } => {
                let purified_body = self.purify_body(body)?;

                Ok(BashStmt::BraceGroup {
                    body: purified_body,
                    subshell: *subshell,
                    span: *span,
                })
            }

            BashStmt::Coproc { name, body, span } => {
                let purified_body = self.purify_body(body)?;

                Ok(BashStmt::Coproc {
                    name: name.clone(),
                    body: purified_body,
                    span: *span,
                })
            }

            _ => Ok(stmt.clone()),
        }
    }

    pub(super) fn make_command_idempotent(
        &mut self,
        name: &str,
        args: &[BashExpr],
        redirects: &[Redirect],
        span: Span,
    ) -> PurificationResult<(Vec<BashStmt>, Option<String>)> {
        match name {
            "mkdir" => self.make_mkdir_idempotent(args, redirects, name, span),
            "rm" => self.make_rm_idempotent(args, redirects, name, span),
            "cp" | "mv" => {
                self.report.warnings.push(format!(
                    "Command '{}' may not be idempotent - consider checking if destination exists",
                    name
                ));
                self.build_default_command(name, args, redirects, span, None)
            }
            "echo" | "cat" | "ls" | "grep" => {
                // Read-only commands are already idempotent
                self.build_default_command(name, args, redirects, span, None)
            }
            _ => {
                if self.options.track_side_effects {
                    self.report
                        .side_effects_isolated
                        .push(format!("Side effect detected: command '{}'", name));
                }
                self.build_default_command(name, args, redirects, span, None)
            }
        }
    }

    fn make_mkdir_idempotent(
        &mut self,
        args: &[BashExpr],
        redirects: &[Redirect],
        name: &str,
        span: Span,
    ) -> PurificationResult<(Vec<BashStmt>, Option<String>)> {
        let purified_args: Result<Vec<_>, _> =
            args.iter().map(|arg| self.purify_expression(arg)).collect();
        let purified_args = purified_args?;

        let mut mkdir_args = if !purified_args
            .iter()
            .any(|arg| matches!(arg, BashExpr::Literal(s) if s.starts_with('-') && s.contains('p')))
        {
            vec![BashExpr::Literal("-p".to_string())]
        } else {
            vec![]
        };
        mkdir_args.extend(purified_args);

        Ok((
            vec![BashStmt::Command {
                name: name.to_string(),
                args: mkdir_args,
                redirects: redirects.to_vec(),
                span,
            }],
            Some("Added -p flag to mkdir for idempotency".to_string()),
        ))
    }

    fn make_rm_idempotent(
        &mut self,
        args: &[BashExpr],
        redirects: &[Redirect],
        name: &str,
        span: Span,
    ) -> PurificationResult<(Vec<BashStmt>, Option<String>)> {
        let has_f_flag = args.iter().any(
            |arg| matches!(arg, BashExpr::Literal(s) if s.starts_with('-') && s.contains('f')),
        );
        if !has_f_flag {
            let purified_args: Result<Vec<_>, _> =
                args.iter().map(|arg| self.purify_expression(arg)).collect();
            let mut new_args = vec![BashExpr::Literal("-f".to_string())];
            new_args.extend(purified_args?);

            return Ok((
                vec![BashStmt::Command {
                    name: name.to_string(),
                    args: new_args,
                    redirects: redirects.to_vec(),
                    span,
                }],
                Some("Added -f flag to rm for idempotency".to_string()),
            ));
        }
        self.build_default_command(name, args, redirects, span, None)
    }

    fn build_default_command(
        &mut self,
        name: &str,
        args: &[BashExpr],
        redirects: &[Redirect],
        span: Span,
        fix_msg: Option<String>,
    ) -> PurificationResult<(Vec<BashStmt>, Option<String>)> {
        let purified_args: Result<Vec<_>, _> =
            args.iter().map(|arg| self.purify_expression(arg)).collect();

        Ok((
            vec![BashStmt::Command {
                name: name.to_string(),
                args: purified_args?,
                redirects: redirects.to_vec(),
                span,
            }],
            fix_msg,
        ))
    }
}
