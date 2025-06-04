/// Rust toolchain installer example for Rash
/// This demonstrates installing Rust via rustup
use std::env;
use std::fs;
use std::process::{Command, exit};
use std::io::Write;

const RUSTUP_URL: &str = "https://sh.rustup.rs";
const RUSTUP_SHA256: &str = "be3535b3033ff5e0ecc4d589a35d3656f681332f860c5fd6684859970165ddcc";

fn main() {
    println!("Rust Toolchain Installer");
    println!("========================");
    
    // Parse arguments
    let args: Vec<String> = env::args().collect();
    
    if args.contains(&"--help".to_string()) {
        print_help();
        return;
    }
    
    // Check if already installed
    if is_rust_installed() {
        println!("Rust is already installed!");
        
        if args.contains(&"--force".to_string()) {
            println!("Proceeding with reinstallation (--force specified)");
        } else {
            println!("Use --force to reinstall");
            return;
        }
    }
    
    // Download rustup
    println!("Downloading rustup installer...");
    download_rustup();
    
    // Verify checksum
    println!("Verifying installer integrity...");
    verify_checksum("rustup-init.sh", RUSTUP_SHA256);
    
    // Make executable
    make_executable("rustup-init.sh");
    
    // Determine installation options
    let mut rustup_args = vec!["-y"];
    
    if args.contains(&"--no-modify-path".to_string()) {
        rustup_args.push("--no-modify-path");
    }
    
    if let Some(profile) = get_arg_value(&args, "--profile") {
        rustup_args.push("--profile");
        rustup_args.push(&profile);
    }
    
    if let Some(toolchain) = get_arg_value(&args, "--default-toolchain") {
        rustup_args.push("--default-toolchain");
        rustup_args.push(&toolchain);
    }
    
    // Run installer
    println!("Installing Rust toolchain...");
    run_rustup_installer(&rustup_args);
    
    // Cleanup
    cleanup();
    
    // Source cargo env
    source_cargo_env();
    
    // Verify installation
    verify_rust_installation();
    
    println!("\n✓ Rust successfully installed!");
    println!("\nTo configure your current shell run:");
    println!("  source $HOME/.cargo/env");
    println!("\nTo get started with Rust:");
    println!("  rustc --version");
    println!("  cargo --version");
    println!("  rustup --version");
}

fn print_help() {
    println!("Usage: install-rust.sh [OPTIONS]");
    println!("\nOptions:");
    println!("  --help                    Show this help message");
    println!("  --force                   Force reinstallation");
    println!("  --no-modify-path          Don't modify PATH variable");
    println!("  --profile PROFILE         Installation profile (minimal/default/complete)");
    println!("  --default-toolchain VER   Default toolchain (stable/beta/nightly)");
}

fn is_rust_installed() -> bool {
    Command::new("rustc")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn download_rustup() {
    let status = Command::new("curl")
        .args(&[
            "--proto", "=https",
            "--tlsv1.2",
            "-sSf",
            "-o", "rustup-init.sh",
            RUSTUP_URL
        ])
        .status()
        .expect("Failed to execute curl");
    
    if !status.success() {
        eprintln!("Failed to download rustup installer");
        eprintln!("Please check your internet connection");
        exit(1);
    }
}

fn verify_checksum(file: &str, expected: &str) {
    let output = Command::new("sha256sum")
        .arg(file)
        .output()
        .expect("Failed to run sha256sum");
    
    if !output.status.success() {
        eprintln!("Failed to calculate checksum");
        exit(1);
    }
    
    let result = String::from_utf8_lossy(&output.stdout);
    let actual = result.split_whitespace().next().unwrap_or("");
    
    if actual != expected {
        eprintln!("Checksum verification failed!");
        eprintln!("Expected: {}", expected);
        eprintln!("Actual:   {}", actual);
        eprintln!("\nThis could indicate a corrupted or tampered download.");
        exit(1);
    }
    
    println!("✓ Checksum verified");
}

fn make_executable(file: &str) {
    let status = Command::new("chmod")
        .args(&["+x", file])
        .status()
        .expect("Failed to run chmod");
    
    if !status.success() {
        eprintln!("Failed to make {} executable", file);
        exit(1);
    }
}

fn get_arg_value(args: &[String], flag: &str) -> Option<String> {
    for i in 0..args.len() - 1 {
        if args[i] == flag {
            return Some(args[i + 1].clone());
        }
    }
    None
}

fn run_rustup_installer(args: &[&str]) {
    let status = Command::new("./rustup-init.sh")
        .args(args)
        .status()
        .expect("Failed to run rustup installer");
    
    if !status.success() {
        eprintln!("Rustup installation failed");
        exit(1);
    }
}

fn cleanup() {
    fs::remove_file("rustup-init.sh").unwrap_or_else(|e| {
        eprintln!("Warning: Failed to remove rustup-init.sh: {}", e);
    });
}

fn source_cargo_env() {
    let home = env::var("HOME").expect("HOME not set");
    let cargo_env = format!("{}/.cargo/env", home);
    
    if fs::metadata(&cargo_env).is_ok() {
        // In the transpiled shell, this becomes: . "$HOME/.cargo/env"
        println!("Sourcing cargo environment...");
    }
}

fn verify_rust_installation() {
    let home = env::var("HOME").expect("HOME not set");
    let cargo_bin = format!("{}/.cargo/bin", home);
    
    // Update PATH for this session
    let path = env::var("PATH").unwrap_or_default();
    env::set_var("PATH", format!("{}:{}", cargo_bin, path));
    
    println!("\nVerifying installation:");
    
    // Check rustc
    if let Ok(output) = Command::new("rustc").arg("--version").output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            print!("  ✓ rustc {}", version);
        }
    }
    
    // Check cargo
    if let Ok(output) = Command::new("cargo").arg("--version").output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            print!("  ✓ cargo {}", version);
        }
    }
    
    // Check rustup
    if let Ok(output) = Command::new("rustup").arg("--version").output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            print!("  ✓ rustup {}", version);
        }
    }
}