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
        assert!(
            !result.contains(":latest"),
            "Generated Dockerfile should use pinned versions (DOCKER002 compliance)"
        );
        assert!(result.contains("FROM rust:1.75-alpine"));
    }

    #[test]
    fn test_DOCKER_BUILD_007_env_directive() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("node".to_string())),
                    Expr::Literal(Literal::Str("20-alpine".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "env".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("NODE_ENV".to_string())),
                    Expr::Literal(Literal::Str("production".to_string())),
                ],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(
            result.contains("ENV NODE_ENV=production"),
            "ENV directive in: {result}"
        );
    }

    #[test]
    fn test_DOCKER_BUILD_008_expose_u16() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("nginx".to_string())),
                    Expr::Literal(Literal::Str("alpine".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "expose".to_string(),
                args: vec![Expr::Literal(Literal::U16(8080))],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("EXPOSE 8080"), "EXPOSE in: {result}");
    }

    #[test]
    fn test_DOCKER_BUILD_009_expose_u32() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("nginx".to_string())),
                    Expr::Literal(Literal::Str("alpine".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "expose".to_string(),
                args: vec![Expr::Literal(Literal::U32(3000))],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("EXPOSE 3000"), "EXPOSE U32 in: {result}");
    }

    #[test]
    fn test_DOCKER_BUILD_010_expose_i32() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("nginx".to_string())),
                    Expr::Literal(Literal::Str("alpine".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "expose".to_string(),
                args: vec![Expr::Literal(Literal::I32(443))],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("EXPOSE 443"), "EXPOSE I32 in: {result}");
    }

    #[test]
    fn test_DOCKER_BUILD_011_cmd_directive() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("node".to_string())),
                    Expr::Literal(Literal::Str("20".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "cmd".to_string(),
                args: vec![Expr::Array(vec![
                    Expr::Literal(Literal::Str("node".to_string())),
                    Expr::Literal(Literal::Str("server.js".to_string())),
                ])],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("CMD"), "CMD in: {result}");
    }

    #[test]
    fn test_DOCKER_BUILD_012_label_directive() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "label".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("maintainer".to_string())),
                    Expr::Literal(Literal::Str("team@example.com".to_string())),
                ],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("LABEL"), "LABEL in: {result}");
    }

    #[test]
    fn test_DOCKER_BUILD_013_healthcheck() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("nginx".to_string())),
                    Expr::Literal(Literal::Str("alpine".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "healthcheck".to_string(),
                args: vec![Expr::Literal(Literal::Str(
                    "curl -f http://localhost/".to_string(),
                ))],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("HEALTHCHECK"), "HEALTHCHECK in: {result}");
    }

    #[test]
    fn test_DOCKER_BUILD_014_comment() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "comment".to_string(),
                args: vec![Expr::Literal(Literal::Str(
                    "Install dependencies".to_string(),
                ))],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(
            result.contains("Install dependencies"),
            "Comment in: {result}"
        );
    }

    #[test]
    fn test_DOCKER_BUILD_015_let_binding_becomes_env() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("python".to_string())),
                    Expr::Literal(Literal::Str("3.12".to_string())),
                ],
            }),
            Stmt::Let {
                name: "app_port".to_string(),
                value: Expr::Literal(Literal::Str("8080".to_string())),
            },
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(
            result.contains("ENV APP_PORT=8080"),
            "Let→ENV in: {result}"
        );
    }

    #[test]
    fn test_DOCKER_BUILD_016_no_from_image_error() {
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "workdir".to_string(),
            args: vec![Expr::Literal(Literal::Str("/app".to_string()))],
        })]);

        let err = emit_dockerfile(&ast).unwrap_err();
        assert!(
            format!("{err}").contains("from_image"),
            "Error should mention from_image: {err}"
        );
    }

    #[test]
    fn test_DOCKER_BUILD_017_from_image_too_few_args() {
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![Expr::Literal(Literal::Str("alpine".to_string()))],
        })]);

        let err = emit_dockerfile(&ast).unwrap_err();
        assert!(
            format!("{err}").contains("2 arguments"),
            "Error: {err}"
        );
    }

    #[test]
    fn test_DOCKER_BUILD_018_from_image_as_too_few_args() {
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "from_image_as".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("rust".to_string())),
                Expr::Literal(Literal::Str("1.75".to_string())),
            ],
        })]);

        let err = emit_dockerfile(&ast).unwrap_err();
        assert!(
            format!("{err}").contains("3 arguments"),
            "Error: {err}"
        );
    }

    #[test]
    fn test_DOCKER_BUILD_019_copy_too_few_args() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "copy".to_string(),
                args: vec![Expr::Literal(Literal::Str(".".to_string()))],
            }),
        ]);

        let err = emit_dockerfile(&ast).unwrap_err();
        assert!(format!("{err}").contains("2 arguments"), "Error: {err}");
    }

    #[test]
    fn test_DOCKER_BUILD_020_copy_from_too_few_args() {
        let ast = make_simple_ast(vec![
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
                    Expr::Literal(Literal::Str("/app".to_string())),
                ],
            }),
        ]);

        let err = emit_dockerfile(&ast).unwrap_err();
        assert!(format!("{err}").contains("3 arguments"), "Error: {err}");
    }

    #[test]
    fn test_DOCKER_BUILD_021_env_too_few_args() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "env".to_string(),
                args: vec![Expr::Literal(Literal::Str("KEY".to_string()))],
            }),
        ]);

        let err = emit_dockerfile(&ast).unwrap_err();
        assert!(format!("{err}").contains("2 arguments"), "Error: {err}");
    }

    #[test]
    fn test_DOCKER_BUILD_022_expose_invalid_type() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "expose".to_string(),
                args: vec![Expr::Literal(Literal::Str("not-a-port".to_string()))],
            }),
        ]);

        let err = emit_dockerfile(&ast).unwrap_err();
        assert!(
            format!("{err}").contains("integer"),
            "Error: {err}"
        );
    }

    #[test]
    fn test_DOCKER_BUILD_023_label_too_few_args() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "label".to_string(),
                args: vec![Expr::Literal(Literal::Str("key".to_string()))],
            }),
        ]);

        let err = emit_dockerfile(&ast).unwrap_err();
        assert!(format!("{err}").contains("2 arguments"), "Error: {err}");
    }

    #[test]
    fn test_DOCKER_BUILD_024_variable_in_expr() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "env".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("PATH".to_string())),
                    Expr::Variable("app_dir".to_string()),
                ],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(
            result.contains("${APP_DIR}"),
            "Variable ref in: {result}"
        );
    }

    // --- Coverage tests for uncovered lines ---

    #[test]
    fn test_DOCKER_COV_001_convert_stmt_return_catchall() {
        // Line 86: _ => Ok(()) in convert_stmt — Stmt::Return is not Expr or Let
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Return(None),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("FROM alpine:3.18"));
    }

    #[test]
    fn test_DOCKER_COV_002_convert_expr_non_functioncall() {
        // Line 100: _ => Ok(()) in convert_expr — Expr::Variable is not FunctionCall
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::Variable("ignored".to_string())),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("FROM alpine:3.18"));
    }

    #[test]
    fn test_DOCKER_COV_003_multi_stage_from_image_as_twice() {
        // Line 142: ir.add_stage(stage) in from_image_as when existing stage
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image_as".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("rust".to_string())),
                    Expr::Literal(Literal::Str("1.75".to_string())),
                    Expr::Literal(Literal::Str("builder".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "run".to_string(),
                args: vec![Expr::Literal(Literal::Str("cargo build".to_string()))],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image_as".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                    Expr::Literal(Literal::Str("runtime".to_string())),
                ],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("FROM rust:1.75 AS builder"));
        assert!(result.contains("FROM alpine:3.18 AS runtime"));
    }

    #[test]
    fn test_DOCKER_COV_004_unknown_function_call() {
        // Line 288: _ => Ok(()) in convert_function_call — unknown function name
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "unknown_instruction".to_string(),
                args: vec![Expr::Literal(Literal::Str("arg".to_string()))],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("FROM alpine:3.18"));
    }

    #[test]
    fn test_DOCKER_COV_005_extract_string_args_non_array() {
        // Line 302: _ => in extract_string_args — non-Array arg
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "run".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("apt-get update".to_string())),
                    Expr::Literal(Literal::Str("apt-get install -y curl".to_string())),
                ],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("RUN"), "RUN in: {result}");
    }

    #[test]
    fn test_DOCKER_COV_006_expr_to_string_u16() {
        // Line 312: Literal::U16 in expr_to_string
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "label".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("port".to_string())),
                    Expr::Literal(Literal::U16(8080)),
                ],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("8080"), "U16 label in: {result}");
    }

    #[test]
    fn test_DOCKER_COV_007_expr_to_string_u32() {
        // Line 313: Literal::U32 in expr_to_string
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "label".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("size".to_string())),
                    Expr::Literal(Literal::U32(65536)),
                ],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("65536"), "U32 label in: {result}");
    }

    #[test]
    fn test_DOCKER_COV_008_expr_to_string_i32() {
        // Line 314: Literal::I32 in expr_to_string
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "label".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("count".to_string())),
                    Expr::Literal(Literal::I32(-1)),
                ],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("-1"), "I32 label in: {result}");
    }

    #[test]
    fn test_DOCKER_COV_009_expr_to_string_bool() {
        // Line 315: Literal::Bool in expr_to_string
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "label".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("debug".to_string())),
                    Expr::Literal(Literal::Bool(true)),
                ],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("true"), "Bool label in: {result}");
    }

    #[test]
    fn test_DOCKER_COV_010_expr_to_string_error() {
        // Lines 317-319: catch-all _ => Err(...) in expr_to_string
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "label".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("key".to_string())),
                    Expr::Array(vec![Expr::Literal(Literal::Str("a".to_string()))]),
                ],
            }),
        ]);

        let err = emit_dockerfile(&ast).unwrap_err();
        assert!(
            format!("{err}").contains("Cannot convert expression"),
            "Error: {err}"
        );
    }

    #[test]
    fn test_DOCKER_COV_011_instructions_before_from_image() {
        // Lines 196, 227, 236, 276, 285: instructions when current_stage is None
        // workdir, expose, user, healthcheck, comment all silently skip
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "workdir".to_string(),
                args: vec![Expr::Literal(Literal::Str("/app".to_string()))],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "expose".to_string(),
                args: vec![Expr::Literal(Literal::U16(8080))],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "user".to_string(),
                args: vec![Expr::Literal(Literal::Str("root".to_string()))],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "healthcheck".to_string(),
                args: vec![Expr::Literal(Literal::Str("curl localhost".to_string()))],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "comment".to_string(),
                args: vec![Expr::Literal(Literal::Str("test".to_string()))],
            }),
            // Now add from_image so we don't get "no stages" error
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("FROM alpine:3.18"));
        // Instructions before from_image had no stage → silently skipped
        assert!(!result.contains("WORKDIR"), "No WORKDIR before FROM");
    }

    #[test]
    fn test_DOCKER_COV_012_instructions_with_empty_args() {
        // Lines 196, 227, 236, 276, 285: implicit else when args is empty
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "workdir".to_string(),
                args: vec![],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "expose".to_string(),
                args: vec![],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "user".to_string(),
                args: vec![],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "healthcheck".to_string(),
                args: vec![],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "comment".to_string(),
                args: vec![],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("FROM alpine:3.18"));
        // Empty args means no instructions generated
        assert!(!result.contains("WORKDIR"), "No WORKDIR with empty args");
        assert!(!result.contains("EXPOSE"), "No EXPOSE with empty args");
        assert!(!result.contains("USER"), "No USER with empty args");
        assert!(!result.contains("HEALTHCHECK"), "No HEALTHCHECK with empty args");
    }

    // ============================================================================
    // Coverage Tests - convert_function_call branches (DOCKER_COV_013-025)
    // ============================================================================

    #[test]
    fn test_DOCKER_COV_013_from_image_too_few_args() {
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![Expr::Literal(Literal::Str("alpine".to_string()))],
        })]);
        let err = emit_dockerfile(&ast).unwrap_err();
        assert!(format!("{err}").contains("2 arguments"), "Error: {err}");
    }

    #[test]
    fn test_DOCKER_COV_014_from_image_as_too_few_args() {
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "from_image_as".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("rust".to_string())),
                Expr::Literal(Literal::Str("1.75".to_string())),
            ],
        })]);
        let err = emit_dockerfile(&ast).unwrap_err();
        assert!(format!("{err}").contains("3 arguments"), "Error: {err}");
    }

    #[test]
    fn test_DOCKER_COV_015_copy_too_few_args() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "copy".to_string(),
                args: vec![Expr::Literal(Literal::Str("src".to_string()))],
            }),
        ]);
        let err = emit_dockerfile(&ast).unwrap_err();
        assert!(format!("{err}").contains("2 arguments"), "Error: {err}");
    }

    #[test]
    fn test_DOCKER_COV_016_copy_from_too_few_args() {
        let ast = make_simple_ast(vec![
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
                    Expr::Literal(Literal::Str("/src".to_string())),
                ],
            }),
        ]);
        let err = emit_dockerfile(&ast).unwrap_err();
        assert!(format!("{err}").contains("3 arguments"), "Error: {err}");
    }

    #[test]
    fn test_DOCKER_COV_017_env_too_few_args() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "env".to_string(),
                args: vec![Expr::Literal(Literal::Str("KEY".to_string()))],
            }),
        ]);
        let err = emit_dockerfile(&ast).unwrap_err();
        assert!(format!("{err}").contains("2 arguments"), "Error: {err}");
    }

    #[test]
    fn test_DOCKER_COV_018_expose_u32() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "expose".to_string(),
                args: vec![Expr::Literal(Literal::U32(3000))],
            }),
        ]);
        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("EXPOSE 3000"), "U32 expose in: {result}");
    }

    #[test]
    fn test_DOCKER_COV_019_expose_i32() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "expose".to_string(),
                args: vec![Expr::Literal(Literal::I32(9090))],
            }),
        ]);
        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("EXPOSE 9090"), "I32 expose in: {result}");
    }

    #[test]
    fn test_DOCKER_COV_020_cmd_instruction() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "cmd".to_string(),
                args: vec![Expr::Array(vec![
                    Expr::Literal(Literal::Str("/bin/sh".to_string())),
                    Expr::Literal(Literal::Str("-c".to_string())),
                    Expr::Literal(Literal::Str("echo hello".to_string())),
                ])],
            }),
        ]);
        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("CMD"), "CMD in: {result}");
    }

    #[test]
    fn test_DOCKER_COV_021_no_stage_run_copy_env() {
        // Instructions without a stage: run, copy, copy_from, env, label, entrypoint, cmd
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "run".to_string(),
                args: vec![Expr::Literal(Literal::Str("echo hi".to_string()))],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "copy".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("src".to_string())),
                    Expr::Literal(Literal::Str("dst".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "env".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("K".to_string())),
                    Expr::Literal(Literal::Str("V".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "label".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("k".to_string())),
                    Expr::Literal(Literal::Str("v".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "entrypoint".to_string(),
                args: vec![Expr::Literal(Literal::Str("/bin/sh".to_string()))],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "cmd".to_string(),
                args: vec![Expr::Literal(Literal::Str("echo".to_string()))],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "copy_from".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("builder".to_string())),
                    Expr::Literal(Literal::Str("/a".to_string())),
                    Expr::Literal(Literal::Str("/b".to_string())),
                ],
            }),
            // Add from_image last so we don't error on "no stages"
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
        ]);
        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("FROM alpine:3.18"));
        // Instructions before FROM should have been skipped (no stage)
        assert!(!result.contains("RUN"), "No RUN before FROM");
    }

    #[test]
    fn test_DOCKER_COV_022_from_image_pushes_existing_stage() {
        // Two from_image calls — second one pushes the first stage
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("rust".to_string())),
                    Expr::Literal(Literal::Str("1.75".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "run".to_string(),
                args: vec![Expr::Literal(Literal::Str("cargo build".to_string()))],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
        ]);
        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("FROM rust:1.75"), "First stage in: {result}");
        assert!(
            result.contains("FROM alpine:3.18"),
            "Second stage in: {result}"
        );
    }

    #[test]
    fn test_DOCKER_COV_023_label_success() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "label".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("maintainer".to_string())),
                    Expr::Literal(Literal::Str("test@example.com".to_string())),
                ],
            }),
        ]);
        let result = emit_dockerfile(&ast).unwrap();
        assert!(
            result.contains("LABEL maintainer"),
            "LABEL in: {result}"
        );
    }

    #[test]
    fn test_DOCKER_COV_024_comment_success() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "comment".to_string(),
                args: vec![Expr::Literal(Literal::Str(
                    "This is a comment".to_string(),
                ))],
            }),
        ]);
        let result = emit_dockerfile(&ast).unwrap();
        assert!(
            result.contains("# This is a comment"),
            "Comment in: {result}"
        );
    }

    #[test]
    fn test_DOCKER_COV_025_entrypoint_single_string() {
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
                args: vec![Expr::Literal(Literal::Str("/app/start".to_string()))],
            }),
        ]);
        let result = emit_dockerfile(&ast).unwrap();
        assert!(
            result.contains("ENTRYPOINT"),
            "ENTRYPOINT in: {result}"
        );
    }

    #[test]
    fn test_DOCKER_BUILD_025_comprehensive_dockerfile() {
        let ast = make_simple_ast(vec![
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
            Stmt::Expr(Expr::FunctionCall {
                name: "copy".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("Cargo.toml".to_string())),
                    Expr::Literal(Literal::Str(".".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "run".to_string(),
                args: vec![Expr::Array(vec![
                    Expr::Literal(Literal::Str("cargo build --release".to_string())),
                ])],
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
                    Expr::Literal(Literal::Str("/app/target/release/app".to_string())),
                    Expr::Literal(Literal::Str("/usr/local/bin/".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "expose".to_string(),
                args: vec![Expr::Literal(Literal::U16(8080))],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "user".to_string(),
                args: vec![Expr::Literal(Literal::Str("65534".to_string()))],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "healthcheck".to_string(),
                args: vec![Expr::Literal(Literal::Str(
                    "curl -f http://localhost:8080/health".to_string(),
                ))],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "entrypoint".to_string(),
                args: vec![Expr::Array(vec![Expr::Literal(Literal::Str(
                    "/usr/local/bin/app".to_string(),
                ))])],
            }),
        ]);

        let result = emit_dockerfile(&ast).unwrap();
        assert!(result.contains("FROM rust:1.75-alpine AS builder"));
        assert!(result.contains("FROM alpine:3.18"));
        assert!(result.contains("COPY --from=builder"));
        assert!(result.contains("EXPOSE 8080"));
        assert!(result.contains("USER 65534"));
        assert!(result.contains("HEALTHCHECK"));
        assert!(result.contains("ENTRYPOINT"));
    }
}
