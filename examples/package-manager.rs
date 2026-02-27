// Package Manager Simulation
// Demonstrates: complex workflows, state management, validation
// Use case: Custom package installation/management tool

fn main() {
    let package_name = env_var_or("PACKAGE", "example-tool");
    let version = env_var_or("VERSION", "latest");

    echo("=== Package Manager ===");
    echo("Package: {package_name}");
    echo("Version: {version}");
    echo("");

    // Check if already installed
    echo("[1/6] Checking existing installation...");
    if is_installed(package_name) {
        echo("  Package {package_name} is already installed");
        echo("  Use --force to reinstall");
        exit(0);
    }
    echo("  No existing installation found");

    // Download package
    echo("[2/6] Downloading package...");
    download_package(package_name, version);

    // Verify download
    echo("[3/6] Verifying download...");
    if !verify_download(package_name) {
        echo("✗ Download verification failed");
    } else {
        echo("✓ Download verified");
    }

    // Extract package
    echo("[4/6] Extracting package...");
    extract_package(package_name);

    // Install
    echo("[5/6] Installing...");
    install_package(package_name);

    // Post-install verification
    echo("[6/6] Verifying installation...");
    if is_installed(package_name) {
        echo("✓ Package {package_name} installed successfully");
    }

    echo("");
    echo("Installation complete");
    echo("Run '{package_name} --version' to verify");
}

fn is_installed(package: &str) -> bool {
    // Check if package is in PATH
    echo("  Checking for {package}...");
    false
}

fn download_package(package: &str, version: &str) {
    echo("  Downloading {package} version {version}...");
    echo("  → Download simulated");
}

fn verify_download(package: &str) -> bool {
    echo("  Verifying checksum for {package}...");
    true
}

fn extract_package(package: &str) {
    echo("  Extracting {package} archive...");
    echo("  → Extraction simulated");
}

fn install_package(package: &str) {
    let install_dir = "/usr/local/bin";
    echo("  Installing to {install_dir}...");
    mkdir_p(install_dir);
    echo("  → Installation simulated");
}

fn env_var_or(key: &str, default: &str) -> String {
    let value = env(key);
    if value == "" {
        default.to_string()
    } else {
        value
    }
}

fn env(key: &str) -> String {
    String::new()
}
fn echo(msg: &str) {}
fn mkdir_p(path: &str) {}
