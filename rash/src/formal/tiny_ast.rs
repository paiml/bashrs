//! Tiny subset of rash AST for formal verification
//!
//! This module defines a minimal subset of the rash AST that is sufficient
//! for basic bootstrap scripts and amenable to formal verification.

use serde::{Deserialize, Serialize};

/// The tiny subset of rash AST nodes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TinyAst {
    /// Execute a simple command with fixed arguments
    ExecuteCommand {
        /// Command name from a restricted allow-list
        command_name: String,
        /// List of literal string arguments
        args: Vec<String>,
    },

    /// Set an environment variable
    SetEnvironmentVariable {
        /// Variable name (valid POSIX variable name)
        name: String,
        /// Literal string value
        value: String,
    },

    /// Sequential execution of commands
    Sequence {
        /// List of commands to execute in order
        commands: Vec<TinyAst>,
    },

    /// Change the current directory
    ChangeDirectory {
        /// Absolute or simple relative path
        path: String,
    },
}

/// Restricted list of allowed commands for bootstrap scripts
pub const ALLOWED_COMMANDS: &[&str] = &[
    "mkdir",
    "echo",
    "rm",
    "cp",
    "mv",
    "chmod",
    "chown",
    "id",
    "test",
    "wget",
    "curl",
    "tar",
    "gzip",
    "gunzip",
    "sha256sum",
    "sha512sum",
];

impl TinyAst {
    /// Validate that a command is in the allowed list
    pub fn validate_command(command: &str) -> bool {
        ALLOWED_COMMANDS.contains(&command)
    }

    /// Validate a variable name according to POSIX rules
    pub fn validate_variable_name(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // Must start with letter or underscore
        let first_char = name.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return false;
        }

        // Rest must be alphanumeric or underscore
        name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    /// Check if the AST node is valid
    pub fn is_valid(&self) -> bool {
        match self {
            TinyAst::ExecuteCommand { command_name, .. } => Self::validate_command(command_name),
            TinyAst::SetEnvironmentVariable { name, .. } => Self::validate_variable_name(name),
            TinyAst::Sequence { commands } => {
                !commands.is_empty() && commands.iter().all(|cmd| cmd.is_valid())
            }
            TinyAst::ChangeDirectory { path } => !path.is_empty() && !path.contains('\0'),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_command() {
        assert!(TinyAst::validate_command("echo"));
        assert!(TinyAst::validate_command("mkdir"));
        assert!(!TinyAst::validate_command("sudo"));
        assert!(!TinyAst::validate_command("eval"));
    }

    #[test]
    fn test_validate_variable_name() {
        assert!(TinyAst::validate_variable_name("PATH"));
        assert!(TinyAst::validate_variable_name("_var"));
        assert!(TinyAst::validate_variable_name("var123"));
        assert!(!TinyAst::validate_variable_name("123var"));
        assert!(!TinyAst::validate_variable_name("var-name"));
        assert!(!TinyAst::validate_variable_name(""));
    }

    #[test]
    fn test_ast_validation() {
        let valid_echo = TinyAst::ExecuteCommand {
            command_name: "echo".to_string(),
            args: vec!["Hello".to_string()],
        };
        assert!(valid_echo.is_valid());

        let invalid_command = TinyAst::ExecuteCommand {
            command_name: "sudo".to_string(),
            args: vec![],
        };
        assert!(!invalid_command.is_valid());

        let valid_var = TinyAst::SetEnvironmentVariable {
            name: "INSTALL_DIR".to_string(),
            value: "/opt/rash".to_string(),
        };
        assert!(valid_var.is_valid());

        let invalid_var = TinyAst::SetEnvironmentVariable {
            name: "install-dir".to_string(),
            value: "/opt/rash".to_string(),
        };
        assert!(!invalid_var.is_valid());
    }
}
