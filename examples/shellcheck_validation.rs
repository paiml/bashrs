/// Example demonstrating ShellCheck-compatible validation in Rash
/// 
/// This example shows how Rash enforces shell safety rules at compile time,
/// preventing common shell scripting errors like unquoted variables,
/// unsafe command substitutions, and improper glob patterns.

use rash::{transpile, Config};
use rash::validation::ValidationLevel;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Safe shell script that passes all validations
    let safe_script = r#"
fn main() {
    // Properly quoted variables
    let user = "john_doe";
    println!("Hello, {}", user);
    
    // Safe command execution
    let date = std::process::Command::new("date")
        .output()
        .expect("Failed to get date");
    
    // Proper error handling
    if std::env::set_current_dir("/tmp").is_err() {
        std::process::exit(1);
    }
    
    // Safe file operations
    let files = vec!["file1.txt", "file2.txt"];
    for file in files {
        println!("Processing: {}", file);
    }
}
"#;

    // Example 2: Unsafe script that would generate validation errors
    let unsafe_script = r#"
fn main() {
    // This would generate unquoted variable warnings
    let path = std::env::var("PATH").unwrap();
    
    // This would generate command substitution warnings
    println!("Current time: {}", get_time());
    
    // This would generate cd without error handling warning
    std::env::set_current_dir("/tmp");
    
    // This could generate glob protection warnings
    remove_files("-rf");
}

fn get_time() -> String {
    "12:00".to_string()
}

fn remove_files(pattern: &str) {
    println!("Would remove: {}", pattern);
}
"#;

    // Configure with minimal validation (default)
    let config_minimal = Config {
        validation_level: Some(ValidationLevel::Minimal),
        strict_mode: false,
        ..Default::default()
    };

    // Configure with strict validation
    let config_strict = Config {
        validation_level: Some(ValidationLevel::Strict),
        strict_mode: true,
        ..Default::default()
    };

    // Transpile safe script - should succeed
    println!("=== Transpiling safe script with minimal validation ===");
    match transpile(safe_script, config_minimal.clone()) {
        Ok(shell_script) => {
            println!("Success! Generated shell script:");
            println!("{}", shell_script);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    println!("\n=== Transpiling unsafe script with minimal validation ===");
    match transpile(unsafe_script, config_minimal) {
        Ok(shell_script) => {
            println!("Generated shell script (may have issues):");
            println!("{}", shell_script);
        }
        Err(e) => {
            println!("Validation error: {}", e);
        }
    }

    println!("\n=== Transpiling unsafe script with strict validation ===");
    match transpile(unsafe_script, config_strict) {
        Ok(_) => {
            println!("Unexpectedly succeeded!");
        }
        Err(e) => {
            println!("Expected validation error: {}", e);
        }
    }

    // Example 3: Demonstrating auto-fix capabilities
    let fixable_script = r#"
fn main() {
    // Using backticks (SC2006)
    let output = `echo hello`;
    
    // Unprotected cd (SC2164)
    cd("/var/log");
    
    // Unicode quotes (SC2220)
    println!("Hello world");
}

fn cd(path: &str) {
    std::env::set_current_dir(path).unwrap();
}
"#;

    println!("\n=== Auto-fixing common issues ===");
    match transpile(fixable_script, config_minimal) {
        Ok(shell_script) => {
            println!("Auto-fixed shell script:");
            println!("{}", shell_script);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    Ok(())
}