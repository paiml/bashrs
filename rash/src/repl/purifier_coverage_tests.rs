//! Coverage tests for repl/purifier.rs — targeting uncovered branches in:
//!   - `explain_purification_changes_detailed`
//!   - `collect_change_explanations`
//!   - `generate_idempotency_alternatives`
//!   - `format_purified_lint_result_with_context` (violation branch)
//!
//! Each test targets a specific uncovered branch identified by the coverage
//! report.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

#[cfg(test)]
#[path = "purifier_coverage_tests_tests_make_lint.rs"]
mod tests_extracted;
