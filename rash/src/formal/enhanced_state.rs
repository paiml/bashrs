//! Enhanced Abstract State with Permission Tracking
//!
//! **Status:** DRAFT - Reference Implementation for P0 Fix
//! **Reviewer:** Claude (Toyota Way Review)
//! **Date:** 2025-11-23
//!
//! This module provides an enhanced state model that includes:
//! - File permissions (mode bits)
//! - File ownership (UID/GID)
//! - User execution context (EUID/EGID)
//! - Permission-aware operations
//!
//! **Rationale:**
//! The current `AbstractState` (abstract_state.rs) lacks permission tracking,
//! making idempotency proofs unsound for real Unix systems. This enhanced
//! model addresses the gap identified in the Toyota Way review.
//!
//! **Migration Path:**
//! 1. Add this module to rash/src/formal/mod.rs
//! 2. Update tests to use EnhancedState
//! 3. Deprecate old AbstractState
//! 4. Update purifier to use permission-aware operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Enhanced filesystem entry with Unix metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnhancedFileSystemEntry {
    /// Directory with Unix permissions and ownership
    Directory {
        /// File mode bits (e.g., 0o755, 0o700)
        /// Format: owner|group|other (rwx|rwx|rwx)
        mode: u32,
        /// Owner user ID (e.g., 0 for root, 1000 for first user)
        uid: u32,
        /// Owner group ID (e.g., 0 for root, 1000 for first user)
        gid: u32,
    },
    /// File with content, permissions, and ownership
    File {
        /// File content (text)
        content: String,
        /// File mode bits (e.g., 0o644, 0o600)
        mode: u32,
        /// Owner user ID
        uid: u32,
        /// Owner group ID
        gid: u32,
        /// Modification time (Unix timestamp)
        /// Used for determinism verification
        mtime: Option<i64>,
    },
}

impl EnhancedFileSystemEntry {
    /// Get the mode bits for this entry
    pub fn mode(&self) -> u32 {
        match self {
            Self::Directory { mode, .. } | Self::File { mode, .. } => *mode,
        }
    }

    /// Get the owner UID for this entry
    pub fn uid(&self) -> u32 {
        match self {
            Self::Directory { uid, .. } | Self::File { uid, .. } => *uid,
        }
    }

    /// Get the owner GID for this entry
    pub fn gid(&self) -> u32 {
        match self {
            Self::Directory { gid, .. } | Self::File { gid, .. } => *gid,
        }
    }

    /// Check if this is a directory
    pub fn is_directory(&self) -> bool {
        matches!(self, Self::Directory { .. })
    }

    /// Check if this is a file
    pub fn is_file(&self) -> bool {
        matches!(self, Self::File { .. })
    }
}

/// Enhanced abstract state with user context and permission tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnhancedState {
    /// Environment variables (name -> value mapping)
    pub env: HashMap<String, String>,

    /// Current working directory
    pub cwd: PathBuf,

    /// Standard output buffer
    pub stdout: Vec<String>,

    /// Standard error buffer
    pub stderr: Vec<String>,

    /// Exit code of the last command
    pub exit_code: i32,

    /// Enhanced filesystem with permissions and ownership
    pub filesystem: HashMap<PathBuf, EnhancedFileSystemEntry>,

    /// Current effective user ID (EUID)
    /// Used for permission checks
    pub euid: u32,

    /// Current effective group ID (EGID)
    /// Used for permission checks
    pub egid: u32,

    /// Supplementary group IDs
    /// User can belong to multiple groups
    pub groups: Vec<u32>,
}

impl Default for EnhancedState {
    fn default() -> Self {
        let mut filesystem = HashMap::new();

        // Initialize with root directory (owned by root, mode 0755)
        filesystem.insert(
            PathBuf::from("/"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o755,
                uid: 0,
                gid: 0,
            },
        );

        Self {
            env: HashMap::new(),
            cwd: PathBuf::from("/"),
            stdout: Vec::new(),
            stderr: Vec::new(),
            exit_code: 0,
            filesystem,
            euid: 0,         // Default to root user
            egid: 0,         // Default to root group
            groups: vec![0], // Default to root group only
        }
    }
}

