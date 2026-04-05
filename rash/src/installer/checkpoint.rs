//! Checkpoint System for Installer Framework (#106)
//!
//! Provides SQLite-based checkpoint storage for resumable installations.
//!
//! # Features
//!
//! - Resume from any failure point
//! - Track step status and state snapshots
//! - Store file state for rollback
//! - Verify hermetic mode consistency
//!
//! # Storage Schema
//!
//! - `installer_runs` - Overall run metadata
//! - `step_checkpoints` - Per-step status and state
//! - `state_files` - File state tracking for rollback

use crate::models::{Error, Result};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Status of an installer run
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    Running,
    Completed,
    Failed,
    Aborted,
}

impl RunStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Aborted => "aborted",
        }
    }

    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "running" => Some(Self::Running),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "aborted" => Some(Self::Aborted),
            _ => None,
        }
    }
}

/// Status of a step checkpoint
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

impl StepStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }

    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "running" => Some(Self::Running),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "skipped" => Some(Self::Skipped),
            _ => None,
        }
    }
}

/// Metadata for an installer run
#[derive(Debug, Clone)]
pub struct InstallerRun {
    /// Unique run identifier
    pub run_id: String,
    /// Name of the installer
    pub installer_name: String,
    /// Version of the installer
    pub installer_version: String,
    /// When the run started
    pub started_at: u64,
    /// When the run completed (if finished)
    pub completed_at: Option<u64>,
    /// Current status
    pub status: RunStatus,
    /// Whether running in hermetic mode
    pub hermetic_mode: bool,
    /// Lockfile hash (for hermetic mode)
    pub lockfile_hash: Option<String>,
}

impl InstallerRun {
    /// Create a new installer run
    pub fn new(installer_name: &str, installer_version: &str) -> Self {
        let run_id = generate_run_id();
        let started_at = current_timestamp();

        Self {
            run_id,
            installer_name: installer_name.to_string(),
            installer_version: installer_version.to_string(),
            started_at,
            completed_at: None,
            status: RunStatus::Running,
            hermetic_mode: false,
            lockfile_hash: None,
        }
    }

    /// Create a new hermetic installer run
    pub fn new_hermetic(
        installer_name: &str,
        installer_version: &str,
        lockfile_hash: &str,
    ) -> Self {
        let mut run = Self::new(installer_name, installer_version);
        run.hermetic_mode = true;
        run.lockfile_hash = Some(lockfile_hash.to_string());
        run
    }

    /// Mark the run as completed
    pub fn complete(&mut self) {
        self.status = RunStatus::Completed;
        self.completed_at = Some(current_timestamp());
    }

    /// Mark the run as failed
    pub fn fail(&mut self) {
        self.status = RunStatus::Failed;
        self.completed_at = Some(current_timestamp());
    }
}

/// Checkpoint for a single step
#[derive(Debug, Clone)]
pub struct StepCheckpoint {
    /// Run this checkpoint belongs to
    pub run_id: String,
    /// Step identifier
    pub step_id: String,
    /// Current status
    pub status: StepStatus,
    /// When the step started
    pub started_at: Option<u64>,
    /// When the step completed
    pub completed_at: Option<u64>,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    /// State snapshot as JSON
    pub state_snapshot: Option<String>,
    /// Output log
    pub output_log: Option<String>,
    /// Error message if failed
    pub error_message: Option<String>,
}

impl StepCheckpoint {
    /// Create a new pending step checkpoint
    pub fn new(run_id: &str, step_id: &str) -> Self {
        Self {
            run_id: run_id.to_string(),
            step_id: step_id.to_string(),
            status: StepStatus::Pending,
            started_at: None,
            completed_at: None,
            duration_ms: None,
            state_snapshot: None,
            output_log: None,
            error_message: None,
        }
    }

    /// Mark the step as running
    pub fn start(&mut self) {
        self.status = StepStatus::Running;
        self.started_at = Some(current_timestamp());
    }

    /// Mark the step as completed
    pub fn complete(&mut self, output: Option<String>) {
        self.status = StepStatus::Completed;
        self.completed_at = Some(current_timestamp());
        self.output_log = output;
        if let (Some(start), Some(end)) = (self.started_at, self.completed_at) {
            self.duration_ms = Some((end - start) * 1000);
        }
    }

    /// Mark the step as failed
    pub fn fail(&mut self, error: &str) {
        self.status = StepStatus::Failed;
        self.completed_at = Some(current_timestamp());
        self.error_message = Some(error.to_string());
        if let (Some(start), Some(end)) = (self.started_at, self.completed_at) {
            self.duration_ms = Some((end - start) * 1000);
        }
    }

    /// Mark the step as skipped
    pub fn skip(&mut self) {
        self.status = StepStatus::Skipped;
    }
}

/// Tracked file state for rollback
#[derive(Debug, Clone)]
pub struct StateFile {
    /// Run this file state belongs to
    pub run_id: String,
    /// Step that modified this file
    pub step_id: String,
    /// Path to the file
    pub file_path: PathBuf,
    /// SHA256 hash of content
    pub content_hash: String,
    /// When the backup was created
    pub backed_up_at: Option<u64>,
    /// Path to the backup file
    pub backup_path: Option<PathBuf>,
}

impl StateFile {
    /// Create a new state file record
    pub fn new(run_id: &str, step_id: &str, file_path: &Path, content_hash: &str) -> Self {
        Self {
            run_id: run_id.to_string(),
            step_id: step_id.to_string(),
            file_path: file_path.to_path_buf(),
            content_hash: content_hash.to_string(),
            backed_up_at: None,
            backup_path: None,
        }
    }

    /// Mark as backed up
    pub fn set_backup(&mut self, backup_path: &Path) {
        self.backed_up_at = Some(current_timestamp());
        self.backup_path = Some(backup_path.to_path_buf());
    }
}

/// Checkpoint storage manager
#[derive(Debug)]
pub struct CheckpointStore {
    /// Path to the checkpoint directory
    checkpoint_dir: PathBuf,
    /// Current run (if any)
    current_run: Option<InstallerRun>,
    /// Step checkpoints for current run
    steps: Vec<StepCheckpoint>,
    /// State files for current run
    state_files: Vec<StateFile>,
}

include!("checkpoint_checkpointstore.rs");
