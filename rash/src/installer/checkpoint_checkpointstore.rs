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


include!("checkpoint_serde.rs");
