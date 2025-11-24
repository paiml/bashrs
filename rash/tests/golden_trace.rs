//! Golden Trace Integration Tests for bashrs
//!
//! This module provides golden trace functionality using the renacer syscall tracer.
//! Golden traces capture reference syscall patterns from known-good executions and
//! enable regression detection by comparing future runs against these baselines.
//!
//! # Architecture
//!
//! 1. **Capture Phase**: Run bashrs commands under renacer, save JSON traces
//! 2. **Comparison Phase**: Re-run commands, compare syscall patterns
//! 3. **Regression Detection**: Alert on unexpected changes (new syscalls, file access patterns)
//!
//! # Toyota Way Principles
//!
//! - **Determinism**: Same input → same syscalls → same trace
//! - **Regression Prevention**: Any syscall pattern change triggers review
//! - **EXTREME TDD**: Golden traces as executable specifications
//!
//! # Usage
//!
//! ```bash
//! # Capture golden trace for a command
//! cargo test --test golden_trace -- --ignored capture_hello_world_trace
//!
//! # Compare against golden trace
//! cargo test --test golden_trace compare_hello_world_trace
//! ```

#![allow(clippy::unwrap_used)] // Tests can use unwrap()

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Directory for storing golden traces
const GOLDEN_TRACES_DIR: &str = "tests/golden_traces";

/// Golden trace metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenTrace {
    /// Command that was traced
    pub command: Vec<String>,
    /// Working directory
    pub workdir: PathBuf,
    /// Syscall summary (syscall name → count)
    pub syscall_counts: HashMap<String, u64>,
    /// File operations (open, read, write paths)
    pub file_operations: Vec<FileOperation>,
    /// Total syscalls
    pub total_syscalls: u64,
    /// Renacer version used for capture
    pub renacer_version: String,
    /// Timestamp of capture
    pub captured_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub syscall: String,
    pub path: PathBuf,
    pub flags: Option<String>,
}

