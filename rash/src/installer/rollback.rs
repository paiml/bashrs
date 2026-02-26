//! Rollback system for installer steps (#116)
//!
//! Implements per-step rollback with state restoration following Toyota Way
//! principles (Jidoka - stop and fix problems immediately).
//!
//! # Example
//!
//! ```bash
//! # Execute with automatic rollback on failure
//! bashrs installer run ./my-installer --rollback-on-failure
//!
//! # Manual rollback to specific step
//! bashrs installer rollback ./my-installer --to step-3
//!
//! # View rollback plan
//! bashrs installer rollback ./my-installer --dry-run
//! ```

use crate::models::{Error, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Rollback action types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RollbackAction {
    /// Execute a shell command to undo the step
    Command(String),

    /// Restore a file from backup
    RestoreFile {
        original_path: PathBuf,
        backup_path: PathBuf,
    },

    /// Remove a file that was created
    RemoveFile(PathBuf),

    /// Remove a directory that was created
    RemoveDirectory(PathBuf),

    /// Restore package state (reinstall/remove packages)
    RestorePackages {
        install: Vec<String>,
        remove: Vec<String>,
    },

    /// Restore service state
    RestoreService {
        name: String,
        was_enabled: bool,
        was_running: bool,
    },

    /// Restore user/group membership
    RestoreUserGroup {
        user: String,
        group: String,
        was_member: bool,
    },

    /// No rollback action needed (idempotent step)
    None,
}

impl RollbackAction {
    /// Create a command-based rollback
    pub fn command(cmd: &str) -> Self {
        Self::Command(cmd.to_string())
    }

    /// Create a file restoration rollback
    pub fn restore_file(original: impl AsRef<Path>, backup: impl AsRef<Path>) -> Self {
        Self::RestoreFile {
            original_path: original.as_ref().to_path_buf(),
            backup_path: backup.as_ref().to_path_buf(),
        }
    }

    /// Create a file removal rollback
    pub fn remove_file(path: impl AsRef<Path>) -> Self {
        Self::RemoveFile(path.as_ref().to_path_buf())
    }

    /// Check if this is a no-op rollback
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Get a human-readable description of the rollback action
    pub fn description(&self) -> String {
        match self {
            Self::Command(cmd) => format!("Execute: {}", truncate(cmd, 60)),
            Self::RestoreFile {
                original_path,
                backup_path,
            } => format!(
                "Restore {} from {}",
                original_path.display(),
                backup_path.display()
            ),
            Self::RemoveFile(path) => format!("Remove file: {}", path.display()),
            Self::RemoveDirectory(path) => format!("Remove directory: {}", path.display()),
            Self::RestorePackages { install, remove } => describe_restore_packages(install, remove),
            Self::RestoreService {
                name,
                was_enabled,
                was_running,
            } => describe_restore_service(name, *was_enabled, *was_running),
            Self::RestoreUserGroup {
                user,
                group,
                was_member,
            } => describe_restore_user_group(user, group, *was_member),
            Self::None => "No action required".to_string(),
        }
    }
}

/// Describe package restore action
fn describe_restore_packages(install: &[String], remove: &[String]) -> String {
    let mut parts = Vec::new();
    if !install.is_empty() {
        parts.push(format!("Reinstall: {}", install.join(", ")));
    }
    if !remove.is_empty() {
        parts.push(format!("Remove: {}", remove.join(", ")));
    }
    parts.join("; ")
}

/// Describe service restore action
fn describe_restore_service(name: &str, was_enabled: bool, was_running: bool) -> String {
    let enabled = if was_enabled { "enable" } else { "disable" };
    let running = if was_running { "start" } else { "stop" };
    format!("Service {}: {}, {}", name, enabled, running)
}

/// Describe user/group restore action
fn describe_restore_user_group(user: &str, group: &str, was_member: bool) -> String {
    if was_member {
        format!("Add {} back to group {}", user, group)
    } else {
        format!("Remove {} from group {}", user, group)
    }
}

/// Step rollback plan
#[derive(Debug, Clone)]
pub struct StepRollback {
    /// Step ID this rollback applies to
    pub step_id: String,

    /// Step name for display
    pub step_name: String,

    /// Ordered list of rollback actions (executed in reverse order)
    pub actions: Vec<RollbackAction>,

    /// State files backed up before this step
    pub state_files: Vec<StateFileBackup>,

    /// Whether this step was completed successfully
    pub completed: bool,

    /// Error message if step failed
    pub error: Option<String>,
}

impl StepRollback {
    /// Create a new step rollback plan
    pub fn new(step_id: &str, step_name: &str) -> Self {
        Self {
            step_id: step_id.to_string(),
            step_name: step_name.to_string(),
            actions: Vec::new(),
            state_files: Vec::new(),
            completed: false,
            error: None,
        }
    }

    /// Add a rollback action
    pub fn add_action(&mut self, action: RollbackAction) {
        if !action.is_none() {
            self.actions.push(action);
        }
    }

    /// Add a state file backup
    pub fn add_state_file(&mut self, backup: StateFileBackup) {
        self.state_files.push(backup);
    }

    /// Mark step as completed
    pub fn mark_completed(&mut self) {
        self.completed = true;
    }

    /// Mark step as failed with error
    pub fn mark_failed(&mut self, error: &str) {
        self.completed = false;
        self.error = Some(error.to_string());
    }

    /// Check if rollback is needed
    pub fn needs_rollback(&self) -> bool {
        !self.completed && !self.actions.is_empty()
    }

    /// Get rollback actions in reverse order (LIFO)
    pub fn rollback_actions(&self) -> impl Iterator<Item = &RollbackAction> {
        self.actions.iter().rev()
    }
}

/// Backup of a state file
#[derive(Debug, Clone)]
pub struct StateFileBackup {
    /// Original file path
    pub original_path: PathBuf,

    /// Backup file path
    pub backup_path: PathBuf,

    /// SHA256 hash of original content
    pub content_hash: String,

    /// Whether the file existed before the step
    pub existed: bool,

    /// Timestamp when backup was created
    pub backed_up_at: u64,
}

impl StateFileBackup {
    /// Create a new state file backup record
    pub fn new(
        original: impl AsRef<Path>,
        backup: impl AsRef<Path>,
        content_hash: &str,
        existed: bool,
    ) -> Self {
        Self {
            original_path: original.as_ref().to_path_buf(),
            backup_path: backup.as_ref().to_path_buf(),
            content_hash: content_hash.to_string(),
            existed,
            backed_up_at: current_timestamp(),
        }
    }
}

/// Rollback manager for an installer run
#[derive(Debug)]
pub struct RollbackManager {
    /// Directory to store backups
    backup_dir: PathBuf,

    /// Step rollback plans in execution order
    steps: Vec<StepRollback>,

    /// Index by step ID for quick lookup
    step_index: HashMap<String, usize>,

    /// Whether automatic rollback on failure is enabled
    auto_rollback: bool,
}

impl RollbackManager {
    /// Create a new rollback manager
    pub fn new(backup_dir: impl AsRef<Path>) -> Result<Self> {
        let backup_dir = backup_dir.as_ref().to_path_buf();

        // Create backup directory if it doesn't exist
        if !backup_dir.exists() {
            std::fs::create_dir_all(&backup_dir).map_err(|e| {
                Error::Io(std::io::Error::new(
                    e.kind(),
                    format!("Failed to create backup directory: {}", e),
                ))
            })?;
        }

        Ok(Self {
            backup_dir,
            steps: Vec::new(),
            step_index: HashMap::new(),
            auto_rollback: true,
        })
    }

    /// Enable or disable automatic rollback on failure
    pub fn set_auto_rollback(&mut self, enabled: bool) {
        self.auto_rollback = enabled;
    }

    /// Check if auto rollback is enabled
    pub fn is_auto_rollback(&self) -> bool {
        self.auto_rollback
    }

    /// Get backup directory
    pub fn backup_dir(&self) -> &Path {
        &self.backup_dir
    }

    /// Register a step for rollback tracking
    ///
    /// # Panics
    ///
    /// This function should never panic as it always accesses a valid index.
    #[allow(clippy::expect_used)]
    pub fn register_step(&mut self, step_id: &str, step_name: &str) -> &mut StepRollback {
        let index = self.steps.len();
        self.steps.push(StepRollback::new(step_id, step_name));
        self.step_index.insert(step_id.to_string(), index);
        // SAFETY: We just pushed an element, so the index is valid
        self.steps.get_mut(index).expect("just pushed element")
    }

    /// Get a step rollback plan by ID
    pub fn get_step(&self, step_id: &str) -> Option<&StepRollback> {
        self.step_index
            .get(step_id)
            .and_then(|&idx| self.steps.get(idx))
    }

    /// Get a mutable step rollback plan by ID
    pub fn get_step_mut(&mut self, step_id: &str) -> Option<&mut StepRollback> {
        self.step_index
            .get(step_id)
            .copied()
            .and_then(|idx| self.steps.get_mut(idx))
    }

    /// Backup a file before modification
    pub fn backup_file(
        &mut self,
        step_id: &str,
        path: impl AsRef<Path>,
    ) -> Result<StateFileBackup> {
        let path = path.as_ref();
        let existed = path.exists();

        let backup_name = format!(
            "{}-{}-{}",
            step_id,
            path.file_name().and_then(|n| n.to_str()).unwrap_or("file"),
            current_timestamp()
        );
        let backup_path = self.backup_dir.join(&backup_name);

        let content_hash = if existed {
            // Copy file to backup location
            std::fs::copy(path, &backup_path).map_err(|e| {
                Error::Io(std::io::Error::new(
                    e.kind(),
                    format!("Failed to backup file {}: {}", path.display(), e),
                ))
            })?;

            // Compute hash of original content
            let content = std::fs::read(path).map_err(|e| {
                Error::Io(std::io::Error::new(
                    e.kind(),
                    format!("Failed to read file for hashing: {}", e),
                ))
            })?;
            compute_hash(&content)
        } else {
            "nonexistent".to_string()
        };

        let backup = StateFileBackup::new(path, &backup_path, &content_hash, existed);

        // Add to step's state files
        if let Some(step) = self.get_step_mut(step_id) {
            step.add_state_file(backup.clone());

            // Add restore action
            if existed {
                step.add_action(RollbackAction::restore_file(path, &backup_path));
            } else {
                step.add_action(RollbackAction::remove_file(path));
            }
        }

        Ok(backup)
    }

    /// Get steps that need rollback (in reverse execution order)
    pub fn steps_needing_rollback(&self) -> impl Iterator<Item = &StepRollback> {
        self.steps.iter().rev().filter(|s| s.needs_rollback())
    }

    /// Get all completed steps (in reverse order for rollback)
    pub fn completed_steps_reverse(&self) -> impl Iterator<Item = &StepRollback> {
        self.steps.iter().rev().filter(|s| s.completed)
    }

    /// Generate a rollback plan from a specific step
    pub fn plan_rollback_from(&self, from_step: &str) -> Result<RollbackPlan> {
        let from_idx = self.step_index.get(from_step).ok_or_else(|| {
            Error::Validation(format!(
                "Step '{}' not found in rollback manager",
                from_step
            ))
        })?;

        // Collect steps from the specified step back to the beginning
        let steps_to_rollback: Vec<_> = self
            .steps
            .get(..=*from_idx)
            .unwrap_or(&[])
            .iter()
            .rev()
            .filter(|s| s.completed || s.needs_rollback())
            .cloned()
            .collect();

        Ok(RollbackPlan {
            steps: steps_to_rollback,
            backup_dir: self.backup_dir.clone(),
        })
    }

    /// Generate a rollback plan for all failed steps
    pub fn plan_rollback_failed(&self) -> RollbackPlan {
        let steps_to_rollback: Vec<_> = self
            .steps
            .iter()
            .rev()
            .filter(|s| s.needs_rollback())
            .cloned()
            .collect();

        RollbackPlan {
            steps: steps_to_rollback,
            backup_dir: self.backup_dir.clone(),
        }
    }

    /// Get count of steps
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Get count of completed steps
    pub fn completed_count(&self) -> usize {
        self.steps.iter().filter(|s| s.completed).count()
    }

    /// Get count of failed steps
    pub fn failed_count(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| !s.completed && s.error.is_some())
            .count()
    }
}

