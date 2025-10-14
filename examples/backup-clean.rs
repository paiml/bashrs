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
//! # Transpile to purified shell
//! cargo run --bin bashrs -- build examples/backup-clean.rs
//!
//! # Run the purified script
//! sh install.sh mydb 1.0.0
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

fn main() {
    // Default values for db_name and version
    // In practice, these would come from command-line arguments
    let db_name = "mydb";
    let version = "1.0.0";

    backup_database(db_name, version);
}

fn backup_database(db_name: &str, version: &str) {
    // Deterministic backup ID based on version (not $RANDOM or timestamp)
    let backup_id_prefix = "backup-";
    let backup_id_full = concat_three(backup_id_prefix, db_name, version);

    // Fixed temp directory (not $$)
    let temp_dir = "/tmp/dbbackup-workspace";
    mkdir_p(temp_dir);

    // Deterministic backup filename (not timestamp)
    let backup_dir = "/backups/";
    let backup_filename = concat_three(db_name, "-", version);
    let backup_file_base = concat_two(backup_dir, backup_filename);
    let backup_file = concat_two(backup_file_base, ".sql.gz");

    // Perform backup
    let dump_file_dir = concat_two(temp_dir, "/dump.sql");
    run_pg_dump(db_name, dump_file_dir);

    // Compress
    compress_file(dump_file_dir);

    // Move to final location
    let compressed = concat_two(dump_file_dir, ".gz");
    move_file(compressed, backup_file);

    // Idempotent cleanup
    rm_rf(temp_dir);

    // Log without timestamp
    let log_msg_part1 = "Backup ";
    let log_msg_part2 = concat_two(log_msg_part1, backup_id_full);
    let log_msg_part3 = concat_two(log_msg_part2, " completed: ");
    let log_msg = concat_two(log_msg_part3, backup_file);
    append_to_log("/var/log/backups.log", log_msg);

    println("Backup completed");
    println(backup_id_full);
    println(backup_file);
}

fn run_pg_dump(db: &str, output: &str) {
    // Shell: pg_dump "$db" -f "$output"
    command_pg_dump(db, output);
}

fn compress_file(file: &str) {
    // Shell: gzip "$file"
    command_gzip(file);
}

fn append_to_log(log_file: &str, message: &str) {
    // Shell: echo "$message" >> "$log_file"
    echo_append(message, log_file);
}

// Helper functions that map to shell commands

fn concat_two(a: &str, b: &str) -> &str {
    // This will become shell string concatenation
    a
}

fn concat_three(a: &str, b: &str, c: &str) -> &str {
    // This will become shell string concatenation
    a
}

fn mkdir_p(dir: &str) {
    // Shell: mkdir -p "$dir"
}

fn rm_rf(dir: &str) {
    // Shell: rm -rf "$dir"
}

fn move_file(src: &str, dst: &str) {
    // Shell: mv "$src" "$dst"
}

fn command_pg_dump(db: &str, output: &str) {
    // Shell: pg_dump "$db" -f "$output"
}

fn command_gzip(file: &str) {
    // Shell: gzip "$file"
}

fn echo_append(msg: &str, file: &str) {
    // Shell: echo "$msg" >> "$file"
}

fn println(msg: &str) {
    // Shell: echo "$msg"
}
