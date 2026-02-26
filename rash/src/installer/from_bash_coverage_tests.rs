//! Coverage tests for installer/from_bash.rs — targeting `convert_file_to_project`
//! and related conversion functions.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::installer::from_bash::{convert_bash_to_installer, convert_file_to_project};
use std::io::Write;
use tempfile::TempDir;

// =============================================================================
// convert_bash_to_installer coverage
// =============================================================================

#[test]
fn test_convert_bash_to_installer_apt_script() {
    let script = r#"#!/bin/bash
if [ "$EUID" -ne 0 ]; then echo "Run as root"; exit 1; fi
apt-get update
apt-get install -y docker-ce nginx
"#;
    let result = convert_bash_to_installer(script, "test-installer");
    assert!(result.is_ok());
    let conv = result.unwrap();
    assert!(conv.installer_toml.contains("test-installer"));
    assert!(conv.stats.steps_generated > 0);
}

#[test]
fn test_convert_bash_to_installer_mkdir_and_download() {
    let script = r#"#!/bin/bash
mkdir -p /opt/myapp
curl -fsSL https://example.com/setup.sh -o /tmp/setup.sh
"#;
    let result = convert_bash_to_installer(script, "download-app");
    assert!(result.is_ok());
    let conv = result.unwrap();
    assert!(conv.installer_toml.contains("download-app"));
}

#[test]
fn test_convert_bash_to_installer_sudo_patterns() {
    let script = r#"#!/bin/bash
sudo systemctl enable docker
sudo usermod -aG docker $USER
"#;
    let result = convert_bash_to_installer(script, "sudo-test");
    assert!(result.is_ok());
    let conv = result.unwrap();
    assert!(conv.stats.sudo_patterns >= 1);
}

#[test]
fn test_convert_bash_to_installer_generic_lines() {
    let script = r#"#!/bin/bash
echo "Installing..."
export PATH=$PATH:/usr/local/bin
"#;
    let result = convert_bash_to_installer(script, "generic");
    assert!(result.is_ok());
}

#[test]
fn test_convert_bash_to_installer_empty_script() {
    let result = convert_bash_to_installer("", "empty");
    assert!(result.is_ok());
}

#[test]
fn test_convert_bash_to_installer_comments_only() {
    let script = "#!/bin/bash\n# Just comments\n# Nothing else\n";
    let result = convert_bash_to_installer(script, "comments");
    assert!(result.is_ok());
}

// =============================================================================
// convert_file_to_project coverage (filesystem interaction)
// =============================================================================

#[test]
fn test_convert_file_to_project_creates_directories() {
    let tmp = TempDir::new().expect("create tempdir");
    let input_dir = tmp.path().join("input");
    std::fs::create_dir_all(&input_dir).unwrap();

    // Create a simple bash script
    let script_path = input_dir.join("install.sh");
    let mut file = std::fs::File::create(&script_path).unwrap();
    writeln!(
        file,
        "#!/bin/bash\napt-get update\napt-get install -y curl wget"
    )
    .unwrap();

    let output_dir = tmp.path().join("output-project");
    let result = convert_file_to_project(&script_path, &output_dir);
    assert!(result.is_ok(), "convert_file_to_project failed: {:?}", result);

    // Verify directory structure was created
    assert!(output_dir.exists(), "Output dir should exist");
    assert!(
        output_dir.join("templates").exists(),
        "templates/ should exist"
    );
    assert!(output_dir.join("tests").exists(), "tests/ should exist");
    assert!(
        output_dir.join("installer.toml").exists(),
        "installer.toml should exist"
    );

    // Verify installer.toml has content
    let toml = std::fs::read_to_string(output_dir.join("installer.toml")).unwrap();
    assert!(!toml.is_empty(), "installer.toml should have content");
    assert!(
        toml.contains("output-project"),
        "Should use directory name as project name"
    );
}

#[test]
fn test_convert_file_to_project_with_heredoc() {
    let tmp = TempDir::new().expect("create tempdir");

    let script_path = tmp.path().join("setup.sh");
    let mut file = std::fs::File::create(&script_path).unwrap();
    writeln!(
        file,
        "#!/bin/bash\ncat <<EOF > /etc/config\nkey=value\nEOF"
    )
    .unwrap();

    let output_dir = tmp.path().join("heredoc-project");
    let result = convert_file_to_project(&script_path, &output_dir);
    assert!(result.is_ok());
}

#[test]
fn test_convert_file_to_project_nonexistent_input() {
    let tmp = TempDir::new().expect("create tempdir");
    let output_dir = tmp.path().join("output");
    let result = convert_file_to_project(std::path::Path::new("/nonexistent/script.sh"), &output_dir);
    assert!(result.is_err());
}

#[test]
fn test_convert_file_to_project_complex_script() {
    let tmp = TempDir::new().expect("create tempdir");

    let script_path = tmp.path().join("complex.sh");
    let mut file = std::fs::File::create(&script_path).unwrap();
    writeln!(
        file,
        r#"#!/bin/bash
set -euo pipefail

# Check root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root"
    exit 1
fi

# Update packages
apt-get update
apt-get install -y docker-ce docker-compose nginx

# Create directories
mkdir -p /opt/myapp/config
mkdir -p /opt/myapp/data

# Download binary
curl -fsSL https://example.com/app-v1.0.tar.gz -o /tmp/app.tar.gz

# Configure
sudo systemctl enable docker
sudo systemctl start docker

echo "Installation complete!"
"#
    )
    .unwrap();

    let output_dir = tmp.path().join("complex-project");
    let result = convert_file_to_project(&script_path, &output_dir);
    assert!(result.is_ok());

    // Verify installer.toml was generated with content
    let toml = std::fs::read_to_string(output_dir.join("installer.toml")).unwrap();
    assert!(toml.contains("complex-project"));
}