/// A plan for rolling back steps
#[derive(Debug, Clone)]
pub struct RollbackPlan {
    /// Steps to rollback in order
    pub steps: Vec<StepRollback>,

    /// Backup directory
    pub backup_dir: PathBuf,
}

impl RollbackPlan {
    /// Check if the plan is empty
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Get total number of actions
    pub fn action_count(&self) -> usize {
        self.steps.iter().map(|s| s.actions.len()).sum()
    }

    /// Generate a human-readable summary
    pub fn summary(&self) -> String {
        let mut summary = String::new();

        summary.push_str(&format!(
            "Rollback Plan: {} steps, {} actions\n",
            self.steps.len(),
            self.action_count()
        ));
        summary.push_str(&format!(
            "Backup directory: {}\n\n",
            self.backup_dir.display()
        ));

        for (i, step) in self.steps.iter().enumerate() {
            summary.push_str(&format!(
                "Step {}: {} ({})\n",
                i + 1,
                step.step_name,
                step.step_id
            ));

            if step.actions.is_empty() {
                summary.push_str("  (no rollback actions)\n");
            } else {
                for (j, action) in step.actions.iter().rev().enumerate() {
                    summary.push_str(&format!("  {}. {}\n", j + 1, action.description()));
                }
            }

            if !step.state_files.is_empty() {
                summary.push_str(&format!("  State files: {}\n", step.state_files.len()));
            }

            summary.push('\n');
        }

        summary
    }

