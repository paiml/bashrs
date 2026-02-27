// Download and Install Pattern
// Demonstrates: typical software installation workflow
// Use case: Bootstrap installers for development tools

fn main() {
    let version = env_var_or("VERSION", "1.0.0");
    let install_dir = env_var_or("INSTALL_DIR", "/usr/local");
    let arch = env_var_or("ARCH", "x86_64");

    echo("=== Software Installation ===");
    echo("Version:     {version}");
    echo("Install Dir: {install_dir}");
    echo("Architecture: {arch}");
    echo("");

    // Check prerequisites
    echo("[1/5] Checking prerequisites...");
    check_prerequisites();

    // Create installation directory
    echo("[2/5] Creating installation directory...");
    let bin_dir = "{install_dir}/bin";
    mkdir_p(bin_dir);

    // Download (simulated)
    echo("[3/5] Downloading software...");
    echo("  URL: https://example.com/app-{version}-{arch}.tar.gz");
    echo("  → Download simulated (use curl/wget in real script)");

    // Install
    echo("[4/5] Installing...");
    echo("  → Extracting archive...");
    echo("  → Copying binaries to {bin_dir}...");
    echo("  → Setting permissions...");

    // Verify
    echo("[5/5] Verifying installation...");
    if path_exists(bin_dir) {
        echo("✓ Installation successful!");
    }

    echo("");
    echo("Installation completed successfully");
    echo("Add {bin_dir} to your PATH to use the software");
}

fn check_prerequisites() {
    echo("  Checking for required tools...");
    echo("  ✓ All prerequisites met");
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
fn path_exists(path: &str) -> bool {
    true
}
