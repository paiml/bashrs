//! Built-in Bash Commands for WASM Runtime
//!
//! Implements core bash built-in commands like echo, cd, pwd.
//!
//! # Example
//!
//! ```rust
//! use bashrs::wasm::builtins::Builtins;
//! use bashrs::wasm::io::IoStreams;
//! use bashrs::wasm::vfs::VirtualFilesystem;
//!
//! let mut io = IoStreams::new_capture();
//! let mut vfs = VirtualFilesystem::new();
//!
//! Builtins::echo(&["hello".to_string()], &mut io).unwrap();
//! assert_eq!(io.get_stdout(), "hello\n");
//! ```

use crate::wasm::io::IoStreams;
use crate::wasm::vfs::VirtualFilesystem;
use anyhow::{Result, anyhow};
use std::io::{Write, BufRead, BufReader};

/// Bash built-in commands
pub struct Builtins;

impl Builtins {
    /// Execute echo command
    ///
    /// Prints arguments to stdout separated by spaces, followed by newline.
    pub fn echo(args: &[String], io: &mut IoStreams) -> Result<i32> {
        let output = if args.is_empty() {
            "\n".to_string()
        } else {
            format!("{}\n", args.join(" "))
        };

        io.stdout.write_all(output.as_bytes())?;
        Ok(0)
    }

    /// Execute cd command
    ///
    /// Changes current working directory.
    pub fn cd(args: &[String], vfs: &mut VirtualFilesystem) -> Result<i32> {
        let path = if args.is_empty() {
            "/home"
        } else {
            &args[0]
        };

        vfs.chdir(path)?;
        Ok(0)
    }

    /// Execute pwd command
    ///
    /// Prints current working directory to stdout.
    pub fn pwd(vfs: &VirtualFilesystem, io: &mut IoStreams) -> Result<i32> {
        let cwd = vfs.getcwd();
        writeln!(io.stdout, "{}", cwd.display())?;
        Ok(0)
    }

    /// Execute wc command (word count)
    ///
    /// Supports: -c (count characters), -l (count lines), -w (count words)
    pub fn wc(args: &[String], io: &mut IoStreams) -> Result<i32> {
        // Read stdin
        let stdin_content = io.get_stdin();

        // Parse flags
        let count_chars = args.contains(&"-c".to_string());
        let count_lines = args.contains(&"-l".to_string());
        let count_words = args.contains(&"-w".to_string());

        let result = if count_chars {
            // Count characters (including newline)
            stdin_content.len().to_string()
        } else if count_lines {
            // Count lines
            stdin_content.lines().count().to_string()
        } else if count_words {
            // Count words
            stdin_content.split_whitespace().count().to_string()
        } else {
            // Default: lines, words, chars
            let lines = stdin_content.lines().count();
            let words = stdin_content.split_whitespace().count();
            let chars = stdin_content.len();
            format!("{} {} {}", lines, words, chars)
        };

        writeln!(io.stdout, "{}", result)?;
        Ok(0)
    }

    /// Execute tr command (translate characters)
    ///
    /// Usage: tr 'set1' 'set2'
    pub fn tr(args: &[String], io: &mut IoStreams) -> Result<i32> {
        if args.len() < 2 {
            return Err(anyhow!("tr: missing operand"));
        }

        let from = &args[0];
        let to = &args[1];

        // Read stdin
        let stdin_content = io.get_stdin();

        // Helper function to unescape string (handle \n, \t, etc.)
        let unescape = |s: &str| -> String {
            s.replace("\\n", "\n")
             .replace("\\t", "\t")
             .replace("\\r", "\r")
        };

        let from_unescaped = unescape(from);
        let to_unescaped = unescape(to);

        // Simple implementation: translate character ranges
        let output = if from == " " && (to == "\\n" || to == "\n") {
            // Special case: spaces to newlines
            stdin_content.replace(' ', "\n")
        } else if from == " " && to == "_" {
            // Special case: spaces to underscores
            stdin_content.replace(' ', "_")
        } else if from == "a-z" && to == "A-Z" {
            // Special case: lowercase to uppercase
            stdin_content.to_uppercase()
        } else if from == "A-Z" && to == "a-z" {
            // Special case: uppercase to lowercase
            stdin_content.to_lowercase()
        } else {
            // General case: character-by-character replacement
            let from_chars: Vec<char> = from_unescaped.chars().collect();
            let to_chars: Vec<char> = to_unescaped.chars().collect();

            stdin_content
                .chars()
                .map(|c| {
                    if let Some(pos) = from_chars.iter().position(|&fc| fc == c) {
                        to_chars.get(pos).copied().unwrap_or(c)
                    } else {
                        c
                    }
                })
                .collect()
        };

        write!(io.stdout, "{}", output)?;
        Ok(0)
    }