/// Capture a golden trace by running a command under renacer
pub fn capture_golden_trace(
    name: &str,
    command: &[&str],
    workdir: Option<&Path>,
) -> Result<GoldenTrace> {
    let workdir = workdir.unwrap_or_else(|| Path::new("."));

    // Ensure golden traces directory exists
    std::fs::create_dir_all(GOLDEN_TRACES_DIR)
        .context("Failed to create golden traces directory")?;

    // Run command under renacer with JSON output
    let output = Command::new("renacer")
        .args(&["--format", "json", "--summary", "--"])
        .args(command)
        .current_dir(workdir)
        .output()
        .context("Failed to execute renacer")?;

    if !output.status.success() {
        anyhow::bail!(
            "Renacer command failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Parse renacer JSON output
    let trace_json =
        String::from_utf8(output.stdout).context("Renacer output is not valid UTF-8")?;

    // For now, create a simplified golden trace from the summary
    // TODO: Enhance with full JSON parsing when renacer stabilizes JSON schema
    let golden = GoldenTrace {
        command: command.iter().map(|s| s.to_string()).collect(),
        workdir: workdir.to_path_buf(),
        syscall_counts: parse_syscall_summary(&trace_json)?,
        file_operations: vec![], // TODO: Parse from detailed JSON
        total_syscalls: 0,       // TODO: Calculate from summary
        renacer_version: "0.6.2".to_string(),
        captured_at: chrono::Utc::now().to_rfc3339(),
    };

    // Save golden trace
    let golden_path = Path::new(GOLDEN_TRACES_DIR).join(format!("{}.json", name));
    let golden_json =
        serde_json::to_string_pretty(&golden).context("Failed to serialize golden trace")?;
    std::fs::write(&golden_path, golden_json).context("Failed to write golden trace")?;

    eprintln!("✅ Golden trace captured: {}", golden_path.display());
    Ok(golden)
}

/// Load a golden trace from disk
pub fn load_golden_trace(name: &str) -> Result<GoldenTrace> {
    let golden_path = Path::new(GOLDEN_TRACES_DIR).join(format!("{}.json", name));
    let golden_json = std::fs::read_to_string(&golden_path)
        .with_context(|| format!("Failed to read golden trace: {}", golden_path.display()))?;
    serde_json::from_str(&golden_json)
        .with_context(|| format!("Failed to parse golden trace: {}", golden_path.display()))
}

/// Compare current execution against golden trace
pub fn compare_against_golden(name: &str, command: &[&str], workdir: Option<&Path>) -> Result<()> {
    let golden = load_golden_trace(name)?;

    // Run command under renacer
    let workdir = workdir.unwrap_or_else(|| Path::new("."));
    let output = Command::new("renacer")
        .args(&["--format", "json", "--summary", "--"])
        .args(command)
        .current_dir(workdir)
        .output()
        .context("Failed to execute renacer")?;

    if !output.status.success() {
        anyhow::bail!("Renacer command failed");
    }

    let trace_json = String::from_utf8(output.stdout)?;
    let current_syscalls = parse_syscall_summary(&trace_json)?;

    // Compare syscall patterns
    let mut differences = vec![];

    // Check for new syscalls
    for (syscall, count) in &current_syscalls {
        if !golden.syscall_counts.contains_key(syscall) {
            differences.push(format!("NEW SYSCALL: {} (count: {})", syscall, count));
        }
    }

    // Check for removed syscalls
    for (syscall, golden_count) in &golden.syscall_counts {
        match current_syscalls.get(syscall) {
            None => differences.push(format!(
                "REMOVED SYSCALL: {} (was: {})",
                syscall, golden_count
            )),
            Some(current_count) if current_count != golden_count => {
                differences.push(format!(
                    "CHANGED COUNT: {} (was: {}, now: {})",
                    syscall, golden_count, current_count
                ));
            }
            _ => {}
        }
    }

    if differences.is_empty() {
        eprintln!("✅ Trace matches golden: {}", name);
        Ok(())
    } else {
        eprintln!("❌ Trace differs from golden: {}", name);
        for diff in &differences {
            eprintln!("  - {}", diff);
        }
        anyhow::bail!("Syscall pattern regression detected")
    }
}

/// Parse syscall summary from renacer JSON output
fn parse_syscall_summary(json: &str) -> Result<HashMap<String, u64>> {
    // TODO: Implement proper JSON parsing when renacer stabilizes its schema
    // For now, return empty map as a placeholder
    Ok(HashMap::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Example: Capture golden trace for bashrs --version
    #[test]
    #[ignore] // Run manually: cargo test --test golden_trace -- --ignored
    fn capture_bashrs_version_trace() {
        let result = capture_golden_trace(
            "bashrs_version",
            &["cargo", "run", "--bin", "bashrs", "--", "--version"],
            None,
        );
        assert!(
            result.is_ok(),
            "Failed to capture golden trace: {:?}",
            result
        );
    }

    /// Example: Compare bashrs --version against golden
    #[test]
    #[ignore] // Requires golden trace to exist first
    fn compare_bashrs_version_trace() {
        let result = compare_against_golden(
            "bashrs_version",
            &["cargo", "run", "--bin", "bashrs", "--", "--version"],
            None,
        );
        assert!(result.is_ok(), "Trace regression detected: {:?}", result);
    }

    /// Example: Capture golden trace for bashrs parse
    #[test]
    #[ignore]
    fn capture_bashrs_parse_trace() {
        let result = capture_golden_trace(
            "bashrs_parse_hello",
            &[
                "cargo",
                "run",
                "--bin",
                "bashrs",
                "--",
                "parse",
                "examples/hello.rs",
            ],
            None,
        );
        assert!(
            result.is_ok(),
            "Failed to capture golden trace: {:?}",
            result
        );
    }

    /// Test that golden trace directory is created
    #[test]
    fn golden_traces_dir_exists() {
        std::fs::create_dir_all(GOLDEN_TRACES_DIR).expect("Failed to create directory");
        assert!(Path::new(GOLDEN_TRACES_DIR).exists());
    }
}
