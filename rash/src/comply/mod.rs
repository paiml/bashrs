//! Shell artifact compliance system (SPEC-COMPLY-2026-001)
//!
//! Three-layer compliance model:
//! - Layer 1 (Jidoka): Automated check
//! - Layer 2 (Genchi Genbutsu): Evidence-based review
//! - Layer 3 (Kansa): Governance audit artifacts

pub mod config;
pub mod discovery;
pub mod rules;
pub mod runner;
pub mod scoring;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "discovery_tests.rs"]
mod discovery_tests;

#[cfg(test)]
#[path = "runner_tests.rs"]
mod comply_runner_tests;

#[cfg(test)]
#[path = "rules_tests.rs"]
mod rules_tests;

#[cfg(test)]
#[path = "scoring_tests.rs"]
mod scoring_tests;
