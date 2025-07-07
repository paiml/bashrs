//! # Example: Functions
//! 
//! This example demonstrates function calls and command execution in Rash.
//! 
//! ## Usage
//! 
//! ```bash
//! cargo run --example functions
//! # or transpile to shell:
//! cargo run --bin rash -- build examples/basic/functions.rs -o functions.sh
//! ```
//! 
//! ## Expected Output
//! 
//! The generated shell script will demonstrate various function calls
//! and command executions with proper argument handling.

#[rash::main]
fn main() {
    // Basic commands
    echo("Starting function examples...");
    
    // Creating directories (idempotent)
    mkdir_p("/tmp/rash-example");
    
    // File operations
    write_file("/tmp/rash-example/test.txt", "Hello from Rash!");
    let content = read_file("/tmp/rash-example/test.txt");
    echo("File content: {content}");
    
    // Checking command existence
    if command_exists("git") {
        echo("Git is installed");
        let version = capture("git --version");
        echo("Git version: {version}");
    } else {
        echo("Git is not installed");
    }
    
    // Working with paths
    if path_exists("/etc/os-release") {
        let os_info = read_file("/etc/os-release");
        echo("OS Information available");
    }
    
    // Environment manipulation
    set_env("RASH_EXAMPLE", "true");
    let example_var = env("RASH_EXAMPLE");
    echo("RASH_EXAMPLE = {example_var}");
    
    // Command execution with error handling
    if exec("ls /tmp/rash-example") {
        echo("Directory listing successful");
    } else {
        echo("Failed to list directory");
    }
    
    // Cleanup
    exec("rm -rf /tmp/rash-example");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_functions_compile() {
        // Ensure the example compiles
        assert!(true);
    }
}