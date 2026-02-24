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
    DiffPreview, DryRunContext, DryRunSummary, FileChange, FileChangeType, PackageOperation,
    ServiceOperation, UserGroupOperation,
};

// =============================================================================
// FileChange — ModeChanged variant
// =============================================================================

#[test]
fn test_DRYRUN_COV_mode_changed_diff() {
    let change = FileChange {
        path: PathBuf::from("/etc/app.conf"),
        before: Some("content".to_string()),
        after: Some("content".to_string()),
        mode: Some(0o755),
        change_type: FileChangeType::ModeChanged,
    };

    let diff = change.to_diff();
    assert!(diff.contains("--- a/etc/app.conf"));
    assert!(diff.contains("+++ b/etc/app.conf"));
    assert!(diff.contains("# chmod 755"));
}

#[test]
fn test_DRYRUN_COV_mode_changed_no_mode() {
    let change = FileChange {
        path: PathBuf::from("/etc/app.conf"),
        before: Some("content".to_string()),
        after: Some("content".to_string()),
        mode: None,
        change_type: FileChangeType::ModeChanged,
    };

    let diff = change.to_diff();
    assert!(diff.contains("--- a/etc/app.conf"));
    assert!(diff.contains("+++ b/etc/app.conf"));
    // No chmod line since mode is None
    assert!(!diff.contains("chmod"));
}

// =============================================================================
// FileChange — compute_unified_diff edge cases
// =============================================================================

#[test]
fn test_DRYRUN_COV_unified_diff_identical_content() {
    let change = FileChange::modified("/etc/same.txt", "line1\nline2\n", "line1\nline2\n");
    let diff = change.to_diff();
    // Identical content: no diff hunks beyond the header
    assert!(diff.contains("--- a/etc/same.txt"));
    assert!(diff.contains("+++ b/etc/same.txt"));
    // No @@ hunk header for identical content
    assert!(!diff.contains("@@"));
}

#[test]
fn test_DRYRUN_COV_unified_diff_only_additions() {
    let change = FileChange::modified("/etc/grow.txt", "line1\n", "line1\nline2\nline3\n");
    let diff = change.to_diff();
    assert!(diff.contains("@@"));
    assert!(diff.contains("+line2"));
    assert!(diff.contains("+line3"));
}

#[test]
fn test_DRYRUN_COV_unified_diff_only_removals() {
    let change = FileChange::modified("/etc/shrink.txt", "line1\nline2\nline3\n", "line1\n");
    let diff = change.to_diff();
    assert!(diff.contains("@@"));
    assert!(diff.contains("-line2"));
    assert!(diff.contains("-line3"));
}

#[test]
fn test_DRYRUN_COV_unified_diff_common_prefix_and_suffix() {
    let before = "header\nold_middle\nfooter\n";
    let after = "header\nnew_middle\nfooter\n";
    let change = FileChange::modified("/etc/middle.txt", before, after);
    let diff = change.to_diff();
    assert!(diff.contains("-old_middle"));
    assert!(diff.contains("+new_middle"));
    // header and footer should not appear in diff
    assert!(!diff.contains("-header"));
    assert!(!diff.contains("+header"));
}

#[test]
fn test_DRYRUN_COV_unified_diff_empty_before() {
    let change = FileChange::modified("/etc/new.txt", "", "new content\n");
    let diff = change.to_diff();
    assert!(diff.contains("+new content"));
}

#[test]
fn test_DRYRUN_COV_unified_diff_empty_after() {
    let change = FileChange::modified("/etc/empty.txt", "old content\n", "");
    let diff = change.to_diff();
    assert!(diff.contains("-old content"));
}

// =============================================================================
// FileChange — created/deleted edge cases
// =============================================================================

#[test]
fn test_DRYRUN_COV_created_multiline() {
    let change = FileChange::created("/etc/multi.conf", "a\nb\nc\n", Some(0o644));
    let diff = change.to_diff();
    assert!(diff.contains("@@ -0,0 +1,3 @@"));
    assert!(diff.contains("+a"));
    assert!(diff.contains("+b"));
    assert!(diff.contains("+c"));
}

