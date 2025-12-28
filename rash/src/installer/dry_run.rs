//! Dry-run diff preview for installers (#111)
//!
//! Simulates installer execution without making changes, generating
//! a unified diff of all planned modifications.
//!
//! # Example
//!
//! ```bash
//! # Preview all changes before execution
//! bashrs installer run ./my-installer --dry-run --diff
//!
//! # JSON output for programmatic use
//! bashrs installer run ./my-installer --dry-run --format json
//! ```

// Note: Error and Result available from crate::models if needed
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A simulated change to a file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    /// Path to the file
    pub path: PathBuf,

    /// Content before the change (None if file didn't exist)
    pub before: Option<String>,

    /// Content after the change (None if file will be deleted)
    pub after: Option<String>,

    /// File mode (permissions) after change
    pub mode: Option<u32>,

    /// Change type
    pub change_type: FileChangeType,
}

/// Type of file change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileChangeType {
    /// New file created
    Created,
    /// Existing file modified
    Modified,
    /// File deleted
    Deleted,
    /// File mode (permissions) changed
    ModeChanged,
}

impl FileChange {
    /// Create a file creation change
    pub fn created(path: impl AsRef<Path>, content: &str, mode: Option<u32>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            before: None,
            after: Some(content.to_string()),
            mode,
            change_type: FileChangeType::Created,
        }
    }

    /// Create a file modification change
    pub fn modified(path: impl AsRef<Path>, before: &str, after: &str) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            before: Some(before.to_string()),
            after: Some(after.to_string()),
            mode: None,
            change_type: FileChangeType::Modified,
        }
    }

    /// Create a file deletion change
    pub fn deleted(path: impl AsRef<Path>, content: &str) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            before: Some(content.to_string()),
            after: None,
            mode: None,
            change_type: FileChangeType::Deleted,
        }
    }

    /// Generate unified diff for this change
    pub fn to_diff(&self) -> String {
        let path_str = self.path.display().to_string();
        let mut diff = String::new();

        match self.change_type {
            FileChangeType::Created => {
                diff.push_str("--- /dev/null\n");
                diff.push_str(&format!("+++ b{}\n", path_str));
                if let Some(ref content) = self.after {
                    let lines: Vec<&str> = content.lines().collect();
                    diff.push_str(&format!("@@ -0,0 +1,{} @@\n", lines.len()));
                    for line in lines {
                        diff.push_str(&format!("+{}\n", line));
                    }
                }
            }
            FileChangeType::Modified => {
                diff.push_str(&format!("--- a{}\n", path_str));
                diff.push_str(&format!("+++ b{}\n", path_str));
                diff.push_str(&self.compute_unified_diff());
            }
            FileChangeType::Deleted => {
                diff.push_str(&format!("--- a{}\n", path_str));
                diff.push_str("+++ /dev/null\n");
                if let Some(ref content) = self.before {
                    let lines: Vec<&str> = content.lines().collect();
                    diff.push_str(&format!("@@ -1,{} +0,0 @@\n", lines.len()));
                    for line in lines {
                        diff.push_str(&format!("-{}\n", line));
                    }
                }
            }
            FileChangeType::ModeChanged => {
                diff.push_str(&format!("--- a{}\n", path_str));
                diff.push_str(&format!("+++ b{}\n", path_str));
                if let Some(mode) = self.mode {
                    diff.push_str(&format!("# chmod {:o}\n", mode));
                }
            }
        }

        diff
    }

    /// Compute unified diff between before and after
    fn compute_unified_diff(&self) -> String {
        let before = self.before.as_deref().unwrap_or("");
        let after = self.after.as_deref().unwrap_or("");

        let before_lines: Vec<&str> = before.lines().collect();
        let after_lines: Vec<&str> = after.lines().collect();

        // Simple diff: show all removed lines, then all added lines
        // (A proper implementation would use LCS algorithm)
        let mut diff = String::new();

        // Find common prefix
        let common_prefix = before_lines
            .iter()
            .zip(after_lines.iter())
            .take_while(|(a, b)| a == b)
            .count();

        // Find common suffix
        let common_suffix = before_lines
            .iter()
            .rev()
            .zip(after_lines.iter().rev())
            .take_while(|(a, b)| a == b)
            .count();

        let before_end = before_lines.len().saturating_sub(common_suffix);
        let after_end = after_lines.len().saturating_sub(common_suffix);

        let before_changed = before_lines
            .get(common_prefix..before_end)
            .unwrap_or(&[]);
        let after_changed = after_lines
            .get(common_prefix..after_end)
            .unwrap_or(&[]);

        if before_changed.is_empty() && after_changed.is_empty() {
            return diff;
        }

        diff.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            common_prefix + 1,
            before_changed.len(),
            common_prefix + 1,
            after_changed.len()
        ));

        for line in before_changed {
            diff.push_str(&format!("-{}\n", line));
        }
        for line in after_changed {
            diff.push_str(&format!("+{}\n", line));
        }

        diff
    }
}

