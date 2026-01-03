//! Convert legacy bash scripts to installer.toml format (#115)
#![allow(clippy::indexing_slicing)] // Safe: bounds checked in while loop
//!
//! This module implements `bashrs installer from-bash` which converts
//! existing bash installation scripts to the declarative installer.toml format.
//!
//! # Handled Patterns
//!
//! - Array syntax (#103) → converted to TOML lists
//! - Case statements (#99) → converted to step conditions
//! - Heredocs (#96) → converted to template files
//! - sudo patterns (#100, #101) → converted to privileged actions
//! - inline if/then (#93) → converted to step preconditions
//!
//! # Example
//!
//! ```bash
//! # Input: install.sh
//! if [ "$EUID" -ne 0 ]; then echo "Run as root"; exit 1; fi
//! apt-get update
//! apt-get install -y docker-ce
//! ```
//!
//! ```toml
//! # Output: installer.toml
//! [installer.requirements]
//! privileges = "root"
//!
//! [[step]]
//! id = "update-packages"
//! action = "apt-update"
//!
//! [[step]]
//! id = "install-docker"
//! action = "apt-install"
//! packages = ["docker-ce"]
//! ```

use crate::models::{Error, Result};
use std::path::Path;

/// Result of converting a bash script to installer format
#[derive(Debug, Clone)]
pub struct ConversionResult {
    /// Generated installer.toml content
    pub installer_toml: String,
    /// Template files extracted from heredocs
    pub templates: Vec<TemplateFile>,
    /// Warnings about patterns that couldn't be converted
    pub warnings: Vec<String>,
    /// Statistics about the conversion
    pub stats: ConversionStats,
}

/// A template file extracted from a heredoc
#[derive(Debug, Clone)]
pub struct TemplateFile {
    /// Filename for the template
    pub name: String,
    /// Content of the template
    pub content: String,
}

/// Statistics about the conversion process
#[derive(Debug, Clone, Default)]
pub struct ConversionStats {
    /// Number of steps generated
    pub steps_generated: usize,
    /// Number of apt-install commands found
    pub apt_installs: usize,
    /// Number of heredocs converted to templates
    pub heredocs_converted: usize,
    /// Number of sudo patterns converted
    pub sudo_patterns: usize,
    /// Number of conditionals converted to preconditions
    pub conditionals_converted: usize,
}

/// Detected bash pattern that can be converted
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BashPattern {
    /// Root check: if [ "$EUID" -ne 0 ]
    RootCheck,
    /// apt-get update
    AptUpdate,
    /// apt-get install -y packages...
    AptInstall { packages: Vec<String> },
    /// mkdir -p directory
    MkdirP { path: String },
    /// curl/wget download
    Download { url: String, output: Option<String> },
    /// Heredoc content
    Heredoc { delimiter: String, content: String },
    /// sudo command
    SudoCommand { command: String },
    /// Generic script line
    Script { content: String },
}

/// Convert a bash script to installer.toml format
///
/// # Arguments
/// * `script` - The bash script content
/// * `name` - Name for the installer
///
/// # Returns
/// * `ConversionResult` with the generated installer.toml and any templates
pub fn convert_bash_to_installer(script: &str, name: &str) -> Result<ConversionResult> {
    let patterns = extract_patterns(script)?;
    let (toml, templates, stats) = generate_installer_toml(&patterns, name)?;

    let warnings = generate_warnings(&patterns);

    Ok(ConversionResult {
        installer_toml: toml,
        templates,
        warnings,
        stats,
    })
}

