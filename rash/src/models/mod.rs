pub mod config;
pub mod error;

pub use config::{Config, ShellDialect, VerificationLevel};
pub use error::{Error, Result};