//! Coverage tests for formal/inspector.rs — targeting uncovered branches in:
//!   - `compute_transformation` (env added/modified/removed, cwd change,
//!     fs changes, output/error produced, exit code change)
//!   - `generate_report` (Failure and Partial VerificationResult variants)
//!   - `compare_filesystems` (path-only-in-rash, path-only-in-posix,
//!     path-differs branches)

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "inspector_coverage_tests_tests_empty_state.rs"]
// FIXME(PMAT-238): mod tests_extracted;
