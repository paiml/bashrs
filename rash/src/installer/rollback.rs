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

include!("rollback_rollbackplan.rs");