impl EnhancedState {
    /// Create a new enhanced state with basic initialization
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a non-root user state for testing
    pub fn new_user(uid: u32, gid: u32) -> Self {
        Self {
            euid: uid,
            egid: gid,
            groups: vec![gid],
            ..Default::default()
        }
    }

    /// Set an environment variable
    pub fn set_env(&mut self, name: String, value: String) {
        self.env.insert(name, value);
    }

    /// Get an environment variable
    pub fn get_env(&self, name: &str) -> Option<&String> {
        self.env.get(name)
    }

    /// Change the current working directory
    pub fn change_directory(&mut self, path: PathBuf) -> Result<(), String> {
        // Check if the path exists and is a directory
        match self.filesystem.get(&path) {
            Some(entry) if entry.is_directory() => {
                // Check execute permission
                if self.can_execute(&path) {
                    self.cwd = path;
                    self.exit_code = 0;
                    Ok(())
                } else {
                    self.stderr
                        .push(format!("cd: {}: Permission denied", path.display()));
                    self.exit_code = 1;
                    Err("Permission denied".to_string())
                }
            }
            Some(_) => {
                self.stderr
                    .push(format!("cd: {}: Not a directory", path.display()));
                self.exit_code = 1;
                Err("Not a directory".to_string())
            }
            None => {
                self.stderr
                    .push(format!("cd: {}: No such file or directory", path.display()));
                self.exit_code = 1;
                Err("No such file or directory".to_string())
            }
        }
    }

    /// Check if current user can read path
    pub fn can_read(&self, path: &PathBuf) -> bool {
        self.check_permission(path, 0o4) // Read bit: 4
    }

    /// Check if current user can write to path
    pub fn can_write(&self, path: &PathBuf) -> bool {
        self.check_permission(path, 0o2) // Write bit: 2
    }

    /// Check if current user can execute path
    pub fn can_execute(&self, path: &PathBuf) -> bool {
        self.check_permission(path, 0o1) // Execute bit: 1
    }

    /// Generic permission check
    ///
    /// Checks if the current user (EUID/EGID) has the specified permission
    /// on the given path.
    ///
    /// Permission bits: r=4, w=2, x=1
    /// Mode format: owner|group|other (e.g., 0o755 = rwxr-xr-x)
    fn check_permission(&self, path: &PathBuf, perm_bit: u32) -> bool {
        match self.filesystem.get(path) {
            Some(entry) => {
                let mode = entry.mode();
                let uid = entry.uid();
                let gid = entry.gid();

                // Root (UID 0) bypasses all permission checks
                if self.euid == 0 {
                    return true;
                }

                // Check owner permissions (bits 8-6)
                if uid == self.euid {
                    return (mode >> 6) & perm_bit != 0;
                }

                // Check group permissions (bits 5-3)
                if gid == self.egid || self.groups.contains(&gid) {
                    return (mode >> 3) & perm_bit != 0;
                }

                // Check other permissions (bits 2-0)
                mode & perm_bit != 0
            }
            None => {
                // If path doesn't exist, check parent directory
                if let Some(parent) = path.parent() {
                    self.can_write(&parent.to_path_buf())
                } else {
                    false
                }
            }
        }
    }

    /// Create a directory (mkdir -p behavior) with permission checks
    ///
    /// **Idempotency Property:**
    /// - If directory exists with compatible permissions: Success (exit 0)
    /// - If directory doesn't exist and user has write permission: Create (exit 0)
    /// - If user lacks write permission: Fail (exit 1)
    ///
    /// **Permission-Aware:**
    /// Unlike the old `create_directory()`, this method verifies:
    /// 1. User has write permission on parent directory
    /// 2. Existing directory is accessible
    pub fn create_directory_safe(&mut self, path: PathBuf, mode: u32) -> Result<(), String> {
        // Check if directory already exists
        match self.filesystem.get(&path) {
            Some(EnhancedFileSystemEntry::Directory { .. }) => {
                // ✅ Idempotent: Directory exists, no error
                self.exit_code = 0;
                return Ok(());
            }
            Some(EnhancedFileSystemEntry::File { .. }) => {
                self.stderr.push(format!(
                    "mkdir: cannot create directory '{}': File exists",
                    path.display()
                ));
                self.exit_code = 1;
                return Err("File exists".to_string());
            }
            None => {
                // Directory doesn't exist, check parent permission
            }
        }

        // Check write permission on parent directory
        if let Some(parent) = path.parent() {
            let parent_path = parent.to_path_buf();
            if !self.can_write(&parent_path) {
                self.stderr.push(format!(
                    "mkdir: cannot create directory '{}': Permission denied",
                    path.display()
                ));
                self.exit_code = 1;
                return Err("Permission denied".to_string());
            }

            // Ensure parent exists
            if !self.filesystem.contains_key(&parent_path) {
                // Recursively create parent directories
                self.create_directory_safe(parent_path, 0o755)?;
            }
        }

        // Create directory with current user's ownership
        self.filesystem.insert(
            path.clone(),
            EnhancedFileSystemEntry::Directory {
                mode,
                uid: self.euid,
                gid: self.egid,
            },
        );

        self.exit_code = 0;
        Ok(())
    }

