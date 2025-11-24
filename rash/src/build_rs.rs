//! # build.rs Integration Module
//!
//! Provides utilities for automatic transpilation in Cargo build scripts.
//! This module enables zero-configuration transpilation of Rust files to shell scripts
//! during the build process.
//!
//! ## Quick Start
//!
//! Add to your `build.rs`:
//!
//! ```rust,no_run
//! // build.rs
//! use bashrs::build_rs::auto_transpile;
//!
//! fn main() {
//!     // Automatically transpile all files in hooks/ directory
//!     auto_transpile("hooks", ".git/hooks", 0o755)
//!         .expect("Failed to transpile git hooks");
//! }
//! ```
//!
//! ## Manual Discovery
//!
//! For more control, use the discovery API:
//!
//! ```rust,no_run
//! use bashrs::build_rs::{discover_sources, TranspileJob};
//!
//! fn main() {
//!     let sources = discover_sources("hooks", "**/*.rs").unwrap();
//!
//!     for job in sources {
//!         job.transpile().expect("Transpilation failed");
//!     }
//! }
//! ```

use crate::models::{Error, Result};
use crate::Transpiler;
use std::fs;
use std::path::{Path, PathBuf};

/// Auto-discover and transpile all Rust files in a directory.
///
/// This is the simplest way to integrate bashrs into your build process.
/// It automatically:
/// 1. Discovers all `.rs` files in the input directory (recursively)
/// 2. Transpiles them to shell scripts in the output directory
/// 3. Preserves directory structure
/// 4. Sets file permissions (Unix only)
/// 5. Prints cargo:rerun-if-changed directives
///
/// # Arguments
///
/// * `input_dir` - Directory containing Rust source files
/// * `output_dir` - Directory for generated shell scripts
/// * `permissions` - Unix file permissions for output files (e.g., 0o755)
///
/// # Returns
///
/// Number of files successfully transpiled.
///
/// # Errors
///
/// Returns an error if:
/// - Input directory doesn't exist
/// - File discovery fails
/// - Any transpilation fails
///
/// # Examples
///
/// ## Git hooks transpilation
///
/// ```rust,no_run
/// // build.rs
/// use bashrs::build_rs::auto_transpile;
///
/// // Transpile all hooks/*.rs to .git/hooks/*
/// auto_transpile("hooks", ".git/hooks", 0o755)
///     .expect("Failed to transpile git hooks");
/// ```
///
/// ## Custom installer scripts
///
/// ```rust,no_run
/// // build.rs
/// use bashrs::build_rs::auto_transpile;
///
/// // Transpile installer scripts
/// auto_transpile("install", "dist", 0o755)
///     .expect("Failed to transpile installers");
/// ```
pub fn auto_transpile<P: AsRef<Path>>(
    input_dir: P,
    output_dir: P,
    permissions: u32,
) -> Result<usize> {
    let input_path = input_dir.as_ref();
    let output_path = output_dir.as_ref();

    if !input_path.exists() {
        return Err(Error::ValidationError(format!(
            "Input directory does not exist: {}",
            input_path.display()
        )));
    }

    let jobs = discover_sources(input_path, "**/*.rs")?;
    let count = jobs.len();

    for job in jobs {
        // Print cargo rerun directive
        println!("cargo:rerun-if-changed={}", job.input.display());

        // Compute relative path and update output
        let relative = job
            .input
            .strip_prefix(input_path)
            .map_err(|e| Error::Internal(format!("Failed to compute relative path: {}", e)))?;

        let output_file = output_path.join(relative).with_extension("sh");

        Transpiler::new()
            .input(&job.input)
            .output(&output_file)
            .permissions(permissions)
            .config(job.config)
            .transpile()?;
    }

    Ok(count)
}

