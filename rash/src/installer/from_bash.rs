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

include!("from_bash_incl3.rs");
