//! # Corpus-Driven Transpilation Quality Module
//!
//! Implements the corpus registry, runner, and scoring system for measuring
//! and improving transpilation quality across Bash, Makefile, and Dockerfile
//! targets. Each corpus entry is a **potential falsifier** (Popper, 1959):
//! an input-output pair that could demonstrate transpilation failure.
//!
//! ## Architecture
//!
//! - **Registry**: `CorpusEntry` and `CorpusRegistry` types for metadata management
//! - **Runner**: `CorpusRunner` that transpiles entries and compares against expected output
//! - **Scoring**: 100-point quality score (transpilation + correctness + coverage + lint + determinism)
//! - **Convergence**: Tracks transpilation rate over iterations (Kaizen methodology)
//!
//! ## V2 Scoring Formula
//!
//! ```text
//! Score = A(30) + B_L1(10) + B_L2(8) + B_L3(7) + C(15) + D(10) + E(10) + F(5) + G(5)
//! ```
//!
//! Gates: A < 60% → B-G = 0; schema_invalid → score = 0; B_L1 fail → B_L2/B_L3 = 0

pub mod adversarial_generator;
pub mod adversarial_templates;
pub mod citl;
pub mod convergence;
#[allow(clippy::expect_used)] // Dataset uses expect() for internal invariants
pub mod dataset;
#[allow(clippy::expect_used)] // Domain categories uses expect() for internal invariants
pub mod domain_categories;
#[allow(clippy::expect_used)] // Error dedup uses expect() for internal invariants
pub mod error_dedup;
pub mod graph_priority;
pub mod oip;
pub mod pattern_store;
#[allow(clippy::expect_used)] // Quality gates uses expect() for internal invariants
pub mod quality_gates;
pub mod registry;
#[allow(clippy::expect_used)] // Runner uses expect() for internal invariants
pub mod runner;
pub mod schema_enforcement;
pub mod tier_analysis;

#[cfg(test)]
#[path = "registry_coverage_tests.rs"]
mod registry_coverage_tests;

#[cfg(test)]
#[path = "runner_coverage_tests.rs"]
mod runner_coverage_tests;

#[cfg(test)]
#[path = "runner_coverage_tests2.rs"]
mod runner_coverage_tests2;

#[cfg(test)]
#[path = "error_dedup_tests.rs"]
mod error_dedup_tests;

#[cfg(test)]
#[path = "pattern_store_tests.rs"]
mod pattern_store_tests;

#[cfg(test)]
#[path = "adversarial_templates_tests.rs"]
mod adversarial_templates_tests;

pub use pattern_store::{PatternStore, ShellFixPattern};
pub use registry::{CorpusEntry, CorpusFormat, CorpusRegistry, CorpusTier};
pub use runner::{ConvergenceEntry, CorpusResult, CorpusRunner, CorpusScore, FormatScore};
