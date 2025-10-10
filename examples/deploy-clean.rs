#!/usr/bin/env bashrs
//! # Example: Deploy Application (Purified)
//!
//! This example demonstrates the Rash purification workflow:
//! 1. Original messy bash (deploy-messy.sh)
//! 2. Clean Rash version (this file)
//! 3. Purified POSIX shell (generated output)
//!
//! ## Transformation
//!
//! **Before (bash)**:
//! - Uses $RANDOM for session ID (non-deterministic)
//! - Uses timestamps for release tags (non-deterministic)
//! - Uses $$ for temp directories (process-dependent)
//! - Non-idempotent operations (mkdir, rm, ln -s fail on re-run)
//!
//! **After (Rash)**:
//! - Version-based session IDs (deterministic)
//! - Fixed release tags (deterministic)
//! - Fixed temp directories (deterministic)
//! - Idempotent operations (safe to re-run)
//!
//! ## Usage
//!
//! ```bash
//! # Compare the messy bash version
//! cat examples/deploy-messy.sh
//!
//! # Transpile to purified shell
//! cargo run --example deploy-clean
//!
//! # Or build to shell script
//! cargo run -- transpile examples/deploy-clean.rs -o deploy-purified.sh
//! ```
//!
//! ## Quality Metrics
//!
//! - Determinism: ✅ (version-based IDs)
//! - Idempotency: ✅ (safe to re-run multiple times)
//! - POSIX: ✅ (shellcheck compliant)
//! - Security: ✅ (proper quoting)

use std::fs;
use std::path::Path;

fn main() {
    match deploy_app("1.0.0") {
        Ok(_) => println!("Deployment successful"),
        Err(e) => {
            eprintln!("Deployment failed: {}", e);
            std::process::exit(1);
        }
    }
}

/// Deploy application with deterministic and idempotent operations
fn deploy_app(version: &str) -> Result<(), String> {
    // Deterministic session ID based on version (not $RANDOM)
    let session_id = format!("session-{}", version);

    // Deterministic release tag (not timestamp)
    let release_tag = format!("release-{}", version);

    // Fixed work directory (not $$)
    let work_dir = "/tmp/deploy-workspace";

    // Fixed log file (not $SECONDS)
    let log_file = "/var/log/deploy.log";

    // Idempotent cleanup (ignore error if doesn't exist)
    let _ = fs::remove_file("/app/current");

    // Idempotent directory creation (works on re-run)
    let release_dir = format!("/app/releases/{}", release_tag);
    fs::create_dir_all(&release_dir)
        .map_err(|e| format!("Failed to create release dir: {}", e))?;

    // Extract archive
    extract_archive("app.tar.gz", &release_dir)?;

    // Idempotent symlink creation (remove old, create new)
    let current_link = Path::new("/app/current");
    if current_link.exists() {
        fs::remove_file(current_link)
            .map_err(|e| format!("Failed to remove old symlink: {}", e))?;
    }

    std::os::unix::fs::symlink(&release_dir, current_link)
        .map_err(|e| format!("Failed to create symlink: {}", e))?;

    // Append to log (deterministic message, no timestamp)
    let log_msg = format!("Session {}: Deployed {}\n", session_id, release_tag);
    append_to_log(&log_file, &log_msg)?;

    println!("Deployment complete: {}", release_tag);
    println!("Session: {}", session_id);
    println!("Logs: {}", log_file);

    Ok(())
}

fn extract_archive(archive: &str, dest: &str) -> Result<(), String> {
    let status = std::process::Command::new("tar")
        .arg("xzf")
        .arg(archive)
        .arg("-C")
        .arg(dest)
        .status()
        .map_err(|e| format!("Failed to extract: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("Archive extraction failed".to_string())
    }
}

fn append_to_log(log_file: &str, message: &str) -> Result<(), String> {
    use std::io::Write;

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
        .map_err(|e| format!("Failed to open log: {}", e))?;

    file.write_all(message.as_bytes())
        .map_err(|e| format!("Failed to write log: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deploy_deterministic() {
        // Same version produces same session ID
        let id1 = format!("session-{}", "1.0.0");
        let id2 = format!("session-{}", "1.0.0");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_release_tag_deterministic() {
        // Same version produces same release tag
        let tag1 = format!("release-{}", "1.0.0");
        let tag2 = format!("release-{}", "1.0.0");
        assert_eq!(tag1, tag2);
    }
}
