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

        // Find the entry point function
        let entry_fn = ast
            .functions
            .iter()
            .find(|f| f.name == ast.entry_point)
            .ok_or_else(|| Error::IrGeneration("Entry point not found".to_string()))?;

        // Convert each statement
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
            Stmt::Let { name, value } => {
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
            "from_image" => {
                // from_image("image", "tag") or from_image("image", "tag")
                if args.len() < 2 {
                    return Err(Error::Validation(
                        "from_image() requires 2 arguments: image, tag".to_string(),
                    ));
                }
                let image = self.expr_to_string(args.first().expect("verified len >= 2"))?;
                let tag = self.expr_to_string(args.get(1).expect("verified len >= 2"))?;

                // Push current stage if exists
                if let Some(stage) = current_stage.take() {
                    ir.add_stage(stage);
                }

                *current_stage = Some(DockerStage::new(&image, &tag));
                Ok(())
            }
            "from_image_as" => {
                // from_image_as("image", "tag", "alias")
                if args.len() < 3 {
                    return Err(Error::Validation(
                        "from_image_as() requires 3 arguments: image, tag, alias".to_string(),
                    ));
                }
                let image = self.expr_to_string(args.first().expect("verified len >= 3"))?;
                let tag = self.expr_to_string(args.get(1).expect("verified len >= 3"))?;
                let alias = self.expr_to_string(args.get(2).expect("verified len >= 3"))?;

                if let Some(stage) = current_stage.take() {
                    ir.add_stage(stage);
                }

                *current_stage = Some(DockerStage::new_named(&image, &tag, &alias));
                Ok(())
            }
            "run" => {
                let cmds = self.extract_string_args(args)?;
                if let Some(stage) = current_stage {
                    stage.add_instruction(DockerInstruction::Run(cmds));
                }
                Ok(())
            }
            "copy" => {
                if args.len() < 2 {
                    return Err(Error::Validation(
                        "copy() requires 2 arguments: src, dst".to_string(),
                    ));
                }
                let src = self.expr_to_string(args.first().expect("verified len >= 2"))?;
                let dst = self.expr_to_string(args.get(1).expect("verified len >= 2"))?;
                if let Some(stage) = current_stage {
                    stage.add_instruction(DockerInstruction::Copy {
                        src,
                        dst,
                        from: None,
                    });
                }
                Ok(())
            }
            "copy_from" => {
                if args.len() < 3 {
                    return Err(Error::Validation(
                        "copy_from() requires 3 arguments: stage, src, dst".to_string(),
                    ));
                }
                let from_stage =
                    self.expr_to_string(args.first().expect("verified len >= 3"))?;
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
            "workdir" => {
                if let Some(first) = args.first() {
                    let path = self.expr_to_string(first)?;
                    if let Some(stage) = current_stage {
                        stage.add_instruction(DockerInstruction::Workdir(path));
                    }
                }
                Ok(())
            }
            "env" => {
                if args.len() < 2 {
                    return Err(Error::Validation(
                        "env() requires 2 arguments: key, value".to_string(),
                    ));
                }
                let key = self.expr_to_string(args.first().expect("verified len >= 2"))?;
                let value = self.expr_to_string(args.get(1).expect("verified len >= 2"))?;
                if let Some(stage) = current_stage {
                    stage.add_instruction(DockerInstruction::Env { key, value });
                }
                Ok(())
            }
            "expose" => {
                if let Some(first) = args.first() {
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
                    if let Some(stage) = current_stage {
                        stage.add_instruction(DockerInstruction::Expose(port));
                    }
                }
                Ok(())
            }
            "user" => {
                if let Some(first) = args.first() {
                    let user = self.expr_to_string(first)?;
                    if let Some(stage) = current_stage {
                        stage.add_instruction(DockerInstruction::User(user));
                    }
                }
                Ok(())
            }
            "entrypoint" => {
                let entries = self.extract_string_args(args)?;
                if let Some(stage) = current_stage {
                    stage.add_instruction(DockerInstruction::Entrypoint(entries));
                }
                Ok(())
            }
            "cmd" => {
                let entries = self.extract_string_args(args)?;
                if let Some(stage) = current_stage {
                    stage.add_instruction(DockerInstruction::Cmd(entries));
                }
                Ok(())
            }
            "label" => {
                if args.len() < 2 {
                    return Err(Error::Validation(
                        "label() requires 2 arguments: key, value".to_string(),
                    ));
                }
                let key = self.expr_to_string(args.first().expect("verified len >= 2"))?;
                let value = self.expr_to_string(args.get(1).expect("verified len >= 2"))?;
                if let Some(stage) = current_stage {
                    stage.add_instruction(DockerInstruction::Label { key, value });
                }
                Ok(())
            }
            "healthcheck" => {
                if let Some(first) = args.first() {
                    let cmd = self.expr_to_string(first)?;
                    if let Some(stage) = current_stage {
                        stage.add_instruction(DockerInstruction::Healthcheck {
                            cmd,
                            interval: None,
                            timeout: None,
                        });
                    }
                }
                Ok(())
            }
            "comment" => {
                if let Some(first) = args.first() {
                    let text = self.expr_to_string(first)?;
                    if let Some(stage) = current_stage {
                        stage.add_instruction(DockerInstruction::Comment(text));
                    }
                }
                Ok(())
            }
            _ => Ok(()), // Ignore unknown function calls
        }
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
    fn test_DOCKER_BUILD_001_basic_generation() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("rust".to_string())),
                    Expr::Literal(Literal::Str("1.75-alpine".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "workdir".to_string(),
                args: vec![Expr::Literal(Literal::Str("/app".to_string()))],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "copy".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str(".".to_string())),
                    Expr::Literal(Literal::Str(".".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "user".to_string(),
                args: vec![Expr::Literal(Literal::Str("65534".to_string()))],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(
            result.contains("FROM rust:1.75-alpine"),
            "Expected FROM in: {}",
            result
        );
        assert!(result.contains("WORKDIR /app"));
        assert!(result.contains("COPY . ."));
        assert!(result.contains("USER 65534"));
    }

    #[test]
    fn test_DOCKER_BUILD_002_multi_stage() {
        let ast = make_simple_ast(vec![
            // Builder stage
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image_as".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("rust".to_string())),
                    Expr::Literal(Literal::Str("1.75-alpine".to_string())),
                    Expr::Literal(Literal::Str("builder".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "workdir".to_string(),
                args: vec![Expr::Literal(Literal::Str("/app".to_string()))],
            }),
            // Runtime stage
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "copy_from".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("builder".to_string())),
                    Expr::Literal(Literal::Str("/app/bin".to_string())),
                    Expr::Literal(Literal::Str("/usr/local/bin/".to_string())),
                ],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("FROM rust:1.75-alpine AS builder"));
        assert!(result.contains("FROM alpine:3.18"));
        assert!(result.contains("COPY --from=builder /app/bin /usr/local/bin/"));
    }

    #[test]
    fn test_DOCKER_BUILD_003_run_chaining() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("ubuntu".to_string())),
                    Expr::Literal(Literal::Str("22.04".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "run".to_string(),
                args: vec![Expr::Array(vec![
                    Expr::Literal(Literal::Str("apt-get update".to_string())),
                    Expr::Literal(Literal::Str("apt-get install -y curl".to_string())),
                    Expr::Literal(Literal::Str("rm -rf /var/lib/apt/lists/*".to_string())),
                ])],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("RUN apt-get update && \\\n"));
    }

    #[test]
    fn test_DOCKER_BUILD_004_entrypoint_exec_form() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "entrypoint".to_string(),
                args: vec![Expr::Array(vec![Expr::Literal(Literal::Str(
                    "/app".to_string(),
                ))])],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("ENTRYPOINT [\"/app\"]"));
    }

    #[test]
    fn test_DOCKER_BUILD_005_user_directive() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "user".to_string(),
                args: vec![Expr::Literal(Literal::Str("65534".to_string()))],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(
            result.contains("USER 65534"),
            "USER directive must be present for DOCKER003 compliance"
        );
    }

    #[test]
    fn test_DOCKER_BUILD_006_no_latest_tag() {
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("rust".to_string())),
                Expr::Literal(Literal::Str("1.75-alpine".to_string())),
            ],
        })]);

        let result = emit_dockerfile(&ast).unwrap();
        // Should use pinned version, not "latest"
        assert!(
            !result.contains(":latest"),
            "Generated Dockerfile should use pinned versions (DOCKER002 compliance)"
        );
        assert!(result.contains("FROM rust:1.75-alpine"));
    }
}
