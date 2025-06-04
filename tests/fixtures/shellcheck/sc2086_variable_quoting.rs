// Test SC2086: Double quote to prevent globbing and word splitting
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let user = env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    let home = env::var("HOME")?;
    let path_with_spaces = "/path with spaces/file.txt";
    
    // These should all be properly quoted in the generated shell
    println!("User: {}", user);
    println!("Home: {}", home);
    println!("Path: {}", path_with_spaces);
    
    // Command arguments that need quoting
    std::process::Command::new("mkdir")
        .arg("-p")
        .arg(&path_with_spaces)
        .status()?;
    
    Ok(())
}