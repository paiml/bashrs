// Test SC2006: Use $(...) instead of legacy `...`
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Modern command substitution patterns
    let timestamp = Command::new("date")
        .arg("+%Y%m%d_%H%M%S")
        .output()?;
    
    let hostname = Command::new("hostname")
        .output()?;
    
    let kernel_version = Command::new("uname")
        .arg("-r")
        .output()?;
    
    let disk_usage = Command::new("df")
        .arg("-h")
        .arg("/")
        .output()?;
    
    // Use the results
    println!("Timestamp: {}", String::from_utf8_lossy(&timestamp.stdout).trim());
    println!("Hostname: {}", String::from_utf8_lossy(&hostname.stdout).trim());
    println!("Kernel: {}", String::from_utf8_lossy(&kernel_version.stdout).trim());
    println!("Disk usage: {}", String::from_utf8_lossy(&disk_usage.stdout));
    
    Ok(())
}