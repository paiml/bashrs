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

impl CheckpointStore {
    /// Create a new checkpoint store
    pub fn new(checkpoint_dir: &Path) -> Result<Self> {
        // Create checkpoint directory if it doesn't exist
        std::fs::create_dir_all(checkpoint_dir).map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to create checkpoint directory: {}", e),
            ))
        })?;

        Ok(Self {
            checkpoint_dir: checkpoint_dir.to_path_buf(),
            current_run: None,
            steps: Vec::new(),
            state_files: Vec::new(),
        })
    }

    /// Start a new installer run
    pub fn start_run(&mut self, installer_name: &str, installer_version: &str) -> Result<String> {
        let run = InstallerRun::new(installer_name, installer_version);
        let run_id = run.run_id.clone();
        self.current_run = Some(run);
        self.steps.clear();
        self.state_files.clear();
        self.save()?;
        Ok(run_id)
    }

    /// Start a hermetic installer run
    pub fn start_hermetic_run(
        &mut self,
        installer_name: &str,
        installer_version: &str,
        lockfile_hash: &str,
    ) -> Result<String> {
        let run = InstallerRun::new_hermetic(installer_name, installer_version, lockfile_hash);
        let run_id = run.run_id.clone();
        self.current_run = Some(run);
        self.steps.clear();
        self.state_files.clear();
        self.save()?;
        Ok(run_id)
    }

    /// Get the current run ID
    pub fn current_run_id(&self) -> Option<&str> {
        self.current_run.as_ref().map(|r| r.run_id.as_str())
    }

    /// Add a step checkpoint
    pub fn add_step(&mut self, step_id: &str) -> Result<()> {
        let run_id = self
            .current_run
            .as_ref()
            .ok_or_else(|| Error::Validation("No active run".to_string()))?
            .run_id
            .clone();

        let checkpoint = StepCheckpoint::new(&run_id, step_id);
        self.steps.push(checkpoint);
        self.save()
    }

    /// Start a step
    pub fn start_step(&mut self, step_id: &str) -> Result<()> {
        let step = self
            .steps
            .iter_mut()
            .find(|s| s.step_id == step_id)
            .ok_or_else(|| Error::Validation(format!("Step not found: {}", step_id)))?;

        step.start();
        self.save()
    }

    /// Complete a step
    pub fn complete_step(&mut self, step_id: &str, output: Option<String>) -> Result<()> {
        let step = self
            .steps
            .iter_mut()
            .find(|s| s.step_id == step_id)
            .ok_or_else(|| Error::Validation(format!("Step not found: {}", step_id)))?;

        step.complete(output);
        self.save()
    }

    /// Fail a step
    pub fn fail_step(&mut self, step_id: &str, error: &str) -> Result<()> {
        let step = self
            .steps
            .iter_mut()
            .find(|s| s.step_id == step_id)
            .ok_or_else(|| Error::Validation(format!("Step not found: {}", step_id)))?;

        step.fail(error);

        // Also mark the run as failed
        if let Some(ref mut run) = self.current_run {
            run.fail();
        }

        self.save()
    }

    /// Complete the run
    pub fn complete_run(&mut self) -> Result<()> {
        if let Some(ref mut run) = self.current_run {
            run.complete();
        }
        self.save()
    }

    /// Get the last successful step
    pub fn last_successful_step(&self) -> Option<&StepCheckpoint> {
        self.steps
            .iter()
            .rev()
            .find(|s| s.status == StepStatus::Completed)
    }

    /// Get step by ID
    pub fn get_step(&self, step_id: &str) -> Option<&StepCheckpoint> {
        self.steps.iter().find(|s| s.step_id == step_id)
    }

    /// Get all steps
    pub fn steps(&self) -> &[StepCheckpoint] {
        &self.steps
    }

    /// Track a state file
    pub fn track_file(
        &mut self,
        step_id: &str,
        file_path: &Path,
        content_hash: &str,
    ) -> Result<()> {
        let run_id = self
            .current_run
            .as_ref()
            .ok_or_else(|| Error::Validation("No active run".to_string()))?
            .run_id
            .clone();

        let state_file = StateFile::new(&run_id, step_id, file_path, content_hash);
        self.state_files.push(state_file);
        self.save()
    }

    /// Get state files for a step
    pub fn state_files_for_step(&self, step_id: &str) -> Vec<&StateFile> {
        self.state_files
            .iter()
            .filter(|sf| sf.step_id == step_id)
            .collect()
    }

    /// Check if in hermetic mode
    pub fn is_hermetic(&self) -> bool {
        self.current_run.as_ref().is_some_and(|r| r.hermetic_mode)
    }

    /// Verify hermetic mode consistency
    pub fn verify_hermetic_consistency(&self, current_lockfile_hash: &str) -> Result<()> {
        if let Some(ref run) = self.current_run {
            if run.hermetic_mode {
                if let Some(ref saved_hash) = run.lockfile_hash {
                    if saved_hash != current_lockfile_hash {
                        return Err(Error::Validation(format!(
                            "Lockfile drift detected: checkpoint={}, current={}",
                            saved_hash, current_lockfile_hash
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    /// Save checkpoint to disk
    fn save(&self) -> Result<()> {
        // For now, save as JSON (SQLite integration in Phase 2.1)
        let checkpoint_file = self.checkpoint_dir.join("checkpoint.json");

        let data = CheckpointData {
            run: self.current_run.clone(),
            steps: self.steps.clone(),
            state_files: self.state_files.clone(),
        };

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| Error::Validation(format!("Failed to serialize checkpoint: {}", e)))?;

        std::fs::write(&checkpoint_file, json).map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to write checkpoint: {}", e),
            ))
        })
    }

    /// Load checkpoint from disk
    pub fn load(checkpoint_dir: &Path) -> Result<Self> {
        let checkpoint_file = checkpoint_dir.join("checkpoint.json");

        if !checkpoint_file.exists() {
            return Self::new(checkpoint_dir);
        }

        let json = std::fs::read_to_string(&checkpoint_file).map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read checkpoint: {}", e),
            ))
        })?;

        let data: CheckpointData = serde_json::from_str(&json)
            .map_err(|e| Error::Validation(format!("Failed to parse checkpoint: {}", e)))?;

        Ok(Self {
            checkpoint_dir: checkpoint_dir.to_path_buf(),
            current_run: data.run,
            steps: data.steps,
            state_files: data.state_files,
        })
    }
}