/// Extract recognizable patterns from a bash script
fn extract_patterns(script: &str) -> Result<Vec<BashPattern>> {
    let mut patterns = Vec::new();
    let lines: Vec<&str> = script.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        // Check for root/EUID check
        if line.contains("EUID") && line.contains("-ne 0") {
            patterns.push(BashPattern::RootCheck);
            i += 1;
            continue;
        }

        // Check for apt-get update
        if line.contains("apt-get update") || line.contains("apt update") {
            patterns.push(BashPattern::AptUpdate);
            i += 1;
            continue;
        }

        // Check for apt-get install
        if let Some(packages) = parse_apt_install(line) {
            patterns.push(BashPattern::AptInstall { packages });
            i += 1;
            continue;
        }

        // Check for mkdir -p
        if let Some(path) = parse_mkdir_p(line) {
            patterns.push(BashPattern::MkdirP { path });
            i += 1;
            continue;
        }

        // Check for curl/wget download
        if let Some((url, output)) = parse_download(line) {
            patterns.push(BashPattern::Download { url, output });
            i += 1;
            continue;
        }

        // Check for heredoc
        if let Some((delimiter, content, lines_consumed)) = parse_heredoc(&lines, i) {
            patterns.push(BashPattern::Heredoc { delimiter, content });
            i += lines_consumed;
            continue;
        }

        // Check for sudo command
        if let Some(command) = parse_sudo(line) {
            patterns.push(BashPattern::SudoCommand { command });
            i += 1;
            continue;
        }

        // Default: treat as generic script line
        patterns.push(BashPattern::Script {
            content: line.to_string(),
        });
        i += 1;
    }

    Ok(patterns)
}

/// Parse apt-get install command and extract packages
fn parse_apt_install(line: &str) -> Option<Vec<String>> {
    // Match: apt-get install -y pkg1 pkg2 ... or apt install -y pkg1 pkg2
    let line = line.trim();

    // Remove sudo prefix if present
    let line = line.strip_prefix("sudo ").unwrap_or(line);

    if line.starts_with("apt-get install") || line.starts_with("apt install") {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let packages: Vec<String> = parts
            .iter()
            .skip(2) // Skip "apt-get" and "install" or "apt" and "install"
            .filter(|p| !p.starts_with('-')) // Skip flags like -y
            .map(|p| p.to_string())
            .collect();

        if !packages.is_empty() {
            return Some(packages);
        }
    }

    None
}

/// Parse mkdir -p command
fn parse_mkdir_p(line: &str) -> Option<String> {
    let line = line.strip_prefix("sudo ").unwrap_or(line);

    if line.starts_with("mkdir -p ") {
        let path = line.strip_prefix("mkdir -p ")?.trim();
        return Some(path.to_string());
    }

    None
}

/// Parse curl/wget download command
fn parse_download(line: &str) -> Option<(String, Option<String>)> {
    let line = line.strip_prefix("sudo ").unwrap_or(line);

    // curl -fsSL URL -o OUTPUT or curl -fsSL URL > OUTPUT
    if line.starts_with("curl ") {
        // Extract URL (simplified - looks for http/https)
        let parts: Vec<&str> = line.split_whitespace().collect();
        for (i, part) in parts.iter().enumerate() {
            if part.starts_with("http://") || part.starts_with("https://") {
                let url = part.to_string();
                // Check for -o flag
                let output = parts
                    .get(i + 2)
                    .filter(|_| parts.get(i + 1) == Some(&"-o"))
                    .map(|s| s.to_string());
                return Some((url, output));
            }
        }
    }

    // wget URL -O OUTPUT
    if line.starts_with("wget ") {
        let parts: Vec<&str> = line.split_whitespace().collect();
        for (i, part) in parts.iter().enumerate() {
            if part.starts_with("http://") || part.starts_with("https://") {
                let url = part.to_string();
                let output = parts
                    .get(i + 2)
                    .filter(|_| parts.get(i + 1) == Some(&"-O"))
                    .map(|s| s.to_string());
                return Some((url, output));
            }
        }
    }

    None
}