#[test]
fn test_DRYRUN_COV_deleted_multiline() {
    let change = FileChange::deleted("/tmp/old.conf", "x\ny\nz\n");
    let diff = change.to_diff();
    assert!(diff.contains("@@ -1,3 +0,0 @@"));
    assert!(diff.contains("-x"));
    assert!(diff.contains("-y"));
    assert!(diff.contains("-z"));
}

#[test]
fn test_DRYRUN_COV_created_single_line() {
    let change = FileChange::created("/etc/one.txt", "only line", None);
    let diff = change.to_diff();
    assert!(diff.contains("@@ -0,0 +1,1 @@"));
    assert!(diff.contains("+only line"));
}

// =============================================================================
// PackageOperation edge cases
// =============================================================================

#[test]
fn test_DRYRUN_COV_package_install_without_version() {
    let op = PackageOperation::install("curl", None);
    assert_eq!(op.to_diff_line(), "+ curl");
}

#[test]
fn test_DRYRUN_COV_package_upgrade() {
    let op = PackageOperation::Upgrade {
        name: "nginx".to_string(),
        from_version: Some("1.20".to_string()),
        to_version: Some("1.24".to_string()),
    };
    assert_eq!(op.to_diff_line(), "~ nginx (1.20 -> 1.24)");
}

#[test]
fn test_DRYRUN_COV_package_upgrade_unknown_versions() {
    let op = PackageOperation::Upgrade {
        name: "nginx".to_string(),
        from_version: None,
        to_version: None,
    };
    assert_eq!(op.to_diff_line(), "~ nginx (? -> ?)");
}

#[test]
fn test_DRYRUN_COV_package_remove() {
    let op = PackageOperation::remove("old-pkg");
    assert_eq!(op.to_diff_line(), "- old-pkg");
}

// =============================================================================
// ServiceOperation edge cases
// =============================================================================

#[test]
fn test_DRYRUN_COV_service_disable() {
    let op = ServiceOperation::Disable {
        name: "apache2".to_string(),
    };
    assert_eq!(op.to_diff_line(), "- systemctl enable apache2");
}

#[test]
fn test_DRYRUN_COV_service_stop() {
    let op = ServiceOperation::Stop {
        name: "mysql".to_string(),
    };
    assert_eq!(op.to_diff_line(), "- systemctl start mysql");
}

#[test]
fn test_DRYRUN_COV_service_restart() {
    let op = ServiceOperation::Restart {
        name: "nginx".to_string(),
    };
    assert_eq!(op.to_diff_line(), "~ systemctl restart nginx");
}

#[test]
fn test_DRYRUN_COV_service_enable() {
    let op = ServiceOperation::Enable {
        name: "docker".to_string(),
    };
    assert_eq!(op.to_diff_line(), "+ systemctl enable docker");
}

#[test]
fn test_DRYRUN_COV_service_start() {
    let op = ServiceOperation::Start {
        name: "redis".to_string(),
    };
    assert_eq!(op.to_diff_line(), "+ systemctl start redis");
}

// =============================================================================
// UserGroupOperation edge cases
// =============================================================================

#[test]
fn test_DRYRUN_COV_remove_from_group() {
    let op = UserGroupOperation::RemoveFromGroup {
        user: "alice".to_string(),
        group: "sudo".to_string(),
    };
    assert_eq!(op.to_diff_line(), "- gpasswd -d alice sudo");
}

#[test]
fn test_DRYRUN_COV_create_user_no_groups() {
    let op = UserGroupOperation::CreateUser {
        name: "deploy".to_string(),
        groups: vec![],
    };
    assert_eq!(op.to_diff_line(), "+ useradd deploy");
}

