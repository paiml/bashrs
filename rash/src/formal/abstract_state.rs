//! Abstract state representation for formal verification
//!
//! This module defines the abstract machine state used to formally
//! specify the semantics of both rash AST and POSIX shell commands.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Abstract representation of the system state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbstractState {
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

    /// Abstract filesystem representation (path -> content)
    /// For simplicity, we only track directories and text files
    pub filesystem: HashMap<PathBuf, FileSystemEntry>,
}

/// Entry in the abstract filesystem
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileSystemEntry {
    /// Directory
    Directory,
    /// Text file with content
    File(String),
}

impl Default for AbstractState {
    fn default() -> Self {
        let mut filesystem = HashMap::new();
        // Initialize with root directory
        filesystem.insert(PathBuf::from("/"), FileSystemEntry::Directory);

        Self {
            env: HashMap::new(),
            cwd: PathBuf::from("/"),
            stdout: Vec::new(),
            stderr: Vec::new(),
            exit_code: 0,
            filesystem,
        }
    }
}

impl AbstractState {
    /// Create a new abstract state with basic initialization
    pub fn new() -> Self {
        Self::default()
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
            Some(FileSystemEntry::Directory) => {
                self.cwd = path;
                self.exit_code = 0;
                Ok(())
            }
            Some(FileSystemEntry::File(_)) => {
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

    /// Create a directory (mkdir -p behavior)
    pub fn create_directory(&mut self, path: PathBuf) -> Result<(), String> {
        // Create all parent directories as well
        let mut current = PathBuf::new();
        for component in path.components() {
            current.push(component);
            if !self.filesystem.contains_key(&current) {
                self.filesystem
                    .insert(current.clone(), FileSystemEntry::Directory);
            } else if let Some(FileSystemEntry::File(_)) = self.filesystem.get(&current) {
                self.stderr.push(format!(
                    "mkdir: cannot create directory '{}': File exists",
                    current.display()
                ));
                self.exit_code = 1;
                return Err("File exists".to_string());
            }
        }
        self.exit_code = 0;
        Ok(())
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
        // For formal verification, we consider states equivalent if:
        // 1. Environment variables are the same
        // 2. Current working directory is the same
        // 3. Exit codes are the same
        // 4. Filesystem state is the same
        // 5. Output buffers contain the same content (order matters)

        self.env == other.env
            && self.cwd == other.cwd
            && self.exit_code == other.exit_code
            && self.filesystem == other.filesystem
            && self.stdout == other.stdout
            && self.stderr == other.stderr
    }

    /// Create a test state with common setup
    pub fn test_state() -> Self {
        let mut state = Self::new();
        // Add common directories
        state
            .filesystem
            .insert(PathBuf::from("/tmp"), FileSystemEntry::Directory);
        state
            .filesystem
            .insert(PathBuf::from("/home"), FileSystemEntry::Directory);
        state
            .filesystem
            .insert(PathBuf::from("/opt"), FileSystemEntry::Directory);
        // Add common environment variables
        state.set_env("PATH".to_string(), "/usr/bin:/bin".to_string());
        state.set_env("HOME".to_string(), "/home/user".to_string());
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = AbstractState::new();
        assert_eq!(state.cwd, PathBuf::from("/"));
        assert_eq!(state.exit_code, 0);
        assert!(state.stdout.is_empty());
        assert!(state.stderr.is_empty());
        assert!(state.filesystem.contains_key(&PathBuf::from("/")));
    }

    #[test]
    fn test_environment_variables() {
        let mut state = AbstractState::new();
        state.set_env("RASH_VERSION".to_string(), "1.0.0".to_string());
        assert_eq!(state.get_env("RASH_VERSION"), Some(&"1.0.0".to_string()));
        assert_eq!(state.get_env("NONEXISTENT"), None);
    }

    #[test]
    fn test_change_directory() {
        let mut state = AbstractState::test_state();

        // Change to existing directory
        assert!(state.change_directory(PathBuf::from("/tmp")).is_ok());
        assert_eq!(state.cwd, PathBuf::from("/tmp"));
        assert_eq!(state.exit_code, 0);

        // Try to change to non-existent directory
        assert!(state
            .change_directory(PathBuf::from("/nonexistent"))
            .is_err());
        assert_eq!(state.cwd, PathBuf::from("/tmp")); // Should not change
        assert_eq!(state.exit_code, 1);
        assert!(!state.stderr.is_empty());
    }

    #[test]
    fn test_create_directory() {
        let mut state = AbstractState::new();

        // Create nested directories
        assert!(state
            .create_directory(PathBuf::from("/opt/rash/bin"))
            .is_ok());
        assert!(state.filesystem.contains_key(&PathBuf::from("/opt")));
        assert!(state.filesystem.contains_key(&PathBuf::from("/opt/rash")));
        assert!(state
            .filesystem
            .contains_key(&PathBuf::from("/opt/rash/bin")));
        assert_eq!(state.exit_code, 0);
    }

    #[test]
    fn test_state_equivalence() {
        let mut state1 = AbstractState::test_state();
        let mut state2 = AbstractState::test_state();

        assert!(state1.is_equivalent(&state2));

        // Different environment variable
        state1.set_env("VAR".to_string(), "value".to_string());
        assert!(!state1.is_equivalent(&state2));

        state2.set_env("VAR".to_string(), "value".to_string());
        assert!(state1.is_equivalent(&state2));

        // Different stdout
        state1.write_stdout("Hello".to_string());
        assert!(!state1.is_equivalent(&state2));
    }
}
