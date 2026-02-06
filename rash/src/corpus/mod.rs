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
//! ## Scoring Formula
//!
//! ```text
//! Score = (A_success × 40) + (B_correct × 25) + (C_coverage × 15)
//!       + (D_lint_pass × 10) + (E_determinism × 10)
//! ```

pub mod registry;
pub mod runner;

pub use registry::{CorpusEntry, CorpusFormat, CorpusRegistry, CorpusTier};
pub use runner::{CorpusResult, CorpusRunner, CorpusScore, ConvergenceEntry};
