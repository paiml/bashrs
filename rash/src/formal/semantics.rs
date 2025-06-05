//! Operational semantics for rash AST and POSIX shell
//!
//! This module defines the formal operational semantics for both
//! the tiny rash AST subset and the corresponding POSIX shell commands.

use crate::formal::{AbstractState, TinyAst};
use std::path::{Path, PathBuf};

/// Result of evaluating an AST node or shell command
pub type EvalResult = Result<AbstractState, String>;

/// Operational semantics for the tiny rash AST subset
pub mod rash_semantics {
    use super::*;

    /// Evaluate a rash AST node in a given state
    pub fn eval_rash(ast: &TinyAst, mut state: AbstractState) -> EvalResult {
        match ast {
            TinyAst::ExecuteCommand { command_name, args } => {
                eval_command(&mut state, command_name, args)?;
                Ok(state)
            }

            TinyAst::SetEnvironmentVariable { name, value } => {
                state.set_env(name.clone(), value.clone());
                Ok(state)
            }

            TinyAst::Sequence { commands } => {
                let mut current_state = state;
                for cmd in commands {
                    current_state = eval_rash(cmd, current_state)?;
                }
                Ok(current_state)
            }

            TinyAst::ChangeDirectory { path } => {
                let path_buf = PathBuf::from(path);
                state.change_directory(path_buf)?;
                Ok(state)
            }
        }
    }

    /// Execute a command in the abstract state
    pub fn eval_command(
        state: &mut AbstractState,
        command: &str,
        args: &[String],
    ) -> Result<(), String> {
        match command {
            "echo" => eval_echo_command(state, args),
            "mkdir" => eval_mkdir_command(state, args),
            "test" => eval_test_command(state, args),
            _ => eval_unknown_command(state, command),
        }
    }

    fn eval_echo_command(state: &mut AbstractState, args: &[String]) -> Result<(), String> {
        let output = if args.is_empty() {
            String::new()
        } else {
            args.join(" ")
        };
        state.write_stdout(output);
        Ok(())
    }

    fn eval_mkdir_command(state: &mut AbstractState, args: &[String]) -> Result<(), String> {
        let (parent_flag, paths) = parse_mkdir_args(state, args)?;

        for path_str in paths {
            let path = resolve_path(state, &path_str);
            create_directory_with_options(state, path, parent_flag)?;
        }
        Ok(())
    }

    fn parse_mkdir_args(
        state: &mut AbstractState,
        args: &[String],
    ) -> Result<(bool, Vec<String>), String> {
        let mut parent_flag = false;
        let mut paths = Vec::new();

        for arg in args.iter() {
            if arg == "-p" {
                parent_flag = true;
            } else if arg.starts_with('-') {
                state.write_stderr(format!("mkdir: invalid option -- '{arg}'"));
                state.exit_code = 1;
                return Err("Invalid option".to_string());
            } else {
                paths.push(arg.clone());
            }
        }
        Ok((parent_flag, paths))
    }

    fn resolve_path(state: &AbstractState, path_str: &str) -> PathBuf {
        if path_str.starts_with('/') {
            PathBuf::from(path_str)
        } else {
            state.cwd.join(path_str)
        }
    }

    fn create_directory_with_options(
        state: &mut AbstractState,
        path: PathBuf,
        parent_flag: bool,
    ) -> Result<(), String> {
        if parent_flag {
            state.create_directory(path)
        } else {
            validate_parent_exists(state, &path)?;
            state.create_directory(path)
        }
    }

