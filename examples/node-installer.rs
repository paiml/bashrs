/// Node.js installer example for Rash
/// This demonstrates a real-world installer that downloads and installs Node.js
use std::env;
use std::fs;
use std::process::{Command, exit};

const NODE_VERSION: &str = "20.10.0";

fn main() {
    println!("Node.js v{} Installer", NODE_VERSION);
    println!("======================");
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let prefix = get_prefix(&args);
    
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_help();
        return;
    }
    
    // Check prerequisites
    println!("Checking prerequisites...");
    check_prerequisites();
    
    // Detect platform
    let platform = detect_platform();
    println!("Detected platform: {}", platform);
    
    // Download Node.js
    let filename = format!("node-v{}-{}.tar.gz", NODE_VERSION, platform);
    let url = format!(
        "https://nodejs.org/dist/v{}/{}",
        NODE_VERSION, filename
    );
    
    println!("Downloading Node.js from {}...", url);
    download_file(&url, &filename);
    
    // Extract
    println!("Extracting {}...", filename);
    extract_tarball(&filename);
    
    // Install
    let extracted_dir = format!("node-v{}-{}", NODE_VERSION, platform);
    install_node(&extracted_dir, &prefix);
    
    // Cleanup
    cleanup(&filename, &extracted_dir);
    
    // Verify installation
    verify_installation(&prefix);
    
    println!("\n✓ Node.js v{} successfully installed to {}", NODE_VERSION, prefix);
    println!("\nTo use Node.js, add this to your shell configuration:");
    println!("  export PATH={}:$PATH", format!("{}/bin", prefix));
}

fn get_prefix(args: &[String]) -> String {
    for i in 0..args.len() - 1 {
        if args[i] == "--prefix" {
            return args[i + 1].clone();
        }
    }
    
    env::var("PREFIX").unwrap_or_else(|_| {
        let home = env::var("HOME").expect("HOME not set");
        format!("{}/.local", home)
    })
}

fn print_help() {
    println!("Usage: install-node.sh [OPTIONS]");
    println!("\nOptions:");
    println!("  -h, --help       Show this help message");
    println!("  --prefix PATH    Install to PATH (default: ~/.local)");
    println!("\nEnvironment variables:");
    println!("  PREFIX           Alternative way to set install prefix");
}

fn check_prerequisites() {
    let required = ["curl", "tar", "gzip"];
    
    for cmd in &required {
        let status = Command::new("which")
            .arg(cmd)
            .status()
            .expect("Failed to run which");
        
        if !status.success() {
            eprintln!("Error: {} is required but not found in PATH", cmd);
            eprintln!("Please install {} and try again", cmd);
            exit(1);
        }
    }
}

fn detect_platform() -> String {
    let os = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else {
        eprintln!("Unsupported operating system");
        exit(1);
    };
    
    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        eprintln!("Unsupported architecture");
        exit(1);
    };
    
    format!("{}-{}", os, arch)
}

fn download_file(url: &str, filename: &str) {
    let status = Command::new("curl")
        .args(&[
            "--proto", "=https",
            "--tlsv1.2",
            "-sSfL",
            "-o", filename,
            "--progress-bar",
            url
        ])
        .status()
        .expect("Failed to run curl");
    
    if !status.success() {
        eprintln!("Failed to download {}", url);
        exit(1);
    }
}

fn extract_tarball(filename: &str) {
    let status = Command::new("tar")
        .args(&["-xzf", filename])
        .status()
        .expect("Failed to run tar");
    
    if !status.success() {
        eprintln!("Failed to extract {}", filename);
        exit(1);
    }
}

fn install_node(source_dir: &str, prefix: &str) {
    println!("Installing to {}...", prefix);
    
    // Create prefix directory
    fs::create_dir_all(prefix).unwrap_or_else(|e| {
        eprintln!("Failed to create directory {}: {}", prefix, e);
        exit(1);
    });
    
    // Copy files
    for subdir in &["bin", "lib", "share", "include"] {
        let src = format!("{}/{}", source_dir, subdir);
        let dst = format!("{}/{}", prefix, subdir);
        
        if fs::metadata(&src).is_ok() {
            println!("  Copying {}...", subdir);
            
            // Create destination directory
            fs::create_dir_all(&dst).unwrap_or_else(|e| {
                eprintln!("Failed to create {}: {}", dst, e);
                exit(1);
            });
            
            // Copy recursively
            let status = Command::new("cp")
                .args(&["-r", &format!("{}/.", src), &dst])
                .status()
                .expect("Failed to run cp");
            
            if !status.success() {
                eprintln!("Failed to copy {}", subdir);
                exit(1);
            }
        }
    }
}

fn cleanup(filename: &str, dir: &str) {
    println!("Cleaning up...");
    
    // Remove tarball
    fs::remove_file(filename).unwrap_or_else(|e| {
        eprintln!("Warning: Failed to remove {}: {}", filename, e);
    });
    
    // Remove extracted directory
    fs::remove_dir_all(dir).unwrap_or_else(|e| {
        eprintln!("Warning: Failed to remove {}: {}", dir, e);
    });
}

fn verify_installation(prefix: &str) {
    println!("\nVerifying installation...");
    
    let node_path = format!("{}/bin/node", prefix);
    let npm_path = format!("{}/bin/npm", prefix);
    
    // Check node
    let output = Command::new(&node_path)
        .arg("--version")
        .output()
        .expect("Failed to run node");
    
    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        println!("  ✓ Node.js {}", version.trim());
    } else {
        eprintln!("  ✗ Node.js installation failed");
        exit(1);
    }
    
    // Check npm
    let output = Command::new(&npm_path)
        .arg("--version")
        .output()
        .expect("Failed to run npm");
    
    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        println!("  ✓ npm {}", version.trim());
    } else {
        eprintln!("  ✗ npm installation failed");
        exit(1);
    }
}