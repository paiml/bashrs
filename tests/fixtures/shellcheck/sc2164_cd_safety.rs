// Test SC2164: Use cd ... || exit in case cd fails
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target_dirs = vec![
        "/tmp",
        "/nonexistent/path", // This should fail
        "/var/log",
        env::var("HOME").unwrap_or_else(|_| "/".to_string()),
    ];
    
    for dir in target_dirs {
        // Change directory operations should include error handling
        env::set_current_dir(&dir)?;
        
        // Do some work in the directory
        std::process::Command::new("pwd").status()?;
        std::process::Command::new("ls").arg("-la").status()?;
    }
    
    Ok(())
}