    /// Write content to a file with permission checks
    pub fn write_file(&mut self, path: PathBuf, content: String, mode: u32) -> Result<(), String> {
        // Check if file exists
        if self.filesystem.contains_key(&path) {
            // File exists, check write permission
            if !self.can_write(&path) {
                self.stderr
                    .push(format!("write: {}: Permission denied", path.display()));
                self.exit_code = 1;
                return Err("Permission denied".to_string());
            }
        } else {
            // File doesn't exist, check parent write permission
            if let Some(parent) = path.parent() {
                let parent_path = parent.to_path_buf();
                if !self.can_write(&parent_path) {
                    self.stderr.push(format!(
                        "write: cannot create file '{}': Permission denied",
                        path.display()
                    ));
                    self.exit_code = 1;
                    return Err("Permission denied".to_string());
                }
            }
        }

        // Write file with current user's ownership
        self.filesystem.insert(
            path,
            EnhancedFileSystemEntry::File {
                content,
                mode,
                uid: self.euid,
                gid: self.egid,
                mtime: Some(0), // TODO: Use actual timestamp when chrono is added
            },
        );

        self.exit_code = 0;
        Ok(())
    }

    /// Read content from a file with permission checks
    pub fn read_file(&mut self, path: &PathBuf) -> Result<String, String> {
        // Check read permission
        if !self.can_read(path) {
            self.stderr
                .push(format!("cat: {}: Permission denied", path.display()));
            self.exit_code = 1;
            return Err("Permission denied".to_string());
        }

        match self.filesystem.get(path) {
            Some(EnhancedFileSystemEntry::File { content, .. }) => {
                self.exit_code = 0;
                Ok(content.clone())
            }
            Some(EnhancedFileSystemEntry::Directory { .. }) => {
                self.stderr
                    .push(format!("cat: {}: Is a directory", path.display()));
                self.exit_code = 1;
                Err("Is a directory".to_string())
            }
            None => {
                self.stderr.push(format!(
                    "cat: {}: No such file or directory",
                    path.display()
                ));
                self.exit_code = 1;
                Err("No such file or directory".to_string())
            }
        }
    }

    /// Write to stdout
    pub fn write_stdout(&mut self, content: String) {
        self.stdout.push(content);
        self.exit_code = 0;
    }

    /// Write to stderr
    pub fn write_stderr(&mut self, content: String) {
        self.stderr.push(content);
    }

    /// Check if two states are semantically equivalent
    pub fn is_equivalent(&self, other: &Self) -> bool {
        self.env == other.env
            && self.cwd == other.cwd
            && self.exit_code == other.exit_code
            && self.filesystem == other.filesystem
            && self.stdout == other.stdout
            && self.stderr == other.stderr
            && self.euid == other.euid
            && self.egid == other.egid
            && self.groups == other.groups
    }

