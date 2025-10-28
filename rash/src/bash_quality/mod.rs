//! Bash Quality Tools
//!
//! Comprehensive quality tooling for bash scripts including:
//! - Testing: Run inline tests with GIVEN/WHEN/THEN syntax
//! - Scoring: TDG-style quality scoring (A+ to F)
//! - Coverage: Line and function coverage tracking
//! - Formatting: Bash script formatting (NEW in v6.14.0)
//!
//! This module provides the foundation for making bashrs the "cargo for bash".

pub mod coverage;
pub mod formatter;
pub mod formatter_config;
pub mod scoring;
pub mod testing;

// Re-export for convenience
pub use formatter::Formatter;
pub use formatter_config::FormatterConfig;
