//! Shell configuration file management
//!
//! This module provides functionality to analyze, lint, and purify shell
//! configuration files like ~/.bashrc, ~/.zshrc, ~/.profile, etc.
//!
//! Key capabilities:
//! - Deduplicate PATH entries
//! - Quote variable expansions
//! - Remove non-deterministic constructs
//! - Performance optimization (lazy-loading)
//! - Cross-shell compatibility checking

pub mod analyzer;
pub mod deduplicator;
pub mod purifier;
pub mod quoter; // CONFIG-002: Quote variable expansions

use std::path::PathBuf;

/// Configuration file types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigType {
    /// ~/.bashrc - Bash interactive shell
    Bashrc,
    /// ~/.bash_profile - Bash login shell
    BashProfile,
    /// ~/.zshrc - Zsh interactive shell
    Zshrc,
    /// ~/.zprofile - Zsh login shell
    Zprofile,
    /// ~/.profile - POSIX login shell
    Profile,
    /// Generic shell script
    Generic,
}

impl ConfigType {
    /// Detect config type from file path
    pub fn from_path(path: &PathBuf) -> Self {
        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

        match filename {
            ".bashrc" | "bashrc" => ConfigType::Bashrc,
            ".bash_profile" | "bash_profile" => ConfigType::BashProfile,
            ".zshrc" | "zshrc" => ConfigType::Zshrc,
            ".zprofile" | "zprofile" => ConfigType::Zprofile,
            ".profile" | "profile" => ConfigType::Profile,
            _ => ConfigType::Generic,
        }
    }

    /// Get expected shell for this config type
    pub fn expected_shell(&self) -> &'static str {
        match self {
            ConfigType::Bashrc | ConfigType::BashProfile => "bash",
            ConfigType::Zshrc | ConfigType::Zprofile => "zsh",
            ConfigType::Profile => "sh",
            ConfigType::Generic => "sh",
        }
    }
}

/// Analysis result for a configuration file
#[derive(Debug, Clone)]
pub struct ConfigAnalysis {
    pub file_path: PathBuf,
    pub config_type: ConfigType,
    pub line_count: usize,
    pub complexity_score: u8,
    pub issues: Vec<ConfigIssue>,
    pub path_entries: Vec<PathEntry>,
    pub performance_issues: Vec<PerformanceIssue>,
}

/// A specific issue found in the config
#[derive(Debug, Clone)]
pub struct ConfigIssue {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// A PATH entry found in the config
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathEntry {
    pub line: usize,
    pub path: String,
    pub is_duplicate: bool,
}

/// Performance issue detected
#[derive(Debug, Clone)]
pub struct PerformanceIssue {
    pub line: usize,
    pub command: String,
    pub estimated_cost_ms: u32,
    pub suggestion: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_type_from_path() {
        assert_eq!(
            ConfigType::from_path(&PathBuf::from("/home/user/.bashrc")),
            ConfigType::Bashrc
        );
        assert_eq!(
            ConfigType::from_path(&PathBuf::from("/home/user/.zshrc")),
            ConfigType::Zshrc
        );
        assert_eq!(
            ConfigType::from_path(&PathBuf::from("/home/user/.profile")),
            ConfigType::Profile
        );
    }

    #[test]
    fn test_expected_shell() {
        assert_eq!(ConfigType::Bashrc.expected_shell(), "bash");
        assert_eq!(ConfigType::Zshrc.expected_shell(), "zsh");
        assert_eq!(ConfigType::Profile.expected_shell(), "sh");
    }
}
