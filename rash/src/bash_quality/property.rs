//! Property-based analysis for bash scripts (Sprint 12, PMAT-218).
//!
//! Analyzes bash scripts for 4 built-in properties:
//! - **idempotency**: Operations safe to re-run (mkdir -p, rm -f, ln -sf)
//! - **determinism**: Same input produces same output (no $RANDOM, $$, date)
//! - **posix**: POSIX sh compliance (no bashisms)
//! - **safety**: No destructive operations without guards

/// A property that can be tested against a bash script.