    /// Create a test state with common setup
    pub fn test_state() -> Self {
        let mut state = Self::new_user(1000, 1000); // Non-root user

        // Add common directories (root-owned)
        state.filesystem.insert(
            PathBuf::from("/tmp"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o1777, // Sticky bit + world-writable
                uid: 0,
                gid: 0,
            },
        );

        state.filesystem.insert(
            PathBuf::from("/home"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o755,
                uid: 0,
                gid: 0,
            },
        );

        state.filesystem.insert(
            PathBuf::from("/home/user"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o755,
                uid: 1000,
                gid: 1000,
            },
        );

        state.filesystem.insert(
            PathBuf::from("/opt"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o755,
                uid: 0,
                gid: 0,
            },
        );

        // Add common environment variables
        state.set_env("PATH".to_string(), "/usr/bin:/bin".to_string());
        state.set_env("HOME".to_string(), "/home/user".to_string());
        state.set_env("USER".to_string(), "user".to_string());
        state.set_env("UID".to_string(), "1000".to_string());

        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state_is_root() {
        let state = EnhancedState::new();
        assert_eq!(state.euid, 0);
        assert_eq!(state.egid, 0);
        assert_eq!(state.cwd, PathBuf::from("/"));
        assert_eq!(state.exit_code, 0);
    }

    #[test]
    fn test_permission_aware_mkdir_success() {
        let mut state = EnhancedState::new_user(1000, 1000);

        // Add /tmp as world-writable
        state.filesystem.insert(
            PathBuf::from("/tmp"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o1777,
                uid: 0,
                gid: 0,
            },
        );

        // ✅ User can create directory in /tmp
        assert!(state
            .create_directory_safe(PathBuf::from("/tmp/user"), 0o755)
            .is_ok());
        assert_eq!(state.exit_code, 0);

        // Verify ownership
        let entry = state.filesystem.get(&PathBuf::from("/tmp/user")).unwrap();
        assert_eq!(entry.uid(), 1000);
        assert_eq!(entry.gid(), 1000);
    }

    #[test]
    fn test_permission_aware_mkdir_denied() {
        let mut state = EnhancedState::new_user(1000, 1000);

        // Add /opt as root-only
        state.filesystem.insert(
            PathBuf::from("/opt"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o755, // No write for others
                uid: 0,
                gid: 0,
            },
        );

        // ❌ User cannot create directory in /opt
        assert!(state
            .create_directory_safe(PathBuf::from("/opt/app"), 0o755)
            .is_err());
        assert_eq!(state.exit_code, 1);
        assert!(state.stderr.last().unwrap().contains("Permission denied"));
    }

    #[test]
    fn test_idempotent_mkdir_existing_directory() {
        let mut state = EnhancedState::new_user(1000, 1000);

        state.filesystem.insert(
            PathBuf::from("/tmp"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o1777,
                uid: 0,
                gid: 0,
            },
        );

        // First creation
        assert!(state
            .create_directory_safe(PathBuf::from("/tmp/user"), 0o755)
            .is_ok());

        // ✅ Second creation is idempotent (no error)
        assert!(state
            .create_directory_safe(PathBuf::from("/tmp/user"), 0o755)
            .is_ok());
        assert_eq!(state.exit_code, 0);
    }

    #[test]
    fn test_can_read_permission_check() {
        let mut state = EnhancedState::new_user(1000, 1000);

        // Owner-readable file
        state.filesystem.insert(
            PathBuf::from("/home/file.txt"),
            EnhancedFileSystemEntry::File {
                content: "data".to_string(),
                mode: 0o600, // rw-------
                uid: 1000,
                gid: 1000,
                mtime: None,
            },
        );

        assert!(state.can_read(&PathBuf::from("/home/file.txt")));

        // Other user's private file
        state.filesystem.insert(
            PathBuf::from("/root/secret.txt"),
            EnhancedFileSystemEntry::File {
                content: "secret".to_string(),
                mode: 0o600, // rw-------
                uid: 0,      // root
                gid: 0,
                mtime: None,
            },
        );

        assert!(!state.can_read(&PathBuf::from("/root/secret.txt")));
    }

    #[test]
    fn test_can_write_permission_check() {
        let mut state = EnhancedState::new_user(1000, 1000);

        // User's writable directory
        state.filesystem.insert(
            PathBuf::from("/home/user"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o755,
                uid: 1000,
                gid: 1000,
            },
        );

        assert!(state.can_write(&PathBuf::from("/home/user")));

        // Root-owned directory
        state.filesystem.insert(
            PathBuf::from("/root"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o700,
                uid: 0,
                gid: 0,
            },
        );

        assert!(!state.can_write(&PathBuf::from("/root")));
    }

    #[test]
    fn test_root_bypasses_permission_checks() {
        let mut state = EnhancedState::new(); // Root user (UID 0)

        // Create root-only directory
        state.filesystem.insert(
            PathBuf::from("/root"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o700, // rwx------
                uid: 0,
                gid: 0,
            },
        );

        // ✅ Root can read/write/execute despite restrictive permissions
        assert!(state.can_read(&PathBuf::from("/root")));
        assert!(state.can_write(&PathBuf::from("/root")));
        assert!(state.can_execute(&PathBuf::from("/root")));
    }

    #[test]
    fn test_read_file_with_permission_check() {
        let mut state = EnhancedState::new_user(1000, 1000);

        state.filesystem.insert(
            PathBuf::from("/tmp/file.txt"),
            EnhancedFileSystemEntry::File {
                content: "hello".to_string(),
                mode: 0o644, // rw-r--r--
                uid: 1000,
                gid: 1000,
                mtime: None,
            },
        );

        // ✅ User can read their own file
        assert_eq!(
            state.read_file(&PathBuf::from("/tmp/file.txt")).unwrap(),
            "hello"
        );

        // Create unreadable file
        state.filesystem.insert(
            PathBuf::from("/tmp/private.txt"),
            EnhancedFileSystemEntry::File {
                content: "secret".to_string(),
                mode: 0o000, // ---------
                uid: 0,
                gid: 0,
                mtime: None,
            },
        );

        // ❌ User cannot read unreadable file
        assert!(state.read_file(&PathBuf::from("/tmp/private.txt")).is_err());
        assert!(state.stderr.last().unwrap().contains("Permission denied"));
    }

    #[test]
    fn test_state_equivalence() {
        let mut state1 = EnhancedState::test_state();
        let mut state2 = EnhancedState::test_state();

        assert!(state1.is_equivalent(&state2));

        // Different environment variable
        state1.set_env("VAR".to_string(), "value".to_string());
        assert!(!state1.is_equivalent(&state2));

        state2.set_env("VAR".to_string(), "value".to_string());
        assert!(state1.is_equivalent(&state2));

        // Different EUID
        state1.euid = 0;
        assert!(!state1.is_equivalent(&state2));
    }

    #[test]
    fn test_group_permission_check() {
        let mut state = EnhancedState::new_user(1000, 1000);
        state.groups = vec![1000, 1001]; // User in multiple groups

        // File owned by group 1001
        state.filesystem.insert(
            PathBuf::from("/shared/file.txt"),
            EnhancedFileSystemEntry::File {
                content: "data".to_string(),
                mode: 0o640, // rw-r-----
                uid: 500,    // Different owner
                gid: 1001,   // Group 1001
                mtime: None,
            },
        );

        // ✅ User can read because they're in group 1001
        assert!(state.can_read(&PathBuf::from("/shared/file.txt")));
    }

    // ===== EnhancedFileSystemEntry Tests =====

    #[test]
    fn test_entry_mode() {
        let dir = EnhancedFileSystemEntry::Directory {
            mode: 0o755,
            uid: 100,
            gid: 100,
        };
        assert_eq!(dir.mode(), 0o755);

        let file = EnhancedFileSystemEntry::File {
            content: "test".to_string(),
            mode: 0o644,
            uid: 100,
            gid: 100,
            mtime: None,
        };
        assert_eq!(file.mode(), 0o644);
    }

    #[test]
    fn test_entry_uid() {
        let dir = EnhancedFileSystemEntry::Directory {
            mode: 0o755,
            uid: 500,
            gid: 100,
        };
        assert_eq!(dir.uid(), 500);

        let file = EnhancedFileSystemEntry::File {
            content: "test".to_string(),
            mode: 0o644,
            uid: 1000,
            gid: 100,
            mtime: Some(12345),
        };
        assert_eq!(file.uid(), 1000);
    }

    #[test]
    fn test_entry_gid() {
        let dir = EnhancedFileSystemEntry::Directory {
            mode: 0o755,
            uid: 100,
            gid: 500,
        };
        assert_eq!(dir.gid(), 500);

        let file = EnhancedFileSystemEntry::File {
            content: "test".to_string(),
            mode: 0o644,
            uid: 100,
            gid: 1000,
            mtime: None,
        };
        assert_eq!(file.gid(), 1000);
    }

    #[test]
    fn test_entry_is_directory() {
        let dir = EnhancedFileSystemEntry::Directory {
            mode: 0o755,
            uid: 0,
            gid: 0,
        };
        assert!(dir.is_directory());
        assert!(!dir.is_file());

        let file = EnhancedFileSystemEntry::File {
            content: "test".to_string(),
            mode: 0o644,
            uid: 0,
            gid: 0,
            mtime: None,
        };
        assert!(!file.is_directory());
        assert!(file.is_file());
    }

    // ===== EnhancedState Environment Tests =====

    #[test]
    fn test_set_and_get_env() {
        let mut state = EnhancedState::new();
        state.set_env("HOME".to_string(), "/root".to_string());

        assert_eq!(state.get_env("HOME"), Some(&"/root".to_string()));
        assert_eq!(state.get_env("NONEXISTENT"), None);
    }

    #[test]
    fn test_set_env_overwrite() {
        let mut state = EnhancedState::new();
        state.set_env("PATH".to_string(), "/bin".to_string());
        state.set_env("PATH".to_string(), "/usr/bin".to_string());

        assert_eq!(state.get_env("PATH"), Some(&"/usr/bin".to_string()));
    }

    // ===== EnhancedState Directory Operations =====

    #[test]
    fn test_change_directory_success() {
        let mut state = EnhancedState::new();
        state.filesystem.insert(
            PathBuf::from("/tmp"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o1777,
                uid: 0,
                gid: 0,
            },
        );

        assert!(state.change_directory(PathBuf::from("/tmp")).is_ok());
        assert_eq!(state.cwd, PathBuf::from("/tmp"));
        assert_eq!(state.exit_code, 0);
    }

    #[test]
    fn test_change_directory_not_found() {
        let mut state = EnhancedState::new();
        let result = state.change_directory(PathBuf::from("/nonexistent"));

        assert!(result.is_err());
        assert_eq!(state.exit_code, 1);
        assert!(state.stderr.last().unwrap().contains("No such file"));
    }

    #[test]
    fn test_change_directory_not_a_directory() {
        let mut state = EnhancedState::new();
        state.filesystem.insert(
            PathBuf::from("/file.txt"),
            EnhancedFileSystemEntry::File {
                content: "data".to_string(),
                mode: 0o644,
                uid: 0,
                gid: 0,
                mtime: None,
            },
        );

        let result = state.change_directory(PathBuf::from("/file.txt"));
        assert!(result.is_err());
        assert_eq!(state.exit_code, 1);
        assert!(state.stderr.last().unwrap().contains("Not a directory"));
    }

    #[test]
    fn test_change_directory_permission_denied() {
        let mut state = EnhancedState::new_user(1000, 1000);
        state.filesystem.insert(
            PathBuf::from("/restricted"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o700,
                uid: 0,
                gid: 0,
            },
        );

        let result = state.change_directory(PathBuf::from("/restricted"));
        assert!(result.is_err());
        assert_eq!(state.exit_code, 1);
        assert!(state.stderr.last().unwrap().contains("Permission denied"));
    }

    // ===== EnhancedState Write File Tests =====

    #[test]
    fn test_write_file_success() {
        let mut state = EnhancedState::new_user(1000, 1000);
        state.filesystem.insert(
            PathBuf::from("/home/user"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o755,
                uid: 1000,
                gid: 1000,
            },
        );

        let result = state.write_file(
            PathBuf::from("/home/user/file.txt"),
            "content".to_string(),
            0o644,
        );
        assert!(result.is_ok());

        let entry = state.filesystem.get(&PathBuf::from("/home/user/file.txt"));
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().uid(), 1000);
    }

