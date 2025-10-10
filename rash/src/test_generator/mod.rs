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
pub mod unit_tests;
pub mod property_tests;
pub mod doctests;
pub mod mutation_config;
pub mod coverage;

pub use core::{TestGenerator, TestGenOptions, TestGenError, TestGenResult};
pub use unit_tests::{UnitTestGenerator, UnitTest, Assertion};
pub use property_tests::{PropertyTestGenerator, PropertyTest, Property};
pub use doctests::{DoctestGenerator, Doctest};
pub use mutation_config::{MutationConfigGenerator, MutationConfig};
pub use coverage::{CoverageTracker, QualityReport};
