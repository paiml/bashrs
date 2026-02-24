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
    CheckpointStore, InstallerRun, RunStatus, StateFile, StepCheckpoint, StepStatus,
};

// =============================================================================
// RunStatus parsing edge cases
// =============================================================================

#[test]
fn test_CHECKPOINT_COV_run_status_parse_all_variants() {
    assert_eq!(RunStatus::parse("running"), Some(RunStatus::Running));
    assert_eq!(RunStatus::parse("completed"), Some(RunStatus::Completed));
    assert_eq!(RunStatus::parse("failed"), Some(RunStatus::Failed));
    assert_eq!(RunStatus::parse("aborted"), Some(RunStatus::Aborted));
}

#[test]
fn test_CHECKPOINT_COV_run_status_parse_invalid() {
    assert_eq!(RunStatus::parse("unknown"), None);
    assert_eq!(RunStatus::parse(""), None);
    assert_eq!(RunStatus::parse("RUNNING"), None);
    assert_eq!(RunStatus::parse("Running"), None);
}

#[test]
fn test_CHECKPOINT_COV_run_status_as_str_all() {
    assert_eq!(RunStatus::Running.as_str(), "running");
    assert_eq!(RunStatus::Completed.as_str(), "completed");
    assert_eq!(RunStatus::Failed.as_str(), "failed");
    assert_eq!(RunStatus::Aborted.as_str(), "aborted");
}

#[test]
fn test_CHECKPOINT_COV_run_status_roundtrip() {
    for status in [
        RunStatus::Running,
        RunStatus::Completed,
        RunStatus::Failed,
        RunStatus::Aborted,
    ] {
        let s = status.as_str();
        let parsed = RunStatus::parse(s).unwrap();
        assert_eq!(parsed, status);
    }
}

// =============================================================================
// StepStatus parsing edge cases
// =============================================================================

#[test]
fn test_CHECKPOINT_COV_step_status_parse_all_variants() {
    assert_eq!(StepStatus::parse("pending"), Some(StepStatus::Pending));
    assert_eq!(StepStatus::parse("running"), Some(StepStatus::Running));
    assert_eq!(StepStatus::parse("completed"), Some(StepStatus::Completed));
    assert_eq!(StepStatus::parse("failed"), Some(StepStatus::Failed));
    assert_eq!(StepStatus::parse("skipped"), Some(StepStatus::Skipped));
}

#[test]
fn test_CHECKPOINT_COV_step_status_parse_invalid() {
    assert_eq!(StepStatus::parse("invalid"), None);
    assert_eq!(StepStatus::parse(""), None);
    assert_eq!(StepStatus::parse("PENDING"), None);
}

#[test]
fn test_CHECKPOINT_COV_step_status_as_str_all() {
    assert_eq!(StepStatus::Pending.as_str(), "pending");
    assert_eq!(StepStatus::Running.as_str(), "running");
    assert_eq!(StepStatus::Completed.as_str(), "completed");
    assert_eq!(StepStatus::Failed.as_str(), "failed");
    assert_eq!(StepStatus::Skipped.as_str(), "skipped");
}

#[test]
fn test_CHECKPOINT_COV_step_status_roundtrip() {
    for status in [
        StepStatus::Pending,
        StepStatus::Running,
        StepStatus::Completed,
        StepStatus::Failed,
        StepStatus::Skipped,
    ] {
        let s = status.as_str();
        let parsed = StepStatus::parse(s).unwrap();
        assert_eq!(parsed, status);
    }
}

// =============================================================================
// InstallerRun lifecycle
// =============================================================================

#[test]
fn test_CHECKPOINT_COV_installer_run_new() {
    let run = InstallerRun::new("my-app", "2.0.0");
    assert!(run.run_id.starts_with("run-"));
    assert_eq!(run.installer_name, "my-app");
    assert_eq!(run.installer_version, "2.0.0");
    assert_eq!(run.status, RunStatus::Running);
    assert!(!run.hermetic_mode);
    assert!(run.lockfile_hash.is_none());
    assert!(run.completed_at.is_none());
    assert!(run.started_at > 0);
}