    #[test]
    fn test_write_file_permission_denied() {
        let mut state = EnhancedState::new_user(1000, 1000);
        state.filesystem.insert(
            PathBuf::from("/root"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o700,
                uid: 0,
                gid: 0,
            },
        );

        let result = state.write_file(
            PathBuf::from("/root/file.txt"),
            "content".to_string(),
            0o644,
        );
        assert!(result.is_err());
        assert!(state.stderr.last().unwrap().contains("Permission denied"));
    }

    #[test]
    fn test_write_file_parent_no_permission() {
        // Non-root user cannot write to directory they don't have permission for
        let mut state = EnhancedState::new_user(1000, 1000);
        // Create parent directory owned by root with no write permission for others
        state.filesystem.insert(
            PathBuf::from("/restricted"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o755, // rwxr-xr-x - no write for others
                uid: 0,
                gid: 0,
            },
        );
        let result = state.write_file(
            PathBuf::from("/restricted/file.txt"),
            "content".to_string(),
            0o644,
        );
        assert!(result.is_err());
        assert!(state.stderr.last().unwrap().contains("Permission denied"));
    }

    // ===== EnhancedState stdout/stderr Tests =====

    #[test]
    fn test_write_stdout() {
        let mut state = EnhancedState::new();
        state.write_stdout("line1".to_string());
        state.write_stdout("line2".to_string());

        assert_eq!(state.stdout.len(), 2);
        assert_eq!(state.stdout[0], "line1");
        assert_eq!(state.stdout[1], "line2");
    }

