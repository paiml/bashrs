// Complex real-world installer script that tests multiple ShellCheck rules
use std::{env, fs, process::Command};

const VERSION: &str = "1.0.0";
const BINARY_NAME: &str = "myapp";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prefix = env::var("PREFIX").unwrap_or_else(|_| "/usr/local".to_string());
    let temp_dir = env::var("TMPDIR").unwrap_or_else(|_| "/tmp".to_string());
    let user = env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    
    println!("🚀 Installing {} v{} for user {}", BINARY_NAME, VERSION, user);
    
    // Create installation directories
    let bin_dir = format!("{}/bin", prefix);
    let lib_dir = format!("{}/lib/{}", prefix, BINARY_NAME);
    let share_dir = format!("{}/share/{}", prefix, BINARY_NAME);
    
    fs::create_dir_all(&bin_dir)?;
    fs::create_dir_all(&lib_dir)?;
    fs::create_dir_all(&share_dir)?;
    
    // Download and verify binary
    let download_url = format!(
        "https://github.com/example/{}/releases/download/v{}/{}-linux-x64.tar.gz",
        BINARY_NAME, VERSION, BINARY_NAME
    );
    
    let archive_path = format!("{}/{}-{}.tar.gz", temp_dir, BINARY_NAME, VERSION);
    
    println!("📥 Downloading from {}", download_url);
    Command::new("curl")
        .arg("-L")
        .arg("--fail")
        .arg("--silent")
        .arg("--show-error")
        .arg("-o")
        .arg(&archive_path)
        .arg(&download_url)
        .status()?;
    
    // Verify checksum
    let expected_sha256 = "abcd1234567890abcd1234567890abcd1234567890abcd1234567890abcd1234";
    Command::new("sh")
        .arg("-c")
        .arg(&format!("echo '{} {}' | sha256sum -c", expected_sha256, archive_path))
        .status()?;
    
    // Extract archive
    env::set_current_dir(&temp_dir)?;
    Command::new("tar")
        .arg("xzf")
        .arg(&archive_path)
        .status()?;
    
    // Install files
    let extracted_dir = format!("{}/{}-{}", temp_dir, BINARY_NAME, VERSION);
    env::set_current_dir(&extracted_dir)?;
    
    // Copy binary
    Command::new("cp")
        .arg(BINARY_NAME)
        .arg(&format!("{}/{}", bin_dir, BINARY_NAME))
        .status()?;
    
    // Make executable
    Command::new("chmod")
        .arg("+x")
        .arg(&format!("{}/{}", bin_dir, BINARY_NAME))
        .status()?;
    
    // Copy libraries
    Command::new("cp")
        .arg("-r")
        .arg("lib/")
        .arg(&lib_dir)
        .status()?;
    
    // Copy documentation
    Command::new("cp")
        .arg("-r")
        .arg("share/")
        .arg(&share_dir)
        .status()?;
    
    // Cleanup
    Command::new("rm")
        .arg("-rf")
        .arg(&archive_path)
        .status()?;
    
    Command::new("rm")
        .arg("-rf")
        .arg(&extracted_dir)
        .status()?;
    
    // Verify installation
    let version_output = Command::new(&format!("{}/{}", bin_dir, BINARY_NAME))
        .arg("--version")
        .output()?;
    
    println!("✅ Installation complete!");
    println!("   Binary: {}/{}", bin_dir, BINARY_NAME);
    println!("   Version: {}", String::from_utf8_lossy(&version_output.stdout).trim());
    
    Ok(())
}