/// Discover Rust source files for transpilation.
///
/// This function recursively searches the input directory for Rust files
/// and creates a list of transpilation jobs.
///
/// # Arguments
///
/// * `input_dir` - Directory to search
/// * `pattern` - Glob pattern (e.g., "**/*.rs" for all .rs files)
///
/// # Returns
///
/// Vector of [`TranspileJob`] objects ready for transpilation.
///
/// # Examples
///
/// ```rust,no_run
/// use bashrs::build_rs::discover_sources;
///
/// let jobs = discover_sources("hooks", "**/*.rs").unwrap();
/// println!("Found {} files to transpile", jobs.len());
///
/// for job in jobs {
///     println!("  - {}", job.input.display());
/// }
/// ```
pub fn discover_sources<P: AsRef<Path>>(input_dir: P, pattern: &str) -> Result<Vec<TranspileJob>> {
    let input_path = input_dir.as_ref();

    if !input_path.exists() {
        return Err(Error::ValidationError(format!(
            "Input directory does not exist: {}",
            input_path.display()
        )));
    }

    let mut jobs = Vec::new();

    // Use a simple recursive directory walk
    // For production, consider using the `walkdir` or `glob` crate
    walk_dir(input_path, &mut |path| {
        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            // Check if pattern matches (simplified - just check extension for now)
            if pattern.ends_with("*.rs") || pattern.contains("**/*.rs") {
                jobs.push(TranspileJob {
                    input: path.to_path_buf(),
                    config: crate::Config::default(),
                });
            }
        }
    })?;

    Ok(jobs)
}

/// A transpilation job representing a single Rust file to transpile.
///
/// This struct is typically created by [`discover_sources`] but can also be
/// constructed manually for custom build workflows.
///
/// # Examples
///
/// ```rust,no_run
/// use bashrs::build_rs::TranspileJob;
/// use bashrs::Config;
///
/// let job = TranspileJob {
///     input: "src/install.rs".into(),
///     config: Config::default(),
/// };
///
/// // Transpile to custom output
/// job.transpile_to("dist/install.sh", 0o755).unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct TranspileJob {
    /// Path to the Rust source file
    pub input: PathBuf,
    /// Transpilation configuration
    pub config: crate::Config,
}

impl TranspileJob {
    /// Transpile this job to a specific output path.
    ///
    /// # Arguments
    ///
    /// * `output` - Path for the generated shell script
    /// * `permissions` - Unix file permissions (e.g., 0o755)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use bashrs::build_rs::TranspileJob;
    ///
    /// let job = TranspileJob {
    ///     input: "hooks/pre-commit.rs".into(),
    ///     config: Default::default(),
    /// };
    ///
    /// job.transpile_to(".git/hooks/pre-commit", 0o755).unwrap();
    /// ```
    pub fn transpile_to<P: AsRef<Path>>(&self, output: P, permissions: u32) -> Result<()> {
        Transpiler::new()
            .input(&self.input)
            .output(output.as_ref())
            .permissions(permissions)
            .config(self.config.clone())
            .transpile()
    }

    /// Transpile this job using the default output path.
    ///
    /// Output path is derived from the input path by replacing `.rs` with `.sh`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use bashrs::build_rs::TranspileJob;
    ///
    /// let job = TranspileJob {
    ///     input: "hooks/pre-commit.rs".into(),
    ///     config: Default::default(),
    /// };
    ///
    /// // Creates hooks/pre-commit.sh
    /// job.transpile().unwrap();
    /// ```
    pub fn transpile(&self) -> Result<()> {
        let output = self.input.with_extension("sh");
        self.transpile_to(output, 0o755)
    }
}