/// A package operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageOperation {
    /// Install a package
    Install {
        name: String,
        version: Option<String>,
    },
    /// Remove a package
    Remove { name: String },
    /// Upgrade a package
    Upgrade {
        name: String,
        from_version: Option<String>,
        to_version: Option<String>,
    },
}

impl PackageOperation {
    /// Create an install operation
    pub fn install(name: &str, version: Option<&str>) -> Self {
        Self::Install {
            name: name.to_string(),
            version: version.map(String::from),
        }
    }

    /// Create a remove operation
    pub fn remove(name: &str) -> Self {
        Self::Remove {
            name: name.to_string(),
        }
    }

    /// Format for diff output
    pub fn to_diff_line(&self) -> String {
        match self {
            Self::Install { name, version } => {
                if let Some(v) = version {
                    format!("+ {} ({})", name, v)
                } else {
                    format!("+ {}", name)
                }
            }
            Self::Remove { name } => format!("- {}", name),
            Self::Upgrade {
                name,
                from_version,
                to_version,
            } => {
                let from = from_version.as_deref().unwrap_or("?");
                let to = to_version.as_deref().unwrap_or("?");
                format!("~ {} ({} -> {})", name, from, to)
            }
        }
    }
}

/// A service operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceOperation {
    /// Enable a service
    Enable { name: String },
    /// Disable a service
    Disable { name: String },
    /// Start a service
    Start { name: String },
    /// Stop a service
    Stop { name: String },
    /// Restart a service
    Restart { name: String },
}

impl ServiceOperation {
    /// Format for diff output
    pub fn to_diff_line(&self) -> String {
        match self {
            Self::Enable { name } => format!("+ systemctl enable {}", name),
            Self::Disable { name } => format!("- systemctl enable {}", name),
            Self::Start { name } => format!("+ systemctl start {}", name),
            Self::Stop { name } => format!("- systemctl start {}", name),
            Self::Restart { name } => format!("~ systemctl restart {}", name),
        }
    }
}

/// A user/group operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserGroupOperation {
    /// Add user to group
    AddToGroup { user: String, group: String },
    /// Remove user from group
    RemoveFromGroup { user: String, group: String },
    /// Create user
    CreateUser { name: String, groups: Vec<String> },
    /// Create group
    CreateGroup { name: String },
}

impl UserGroupOperation {
    /// Format for diff output
    pub fn to_diff_line(&self) -> String {
        match self {
            Self::AddToGroup { user, group } => format!("+ usermod -aG {} {}", group, user),
            Self::RemoveFromGroup { user, group } => format!("- gpasswd -d {} {}", user, group),
            Self::CreateUser { name, groups } => {
                if groups.is_empty() {
                    format!("+ useradd {}", name)
                } else {
                    format!("+ useradd -G {} {}", groups.join(","), name)
                }
            }
            Self::CreateGroup { name } => format!("+ groupadd {}", name),
        }
    }
}

/// Dry-run execution context that captures all simulated changes
#[derive(Debug, Default)]
pub struct DryRunContext {
    /// Virtual filesystem overlay
    fs_changes: HashMap<PathBuf, FileChange>,

    /// Captured package operations
    package_ops: Vec<PackageOperation>,

    /// Captured service operations
    service_ops: Vec<ServiceOperation>,

    /// Captured user/group operations
    user_ops: Vec<UserGroupOperation>,

    /// Captured environment variable changes (reserved for future use)
    #[allow(dead_code)]
    env_changes: HashMap<String, Option<String>>,

    /// Step-by-step simulation log
    simulation_log: Vec<SimulationEntry>,
}

/// A log entry from simulation
#[derive(Debug, Clone)]
pub struct SimulationEntry {
    /// Step ID
    pub step_id: String,

    /// Step name
    pub step_name: String,

    /// What would be done
    pub description: String,

    /// Whether the step would succeed
    pub would_succeed: bool,

