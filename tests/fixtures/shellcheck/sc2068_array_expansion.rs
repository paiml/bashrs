// Test SC2068: Double quote array expansions to avoid re-splitting elements
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_extensions = vec!["*.txt", "*.rs", "*.md", "*.toml"];
    let search_paths = vec!["/usr/bin", "/usr/local/bin", "/bin"];
    let compiler_flags = vec!["-O2", "-Wall", "-Wextra", "-pedantic"];
    
    // Find files with multiple extensions
    let mut find_cmd = Command::new("find");
    find_cmd.arg(".");
    for ext in &file_extensions {
        find_cmd.arg("-name").arg(ext).arg("-o");
    }
    find_cmd.status()?;
    
    // Search in multiple paths
    for path in &search_paths {
        Command::new("ls")
            .arg("-la")
            .arg(path)
            .status()?;
    }
    
    // Compile with multiple flags (simulated)
    let mut gcc_cmd = Command::new("echo");
    gcc_cmd.arg("gcc");
    for flag in &compiler_flags {
        gcc_cmd.arg(flag);
    }
    gcc_cmd.arg("main.c");
    gcc_cmd.status()?;
    
    Ok(())
}