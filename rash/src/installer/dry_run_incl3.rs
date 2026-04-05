impl DryRunContext {
    /// Create a new dry-run context
    pub fn new() -> Self {
        Self::default()
    }

    /// Simulate a file write
    pub fn simulate_file_write(
        &mut self,
        path: impl AsRef<Path>,
        content: &str,
        mode: Option<u32>,
    ) {
        let path = path.as_ref().to_path_buf();

        // Check if file exists (would need real filesystem in practice)
        let change = FileChange::created(&path, content, mode);
        self.fs_changes.insert(path, change);
    }

    /// Simulate a file modification
    pub fn simulate_file_modify(&mut self, path: impl AsRef<Path>, before: &str, after: &str) {
        let path = path.as_ref().to_path_buf();
        let change = FileChange::modified(&path, before, after);
        self.fs_changes.insert(path, change);
    }

    /// Simulate a file deletion
    pub fn simulate_file_delete(&mut self, path: impl AsRef<Path>, content: &str) {
        let path = path.as_ref().to_path_buf();
        let change = FileChange::deleted(&path, content);
        self.fs_changes.insert(path, change);
    }

    /// Simulate package installation
    pub fn simulate_package_install(&mut self, name: &str, version: Option<&str>) {
        self.package_ops
            .push(PackageOperation::install(name, version));
    }

    /// Simulate package removal
    pub fn simulate_package_remove(&mut self, name: &str) {
        self.package_ops.push(PackageOperation::remove(name));
    }

    /// Simulate service enable
    pub fn simulate_service_enable(&mut self, name: &str) {
        self.service_ops.push(ServiceOperation::Enable {
            name: name.to_string(),
        });
    }

    /// Simulate service start
    pub fn simulate_service_start(&mut self, name: &str) {
        self.service_ops.push(ServiceOperation::Start {
            name: name.to_string(),
        });
    }

    /// Simulate adding user to group
    pub fn simulate_add_to_group(&mut self, user: &str, group: &str) {
        self.user_ops.push(UserGroupOperation::AddToGroup {
            user: user.to_string(),
            group: group.to_string(),
        });
    }

    /// Log a simulation step
    pub fn log_step(&mut self, step_id: &str, step_name: &str, description: &str) {
        self.simulation_log.push(SimulationEntry {
            step_id: step_id.to_string(),
            step_name: step_name.to_string(),
            description: description.to_string(),
            would_succeed: true,
            failure_reason: None,
        });
    }

    /// Log a step that would fail
    pub fn log_step_failure(&mut self, step_id: &str, step_name: &str, reason: &str) {
        self.simulation_log.push(SimulationEntry {
            step_id: step_id.to_string(),
            step_name: step_name.to_string(),
            description: format!("Would fail: {}", reason),
            would_succeed: false,
            failure_reason: Some(reason.to_string()),
        });
    }

    /// Get file changes
    pub fn file_changes(&self) -> impl Iterator<Item = &FileChange> {
        self.fs_changes.values()
    }

    /// Get package operations
    pub fn package_operations(&self) -> &[PackageOperation] {
        &self.package_ops
    }

    /// Get service operations
    pub fn service_operations(&self) -> &[ServiceOperation] {
        &self.service_ops
    }

    /// Get user/group operations
    pub fn user_group_operations(&self) -> &[UserGroupOperation] {
        &self.user_ops
    }

    /// Get simulation log
    pub fn simulation_log(&self) -> &[SimulationEntry] {
        &self.simulation_log
    }

    /// Generate a complete diff preview
    pub fn generate_diff(&self) -> DiffPreview {
        DiffPreview {
            file_changes: self.fs_changes.values().cloned().collect(),
            package_ops: self.package_ops.clone(),
            service_ops: self.service_ops.clone(),
            user_ops: self.user_ops.clone(),
        }
    }

    /// Get summary statistics
    pub fn summary(&self) -> DryRunSummary {
        let files_created = self
            .fs_changes
            .values()
            .filter(|c| c.change_type == FileChangeType::Created)
            .count();
        let files_modified = self
            .fs_changes
            .values()
            .filter(|c| c.change_type == FileChangeType::Modified)
            .count();
        let files_deleted = self
            .fs_changes
            .values()
            .filter(|c| c.change_type == FileChangeType::Deleted)
            .count();

        let packages_installed = self
            .package_ops
            .iter()
            .filter(|p| matches!(p, PackageOperation::Install { .. }))
            .count();
        let packages_removed = self
            .package_ops
            .iter()
            .filter(|p| matches!(p, PackageOperation::Remove { .. }))
            .count();

        let services_enabled = self
            .service_ops
            .iter()
            .filter(|s| matches!(s, ServiceOperation::Enable { .. }))
            .count();

        let users_modified = self.user_ops.len();

        DryRunSummary {
            files_created,
            files_modified,
            files_deleted,
            packages_installed,
            packages_removed,
            services_enabled,
            users_modified,
            steps_would_fail: self
                .simulation_log
                .iter()
                .filter(|e| !e.would_succeed)
                .count(),
        }
    }
}


include!("dry_run_incl2.rs");