    /// Reason for failure if applicable
    pub failure_reason: Option<String>,
}

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
    pub fn simulate_file_modify(
        &mut self,
        path: impl AsRef<Path>,
        before: &str,
        after: &str,
    ) {
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
        self.package_ops.push(PackageOperation::install(name, version));
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

/// Summary of dry-run changes
#[derive(Debug, Clone, Default)]
pub struct DryRunSummary {
    /// Number of files to be created
    pub files_created: usize,
    /// Number of files to be modified
    pub files_modified: usize,
    /// Number of files to be deleted
    pub files_deleted: usize,
    /// Number of packages to be installed
    pub packages_installed: usize,
    /// Number of packages to be removed
    pub packages_removed: usize,
    /// Number of services to be enabled
    pub services_enabled: usize,
    /// Number of users to be modified
    pub users_modified: usize,
    /// Number of steps that would fail
    pub steps_would_fail: usize,
}

impl DryRunSummary {
    /// Format summary as human-readable text
    pub fn to_text(&self) -> String {
        let mut text = String::from("=== Summary ===\n\n");
        text.push_str(&format!("  Files created:      {}\n", self.files_created));
        text.push_str(&format!("  Files modified:     {}\n", self.files_modified));
        text.push_str(&format!("  Files deleted:      {}\n", self.files_deleted));
        text.push_str(&format!("  Packages installed: {}\n", self.packages_installed));
        text.push_str(&format!("  Packages removed:   {}\n", self.packages_removed));
        text.push_str(&format!("  Services enabled:   {}\n", self.services_enabled));
        text.push_str(&format!("  Users modified:     {}\n", self.users_modified));

        if self.steps_would_fail > 0 {
            text.push_str(&format!(
                "\n  ⚠ {} step(s) would fail\n",
                self.steps_would_fail
            ));
        }

        text
    }

    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        self.files_created > 0
            || self.files_modified > 0
            || self.files_deleted > 0
            || self.packages_installed > 0
            || self.packages_removed > 0
            || self.services_enabled > 0
            || self.users_modified > 0
    }
}

/// Complete diff preview
#[derive(Debug, Clone)]
pub struct DiffPreview {
    /// File changes
    pub file_changes: Vec<FileChange>,
    /// Package operations
    pub package_ops: Vec<PackageOperation>,
    /// Service operations
    pub service_ops: Vec<ServiceOperation>,
    /// User/group operations
    pub user_ops: Vec<UserGroupOperation>,
}

impl DiffPreview {
    /// Format as unified diff text
    pub fn to_diff_text(&self) -> String {
        let mut diff = String::new();

        // File changes section
        if !self.file_changes.is_empty() {
            diff.push_str("=== Filesystem Changes ===\n\n");
            for change in &self.file_changes {
                diff.push_str(&change.to_diff());
                diff.push('\n');
            }
        }

        // Package changes section
        if !self.package_ops.is_empty() {
            diff.push_str("=== Package Changes ===\n\n");
            for op in &self.package_ops {
                diff.push_str(&op.to_diff_line());
                diff.push('\n');
            }
            diff.push('\n');
        }

        // Service changes section
        if !self.service_ops.is_empty() {
            diff.push_str("=== Service Changes ===\n\n");
            for op in &self.service_ops {
                diff.push_str(&op.to_diff_line());
                diff.push('\n');
            }
            diff.push('\n');
        }

        // User/group changes section
        if !self.user_ops.is_empty() {
            diff.push_str("=== User/Group Changes ===\n\n");
            for op in &self.user_ops {
                diff.push_str(&op.to_diff_line());
                diff.push('\n');
            }
            diff.push('\n');
        }

        diff
    }

    /// Check if there are any changes
    pub fn is_empty(&self) -> bool {
        self.file_changes.is_empty()
            && self.package_ops.is_empty()
            && self.service_ops.is_empty()
            && self.user_ops.is_empty()
    }
}

#[cfg(test)]
mod tests {
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

        ctx.simulate_file_write("/etc/docker/daemon.json", "{\n  \"storage-driver\": \"overlay2\"\n}\n", None);
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
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: File creation always produces valid diff
        #[test]
        fn prop_file_creation_diff_valid(
            path in "/[a-z]+/[a-z]+\\.[a-z]+",
            content in "[a-zA-Z0-9 \n]{1,100}"
        ) {
            let change = FileChange::created(&path, &content, None);
            let diff = change.to_diff();

            prop_assert!(diff.contains("--- /dev/null"));
            let expected_path = format!("+++ b{}", path);
            prop_assert!(diff.contains(&expected_path));
        }

        /// Property: Package install always produces + prefix
        #[test]
        fn prop_package_install_prefix(
            name in "[a-z][a-z0-9-]{0,20}",
            version in "[0-9]+\\.[0-9]+\\.[0-9]+"
        ) {
            let op = PackageOperation::install(&name, Some(&version));
            let line = op.to_diff_line();

            prop_assert!(line.starts_with("+ "));
            prop_assert!(line.contains(&name));
        }

        /// Property: Summary correctly counts operations
        #[test]
        fn prop_summary_counts_correct(
            file_count in 0usize..10,
            pkg_count in 0usize..10
        ) {
            let mut ctx = DryRunContext::new();

            for i in 0..file_count {
                ctx.simulate_file_write(format!("/tmp/file-{}.txt", i), "content", None);
            }

            for i in 0..pkg_count {
                ctx.simulate_package_install(&format!("pkg-{}", i), None);
            }

            let summary = ctx.summary();
            prop_assert_eq!(summary.files_created, file_count);
            prop_assert_eq!(summary.packages_installed, pkg_count);
        }
    }
}
