#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)] // Examples can use unwrap() for simplicity

//! TDD-First Installer Framework Demonstration
//!
//! This example demonstrates the bashrs installer command, showing how to:
//! - Initialize a new installer project
//! - Validate installer.toml specifications
//! - Generate dependency graphs
//! - Run dry-run executions
//!
//! Run with: cargo run --example installer_demo

use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// ANSI color codes for output
mod colors {
    pub(crate) const RESET: &str = "\x1b[0m";
    pub(crate) const BOLD: &str = "\x1b[1m";
    pub(crate) const GREEN: &str = "\x1b[32m";
    pub(crate) const BLUE: &str = "\x1b[34m";
    pub(crate) const YELLOW: &str = "\x1b[33m";
    pub(crate) const CYAN: &str = "\x1b[36m";
    pub(crate) const RED: &str = "\x1b[31m";
}

fn print_header(text: &str) {
    println!(
        "\n{}{}═══════════════════════════════════════════════════════════════{}",
        colors::BOLD,
        colors::BLUE,
        colors::RESET
    );
    println!(
        "{}{}  {}  {}",
        colors::BOLD,
        colors::BLUE,
        text,
        colors::RESET
    );
    println!(
        "{}{}═══════════════════════════════════════════════════════════════{}",
        colors::BOLD,
        colors::BLUE,
        colors::RESET
    );
}

fn print_section(text: &str) {
    println!(
        "\n{}{}▸ {}{}",
        colors::BOLD,
        colors::CYAN,
        text,
        colors::RESET
    );
    println!(
        "{}───────────────────────────────────────────{}",
        colors::CYAN,
        colors::RESET
    );
}

fn print_success(text: &str) {
    println!(
        "{}{}✓ {}{}",
        colors::BOLD,
        colors::GREEN,
        text,
        colors::RESET
    );
}

fn print_info(text: &str) {
    println!("{}ℹ {}{}", colors::YELLOW, text, colors::RESET);
}

fn run_bashrs(args: &[&str]) -> Result<String> {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--bin")
        .arg("bashrs")
        .arg("--")
        .args(args)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() && !stderr.is_empty() {
        println!("{}stderr: {}{}", colors::RED, stderr, colors::RESET);
    }

    Ok(stdout)
}

fn demo_installer_init(temp_dir: &Path) -> Result<()> {
    print_section("1. Initialize Installer Project");

    let project_path = temp_dir.join("my-app-installer");
    let project_str = project_path.to_string_lossy();

    print_info(&format!("Creating installer project at: {}", project_str));

    let output = run_bashrs(&[
        "installer",
        "init",
        &project_str,
        "--description",
        "Demo application installer",
    ])?;

    println!("{}", output);

    // Show generated files
    print_info("Generated project structure:");
    println!("  {}/", project_path.file_name().unwrap().to_string_lossy());
    println!("  ├── installer.toml");
    println!("  ├── tests/");
    println!("  │   ├── mod.rs");
    println!("  │   └── falsification.rs");
    println!("  └── templates/");

    // Show installer.toml content
    let toml_content = fs::read_to_string(project_path.join("installer.toml"))?;
    print_info("installer.toml content (first 30 lines):");
    for (i, line) in toml_content.lines().take(30).enumerate() {
        println!("  {:3} │ {}", i + 1, line);
    }

    print_success("Installer project initialized");
    Ok(())
}

fn demo_installer_validate(temp_dir: &Path) -> Result<()> {
    print_section("2. Validate Installer Specification");

    let project_path = temp_dir.join("my-app-installer");
    let project_str = project_path.to_string_lossy();

    print_info(&format!("Validating: {}", project_str));

    let output = run_bashrs(&["installer", "validate", &project_str])?;
    println!("{}", output);

    print_success("Installer validation passed");
    Ok(())
}

fn demo_installer_graph(temp_dir: &Path) -> Result<()> {
    print_section("3. Generate Dependency Graph");

    let project_path = temp_dir.join("my-app-installer");
    let project_str = project_path.to_string_lossy();

    // Mermaid format
    print_info("Mermaid graph format:");
    let output = run_bashrs(&["installer", "graph", &project_str, "--format", "mermaid"])?;
    println!("{}", output);

    // DOT format
    print_info("Graphviz DOT format:");
    let output = run_bashrs(&["installer", "graph", &project_str, "--format", "dot"])?;
    println!("{}", output);

    // JSON format
    print_info("JSON format:");
    let output = run_bashrs(&["installer", "graph", &project_str, "--format", "json"])?;
    println!("{}", output);

    print_success("Graph generation complete");
    Ok(())
}

fn demo_installer_dry_run(temp_dir: &Path) -> Result<()> {
    print_section("4. Dry-Run Execution");

    let project_path = temp_dir.join("my-app-installer");
    let project_str = project_path.to_string_lossy();

    print_info("Running installer in dry-run mode:");
    let output = run_bashrs(&["installer", "run", &project_str, "--dry-run"])?;
    println!("{}", output);

    print_info("Dry-run with diff preview:");
    let output = run_bashrs(&["installer", "run", &project_str, "--dry-run", "--diff"])?;
    println!("{}", output);

    print_success("Dry-run complete (no changes made)");
    Ok(())
}

