pub mod ast;
pub mod cli;
pub mod compiler;
pub mod container;
pub mod emitter;
pub mod formal;
pub mod formatter;
pub mod ir;
pub mod models;
pub mod services;
pub mod stdlib;
pub mod validation;
pub mod verifier;

#[cfg(test)]
pub mod testing;

#[cfg(feature = "playground")]
pub mod playground;

pub use models::{Config, Error, Result};

/// Transpile Rust source code to POSIX shell script
pub fn transpile(input: &str, config: Config) -> Result<String> {
    let validation_pipeline = validation::pipeline::ValidationPipeline::new(&config);

    let ast = services::parser::parse(input)?;
    ast::validate(&ast)?;
    validation_pipeline.validate_ast(&ast)?;

    let ir = ir::from_ast(&ast)?;
    validation_pipeline.validate_ir(&ir)?;

    let optimized = ir::optimize(ir, &config)?;
    let shell_code = emitter::emit(&optimized, &config)?;

    validation_pipeline.validate_output(&shell_code)?;

    Ok(shell_code)
}

/// Check if the given Rust code is valid for transpilation
pub fn check(input: &str) -> Result<()> {
    let ast = services::parser::parse(input)?;
    ast::validate(&ast)?;
    Ok(())
}
