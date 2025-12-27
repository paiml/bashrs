//! # Transpiler Builder API
//!
//! Provides a builder pattern for programmatic transpilation in build scripts, xtask commands,
//! and CI/CD pipelines. This is the recommended API for integrating bashrs into your build process.
//!
//! ## Examples
//!
//! ### Basic Usage
//!
//! ```rust,no_run
//! use bashrs::Transpiler;
//!
//! # fn main() -> bashrs::Result<()> {
//! Transpiler::new()
//!     .input("src/install.rs")
//!     .output("target/install.sh")
//!     .transpile()?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Git Hooks in xtask
//!
//! ```rust,no_run
//! // xtask/src/main.rs
//! use bashrs::Transpiler;
//!
//! fn transpile_hooks() -> bashrs::Result<()> {
//!     Transpiler::new()
//!         .input("hooks/pre-commit.rs")
//!         .output(".git/hooks/pre-commit")
//!         .permissions(0o755)
//!         .transpile()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### build.rs Integration
//!
//! ```rust,no_run
//! // build.rs
//! use bashrs::Transpiler;
//!
//! fn main() {
//!     println!("cargo:rerun-if-changed=src/install.rs");
//!
//!     Transpiler::new()
//!         .input("src/install.rs")
//!         .output("target/install.sh")
//!         .permissions(0o755)
//!         .transpile()
//!         .expect("Failed to transpile install script");
//! }
//! ```

use crate::models::{Config, Error, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Builder for transpiling Rust files to shell scripts with full control over I/O and permissions.
///
/// This API is designed for programmatic use in build scripts, xtask commands, and CI/CD pipelines.
/// It provides a fluent interface for configuring transpilation with proper error handling and
/// file system integration.
///
/// # Examples
///
/// ## Basic transpilation
///
/// ```rust,no_run
/// use bashrs::Transpiler;
///
/// # fn main() -> bashrs::Result<()> {
/// Transpiler::new()
///     .input("src/install.rs")
///     .output("target/install.sh")
///     .transpile()?;
/// # Ok(())
/// # }
/// ```
///
/// ## With custom permissions (Unix only)
///
/// ```rust,no_run
/// use bashrs::Transpiler;
///
/// # fn main() -> bashrs::Result<()> {
/// Transpiler::new()
///     .input("hooks/pre-commit.rs")
///     .output(".git/hooks/pre-commit")
///     .permissions(0o755)  // rwxr-xr-x
///     .transpile()?;
/// # Ok(())
/// # }
/// ```
///
/// ## With custom configuration
///
/// ```rust,no_run
/// use bashrs::{Transpiler, Config};
/// use bashrs::models::{ShellDialect, VerificationLevel};
///
/// # fn main() -> bashrs::Result<()> {
/// let config = Config {
///     target: ShellDialect::Posix,
///     verify: VerificationLevel::Strict,
///     optimize: true,
///     ..Default::default()
/// };
///
/// Transpiler::new()
///     .input("src/main.rs")
///     .output("dist/script.sh")
///     .config(config)
///     .transpile()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Transpiler {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    permissions: Option<u32>,
    config: Config,
}

impl Transpiler {
    /// Create a new transpiler builder with default configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bashrs::Transpiler;
    ///
    /// let transpiler = Transpiler::new();
    /// ```
    pub fn new() -> Self {
        Self {
            input: None,
            output: None,
            permissions: None,
            config: Config::default(),
        }
    }

    /// Set the input Rust file path.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bashrs::Transpiler;
    ///
    /// let transpiler = Transpiler::new()
    ///     .input("src/install.rs");
    /// ```
    pub fn input<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.input = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the output shell script path.
    ///
    /// The parent directory will be created automatically if it doesn't exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bashrs::Transpiler;
    ///
    /// let transpiler = Transpiler::new()
    ///     .output("target/script.sh");
    /// ```
    pub fn output<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.output = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the file permissions for the output file (Unix only).
    ///
    /// On non-Unix platforms, this setting is ignored.
    ///
    /// # Arguments
    ///
    /// * `mode` - Unix file permissions in octal format (e.g., 0o755 for rwxr-xr-x)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bashrs::Transpiler;
    ///
    /// // Make script executable (rwxr-xr-x)
    /// let transpiler = Transpiler::new()
    ///     .permissions(0o755);
    ///
    /// // Read-only for everyone (r--r--r--)
    /// let transpiler = Transpiler::new()
    ///     .permissions(0o444);
    /// ```
    pub fn permissions(mut self, mode: u32) -> Self {
        self.permissions = Some(mode);
        self
    }

