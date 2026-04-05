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
