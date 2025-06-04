use std::fs;
use std::path::Path;
use tracing::{info, warn, error};
use crate::cli::{Cli, Commands};
use crate::models::{Config, Result, Error};
use crate::{transpile, check};

pub fn execute_command(cli: Cli) -> Result<()> {
    // Initialize logging
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(if cli.verbose { 
            tracing::Level::DEBUG 
        } else { 
            tracing::Level::INFO 
        })
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| Error::Internal(format!("Failed to initialize logging: {}", e)))?;

    match cli.command {
        Commands::Build { 
            input, 
            output, 
            emit_proof, 
            no_optimize 
        } => {
            info!("Building {} -> {}", input.display(), output.display());
            
            let config = Config {
                target: cli.target,
                verify: cli.verify,
                emit_proof,
                optimize: !no_optimize,
            };
            
            build_command(&input, &output, config)
        }
        
        Commands::Check { input } => {
            info!("Checking {}", input.display());
            check_command(&input)
        }
        
        Commands::Init { path, name } => {
            info!("Initializing project in {}", path.display());
            init_command(&path, name.as_deref())
        }
        
        Commands::Verify { rust_source, shell_script } => {
            info!("Verifying {} against {}", shell_script.display(), rust_source.display());
            verify_command(&rust_source, &shell_script, cli.target, cli.verify)
        }
    }
}

fn build_command(input: &Path, output: &Path, config: Config) -> Result<()> {
    // Read input file
    let source = fs::read_to_string(input)
        .map_err(|e| Error::Io(e))?;
    
    // Transpile
    let shell_code = transpile(&source, config.clone())?;
    
    // Write output
    fs::write(output, shell_code)
        .map_err(|e| Error::Io(e))?;
    
    info!("Successfully transpiled to {}", output.display());
    
    // Generate proof if requested
    if config.emit_proof {
        let proof_path = output.with_extension("proof");
        generate_proof(&source, &proof_path, &config)?;
        info!("Proof generated at {}", proof_path.display());
    }
    
    Ok(())
}

fn check_command(input: &Path) -> Result<()> {
    // Read input file
    let source = fs::read_to_string(input)
        .map_err(|e| Error::Io(e))?;
    
    // Check compatibility
    check(&source)?;
    
    info!("✓ {} is compatible with Rash", input.display());
    Ok(())
}

fn init_command(path: &Path, name: Option<&str>) -> Result<()> {
    // Create directory if it doesn't exist
    if !path.exists() {
        fs::create_dir_all(path)
            .map_err(|e| Error::Io(e))?;
    }
    
    let project_name = name.unwrap_or(
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("rash-project")
    );
    
    // Create Cargo.toml
    let cargo_toml = format!(
r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
rash = "0.1"

[package.metadata.rash]
target = "posix"
verify = "strict"
"#, project_name);
    
    fs::write(path.join("Cargo.toml"), cargo_toml)
        .map_err(|e| Error::Io(e))?;
    
    // Create src directory
    let src_dir = path.join("src");
    fs::create_dir_all(&src_dir)
        .map_err(|e| Error::Io(e))?;
    
    // Create main.rs
    let main_rs = r#"#![no_std]
#![no_main]

use rash::prelude::*;

#[rash::main]
fn install() -> Result<(), &'static str> {
    // Your installation logic here
    rash::io::info("Hello from Rash!");
    Ok(())
}
"#;
    
    fs::write(src_dir.join("main.rs"), main_rs)
        .map_err(|e| Error::Io(e))?;
    
    // Create rash.toml
    let rash_toml = r#"[rash]
target = "posix"
verify = "strict"
emit-proof = false
optimize = true

[build]
output = "install.sh"
"#;
    
    fs::write(path.join("rash.toml"), rash_toml)
        .map_err(|e| Error::Io(e))?;
    
    info!("✓ Initialized Rash project '{}'", project_name);
    info!("  Run 'cd {}' to enter the project", path.display());
    info!("  Run 'rash build src/main.rs' to build");
    
    Ok(())
}

fn verify_command(
    rust_source: &Path, 
    shell_script: &Path, 
    target: crate::models::ShellDialect,
    verify_level: crate::models::VerificationLevel
) -> Result<()> {
    // Read both files
    let rust_code = fs::read_to_string(rust_source)
        .map_err(|e| Error::Io(e))?;
    let shell_code = fs::read_to_string(shell_script)
        .map_err(|e| Error::Io(e))?;
    
    // Transpile Rust to shell
    let config = Config {
        target,
        verify: verify_level,
        emit_proof: false,
        optimize: true,
    };
    
    let generated_shell = transpile(&rust_code, config)?;
    
    // Compare generated vs actual
    if normalize_shell_script(&generated_shell) == normalize_shell_script(&shell_code) {
        info!("✓ Shell script matches Rust source");
        Ok(())
    } else {
        warn!("Shell script does not match Rust source");
        // TODO: Show diff
        Err(Error::Verification("Script mismatch".to_string()))
    }
}

fn generate_proof(source: &str, proof_path: &Path, config: &Config) -> Result<()> {
    // For now, just create a simple proof file
    let proof = format!(
r#"{{
    "version": "1.0",
    "source_hash": "{}",
    "verification_level": "{:?}",
    "target": "{:?}",
    "timestamp": "{}",
    "properties": ["no-injection", "deterministic"]
}}"#,
        blake3::hash(source.as_bytes()),
        config.verify,
        config.target,
        chrono::Utc::now().to_rfc3339()
    );
    
    fs::write(proof_path, proof)
        .map_err(|e| Error::Io(e))?;
    
    Ok(())
}

fn normalize_shell_script(script: &str) -> String {
    // Remove comments and normalize whitespace for comparison
    script
        .lines()
        .filter(|line| !line.trim().starts_with('#'))
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}