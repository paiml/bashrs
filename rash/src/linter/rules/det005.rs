//! DET005: Time-dependent control flow (#232, split from DET002)
//!
//! RED stub: implementation lands in the GREEN commit.

use crate::linter::LintResult;

/// Check for a wall-clock value reaching a branch condition (#232).
pub fn check(_source: &str) -> LintResult {
    LintResult::new()
}

#[cfg(test)]
#[path = "det005_tests.rs"]
mod det005_tests;