/// Serializable checkpoint data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CheckpointData {
    run: Option<InstallerRun>,
    steps: Vec<StepCheckpoint>,
    state_files: Vec<StateFile>,
}

// Implement serde for our types
impl serde::Serialize for InstallerRun {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("InstallerRun", 8)?;
        state.serialize_field("run_id", &self.run_id)?;
        state.serialize_field("installer_name", &self.installer_name)?;
        state.serialize_field("installer_version", &self.installer_version)?;
        state.serialize_field("started_at", &self.started_at)?;
        state.serialize_field("completed_at", &self.completed_at)?;
        state.serialize_field("status", &self.status.as_str())?;
        state.serialize_field("hermetic_mode", &self.hermetic_mode)?;
        state.serialize_field("lockfile_hash", &self.lockfile_hash)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for InstallerRun {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct RunHelper {
            run_id: String,
            installer_name: String,
            installer_version: String,
            started_at: u64,
            completed_at: Option<u64>,
            status: String,
            hermetic_mode: bool,
            lockfile_hash: Option<String>,
        }

        let helper = RunHelper::deserialize(deserializer)?;
        let status = RunStatus::parse(&helper.status)
            .ok_or_else(|| serde::de::Error::custom("Invalid status"))?;

        Ok(InstallerRun {
            run_id: helper.run_id,
            installer_name: helper.installer_name,
            installer_version: helper.installer_version,
            started_at: helper.started_at,
            completed_at: helper.completed_at,
            status,
            hermetic_mode: helper.hermetic_mode,
            lockfile_hash: helper.lockfile_hash,
        })
    }
}