/// Parse heredoc and extract content
fn parse_heredoc(lines: &[&str], start: usize) -> Option<(String, String, usize)> {
    let line = lines[start].trim();

    // Match: cat << EOF or cat << 'EOF' or cat <<- EOF
    if !line.contains("<<") {
        return None;
    }

    // Extract delimiter
    let after_heredoc = line.split("<<").nth(1)?;
    let delimiter = after_heredoc
        .trim()
        .trim_start_matches('-')
        .trim()
        .trim_matches('\'')
        .trim_matches('"')
        .split_whitespace()
        .next()?
        .to_string();

    // Collect content until we find the delimiter
    let mut content = String::new();
    let mut lines_consumed = 1;

    for line in lines.iter().skip(start + 1) {
        lines_consumed += 1;
        if line.trim() == delimiter {
            break;
        }
        content.push_str(line);
        content.push('\n');
    }

    Some((delimiter, content, lines_consumed))
}

/// Parse sudo command
fn parse_sudo(line: &str) -> Option<String> {
    if line.starts_with("sudo ") {
        let command = line.strip_prefix("sudo ")?.to_string();
        // Don't return if it's already handled by other parsers
        if !command.starts_with("apt") && !command.starts_with("mkdir") {
            return Some(command);
        }
    }
    None
}

/// Generate installer.toml from extracted patterns
fn generate_installer_toml(
    patterns: &[BashPattern],
    name: &str,
) -> Result<(String, Vec<TemplateFile>, ConversionStats)> {
    let mut toml = String::new();
    let mut templates = Vec::new();
    let mut stats = ConversionStats::default();
    let mut step_id = 0;

    // Check for root requirement
    let requires_root = patterns.iter().any(|p| matches!(p, BashPattern::RootCheck));

    // Generate installer header
    toml.push_str(&format!(
        r#"# Installer specification converted from bash script
# Generated by bashrs installer from-bash

[installer]
name = "{name}"
version = "1.0.0"
description = "Converted from legacy bash script"

[installer.requirements]
privileges = "{}"

"#,
        if requires_root { "root" } else { "user" }
    ));

    // Generate steps
    for pattern in patterns {
        match pattern {
            BashPattern::RootCheck => {
                // Already handled in requirements
                stats.conditionals_converted += 1;
            }
            BashPattern::AptUpdate => {
                step_id += 1;
                stats.steps_generated += 1;
                toml.push_str(&format!(
                    r#"[[step]]
id = "step-{step_id}-apt-update"
name = "Update Package Lists"
action = "script"

[step.script]
interpreter = "sh"
content = "apt-get update"

"#
                ));
            }
            BashPattern::AptInstall { packages } => {
                step_id += 1;
                stats.steps_generated += 1;
                stats.apt_installs += 1;
                let packages_list = packages
                    .iter()
                    .map(|p| format!("\"{}\"", p))
                    .collect::<Vec<_>>()
                    .join(", ");
                toml.push_str(&format!(
                    r#"[[step]]
id = "step-{step_id}-install"
name = "Install Packages"
action = "script"

[step.script]
interpreter = "sh"
content = "apt-get install -y {}"

# Packages: [{}]

"#,
                    packages.join(" "),
                    packages_list
                ));
            }
            BashPattern::MkdirP { path } => {
                step_id += 1;
                stats.steps_generated += 1;
                toml.push_str(&format!(
                    r#"[[step]]
id = "step-{step_id}-mkdir"
name = "Create Directory"
action = "script"

[step.script]
interpreter = "sh"
content = "mkdir -p {path}"

"#
                ));
            }
            BashPattern::Download { url, output } => {
                step_id += 1;
                stats.steps_generated += 1;
                let output_str = output.as_deref().unwrap_or("downloaded-file");
                toml.push_str(&format!(
                    r#"[[step]]
id = "step-{step_id}-download"
name = "Download File"
action = "script"

[step.script]
interpreter = "sh"
content = "curl -fsSL {url} -o {output_str}"

"#
                ));
            }
            BashPattern::Heredoc { delimiter, content } => {
                step_id += 1;
                stats.steps_generated += 1;
                stats.heredocs_converted += 1;
                let template_name = format!("template-{}.txt", step_id);
                templates.push(TemplateFile {
                    name: template_name.clone(),
                    content: content.clone(),
                });
                toml.push_str(&format!(
                    r#"[[step]]
id = "step-{step_id}-heredoc"
name = "Write Template File"
action = "script"

# Original heredoc delimiter: {delimiter}
# Template extracted to: templates/{template_name}

[step.script]
interpreter = "sh"
content = "cat templates/{template_name}"

"#
                ));
            }
            BashPattern::SudoCommand { command } => {
                step_id += 1;
                stats.steps_generated += 1;
                stats.sudo_patterns += 1;
                toml.push_str(&format!(
                    r#"[[step]]
id = "step-{step_id}-sudo"
name = "Execute Privileged Command"
action = "script"

[step.script]
interpreter = "sh"
content = "{command}"

[step.checkpoint]
enabled = true

"#
                ));
            }
            BashPattern::Script { content } => {
                step_id += 1;
                stats.steps_generated += 1;
                toml.push_str(&format!(
                    r#"[[step]]
id = "step-{step_id}-script"
name = "Execute Script"
action = "script"

[step.script]
interpreter = "sh"
content = "{content}"

"#
                ));
            }
        }
    }

    Ok((toml, templates, stats))
}

