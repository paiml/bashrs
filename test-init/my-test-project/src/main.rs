/// Example installer script for Rash
/// This will be transpiled to POSIX-compliant shell script
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, exit};

const VERSION: &str = "0.1.0";
const BINARY_NAME: &str = "myapp";

fn main() {
    println!("{} installer v{}", BINARY_NAME, VERSION);
    println!("=======================");
    
    // Parse arguments
    let args: Vec<String> = env::args().collect();
    if args.contains(&"--help".to_string()) {
        print_help();
        return;
    }
    
    // Determine installation directory
    let prefix = env::var("PREFIX").unwrap_or_else(|_| "/usr/local".to_string());
    let bin_dir = format!("{}/bin", prefix);
    
    println!("Installing to: {}", bin_dir);
    
    // Create directory
    if let Err(e) = fs::create_dir_all(&bin_dir) {
        eprintln!("Failed to create directory: {}", e);
        exit(1);
    }
    
    // Download binary (example URL)
    let url = format!(
        "https://github.com/example/{}/releases/download/v{}/{}-{}.tar.gz",
        BINARY_NAME, VERSION, BINARY_NAME, detect_platform()
    );
    
    println!("Downloading from: {}", url);
    
    // TODO: Add your actual installation logic here
    // - Download binary
    // - Verify checksum
    // - Extract and install
    // - Set permissions
    
    println!("\n✓ {} installed successfully!", BINARY_NAME);
    println!("\nTo get started, run:");
    println!("  {} --help", BINARY_NAME);
}

fn print_help() {
    println!("Usage: install.sh [OPTIONS]");
    println!("\nOptions:");
    println!("  --help       Show this help message");
    println!("  --prefix DIR Install to DIR (default: /usr/local)");
}

fn detect_platform() -> &'static str {
    // Simplified platform detection
    if cfg!(target_os = "linux") {
        if cfg!(target_arch = "x86_64") {
            "linux-amd64"
        } else {
            "linux-arm64"
        }
    } else if cfg!(target_os = "macos") {
        "darwin-amd64"
    } else {
        panic!("Unsupported platform");
    }
}