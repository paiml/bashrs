use crate::cli::args::{InstallerCommands, KeyringCommands};
use crate::cli::logic::{format_timestamp, hex_encode, truncate_str};
use crate::models::{Error, Result};
use std::path::{Path, PathBuf};
use tracing::info;

// Extracted submodules for installer subcommands
#[path = "installer_golden_logic.rs"]
mod installer_golden_logic;
#[path = "installer_logic.rs"]
mod installer_logic;
#[path = "installer_run_logic.rs"]
mod installer_run_logic;

// ============================================================================
// INSTALLER FRAMEWORK COMMANDS (v7.0 - Issue #104)
// TDD-first installer framework with checkpointing, observability, and hermetic builds
// ============================================================================

pub(crate) fn handle_installer_command(command: InstallerCommands) -> Result<()> {
    use crate::installer;

    match command {
        InstallerCommands::Init { name, description } => {
            info!("Initializing installer project: {}", name.display());
            let project = installer::init_project(&name, description.as_deref())?;
            println!("\u{2713} Initialized installer project: {}", project.name);
            println!("  Path: {}", project.path.display());
            println!();
            println!("  Created:");
            println!("    - installer.toml (installer specification)");
            println!("    - tests/mod.rs (test harness)");
            println!("    - tests/falsification.rs (Popper-style tests)");
            println!("    - templates/ (template files)");
            println!();
            println!("Next steps:");
            println!("  1. Edit installer.toml to define steps");
            println!("  2. Run: bashrs installer validate {}", name.display());
            println!(
                "  3. Run: bashrs installer run {} --dry-run",
                name.display()
            );
            Ok(())
        }

        InstallerCommands::FromBash { input, output } => {
            info!(
                "Converting bash script to installer format: {}",
                input.display()
            );
            installer_from_bash_command(&input, output.as_deref())
        }

        InstallerCommands::Run {
            path,
            checkpoint_dir,
            dry_run,
            diff,
            hermetic,
            verify_signatures,
            parallel,
            trace,
            trace_file,
        } => {
            info!("Running installer from: {}", path.display());
            installer_run_logic::installer_run_command(
                &path,
                checkpoint_dir.as_deref(),
                dry_run,
                diff,
                hermetic,
                verify_signatures,
                parallel,
                trace,
                trace_file.as_deref(),
            )
        }

        InstallerCommands::Resume { path, from } => {
            info!("Resuming installer from: {}", path.display());
            installer_logic::installer_resume_command(&path, from.as_deref())
        }

        InstallerCommands::Validate { path } => {
            info!("Validating installer: {}", path.display());
            let result = installer::validate_installer(&path)?;
            println!("\u{2713} Installer is valid");
            println!("  Steps: {}", result.steps);
            println!("  Artifacts: {}", result.artifacts);
            if !result.warnings.is_empty() {
                println!();
                println!("Warnings:");
                for warning in &result.warnings {
                    println!("  \u{26a0} {}", warning);
                }
            }
            Ok(())
        }

        InstallerCommands::Test {
            path,
            matrix,
            coverage,
        } => {
            info!("Testing installer: {}", path.display());
            installer_logic::installer_test_command(&path, matrix.as_deref(), coverage)
        }

        InstallerCommands::Lock {
            path,
            update,
            verify,
        } => {
            info!("Managing lockfile for: {}", path.display());
            installer_logic::installer_lock_command(&path, update, verify)
        }

        InstallerCommands::Graph { path, format } => {
            info!("Generating graph for: {}", path.display());
            installer_logic::installer_graph_command(&path, format)
        }

        InstallerCommands::GoldenCapture { path, trace } => {
            info!("Capturing golden trace: {}", trace);
            installer_golden_logic::installer_golden_capture_command(&path, &trace)
        }

        InstallerCommands::GoldenCompare { path, trace } => {
            info!("Comparing against golden trace: {}", trace);
            installer_golden_logic::installer_golden_compare_command(&path, &trace)
        }

        InstallerCommands::Audit {
            path,
            format,
            security_only,
            min_severity,
            ignore,
        } => {
            info!("Auditing installer at {}", path.display());
            installer_golden_logic::installer_audit_command(
                &path,
                format,
                security_only,
                min_severity.as_deref(),
                &ignore,
            )
        }

        InstallerCommands::Keyring { command } => handle_keyring_command(command),
    }
}

fn handle_keyring_command(command: KeyringCommands) -> Result<()> {
    let keyring_path = keyring_default_path();

    match command {
        KeyringCommands::Init { import } => keyring_init_command(&keyring_path, import),
        KeyringCommands::Add { key, id } => keyring_add_command(&keyring_path, &key, &id),
        KeyringCommands::List => keyring_list_command(&keyring_path),
        KeyringCommands::Remove { id } => keyring_remove_command(&keyring_path, &id),
    }
}

fn keyring_default_path() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("bashrs")
        .join("installer")
        .join("keyring.json")
}

fn require_keyring_exists(keyring_path: &Path) -> Result<()> {
    if !keyring_path.exists() {
        return Err(Error::Validation(
            "Keyring not initialized. Run 'bashrs installer keyring init' first.".to_string(),
        ));
    }
    Ok(())
}

