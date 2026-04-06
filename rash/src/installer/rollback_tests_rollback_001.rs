
use super::*;
use tempfile::TempDir;

#[test]
fn test_ROLLBACK_001_manager_creation() {
    let temp_dir = TempDir::new().unwrap();
    let manager = RollbackManager::new(temp_dir.path().join("backups")).unwrap();

    assert!(manager.backup_dir().exists());
    assert!(manager.is_auto_rollback());
    assert_eq!(manager.step_count(), 0);
}

#[test]
fn test_ROLLBACK_002_register_step() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = RollbackManager::new(temp_dir.path().join("backups")).unwrap();

    manager.register_step("step-1", "First Step");
    manager.register_step("step-2", "Second Step");

    assert_eq!(manager.step_count(), 2);
    assert!(manager.get_step("step-1").is_some());
    assert!(manager.get_step("step-2").is_some());
    assert!(manager.get_step("step-3").is_none());
}

#[test]
fn test_ROLLBACK_003_add_rollback_action() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = RollbackManager::new(temp_dir.path().join("backups")).unwrap();

    let step = manager.register_step("step-1", "Test Step");
    step.add_action(RollbackAction::command("rm -f /tmp/test"));
    step.add_action(RollbackAction::remove_file("/tmp/created"));

    let step = manager.get_step("step-1").unwrap();
    assert_eq!(step.actions.len(), 2);
}

#[test]
fn test_ROLLBACK_004_step_completion() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = RollbackManager::new(temp_dir.path().join("backups")).unwrap();

    let step = manager.register_step("step-1", "Test Step");
    step.add_action(RollbackAction::command("echo rollback"));
    assert!(!step.completed);
    assert!(step.needs_rollback()); // Has actions but not completed

    step.mark_completed();
    assert!(step.completed);
    assert!(!step.needs_rollback()); // Completed, no rollback needed
}

#[test]
fn test_ROLLBACK_005_step_failure() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = RollbackManager::new(temp_dir.path().join("backups")).unwrap();

    let step = manager.register_step("step-1", "Test Step");
    step.add_action(RollbackAction::command("echo rollback"));
    step.mark_failed("Command failed with exit code 1");

    let step = manager.get_step("step-1").unwrap();
    assert!(!step.completed);
    assert!(step.needs_rollback());
    assert_eq!(
        step.error,
        Some("Command failed with exit code 1".to_string())
    );
}

#[test]
fn test_ROLLBACK_006_backup_file() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = RollbackManager::new(temp_dir.path().join("backups")).unwrap();

    // Create a test file
    let test_file = temp_dir.path().join("test.txt");
    std::fs::write(&test_file, "original content").unwrap();

    manager.register_step("step-1", "Test Step");
    let backup = manager.backup_file("step-1", &test_file).unwrap();

    assert!(backup.existed);
    assert!(backup.backup_path.exists());
    assert_eq!(
        std::fs::read_to_string(&backup.backup_path).unwrap(),
        "original content"
    );

    // Verify rollback action was added
    let step = manager.get_step("step-1").unwrap();
    assert_eq!(step.actions.len(), 1);
    assert!(matches!(
        &step.actions[0],
        RollbackAction::RestoreFile { .. }
    ));
}

#[test]
fn test_ROLLBACK_007_backup_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = RollbackManager::new(temp_dir.path().join("backups")).unwrap();

    let test_file = temp_dir.path().join("nonexistent.txt");

    manager.register_step("step-1", "Test Step");
    let backup = manager.backup_file("step-1", &test_file).unwrap();

    assert!(!backup.existed);
    assert_eq!(backup.content_hash, "nonexistent");

    // Verify remove action was added
    let step = manager.get_step("step-1").unwrap();
    assert_eq!(step.actions.len(), 1);
    assert!(matches!(&step.actions[0], RollbackAction::RemoveFile(_)));
}

