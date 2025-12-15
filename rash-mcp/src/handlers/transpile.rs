use bashrs::{transpile, Config};
use pforge_runtime::{Handler, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct TranspileInput {
    /// Rust source code to transpile
    pub source: String,
    /// Whether to optimize the output (default: false)
    #[serde(default)]
    pub optimize: bool,
    /// Whether to enable strict mode (default: false)
    #[serde(default)]
    pub strict: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct TranspileOutput {
    /// Generated POSIX shell script
    pub shell_script: String,
    /// Any warnings during transpilation
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

pub(crate) struct TranspileHandler;

#[async_trait::async_trait]
impl Handler for TranspileHandler {
    type Input = TranspileInput;
    type Output = TranspileOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        let config = Config {
            optimize: input.optimize,
            strict_mode: input.strict,
            ..Default::default()
        };

        match transpile(&input.source, config) {
            Ok(shell_script) => Ok(TranspileOutput {
                shell_script,
                warnings: vec![],
            }),
            Err(e) => Err(pforge_runtime::Error::Handler(format!(
                "Transpilation failed: {}",
                e
            ))),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transpile_simple() {
        let handler = TranspileHandler;
        let input = TranspileInput {
            source: r#"
                fn main() {
                    let x = 42;
                }
            "#
            .to_string(),
            optimize: false,
            strict: false,
        };

        let result = handler.handle(input).await.unwrap();
        assert!(result.shell_script.contains("x=42"));
        assert!(result.shell_script.contains("#!/bin/sh"));
    }

    #[tokio::test]
    async fn test_transpile_println() {
        let handler = TranspileHandler;
        let input = TranspileInput {
            source: r#"
                fn main() {
                    println!("Hello, World!");
                }
            "#
            .to_string(),
            optimize: false,
            strict: false,
        };

        let result = handler.handle(input).await.unwrap();
        assert!(result.shell_script.contains("rash_println"));
        assert!(result.shell_script.contains("Hello, World!"));
    }

    #[tokio::test]
    async fn test_transpile_invalid_syntax() {
        let handler = TranspileHandler;
        let input = TranspileInput {
            source: "invalid rust code".to_string(),
            optimize: false,
            strict: false,
        };

        let result = handler.handle(input).await;
        assert!(result.is_err());
    }
}
