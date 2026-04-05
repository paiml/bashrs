#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Coverage tests for installer checkpoint module.
//!
//! Focuses on uncovered branches in:
//! - RunStatus and StepStatus parsing (invalid values, all variants)
//! - InstallerRun lifecycle (new, new_hermetic, complete, fail)
//! - StepCheckpoint lifecycle (new, start, complete, fail, skip)
//! - StepCheckpoint duration calculation
//! - StateFile creation and backup tracking
//! - CheckpointStore operations without active run (error paths)
//! - CheckpointStore step lookup (missing step errors)
//! - CheckpointStore hermetic consistency verification edge cases
//! - CheckpointStore persistence and reload
//! - Serialization/deserialization roundtrip

use std::path::Path;
use tempfile::TempDir;

use crate::installer::checkpoint::{
#[test]
fn test_CHECKPOINT_COV_last_successful_step_all_failed() {
    let temp_dir = TempDir::new().unwrap();
    let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
    store.start_run("app", "1.0").unwrap();
    store.add_step("s1").unwrap();
    store.start_step("s1").unwrap();
    store.fail_step("s1", "err").unwrap();

    assert!(store.last_successful_step().is_none());
}

#[test]
fn test_CHECKPOINT_COV_last_successful_step_returns_latest() {
    let temp_dir = TempDir::new().unwrap();
    let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
    store.start_run("app", "1.0").unwrap();

    store.add_step("s1").unwrap();
    store.start_step("s1").unwrap();
    store.complete_step("s1", None).unwrap();

    store.add_step("s2").unwrap();
    store.start_step("s2").unwrap();
    store.complete_step("s2", Some("done".to_string())).unwrap();

    store.add_step("s3").unwrap();
    // s3 still pending

    let last = store.last_successful_step().unwrap();
    assert_eq!(last.step_id, "s2");
}

// =============================================================================
// CheckpointStore — get_step
// =============================================================================

#[test]
fn test_CHECKPOINT_COV_get_step_found() {
    let temp_dir = TempDir::new().unwrap();
    let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
    store.start_run("app", "1.0").unwrap();
    store.add_step("step-x").unwrap();

    let step = store.get_step("step-x");
    assert!(step.is_some());
    assert_eq!(step.unwrap().step_id, "step-x");
}

#[test]
fn test_CHECKPOINT_COV_get_step_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
    store.start_run("app", "1.0").unwrap();

    let step = store.get_step("not-here");
    assert!(step.is_none());
}

// =============================================================================
// Persistence — load from nonexistent creates new
// =============================================================================

#[test]
fn test_CHECKPOINT_COV_load_creates_new_if_no_file() {
    let temp_dir = TempDir::new().unwrap();
    // Load from dir with no checkpoint.json
    let store = CheckpointStore::load(temp_dir.path()).unwrap();
    assert!(store.current_run_id().is_none());
    assert_eq!(store.steps().len(), 0);
}

// =============================================================================
// Persistence — full round-trip with all data
// =============================================================================

#[test]
fn test_CHECKPOINT_COV_persistence_full_roundtrip() {
    let temp_dir = TempDir::new().unwrap();

    // Create a store with various states
    {
        let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
        store
            .start_hermetic_run("my-app", "3.0.0", "lockhash")
            .unwrap();

        store.add_step("s1").unwrap();
        store.start_step("s1").unwrap();
        store
            .complete_step("s1", Some("output1".to_string()))
            .unwrap();

        store.add_step("s2").unwrap();
        store.start_step("s2").unwrap();
        store.fail_step("s2", "disk full").unwrap();

        store.add_step("s3").unwrap();
        // s3 stays pending

        store
            .track_file("s1", Path::new("/etc/conf"), "sha256:abc")
            .unwrap();
    }

    // Reload and verify everything
    {
        let store = CheckpointStore::load(temp_dir.path()).unwrap();
        assert!(store.current_run_id().is_some());
        assert!(store.is_hermetic());

        let s1 = store.get_step("s1").unwrap();
        assert_eq!(s1.status, StepStatus::Completed);
        assert_eq!(s1.output_log.as_deref(), Some("output1"));

        let s2 = store.get_step("s2").unwrap();
        assert_eq!(s2.status, StepStatus::Failed);
        assert_eq!(s2.error_message.as_deref(), Some("disk full"));

        let s3 = store.get_step("s3").unwrap();
        assert_eq!(s3.status, StepStatus::Pending);

        let files = store.state_files_for_step("s1");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].content_hash, "sha256:abc");

        let last = store.last_successful_step().unwrap();
        assert_eq!(last.step_id, "s1");
    }
}

// =============================================================================
// CheckpointStore — start_run clears previous state
// =============================================================================

#[test]
fn test_CHECKPOINT_COV_start_run_clears_previous() {
    let temp_dir = TempDir::new().unwrap();
    let mut store = CheckpointStore::new(temp_dir.path()).unwrap();

    // First run
    store.start_run("app-1", "1.0").unwrap();
    store.add_step("old-step").unwrap();
    store
        .track_file("old-step", Path::new("/old"), "hash")
        .unwrap();

    // Second run should clear
    store.start_run("app-2", "2.0").unwrap();
    assert_eq!(store.steps().len(), 0);
    assert_eq!(store.state_files_for_step("old-step").len(), 0);
}
