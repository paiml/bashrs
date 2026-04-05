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
        text.push_str(&format!(
            "  Packages installed: {}\n",
            self.packages_installed
        ));
        text.push_str(&format!(
            "  Packages removed:   {}\n",
            self.packages_removed
        ));
        text.push_str(&format!(
            "  Services enabled:   {}\n",
            self.services_enabled
        ));
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
#[path = "dry_run_tests_extracted.rs"]
mod tests_extracted;