fn demo_complex_installer(temp_dir: &Path) -> Result<()> {
    print_section("5. Complex Installer with Dependencies");

    let project_path = temp_dir.join("complex-installer");
    fs::create_dir_all(&project_path)?;

    // Create a more complex installer.toml
    let complex_toml = r#"# Complex installer with dependencies
[installer]
name = "complex-app"
version = "2.0.0"
description = "Multi-step installer with dependencies"
author = "bashrs team"

[installer.requirements]
os = ["ubuntu >= 20.04", "debian >= 11"]
arch = ["x86_64", "aarch64"]
privileges = "user"
network = true

[installer.security]
trust_model = "tofu"
require_signatures = false

# Step 1: Prepare environment
[[step]]
id = "prepare-env"
name = "Prepare Environment"
action = "script"

[step.script]
interpreter = "sh"
content = '''
echo "Preparing environment..."
mkdir -p ~/.local/bin
'''

[step.checkpoint]
enabled = true

# Step 2: Download artifact (depends on prepare-env)
[[step]]
id = "download-app"
name = "Download Application"
action = "script"
depends_on = ["prepare-env"]

[step.script]
content = '''
echo "Downloading application..."
'''

# Step 3: Install (depends on download)
[[step]]
id = "install-app"
name = "Install Application"
action = "script"
depends_on = ["download-app"]

[step.script]
content = '''
echo "Installing application..."
'''

# Step 4: Configure (depends on install)
[[step]]
id = "configure-app"
name = "Configure Application"
action = "script"
depends_on = ["install-app"]

[step.script]
content = '''
echo "Configuring application..."
'''

# Step 5: Verify (depends on configure)
[[step]]
id = "verify-install"
name = "Verify Installation"
action = "script"
depends_on = ["configure-app"]

[step.script]
content = '''
echo "Verifying installation..."
'''
"#;

    fs::write(project_path.join("installer.toml"), complex_toml)?;

    print_info("Created complex installer with 5 dependent steps");

    // Validate
    let project_str = project_path.to_string_lossy();
    let output = run_bashrs(&["installer", "validate", &project_str])?;
    println!("{}", output);

    // Show graph
    print_info("Dependency graph (mermaid):");
    let output = run_bashrs(&["installer", "graph", &project_str, "--format", "mermaid"])?;
    println!("{}", output);

    print_success("Complex installer validated");
    Ok(())
}

fn demo_test_matrix(temp_dir: &Path) -> Result<()> {
    print_section("6. Container Test Matrix");

    let project_path = temp_dir.join("my-app-installer");
    let project_str = project_path.to_string_lossy();

    print_info("Running test matrix across platforms:");
    let output = run_bashrs(&[
        "installer",
        "test",
        &project_str,
        "--matrix",
        "ubuntu:22.04,debian:12,alpine:3.19",
    ])?;
    println!("{}", output);

    print_success("Test matrix execution planned");
    Ok(())
}

fn demo_keyring() -> Result<()> {
    print_section("7. Keyring Management (Ed25519 Signatures)");

    print_info("Initializing keyring:");
    let output = run_bashrs(&["installer", "keyring", "init"])?;
    println!("{}", output);

    print_info("Listing keyring:");
    let output = run_bashrs(&["installer", "keyring", "list"])?;
    println!("{}", output);

    print_success("Keyring operations complete");
    Ok(())
}

fn main() -> Result<()> {
    print_header("TDD-First Installer Framework Demo");

    println!("\nThis demo showcases the bashrs installer command features:");
    println!("  • Project initialization with TDD test harness");
    println!("  • installer.toml validation");
    println!("  • Dependency graph visualization");
    println!("  • Dry-run execution");
    println!("  • Container test matrix");
    println!("  • Keyring management");

    // Create temp directory for demos
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    print_info(&format!("Working directory: {}", temp_path.display()));

    // Run demos
    demo_installer_init(temp_path)?;
    demo_installer_validate(temp_path)?;
    demo_installer_graph(temp_path)?;
    demo_installer_dry_run(temp_path)?;
    demo_complex_installer(temp_path)?;
    demo_test_matrix(temp_path)?;
    demo_keyring()?;

    print_header("Demo Complete");

    println!("\n{}Next Steps:{}", colors::BOLD, colors::RESET);
    println!("  1. Create your own installer: bashrs installer init my-project");
    println!("  2. Edit installer.toml to define your steps");
    println!("  3. Validate: bashrs installer validate my-project");
    println!("  4. Test: bashrs installer run my-project --dry-run");
    println!("  5. Deploy: bashrs installer run my-project");

    println!(
        "\n{}Documentation:{} https://paiml.github.io/bashrs/installer/",
        colors::BOLD,
        colors::RESET
    );

    Ok(())
}