#[test]
fn test_DRYRUN_COV_create_user_with_groups() {
    let op = UserGroupOperation::CreateUser {
        name: "deploy".to_string(),
        groups: vec!["docker".to_string(), "www-data".to_string()],
    };
    assert_eq!(op.to_diff_line(), "+ useradd -G docker,www-data deploy");
}

#[test]
fn test_DRYRUN_COV_create_group() {
    let op = UserGroupOperation::CreateGroup {
        name: "myapp".to_string(),
    };
    assert_eq!(op.to_diff_line(), "+ groupadd myapp");
}

#[test]
fn test_DRYRUN_COV_add_to_group() {
    let op = UserGroupOperation::AddToGroup {
        user: "bob".to_string(),
        group: "staff".to_string(),
    };
    assert_eq!(op.to_diff_line(), "+ usermod -aG staff bob");
}

// =============================================================================
// DryRunContext — simulate methods and accessors
// =============================================================================

#[test]
fn test_DRYRUN_COV_simulate_file_modify() {
    let mut ctx = DryRunContext::new();
    ctx.simulate_file_modify("/etc/config.yml", "old: val\n", "new: val\n");

    let summary = ctx.summary();
    assert_eq!(summary.files_modified, 1);
    assert_eq!(summary.files_created, 0);
}

#[test]
fn test_DRYRUN_COV_simulate_file_delete() {
    let mut ctx = DryRunContext::new();
    ctx.simulate_file_delete("/tmp/trash.txt", "garbage\n");

    let summary = ctx.summary();
    assert_eq!(summary.files_deleted, 1);
}

#[test]
fn test_DRYRUN_COV_simulate_service_start() {
    let mut ctx = DryRunContext::new();
    ctx.simulate_service_start("postgres");

    let ops = ctx.service_operations();
    assert_eq!(ops.len(), 1);
    assert!(matches!(&ops[0], ServiceOperation::Start { name } if name == "postgres"));
}

#[test]
fn test_DRYRUN_COV_file_changes_iterator() {
    let mut ctx = DryRunContext::new();
    ctx.simulate_file_write("/a.txt", "a", None);
    ctx.simulate_file_write("/b.txt", "b", None);

    let changes: Vec<_> = ctx.file_changes().collect();
    assert_eq!(changes.len(), 2);
}

#[test]
fn test_DRYRUN_COV_package_operations_accessor() {
    let mut ctx = DryRunContext::new();
    ctx.simulate_package_install("pkg1", Some("1.0"));
    ctx.simulate_package_install("pkg2", None);
    ctx.simulate_package_remove("old-pkg");

    let ops = ctx.package_operations();
    assert_eq!(ops.len(), 3);
}

#[test]
fn test_DRYRUN_COV_user_group_operations_accessor() {
    let mut ctx = DryRunContext::new();
    ctx.simulate_add_to_group("user1", "group1");
    ctx.simulate_add_to_group("user2", "group2");

    let ops = ctx.user_group_operations();
    assert_eq!(ops.len(), 2);
}

// =============================================================================
// DryRunSummary edge cases
// =============================================================================

#[test]
fn test_DRYRUN_COV_summary_has_changes_true_from_packages() {
    let summary = DryRunSummary {
        packages_installed: 1,
        ..Default::default()
    };
    assert!(summary.has_changes());
}

#[test]
fn test_DRYRUN_COV_summary_has_changes_true_from_services() {
    let summary = DryRunSummary {
        services_enabled: 1,
        ..Default::default()
    };
    assert!(summary.has_changes());
}

#[test]
fn test_DRYRUN_COV_summary_has_changes_true_from_users() {
    let summary = DryRunSummary {
        users_modified: 1,
        ..Default::default()
    };
    assert!(summary.has_changes());
}

#[test]
fn test_DRYRUN_COV_summary_has_changes_true_from_deletes() {
    let summary = DryRunSummary {
        files_deleted: 1,
        ..Default::default()
    };
    assert!(summary.has_changes());
}

