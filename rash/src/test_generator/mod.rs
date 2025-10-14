//! Test Generation Module
//!
//! Automatically generates comprehensive test suites for transpiled Rust code:
//! - Unit tests (branch coverage, edge cases, error cases)
//! - Property tests (determinism, idempotency, bounds)
//! - Doctests (from bash comments)
//! - Mutation test configuration
//!
//! Targets: ≥80% coverage, ≥85% mutation score

pub mod core;
pub mod coverage;
pub mod doctests;
pub mod mutation_config;
pub mod property_tests;
pub mod unit_tests;

pub use core::{TestGenError, TestGenOptions, TestGenResult, TestGenerator};
pub use coverage::{CoverageTracker, QualityReport};
pub use doctests::{Doctest, DoctestGenerator};
pub use mutation_config::{MutationConfig, MutationConfigGenerator};
pub use property_tests::{Property, PropertyTest, PropertyTestGenerator};
pub use unit_tests::{Assertion, UnitTest, UnitTestGenerator};
