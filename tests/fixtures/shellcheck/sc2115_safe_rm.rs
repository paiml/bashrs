// Test SC2115: Use ${var:?} to ensure variable is not empty before rm
use std::process::Command;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // These operations could be dangerous if variables are empty
    let temp_dir = env::var("TMPDIR").unwrap_or_else(|_| "/tmp".to_string());
    let build_dir = env::var("BUILD_DIR").unwrap_or_else(|_| "./build".to_string());
    let cache_dir = env::var("CACHE_DIR").unwrap_or_else(|_| "./cache".to_string());
    
    // Safe removal operations - should validate variables are not empty
    if !temp_dir.is_empty() {
        Command::new("rm")
            .arg("-rf")
            .arg(format!("{}/my-temp-files", temp_dir))
            .status()?;
    }
    
    if !build_dir.is_empty() {
        Command::new("rm")
            .arg("-rf")
            .arg(&build_dir)
            .status()?;
    }
    
    if !cache_dir.is_empty() {
        Command::new("find")
            .arg(&cache_dir)
            .arg("-type")
            .arg("f")
            .arg("-delete")
            .status()?;
    }
    
    Ok(())
}