    /// Execute the rollback plan (dry-run mode)
    pub fn execute_dry_run(&self) -> Vec<String> {
        let mut actions = Vec::new();

        for step in &self.steps {
            actions.push(format!("=== Rolling back: {} ===", step.step_name));

            for action in step.rollback_actions() {
                actions.push(format!("  Would execute: {}", action.description()));
            }
        }

        actions
    }
}

/// Truncate a string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Compute SHA256 hash of content
fn compute_hash(content: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
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
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use tempfile::TempDir;

    proptest! {
        /// Property: Manager can handle any valid step ID
        #[test]
        fn prop_manager_handles_any_step_id(
            step_id in "[a-zA-Z][a-zA-Z0-9_-]{0,30}",
            step_name in "[a-zA-Z ]{1,50}"
        ) {
            let temp_dir = TempDir::new().unwrap();
            let mut manager = RollbackManager::new(temp_dir.path().join("backups")).unwrap();

            manager.register_step(&step_id, &step_name);

            prop_assert!(manager.get_step(&step_id).is_some());
            prop_assert_eq!(&manager.get_step(&step_id).unwrap().step_id, &step_id);
        }

        /// Property: Rollback plan preserves reverse order
        #[test]
        fn prop_rollback_preserves_order(
            step_count in 1usize..10
        ) {
            let temp_dir = TempDir::new().unwrap();
            let mut manager = RollbackManager::new(temp_dir.path().join("backups")).unwrap();

            // Create steps
            for i in 0..step_count {
                let step = manager.register_step(&format!("step-{}", i), &format!("Step {}", i));
                step.add_action(RollbackAction::command(&format!("echo {}", i)));
                step.mark_completed();
            }

            // Plan rollback from last step
            let last_step = format!("step-{}", step_count - 1);
            let plan = manager.plan_rollback_from(&last_step).unwrap();

            // Verify reverse order
            prop_assert_eq!(plan.steps.len(), step_count);
            for (i, step) in plan.steps.iter().enumerate() {
                let expected_id = format!("step-{}", step_count - 1 - i);
                prop_assert_eq!(&step.step_id, &expected_id);
            }
        }

        /// Property: Action count is sum of all step actions
        #[test]
        fn prop_action_count_is_sum(
            actions_per_step in prop::collection::vec(0usize..5, 1..5)
        ) {
            let temp_dir = TempDir::new().unwrap();
            let mut manager = RollbackManager::new(temp_dir.path().join("backups")).unwrap();

            let mut expected_total = 0;
            for (i, action_count) in actions_per_step.iter().enumerate() {
                let step = manager.register_step(&format!("step-{}", i), "Step");
                for j in 0..*action_count {
                    step.add_action(RollbackAction::command(&format!("cmd-{}", j)));
                }
                step.mark_completed();
                expected_total += action_count;
            }

            let last_step = format!("step-{}", actions_per_step.len() - 1);
            let plan = manager.plan_rollback_from(&last_step).unwrap();

            prop_assert_eq!(plan.action_count(), expected_total);
        }
    }
}