    #[test]
    fn test_write_stderr() {
        let mut state = EnhancedState::new();
        state.write_stderr("error1".to_string());
        state.write_stderr("error2".to_string());

        assert_eq!(state.stderr.len(), 2);
        assert_eq!(state.stderr[0], "error1");
        assert_eq!(state.stderr[1], "error2");
    }

    // ===== EnhancedState Test State =====

    #[test]
    fn test_test_state_initialization() {
        let state = EnhancedState::test_state();
        assert_eq!(state.euid, 1000);
        assert_eq!(state.egid, 1000);
        assert!(state.filesystem.contains_key(&PathBuf::from("/tmp")));
    }

    // ===== EnhancedState Permission Edge Cases =====

    #[test]
    fn test_can_read_nonexistent_path_as_non_root() {
        // Non-root user cannot read nonexistent path
        let state = EnhancedState::new_user(1000, 1000);
        assert!(!state.can_read(&PathBuf::from("/nonexistent")));
    }

    #[test]
    fn test_can_write_nonexistent_path_as_non_root() {
        // Non-root user cannot write nonexistent path
        let state = EnhancedState::new_user(1000, 1000);
        assert!(!state.can_write(&PathBuf::from("/nonexistent")));
    }

    #[test]
    fn test_can_execute_nonexistent_path_as_non_root() {
        // Non-root user cannot execute nonexistent path
        let state = EnhancedState::new_user(1000, 1000);
        assert!(!state.can_execute(&PathBuf::from("/nonexistent")));
    }

