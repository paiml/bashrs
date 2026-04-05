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

        let before_changed = before_lines.get(common_prefix..before_end).unwrap_or(&[]);
        let after_changed = after_lines.get(common_prefix..after_end).unwrap_or(&[]);

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
    _env_changes: HashMap<String, Option<String>>,

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

include!("dry_run_incl3.rs");