impl serde::Serialize for StepCheckpoint {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("StepCheckpoint", 9)?;
        state.serialize_field("run_id", &self.run_id)?;
        state.serialize_field("step_id", &self.step_id)?;
        state.serialize_field("status", &self.status.as_str())?;
        state.serialize_field("started_at", &self.started_at)?;
        state.serialize_field("completed_at", &self.completed_at)?;
        state.serialize_field("duration_ms", &self.duration_ms)?;
        state.serialize_field("state_snapshot", &self.state_snapshot)?;
        state.serialize_field("output_log", &self.output_log)?;
        state.serialize_field("error_message", &self.error_message)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for StepCheckpoint {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct StepHelper {
            run_id: String,
            step_id: String,
            status: String,
            started_at: Option<u64>,
            completed_at: Option<u64>,
            duration_ms: Option<u64>,
            state_snapshot: Option<String>,
            output_log: Option<String>,
            error_message: Option<String>,
        }

        let helper = StepHelper::deserialize(deserializer)?;
        let status = StepStatus::parse(&helper.status)
            .ok_or_else(|| serde::de::Error::custom("Invalid status"))?;

        Ok(StepCheckpoint {
            run_id: helper.run_id,
            step_id: helper.step_id,
            status,
            started_at: helper.started_at,
            completed_at: helper.completed_at,
            duration_ms: helper.duration_ms,
            state_snapshot: helper.state_snapshot,
            output_log: helper.output_log,
            error_message: helper.error_message,
        })
    }
}

impl serde::Serialize for StateFile {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("StateFile", 6)?;
        state.serialize_field("run_id", &self.run_id)?;
        state.serialize_field("step_id", &self.step_id)?;
        state.serialize_field("file_path", &self.file_path)?;
        state.serialize_field("content_hash", &self.content_hash)?;
        state.serialize_field("backed_up_at", &self.backed_up_at)?;
        state.serialize_field("backup_path", &self.backup_path)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for StateFile {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct FileHelper {
            run_id: String,
            step_id: String,
            file_path: PathBuf,
            content_hash: String,
            backed_up_at: Option<u64>,
            backup_path: Option<PathBuf>,
        }

        let helper = FileHelper::deserialize(deserializer)?;

        Ok(StateFile {
            run_id: helper.run_id,
            step_id: helper.step_id,
            file_path: helper.file_path,
            content_hash: helper.content_hash,
            backed_up_at: helper.backed_up_at,
            backup_path: helper.backup_path,
        })
    }
}

/// Generate a unique run ID
fn generate_run_id() -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    current_timestamp().hash(&mut hasher);
    std::process::id().hash(&mut hasher);
    format!("run-{:016x}", hasher.finish())
}

