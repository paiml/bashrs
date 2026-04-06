
use super::*;

#[test]
fn test_DRYRUN_001_file_change_created() {
    let change = FileChange::created("/etc/config.txt", "new content\nline 2\n", Some(0o644));

    assert_eq!(change.change_type, FileChangeType::Created);
    assert!(change.before.is_none());
    assert!(change.after.is_some());

    let diff = change.to_diff();
    assert!(diff.contains("--- /dev/null"));
    assert!(diff.contains("+++ b/etc/config.txt"));
    assert!(diff.contains("+new content"));
}

#[test]
fn test_DRYRUN_002_file_change_modified() {
    let change = FileChange::modified("/etc/config.txt", "old\n", "new\n");

    assert_eq!(change.change_type, FileChangeType::Modified);

    let diff = change.to_diff();
    assert!(diff.contains("--- a/etc/config.txt"));
    assert!(diff.contains("+++ b/etc/config.txt"));
    assert!(diff.contains("-old"));
    assert!(diff.contains("+new"));
}

#[test]
fn test_DRYRUN_003_file_change_deleted() {
    let change = FileChange::deleted("/tmp/old.txt", "content\n");

    assert_eq!(change.change_type, FileChangeType::Deleted);

    let diff = change.to_diff();
    assert!(diff.contains("--- a/tmp/old.txt"));
    assert!(diff.contains("+++ /dev/null"));
    assert!(diff.contains("-content"));
}

#[test]
fn test_DRYRUN_004_package_operations() {
    let install = PackageOperation::install("docker-ce", Some("24.0.7"));
    let remove = PackageOperation::remove("docker.io");

    assert_eq!(install.to_diff_line(), "+ docker-ce (24.0.7)");
    assert_eq!(remove.to_diff_line(), "- docker.io");
}

#[test]
fn test_DRYRUN_005_service_operations() {
    let enable = ServiceOperation::Enable {
        name: "docker".to_string(),
    };
    let start = ServiceOperation::Start {
        name: "nginx".to_string(),
    };

    assert_eq!(enable.to_diff_line(), "+ systemctl enable docker");
    assert_eq!(start.to_diff_line(), "+ systemctl start nginx");
}

#[test]
fn test_DRYRUN_006_user_group_operations() {
    let add = UserGroupOperation::AddToGroup {
        user: "alice".to_string(),
        group: "docker".to_string(),
    };

    assert_eq!(add.to_diff_line(), "+ usermod -aG docker alice");
}

#[test]
fn test_DRYRUN_007_context_file_operations() {
    let mut ctx = DryRunContext::new();

    ctx.simulate_file_write("/etc/app/config.yaml", "key: value\n", Some(0o644));
    ctx.simulate_package_install("nginx", Some("1.24.0"));
    ctx.simulate_service_enable("nginx");
    ctx.simulate_add_to_group("deploy", "www-data");

    let summary = ctx.summary();
    assert_eq!(summary.files_created, 1);
    assert_eq!(summary.packages_installed, 1);
    assert_eq!(summary.services_enabled, 1);
    assert_eq!(summary.users_modified, 1);
}

#[test]
fn test_DRYRUN_008_generate_diff() {
    let mut ctx = DryRunContext::new();

    ctx.simulate_file_write(
        "/etc/docker/daemon.json",
        "{\n  \"storage-driver\": \"overlay2\"\n}\n",
        None,
    );
    ctx.simulate_package_install("docker-ce", Some("24.0.7"));
    ctx.simulate_package_remove("docker.io");
    ctx.simulate_service_enable("docker");

    let diff = ctx.generate_diff();

    assert_eq!(diff.file_changes.len(), 1);
    assert_eq!(diff.package_ops.len(), 2);
    assert_eq!(diff.service_ops.len(), 1);
}

#[test]
fn test_DRYRUN_009_diff_text_output() {
    let mut ctx = DryRunContext::new();

    ctx.simulate_file_write("/etc/test.conf", "content\n", None);
    ctx.simulate_package_install("test-pkg", None);

    let preview = ctx.generate_diff();
    let text = preview.to_diff_text();

    assert!(text.contains("=== Filesystem Changes ==="));
    assert!(text.contains("=== Package Changes ==="));
    assert!(text.contains("+++ b/etc/test.conf"));
    assert!(text.contains("+ test-pkg"));
}

#[test]
fn test_DRYRUN_010_summary_text() {
    let summary = DryRunSummary {
        files_created: 2,
        files_modified: 1,
        files_deleted: 0,
        packages_installed: 5,
        packages_removed: 2,
        services_enabled: 1,
        users_modified: 1,
        steps_would_fail: 0,
    };

    let text = summary.to_text();
    assert!(text.contains("Files created:      2"));
    assert!(text.contains("Packages installed: 5"));
}

#[test]
fn test_DRYRUN_011_simulation_log() {
    let mut ctx = DryRunContext::new();

    ctx.log_step("step-1", "Install Package", "Would install docker-ce");
    ctx.log_step_failure("step-2", "Verify", "File not found");

    let log = ctx.simulation_log();
    assert_eq!(log.len(), 2);
    assert!(log[0].would_succeed);
    assert!(!log[1].would_succeed);

    let summary = ctx.summary();
    assert_eq!(summary.steps_would_fail, 1);
}

#[test]
fn test_DRYRUN_012_empty_preview() {
    let ctx = DryRunContext::new();
    let summary = ctx.summary();

    assert!(!summary.has_changes());

    let preview = ctx.generate_diff();
    assert!(preview.is_empty());
}