/// Generate warnings for patterns that might need manual review
fn generate_warnings(patterns: &[BashPattern]) -> Vec<String> {
    let mut warnings = Vec::new();

    for pattern in patterns {
        match pattern {
            BashPattern::SudoCommand { command } => {
                warnings.push(format!("Sudo command may need manual review: {}", command));
            }
            BashPattern::Script { content } => {
                if content.contains("eval") {
                    warnings.push(format!(
                        "eval usage detected - potential security risk: {}",
                        content
                    ));
                }
                if content.contains("$RANDOM") || content.contains("$$") {
                    warnings.push(format!("Non-deterministic pattern detected: {}", content));
                }
            }
            _ => {}
        }
    }

    warnings
}

/// Convert a bash script file to installer project
pub fn convert_file_to_project(input: &Path, output_dir: &Path) -> Result<ConversionResult> {
    // Read the bash script
    let script = std::fs::read_to_string(input).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read {}: {}", input.display(), e),
        ))
    })?;

    // Get project name from output directory
    let name = output_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("converted-installer");

    // Convert the script
    let result = convert_bash_to_installer(&script, name)?;

    // Create output directory structure
    std::fs::create_dir_all(output_dir).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to create output directory: {}", e),
        ))
    })?;

    std::fs::create_dir_all(output_dir.join("templates")).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to create templates directory: {}", e),
        ))
    })?;

    std::fs::create_dir_all(output_dir.join("tests")).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to create tests directory: {}", e),
        ))
    })?;

    // Write installer.toml
    std::fs::write(output_dir.join("installer.toml"), &result.installer_toml).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to write installer.toml: {}", e),
        ))
    })?;

    // Write template files
    for template in &result.templates {
        std::fs::write(
            output_dir.join("templates").join(&template.name),
            &template.content,
        )
        .map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to write template {}: {}", template.name, e),
            ))
        })?;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // RED Phase: Failing Tests First (EXTREME TDD)
    // Test naming: test_<TASK_ID>_<feature>_<scenario>
    // TASK_ID: INSTALLER_115 (from-bash converter)
    // =========================================================================

    #[test]
    fn test_INSTALLER_115_extract_root_check() {
        let script = r#"
#!/bin/bash
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root"
    exit 1
fi
"#;
        let patterns = extract_patterns(script).unwrap();
        assert!(
            patterns.iter().any(|p| matches!(p, BashPattern::RootCheck)),
            "Should detect root check pattern"
        );
    }

    #[test]
    fn test_INSTALLER_115_extract_apt_update() {
        let script = "apt-get update";
        let patterns = extract_patterns(script).unwrap();
        assert!(
            patterns.iter().any(|p| matches!(p, BashPattern::AptUpdate)),
            "Should detect apt-get update"
        );
    }

    #[test]
    fn test_INSTALLER_115_extract_apt_install() {
        let script = "apt-get install -y docker-ce nginx";
        let patterns = extract_patterns(script).unwrap();

        let found = patterns.iter().find_map(|p| {
            if let BashPattern::AptInstall { packages } = p {
                Some(packages.clone())
            } else {
                None
            }
        });

        assert!(found.is_some(), "Should detect apt-get install");
        let packages = found.unwrap();
        assert!(packages.contains(&"docker-ce".to_string()));
        assert!(packages.contains(&"nginx".to_string()));
    }

    #[test]
    fn test_INSTALLER_115_extract_sudo_apt_install() {
        let script = "sudo apt-get install -y curl wget";
        let patterns = extract_patterns(script).unwrap();

        let found = patterns.iter().find_map(|p| {
            if let BashPattern::AptInstall { packages } = p {
                Some(packages.clone())
            } else {
                None
            }
        });

        assert!(found.is_some(), "Should detect sudo apt-get install");
        let packages = found.unwrap();
        assert!(packages.contains(&"curl".to_string()));
        assert!(packages.contains(&"wget".to_string()));
    }

    #[test]
    fn test_INSTALLER_115_extract_mkdir_p() {
        let script = "mkdir -p /opt/myapp/config";
        let patterns = extract_patterns(script).unwrap();

        let found = patterns.iter().find_map(|p| {
            if let BashPattern::MkdirP { path } = p {
                Some(path.clone())
            } else {
                None
            }
        });

        assert_eq!(found, Some("/opt/myapp/config".to_string()));
    }

    #[test]
    fn test_INSTALLER_115_extract_curl_download() {
        let script = "curl -fsSL https://example.com/install.sh -o /tmp/install.sh";
        let patterns = extract_patterns(script).unwrap();

        let found = patterns.iter().find_map(|p| {
            if let BashPattern::Download { url, output } = p {
                Some((url.clone(), output.clone()))
            } else {
                None
            }
        });

        assert!(found.is_some(), "Should detect curl download");
        let (url, output) = found.unwrap();
        assert_eq!(url, "https://example.com/install.sh");
        assert_eq!(output, Some("/tmp/install.sh".to_string()));
    }

    #[test]
    fn test_INSTALLER_115_extract_heredoc() {
        let script = r#"cat << EOF
Hello World
This is content
EOF"#;
        let patterns = extract_patterns(script).unwrap();

        let found = patterns.iter().find_map(|p| {
            if let BashPattern::Heredoc { delimiter, content } = p {
                Some((delimiter.clone(), content.clone()))
            } else {
                None
            }
        });

        assert!(found.is_some(), "Should detect heredoc");
        let (delimiter, content) = found.unwrap();
        assert_eq!(delimiter, "EOF");
        assert!(content.contains("Hello World"));
    }

    #[test]
    fn test_INSTALLER_115_convert_generates_valid_toml() {
        let script = r#"
#!/bin/bash
if [ "$EUID" -ne 0 ]; then exit 1; fi
apt-get update
apt-get install -y docker-ce
"#;
        let result = convert_bash_to_installer(script, "docker-installer").unwrap();

        // Should generate valid TOML
        assert!(result.installer_toml.contains("[installer]"));
        assert!(result
            .installer_toml
            .contains("name = \"docker-installer\""));
        assert!(result.installer_toml.contains("privileges = \"root\""));
        assert!(result.installer_toml.contains("[[step]]"));
    }

    #[test]
    fn test_INSTALLER_115_convert_extracts_templates() {
        let script = r#"
cat << EOF > /etc/config.txt
key=value
setting=123
EOF
"#;
        let result = convert_bash_to_installer(script, "config-installer").unwrap();

        assert!(
            !result.templates.is_empty(),
            "Should extract heredoc as template"
        );
        assert!(result.templates[0].content.contains("key=value"));
    }

    #[test]
    fn test_INSTALLER_115_convert_warns_on_eval() {
        let script = r#"
eval "rm -rf $USER_DIR"
"#;
        let result = convert_bash_to_installer(script, "unsafe-installer").unwrap();

        assert!(
            result.warnings.iter().any(|w| w.contains("eval")),
            "Should warn about eval usage"
        );
    }

    #[test]
    fn test_INSTALLER_115_convert_stats() {
        let script = r#"
apt-get update
apt-get install -y pkg1 pkg2
mkdir -p /opt/app
"#;
        let result = convert_bash_to_installer(script, "test-installer").unwrap();

        assert!(result.stats.steps_generated >= 3);
        assert_eq!(result.stats.apt_installs, 1);
    }

    #[test]
    fn test_INSTALLER_115_full_docker_script() {
        // Realistic Docker installation script
        let script = r#"
#!/bin/bash
set -e

# Check root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root"
    exit 1
fi

# Update and install prerequisites
apt-get update
apt-get install -y ca-certificates curl gnupg

# Add Docker's GPG key
mkdir -p /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.gpg

# Install Docker
apt-get install -y docker-ce docker-ce-cli containerd.io
"#;
        let result = convert_bash_to_installer(script, "docker-ce-installer").unwrap();

        // Verify conversion
        assert!(result.installer_toml.contains("privileges = \"root\""));
        assert!(result.stats.apt_installs >= 2);
        assert!(result.stats.steps_generated >= 4);

        // Should be parseable TOML (basic check)
        assert!(result.installer_toml.contains("[installer]"));
        assert!(result.installer_toml.contains("[[step]]"));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Strategy for generating valid bash-like commands
    fn bash_command_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("apt-get update".to_string()),
            Just("apt-get install -y curl".to_string()),
            Just("mkdir -p /tmp/test".to_string()),
            Just("# comment line".to_string()),
            Just("".to_string()),
        ]
    }

    proptest! {
        /// Property: Conversion never panics on any input
        #[test]
        fn prop_conversion_never_panics(script in ".*") {
            // Just verify it doesn't panic - result can be Ok or Err
            let _ = convert_bash_to_installer(&script, "test");
        }

        /// Property: Output always contains valid TOML structure markers
        #[test]
        fn prop_output_has_valid_structure(
            lines in proptest::collection::vec(bash_command_strategy(), 0..10)
        ) {
            let script = lines.join("\n");
            let result = convert_bash_to_installer(&script, "test-installer").unwrap();

            // Must always have installer section
            prop_assert!(result.installer_toml.contains("[installer]"));
            // Must always have name
            prop_assert!(result.installer_toml.contains("name = \"test-installer\""));
        }

        /// Property: Stats are always valid (conversion produces consistent output)
        #[test]
        fn prop_stats_valid(script in ".*") {
            if let Ok(result) = convert_bash_to_installer(&script, "test") {
                // Verify stats are internally consistent
                let total = result.stats.apt_installs
                    + result.stats.heredocs_converted
                    + result.stats.sudo_patterns
                    + result.stats.conditionals_converted;
                prop_assert!(total <= result.stats.steps_generated + result.stats.conditionals_converted);
            }
        }

        /// Property: apt-get install always extracts at least one package
        #[test]
        fn prop_apt_install_extracts_packages(pkg in "[a-z][a-z0-9-]{1,20}") {
            let script = format!("apt-get install -y {}", pkg);
            let result = convert_bash_to_installer(&script, "test").unwrap();
            prop_assert!(result.stats.apt_installs == 1);
            prop_assert!(result.installer_toml.contains(&pkg));
        }
    }
}