fn keyring_init_command(keyring_path: &Path, import: Vec<PathBuf>) -> Result<()> {
    use crate::installer::{Keyring, TrustedKey};

    info!("Initializing keyring at {}", keyring_path.display());

    if let Some(parent) = keyring_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to create keyring directory: {}", e),
            ))
        })?;
    }

    let mut keyring = Keyring::with_storage(keyring_path)?;
    keyring.enable_tofu();

    println!("\u{2713} Initialized keyring at {}", keyring_path.display());
    println!("  TOFU mode: enabled");

    for key_path in import {
        if !key_path.exists() {
            println!("  \u{26a0} Key file not found: {}", key_path.display());
            continue;
        }
        let content = std::fs::read_to_string(&key_path).map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read key file: {}", e),
            ))
        })?;
        let key_id = key_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("imported-key")
            .to_string();
        let public_key = parse_public_key(content.trim())?;
        let trusted_key = TrustedKey::new(&key_id, public_key);
        keyring.add_key(trusted_key)?;
        println!("  Imported: {} ({})", key_id, key_path.display());
    }

    Ok(())
}

fn keyring_add_command(keyring_path: &Path, key: &Path, id: &str) -> Result<()> {
    use crate::installer::{Keyring, TrustedKey};

    info!("Adding key {} from {}", id, key.display());
    require_keyring_exists(keyring_path)?;

    let mut keyring = Keyring::with_storage(keyring_path)?;
    let content = std::fs::read_to_string(key).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read key file: {}", e),
        ))
    })?;

    let public_key = parse_public_key(content.trim())?;
    let trusted_key = TrustedKey::new(id, public_key);
    keyring.add_key(trusted_key)?;
    println!("\u{2713} Added key: {}", id);
    println!("  Fingerprint: {}", &hex_encode(&public_key[..8]));

    Ok(())
}

fn keyring_list_command(keyring_path: &Path) -> Result<()> {
    use crate::installer::Keyring;

    info!("Listing keyring");

    if !keyring_path.exists() {
        println!("Keyring not initialized.");
        println!("  Run: bashrs installer keyring init");
        return Ok(());
    }

    let keyring = Keyring::with_storage(keyring_path)?;
    let keys = keyring.list_keys();

    if keys.is_empty() {
        println!("Keyring contents:");
        println!("  (no keys configured)");
    } else {
        println!("Keyring contents ({} keys):", keys.len());
        println!();
        println!("  ID                  Fingerprint       TOFU    Added");
        println!("  \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}");
        for key in keys {
            let tofu_marker = if key.is_tofu { "yes" } else { "no" };
            let added = format_timestamp(key.added_at);
            println!(
                "  {:<20}{:<18}{:<8}{}",
                truncate_str(&key.id, 20),
                key.fingerprint(),
                tofu_marker,
                added
            );
        }
    }

    println!();
    println!("  Keyring path: {}", keyring_path.display());
    let tofu_status = if keyring.is_tofu_enabled() {
        "enabled"
    } else {
        "disabled"
    };
    println!("  TOFU mode: {}", tofu_status);

    Ok(())
}

fn keyring_remove_command(keyring_path: &Path, id: &str) -> Result<()> {
    use crate::installer::Keyring;

    info!("Removing key: {}", id);
    require_keyring_exists(keyring_path)?;

    let mut keyring = Keyring::with_storage(keyring_path)?;
    if keyring.remove_key(id)? {
        println!("\u{2713} Removed key: {}", id);
    } else {
        println!("\u{26a0} Key not found: {}", id);
    }

    Ok(())
}

/// Parse a hex-encoded public key (64 hex chars = 32 bytes)
pub(crate) fn parse_public_key(hex_str: &str) -> Result<crate::installer::PublicKey> {
    if hex_str.len() != 64 {
        return Err(Error::Validation(format!(
            "Invalid public key length: expected 64 hex chars, got {}",
            hex_str.len()
        )));
    }

    let mut result = [0u8; 32];
    for (dest, chunk) in result.iter_mut().zip(hex_str.as_bytes().chunks(2)) {
        let hex = std::str::from_utf8(chunk)
            .map_err(|_| Error::Validation("Invalid hex string".to_string()))?;
        *dest = u8::from_str_radix(hex, 16)
            .map_err(|_| Error::Validation("Invalid hex character".to_string()))?;
    }

    Ok(result)
}

fn installer_from_bash_command(input: &Path, output: Option<&Path>) -> Result<()> {
    use crate::installer;

    // Validate file exists
    if !input.exists() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Bash script not found: {}", input.display()),
        )));
    }

    // Determine output directory
    let output_dir = match output {
        Some(path) => path.to_path_buf(),
        None => {
            // Default: same name as input file without extension, or "converted-installer"
            let stem = input
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("converted-installer");
            std::path::PathBuf::from(format!("{}-installer", stem))
        }
    };

    println!("Converting bash script to installer format...");
    println!("  Input: {}", input.display());
    println!("  Output: {}", output_dir.display());

    let result = installer::convert_file_to_project(input, &output_dir)?;

    println!();
    println!("Conversion complete!");
    println!("  Steps generated: {}", result.stats.steps_generated);
    println!("  Apt installs: {}", result.stats.apt_installs);
    println!("  Heredocs converted: {}", result.stats.heredocs_converted);
    println!("  Sudo patterns: {}", result.stats.sudo_patterns);
    println!(
        "  Conditionals converted: {}",
        result.stats.conditionals_converted
    );

    if !result.templates.is_empty() {
        println!();
        println!("Templates extracted:");
        for template in &result.templates {
            println!("  - templates/{}", template.name);
        }
    }

    if !result.warnings.is_empty() {
        println!();
        println!("Warnings (review manually):");
        for warning in &result.warnings {
            println!("  \u{26a0} {}", warning);
        }
    }

    println!();
    println!("Next steps:");
    println!("  1. Review: {}/installer.toml", output_dir.display());
    println!(
        "  2. Validate: bashrs installer validate {}",
        output_dir.display()
    );
    println!(
        "  3. Test: bashrs installer run {} --dry-run",
        output_dir.display()
    );

    Ok(())
}
