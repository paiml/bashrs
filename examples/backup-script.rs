// Backup Script
// Demonstrates: file copying, timestamp generation, error handling
// Use case: Creating timestamped backups of important files

fn main() {
    let source_dir = env_var_or("BACKUP_SOURCE", "/etc");
    let backup_dir = env_var_or("BACKUP_DIR", "/var/backups");
    let timestamp = "20251004"; // In real script, would use $(date +%Y%m%d)

    echo("=== Backup Script ===");
    echo("Source:  {source_dir}");
    echo("Backup:  {backup_dir}");
    echo("Date:    {timestamp}");
    echo("");

    // Create backup directory
    echo("[1/3] Preparing backup directory...");
    let backup_target = "{backup_dir}/backup-{timestamp}";
    mkdir_p(backup_target);

    if path_exists(backup_target) {
        echo("✓ Backup directory created: {backup_target}");
    }

    // Perform backup
    echo("[2/3] Copying files...");
    echo("  Copying {source_dir} to {backup_target}");
    echo("  → Backup simulated (use cp -r in real script)");

    // Verify backup
    echo("[3/3] Verifying backup...");
    if path_exists(backup_target) {
        echo("✓ Backup completed successfully");
        echo("  Location: {backup_target}");
    }

    echo("");
    echo("Backup completed: {backup_target}");
}

fn env_var_or(key: &str, default: &str) -> String {
    let value = env(key);
    if value == "" {
        default.to_string()
    } else {
        value
    }
}

fn env(key: &str) -> String {
    String::new()
}
fn echo(msg: &str) {}
fn mkdir_p(path: &str) {}
fn path_exists(path: &str) -> bool {
    true
}
