pub mod config;
pub mod diagnostic;
pub mod error;

#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;

pub use config::{Config, ShellDialect, VerificationLevel};
pub use diagnostic::{Diagnostic, ErrorCategory};
pub use error::{Error, Result};
