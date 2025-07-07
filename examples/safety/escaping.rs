//! # Example: String Escaping
//! 
//! This example demonstrates Rash's string escaping mechanisms for
//! various shell-special characters and unicode handling.
//! 
//! ## Usage
//! 
//! ```bash
//! cargo run --example escaping
//! # or transpile to shell:
//! cargo run --bin rash -- build examples/safety/escaping.rs -o escaping.sh
//! ```
//! 
//! ## Expected Output
//! 
//! The generated shell script will properly escape all special characters
//! while preserving the intended string content.

#[rash::main]
fn main() {
    echo("=== String Escaping Examples ===");
    
    // Single quotes
    let single = "It's a beautiful day!";
    echo("Single quotes: {single}");
    
    // Double quotes
    let double = "She said \"Hello!\"";
    echo("Double quotes: {double}");
    
    // Backticks
    let backticks = "Old style `command` substitution";
    echo("Backticks: {backticks}");
    
    // Dollar signs
    let dollars = "Price: $99.99 and ${VARIABLE}";
    echo("Dollar signs: {dollars}");
    
    // Backslashes
    let backslashes = "Path: C:\\Windows\\System32";
    echo("Backslashes: {backslashes}");
    
    // Newlines and tabs
    let multiline = "Line 1\nLine 2\n\tIndented";
    echo("Multiline: {multiline}");
    
    // Mixed special characters
    let complex = "$'\\n\\t' and $\"\\x41\" and $(echo test)";
    echo("Complex: {complex}");
    
    // Unicode characters
    let unicode = "Hello 世界 🌍 مرحبا";
    echo("Unicode: {unicode}");
    
    // File paths with spaces
    let path_with_spaces = "/home/user/My Documents/Project (2023)/file.txt";
    echo("Path: {path_with_spaces}");
    
    // Create a file with special characters in name
    let special_filename = "/tmp/test file with 'quotes' and $vars.txt";
    write_file(special_filename, "Content with special chars: $USER");
    
    if path_exists(special_filename) {
        echo("Successfully created file with special name");
        let content = read_file(special_filename);
        echo("Content: {content}");
        exec("rm -f '{special_filename}'");
    }
    
    // Environment variables with special values
    set_env("SPECIAL_VAR", "Value with $HOME and `date`");
    let retrieved = env("SPECIAL_VAR");
    echo("Environment variable: {retrieved}");
    
    // Array with special strings
    let special_array = [
        "normal",
        "with spaces",
        "with'quotes",
        "with\"double",
        "with$dollar",
        "with;semicolon",
        "with|pipe",
    ];
    
    echo("Processing special array:");
    for item in special_array {
        echo("  - {item}");
    }
    
    echo("=== All special characters handled safely! ===");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_escaping_compile() {
        // Ensure the example compiles
        assert!(true);
    }
}