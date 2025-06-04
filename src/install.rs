/// Rash self-hosted installer - transpiled to install.sh
/// This installer bootstraps Rash using Rash itself (dogfooding)
use std::env;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, exit};

const VERSION: &str = "0.1.0";
const GITHUB_REPO: &str = "rash-sh/rash";

/// Supported platforms for binary distribution
const PLATFORMS: &[(&str, &str)] = &[
    ("x86_64-unknown-linux-musl", "linux-amd64"),
    ("aarch64-unknown-linux-musl", "linux-arm64"),
    ("x86_64-apple-darwin", "darwin-amd64"),
    ("aarch64-apple-darwin", "darwin-arm64"),
    ("x86_64-pc-windows-msvc", "windows-amd64"),
];

fn main() {
    println!("Rash installer v{}", VERSION);
    println!("========================");
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_help();
        return;
    }
    
    if args.contains(&"--version".to_string()) || args.contains(&"-V".to_string()) {
        println!("rash-installer {}", VERSION);
        return;
    }
    
    // Detect platform
    let platform = detect_platform();
    println!("Detected platform: {}", platform);
    
    // Determine installation directory
    let install_dir = get_install_dir();
    println!("Installing to: {}", install_dir);
    
    // Create installation directory
    if let Err(e) = fs::create_dir_all(&install_dir) {
        eprintln!("Failed to create directory {}: {}", install_dir, e);
        exit(1);
    }
    
    // Construct download URL using GitHub releases
    let binary_name = format!("rash-{}-{}", VERSION, platform);
    let download_url = format!(
        "https://github.com/{}/releases/download/v{}/{}.tar.gz",
        GITHUB_REPO, VERSION, binary_name
    );
    
    println!("Downloading from: {}", download_url);
    
    // Download binary
    let temp_file = format!("/tmp/rash-{}.tar.gz", VERSION);
    if let Err(e) = download_file(&download_url, &temp_file) {
        eprintln!("Failed to download: {}", e);
        exit(1);
    }
    
    // Verify checksum
    let checksum_url = format!(
        "https://github.com/{}/releases/download/v{}/SHA256SUMS",
        GITHUB_REPO, VERSION
    );
    
    if let Err(e) = verify_checksum(&temp_file, &checksum_url, &binary_name) {
        eprintln!("Checksum verification failed: {}", e);
        exit(1);
    }
    
    // Extract binary
    println!("Extracting binary...");
    if let Err(e) = extract_binary(&temp_file, &install_dir) {
        eprintln!("Failed to extract: {}", e);
        exit(1);
    }
    
    // Set executable permissions
    let binary_path = format!("{}/rash", install_dir);
    if let Err(e) = set_executable(&binary_path) {
        eprintln!("Failed to set permissions: {}", e);
        exit(1);
    }
    
    // Add to PATH if needed
    add_to_path(&install_dir);
    
    // Clean up
    let _ = fs::remove_file(&temp_file);
    
    println!("\n✓ Rash {} installed successfully!", VERSION);
    println!("\nTo get started:");
    println!("  rash --help");
    println!("  rash init my-installer");
    println!("\nFor updates, run:");
    println!("  rash self-update");
}

fn print_help() {
    println!("Rash installer - Install Rash transpiler");
    println!("\nUsage: install.sh [OPTIONS]");
    println!("\nOptions:");
    println!("  -h, --help       Show this help message");
    println!("  -V, --version    Show installer version");
    println!("  --prefix DIR     Install to DIR (default: $HOME/.rash)");
    println!("  --no-path        Don't modify PATH");
}

fn detect_platform() -> String {
    // Detect OS
    let os = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        eprintln!("Unsupported operating system");
        exit(1);
    };
    
    // Detect architecture
    let arch = if cfg!(target_arch = "x86_64") {
        "amd64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        eprintln!("Unsupported architecture");
        exit(1);
    };
    
    format!("{}-{}", os, arch)
}

