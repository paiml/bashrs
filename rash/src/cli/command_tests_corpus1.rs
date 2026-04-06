//! Tests for corpus helper functions in viz, ranking, entry, failure, and score_print modules.
//! These tests target lightweight pure functions that do not invoke runner.run().
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

// ---------------------------------------------------------------------------
// corpus_ranking_commands::sparkline_str
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "command_tests_corpus1_tests_sparkline_em.rs"]
mod tests_extracted;