    fn validate_parent_exists(state: &mut AbstractState, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            if !state.filesystem.contains_key(parent) {
                let error_msg = format!(
                    "mkdir: cannot create directory '{}': No such file or directory",
                    path.display()
                );
                state.write_stderr(error_msg);
                state.exit_code = 1;
                return Err("Parent directory does not exist".to_string());
            }
        }
        Ok(())
    }

    fn eval_test_command(state: &mut AbstractState, args: &[String]) -> Result<(), String> {
        if args.is_empty() {
            state.exit_code = 1;
            return Ok(());
        }

        match args[0].as_str() {
            "-d" => test_directory_exists(state, args),
            "-f" => test_file_exists(state, args),
            _ => {
                state.exit_code = 1;
                Ok(())
            }
        }
    }

    fn test_directory_exists(state: &mut AbstractState, args: &[String]) -> Result<(), String> {
        if args.len() < 2 {
            state.exit_code = 1;
            return Ok(());
        }

        let path = PathBuf::from(&args[1]);
        state.exit_code = match state.filesystem.get(&path) {
            Some(crate::formal::FileSystemEntry::Directory) => 0,
            _ => 1,
        };
        Ok(())
    }

    fn test_file_exists(state: &mut AbstractState, args: &[String]) -> Result<(), String> {
        if args.len() < 2 {
            state.exit_code = 1;
            return Ok(());
        }

        let path = PathBuf::from(&args[1]);
        state.exit_code = match state.filesystem.get(&path) {
            Some(crate::formal::FileSystemEntry::File(_)) => 0,
            _ => 1,
        };
        Ok(())
    }

    fn eval_unknown_command(state: &mut AbstractState, command: &str) -> Result<(), String> {
        state.write_stderr(format!("{command}: command not fully modeled"));
        state.exit_code = 0;
        Ok(())
    }
}

/// Operational semantics for POSIX shell commands
pub mod posix_semantics {
    use super::*;

    /// Evaluate a POSIX shell command string in a given state
    pub fn eval_posix(command: &str, mut state: AbstractState) -> EvalResult {
        // Parse the command string into components
        let parsed = parse_posix_command(command)?;

        for cmd in parsed {
            state = eval_single_posix_command(cmd, state)?;
        }

        Ok(state)
    }

    /// Simple POSIX command representation
    #[derive(Debug, Clone)]
    enum PosixCommand {
        SimpleCommand { name: String, args: Vec<String> },
        Assignment { name: String, value: String },
        ChangeDir { path: String },
    }

    /// Parse a POSIX command string (simplified for our tiny subset)
    fn parse_posix_command(command: &str) -> Result<Vec<PosixCommand>, String> {
        let mut commands = Vec::new();

        // Split by semicolons for sequential commands
        for cmd_str in command.split(';') {
            let cmd_str = cmd_str.trim();
            if cmd_str.is_empty() {
                continue;
            }

            // Check for variable assignment (VAR=value)
            if let Some(eq_pos) = cmd_str.find('=') {
                let (name, value) = cmd_str.split_at(eq_pos);
                let name = name.trim();
                let value = value[1..].trim(); // Skip the '='

                // Check if this is a valid assignment (no spaces in name)
                if !name.contains(' ') && crate::formal::TinyAst::validate_variable_name(name) {
                    // Remove quotes if present
                    let value = value.trim_matches('"').to_string();
                    commands.push(PosixCommand::Assignment {
                        name: name.to_string(),
                        value,
                    });
                    continue;
                }
            }

            // Check for cd command
            if let Some(path_part) = cmd_str.strip_prefix("cd ") {
                let path = path_part.trim().trim_matches('"');
                commands.push(PosixCommand::ChangeDir {
                    path: path.to_string(),
                });
                continue;
            }

            // Parse as simple command
            let parts = parse_command_line(cmd_str)?;
            if !parts.is_empty() {
                commands.push(PosixCommand::SimpleCommand {
                    name: parts[0].clone(),
                    args: parts[1..].to_vec(),
                });
            }
        }

        Ok(commands)
    }

