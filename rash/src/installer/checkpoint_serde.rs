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

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "checkpoint_tests_checkpoint_1.rs"]
// FIXME(PMAT-238): mod tests_extracted;
