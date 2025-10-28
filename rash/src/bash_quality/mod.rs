//! Bash Quality Tools
//!
//! Comprehensive quality tooling for bash scripts including:
//! - Testing: Run inline tests with GIVEN/WHEN/THEN syntax
//! - Scoring: TDG-style quality scoring (A+ to F)
//! - Coverage: Line and function coverage tracking (future)
//! - Formatting: Bash script formatting (future)
//!
//! This module provides the foundation for making bashrs the "cargo for bash".

pub mod coverage;
pub mod scoring;
pub mod testing;
