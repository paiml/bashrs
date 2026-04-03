//! Dockerfile code emitter
//!
//! Converts a RestrictedAst (Rust DSL) into a valid Dockerfile using conventions:
//! - `from_image("image", "tag")` -> `FROM image:tag`
//! - `from_image_as("image", "tag", "alias")` -> `FROM image:tag AS alias`
//! - `run(&["cmd1", "cmd2"])` -> `RUN cmd1 && cmd2`
//! - `copy("src", "dst")` -> `COPY src dst`
//! - `copy_from("stage", "src", "dst")` -> `COPY --from=stage src dst`
//! - `workdir("/app")` -> `WORKDIR /app`
//! - `env("KEY", "value")` -> `ENV KEY=value`
//! - `expose(8080)` -> `EXPOSE 8080`
//! - `user("65534")` -> `USER 65534`
//! - `entrypoint(&["/app"])` -> `ENTRYPOINT ["/app"]`
//! - `cmd(&["arg1"])` -> `CMD ["arg1"]`
//! - `label("key", "value")` -> `LABEL key="value"`
//! - `healthcheck("curl -f http://localhost/")` -> `HEALTHCHECK CMD curl -f http://localhost/`

use crate::ast::restricted::Literal;
use crate::ast::{Expr, RestrictedAst, Stmt};
use crate::ir::dockerfile_ir::{DockerInstruction, DockerStage, DockerfileIR};
use crate::models::{Error, Result};

/// Split an "image:tag" string on the last colon.
/// If no colon, uses "latest" as the tag.
fn split_image_tag(combined: &str) -> (String, String) {
    if let Some(pos) = combined.rfind(':') {
        let image = combined[..pos].to_string();
        let tag = combined[pos + 1..].to_string();
        if image.is_empty() || tag.is_empty() {
            (combined.to_string(), "latest".to_string())
        } else {
            (image, tag)
        }
    } else {
        (combined.to_string(), "latest".to_string())
    }
}

/// Convert a RestrictedAst (Rust DSL) to a Dockerfile string.
pub fn emit_dockerfile(ast: &RestrictedAst) -> Result<String> {
    let converter = DockerfileConverter::new();
    let ir = converter.convert(ast)?;
    Ok(ir.emit())
}

struct DockerfileConverter;

impl DockerfileConverter {
    fn new() -> Self {
        Self
    }

    fn convert(&self, ast: &RestrictedAst) -> Result<DockerfileIR> {
        let mut ir = DockerfileIR::new();
        let mut current_stage: Option<DockerStage> = None;

        // Process non-main functions first as preceding build stages
        for func in &ast.functions {
            if func.name == ast.entry_point {
                continue;
            }
            for stmt in &func.body {
                self.convert_stmt(stmt, &mut ir, &mut current_stage)?;
            }
            // Flush stage after each non-main function
            if let Some(stage) = current_stage.take() {
                ir.add_stage(stage);
            }
        }

        // Find the entry point function
        let entry_fn = ast
            .functions
            .iter()
            .find(|f| f.name == ast.entry_point)
            .ok_or_else(|| Error::IrGeneration("Entry point not found".to_string()))?;

        // If main has no from_image() and we already have stages from non-main functions,
        // re-open the last stage so main's instructions attach to it
        let main_has_from = entry_fn.body.iter().any(|s| {
            matches!(s, Stmt::Expr(Expr::FunctionCall { name, .. }) if name == "from_image" || name == "from_image_as")
        });
        if !main_has_from && !ir.stages.is_empty() {
            current_stage = Some(ir.stages.pop().expect("verified non-empty"));
        }

        // Convert each statement in main
        for stmt in &entry_fn.body {
            self.convert_stmt(stmt, &mut ir, &mut current_stage)?;
        }

        // Push the last stage if any
        if let Some(stage) = current_stage {
            ir.add_stage(stage);
        }

        if ir.stages.is_empty() {
            return Err(Error::Validation(
                "Dockerfile DSL requires at least one from_image() call".to_string(),
            ));
        }

        Ok(ir)
    }

    fn convert_stmt(
        &self,
        stmt: &Stmt,
        ir: &mut DockerfileIR,
        current_stage: &mut Option<DockerStage>,
    ) -> Result<()> {
        match stmt {
            Stmt::Expr(expr) => self.convert_expr(expr, ir, current_stage),
            Stmt::Let { name, value, .. } => {
                // let bindings become ENV or ARG depending on context
                let val = self.expr_to_string(value)?;
                if let Some(stage) = current_stage {
                    stage.add_instruction(DockerInstruction::Env {
                        key: name.to_uppercase(),
                        value: val,
                    });
                }
                Ok(())
            }
            _ => Ok(()), // Ignore other statements
        }
    }