    /// Set the transpilation configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bashrs::{Transpiler, Config};
    /// use bashrs::models::ShellDialect;
    ///
    /// let config = Config {
    ///     target: ShellDialect::Bash,
    ///     ..Default::default()
    /// };
    ///
    /// let transpiler = Transpiler::new()
    ///     .config(config);
    /// ```
    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    /// Transpile the input file to a shell script.
    ///
    /// This method:
    /// 1. Reads the input Rust file
    /// 2. Transpiles it to shell script
    /// 3. Creates the output directory if needed
    /// 4. Writes the shell script to the output file
    /// 5. Sets file permissions if specified (Unix only)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Input or output path not set
    /// - Input file cannot be read
    /// - Transpilation fails (parse errors, validation errors, etc.)
    /// - Output directory cannot be created
    /// - Output file cannot be written
    /// - Permissions cannot be set (Unix only)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use bashrs::Transpiler;
    ///
    /// # fn main() -> bashrs::Result<()> {
    /// Transpiler::new()
    ///     .input("src/install.rs")
    ///     .output("target/install.sh")
    ///     .permissions(0o755)
    ///     .transpile()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn transpile(self) -> Result<()> {
        // Validate required fields
        let input = self
            .input
            .ok_or_else(|| Error::ValidationError("Input path not set".to_string()))?;
        let output = self
            .output
            .ok_or_else(|| Error::ValidationError("Output path not set".to_string()))?;

        // Read input file
        let source = fs::read_to_string(&input).map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read input file {}: {}", input.display(), e),
            ))
        })?;

        // Transpile to shell script
        let shell_code = crate::transpile(&source, self.config)?;

        // Ensure output directory exists
        if let Some(parent) = output.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| {
                    Error::Io(std::io::Error::new(
                        e.kind(),
                        format!(
                            "Failed to create output directory {}: {}",
                            parent.display(),
                            e
                        ),
                    ))
                })?;
            }
        }

        // Write output file
        fs::write(&output, shell_code).map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to write output file {}: {}", output.display(), e),
            ))
        })?;

        // Set permissions if specified (Unix only)
        #[cfg(unix)]
        if let Some(mode) = self.permissions {
            use std::fs::Permissions;
            let perms = Permissions::from_mode(mode);
            fs::set_permissions(&output, perms).map_err(|e| {
                Error::Io(std::io::Error::new(
                    e.kind(),
                    format!("Failed to set permissions on {}: {}", output.display(), e),
                ))
            })?;
        }

        Ok(())
    }
}

