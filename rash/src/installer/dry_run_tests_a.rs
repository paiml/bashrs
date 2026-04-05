#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Coverage tests for installer dry_run module.
//!
//! Focuses on uncovered branches in:
//! - FileChange::to_diff() for ModeChanged variant
//! - FileChange::compute_unified_diff() edge cases (identical, prefix/suffix overlap)
//! - PackageOperation formatting (upgrade, install without version)
//! - ServiceOperation formatting (Disable, Stop, Restart)
//! - UserGroupOperation formatting (RemoveFromGroup, CreateUser, CreateGroup)
//! - DryRunContext simulate methods and accessors
//! - DryRunSummary edge cases (has_changes, steps_would_fail)
//! - DiffPreview sections rendering
//! - DiffPreview::is_empty and mixed content

use std::path::PathBuf;

use crate::installer::dry_run::{
#[test]
fn test_DRYRUN_COV_diff_preview_is_empty() {
    let preview = DiffPreview {
        file_changes: vec![],
        package_ops: vec![],
        service_ops: vec![],
        user_ops: vec![],
    };
    assert!(preview.is_empty());
    assert_eq!(preview.to_diff_text(), "");
}

#[test]
fn test_DRYRUN_COV_diff_preview_not_empty() {
    let preview = DiffPreview {
        file_changes: vec![FileChange::created("/a.txt", "x", None)],
        package_ops: vec![],
        service_ops: vec![],
        user_ops: vec![],
    };
    assert!(!preview.is_empty());
}

// =============================================================================
// DryRunContext — overwrite behavior
// =============================================================================

#[test]
fn test_DRYRUN_COV_simulate_file_write_overwrites_previous() {
    let mut ctx = DryRunContext::new();
    ctx.simulate_file_write("/etc/config.yml", "first", None);
    ctx.simulate_file_write("/etc/config.yml", "second", Some(0o600));

    let changes: Vec<_> = ctx.file_changes().collect();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].after.as_deref(), Some("second"));
    assert_eq!(changes[0].mode, Some(0o600));
}

// =============================================================================
// SimulationEntry logging
// =============================================================================

#[test]
fn test_DRYRUN_COV_simulation_log_success_and_failure() {
    let mut ctx = DryRunContext::new();
    ctx.log_step("s1", "Step 1", "Would do A");
    ctx.log_step("s2", "Step 2", "Would do B");
    ctx.log_step_failure("s3", "Step 3", "Missing dependency");

    let log = ctx.simulation_log();
    assert_eq!(log.len(), 3);

    assert_eq!(log[0].step_id, "s1");
    assert!(log[0].would_succeed);
    assert!(log[0].failure_reason.is_none());

    assert_eq!(log[2].step_id, "s3");
    assert!(!log[2].would_succeed);
    assert_eq!(log[2].failure_reason.as_deref(), Some("Missing dependency"));
    assert!(log[2].description.contains("Would fail:"));
}

// =============================================================================
// Combined summary correctness
// =============================================================================

#[test]
fn test_DRYRUN_COV_summary_counts_mixed_operations() {
    let mut ctx = DryRunContext::new();

    // 2 creates, 1 modify, 1 delete
    ctx.simulate_file_write("/a.txt", "a", None);
    ctx.simulate_file_write("/b.txt", "b", None);
    ctx.simulate_file_modify("/c.txt", "old", "new");
    ctx.simulate_file_delete("/d.txt", "gone");

    // 2 installs, 1 remove
    ctx.simulate_package_install("p1", None);
    ctx.simulate_package_install("p2", Some("1.0"));
    ctx.simulate_package_remove("p3");

    // 1 enable
    ctx.simulate_service_enable("svc1");

    // 2 user ops
    ctx.simulate_add_to_group("u1", "g1");
    ctx.simulate_add_to_group("u2", "g2");

    // 1 failure
    ctx.log_step_failure("bad", "Bad Step", "oops");

    let summary = ctx.summary();
    assert_eq!(summary.files_created, 2);
    assert_eq!(summary.files_modified, 1);
    assert_eq!(summary.files_deleted, 1);
    assert_eq!(summary.packages_installed, 2);
    assert_eq!(summary.packages_removed, 1);
    assert_eq!(summary.services_enabled, 1);
    assert_eq!(summary.users_modified, 2);
    assert_eq!(summary.steps_would_fail, 1);
    assert!(summary.has_changes());
}
