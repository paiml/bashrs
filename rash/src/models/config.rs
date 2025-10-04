use crate::validation::ValidationLevel;
use serde::{Deserialize, Serialize};

/// Configuration options for transpilation.
///
/// Controls the target shell dialect, verification level, optimization,
/// and other transpilation behaviors.
///
/// # Examples
///
/// ## Default Configuration
///
/// ```rust
/// use bashrs::Config;
///
/// let config = Config::default();
/// assert_eq!(config.optimize, true);
/// ```
///
/// ## Custom Configuration
///
/// ```rust
/// use bashrs::Config;
/// use bashrs::models::{ShellDialect, VerificationLevel};
///
/// let config = Config {
///     target: ShellDialect::Bash,
///     verify: VerificationLevel::Paranoid,
///     optimize: false,
///     ..Config::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Target shell dialect for generated scripts
    pub target: ShellDialect,

    /// Level of verification to apply during transpilation
    pub verify: VerificationLevel,

    /// Whether to emit formal verification proofs
    pub emit_proof: bool,

    /// Enable IR optimization passes
    pub optimize: bool,

    /// ShellCheck validation level
    pub validation_level: Option<ValidationLevel>,

    /// Enable strict POSIX mode (no extensions)
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

/// Target shell dialect for generated scripts.
///
/// Determines which shell-specific features are available and
/// how the output is optimized.
///
/// # Examples
///
/// ```rust
/// use bashrs::models::ShellDialect;
///
/// let dialect = ShellDialect::Posix; // Maximum compatibility
/// let dialect = ShellDialect::Bash;  // Bash-specific optimizations
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShellDialect {
    /// POSIX sh - maximum compatibility
    Posix,

    /// Bash (version 3.2+)
    Bash,

    /// Debian Almquist Shell
    Dash,

    /// Almquist Shell (BusyBox)
    Ash,
}

/// Level of verification applied during transpilation.
///
/// Higher levels perform more safety checks but may be more strict
/// about accepting code patterns.
///
/// # Examples
///
/// ```rust
/// use bashrs::models::VerificationLevel;
///
/// let level = VerificationLevel::Basic;   // Fast, essential checks
/// let level = VerificationLevel::Strict;  // Recommended for production
/// let level = VerificationLevel::Paranoid; // Maximum safety
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationLevel {
    /// No verification (not recommended)
    None,

    /// Basic safety checks only
    Basic,

    /// Strict verification (recommended)
    Strict,

    /// Maximum verification (slowest)
    Paranoid,
}