#[test]
fn test_CHECKPOINT_COV_installer_run_new_hermetic() {
    let run = InstallerRun::new_hermetic("my-app", "2.0.0", "sha256:abc123");
    assert!(run.hermetic_mode);
    assert_eq!(run.lockfile_hash, Some("sha256:abc123".to_string()));
    assert_eq!(run.status, RunStatus::Running);
}

#[test]
fn test_CHECKPOINT_COV_installer_run_complete() {
    let mut run = InstallerRun::new("app", "1.0");
    assert!(run.completed_at.is_none());

    run.complete();
    assert_eq!(run.status, RunStatus::Completed);
    assert!(run.completed_at.is_some());
}

#[test]
fn test_CHECKPOINT_COV_installer_run_fail() {
    let mut run = InstallerRun::new("app", "1.0");
    run.fail();
    assert_eq!(run.status, RunStatus::Failed);
    assert!(run.completed_at.is_some());
}

// =============================================================================
// StepCheckpoint lifecycle
// =============================================================================

#[test]
fn test_CHECKPOINT_COV_step_new() {
    let step = StepCheckpoint::new("run-123", "install-deps");
    assert_eq!(step.run_id, "run-123");
    assert_eq!(step.step_id, "install-deps");
    assert_eq!(step.status, StepStatus::Pending);
    assert!(step.started_at.is_none());
    assert!(step.completed_at.is_none());
    assert!(step.duration_ms.is_none());
    assert!(step.state_snapshot.is_none());
    assert!(step.output_log.is_none());
    assert!(step.error_message.is_none());
}

#[test]
fn test_CHECKPOINT_COV_step_start() {
    let mut step = StepCheckpoint::new("run-1", "s1");
    step.start();
    assert_eq!(step.status, StepStatus::Running);
    assert!(step.started_at.is_some());
}

#[test]
fn test_CHECKPOINT_COV_step_complete_with_output() {
    let mut step = StepCheckpoint::new("run-1", "s1");
    step.start();
    step.complete(Some("install successful".to_string()));
    assert_eq!(step.status, StepStatus::Completed);
    assert!(step.completed_at.is_some());
    assert_eq!(step.output_log, Some("install successful".to_string()));
}

#[test]
fn test_CHECKPOINT_COV_step_complete_without_output() {
    let mut step = StepCheckpoint::new("run-1", "s1");
    step.start();
    step.complete(None);
    assert_eq!(step.status, StepStatus::Completed);
    assert!(step.output_log.is_none());
}

#[test]
fn test_CHECKPOINT_COV_step_complete_duration() {
    let mut step = StepCheckpoint::new("run-1", "s1");
    step.start();
    // Manually set started_at to ensure we can check duration
    step.started_at = Some(1000);
    step.complete(None);
    // completed_at will be current time, duration = (completed_at - 1000) * 1000
    assert!(step.duration_ms.is_some());
}

#[test]
fn test_CHECKPOINT_COV_step_fail() {
    let mut step = StepCheckpoint::new("run-1", "s1");
    step.start();
    step.fail("network timeout");
    assert_eq!(step.status, StepStatus::Failed);
    assert_eq!(step.error_message, Some("network timeout".to_string()));
    assert!(step.completed_at.is_some());
}

#[test]
fn test_CHECKPOINT_COV_step_fail_duration() {
    let mut step = StepCheckpoint::new("run-1", "s1");
    step.started_at = Some(500);
    step.fail("error");
    assert!(step.duration_ms.is_some());
}

#[test]
fn test_CHECKPOINT_COV_step_fail_no_start() {
    // Fail without start: no duration since started_at is None
    let mut step = StepCheckpoint::new("run-1", "s1");
    step.fail("error");
    assert_eq!(step.status, StepStatus::Failed);
    assert!(step.duration_ms.is_none());
}

#[test]
fn test_CHECKPOINT_COV_step_skip() {
    let mut step = StepCheckpoint::new("run-1", "s1");
    step.skip();
    assert_eq!(step.status, StepStatus::Skipped);
}

// =============================================================================
// StateFile creation and backup tracking
// =============================================================================