    /// Parse a command line into words (simplified shell parsing)
    fn parse_command_line(line: &str) -> Result<Vec<String>, String> {
        let mut words = Vec::new();
        let mut current_word = String::new();
        let mut in_quotes = false;
        let mut escape_next = false;

        for ch in line.chars() {
            if escape_next {
                current_word.push(ch);
                escape_next = false;
                continue;
            }

            match ch {
                '\\' => {
                    escape_next = true;
                }
                '"' => {
                    if in_quotes {
                        // Closing quote - push the word even if empty
                        words.push(current_word.clone());
                        current_word.clear();
                    }
                    in_quotes = !in_quotes;
                }
                ' ' | '\t' => {
                    if in_quotes {
                        current_word.push(ch);
                    } else if !current_word.is_empty() {
                        words.push(current_word.clone());
                        current_word.clear();
                    }
                }
                _ => {
                    current_word.push(ch);
                }
            }
        }

        if in_quotes {
            return Err("Unterminated quote".to_string());
        }

        if !current_word.is_empty() {
            words.push(current_word);
        }

        Ok(words)
    }

    /// Evaluate a single POSIX command
    fn eval_single_posix_command(cmd: PosixCommand, mut state: AbstractState) -> EvalResult {
        match cmd {
            PosixCommand::SimpleCommand { name, args } => {
                // Delegate to rash semantics for consistency
                rash_semantics::eval_command(&mut state, &name, &args)?;
                Ok(state)
            }

            PosixCommand::Assignment { name, value } => {
                state.set_env(name, value);
                Ok(state)
            }

            PosixCommand::ChangeDir { path } => {
                state.change_directory(PathBuf::from(path))?;
                Ok(state)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rash_echo() {
        let ast = TinyAst::ExecuteCommand {
            command_name: "echo".to_string(),
            args: vec!["Hello".to_string(), "World".to_string()],
        };

        let initial_state = AbstractState::new();
        let result = rash_semantics::eval_rash(&ast, initial_state).unwrap();

        assert_eq!(result.stdout, vec!["Hello World"]);
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_rash_set_env() {
        let ast = TinyAst::SetEnvironmentVariable {
            name: "RASH_TEST".to_string(),
            value: "test_value".to_string(),
        };

        let initial_state = AbstractState::new();
        let result = rash_semantics::eval_rash(&ast, initial_state).unwrap();

        assert_eq!(result.get_env("RASH_TEST"), Some(&"test_value".to_string()));
    }

    #[test]
    fn test_rash_sequence() {
        let ast = TinyAst::Sequence {
            commands: vec![
                TinyAst::SetEnvironmentVariable {
                    name: "DIR".to_string(),
                    value: "/tmp/test".to_string(),
                },
                TinyAst::ExecuteCommand {
                    command_name: "mkdir".to_string(),
                    args: vec!["-p".to_string(), "/tmp/test".to_string()],
                },
            ],
        };

        let initial_state = AbstractState::new();
        let result = rash_semantics::eval_rash(&ast, initial_state).unwrap();

        assert_eq!(result.get_env("DIR"), Some(&"/tmp/test".to_string()));
        assert!(result.filesystem.contains_key(&PathBuf::from("/tmp/test")));
    }

    #[test]
    fn test_posix_echo() {
        let command = r#"echo "Hello World""#;
        let initial_state = AbstractState::new();
        let result = posix_semantics::eval_posix(command, initial_state).unwrap();

        assert_eq!(result.stdout, vec!["Hello World"]);
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_posix_assignment() {
        let command = "RASH_TEST=\"test_value\"";
        let initial_state = AbstractState::new();
        let result = posix_semantics::eval_posix(command, initial_state).unwrap();

        assert_eq!(result.get_env("RASH_TEST"), Some(&"test_value".to_string()));
    }

    #[test]
    fn test_posix_sequence() {
        let command = "DIR=\"/tmp/test\"; mkdir -p /tmp/test";
        let initial_state = AbstractState::new();
        let result = posix_semantics::eval_posix(command, initial_state).unwrap();

        assert_eq!(result.get_env("DIR"), Some(&"/tmp/test".to_string()));
        assert!(result.filesystem.contains_key(&PathBuf::from("/tmp/test")));
    }
}
