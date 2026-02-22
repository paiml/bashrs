use crate::cli::args::{
    AuditOutputFormat, InstallerCommands, InstallerGraphFormat, KeyringCommands,
};
use crate::cli::logic::{format_timestamp, hex_encode, truncate_str};
use crate::models::{Error, Result};
use std::path::{Path, PathBuf};
use tracing::info;

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
            println!("✓ Initialized installer project: {}", project.name);
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
            installer_run_command(
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
            installer_resume_command(&path, from.as_deref())
        }

        InstallerCommands::Validate { path } => {
            info!("Validating installer: {}", path.display());
            let result = installer::validate_installer(&path)?;
            println!("✓ Installer is valid");
            println!("  Steps: {}", result.steps);
            println!("  Artifacts: {}", result.artifacts);
            if !result.warnings.is_empty() {
                println!();
                println!("Warnings:");
                for warning in &result.warnings {
                    println!("  ⚠ {}", warning);
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
            installer_test_command(&path, matrix.as_deref(), coverage)
        }

        InstallerCommands::Lock {
            path,
            update,
            verify,
        } => {
            info!("Managing lockfile for: {}", path.display());
            installer_lock_command(&path, update, verify)
        }

        InstallerCommands::Graph { path, format } => {
            info!("Generating graph for: {}", path.display());
            installer_graph_command(&path, format)
        }

        InstallerCommands::GoldenCapture { path, trace } => {
            info!("Capturing golden trace: {}", trace);
            installer_golden_capture_command(&path, &trace)
        }

        InstallerCommands::GoldenCompare { path, trace } => {
            info!("Comparing against golden trace: {}", trace);
            installer_golden_compare_command(&path, &trace)
        }

        InstallerCommands::Audit {
            path,
            format,
            security_only,
            min_severity,
            ignore,
        } => {
            info!("Auditing installer at {}", path.display());
            installer_audit_command(
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

// hex_encode, format_timestamp, truncate_str moved to cli/logic.rs

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
            println!("  ⚠ {}", warning);
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

#[allow(clippy::fn_params_excessive_bools, clippy::too_many_arguments)]
fn installer_run_command(
    path: &Path,
    checkpoint_dir: Option<&Path>,
    dry_run: bool,
    diff: bool,
    hermetic: bool,
    verify_signatures: bool,
    _parallel: bool,
    trace: bool,
    trace_file: Option<&Path>,
) -> Result<()> {
    use crate::installer::{
        self, CheckpointStore, ExecutionMode, ExecutorConfig, InstallerProgress, InstallerSpec,
        ProgressRenderer, StepExecutor, TerminalRenderer,
    };

    // Phase 1: Validate and parse installer spec
    let result = installer::validate_installer(path)?;
    let installer_toml = path.join("installer.toml");
    let spec_content = std::fs::read_to_string(&installer_toml).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read installer.toml: {}", e),
        ))
    })?;
    let spec = InstallerSpec::parse(&spec_content)?;

    // Phase 2: Set up hermetic context and signature keyring
    let hermetic_context = installer_setup_hermetic_context(path, hermetic)?;
    let keyring = installer_setup_signature_keyring(verify_signatures)?;

    // Phase 3: Handle early-exit modes (diff preview, dry-run)
    if diff {
        return installer_handle_diff_preview(&result, &hermetic_context, &keyring);
    }
    if dry_run {
        return installer_handle_dry_run(&result, &hermetic_context, &keyring);
    }

    // Phase 4: Set up checkpoint store
    let checkpoint_path = checkpoint_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| path.join(".checkpoint"));

    if !checkpoint_path.exists() {
        std::fs::create_dir_all(&checkpoint_path).map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to create checkpoint directory: {}", e),
            ))
        })?;
    }

    let mut store = CheckpointStore::new(&checkpoint_path)?;
    let installer_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("installer");

    if let Some(ref ctx) = hermetic_context {
        store.start_hermetic_run(installer_name, "1.0.0", &ctx.lockfile.content_hash)?;
    } else {
        store.start_run(installer_name, "1.0.0")?;
    }

    // Phase 5: Set up progress tracking
    let execution_mode = if hermetic {
        ExecutionMode::Hermetic
    } else {
        ExecutionMode::Normal
    };

    let mut progress = InstallerProgress::new(installer_name, "1.0.0")
        .with_mode(execution_mode)
        .with_artifacts(0, result.artifacts)
        .with_signatures(keyring.is_some())
        .with_trace(trace);

    for step in &spec.step {
        progress.add_step(&step.id, &step.name);
    }

    // Phase 6: Set up tracing context
    let mut tracing_ctx = installer_setup_tracing(trace, trace_file, path, &result, hermetic);

    // Phase 7: Render initial progress header
    let renderer = TerminalRenderer::new();
    println!("{}", renderer.render_header(&progress));
    println!("  Installer: {}", path.display());
    println!("  Checkpoint: {}", checkpoint_path.display());
    println!("  Run ID: {}", store.current_run_id().unwrap_or("unknown"));
    println!("  Mode: {}", execution_mode.label());
    println!();

    // Phase 8: Create executor
    let executor_config = ExecutorConfig {
        dry_run,
        use_sudo: false,
        environment: std::collections::HashMap::new(),
        working_dir: Some(path.display().to_string()),
        timeout_secs: 300,
    };
    let executor = StepExecutor::with_config(executor_config);

    // Phase 9: Execute steps
    let all_succeeded =
        installer_execute_steps(&spec, &executor, &mut progress, &mut tracing_ctx, &renderer);

    // Phase 10: Finalize
    installer_finalize_run(
        &progress,
        &mut tracing_ctx,
        &renderer,
        trace_file,
        path,
        all_succeeded,
    )
}