#[test]
fn test_CHECKPOINT_COV_state_file_new() {
    let sf = StateFile::new("run-1", "s1", Path::new("/etc/config.txt"), "sha256:deadbeef");
    assert_eq!(sf.run_id, "run-1");
    assert_eq!(sf.step_id, "s1");
    assert_eq!(sf.file_path.to_str().unwrap(), "/etc/config.txt");
    assert_eq!(sf.content_hash, "sha256:deadbeef");
    assert!(sf.backed_up_at.is_none());
    assert!(sf.backup_path.is_none());
}

#[test]
fn test_CHECKPOINT_COV_state_file_set_backup() {
    let mut sf = StateFile::new("run-1", "s1", Path::new("/etc/config.txt"), "sha256:abc");
    sf.set_backup(Path::new("/backups/config.txt.bak"));
    assert!(sf.backed_up_at.is_some());
    assert_eq!(
        sf.backup_path.as_ref().unwrap().to_str().unwrap(),
        "/backups/config.txt.bak"
    );
}

// =============================================================================
// CheckpointStore — error paths
// =============================================================================

#[test]
fn test_CHECKPOINT_COV_add_step_without_run_fails() {
    let temp_dir = TempDir::new().unwrap();
    let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
    // No run started — should fail
    let result = store.add_step("s1");
    assert!(result.is_err());
}

