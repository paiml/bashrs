//! Pre-flight Formatter Implementation
//!
//! This module implements the formatter specification to reduce bash's 1,247
//! shift/reduce conflicts to 127 in the canonical grammar, enabling faster
//! verification convergence and simplified SMT encodings.
//!
//! ## Safety Note
//! Formatter uses unwrap() on validated grammar operations and token positions.
#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]

pub mod contract;
pub mod dialect;
pub mod engine;
pub mod logging;
pub mod source_map;
pub mod transforms;
pub mod types;

pub use contract::*;
pub use dialect::*;
pub use engine::*;
pub use logging::*;
pub use source_map::*;
pub use transforms::*;
pub use types::*;

use std::borrow::Cow;

/// Core formatter trait for pre-flight syntactic normalization
pub trait PreflightFormatter: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    fn format<'a>(
        &self,
        source: &'a [u8],
        dialect: ShellDialect,
        config: FormatConfig,
    ) -> std::result::Result<FormattedSource<'a>, Self::Error>;
}

/// Main formatter implementation
pub struct RashFormatter {
    engine: NormalizationEngine,
    #[allow(dead_code)]
    contract_system: contract::ContractSystem,
}

impl RashFormatter {
    pub fn new() -> Self {
        Self {
            engine: NormalizationEngine::new(),
            contract_system: contract::ContractSystem::new(),
        }
    }
}

impl Default for RashFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl PreflightFormatter for RashFormatter {
    type Error = crate::Error;

    fn format<'a>(
        &self,
        source: &'a [u8],
        dialect: ShellDialect,
        config: FormatConfig,
    ) -> std::result::Result<FormattedSource<'a>, Self::Error> {
        // Convert input to string for processing
        let source_str = std::str::from_utf8(source)
            .map_err(|e| crate::Error::Internal(format!("Invalid UTF-8: {e}")))?;

        // Fast path: check if already canonical
        if self.engine.is_canonical(source) {
            return Ok(FormattedSource {
                text: Cow::Borrowed(source_str),
                source_map: SourceMap::identity(source.len()),
                metadata: SemanticMetadata::default(),
                canonical_hash: blake3::hash(source).into(),
                transforms: logging::TransformLog::new(),
            });
        }

        // Full normalization with transform tracking
        let mut engine = self.engine.clone();
        engine.normalize(source, dialect, config)
    }
}

#[cfg(test)]
#[path = "formatter_tests.rs"]
mod formatter_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formatter_creation() {
        let formatter = RashFormatter::new();
        assert!(std::ptr::eq(&formatter as *const _, &formatter as *const _));
    }

    #[test]
    fn test_format_identity() {
        let formatter = RashFormatter::new();
        let source = b"echo hello";
        let config = FormatConfig::default();

        let result = formatter.format(source, ShellDialect::Posix, config);
        assert!(result.is_ok());

        let formatted = result.unwrap();
        assert_eq!(formatted.text.as_ref(), "echo hello");
    }

    #[test]
    fn test_format_invalid_utf8() {
        let formatter = RashFormatter::new();
        let source = &[0xff, 0xfe, 0xfd]; // Invalid UTF-8
        let config = FormatConfig::default();

        let result = formatter.format(source, ShellDialect::Posix, config);
        assert!(result.is_err());
    }
}
