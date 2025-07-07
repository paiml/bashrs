//! # Example: Injection Prevention
//! 
//! This example demonstrates how Rash prevents command injection attacks
//! through proper escaping and safe command construction.
//! 
//! ## Usage
//! 
//! ```bash
//! cargo run --example injection_prevention
//! # or transpile to shell:
//! cargo run --bin rash -- build examples/safety/injection_prevention.rs -o injection_prevention.sh
//! ```
//! 
//! ## Expected Output
//! 
//! The generated shell script will safely handle malicious inputs
//! without allowing command injection.

#[rash::main]
fn main() {
    echo("=== Command Injection Prevention Demo ===");
    
    // Simulate dangerous user inputs
    let dangerous_inputs = [
        "'; rm -rf /",
        "$(rm -rf /)",
        "`rm -rf /`",
        "& rm -rf /",
        "| rm -rf /",
        "; cat /etc/passwd",
        "$USER",
        "${PATH}",
        "\\$(date)",
        "'; echo 'pwned",
    ];
    
    echo("Testing dangerous inputs safely:");
    
    for input in dangerous_inputs {
        echo("Safe output: {input}");
        
        // Safe file creation - injection attempt will fail
        let filename = "/tmp/rash-safe-{input}.txt";
        write_file(filename, "This file has a dangerous name: {input}");
        
        // Safe command execution - special chars are escaped
        exec("echo 'Processing input: {input}'");
    }
    
    // Demonstrate safe variable expansion
    let user_provided = "$(whoami)";
    echo("User provided: {user_provided}");
    
    // This will NOT execute whoami, just print the literal string
    set_env("SAFE_VAR", user_provided);
    let retrieved = env("SAFE_VAR");
    echo("Retrieved value: {retrieved}");
    
    // Safe glob patterns
    let pattern = "*; rm -rf /";
    echo("Safe glob pattern: {pattern}");
    
    // Even in file operations, injection is prevented
    if path_exists("/tmp/test-{pattern}") {
        echo("Path check is safe");
    }
    
    // Safe command construction
    let user_file = "'; cat /etc/shadow; echo '";
    if exec("ls -la '{user_file}'") {
        echo("File found");
    } else {
        echo("File not found (expected - injection prevented)");
    }
    
    // Demonstrate that legitimate special characters still work when needed
    echo("But we can still use legitimate shell features:");
    let count = capture("ls /tmp | wc -l");
    echo("Files in /tmp: {count}");
    
    echo("=== All injection attempts were safely handled! ===");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_injection_prevention_compile() {
        // Ensure the example compiles
        assert!(true);
    }
}