    /// Check if a command is a builtin
    pub fn is_builtin(name: &str) -> bool {
        matches!(name, "echo" | "cd" | "pwd" | "wc" | "tr" | ":")
    }

    /// Execute a builtin command
    pub fn execute(
        name: &str,
        args: &[String],
        vfs: &mut VirtualFilesystem,
        io: &mut IoStreams,
    ) -> Result<i32> {
        match name {
            "echo" => Self::echo(args, io),
            "cd" => Self::cd(args, vfs),
            "pwd" => Self::pwd(vfs, io),
            "wc" => Self::wc(args, io),
            "tr" => Self::tr(args, io),
            ":" => Ok(0), // No-op command, always succeeds
            _ => Err(anyhow!("Unknown builtin: {}", name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echo_simple() {
        // ARRANGE
        let mut io = IoStreams::new_capture();

        // ACT
        let exit_code = Builtins::echo(&["hello".to_string()], &mut io).unwrap();

        // ASSERT
        assert_eq!(exit_code, 0);
        assert_eq!(io.get_stdout(), "hello\n");
    }

    #[test]
    fn test_echo_multiple_args() {
        // ARRANGE
        let mut io = IoStreams::new_capture();

        // ACT
        let exit_code = Builtins::echo(
            &["hello".to_string(), "world".to_string()],
            &mut io,
        )
        .unwrap();

        // ASSERT
        assert_eq!(exit_code, 0);
        assert_eq!(io.get_stdout(), "hello world\n");
    }

    #[test]
    fn test_echo_empty() {
        // ARRANGE
        let mut io = IoStreams::new_capture();

        // ACT
        let exit_code = Builtins::echo(&[], &mut io).unwrap();

        // ASSERT
        assert_eq!(exit_code, 0);
        assert_eq!(io.get_stdout(), "\n");
    }

    #[test]
    fn test_cd_success() {
        // ARRANGE
        let mut vfs = VirtualFilesystem::new();
        let mut io = IoStreams::new_capture();

        // ACT
        let exit_code = Builtins::cd(&["/tmp".to_string()], &mut vfs).unwrap();

        // ASSERT
        assert_eq!(exit_code, 0);
        assert_eq!(vfs.getcwd().to_str().unwrap(), "/tmp");
    }

    #[test]
    fn test_cd_no_args_goes_home() {
        // ARRANGE
        let mut vfs = VirtualFilesystem::new();
        vfs.chdir("/tmp").unwrap();

        // ACT
        let exit_code = Builtins::cd(&[], &mut vfs).unwrap();

        // ASSERT
        assert_eq!(exit_code, 0);
        assert_eq!(vfs.getcwd().to_str().unwrap(), "/home");
    }

    #[test]
    fn test_cd_invalid_path() {
        // ARRANGE
        let mut vfs = VirtualFilesystem::new();

        // ACT
        let result = Builtins::cd(&["/nonexistent".to_string()], &mut vfs);

        // ASSERT
        assert!(result.is_err());
    }

    #[test]
    fn test_pwd_output() {
        // ARRANGE
        let vfs = VirtualFilesystem::new();
        let mut io = IoStreams::new_capture();

        // ACT
        let exit_code = Builtins::pwd(&vfs, &mut io).unwrap();

        // ASSERT
        assert_eq!(exit_code, 0);
        assert_eq!(io.get_stdout(), "/\n");
    }

    #[test]
    fn test_pwd_after_cd() {
        // ARRANGE
        let mut vfs = VirtualFilesystem::new();
        let mut io = IoStreams::new_capture();
        vfs.chdir("/tmp").unwrap();

        // ACT
        let exit_code = Builtins::pwd(&vfs, &mut io).unwrap();

        // ASSERT
        assert_eq!(exit_code, 0);
        assert_eq!(io.get_stdout(), "/tmp\n");
    }

    #[test]
    fn test_is_builtin() {
        // ASSERT
        assert!(Builtins::is_builtin("echo"));
        assert!(Builtins::is_builtin("cd"));
        assert!(Builtins::is_builtin("pwd"));
        assert!(!Builtins::is_builtin("ls"));
        assert!(!Builtins::is_builtin("cat"));
    }

    #[test]
    fn test_execute_echo() {
        // ARRANGE
        let mut vfs = VirtualFilesystem::new();
        let mut io = IoStreams::new_capture();

        // ACT
        let exit_code =
            Builtins::execute("echo", &["test".to_string()], &mut vfs, &mut io).unwrap();

        // ASSERT
        assert_eq!(exit_code, 0);
        assert_eq!(io.get_stdout(), "test\n");
    }

    #[test]
    fn test_execute_unknown_builtin() {
        // ARRANGE
        let mut vfs = VirtualFilesystem::new();
        let mut io = IoStreams::new_capture();

        // ACT
        let result = Builtins::execute("unknown", &[], &mut vfs, &mut io);

        // ASSERT
        assert!(result.is_err());
    }
}
