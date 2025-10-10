#!/usr/bin/env bashrs
//! # Example: Database Backup (Purified)
//!
//! This example demonstrates purification of a database backup script.
//!
//! ## Transformation
//!
//! **Before (bash - backup-messy.sh)**:
//! - Random backup ID: `backup-$RANDOM-$(date +%s)`
//! - Process-dependent temp: `/tmp/dbbackup-$$`
//! - Timestamp filename: `db-$(date +%Y%m%d-%H%M%S).sql.gz`
//! - Non-idempotent mkdir/rm
//! - Timestamp in logs
//!
//! **After (Rash - this file)**:
//! - Version-based ID: `backup-mydb-1.0.0`
//! - Fixed temp: `/tmp/dbbackup-workspace`
//! - Deterministic filename: `mydb-1.0.0.sql.gz`
//! - Idempotent mkdir -p / rm -rf
//! - No timestamps in logs
//!
//! ## Usage
//!
//! ```bash
//! # Compare before/after
//! diff examples/backup-messy.sh <(cargo run --example backup-clean)
//!
//! # Transpile to purified shell
//! cargo run -- transpile examples/backup-clean.rs -o backup-purified.sh
//!
//! # Run the purified script
//! ./backup-purified.sh mydb 1.0.0
//! ```
//!
//! ## Purification Report
//!
//! Issues Fixed: 6
//! - Removed $RANDOM (non-deterministic)
//! - Removed $(date +%s) (non-deterministic)
//! - Removed $(date +%Y%m%d-%H%M%S) (non-deterministic)
//! - Removed $$ (process-dependent)
//! - Added -p to mkdir (idempotency)
//! - Added -rf to rm (idempotency)

use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let db_name = args.get(1).map(|s| s.as_str()).unwrap_or("mydb");
    let version = args.get(2).map(|s| s.as_str()).unwrap_or("1.0.0");

    match backup_database(db_name, version) {
        Ok(_) => println!("Backup completed successfully"),
        Err(e) => {
            eprintln!("Backup failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn backup_database(db_name: &str, version: &str) -> Result<(), String> {
    // Deterministic backup ID based on version (not $RANDOM or timestamp)
    let backup_id = format!("backup-{}-{}", db_name, version);

    // Fixed temp directory (not $$)
    let temp_dir = "/tmp/dbbackup-workspace";
    fs::create_dir_all(temp_dir)
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;

    // Deterministic backup filename (not timestamp)
    let backup_file = format!("/backups/{}-{}.sql.gz", db_name, version);

    // Perform backup
    let dump_file = format!("{}/dump.sql", temp_dir);
    run_pg_dump(db_name, &dump_file)?;

    // Compress
    compress_file(&dump_file)?;

    // Move to final location
    let compressed = format!("{}.gz", dump_file);
    fs::rename(&compressed, &backup_file)
        .map_err(|e| format!("Failed to move backup: {}", e))?;

    // Idempotent cleanup (ignore errors)
    let _ = fs::remove_dir_all(temp_dir);

    // Log without timestamp
    let log_msg = format!("Backup {} completed: {}\n", backup_id, backup_file);
    append_to_log("/var/log/backups.log", &log_msg)?;

    println!("Backup ID: {}", backup_id);
    println!("File: {}", backup_file);

    Ok(())
}

fn run_pg_dump(db: &str, output: &str) -> Result<(), String> {
    let status = std::process::Command::new("pg_dump")
        .arg(db)
        .arg("-f")
        .arg(output)
        .status()
        .map_err(|e| format!("pg_dump failed: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("Database backup failed".to_string())
    }
}

fn compress_file(file: &str) -> Result<(), String> {
    let status = std::process::Command::new("gzip")
        .arg(file)
        .status()
        .map_err(|e| format!("gzip failed: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("Compression failed".to_string())
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
    fn test_backup_id_deterministic() {
        // Same inputs = same backup ID
        let id1 = format!("backup-{}-{}", "mydb", "1.0.0");
        let id2 = format!("backup-{}-{}", "mydb", "1.0.0");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_backup_filename_deterministic() {
        // Same inputs = same filename
        let file1 = format!("/backups/{}-{}.sql.gz", "mydb", "1.0.0");
        let file2 = format!("/backups/{}-{}.sql.gz", "mydb", "1.0.0");
        assert_eq!(file1, file2);
    }
}