#[test]
fn test_DRYRUN_COV_summary_has_changes_true_from_removes() {
    let summary = DryRunSummary {
        packages_removed: 1,
        ..Default::default()
    };
    assert!(summary.has_changes());
}

#[test]
fn test_DRYRUN_COV_summary_no_changes() {
    let summary = DryRunSummary::default();
    assert!(!summary.has_changes());
}

#[test]
fn test_DRYRUN_COV_summary_text_with_failures() {
    let summary = DryRunSummary {
        files_created: 1,
        steps_would_fail: 3,
        ..Default::default()
    };
    let text = summary.to_text();
    assert!(text.contains("3 step(s) would fail"));
}

#[test]
fn test_DRYRUN_COV_summary_text_no_failures() {
    let summary = DryRunSummary {
        files_modified: 2,
        packages_installed: 1,
        ..Default::default()
    };
    let text = summary.to_text();
    assert!(!text.contains("would fail"));
    assert!(text.contains("Files modified:     2"));
    assert!(text.contains("Packages installed: 1"));
}

#[test]
fn test_DRYRUN_COV_summary_text_all_fields() {
    let summary = DryRunSummary {
        files_created: 1,
        files_modified: 2,
        files_deleted: 3,
        packages_installed: 4,
        packages_removed: 5,
        services_enabled: 6,
        users_modified: 7,
        steps_would_fail: 0,
    };
    let text = summary.to_text();
    assert!(text.contains("Files created:      1"));
    assert!(text.contains("Files modified:     2"));
    assert!(text.contains("Files deleted:      3"));
    assert!(text.contains("Packages installed: 4"));
    assert!(text.contains("Packages removed:   5"));
    assert!(text.contains("Services enabled:   6"));
    assert!(text.contains("Users modified:     7"));
}

// =============================================================================
// DiffPreview sections rendering
// =============================================================================

#[test]
fn test_DRYRUN_COV_diff_preview_all_sections() {
    let mut ctx = DryRunContext::new();
    ctx.simulate_file_write("/etc/app.conf", "content\n", None);
    ctx.simulate_package_install("nginx", Some("1.24"));
    ctx.simulate_service_enable("nginx");
    ctx.simulate_add_to_group("deploy", "www-data");

    let preview = ctx.generate_diff();
    let text = preview.to_diff_text();

    assert!(text.contains("=== Filesystem Changes ==="));
    assert!(text.contains("=== Package Changes ==="));
    assert!(text.contains("=== Service Changes ==="));
    assert!(text.contains("=== User/Group Changes ==="));
}

#[test]
fn test_DRYRUN_COV_diff_preview_only_packages() {
    let mut ctx = DryRunContext::new();
    ctx.simulate_package_install("vim", None);

    let preview = ctx.generate_diff();
    let text = preview.to_diff_text();

    assert!(!text.contains("=== Filesystem Changes ==="));
    assert!(text.contains("=== Package Changes ==="));
    assert!(!text.contains("=== Service Changes ==="));
    assert!(!text.contains("=== User/Group Changes ==="));
}

#[test]
fn test_DRYRUN_COV_diff_preview_only_services() {
    let mut ctx = DryRunContext::new();
    ctx.simulate_service_start("redis");

    let preview = ctx.generate_diff();
    let text = preview.to_diff_text();

    assert!(!text.contains("=== Filesystem Changes ==="));
    assert!(!text.contains("=== Package Changes ==="));
    assert!(text.contains("=== Service Changes ==="));
}

#[test]
fn test_DRYRUN_COV_diff_preview_only_users() {
    let mut ctx = DryRunContext::new();
    ctx.simulate_add_to_group("alice", "docker");

    let preview = ctx.generate_diff();
    let text = preview.to_diff_text();

    assert!(!text.contains("=== Filesystem Changes ==="));
    assert!(!text.contains("=== Package Changes ==="));
    assert!(text.contains("=== User/Group Changes ==="));
}

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