/// Walk a directory recursively and apply a function to each file.
fn walk_dir<P: AsRef<Path>, F>(dir: P, callback: &mut F) -> Result<()>
where
    F: FnMut(&Path),
{
    let dir_path = dir.as_ref();

    if !dir_path.is_dir() {
        return Ok(());
    }

    let entries = fs::read_dir(dir_path).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read directory {}: {}", dir_path.display(), e),
        ))
    })?;

    for entry in entries {
        let entry = entry.map_err(Error::Io)?;
        let path = entry.path();

        if path.is_dir() {
            // Recurse into subdirectories
            walk_dir(&path, callback)?;
        } else if path.is_file() {
            // Apply callback to files
            callback(&path);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // Test naming convention: test_<TASK_ID>_<feature>_<scenario>
    // TASK_ID: XTASK_002 (build.rs integration - Issue #25)

    #[test]
    fn test_XTASK_002_discover_sources_finds_rust_files() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create test files
        fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();
        fs::write(src_dir.join("lib.rs"), "pub fn hello() {}").unwrap();
        fs::write(src_dir.join("README.md"), "# Docs").unwrap();

        let jobs = discover_sources(&src_dir, "**/*.rs").unwrap();

        assert_eq!(jobs.len(), 2);
        assert!(jobs.iter().any(|j| j.input.ends_with("main.rs")));
        assert!(jobs.iter().any(|j| j.input.ends_with("lib.rs")));
    }

    #[test]
    fn test_XTASK_002_discover_sources_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create nested structure
        fs::create_dir_all(base.join("hooks/pre")).unwrap();
        fs::write(base.join("hooks/pre-commit.rs"), "fn main() {}").unwrap();
        fs::write(base.join("hooks/pre/push.rs"), "fn main() {}").unwrap();

        let jobs = discover_sources(base.join("hooks"), "**/*.rs").unwrap();

        assert_eq!(jobs.len(), 2);
    }

    #[test]
    fn test_XTASK_002_discover_sources_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let jobs = discover_sources(temp_dir.path(), "**/*.rs").unwrap();

        assert_eq!(jobs.len(), 0);
    }

    #[test]
    fn test_XTASK_002_discover_sources_nonexistent_directory() {
        let result = discover_sources("/nonexistent/path/12345", "**/*.rs");

        assert!(result.is_err());
        match result {
            Err(Error::ValidationError(msg)) => {
                assert!(msg.contains("does not exist"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_XTASK_002_transpile_job_basic() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.rs");
        let output = temp_dir.path().join("output.sh");

        fs::write(&input, "fn main() { let x = 1; }").unwrap();

        let job = TranspileJob {
            input: input.clone(),
            config: crate::Config::default(),
        };

        let result = job.transpile_to(&output, 0o755);
        assert!(result.is_ok());
        assert!(output.exists());
    }

    #[test]
    fn test_XTASK_002_auto_transpile_basic() {
        let temp_dir = TempDir::new().unwrap();
        let input_dir = temp_dir.path().join("src");
        let output_dir = temp_dir.path().join("dist");

        fs::create_dir(&input_dir).unwrap();
        fs::write(input_dir.join("install.rs"), "fn main() {}").unwrap();

        let count = auto_transpile(&input_dir, &output_dir, 0o755).unwrap();

        assert_eq!(count, 1);
        assert!(output_dir.join("install.sh").exists());
    }

    #[test]
    fn test_XTASK_002_auto_transpile_preserves_structure() {
        let temp_dir = TempDir::new().unwrap();
        let input_dir = temp_dir.path().join("hooks");
        let output_dir = temp_dir.path().join(".git/hooks");

        fs::create_dir_all(input_dir.join("pre")).unwrap();
        fs::write(input_dir.join("pre-commit.rs"), "fn main() {}").unwrap();
        fs::write(input_dir.join("pre/push.rs"), "fn main() {}").unwrap();

        let count = auto_transpile(&input_dir, &output_dir, 0o755).unwrap();

        assert_eq!(count, 2);
        assert!(output_dir.join("pre-commit.sh").exists());
        assert!(output_dir.join("pre/push.sh").exists());
    }

    #[test]
    #[cfg(unix)]
    fn test_XTASK_002_auto_transpile_sets_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let input_dir = temp_dir.path().join("src");
        let output_dir = temp_dir.path().join("dist");

        fs::create_dir(&input_dir).unwrap();
        fs::write(input_dir.join("script.rs"), "fn main() {}").unwrap();

        auto_transpile(&input_dir, &output_dir, 0o755).unwrap();

        let metadata = fs::metadata(output_dir.join("script.sh")).unwrap();
        let mode = metadata.permissions().mode() & 0o777;

        assert_eq!(mode, 0o755);
    }

    #[test]
    fn test_XTASK_002_auto_transpile_nonexistent_input() {
        let result = auto_transpile("/nonexistent/12345", "/tmp/output", 0o755);

        assert!(result.is_err());
        match result {
            Err(Error::ValidationError(msg)) => {
                assert!(msg.contains("does not exist"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_XTASK_002_walk_dir_basic() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("file1.txt"), "test").unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "test").unwrap();

        let mut count = 0;
        walk_dir(temp_dir.path(), &mut |_| {
            count += 1;
        })
        .unwrap();

        assert_eq!(count, 2);
    }

    #[test]
    fn test_XTASK_002_walk_dir_recursive() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("a/b/c")).unwrap();
        fs::write(temp_dir.path().join("a/file1.txt"), "test").unwrap();
        fs::write(temp_dir.path().join("a/b/file2.txt"), "test").unwrap();
        fs::write(temp_dir.path().join("a/b/c/file3.txt"), "test").unwrap();

        let mut count = 0;
        walk_dir(temp_dir.path(), &mut |_| {
            count += 1;
        })
        .unwrap();

        assert_eq!(count, 3);
    }
}