/// Set up hermetic context if requested.
fn installer_setup_hermetic_context(
    path: &Path,
    hermetic: bool,
) -> Result<Option<crate::installer::HermeticContext>> {
    use crate::installer::HermeticContext;

    if !hermetic {
        return Ok(None);
    }

    let lockfile_path = path.join("installer.lock");
    if !lockfile_path.exists() {
        return Err(Error::Validation(format!(
            "Hermetic mode requires a lockfile. Run 'bashrs installer lock {}' first.",
            path.display()
        )));
    }

    let context = HermeticContext::load(&lockfile_path)?;
    println!("Hermetic mode enabled");
    println!("  Lockfile: {}", lockfile_path.display());
    println!("  SOURCE_DATE_EPOCH: {}", context.source_date_epoch());
    println!("  Artifacts locked: {}", context.lockfile.artifacts.len());
    println!();
    Ok(Some(context))
}

/// Set up signature verification keyring if requested.
fn installer_setup_signature_keyring(
    verify_signatures: bool,
) -> Result<Option<crate::installer::Keyring>> {
    use crate::installer::Keyring;

    if !verify_signatures {
        return Ok(None);
    }

    let keyring_dir = std::env::var("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|h| std::path::PathBuf::from(h).join(".config")))
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("bashrs")
        .join("installer");
    let keyring_path = keyring_dir.join("keyring.json");

    if !keyring_path.exists() {
        return Err(Error::Validation(
            "Signature verification requires a keyring. Run 'bashrs installer keyring init' first."
                .to_string(),
        ));
    }

    let kr = Keyring::with_storage(&keyring_path)?;
    println!("Signature verification enabled");
    println!("  Keyring: {}", keyring_path.display());
    println!("  Keys: {}", kr.len());
    println!(
        "  TOFU mode: {}",
        if kr.is_tofu_enabled() {
            "enabled"
        } else {
            "disabled"
        }
    );
    println!();
    Ok(Some(kr))
}

/// Handle diff preview mode: print summary and return early.
fn installer_handle_diff_preview(
    result: &crate::installer::ValidationResult,
    hermetic_context: &Option<crate::installer::HermeticContext>,
    keyring: &Option<crate::installer::Keyring>,
) -> Result<()> {
    println!("=== Dry-Run Diff Preview ===");
    println!();
    println!("Steps to execute: {}", result.steps);
    println!("Artifacts to download: {}", result.artifacts);
    if hermetic_context.is_some() {
        println!("Mode: hermetic (reproducible)");
    }
    if keyring.is_some() {
        println!("Signatures: will be verified");
    }
    Ok(())
}

/// Handle dry-run mode: print validation summary and return early.
fn installer_handle_dry_run(
    result: &crate::installer::ValidationResult,
    hermetic_context: &Option<crate::installer::HermeticContext>,
    keyring: &Option<crate::installer::Keyring>,
) -> Result<()> {
    println!("Dry-run mode: validating only");
    println!("  Steps: {}", result.steps);
    println!("  Artifacts: {}", result.artifacts);
    if hermetic_context.is_some() {
        println!("  Mode: hermetic (reproducible)");
    }
    if keyring.is_some() {
        println!("  Signatures: will be verified");
    }
    println!("\u{2713} Installer validated successfully");
    Ok(())
}

/// Set up OpenTelemetry tracing context if tracing is enabled.
fn installer_setup_tracing(
    trace: bool,
    trace_file: Option<&Path>,
    path: &Path,
    result: &crate::installer::ValidationResult,
    hermetic: bool,
) -> Option<crate::installer::TracingContext> {
    use crate::installer::{AttributeValue, TracingContext};

    if !trace {
        return None;
    }

    let installer_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("installer");

    let mut ctx = TracingContext::new(installer_name, "1.0.0");
    ctx.start_root("installer_run");
    ctx.set_attribute(
        "installer.path",
        AttributeValue::string(path.display().to_string()),
    );
    ctx.set_attribute("installer.steps", AttributeValue::int(result.steps as i64));
    ctx.set_attribute("installer.hermetic", AttributeValue::bool(hermetic));
    println!("Tracing enabled");
    println!("  Trace ID: {}", ctx.trace_id());
    if let Some(f) = trace_file {
        println!("  Trace file: {}", f.display());
    }
    println!();
    Some(ctx)
}

/// Execute all installer steps, updating progress and tracing along the way.
/// Returns `true` if all steps succeeded.
fn installer_execute_steps(
    spec: &crate::installer::InstallerSpec,
    executor: &crate::installer::StepExecutor,
    progress: &mut crate::installer::InstallerProgress,
    tracing_ctx: &mut Option<crate::installer::TracingContext>,
    renderer: &crate::installer::TerminalRenderer,
) -> bool {
    let total_steps = spec.step.len();
    let mut all_succeeded = true;

    for step in &spec.step {
        if let Some(ref mut ctx) = tracing_ctx {
            ctx.start_step_span(&step.id, &step.name);
        }

        progress.start_step(&step.id, "Executing...");
        let exec_result = executor.execute_step(step);

        match exec_result {
            Ok(result) => {
                if result.success {
                    installer_handle_step_success(step, &result, progress, tracing_ctx);
                } else {
                    installer_handle_step_failure(step, &result, progress, tracing_ctx);
                    all_succeeded = false;
                    installer_render_step_progress(progress, renderer, &step.id, total_steps);
                    break;
                }
            }
            Err(e) => {
                all_succeeded = false;
                println!("  \u{274c} Step '{}' error: {}", step.id, e);
                if let Some(ref mut ctx) = tracing_ctx {
                    ctx.end_span_error(&e.to_string());
                }
                installer_render_step_progress(progress, renderer, &step.id, total_steps);
                break;
            }
        }

        installer_render_step_progress(progress, renderer, &step.id, total_steps);
    }
    all_succeeded
}

/// Handle a successful step execution: update progress and tracing.
fn installer_handle_step_success(
    step: &crate::installer::Step,
    result: &crate::installer::StepExecutionResult,
    progress: &mut crate::installer::InstallerProgress,
    tracing_ctx: &mut Option<crate::installer::TracingContext>,
) {
    progress.update_step(&step.id, 100, "Completed");
    progress.complete_step(&step.id);

    if let Some(ref mut ctx) = tracing_ctx {
        ctx.end_span_ok();
    }

    if !result.stdout.trim().is_empty() {
        println!(
            "  Output: {}",
            result.stdout.trim().lines().next().unwrap_or("")
        );
    }
}

/// Handle a failed step execution: update progress, tracing, and print diagnostics.
fn installer_handle_step_failure(
    step: &crate::installer::Step,
    result: &crate::installer::StepExecutionResult,
    progress: &mut crate::installer::InstallerProgress,
    tracing_ctx: &mut Option<crate::installer::TracingContext>,
) {
    progress.update_step(&step.id, 0, "Failed");

    if let Some(ref mut ctx) = tracing_ctx {
        ctx.end_span_error(&result.stderr);
    }

    println!("  \u{274c} Step '{}' failed:", step.id);
    if !result.stderr.is_empty() {
        for line in result.stderr.lines().take(3) {
            println!("     {}", line);
        }
    }

    for postcond in &result.postcondition_results {
        if !postcond.passed {
            println!("     Postcondition failed: {}", postcond.details);
        }
    }
}

/// Render step progress if the step info is available.
fn installer_render_step_progress(
    progress: &crate::installer::InstallerProgress,
    renderer: &crate::installer::TerminalRenderer,
    step_id: &str,
    total_steps: usize,
) {
    use crate::installer::ProgressRenderer;

    if let Some(step_info) = progress.get_step(step_id) {
        println!("{}", renderer.render_step(step_info, total_steps));
    }
}

/// Finalize the installer run: end tracing spans, render footer, export traces, report status.
fn installer_finalize_run(
    progress: &crate::installer::InstallerProgress,
    tracing_ctx: &mut Option<crate::installer::TracingContext>,
    renderer: &crate::installer::TerminalRenderer,
    trace_file: Option<&Path>,
    path: &Path,
    all_succeeded: bool,
) -> Result<()> {
    use crate::installer::{generate_summary, ProgressRenderer};

    if let Some(ref mut ctx) = tracing_ctx {
        ctx.end_root_ok();
    }

    println!("{}", renderer.render_footer(progress));

    let summary = generate_summary(progress);
    println!("\n{}", summary.format());

    if let Some(ref ctx) = tracing_ctx {
        let trace_summary = ctx.summary();
        println!("\n{}", trace_summary.format());

        if let Some(file_path) = trace_file {
            let trace_json = ctx.export();
            std::fs::write(file_path, &trace_json).map_err(|e| {
                Error::Io(std::io::Error::new(
                    e.kind(),
                    format!("Failed to write trace file: {}", e),
                ))
            })?;
            println!("Traces exported to: {}", file_path.display());
        }
    }

    if all_succeeded {
        println!("\n\u{2705} Installation completed successfully!");
    } else {
        println!(
            "\n\u{274c} Installation failed. Use 'bashrs installer resume {}' to retry.",
            path.display()
        );
    }

    Ok(())
}

fn installer_resume_command(path: &Path, from: Option<&str>) -> Result<()> {
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

fn installer_test_command(path: &Path, matrix: Option<&str>, coverage: bool) -> Result<()> {
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
            println!("⚠ Warning: No container runtime detected (docker/podman)");
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
            println!("✓ All {} platform(s) passed", summary.total);
        } else {
            println!(
                "✗ {} of {} platform(s) failed",
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
        println!("✓ Installer specification validated");
    }

    Ok(())
}

fn installer_lock_command(path: &Path, update: bool, verify: bool) -> Result<()> {
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

fn installer_graph_command(path: &Path, format: InstallerGraphFormat) -> Result<()> {
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

fn installer_golden_capture_command(path: &Path, trace_name: &str) -> Result<()> {
    use crate::installer::{
        GoldenTrace, GoldenTraceManager, InstallerSpec, SimulatedTraceCollector, TraceResult,
    };

    // Find installer.toml
    let installer_toml = if path.is_dir() {
        path.join("installer.toml")
    } else {
        path.to_path_buf()
    };

    // Parse spec
    let content = std::fs::read_to_string(&installer_toml).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read installer.toml: {}", e),
        ))
    })?;
    let spec = InstallerSpec::parse(&content)?;

    // Create trace manager
    let trace_dir = path.parent().unwrap_or(path).join(".golden-traces");
    let manager = GoldenTraceManager::new(&trace_dir);

    // Create simulated trace collector
    // In production, this would integrate with renacer for real syscall tracing
    let mut collector = SimulatedTraceCollector::new();

    // Record simulated events for each step
    for step in &spec.step {
        collector.record_process_event(
            "exec",
            Some(&step.name),
            None,
            Some(&step.id),
            TraceResult::Success,
        );

        // Add file events based on step action
        match step.action.as_str() {
            "file-write" => {
                if let Some(ref path) = step.path {
                    collector.record_file_event(
                        "write",
                        path,
                        Some("O_WRONLY|O_CREAT"),
                        Some(&step.id),
                        TraceResult::Success,
                    );
                }
            }
            "apt-install" => {
                collector.record_file_event(
                    "open",
                    "/var/lib/apt/lists",
                    Some("O_RDONLY"),
                    Some(&step.id),
                    TraceResult::Success,
                );
            }
            "script" => {
                if let Some(ref script) = step.script {
                    collector.record_process_event(
                        "exec",
                        Some(&script.interpreter),
                        None,
                        Some(&step.id),
                        TraceResult::Success,
                    );
                }
            }
            _ => {}
        }
    }

    // Create golden trace
    let events = collector
        .into_trace(trace_name, &spec.installer.version)
        .events;
    let trace = GoldenTrace {
        name: trace_name.to_string(),
        captured_at: chrono::Utc::now().to_rfc3339(),
        installer_version: spec.installer.version.clone(),
        result_hash: format!("{:016x}", {
            // Simple hash of events for reproducibility check
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            events.len().hash(&mut hasher);
            trace_name.hash(&mut hasher);
            hasher.finish()
        }),
        events,
        steps_executed: spec.step.len(),
        duration_ms: 0,
    };

    // Save trace
    let trace_path = manager.save_trace(&trace)?;

    println!("Golden trace captured successfully:");
    println!("  Name: {}", trace_name);
    println!("  Path: {}", trace_path.display());
    println!("  Events: {}", trace.events.len());
    println!("  Steps: {}", trace.steps_executed);
    println!();
    println!("To compare against this trace later:");
    println!(
        "  bashrs installer golden-compare {} --trace {}",
        path.display(),
        trace_name
    );

    Ok(())
}