#[test]
fn test_ROLLBACK_008_plan_rollback_from_step() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = RollbackManager::new(temp_dir.path().join("backups")).unwrap();

    // Register and complete steps
    let step = manager.register_step("step-1", "First");
    step.add_action(RollbackAction::command("echo 1"));
    step.mark_completed();

    let step = manager.register_step("step-2", "Second");
    step.add_action(RollbackAction::command("echo 2"));
    step.mark_completed();

    let step = manager.register_step("step-3", "Third");
    step.add_action(RollbackAction::command("echo 3"));
    // Not completed

    // Plan rollback from step-2
    let plan = manager.plan_rollback_from("step-2").unwrap();

    // Should include step-2 and step-1 (reverse order)
    assert_eq!(plan.steps.len(), 2);
    assert_eq!(plan.steps[0].step_id, "step-2");
    assert_eq!(plan.steps[1].step_id, "step-1");
}

#[test]
fn test_ROLLBACK_009_plan_summary() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = RollbackManager::new(temp_dir.path().join("backups")).unwrap();

    let step = manager.register_step("step-1", "Install Package");
    step.add_action(RollbackAction::RestorePackages {
        install: vec!["pkg-old".to_string()],
        remove: vec!["pkg-new".to_string()],
    });
    step.mark_completed();

    let plan = manager.plan_rollback_from("step-1").unwrap();
    let summary = plan.summary();

    assert!(summary.contains("Install Package"));
    assert!(summary.contains("Reinstall: pkg-old"));
    assert!(summary.contains("Remove: pkg-new"));
}

#[test]
fn test_ROLLBACK_010_action_descriptions() {
    let actions = [
        RollbackAction::Command("rm -f /tmp/test".to_string()),
        RollbackAction::RestoreFile {
            original_path: PathBuf::from("/etc/config"),
            backup_path: PathBuf::from("/backup/config"),
        },
        RollbackAction::RemoveFile(PathBuf::from("/tmp/created")),
        RollbackAction::RemoveDirectory(PathBuf::from("/opt/app")),
        RollbackAction::RestoreService {
            name: "nginx".to_string(),
            was_enabled: true,
            was_running: false,
        },
        RollbackAction::RestoreUserGroup {
            user: "alice".to_string(),
            group: "docker".to_string(),
            was_member: false,
        },
        RollbackAction::None,
    ];

    let descriptions: Vec<_> = actions.iter().map(|a| a.description()).collect();

    assert!(descriptions[0].contains("Execute: rm -f"));
    assert!(descriptions[1].contains("Restore /etc/config"));
    assert!(descriptions[2].contains("Remove file: /tmp/created"));
    assert!(descriptions[3].contains("Remove directory: /opt/app"));
    assert!(descriptions[4].contains("Service nginx: enable, stop"));
    assert!(descriptions[5].contains("Remove alice from group docker"));
    assert!(descriptions[6].contains("No action required"));
}

#[test]
fn test_ROLLBACK_011_rollback_actions_reverse_order() {
    let mut step = StepRollback::new("step-1", "Test");
    step.add_action(RollbackAction::command("first"));
    step.add_action(RollbackAction::command("second"));
    step.add_action(RollbackAction::command("third"));

    let actions: Vec<_> = step.rollback_actions().collect();

    // Should be in reverse order (LIFO)
    assert!(matches!(&actions[0], RollbackAction::Command(s) if s == "third"));
    assert!(matches!(&actions[1], RollbackAction::Command(s) if s == "second"));
    assert!(matches!(&actions[2], RollbackAction::Command(s) if s == "first"));
}

#[test]
fn test_ROLLBACK_012_counts() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = RollbackManager::new(temp_dir.path().join("backups")).unwrap();

    let step = manager.register_step("step-1", "First");
    step.mark_completed();

    let step = manager.register_step("step-2", "Second");
    step.mark_completed();

    let step = manager.register_step("step-3", "Third");
    step.mark_failed("Error");

    assert_eq!(manager.step_count(), 3);
    assert_eq!(manager.completed_count(), 2);
    assert_eq!(manager.failed_count(), 1);
}
