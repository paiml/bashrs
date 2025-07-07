//! # Example: Conditionals
//! 
//! This example demonstrates if/else statements in Rash.
//! 
//! ## Usage
//! 
//! ```bash
//! cargo run --example conditionals
//! # or transpile to shell:
//! cargo run --bin bashrs -- build examples/control_flow/conditionals.rs -o conditionals.sh
//! ```
//! 
//! ## Expected Output
//! 
//! The generated shell script will demonstrate various conditional
//! patterns with proper shell syntax.

#[bashrs::main]
fn main() {
    // Simple if statement
    let user = env("USER");
    if user == "root" {
        echo("WARNING: Running as root!");
    } else {
        echo("Running as user: {user}");
    }
    
    // Checking file existence
    if path_exists("/etc/debian_version") {
        echo("This is a Debian-based system");
    } else if path_exists("/etc/redhat-release") {
        echo("This is a RedHat-based system");
    } else if path_exists("/etc/alpine-release") {
        echo("This is an Alpine Linux system");
    } else {
        echo("Unknown Linux distribution");
    }
    
    // Command existence check
    if command_exists("docker") {
        echo("Docker is installed");
        if exec("docker ps") {
            echo("Docker daemon is running");
        } else {
            echo("Docker daemon is not running");
        }
    } else {
        echo("Docker is not installed");
    }
    
    // Numeric comparisons
    let cpu_count = capture("nproc");
    if cpu_count > "4" {
        echo("System has more than 4 CPUs");
    } else {
        echo("System has 4 or fewer CPUs");
    }
    
    // Complex conditions
    let os_type = capture("uname -s");
    let arch = capture("uname -m");
    
    if os_type == "Linux" && arch == "x86_64" {
        echo("Running on 64-bit Linux");
    } else if os_type == "Darwin" {
        echo("Running on macOS");
    } else {
        echo("Running on {os_type} {arch}");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_conditionals_compile() {
        // Ensure the example compiles
        assert!(true);
    }
}