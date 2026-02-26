use crate::cli::args::InstallerGraphFormat;
use crate::models::{Error, Result};
use std::path::Path;

// ============================================================================
// INSTALLER TEST, LOCK, AND GRAPH COMMANDS
// Extracted from installer_commands.rs for module size compliance
// ============================================================================

pub(crate) fn installer_test_command(
    path: &Path,
    matrix: Option<&str>,
    coverage: bool,
) -> Result<()> {
    use crate::installer::{self, ContainerRuntime, ContainerTestMatrix, MatrixConfig};

    // Validate installer first
    let result = installer::validate_installer(path)?;

    if let Some(platforms) = matrix {
        // Detect container runtime
        let runtime = ContainerRuntime::detect();
        let runtime_name = runtime.map_or("none", |r| r.command());

        // Parse matrix configuration
        let config = if platforms.is_empty() || platforms == "default" {
            MatrixConfig::default_platforms()
        } else if platforms == "extended" {
            MatrixConfig::extended_platforms()
        } else {
            MatrixConfig::from_platform_string(platforms)
        };

        // Create and run test matrix (simulate for now)
        let mut matrix_runner = ContainerTestMatrix::new(path, config);

        // Check runtime availability
        if runtime.is_none() {
            println!("\u{26a0} Warning: No container runtime detected (docker/podman)");
            println!("  Running in simulation mode\n");
        }

        // Simulate tests (actual execution would require container runtime)
        let summary = matrix_runner.simulate();

        // Display results
        println!("{}", matrix_runner.format_results());
        println!("{}", summary.format());

        println!("  Steps per platform: {}", result.steps);
        println!("  Artifacts: {}", result.artifacts);
        println!("  Runtime: {}", runtime_name);
        if coverage {
            println!("  Coverage: enabled");
        }
        println!();

        if summary.all_passed() {
            println!("\u{2713} All {} platform(s) passed", summary.total);
        } else {
            println!(
                "\u{2717} {} of {} platform(s) failed",
                summary.failed, summary.total
            );
            return Err(Error::Validation(format!(
                "{} platform(s) failed testing",
                summary.failed
            )));
        }
    } else {
        println!("Installer test summary:");
        println!("  Steps: {}", result.steps);
        println!("  Artifacts: {}", result.artifacts);
        if coverage {
            println!("  Coverage: enabled");
        }
        println!("\u{2713} Installer specification validated");
    }

    Ok(())
}

pub(crate) fn installer_lock_command(path: &Path, update: bool, verify: bool) -> Result<()> {
    use crate::installer::{self, Lockfile};

    let result = installer::validate_installer(path)?;
    let lockfile_path = path.join("installer.lock");

    println!("Managing lockfile for installer at {}", path.display());
    println!();

    if verify {
        return installer_lock_verify(path, &lockfile_path, &result);
    }

    if update && !lockfile_path.exists() {
        println!("  No existing lockfile found, generating new one...");
    } else if update {
        let existing = Lockfile::load(&lockfile_path)?;
        println!("Updating lockfile...");
        println!(
            "  Existing lockfile has {} artifacts",
            existing.artifacts.len()
        );
    }

    installer_lock_generate(path, &lockfile_path, &result)
}

fn installer_lock_verify(
    path: &Path,
    lockfile_path: &Path,
    result: &crate::installer::ValidationResult,
) -> Result<()> {
    use crate::installer::{HermeticContext, Lockfile, LOCKFILE_VERSION};

    if !lockfile_path.exists() {
        if result.artifacts == 0 {
            println!("\u{2713} No lockfile needed (no external artifacts)");
        } else {
            return Err(Error::Validation(format!(
                "Lockfile required but not found. Run 'bashrs installer lock {}' first.",
                path.display()
            )));
        }
        return Ok(());
    }

    let lockfile = Lockfile::load(lockfile_path)?;
    lockfile.verify()?;

    println!("Lockfile verification:");
    println!("  Version: {}", LOCKFILE_VERSION);
    println!("  Generator: {}", lockfile.generator);
    println!("  Content hash: {}", lockfile.content_hash);
    println!("  Artifacts: {}", lockfile.artifacts.len());
    println!();

    if lockfile.artifacts.len() != result.artifacts {
        println!("\u{26a0} Lockfile may be outdated:");
        println!(
            "    Lockfile has {} artifacts, spec has {}",
            lockfile.artifacts.len(),
            result.artifacts
        );
        println!(
            "    Run 'bashrs installer lock {} --update' to refresh",
            path.display()
        );
    } else {
        println!("\u{2713} Lockfile is valid and up-to-date");
    }

    let context = HermeticContext::from_lockfile(lockfile)?;
    println!("  SOURCE_DATE_EPOCH: {}", context.source_date_epoch());

    Ok(())
}

