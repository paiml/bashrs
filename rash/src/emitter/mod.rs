pub mod escape;
pub mod posix;

#[cfg(test)]
mod tests;

pub use posix::PosixEmitter;

use crate::ir::ShellIR;
use crate::models::{Config, Result};

/// Emit shell code from IR based on target dialect
pub fn emit(ir: &ShellIR, config: &Config) -> Result<String> {
    match config.target {
        crate::models::config::ShellDialect::Posix => {
            let emitter = PosixEmitter::new(config.clone());
            emitter.emit(ir)
        }
        crate::models::config::ShellDialect::Bash => {
            // For now, use POSIX emitter for Bash too
            let emitter = PosixEmitter::new(config.clone());
            emitter.emit(ir)
        }
        _ => {
            // Default to POSIX for other dialects
            let emitter = PosixEmitter::new(config.clone());
            emitter.emit(ir)
        }
    }
}
