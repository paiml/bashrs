// Test comprehensive error handling patterns
use std::{env, process::Command};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test various error handling scenarios
    
    // Network operation with retries
    for attempt in 1..=3 {
        println!("Attempt {} of 3", attempt);
        
        let result = Command::new("curl")
            .arg("-f")
            .arg("--connect-timeout")
            .arg("30")
            .arg("https://httpbin.org/status/200")
            .status();
            
        match result {
            Ok(status) if status.success() => {
                println!("✅ Network test successful");
                break;
            }
            Ok(_) => {
                println!("❌ Network test failed, retrying...");
                if attempt == 3 {
                    return Err("Network test failed after 3 attempts".into());
                }
            }
            Err(e) => {
                return Err(format!("Failed to execute curl: {}", e).into());
            }
        }
    }
    
    // File operation with validation
    let config_file = env::var("CONFIG_FILE").unwrap_or_else(|_| "./config.toml".to_string());
    
    if !std::path::Path::new(&config_file).exists() {
        println!("⚠️  Config file {} not found, creating default", config_file);
        
        Command::new("touch")
            .arg(&config_file)
            .status()?;
    }
    
    // Directory operation with cleanup
    let work_dir = "/tmp/rash-test-work";
    
    // Ensure clean state
    Command::new("rm")
        .arg("-rf")
        .arg(work_dir)
        .status()?;
        
    Command::new("mkdir")
        .arg("-p")
        .arg(work_dir)
        .status()?;
    
    // Do work with proper cleanup
    env::set_current_dir(work_dir)?;
    
    Command::new("touch")
        .arg("test1.txt")
        .arg("test2.txt")
        .status()?;
    
    // Verify work
    let ls_result = Command::new("ls")
        .arg("-la")
        .output()?;
        
    if !ls_result.status.success() {
        return Err("Failed to list directory contents".into());
    }
    
    println!("Work directory contents:\n{}", String::from_utf8_lossy(&ls_result.stdout));
    
    // Cleanup
    env::set_current_dir("/")?;
    Command::new("rm")
        .arg("-rf")
        .arg(work_dir)
        .status()?;
    
    println!("✅ Error handling test completed successfully");
    
    Ok(())
}