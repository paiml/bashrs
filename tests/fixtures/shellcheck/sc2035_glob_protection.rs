// Test SC2035: Use ./* to prevent filenames starting with dashes being interpreted as options
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // These patterns could be dangerous without proper protection
    let dangerous_patterns = vec!["-rf", "--verbose", "-n", "--help"];
    
    for pattern in dangerous_patterns {
        // Remove files matching pattern - should be protected
        Command::new("rm")
            .arg("-f")
            .arg(pattern)
            .status()?;
    }
    
    // List files that might start with dashes
    Command::new("ls")
        .arg("-la")
        .arg("./")
        .status()?;
    
    Ok(())
}