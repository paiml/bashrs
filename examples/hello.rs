/// Simple hello world example for Rash
/// This demonstrates basic Rash functionality
use std::env;

fn main() {
    // Basic output
    println!("Hello from Rash!");
    
    // Environment variables
    let user = env::var("USER").unwrap_or("world".to_string());
    println!("Hello, {}!", user);
    
    // Command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        println!("You passed {} arguments:", args.len() - 1);
        for (i, arg) in args.iter().skip(1).enumerate() {
            println!("  {}: {}", i + 1, arg);
        }
    }
    
    // Working with paths
    let home = env::var("HOME").unwrap_or("/tmp".to_string());
    println!("Your home directory is: {}", home);
    
    // Basic conditionals
    if user == "root" {
        println!("Warning: Running as root!");
    } else {
        println!("Running as regular user: {}", user);
    }
    
    // Exit codes
    if args.contains(&"--fail".to_string()) {
        eprintln!("Failing as requested!");
        std::process::exit(1);
    }
    
    println!("\nRash transpilation successful! 🦀 → 🐚");
}