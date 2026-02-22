use crate::models::{Error, Result};
use std::path::Path;

// ============================================================================
// INSTALLER RUN + RESUME COMMANDS
// Extracted from installer_commands.rs for module size compliance
// ============================================================================

#[allow(clippy::fn_params_excessive_bools, clippy::too_many_arguments)]
pub(crate) fn installer_run_command(
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