/// Get current timestamp as seconds since epoch
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // =========================================================================
    // RED Phase: Failing Tests First (EXTREME TDD)
    // Test naming: test_<TASK_ID>_<feature>_<scenario>
    // TASK_ID: CHECKPOINT_106
    // =========================================================================

    #[test]
    fn test_CHECKPOINT_106_create_store() {
        let temp_dir = TempDir::new().unwrap();
        let store = CheckpointStore::new(temp_dir.path()).unwrap();
        assert!(store.current_run_id().is_none());
    }

    #[test]
    fn test_CHECKPOINT_106_start_run() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = CheckpointStore::new(temp_dir.path()).unwrap();

        let run_id = store.start_run("my-installer", "1.0.0").unwrap();
        assert!(run_id.starts_with("run-"));
        assert!(store.current_run_id().is_some());
    }

    #[test]
    fn test_CHECKPOINT_106_add_step() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = CheckpointStore::new(temp_dir.path()).unwrap();

        store.start_run("my-installer", "1.0.0").unwrap();
        store.add_step("step-1").unwrap();

        let step = store.get_step("step-1").unwrap();
        assert_eq!(step.status, StepStatus::Pending);
    }

    #[test]
    fn test_CHECKPOINT_106_step_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = CheckpointStore::new(temp_dir.path()).unwrap();

        store.start_run("my-installer", "1.0.0").unwrap();
        store.add_step("step-1").unwrap();

        // Start
        store.start_step("step-1").unwrap();
        assert_eq!(
            store.get_step("step-1").unwrap().status,
            StepStatus::Running
        );

        // Complete
        store
            .complete_step("step-1", Some("output".to_string()))
            .unwrap();
        let step = store.get_step("step-1").unwrap();
        assert_eq!(step.status, StepStatus::Completed);
        assert_eq!(step.output_log, Some("output".to_string()));
    }

    #[test]
    fn test_CHECKPOINT_106_step_failure() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = CheckpointStore::new(temp_dir.path()).unwrap();

        store.start_run("my-installer", "1.0.0").unwrap();
        store.add_step("step-1").unwrap();
        store.start_step("step-1").unwrap();
        store.fail_step("step-1", "Something went wrong").unwrap();

        let step = store.get_step("step-1").unwrap();
        assert_eq!(step.status, StepStatus::Failed);
        assert_eq!(step.error_message, Some("Something went wrong".to_string()));
    }

    #[test]
    fn test_CHECKPOINT_106_last_successful_step() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = CheckpointStore::new(temp_dir.path()).unwrap();

        store.start_run("my-installer", "1.0.0").unwrap();

        store.add_step("step-1").unwrap();
        store.start_step("step-1").unwrap();
        store.complete_step("step-1", None).unwrap();

        store.add_step("step-2").unwrap();
        store.start_step("step-2").unwrap();
        store.complete_step("step-2", None).unwrap();

        store.add_step("step-3").unwrap();
        store.start_step("step-3").unwrap();
        store.fail_step("step-3", "error").unwrap();

        let last = store.last_successful_step().unwrap();
        assert_eq!(last.step_id, "step-2");
    }

    #[test]
    fn test_CHECKPOINT_106_hermetic_mode() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = CheckpointStore::new(temp_dir.path()).unwrap();

        store
            .start_hermetic_run("my-installer", "1.0.0", "abc123")
            .unwrap();
        assert!(store.is_hermetic());

        // Verify consistency with same hash
        store.verify_hermetic_consistency("abc123").unwrap();

        // Verify fails with different hash
        let result = store.verify_hermetic_consistency("different");
        assert!(result.is_err());
    }

    #[test]
    fn test_CHECKPOINT_106_track_file() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = CheckpointStore::new(temp_dir.path()).unwrap();

        store.start_run("my-installer", "1.0.0").unwrap();
        store.add_step("step-1").unwrap();

        store
            .track_file("step-1", Path::new("/etc/config.txt"), "sha256:abc")
            .unwrap();

        let files = store.state_files_for_step("step-1");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].content_hash, "sha256:abc");
    }

    #[test]
    fn test_CHECKPOINT_106_persistence() {
        let temp_dir = TempDir::new().unwrap();

        // Create and populate store
        {
            let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
            store.start_run("my-installer", "1.0.0").unwrap();
            store.add_step("step-1").unwrap();
            store.start_step("step-1").unwrap();
            store
                .complete_step("step-1", Some("done".to_string()))
                .unwrap();
        }

        // Load from disk
        {
            let store = CheckpointStore::load(temp_dir.path()).unwrap();
            assert!(store.current_run_id().is_some());
            let step = store.get_step("step-1").unwrap();
            assert_eq!(step.status, StepStatus::Completed);
        }
    }

    #[test]
    fn test_CHECKPOINT_106_run_status_roundtrip() {
        for status in [
            RunStatus::Running,
            RunStatus::Completed,
            RunStatus::Failed,
            RunStatus::Aborted,
        ] {
            let s = status.as_str();
            assert_eq!(RunStatus::parse(s), Some(status));
        }
    }

    #[test]
    fn test_CHECKPOINT_106_step_status_roundtrip() {
        for status in [
            StepStatus::Pending,
            StepStatus::Running,
            StepStatus::Completed,
            StepStatus::Failed,
            StepStatus::Skipped,
        ] {
            let s = status.as_str();
            assert_eq!(StepStatus::parse(s), Some(status));
        }
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use tempfile::TempDir;

    proptest! {
        /// Property: Store never panics on any installer name
        #[test]
        fn prop_store_handles_any_name(name in ".*") {
            let temp_dir = TempDir::new().unwrap();
            let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
            // Should not panic
            let _ = store.start_run(&name, "1.0.0");
        }

        /// Property: Step IDs are preserved exactly
        #[test]
        fn prop_step_id_preserved(step_id in "[a-zA-Z][a-zA-Z0-9_-]{0,50}") {
            let temp_dir = TempDir::new().unwrap();
            let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
            store.start_run("test", "1.0.0").unwrap();
            store.add_step(&step_id).unwrap();

            let step = store.get_step(&step_id);
            prop_assert!(step.is_some());
            prop_assert_eq!(&step.unwrap().step_id, &step_id);
        }
    }
}
