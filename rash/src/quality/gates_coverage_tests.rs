//! Additional coverage tests for quality/gates.rs
//!
//! These tests focus on data structures, configuration parsing, formatting,
//! threshold logic, and the disabled-gate paths that don't shell out to
//! external processes. NO external commands are invoked.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "gates_coverage_tests_tests_gate_result.rs"]
// FIXME(PMAT-238): mod tests_extracted;