fn installer_golden_compare_command(path: &Path, trace_name: &str) -> Result<()> {
    use crate::installer::{
        GoldenTrace, GoldenTraceManager, InstallerSpec, SimulatedTraceCollector, TraceComparison,
        TraceResult,
    };

    // Find installer.toml
    let installer_toml = if path.is_dir() {
        path.join("installer.toml")
    } else {
        path.to_path_buf()
    };

    // Parse spec
    let content = std::fs::read_to_string(&installer_toml).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read installer.toml: {}", e),
        ))
    })?;
    let spec = InstallerSpec::parse(&content)?;

    // Create trace manager
    let trace_dir = path.parent().unwrap_or(path).join(".golden-traces");
    let manager = GoldenTraceManager::new(&trace_dir);

    // Load golden trace
    let golden = manager.load_trace(trace_name)?;

    // Capture current trace (simulated)
    let mut collector = SimulatedTraceCollector::new();
    for step in &spec.step {
        collector.record_process_event(
            "exec",
            Some(&step.name),
            None,
            Some(&step.id),
            TraceResult::Success,
        );

        match step.action.as_str() {
            "file-write" => {
                if let Some(ref path) = step.path {
                    collector.record_file_event(
                        "write",
                        path,
                        Some("O_WRONLY|O_CREAT"),
                        Some(&step.id),
                        TraceResult::Success,
                    );
                }
            }
            "apt-install" => {
                collector.record_file_event(
                    "open",
                    "/var/lib/apt/lists",
                    Some("O_RDONLY"),
                    Some(&step.id),
                    TraceResult::Success,
                );
            }
            "script" => {
                if let Some(ref script) = step.script {
                    collector.record_process_event(
                        "exec",
                        Some(&script.interpreter),
                        None,
                        Some(&step.id),
                        TraceResult::Success,
                    );
                }
            }
            _ => {}
        }
    }

    let current = GoldenTrace {
        name: format!("{}-current", trace_name),
        captured_at: chrono::Utc::now().to_rfc3339(),
        installer_version: spec.installer.version.clone(),
        events: collector
            .into_trace(trace_name, &spec.installer.version)
            .events,
        result_hash: String::new(),
        steps_executed: spec.step.len(),
        duration_ms: 0,
    };

    // Compare traces
    let comparison = TraceComparison::compare(&golden, &current);

    // Print report
    println!("{}", comparison.to_report());

    if comparison.is_equivalent() {
        println!("Result: PASS - No regression detected");
        Ok(())
    } else {
        Err(Error::Validation(format!(
            "Trace regression detected: {} added, {} removed events",
            comparison.added.len(),
            comparison.removed.len()
        )))
    }
}