    fn convert_expr(
        &self,
        expr: &Expr,
        ir: &mut DockerfileIR,
        current_stage: &mut Option<DockerStage>,
    ) -> Result<()> {
        match expr {
            Expr::FunctionCall { name, args } => {
                self.convert_function_call(name, args, ir, current_stage)
            }
            _ => Ok(()),
        }
    }

    fn convert_function_call(
        &self,
        name: &str,
        args: &[Expr],
        ir: &mut DockerfileIR,
        current_stage: &mut Option<DockerStage>,
    ) -> Result<()> {
        match name {
            "from_image" => self.convert_from_image(args, ir, current_stage),
            "from_image_as" => self.convert_from_image_as(args, ir, current_stage),
            "copy" => self.convert_copy(args, current_stage),
            "copy_from" => self.convert_copy_from(args, current_stage),
            _ => self.convert_simple_instruction(name, args, current_stage),
        }
    }

    fn convert_from_image(
        &self,
        args: &[Expr],
        ir: &mut DockerfileIR,
        current_stage: &mut Option<DockerStage>,
    ) -> Result<()> {
        if args.is_empty() {
            return Err(Error::Validation(
                "from_image() requires at least 1 argument".to_string(),
            ));
        }

        let (image, tag) = if args.len() == 1 {
            let combined = self.expr_to_string(args.first().expect("verified len >= 1"))?;
            split_image_tag(&combined)
        } else {
            let image = self.expr_to_string(args.first().expect("verified len >= 2"))?;
            let tag = self.expr_to_string(args.get(1).expect("verified len >= 2"))?;
            (image, tag)
        };

        if let Some(stage) = current_stage.take() {
            ir.add_stage(stage);
        }
        *current_stage = Some(DockerStage::new(&image, &tag));
        Ok(())
    }

    fn convert_from_image_as(
        &self,
        args: &[Expr],
        ir: &mut DockerfileIR,
        current_stage: &mut Option<DockerStage>,
    ) -> Result<()> {
        if args.len() < 2 {
            return Err(Error::Validation(
                "from_image_as() requires at least 2 arguments".to_string(),
            ));
        }

        let (image, tag, alias) = if args.len() == 2 {
            let combined = self.expr_to_string(args.first().expect("verified len >= 2"))?;
            let alias = self.expr_to_string(args.get(1).expect("verified len >= 2"))?;
            let (img, tg) = split_image_tag(&combined);
            (img, tg, alias)
        } else {
            let image = self.expr_to_string(args.first().expect("verified len >= 3"))?;
            let tag = self.expr_to_string(args.get(1).expect("verified len >= 3"))?;
            let alias = self.expr_to_string(args.get(2).expect("verified len >= 3"))?;
            (image, tag, alias)
        };

        if let Some(stage) = current_stage.take() {
            ir.add_stage(stage);
        }
        *current_stage = Some(DockerStage::new_named(&image, &tag, &alias));
        Ok(())
    }

    fn convert_copy(&self, args: &[Expr], current_stage: &mut Option<DockerStage>) -> Result<()> {
        if args.len() < 2 {
            return Err(Error::Validation(
                "copy() requires at least 2 arguments: src, dst".to_string(),
            ));
        }
        if args.len() == 3 {
            let src1 = self.expr_to_string(args.first().expect("verified len >= 3"))?;
            let src2 = self.expr_to_string(args.get(1).expect("verified len >= 3"))?;
            let dst = self.expr_to_string(args.get(2).expect("verified len >= 3"))?;
            if let Some(stage) = current_stage {
                stage.add_instruction(DockerInstruction::Copy {
                    src: format!("{src1} {src2}"),
                    dst,
                    from: None,
                });
            }
        } else {
            let src = self.expr_to_string(args.first().expect("verified len >= 2"))?;
            let dst = self.expr_to_string(args.get(1).expect("verified len >= 2"))?;
            if let Some(stage) = current_stage {
                stage.add_instruction(DockerInstruction::Copy {
                    src,
                    dst,
                    from: None,
                });
            }
        }
        Ok(())
    }

    fn convert_copy_from(
        &self,
        args: &[Expr],
        current_stage: &mut Option<DockerStage>,
    ) -> Result<()> {
        if args.len() < 3 {
            return Err(Error::Validation(
                "copy_from() requires 3 arguments: stage, src, dst".to_string(),
            ));
        }
        let from_stage = self.expr_to_string(args.first().expect("verified len >= 3"))?;
        let src = self.expr_to_string(args.get(1).expect("verified len >= 3"))?;
        let dst = self.expr_to_string(args.get(2).expect("verified len >= 3"))?;
        if let Some(stage) = current_stage {
            stage.add_instruction(DockerInstruction::Copy {
                src,
                dst,
                from: Some(from_stage),
            });
        }
        Ok(())
    }

