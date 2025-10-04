// System Information Gathering Script
// Demonstrates: command execution, variable assignment, output formatting
// Use case: Collecting system information for diagnostics or setup validation

fn main() {
    let os_type = env_var_or("OSTYPE", "unknown");
    let hostname = env_var_or("HOSTNAME", "localhost");
    let user = env_var_or("USER", "unknown");
    let home = env_var_or("HOME", "/");
    let shell = env_var_or("SHELL", "/bin/sh");

    echo("=== System Information ===");
    echo("");
    echo("OS Type:    {os_type}");
    echo("Hostname:   {hostname}");
    echo("User:       {user}");
    echo("Home:       {home}");
    echo("Shell:      {shell}");
    echo("");
    echo("System information collected successfully");
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