impl Default for Transpiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]
    use super::*;
    use tempfile::TempDir;

    // Test naming convention: test_<TASK_ID>_<feature>_<scenario>
    // TASK_ID: XTASK_001 (xtask integration - Issue #25)

    #[test]
    fn test_XTASK_001_transpiler_new_has_defaults() {
        let transpiler = Transpiler::new();
        assert!(transpiler.input.is_none());
        assert!(transpiler.output.is_none());
        assert!(transpiler.permissions.is_none());
    }

    #[test]
    fn test_XTASK_001_transpiler_builder_sets_input() {
        let transpiler = Transpiler::new().input("src/main.rs");
        assert_eq!(transpiler.input.unwrap().to_str().unwrap(), "src/main.rs");
    }

    #[test]
    fn test_XTASK_001_transpiler_builder_sets_output() {
        let transpiler = Transpiler::new().output("target/script.sh");
        assert_eq!(
            transpiler.output.unwrap().to_str().unwrap(),
            "target/script.sh"
        );
    }

    #[test]
    fn test_XTASK_001_transpiler_builder_sets_permissions() {
        let transpiler = Transpiler::new().permissions(0o755);
        assert_eq!(transpiler.permissions, Some(0o755));
    }

    #[test]
    fn test_XTASK_001_transpiler_builder_fluent_interface() {
        let transpiler = Transpiler::new()
            .input("src/main.rs")
            .output("target/script.sh")
            .permissions(0o755);

        assert!(transpiler.input.is_some());
        assert!(transpiler.output.is_some());
        assert_eq!(transpiler.permissions, Some(0o755));
    }

    #[test]
    fn test_XTASK_001_transpiler_requires_input() {
        let result = Transpiler::new().output("out.sh").transpile();

        assert!(result.is_err());
        match result {
            Err(Error::ValidationError(msg)) => assert!(msg.contains("Input path not set")),
            _ => panic!("Expected ValidationError for missing input"),
        }
    }

    #[test]
    fn test_XTASK_001_transpiler_requires_output() {
        let result = Transpiler::new().input("in.rs").transpile();

        assert!(result.is_err());
        match result {
            Err(Error::ValidationError(msg)) => assert!(msg.contains("Output path not set")),
            _ => panic!("Expected ValidationError for missing output"),
        }
    }

    #[test]
    fn test_XTASK_001_transpiler_basic_transpilation() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.rs");
        let output_path = temp_dir.path().join("output.sh");

        // Write a simple Rust file
        fs::write(
            &input_path,
            r#"
            fn main() {
                let greeting = "Hello, World!";
                echo(greeting);
            }
            fn echo(msg: &str) {}
            "#,
        )
        .unwrap();

        // Transpile
        let result = Transpiler::new()
            .input(&input_path)
            .output(&output_path)
            .transpile();

        assert!(result.is_ok(), "Transpilation failed: {:?}", result);
        assert!(output_path.exists(), "Output file not created");

        let shell_code = fs::read_to_string(&output_path).unwrap();
        assert!(shell_code.contains("#!/bin/sh"), "Missing shebang");
    }

    #[test]
    fn test_XTASK_001_transpiler_creates_output_directory() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.rs");
        let output_path = temp_dir.path().join("nested/deep/output.sh");

        // Write a simple Rust file
        fs::write(
            &input_path,
            r#"
            fn main() {
                let x = 42;
            }
            "#,
        )
        .unwrap();

        // Transpile (should create nested directories)
        let result = Transpiler::new()
            .input(&input_path)
            .output(&output_path)
            .transpile();

        assert!(result.is_ok(), "Transpilation failed: {:?}", result);
        assert!(output_path.exists(), "Output file not created");
        assert!(
            output_path.parent().unwrap().exists(),
            "Output directory not created"
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_XTASK_001_transpiler_sets_permissions_unix() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.rs");
        let output_path = temp_dir.path().join("output.sh");

        // Write a simple Rust file
        fs::write(&input_path, "fn main() {}").unwrap();

        // Transpile with permissions
        let result = Transpiler::new()
            .input(&input_path)
            .output(&output_path)
            .permissions(0o755)
            .transpile();

        assert!(result.is_ok(), "Transpilation failed: {:?}", result);
        assert!(output_path.exists(), "Output file not created");

        // Check permissions
        let metadata = fs::metadata(&output_path).unwrap();
        let permissions = metadata.permissions();
        let mode = permissions.mode() & 0o777;

        assert_eq!(mode, 0o755, "Permissions not set correctly");
    }

    #[test]
    fn test_XTASK_001_transpiler_handles_invalid_rust() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("invalid.rs");
        let output_path = temp_dir.path().join("output.sh");

        // Write invalid Rust code
        fs::write(&input_path, "fn main( { }").unwrap();

        // Transpile should fail
        let result = Transpiler::new()
            .input(&input_path)
            .output(&output_path)
            .transpile();

        assert!(result.is_err(), "Expected transpilation to fail");
    }

    #[test]
    fn test_XTASK_001_transpiler_handles_missing_input_file() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("nonexistent.rs");
        let output_path = temp_dir.path().join("output.sh");

        let result = Transpiler::new()
            .input(&input_path)
            .output(&output_path)
            .transpile();

        assert!(result.is_err(), "Expected error for missing input");
        match result {
            Err(Error::Io(_)) => (), // Expected
            _ => panic!("Expected IO error for missing input file"),
        }
    }

    #[test]
    fn test_XTASK_001_transpiler_with_custom_config() {
        use crate::models::{ShellDialect, VerificationLevel};

        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.rs");
        let output_path = temp_dir.path().join("output.sh");

        fs::write(&input_path, "fn main() { let x = 1; }").unwrap();

        let config = Config {
            target: ShellDialect::Posix,
            verify: VerificationLevel::Strict,
            optimize: true,
            ..Default::default()
        };

        let result = Transpiler::new()
            .input(&input_path)
            .output(&output_path)
            .config(config)
            .transpile();

        assert!(
            result.is_ok(),
            "Transpilation with config failed: {:?}",
            result
        );
        assert!(output_path.exists());
    }
}