    fn convert_simple_instruction(
        &self,
        name: &str,
        args: &[Expr],
        current_stage: &mut Option<DockerStage>,
    ) -> Result<()> {
        let instruction = match name {
            "run" => Some(self.build_run(args)?),
            "workdir" => self.build_single_string_instruction(args, DockerInstruction::Workdir),
            "env" | "env_set" => Some(self.build_env(args)?),
            "expose" => self.build_expose(args)?,
            "user" => self.build_single_string_instruction(args, DockerInstruction::User),
            "entrypoint" => Some(self.build_string_list(args, DockerInstruction::Entrypoint)?),
            "cmd" => Some(self.build_string_list(args, DockerInstruction::Cmd)?),
            "label" => Some(self.build_label(args)?),
            "healthcheck" => self.build_healthcheck(args)?,
            "comment" => self.build_single_string_instruction(args, DockerInstruction::Comment),
            _ => None,
        };
        if let (Some(inst), Some(stage)) = (instruction, current_stage.as_mut()) {
            stage.add_instruction(inst);
        }
        Ok(())
    }

    fn build_run(&self, args: &[Expr]) -> Result<DockerInstruction> {
        let cmds = self.extract_string_args(args)?;
        Ok(DockerInstruction::Run(cmds))
    }

    fn build_single_string_instruction(
        &self,
        args: &[Expr],
        constructor: fn(String) -> DockerInstruction,
    ) -> Option<DockerInstruction> {
        args.first()
            .and_then(|first| self.expr_to_string(first).ok())
            .map(constructor)
    }

    fn build_env(&self, args: &[Expr]) -> Result<DockerInstruction> {
        if args.len() < 2 {
            return Err(Error::Validation(
                "env()/env_set() requires 2 arguments: key, value".to_string(),
            ));
        }
        let key = self.expr_to_string(args.first().expect("verified len >= 2"))?;
        let value = self.expr_to_string(args.get(1).expect("verified len >= 2"))?;
        Ok(DockerInstruction::Env { key, value })
    }

    fn build_expose(&self, args: &[Expr]) -> Result<Option<DockerInstruction>> {
        let Some(first) = args.first() else {
            return Ok(None);
        };
        let port = match first {
            Expr::Literal(Literal::U16(n)) => *n,
            Expr::Literal(Literal::U32(n)) => *n as u16,
            Expr::Literal(Literal::I32(n)) => *n as u16,
            _ => {
                return Err(Error::Validation(
                    "expose() requires an integer port number".to_string(),
                ))
            }
        };
        Ok(Some(DockerInstruction::Expose(port)))
    }

    fn build_string_list(
        &self,
        args: &[Expr],
        constructor: fn(Vec<String>) -> DockerInstruction,
    ) -> Result<DockerInstruction> {
        let entries = self.extract_string_args(args)?;
        Ok(constructor(entries))
    }

    fn build_label(&self, args: &[Expr]) -> Result<DockerInstruction> {
        if args.len() < 2 {
            return Err(Error::Validation(
                "label() requires 2 arguments: key, value".to_string(),
            ));
        }
        let key = self.expr_to_string(args.first().expect("verified len >= 2"))?;
        let value = self.expr_to_string(args.get(1).expect("verified len >= 2"))?;
        Ok(DockerInstruction::Label { key, value })
    }

    fn build_healthcheck(&self, args: &[Expr]) -> Result<Option<DockerInstruction>> {
        let Some(first) = args.first() else {
            return Ok(None);
        };
        let cmd = self.expr_to_string(first)?;
        Ok(Some(DockerInstruction::Healthcheck {
            cmd,
            interval: None,
            timeout: None,
        }))
    }

    fn extract_string_args(&self, args: &[Expr]) -> Result<Vec<String>> {
        let mut result = Vec::new();
        for arg in args {
            match arg {
                Expr::Array(items) => {
                    for item in items {
                        result.push(self.expr_to_string(item)?);
                    }
                }
                _ => {
                    result.push(self.expr_to_string(arg)?);
                }
            }
        }
        Ok(result)
    }

    fn expr_to_string(&self, expr: &Expr) -> Result<String> {
        match expr {
            Expr::Literal(Literal::Str(s)) => Ok(s.clone()),
            Expr::Literal(Literal::U16(n)) => Ok(n.to_string()),
            Expr::Literal(Literal::U32(n)) => Ok(n.to_string()),
            Expr::Literal(Literal::I32(n)) => Ok(n.to_string()),
            Expr::Literal(Literal::Bool(b)) => Ok(b.to_string()),
            Expr::Variable(name) => Ok(format!("${{{}}}", name.to_uppercase())),
            _ => Err(Error::Validation(
                "Cannot convert expression to Dockerfile value".to_string(),
            )),
        }
    }
}

#[cfg(test)]
#[path = "dockerfile_tests.rs"]
mod tests;
