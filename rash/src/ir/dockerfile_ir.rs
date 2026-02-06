//! Dockerfile Intermediate Representation
//!
//! Defines the IR types for representing Dockerfile instructions.
//! Used by the Dockerfile emitter to generate valid Dockerfiles from Rust DSL.

use serde::{Deserialize, Serialize};

/// A complete Dockerfile IR with one or more stages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerfileIR {
    /// Build stages (multi-stage builds have multiple stages)
    pub stages: Vec<DockerStage>,
}

impl DockerfileIR {
    /// Create a new empty DockerfileIR
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    /// Add a stage to the Dockerfile
    pub fn add_stage(&mut self, stage: DockerStage) {
        self.stages.push(stage);
    }

    /// Emit the Dockerfile as a string
    pub fn emit(&self) -> String {
        let mut output = String::new();

        for (i, stage) in self.stages.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }
            stage.emit(&mut output);
        }

        output
    }
}

impl Default for DockerfileIR {
    fn default() -> Self {
        Self::new()
    }
}

/// A single stage in a Dockerfile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerStage {
    /// FROM instruction (base image)
    pub from: FromInstruction,
    /// Instructions in this stage
    pub instructions: Vec<DockerInstruction>,
}

impl DockerStage {
    /// Create a new stage with a base image
    pub fn new(image: &str, tag: &str) -> Self {
        Self {
            from: FromInstruction {
                image: image.to_string(),
                tag: tag.to_string(),
                alias: None,
            },
            instructions: Vec::new(),
        }
    }

    /// Create a named stage (for multi-stage builds)
    pub fn new_named(image: &str, tag: &str, alias: &str) -> Self {
        Self {
            from: FromInstruction {
                image: image.to_string(),
                tag: tag.to_string(),
                alias: Some(alias.to_string()),
            },
            instructions: Vec::new(),
        }
    }

    /// Add an instruction to this stage
    pub fn add_instruction(&mut self, instruction: DockerInstruction) {
        self.instructions.push(instruction);
    }

    fn emit(&self, output: &mut String) {
        // FROM line
        output.push_str(&format!("FROM {}:{}", self.from.image, self.from.tag));
        if let Some(alias) = &self.from.alias {
            output.push_str(&format!(" AS {}", alias));
        }
        output.push('\n');

        // Instructions
        for instruction in &self.instructions {
            instruction.emit(output);
        }
    }
}

/// FROM instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FromInstruction {
    /// Image name (e.g., "rust", "alpine")
    pub image: String,
    /// Image tag (e.g., "1.75-alpine", "3.18")
    pub tag: String,
    /// Optional stage alias (e.g., "builder")
    pub alias: Option<String>,
}

/// Docker instruction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DockerInstruction {
    /// RUN command(s)
    Run(Vec<String>),
    /// COPY source destination
    Copy {
        src: String,
        dst: String,
        from: Option<String>,
    },
    /// WORKDIR path
    Workdir(String),
    /// ENV key=value
    Env { key: String, value: String },
    /// ARG name[=default]
    Arg {
        name: String,
        default: Option<String>,
    },
    /// EXPOSE port
    Expose(u16),
    /// USER user
    User(String),
    /// ENTRYPOINT [exec form]
    Entrypoint(Vec<String>),
    /// CMD [exec form]
    Cmd(Vec<String>),
    /// LABEL key=value
    Label { key: String, value: String },
    /// HEALTHCHECK
    Healthcheck {
        cmd: String,
        interval: Option<String>,
        timeout: Option<String>,
    },
    /// Comment
    Comment(String),
}

