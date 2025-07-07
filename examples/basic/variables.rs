//! # Example: Variables
//! 
//! This example demonstrates variable usage and string escaping in Rash.
//! 
//! ## Usage
//! 
//! ```bash
//! cargo run --example variables
//! # or transpile to shell:
//! cargo run --bin rash -- build examples/basic/variables.rs -o variables.sh
//! ```
//! 
//! ## Expected Output
//! 
//! The generated shell script will demonstrate safe variable handling
//! and proper escaping of shell-special characters.

#[rash::main]
fn main() {
    // Simple variable assignment
    let name = "Alice";
    println!("Hello, {name}!");
    
    // Environment variable access
    let user = env("USER");
    let home = env("HOME");
    println!("Current user: {user}");
    println!("Home directory: {home}");
    
    // Safe handling of special characters
    let dangerous_input = "'; rm -rf /";
    echo("This is safe: {dangerous_input}");
    
    // Variable with default value
    let shell = env_var_or("SHELL", "/bin/sh");
    echo("Using shell: {shell}");
    
    // Numeric variables
    let count = 42;
    echo("The answer is: {count}");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_variables_compile() {
        // Ensure the example compiles
        assert!(true);
    }
}