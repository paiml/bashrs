use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub target: ShellDialect,
    pub verify: VerificationLevel,
    pub emit_proof: bool,
    pub optimize: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            target: ShellDialect::Posix,
            verify: VerificationLevel::Strict,
            emit_proof: false,
            optimize: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ShellDialect {
    Posix,
    Bash,
    Dash,
    Ash,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VerificationLevel {
    None,
    Basic,
    Strict,
    Paranoid,
}