impl DockerInstruction {
    fn emit(&self, output: &mut String) {
        match self {
            DockerInstruction::Run(cmds) => {
                if cmds.len() == 1 {
                    output.push_str(&format!("RUN {}\n", cmds[0]));
                } else {
                    // Chain with &&
                    output.push_str("RUN ");
                    for (i, cmd) in cmds.iter().enumerate() {
                        if i > 0 {
                            output.push_str(" && \\\n    ");
                        }
                        output.push_str(cmd);
                    }
                    output.push('\n');
                }
            }
            DockerInstruction::Copy { src, dst, from } => {
                if let Some(stage) = from {
                    output.push_str(&format!("COPY --from={} {} {}\n", stage, src, dst));
                } else {
                    output.push_str(&format!("COPY {} {}\n", src, dst));
                }
            }
            DockerInstruction::Workdir(path) => {
                output.push_str(&format!("WORKDIR {}\n", path));
            }
            DockerInstruction::Env { key, value } => {
                output.push_str(&format!("ENV {}={}\n", key, value));
            }
            DockerInstruction::Arg { name, default } => {
                if let Some(def) = default {
                    output.push_str(&format!("ARG {}={}\n", name, def));
                } else {
                    output.push_str(&format!("ARG {}\n", name));
                }
            }
            DockerInstruction::Expose(port) => {
                output.push_str(&format!("EXPOSE {}\n", port));
            }
            DockerInstruction::User(user) => {
                output.push_str(&format!("USER {}\n", user));
            }
            DockerInstruction::Entrypoint(args) => {
                let json_args: Vec<String> = args.iter().map(|a| format!("\"{}\"", a)).collect();
                output.push_str(&format!("ENTRYPOINT [{}]\n", json_args.join(", ")));
            }
            DockerInstruction::Cmd(args) => {
                let json_args: Vec<String> = args.iter().map(|a| format!("\"{}\"", a)).collect();
                output.push_str(&format!("CMD [{}]\n", json_args.join(", ")));
            }
            DockerInstruction::Label { key, value } => {
                output.push_str(&format!("LABEL {}=\"{}\"\n", key, value));
            }
            DockerInstruction::Healthcheck {
                cmd,
                interval,
                timeout,
            } => {
                output.push_str("HEALTHCHECK");
                if let Some(iv) = interval {
                    output.push_str(&format!(" --interval={}", iv));
                }
                if let Some(to) = timeout {
                    output.push_str(&format!(" --timeout={}", to));
                }
                output.push_str(&format!(" CMD {}\n", cmd));
            }
            DockerInstruction::Comment(text) => {
                output.push_str(&format!("# {}\n", text));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_simple_dockerfile_ir() {
        let mut ir = DockerfileIR::new();
        let mut stage = DockerStage::new("rust", "1.75-alpine");
        stage.add_instruction(DockerInstruction::Workdir("/app".to_string()));
        stage.add_instruction(DockerInstruction::Copy {
            src: ".".to_string(),
            dst: ".".to_string(),
            from: None,
        });
        stage.add_instruction(DockerInstruction::Run(vec![
            "cargo build --release".to_string()
        ]));
        stage.add_instruction(DockerInstruction::User("65534".to_string()));
        ir.add_stage(stage);

        let result = ir.emit();
        assert!(result.contains("FROM rust:1.75-alpine"));
        assert!(result.contains("WORKDIR /app"));
        assert!(result.contains("COPY . ."));
        assert!(result.contains("RUN cargo build --release"));
        assert!(result.contains("USER 65534"));
    }

    #[test]
    fn test_multi_stage_dockerfile_ir() {
        let mut ir = DockerfileIR::new();

        // Builder stage
        let mut builder = DockerStage::new_named("rust", "1.75-alpine", "builder");
        builder.add_instruction(DockerInstruction::Workdir("/app".to_string()));
        builder.add_instruction(DockerInstruction::Run(vec![
            "cargo build --release".to_string()
        ]));
        ir.add_stage(builder);

        // Runtime stage
        let mut runtime = DockerStage::new("alpine", "3.18");
        runtime.add_instruction(DockerInstruction::Copy {
            src: "/app/target/release/myapp".to_string(),
            dst: "/usr/local/bin/myapp".to_string(),
            from: Some("builder".to_string()),
        });
        runtime.add_instruction(DockerInstruction::User("65534".to_string()));
        runtime.add_instruction(DockerInstruction::Entrypoint(vec![
            "/usr/local/bin/myapp".to_string()
        ]));
        ir.add_stage(runtime);

        let result = ir.emit();
        assert!(result.contains("FROM rust:1.75-alpine AS builder"));
        assert!(result.contains("FROM alpine:3.18"));
        assert!(result.contains("COPY --from=builder"));
    }

    #[test]
    fn test_run_chaining() {
        let instruction = DockerInstruction::Run(vec![
            "apt-get update".to_string(),
            "apt-get install -y curl".to_string(),
            "rm -rf /var/lib/apt/lists/*".to_string(),
        ]);

        let mut output = String::new();
        instruction.emit(&mut output);
        assert!(output.contains("apt-get update && \\\n"));
        assert!(output.contains("apt-get install -y curl && \\\n"));
    }

    #[test]
    fn test_entrypoint_exec_form() {
        let instruction = DockerInstruction::Entrypoint(vec!["/app".to_string()]);
        let mut output = String::new();
        instruction.emit(&mut output);
        assert_eq!(output, "ENTRYPOINT [\"/app\"]\n");
    }
}
