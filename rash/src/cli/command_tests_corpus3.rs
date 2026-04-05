//! Tests for corpus decision, analysis coverage, and ranking dimension stats modules.
//! These tests target lightweight pure functions that do not invoke runner.run().
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

// ---------------------------------------------------------------------------
// corpus_decision_commands::score_impact_color
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "command_tests_corpus3_tests_extracted.rs"]
mod tests_extracted;
