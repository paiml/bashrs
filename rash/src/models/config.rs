use crate::validation::ValidationLevel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub target: ShellDialect,
    pub verify: VerificationLevel,
    pub emit_proof: bool,
    pub optimize: bool,
    pub validation_level: Option<ValidationLevel>,
    pub strict_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            target: ShellDialect::Posix,
            verify: VerificationLevel::Strict,
            emit_proof: false,
            optimize: true,
            validation_level: Some(ValidationLevel::Minimal),
            strict_mode: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShellDialect {
    Posix,
    Bash,
    Dash,
    Ash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationLevel {
    None,
    Basic,
    Strict,
    Paranoid,
}
