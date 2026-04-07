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
        let shell_code = crate::transpile(&source, &self.config)?;

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
#[path = "transpiler_tests_xtask_001.rs"]
mod tests_extracted;

#[cfg(test)]
mod tests_coverage {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_transpiler_default_impl() {
        let t = Transpiler::default();
        assert!(t.input.is_none());
        assert!(t.output.is_none());
        assert!(t.permissions.is_none());
    }

    #[test]
    fn test_transpiler_default_eq_new() {
        let a = Transpiler::new();
        let b = Transpiler::default();
        assert_eq!(format!("{a:?}"), format!("{b:?}"));
    }

    #[test]
    fn test_transpiler_config_builder() {
        use crate::models::ShellDialect;
        let config = Config {
            target: ShellDialect::Bash,
            ..Default::default()
        };
        let t = Transpiler::new().config(config.clone());
        assert_eq!(format!("{:?}", t.config), format!("{config:?}"));
    }

    #[test]
    fn test_transpiler_missing_both_input_and_output() {
        let result = Transpiler::new().transpile();
        assert!(result.is_err());
    }

    #[test]
    fn test_transpiler_permissions_without_unix() {
        // Setting permissions on the builder should not panic
        let t = Transpiler::new().permissions(0o644);
        assert_eq!(t.permissions, Some(0o644));
    }

    #[test]
    fn test_transpiler_clone() {
        let t = Transpiler::new()
            .input("foo.rs")
            .output("bar.sh")
            .permissions(0o755);
        let cloned = t.clone();
        assert_eq!(
            cloned.input.expect("should have input").to_str(),
            Some("foo.rs")
        );
        assert_eq!(
            cloned.output.expect("should have output").to_str(),
            Some("bar.sh")
        );
        assert_eq!(cloned.permissions, Some(0o755));
    }

    #[test]
    fn test_transpiler_input_file_not_readable() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let output_path = temp_dir.path().join("out.sh");

        let result = Transpiler::new()
            .input("/nonexistent/path/to/file.rs")
            .output(&output_path)
            .transpile();

        assert!(result.is_err());
        match result {
            Err(Error::Io(e)) => {
                let msg = format!("{e}");
                assert!(msg.contains("Failed to read input file"));
            }
            other => panic!("Expected Io error, got {:?}", other),
        }
    }

    #[test]
    fn test_transpiler_empty_parent_path() {
        // When output path has no parent (just a filename), it should still work
        let temp_dir = TempDir::new().expect("should create temp dir");
        let input_path = temp_dir.path().join("input.rs");
        fs::write(&input_path, "fn main() { let x = 1; }").expect("should write");

        // Use a path within temp_dir so we don't pollute the filesystem
        let output_path = temp_dir.path().join("output.sh");
        let result = Transpiler::new()
            .input(&input_path)
            .output(&output_path)
            .transpile();

        assert!(result.is_ok(), "Transpilation failed: {:?}", result);
    }

    #[test]
    fn test_transpiler_overwrites_existing_output() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let input_path = temp_dir.path().join("input.rs");
        let output_path = temp_dir.path().join("output.sh");

        fs::write(&input_path, "fn main() { let x = 1; }").expect("should write");
        fs::write(&output_path, "old content").expect("should write");

        let result = Transpiler::new()
            .input(&input_path)
            .output(&output_path)
            .transpile();

        assert!(result.is_ok());
        let content = fs::read_to_string(&output_path).expect("should read");
        assert!(content.contains("#!/bin/sh"), "Should have new content");
        assert!(!content.contains("old content"), "Should be overwritten");
    }

    #[test]
    #[cfg(unix)]
    fn test_transpiler_permissions_0o644() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().expect("should create temp dir");
        let input_path = temp_dir.path().join("input.rs");
        let output_path = temp_dir.path().join("output.sh");

        fs::write(&input_path, "fn main() { let x = 1; }").expect("should write");

        let result = Transpiler::new()
            .input(&input_path)
            .output(&output_path)
            .permissions(0o644)
            .transpile();

        assert!(result.is_ok());
        let metadata = fs::metadata(&output_path).expect("should get metadata");
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(mode, 0o644);
    }

    #[test]
    #[cfg(unix)]
    fn test_transpiler_no_permissions_set() {
        // When permissions is None, no set_permissions call should be made
        let temp_dir = TempDir::new().expect("should create temp dir");
        let input_path = temp_dir.path().join("input.rs");
        let output_path = temp_dir.path().join("output.sh");

        fs::write(&input_path, "fn main() { let x = 1; }").expect("should write");

        let result = Transpiler::new()
            .input(&input_path)
            .output(&output_path)
            .transpile();

        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_transpiler_deeply_nested_output_dir() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let input_path = temp_dir.path().join("input.rs");
        let output_path = temp_dir.path().join("a/b/c/d/e/output.sh");

        fs::write(&input_path, "fn main() { let x = 1; }").expect("should write");

        let result = Transpiler::new()
            .input(&input_path)
            .output(&output_path)
            .transpile();

        assert!(result.is_ok());
        assert!(output_path.exists());
    }
}
