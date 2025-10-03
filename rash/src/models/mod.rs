pub mod config;
pub mod diagnostic;
pub mod error;

pub use config::{Config, ShellDialect, VerificationLevel};
pub use diagnostic::{Diagnostic, ErrorCategory};
pub use error::{Error, Result};