fn get_install_dir() -> String {
    // Check for --prefix argument
    let args: Vec<String> = env::args().collect();
    for i in 0..args.len() {
        if args[i] == "--prefix" && i + 1 < args.len() {
            return args[i + 1].clone();
        }
    }
    
    // Default to $HOME/.rash/bin
    match env::var("HOME") {
        Ok(home) => format!("{}/.rash/bin", home),
        Err(_) => {
            eprintln!("HOME environment variable not set");
            exit(1);
        }
    }
}

fn download_file(url: &str, dest: &str) -> io::Result<()> {
    println!("Downloading {}...", url);
    
    // Security: Force HTTPS protocol and modern TLS version
    let status = Command::new("curl")
        .args(&[
            "--proto", "=https",
            "--tlsv1.2",
            "-sSfL",
            "-o", dest,
            url
        ])
        .status()?;
    
    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Download failed"
        ));
    }
    
    Ok(())
}

fn verify_checksum(file: &str, checksum_url: &str, binary_name: &str) -> io::Result<()> {
    println!("Verifying checksum...");
    
    // Download checksums
    let checksum_file = "/tmp/rash-checksums.txt";
    download_file(checksum_url, checksum_file)?;
    
    // Calculate SHA256 of downloaded file
    let output = Command::new("sha256sum")
        .arg(file)
        .output()?;
    
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to calculate checksum"
        ));
    }
    
    let calculated = String::from_utf8_lossy(&output.stdout);
    let calculated_hash = calculated.split_whitespace().next().unwrap_or("");
    
    // Read expected checksum
    let checksums = fs::read_to_string(checksum_file)?;
    let expected_line = checksums
        .lines()
        .find(|line| line.contains(&format!("{}.tar.gz", binary_name)))
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::Other,
            "Checksum not found"
        ))?;
    
    let expected_hash = expected_line.split_whitespace().next().unwrap_or("");
    
    if calculated_hash != expected_hash {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Checksum mismatch: {} != {}", calculated_hash, expected_hash)
        ));
    }
    
    // Clean up
    let _ = fs::remove_file(checksum_file);
    
    Ok(())
}

fn extract_binary(archive: &str, dest_dir: &str) -> io::Result<()> {
    // Extract using tar
    let status = Command::new("tar")
        .args(&[
            "-xzf", archive,
            "-C", dest_dir,
            "--strip-components=1"
        ])
        .status()?;
    
    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Extraction failed"
        ));
    }
    
    Ok(())
}

fn set_executable(path: &str) -> io::Result<()> {
    let file = fs::File::open(path)?;
    let mut perms = file.metadata()?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}

fn add_to_path(install_dir: &str) {
    // Check if --no-path was specified
    let args: Vec<String> = env::args().collect();
    if args.contains(&"--no-path".to_string()) {
        return;
    }
    
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    
    // Determine shell configuration file
    let config_file = if shell.contains("bash") {
        ".bashrc"
    } else if shell.contains("zsh") {
        ".zshrc"
    } else if shell.contains("fish") {
        ".config/fish/config.fish"
    } else {
        ".profile"
    };
    
    let home = match env::var("HOME") {
        Ok(h) => h,
        Err(_) => return,
    };
    
    let config_path = format!("{}/{}", home, config_file);
    
    // Check if PATH already contains install_dir
    if let Ok(path) = env::var("PATH") {
        if path.contains(install_dir) {
            return;
        }
    }
    
    // Add to shell config
    let export_line = if config_file.contains("fish") {
        format!("\nset -gx PATH {} $PATH\n", install_dir)
    } else {
        format!("\nexport PATH=\"{}:$PATH\"\n", install_dir)
    };
    
    if let Ok(mut file) = fs::OpenOptions::new()
        .append(true)
        .open(&config_path) 
    {
        let _ = file.write_all(export_line.as_bytes());
        println!("\n📝 Added {} to PATH in {}", install_dir, config_file);
        println!("   Run 'source {}' or start a new shell", config_path);
    }
}