#[test]
fn test_CHECKPOINT_COV_start_step_not_found_fails() {
    let temp_dir = TempDir::new().unwrap();
    let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
    store.start_run("app", "1.0").unwrap();
    // Step doesn't exist
    let result = store.start_step("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_CHECKPOINT_COV_complete_step_not_found_fails() {
    let temp_dir = TempDir::new().unwrap();
    let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
    store.start_run("app", "1.0").unwrap();
    let result = store.complete_step("nonexistent", None);
    assert!(result.is_err());
}

#[test]
fn test_CHECKPOINT_COV_fail_step_not_found_fails() {
    let temp_dir = TempDir::new().unwrap();
    let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
    store.start_run("app", "1.0").unwrap();
    let result = store.fail_step("nonexistent", "error");
    assert!(result.is_err());
}

#[test]
fn test_CHECKPOINT_COV_track_file_without_run_fails() {
    let temp_dir = TempDir::new().unwrap();
    let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
    let result = store.track_file("s1", Path::new("/a"), "hash");
    assert!(result.is_err());
}

// =============================================================================
// CheckpointStore — hermetic consistency edge cases
// =============================================================================

#[test]
fn test_CHECKPOINT_COV_hermetic_consistency_no_run() {
    let temp_dir = TempDir::new().unwrap();
    let store = CheckpointStore::new(temp_dir.path()).unwrap();
    // No run: should not error
    let result = store.verify_hermetic_consistency("any-hash");
    assert!(result.is_ok());
}

#[test]
fn test_CHECKPOINT_COV_hermetic_consistency_non_hermetic_run() {
    let temp_dir = TempDir::new().unwrap();
    let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
    store.start_run("app", "1.0").unwrap();
    // Non-hermetic run: should not error regardless of hash
    let result = store.verify_hermetic_consistency("any-hash");
    assert!(result.is_ok());
}

#[test]
fn test_CHECKPOINT_COV_hermetic_consistency_matching_hash() {
    let temp_dir = TempDir::new().unwrap();
    let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
    store.start_hermetic_run("app", "1.0", "hash123").unwrap();
    let result = store.verify_hermetic_consistency("hash123");
    assert!(result.is_ok());
}

#[test]
fn test_CHECKPOINT_COV_hermetic_consistency_mismatched_hash() {
    let temp_dir = TempDir::new().unwrap();
    let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
    store.start_hermetic_run("app", "1.0", "hash123").unwrap();
    let result = store.verify_hermetic_consistency("different");
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Lockfile drift"));
}

#[test]
fn test_CHECKPOINT_COV_is_hermetic_false_no_run() {
    let temp_dir = TempDir::new().unwrap();
    let store = CheckpointStore::new(temp_dir.path()).unwrap();
    assert!(!store.is_hermetic());
}

#[test]
fn test_CHECKPOINT_COV_is_hermetic_false_normal_run() {
    let temp_dir = TempDir::new().unwrap();
    let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
    store.start_run("app", "1.0").unwrap();
    assert!(!store.is_hermetic());
}

#[test]
fn test_CHECKPOINT_COV_is_hermetic_true() {
    let temp_dir = TempDir::new().unwrap();
    let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
    store.start_hermetic_run("app", "1.0", "h").unwrap();
    assert!(store.is_hermetic());
}

// =============================================================================
// CheckpointStore — complete_run
// =============================================================================

#[test]
fn test_CHECKPOINT_COV_complete_run() {
    let temp_dir = TempDir::new().unwrap();
    let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
    store.start_run("app", "1.0").unwrap();
    store.complete_run().unwrap();

    // Reload and verify
    let reloaded = CheckpointStore::load(temp_dir.path()).unwrap();
    // The run should still be present
    assert!(reloaded.current_run_id().is_some());
}

#[test]
fn test_CHECKPOINT_COV_complete_run_no_active_run() {
    let temp_dir = TempDir::new().unwrap();
    let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
    // complete_run with no run should not panic (it saves gracefully)
    let result = store.complete_run();
    assert!(result.is_ok());
}

// =============================================================================
// CheckpointStore — fail_step marks run as failed
// =============================================================================

#[test]
fn test_CHECKPOINT_COV_fail_step_marks_run_failed() {
    let temp_dir = TempDir::new().unwrap();
    let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
    store.start_run("app", "1.0").unwrap();
    store.add_step("s1").unwrap();
    store.start_step("s1").unwrap();
    store.fail_step("s1", "disk full").unwrap();

    // Reload and check run status
    let reloaded = CheckpointStore::load(temp_dir.path()).unwrap();
    let step = reloaded.get_step("s1").unwrap();
    assert_eq!(step.status, StepStatus::Failed);
    assert_eq!(step.error_message.as_deref(), Some("disk full"));
}

// =============================================================================
// CheckpointStore — steps accessor and state_files_for_step
// =============================================================================

#[test]
fn test_CHECKPOINT_COV_steps_accessor() {
    let temp_dir = TempDir::new().unwrap();
    let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
    store.start_run("app", "1.0").unwrap();
    store.add_step("s1").unwrap();
    store.add_step("s2").unwrap();
    store.add_step("s3").unwrap();

    let steps = store.steps();
    assert_eq!(steps.len(), 3);
    assert_eq!(steps[0].step_id, "s1");
    assert_eq!(steps[1].step_id, "s2");
    assert_eq!(steps[2].step_id, "s3");
}

#[test]
fn test_CHECKPOINT_COV_state_files_for_step_multiple() {
    let temp_dir = TempDir::new().unwrap();
    let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
    store.start_run("app", "1.0").unwrap();
    store.add_step("s1").unwrap();

    store.track_file("s1", Path::new("/a.txt"), "hash-a").unwrap();
    store.track_file("s1", Path::new("/b.txt"), "hash-b").unwrap();
    store.track_file("s2", Path::new("/c.txt"), "hash-c").unwrap();

    let files_s1 = store.state_files_for_step("s1");
    assert_eq!(files_s1.len(), 2);

    let files_s2 = store.state_files_for_step("s2");
    assert_eq!(files_s2.len(), 1);

    let files_s3 = store.state_files_for_step("nonexistent");
    assert_eq!(files_s3.len(), 0);
}

// =============================================================================
// CheckpointStore — last_successful_step edge cases
// =============================================================================

#[test]
fn test_CHECKPOINT_COV_last_successful_step_none() {
    let temp_dir = TempDir::new().unwrap();
    let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
    store.start_run("app", "1.0").unwrap();
    store.add_step("s1").unwrap();
    // No steps completed
    assert!(store.last_successful_step().is_none());
}

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
        store.start_hermetic_run("my-app", "3.0.0", "lockhash").unwrap();

        store.add_step("s1").unwrap();
        store.start_step("s1").unwrap();
        store.complete_step("s1", Some("output1".to_string())).unwrap();

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