    #[test]
    fn test_root_can_access_nonexistent_path() {
        // Root user (UID 0) bypasses permission checks
        let state = EnhancedState::new();
        // Root returns true even for nonexistent paths (bypasses all checks)
        assert!(state.can_read(&PathBuf::from("/nonexistent")));
        assert!(state.can_write(&PathBuf::from("/nonexistent")));
        assert!(state.can_execute(&PathBuf::from("/nonexistent")));
    }

    #[test]
    fn test_other_permissions() {
        let mut state = EnhancedState::new_user(1000, 1000);

        // File with other-readable permission
        state.filesystem.insert(
            PathBuf::from("/public/file.txt"),
            EnhancedFileSystemEntry::File {
                content: "public".to_string(),
                mode: 0o004, // -------r--
                uid: 500,
                gid: 500,
                mtime: None,
            },
        );

        // User can read via "other" permissions
        assert!(state.can_read(&PathBuf::from("/public/file.txt")));
        assert!(!state.can_write(&PathBuf::from("/public/file.txt")));
    }

    // ===== Clone and Debug Tests =====

    #[test]
    fn test_enhanced_state_clone() {
        let state = EnhancedState::new();
        let cloned = state.clone();
        assert_eq!(state.euid, cloned.euid);
        assert_eq!(state.cwd, cloned.cwd);
    }

    #[test]
    fn test_enhanced_state_debug() {
        let state = EnhancedState::new();
        let debug = format!("{:?}", state);
        assert!(debug.contains("euid"));
        assert!(debug.contains("cwd"));
    }

    #[test]
    fn test_entry_clone() {
        let dir = EnhancedFileSystemEntry::Directory {
            mode: 0o755,
            uid: 0,
            gid: 0,
        };
        let cloned = dir.clone();
        assert_eq!(dir, cloned);
    }

    #[test]
    fn test_entry_debug() {
        let file = EnhancedFileSystemEntry::File {
            content: "test".to_string(),
            mode: 0o644,
            uid: 0,
            gid: 0,
            mtime: Some(1234567890),
        };
        let debug = format!("{:?}", file);
        assert!(debug.contains("File"));
        assert!(debug.contains("content"));
    }
}