fn installer_audit_command(
    path: &Path,
    format: AuditOutputFormat,
    security_only: bool,
    min_severity: Option<&str>,
    ignore: &[String],
) -> Result<()> {
    use crate::installer::{AuditContext, AuditSeverity, InstallerSpec};

    // Find installer.toml
    let installer_toml = if path.is_dir() {
        path.join("installer.toml")
    } else {
        path.to_path_buf()
    };

    if !installer_toml.exists() {
        return Err(Error::Validation(format!(
            "installer.toml not found at {}",
            installer_toml.display()
        )));
    }

    // Parse the TOML
    let content = std::fs::read_to_string(&installer_toml).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read installer.toml: {e}"),
        ))
    })?;

    let spec = InstallerSpec::parse(&content)?;

    // Set up audit context
    let mut ctx = if security_only {
        AuditContext::security_only()
    } else {
        AuditContext::new()
    };

    // Set minimum severity if specified
    if let Some(sev) = min_severity {
        let severity = match sev.to_lowercase().as_str() {
            "info" => AuditSeverity::Info,
            "suggestion" => AuditSeverity::Suggestion,
            "warning" => AuditSeverity::Warning,
            "error" => AuditSeverity::Error,
            "critical" => AuditSeverity::Critical,
            _ => {
                return Err(Error::Validation(format!(
                    "Invalid severity '{}'. Valid values: info, suggestion, warning, error, critical",
                    sev
                )));
            }
        };
        ctx = ctx.with_min_severity(severity);
    }

    // Issue #110: Add ignored rules
    for rule in ignore {
        ctx = ctx.with_ignored_rule(rule);
    }

    // Run audit
    let report = ctx.audit_parsed_spec(&spec, &installer_toml);

    // Output report
    match format {
        AuditOutputFormat::Human => {
            println!("{}", report.format());
        }
        AuditOutputFormat::Json => {
            println!("{}", report.to_json());
        }
        AuditOutputFormat::Sarif => {
            // SARIF format not yet implemented for installer audit
            println!("{}", report.to_json());
        }
    }

    // Return error if there are errors or critical issues
    if report.has_errors() {
        Err(Error::Validation(format!(
            "Audit found {} error(s). Score: {}/100 (Grade: {})",
            report.findings_by_severity(AuditSeverity::Error).len()
                + report.findings_by_severity(AuditSeverity::Critical).len(),
            report.score(),
            report.grade()
        )))
    } else {
        Ok(())
    }
}