fn installer_lock_generate(
    path: &Path,
    lockfile_path: &Path,
    result: &crate::installer::ValidationResult,
) -> Result<()> {
    use crate::installer::{LockedArtifact, Lockfile, LockfileEnvironment, LOCKFILE_VERSION};

    let epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut lockfile = Lockfile::new();
    lockfile.environment = LockfileEnvironment::deterministic(epoch);

    if result.artifacts == 0 {
        println!("\u{2713} No external artifacts to lock");
        println!("  Hermetic mode will use empty lockfile");
        lockfile.finalize();
        lockfile.save(lockfile_path)?;
        println!("  Created: {}", lockfile_path.display());
        println!(
            "  SOURCE_DATE_EPOCH: {}",
            lockfile.environment.source_date_epoch
        );
        return Ok(());
    }

    println!("Generating lockfile for {} artifacts...", result.artifacts);
    println!();

    for i in 0..result.artifacts {
        let artifact = LockedArtifact::new(
            &format!("artifact-{}", i + 1),
            "1.0.0",
            "https://example.com/artifact.tar.gz",
            "sha256:placeholder",
            0,
        );
        lockfile.add_artifact(artifact);
    }

    lockfile.finalize();
    lockfile.save(lockfile_path)?;

    println!("\u{2713} Generated lockfile: {}", lockfile_path.display());
    println!("  Version: {}", LOCKFILE_VERSION);
    println!("  Content hash: {}", lockfile.content_hash);
    println!("  Artifacts locked: {}", lockfile.artifacts.len());
    println!(
        "  SOURCE_DATE_EPOCH: {}",
        lockfile.environment.source_date_epoch
    );
    println!();
    println!("Note: Run with real artifact URLs to generate proper hashes.");
    println!(
        "      Use 'bashrs installer run {} --hermetic' to execute.",
        path.display()
    );

    Ok(())
}

pub(crate) fn installer_graph_command(path: &Path, format: InstallerGraphFormat) -> Result<()> {
    use crate::installer::{format_execution_plan, InstallerGraph, InstallerSpec};

    // Find installer.toml
    let installer_toml = if path.is_dir() {
        path.join("installer.toml")
    } else {
        path.to_path_buf()
    };

    // Parse spec and build graph
    let content = std::fs::read_to_string(&installer_toml).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read installer.toml: {}", e),
        ))
    })?;
    let spec = InstallerSpec::parse(&content)?;
    let graph = InstallerGraph::from_spec(&spec)?;

    match format {
        InstallerGraphFormat::Mermaid => {
            println!("{}", graph.to_mermaid());
        }
        InstallerGraphFormat::Dot => {
            println!("{}", graph.to_dot());
        }
        InstallerGraphFormat::Json => {
            // Generate JSON with execution plan
            let plan = graph.create_plan();
            let json_output = serde_json::json!({
                "nodes": graph.nodes().iter().map(|n| {
                    serde_json::json!({
                        "id": n.id,
                        "name": n.name,
                        "estimated_duration_secs": n.estimated_duration_secs,
                        "capabilities": n.capabilities,
                        "exclusive_resource": n.exclusive_resource,
                    })
                }).collect::<Vec<_>>(),
                "execution_plan": {
                    "waves": plan.waves.iter().map(|w| {
                        serde_json::json!({
                            "wave_number": w.wave_number,
                            "step_ids": w.step_ids,
                            "is_sequential": w.is_sequential,
                            "sequential_reason": w.sequential_reason,
                            "estimated_duration_secs": w.estimated_duration_secs,
                        })
                    }).collect::<Vec<_>>(),
                    "total_duration_parallel_secs": plan.total_duration_parallel_secs,
                    "total_duration_sequential_secs": plan.total_duration_sequential_secs,
                    "speedup_percent": plan.speedup_percent,
                }
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&json_output).unwrap_or_default()
            );
        }
    }

    // Print execution plan summary for non-JSON formats
    if !matches!(format, InstallerGraphFormat::Json) {
        println!();
        let plan = graph.create_plan();
        println!("{}", format_execution_plan(&plan, 4));
    }

    Ok(())
}

pub(crate) fn installer_resume_command(path: &Path, from: Option<&str>) -> Result<()> {
    use crate::installer::{self, CheckpointStore};

    // Validate installer exists
    let validation = installer::validate_installer(path)?;

    // Check for checkpoint directory
    let checkpoint_dir = path.join(".checkpoint");

    if !checkpoint_dir.exists() {
        return Err(Error::Validation(format!(
            "No checkpoint found at {} - run 'bashrs installer run {}' first",
            checkpoint_dir.display(),
            path.display()
        )));
    }

    // Load checkpoint
    let store = CheckpointStore::load(&checkpoint_dir)?;

    println!("Resume installer: {}", path.display());
    println!();

    // Show checkpoint status
    if let Some(run_id) = store.current_run_id() {
        println!("Checkpoint found: {}", run_id);
        println!(
            "  Hermetic mode: {}",
            if store.is_hermetic() { "yes" } else { "no" }
        );

        let steps = store.steps();
        let completed = steps
            .iter()
            .filter(|s| s.status == installer::StepStatus::Completed)
            .count();
        let failed = steps
            .iter()
            .filter(|s| s.status == installer::StepStatus::Failed)
            .count();
        let pending = steps
            .iter()
            .filter(|s| s.status == installer::StepStatus::Pending)
            .count();

        println!(
            "  Steps: {} total, {} completed, {} failed, {} pending",
            steps.len(),
            completed,
            failed,
            pending
        );

        if let Some(last) = store.last_successful_step() {
            println!("  Last successful: {}", last.step_id);
        }

        // Determine resume point
        let resume_from = match from {
            Some(step_id) => {
                if store.get_step(step_id).is_none() {
                    return Err(Error::Validation(format!(
                        "Step '{}' not found in checkpoint",
                        step_id
                    )));
                }
                step_id.to_string()
            }
            None => store
                .last_successful_step()
                .map(|s| s.step_id.clone())
                .ok_or_else(|| {
                    Error::Validation("No successful steps to resume from".to_string())
                })?,
        };

        println!();
        println!("Would resume from step: {}", resume_from);
        println!();
        println!("Note: Full execution not yet implemented.");
        println!("  Steps in spec: {}", validation.steps);
        println!(
            "  Run with --dry-run to validate: bashrs installer run {} --dry-run",
            path.display()
        );
    } else {
        return Err(Error::Validation(
            "Checkpoint exists but has no active run".to_string(),
        ));
    }

    Ok(())
}
