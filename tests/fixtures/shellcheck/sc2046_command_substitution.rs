// Test SC2046: Quote command substitutions to prevent word splitting
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Command substitutions that should be quoted
    let files = Command::new("find")
        .arg(".")
        .arg("-name")
        .arg("*.txt")
        .output()?;
    
    let current_dir = Command::new("pwd")
        .output()?;
        
    let file_count = Command::new("ls")
        .arg("-1")
        .arg(".")
        .output()?;
    
    // Use the results
    println!("Files found: {}", String::from_utf8_lossy(&files.stdout));
    println!("Current directory: {}", String::from_utf8_lossy(&current_dir.stdout));
    println!("File count: {}", String::from_utf8_lossy(&file_count.stdout).lines().count());
    
    Ok(())
}