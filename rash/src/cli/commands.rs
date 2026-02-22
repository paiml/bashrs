#[cfg(feature = "oracle")]
use crate::cli::args::ExplainErrorFormat;
use crate::cli::args::{
    CompileRuntime, ContainerFormatArg, InspectionFormat,
};
#[cfg(feature = "oracle")]
use crate::cli::logic::extract_exit_code;
use crate::cli::logic::{is_shell_script_file, normalize_shell_script};
// Test-only imports from crate::cli::logic (needed by command_tests.rs via `super::*`)
#[cfg(test)]
use crate::cli::args::{
    ConfigOutputFormat, LintFormat, MakeOutputFormat,
};
#[cfg(test)]
use crate::cli::logic::{
    add_no_install_recommends, add_package_manager_cleanup, convert_add_to_copy_if_local,
    find_devcontainer_json as logic_find_devcontainer_json, format_timestamp, generate_diff_lines,
    hex_encode, pin_base_image_version, truncate_str,
};
use crate::cli::{Cli, Commands};
use crate::models::{Config, Error, Result};
use crate::{check, transpile};
use std::fs;
use std::path::Path;
use tracing::{info, warn};

#[cfg(test)]
#[path = "command_tests.rs"]
mod command_tests;


// ---------------------------------------------------------------------------
// Extracted command modules (thin dispatch -> dedicated files)
// ---------------------------------------------------------------------------

// Lint, purify, format, playbook, mutate, simulate command modules
#[path = "lint_commands.rs"]
mod lint_cmds;
#[path = "purify_commands.rs"]
mod purify_cmds;
#[path = "format_commands.rs"]
mod format_cmds;
#[path = "playbook_commands.rs"]
mod playbook_cmds;
#[path = "mutate_commands.rs"]
mod mutate_cmds;
#[path = "simulate_commands.rs"]
mod simulate_cmds;

// Re-import so existing dispatch calls and tests still work
use lint_cmds::lint_command;
use purify_cmds::purify_command;
use format_cmds::format_command;
use playbook_cmds::playbook_command;
use mutate_cmds::mutate_command;
use simulate_cmds::simulate_command;

// Quality command modules
#[path = "test_commands.rs"]
mod test_commands;
#[path = "score_commands.rs"]
mod score_commands;
#[path = "audit_commands.rs"]
mod audit_commands;
#[path = "coverage_commands.rs"]
mod coverage_commands;

#[cfg(test)]
use test_commands::test_command;
#[cfg(test)]
use score_commands::score_command;
#[cfg(test)]
use audit_commands::audit_command;
#[cfg(test)]
use coverage_commands::coverage_command;

// Gate, make, devcontainer, config, comply command modules
#[path = "gate_commands.rs"]
mod gate_cmds;
#[path = "make_commands.rs"]
mod make_cmds;
#[path = "devcontainer_commands.rs"]
mod devcontainer_cmds;
#[path = "config_commands.rs"]
mod config_cmds;
#[path = "comply_commands.rs"]
mod comply_cmds;

// Corpus command modules (25 files).
// Module names must match the `super::xxx` references used inside these files.
#[path = "corpus_core_commands.rs"]
mod corpus_core_cmds;
#[path = "corpus_score_print_commands.rs"]
pub(super) mod corpus_score_print_commands;
#[path = "corpus_report_commands.rs"]
pub(super) mod corpus_report_commands;
#[path = "corpus_entry_commands.rs"]
pub(super) mod corpus_entry_commands;
#[path = "corpus_analysis_commands.rs"]
pub(super) mod corpus_analysis_commands;
#[path = "corpus_diff_commands.rs"]
pub(super) mod corpus_diff_commands;
#[path = "corpus_display_commands.rs"]
pub(super) mod corpus_display_commands;
#[path = "corpus_ranking_commands.rs"]
pub(super) mod corpus_ranking_commands;
#[path = "corpus_failure_commands.rs"]
pub(super) mod corpus_failure_commands;
#[path = "corpus_gate_commands.rs"]
pub(super) mod corpus_gate_commands;
#[path = "corpus_diag_commands.rs"]
pub(super) mod corpus_diag_commands;
#[path = "corpus_tier_commands.rs"]
pub(super) mod corpus_tier_commands;
#[path = "corpus_time_commands.rs"]
pub(super) mod corpus_time_commands;
#[path = "corpus_ops_commands.rs"]
pub(super) mod corpus_ops_commands;
#[path = "corpus_compare_commands.rs"]
pub(super) mod corpus_compare_commands;
#[path = "corpus_metrics_commands.rs"]
pub(super) mod corpus_metrics_commands;
#[path = "corpus_viz_commands.rs"]
pub(super) mod corpus_viz_commands;
#[path = "corpus_weight_commands.rs"]
pub(super) mod corpus_weight_commands;
#[path = "corpus_convergence_commands.rs"]
pub(super) mod corpus_convergence_commands;
#[path = "corpus_b2_commands.rs"]
pub(super) mod corpus_b2_commands;
#[path = "corpus_b2_fix_commands.rs"]
pub(super) mod corpus_b2_fix_commands;
#[path = "corpus_decision_commands.rs"]
pub(super) mod corpus_decision_commands;
#[path = "corpus_advanced_commands.rs"]
pub(super) mod corpus_advanced_commands;
#[path = "corpus_pipeline_commands.rs"]
pub(super) mod corpus_pipeline_commands;
#[path = "corpus_config_commands.rs"]
pub(super) mod corpus_config_commands;

// Re-export convert_lint_format at module scope (needed by lint_cmds via super::)
use make_cmds::convert_lint_format;

// Re-exports needed only by tests (command_tests.rs and inline test modules use `super::*`)
#[cfg(test)]
use make_cmds::{
    make_lint_command, make_parse_command, make_purify_command,
    run_filtered_lint, show_lint_results,
};
#[cfg(test)]
use config_cmds::{
    config_analyze_command, config_lint_command, count_duplicate_path_entries,
    handle_output_to_file, should_output_to_stdout,
};
// Dockerfile and installer are sibling modules declared in cli/mod.rs.
// Re-export their public functions so command_tests.rs (`super::*`) can reach them.
#[cfg(test)]
use super::dockerfile_commands::{
    dockerfile_lint_command, dockerfile_purify_command, purify_dockerfile,
};
#[cfg(test)]
use super::dockerfile_profile_commands::{
    dockerfile_profile_command, dockerfile_size_check_command, estimate_build_time,
};
#[cfg(test)]
use super::dockerfile_validate_commands::dockerfile_full_validate_command;
#[cfg(test)]
use super::installer_commands::parse_public_key;

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
        .map_err(|e| Error::Internal(format!("Failed to initialize logging: {e}")))?;

    match cli.command {
        Commands::Build {
            input,
            output,
            emit_proof,
            no_optimize,
        } => {
            info!("Building {} -> {}", input.display(), output.display());

            let config = Config {
                target: cli.target,
                verify: cli.verify,
                emit_proof,
                optimize: !no_optimize,
                validation_level: Some(cli.validation),
                strict_mode: cli.strict,
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

        Commands::Verify {
            rust_source,
            shell_script,
        } => {
            info!(
                "Verifying {} against {}",
                shell_script.display(),
                rust_source.display()
            );
            verify_command(&rust_source, &shell_script, cli.target, cli.verify)
        }

        Commands::Inspect {
            input,
            format,
            output,
            detailed,
        } => {
            info!("Generating inspection report for: {}", input);
            inspect_command(&input, format, output.as_deref(), detailed)
        }

        Commands::Compile {
            rust_source,
            output,
            runtime,
            self_extracting,
            container,
            container_format,
        } => {
            let config = Config {
                target: cli.target,
                verify: cli.verify,
                emit_proof: false,
                optimize: true,
                validation_level: Some(cli.validation),
                strict_mode: cli.strict,
            };

            handle_compile(
                &rust_source,
                &output,
                runtime,
                self_extracting,
                container,
                container_format,
                &config,
            )
        }

        Commands::Lint {
            input,
            format,
            fix,
            fix_assumptions,
            output,
            no_ignore,
            ignore_file,
            quiet,
            level,
            ignore,
            exclude,
            citl_export,
            profile,
            graded,
        } => {
            info!("Linting {}", input.display());
            lint_command(
                &input,
                format,
                fix,
                fix_assumptions,
                output.as_deref(),
                no_ignore,
                ignore_file.as_deref(),
                quiet,
                level,
                ignore.as_deref(),
                exclude.as_deref(),
                citl_export.as_deref(),
                profile,
                graded,
            )
        }

        Commands::Purify {
            input,
            output,
            report,
            with_tests,
            property_tests,
            type_check,
            emit_guards,
            type_strict,
        } => {
            info!("Purifying {}", input.display());
            purify_command(
                &input,
                output.as_deref(),
                report,
                with_tests,
                property_tests,
                type_check,
                emit_guards,
                type_strict,
            )
        }

        Commands::Make { command } => make_cmds::handle_make_command(command),

        Commands::Dockerfile { command } => {
            super::dockerfile_commands::handle_dockerfile_command(command)
        }

        Commands::Devcontainer { command } => {
            devcontainer_cmds::handle_devcontainer_command(command)
        }

        Commands::Config { command } => config_cmds::handle_config_command(command),

        Commands::Repl {
            debug,
            sandboxed,
            max_memory,
            timeout,
            max_depth,
        } => {
            info!("Starting interactive REPL");
            handle_repl_command(debug, sandboxed, max_memory, timeout, max_depth)
        }

        #[cfg(feature = "tui")]
        Commands::Tui => {
            info!("Starting TUI");
            crate::tui::run()
                .map_err(|e| crate::models::Error::Io(std::io::Error::other(e.to_string())))
        }

        Commands::Test {
            input,
            format,
            detailed,
            pattern,
        } => {
            info!("Running tests in {}", input.display());
            test_commands::test_command(&input, format, detailed, pattern.as_deref())
        }

        Commands::Score {
            input,
            format,
            detailed,
            dockerfile,
            runtime,
            grade,
            profile,
        } => {
            info!("Scoring {}", input.display());
            score_commands::score_command(
                &input, format, detailed, dockerfile, runtime, grade, profile,
            )
        }

        Commands::Audit {
            input,
            format,
            strict,
            detailed,
            min_grade,
        } => {
            info!("Running comprehensive quality audit on {}", input.display());
            audit_commands::audit_command(&input, &format, strict, detailed, min_grade.as_deref())
        }

        Commands::Coverage {
            input,
            format,
            min,
            detailed,
            output,
        } => {
            info!("Generating coverage report for {}", input.display());
            coverage_commands::coverage_command(&input, &format, min, detailed, output.as_deref())
        }

        Commands::Format {
            inputs,
            check,
            dry_run,
            output,
        } => {
            info!("Formatting bash script(s)");
            format_command(&inputs, check, dry_run, output.as_deref())
        }

        Commands::Bench {
            scripts,
            warmup,
            iterations,
            output,
            strict,
            verify_determinism,
            show_raw,
            quiet,
            measure_memory,
            csv,
            no_color,
        } => {
            info!("Benchmarking script(s)");
            use crate::cli::bench::{bench_command, BenchOptions};

            let options = BenchOptions {
                scripts,
                warmup,
                iterations,
                output,
                strict,
                verify_determinism,
                show_raw,
                quiet,
                measure_memory,
                csv,
                no_color,
            };

            bench_command(options)
        }

        Commands::Gate { tier, report } => {
            info!("Executing Tier {} quality gates", tier);
            gate_cmds::handle_gate_command(tier, report)
        }

        #[cfg(feature = "oracle")]
        Commands::ExplainError {
            error,
            command,
            shell,
            format,
            detailed,
        } => {
            info!("Explaining error using ML oracle");
            explain_error_command(&error, command.as_deref(), &shell, format, detailed)
        }

        Commands::Playbook {
            input,
            run,
            format,
            verbose,
            dry_run,
        } => {
            info!("Executing playbook: {}", input.display());
            playbook_command(&input, run, format, verbose, dry_run)
        }

        Commands::Mutate {
            input,
            config,
            format,
            count,
            show_survivors,
            output,
        } => {
            info!("Mutation testing: {}", input.display());
            mutate_command(
                &input,
                config.as_deref(),
                format,
                count,
                show_survivors,
                output.as_deref(),
            )
        }

        Commands::Simulate {
            input,
            seed,
            verify,
            mock_externals,
            format,
            trace,
        } => {
            info!("Simulating: {} with seed {}", input.display(), seed);
            simulate_command(&input, seed, verify, mock_externals, format, trace)
        }

        Commands::Installer { command } => {
            info!("Executing installer command");
            super::installer_commands::handle_installer_command(command)
        }

        Commands::Comply { command } => {
            info!("Executing comply command");
            comply_cmds::handle_comply_command(command)
        }

        Commands::Corpus { command } => {
            info!("Executing corpus command");
            corpus_core_cmds::handle_corpus_command(command)
        }
    }
}

// ---------------------------------------------------------------------------
// Core functions (small, kept in commands.rs)
// ---------------------------------------------------------------------------

/// Explain shell error using ML classification (v6.40.0)
#[cfg(feature = "oracle")]
fn explain_error_command(
    error: &str,
    command: Option<&str>,
    _shell: &str,
    format: ExplainErrorFormat,
    detailed: bool,
) -> Result<()> {
    use bashrs_oracle::{ErrorFeatures, Oracle};

    // Load or train the oracle (cached model for performance)
    let oracle = Oracle::load_or_train()
        .map_err(|e| Error::Internal(format!("Failed to load ML oracle: {e}")))?;

    // Extract exit code from error message if present (e.g., "exit code 127")
    let exit_code = extract_exit_code(error);

    // Classify the error
    let features = ErrorFeatures::extract(exit_code, error, command);
    let result = oracle
        .classify(&features)
        .map_err(|e| Error::Internal(format!("Classification failed: {e}")))?;

    match format {
        ExplainErrorFormat::Human => {
            println!("Category: {}", result.category.name());
            println!("Confidence: {:.1}%", result.confidence * 100.0);
            println!();
            if let Some(fix) = &result.suggested_fix {
                println!("Suggested Fix:");
                println!("  {fix}");
            } else {
                println!("Suggested Fix:");
                println!("  {}", result.category.fix_suggestion());
            }

            if detailed && !result.related_patterns.is_empty() {
                println!();
                println!("Related Patterns:");
                for pattern in &result.related_patterns {
                    println!("  - {pattern}");
                }
            }

            if detailed {
                println!();
                println!("Error Analysis:");
                println!("  Exit code: {exit_code}");
                if let Some(cmd) = command {
                    println!("  Command: {cmd}");
                }
            }
        }
        ExplainErrorFormat::Json => {
            let output = serde_json::json!({
                "category": result.category.name(),
                "confidence": result.confidence,
                "suggested_fix": result.suggested_fix.as_deref()
                    .unwrap_or_else(|| result.category.fix_suggestion()),
                "related_patterns": result.related_patterns,
                "exit_code": exit_code,
                "command": command,
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&output)
                    .map_err(|e| Error::Internal(format!("JSON serialization failed: {e}")))?
            );
        }
    }

    Ok(())
}

// extract_exit_code moved to cli/logic.rs

/// Wrap an error with file path and source code context for rich diagnostics
fn with_context(error: Error, file: &Path, source: &str) -> Error {
    Error::WithContext {
        inner: Box::new(error),
        file: Some(file.display().to_string()),
        source_code: Some(source.to_string()),
    }
}

fn build_command(input: &Path, output: &Path, config: Config) -> Result<()> {
    // Read input file
    let source = fs::read_to_string(input).map_err(Error::Io)?;

    // Transpile (wrap errors with source context)
    let shell_code =
        transpile(&source, config.clone()).map_err(|e| with_context(e, input, &source))?;

    // Write output
    fs::write(output, shell_code).map_err(Error::Io)?;

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
    let source = fs::read_to_string(input).map_err(Error::Io)?;

    // Issue #84: Detect if this is a shell script (not Rash source)
    // This prevents confusing false positives when users run `bashrs check` on .sh files
    let is_shell_script = is_shell_script_file(input, &source);

    if is_shell_script {
        // Provide helpful guidance instead of a confusing parse error
        return Err(Error::CommandFailed {
            message: format!(
                "File '{}' appears to be a shell script, not Rash source.\n\n\
                 The 'check' command is for verifying Rash (.rs) source files that will be\n\
                 transpiled to shell scripts.\n\n\
                 For linting existing shell scripts, use:\n\
                 \x1b[1m  bashrs lint {}\x1b[0m\n\n\
                 For purifying shell scripts (adding determinism/idempotency):\n\
                 \x1b[1m  bashrs purify {}\x1b[0m",
                input.display(),
                input.display(),
                input.display()
            ),
        });
    }

    // Check Rash compatibility (wrap errors with source context)
    check(&source).map_err(|e| with_context(e, input, &source))?;

    info!("✓ {} is compatible with Rash", input.display());
    Ok(())
}

fn init_command(path: &Path, name: Option<&str>) -> Result<()> {
    // Create directory if it doesn't exist
    if !path.exists() {
        fs::create_dir_all(path).map_err(Error::Io)?;
    }

    let project_name = name.unwrap_or(
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-installer"),
    );

    // Create Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
# No dependencies needed - Rash transpiles to pure shell

[[bin]]
name = "install"
path = "src/main.rs"
"#
    );

    fs::write(path.join("Cargo.toml"), cargo_toml).map_err(Error::Io)?;

    // Create src directory
    let src_dir = path.join("src");
    fs::create_dir_all(&src_dir).map_err(Error::Io)?;

    // Create main.rs with example installer
    let main_rs = r#"/// Example installer script for Rash
/// This will be transpiled to POSIX-compliant shell script
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, exit};

const VERSION: &str = "0.1.0";
const BINARY_NAME: &str = "myapp";

fn main() {
    println!("{} installer v{}", BINARY_NAME, VERSION);
    println!("=======================");
    
    // Parse arguments
    let args: Vec<String> = env::args().collect();
    if args.contains(&"--help".to_string()) {
        print_help();
        return;
    }
    
    // Determine installation directory
    let prefix = env::var("PREFIX").unwrap_or_else(|_| "/usr/local".to_string());
    let bin_dir = format!("{}/bin", prefix);
    
    println!("Installing to: {}", bin_dir);
    
    // Create directory
    if let Err(e) = fs::create_dir_all(&bin_dir) {
        eprintln!("Failed to create directory: {}", e);
        exit(1);
    }
    
    // Download binary (example URL)
    let url = format!(
        "https://github.com/example/{}/releases/download/v{}/{}-{}.tar.gz",
        BINARY_NAME, VERSION, BINARY_NAME, detect_platform()
    );
    
    println!("Downloading from: {}", url);

    // Installation logic would go here:
    // - Download binary
    // - Verify checksum
    // - Extract and install
    // - Set permissions

    println!("\n✓ {} installed successfully!", BINARY_NAME);
    println!("\nTo get started, run:");
    println!("  {} --help", BINARY_NAME);
}

fn print_help() {
    println!("Usage: install.sh [OPTIONS]");
    println!("\nOptions:");
    println!("  --help       Show this help message");
    println!("  --prefix DIR Install to DIR (default: /usr/local)");
}

fn detect_platform() -> &'static str {
    // Simplified platform detection
    if cfg!(target_os = "linux") {
        if cfg!(target_arch = "x86_64") {
            "linux-amd64"
        } else {
            "linux-arm64"
        }
    } else if cfg!(target_os = "macos") {
        "darwin-amd64"
    } else {
        panic!("Unsupported platform");
    }
}"#;

    fs::write(src_dir.join("main.rs"), main_rs).map_err(Error::Io)?;

    // Create .rash.toml
    let rash_toml = r##"# Rash configuration file
[transpiler]
target = "posix"          # Target shell dialect
strict_mode = true        # Fail on warnings
preserve_comments = false # Strip comments for smaller output

[validation]
level = "strict"          # ShellCheck compliance level
rules = ["all"]           # Can disable specific: ["-SC2034"]
external_check = false    # Run actual shellcheck binary

[output]
shebang = "#!/bin/sh"     # POSIX shebang
set_flags = "euf"         # set -euf (no pipefail in POSIX)
optimize_size = true      # Minimize output script size

[style]
indent = "    "           # 4 spaces
max_line_length = 100     # Wrap long commands
"##;

    fs::write(path.join(".rash.toml"), rash_toml).map_err(Error::Io)?;

    info!("✓ Initialized Rash project '{}'", project_name);
    info!("  Run 'cd {}' to enter the project", path.display());
    info!("  Run 'rash build src/main.rs' to build");

    Ok(())
}

fn verify_command(
    rust_source: &Path,
    shell_script: &Path,
    target: crate::models::ShellDialect,
    verify_level: crate::models::VerificationLevel,
) -> Result<()> {
    // Read both files
    let rust_code = fs::read_to_string(rust_source).map_err(Error::Io)?;
    let shell_code = fs::read_to_string(shell_script).map_err(Error::Io)?;

    // Transpile Rust to shell
    let config = Config {
        target,
        verify: verify_level,
        emit_proof: false,
        optimize: true,
        strict_mode: true,
        validation_level: Some(crate::validation::ValidationLevel::Strict),
    };

    let generated_shell = transpile(&rust_code, config)?;

    // Compare generated vs actual
    if normalize_shell_script(&generated_shell) == normalize_shell_script(&shell_code) {
        info!("✓ Shell script matches Rust source");
        Ok(())
    } else {
        warn!("Shell script does not match Rust source");
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

    fs::write(proof_path, proof).map_err(Error::Io)?;

    Ok(())
}

fn inspect_command(
    input: &str,
    format: InspectionFormat,
    output: Option<&Path>,
    _detailed: bool,
) -> Result<()> {
    use crate::formal::{AbstractState, ProofInspector, TinyAst};

    // Parse input - for now, we'll support JSON AST or a few predefined examples
    let ast = if input.starts_with('{') {
        // JSON input
        serde_json::from_str::<TinyAst>(input)
            .map_err(|e| Error::Internal(format!("Invalid AST JSON: {e}")))?
    } else {
        // Predefined examples or simple DSL
        match input {
            "echo-example" => TinyAst::ExecuteCommand {
                command_name: "echo".to_string(),
                args: vec!["Hello, World!".to_string()],
            },
            "bootstrap-example" => TinyAst::Sequence {
                commands: vec![
                    TinyAst::SetEnvironmentVariable {
                        name: "INSTALL_DIR".to_string(),
                        value: "/opt/rash".to_string(),
                    },
                    TinyAst::ExecuteCommand {
                        command_name: "mkdir".to_string(),
                        args: vec!["-p".to_string(), "/opt/rash/bin".to_string()],
                    },
                    TinyAst::ChangeDirectory {
                        path: "/opt/rash".to_string(),
                    },
                    TinyAst::ExecuteCommand {
                        command_name: "echo".to_string(),
                        args: vec!["Installation ready".to_string()],
                    },
                ],
            },
            _ => {
                return Err(Error::Internal(format!("Unknown example: {input}. Try 'echo-example' or 'bootstrap-example', or provide JSON AST")));
            }
        }
    };

    // Validate the AST
    if !ast.is_valid() {
        return Err(Error::Validation("Invalid AST".to_string()));
    }

    // Generate inspection report
    let mut initial_state = AbstractState::new();
    // Add common directories for testing
    initial_state.filesystem.insert(
        std::path::PathBuf::from("/opt"),
        crate::formal::FileSystemEntry::Directory,
    );

    let report = ProofInspector::inspect(&ast, initial_state);

    // Format output
    let output_content = match format {
        InspectionFormat::Markdown => ProofInspector::generate_report(&report),
        InspectionFormat::Json => serde_json::to_string_pretty(&report)
            .map_err(|e| Error::Internal(format!("JSON serialization failed: {e}")))?,
        InspectionFormat::Html => {
            // Convert markdown to HTML (simplified)
            let markdown = ProofInspector::generate_report(&report);
            format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <title>Formal Verification Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 2em; }}
        pre {{ background: #f5f5f5; padding: 1em; border-radius: 4px; }}
        .success {{ color: green; }}
        .failure {{ color: red; }}
        .warning {{ color: orange; }}
    </style>
</head>
<body>
<pre>{}</pre>
</body>
</html>"#,
                markdown
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
            )
        }
    };

    // Write output
    match output {
        Some(path) => {
            fs::write(path, &output_content).map_err(Error::Io)?;
            info!("Inspection report written to {}", path.display());
        }
        None => {
            println!("{output_content}");
        }
    }

    Ok(())
}

fn handle_compile(
    rust_source: &Path,
    output: &Path,
    runtime: CompileRuntime,
    self_extracting: bool,
    container: bool,
    container_format: ContainerFormatArg,
    config: &Config,
) -> Result<()> {
    use crate::compiler::{create_self_extracting_script, BinaryCompiler, RuntimeType};
    use crate::container::{ContainerFormat, DistrolessBuilder};

    info!(
        "Compiling {} to {}",
        rust_source.display(),
        output.display()
    );

    // Read and transpile the source
    let source = fs::read_to_string(rust_source).map_err(Error::Io)?;
    let shell_code = transpile(&source, config.clone())?;

    if self_extracting {
        // Create self-extracting script
        let output_str = output
            .to_str()
            .ok_or_else(|| Error::Validation("Output path contains invalid UTF-8".to_string()))?;
        create_self_extracting_script(&shell_code, output_str)?;
        info!("Created self-extracting script at {}", output.display());
    } else if container {
        // Create container image
        let runtime_type = match runtime {
            CompileRuntime::Dash => RuntimeType::Dash,
            CompileRuntime::Busybox => RuntimeType::Busybox,
            CompileRuntime::Minimal => RuntimeType::Minimal,
        };

        let compiler = BinaryCompiler::new(runtime_type);
        let binary = compiler.compile(&shell_code)?;

        let format = match container_format {
            ContainerFormatArg::Oci => ContainerFormat::OCI,
            ContainerFormatArg::Docker => ContainerFormat::Docker,
        };

        let builder = DistrolessBuilder::new(binary).with_format(format);
        let container_data = builder.build()?;

        fs::write(output, container_data).map_err(Error::Io)?;
        info!("Created container image at {}", output.display());
    } else {
        // Create standalone binary (not fully implemented)
        warn!(
            "Binary compilation not yet fully implemented, creating self-extracting script instead"
        );
        let output_str = output
            .to_str()
            .ok_or_else(|| Error::Validation("Output path contains invalid UTF-8".to_string()))?;
        create_self_extracting_script(&shell_code, output_str)?;
    }

    Ok(())
}












#[cfg(test)]
mod config_purify_tests {
    use super::*;

    // ===== NASA-QUALITY UNIT TESTS for config_purify_command helpers =====

    #[test]
    fn test_should_output_to_stdout_dash() {
        let stdout_path = Path::new("-");
        assert!(
            should_output_to_stdout(stdout_path),
            "Path '-' should output to stdout"
        );
    }

    #[test]
    fn test_should_output_to_stdout_regular_file() {
        let file_path = Path::new("/tmp/output.txt");
        assert!(
            !should_output_to_stdout(file_path),
            "Regular file path should NOT output to stdout"
        );
    }

    #[test]
    fn test_should_output_to_stdout_empty_path() {
        let empty_path = Path::new("");
        assert!(
            !should_output_to_stdout(empty_path),
            "Empty path should NOT output to stdout"
        );
    }

    #[test]
    fn test_generate_diff_lines_no_changes() {
        let original = "line1\nline2\nline3";
        let purified = "line1\nline2\nline3";

        let diffs = generate_diff_lines(original, purified);

        assert!(
            diffs.is_empty(),
            "Identical content should produce no diff lines"
        );
    }

    #[test]
    fn test_generate_diff_lines_single_change() {
        let original = "line1\nline2\nline3";
        let purified = "line1\nMODIFIED\nline3";

        let diffs = generate_diff_lines(original, purified);

        assert_eq!(diffs.len(), 1, "Should have exactly 1 diff");
        let (line_num, orig, pure) = &diffs[0];
        assert_eq!(*line_num, 2, "Diff should be on line 2");
        assert_eq!(orig, "line2", "Original line should be 'line2'");
        assert_eq!(pure, "MODIFIED", "Purified line should be 'MODIFIED'");
    }

    #[test]
    fn test_generate_diff_lines_multiple_changes() {
        let original = "line1\nline2\nline3\nline4";
        let purified = "CHANGED1\nline2\nCHANGED3\nline4";

        let diffs = generate_diff_lines(original, purified);

        assert_eq!(diffs.len(), 2, "Should have exactly 2 diffs");

        let (line_num1, orig1, pure1) = &diffs[0];
        assert_eq!(*line_num1, 1, "First diff on line 1");
        assert_eq!(orig1, "line1");
        assert_eq!(pure1, "CHANGED1");

        let (line_num2, orig2, pure2) = &diffs[1];
        assert_eq!(*line_num2, 3, "Second diff on line 3");
        assert_eq!(orig2, "line3");
        assert_eq!(pure2, "CHANGED3");
    }

    #[test]
    fn test_generate_diff_lines_empty_strings() {
        let original = "";
        let purified = "";

        let diffs = generate_diff_lines(original, purified);

        assert!(diffs.is_empty(), "Empty strings should produce no diffs");
    }

    #[test]
    fn test_generate_diff_lines_all_lines_changed() {
        let original = "A\nB\nC";
        let purified = "X\nY\nZ";

        let diffs = generate_diff_lines(original, purified);

        assert_eq!(diffs.len(), 3, "All 3 lines should be different");
        assert_eq!(diffs[0].0, 1);
        assert_eq!(diffs[1].0, 2);
        assert_eq!(diffs[2].0, 3);
    }

    #[test]
    fn test_generate_diff_lines_preserves_whitespace() {
        let original = "  line1  \nline2";
        let purified = "line1\nline2";

        let diffs = generate_diff_lines(original, purified);

        assert_eq!(diffs.len(), 1, "Should detect whitespace change");
        let (_, orig, pure) = &diffs[0];
        assert_eq!(orig, "  line1  ", "Should preserve original whitespace");
        assert_eq!(pure, "line1", "Should preserve purified whitespace");
    }
}
fn handle_repl_command(
    debug: bool,
    sandboxed: bool,
    max_memory: Option<usize>,
    timeout: Option<u64>,
    max_depth: Option<usize>,
) -> Result<()> {
    use crate::repl::{run_repl, ReplConfig};
    use std::time::Duration;

    // Build config from CLI args
    let mut config = if sandboxed {
        ReplConfig::sandboxed()
    } else {
        ReplConfig::default()
    };

    // Apply debug mode if requested
    if debug {
        config = config.with_debug();
    }

    // Apply CLI overrides
    if let Some(mem) = max_memory {
        config = config.with_max_memory(mem);
    }
    if let Some(t) = timeout {
        config = config.with_timeout(Duration::from_secs(t));
    }
    if let Some(depth) = max_depth {
        config = config.with_max_depth(depth);
    }

    // Run REPL
    run_repl(config).map_err(|e| Error::Internal(format!("REPL error: {e}")))
}


// coverage_class moved to cli/logic.rs






// ============================================================================
// Dockerfile Scoring Output Functions (Issue #10)
// ============================================================================

// ============================================================================
// NASA-QUALITY UNIT TESTS for Dockerfile Helper Functions
// ============================================================================
// TDG Improvement: src/cli/commands.rs scored 67.7/100 (C+)
// Target: Add 52+ direct unit tests to improve score to >85/100 (A)
//
// Test Coverage Strategy:
// - convert_add_to_copy_if_local(): 13 tests (happy path, edges, boundaries, errors)
// - add_no_install_recommends(): 13 tests
// - add_package_manager_cleanup(): 13 tests
// - pin_base_image_version(): 13 tests
//
// Test Naming Convention: test_<function>_<scenario>
// ============================================================================

#[cfg(test)]
mod dockerfile_helper_tests {
    use super::*;

    // ========================================================================
    // FUNCTION 1: convert_add_to_copy_if_local() - 13 tests
    // ========================================================================

    #[test]
    fn test_convert_add_to_copy_if_local_happy_path_local_file() {
        let line = "ADD myfile.txt /app/";
        let result = convert_add_to_copy_if_local(line);
        assert_eq!(
            result, "COPY myfile.txt /app/",
            "Local file should convert ADD to COPY"
        );
    }

    #[test]
    fn test_convert_add_to_copy_if_local_preserves_http_url() {
        let line = "ADD http://example.com/file.tar.gz /tmp/";
        let result = convert_add_to_copy_if_local(line);
        assert_eq!(
            result, line,
            "HTTP URLs should preserve ADD (not convert to COPY)"
        );
    }

    #[test]
    fn test_convert_add_to_copy_if_local_preserves_https_url() {
        let line = "ADD https://example.com/archive.zip /tmp/";
        let result = convert_add_to_copy_if_local(line);
        assert_eq!(
            result, line,
            "HTTPS URLs should preserve ADD (not convert to COPY)"
        );
    }

    #[test]
    fn test_convert_add_to_copy_if_local_preserves_tar_archive() {
        let line = "ADD archive.tar /tmp/";
        let result = convert_add_to_copy_if_local(line);
        assert_eq!(
            result, line,
            ".tar archives should preserve ADD (auto-extraction feature)"
        );
    }

    #[test]
    fn test_convert_add_to_copy_if_local_preserves_tar_gz() {
        let line = "ADD file.tar.gz /app/";
        let result = convert_add_to_copy_if_local(line);
        assert_eq!(
            result, line,
            ".tar.gz archives should preserve ADD (auto-extraction)"
        );
    }

    #[test]
    fn test_convert_add_to_copy_if_local_preserves_tgz() {
        let line = "ADD package.tgz /opt/";
        let result = convert_add_to_copy_if_local(line);
        assert_eq!(
            result, line,
            ".tgz archives should preserve ADD (auto-extraction)"
        );
    }

    #[test]
    fn test_convert_add_to_copy_if_local_preserves_tar_bz2() {
        let line = "ADD data.tar.bz2 /data/";
        let result = convert_add_to_copy_if_local(line);
        assert_eq!(
            result, line,
            ".tar.bz2 archives should preserve ADD (auto-extraction)"
        );
    }

    #[test]
    fn test_convert_add_to_copy_if_local_preserves_tar_xz() {
        let line = "ADD compressed.tar.xz /usr/local/";
        let result = convert_add_to_copy_if_local(line);
        assert_eq!(
            result, line,
            ".tar.xz archives should preserve ADD (auto-extraction)"
        );
    }

    #[test]
    fn test_convert_add_to_copy_if_local_preserves_tar_Z() {
        let line = "ADD legacy.tar.Z /legacy/";
        let result = convert_add_to_copy_if_local(line);
        assert_eq!(
            result, line,
            ".tar.Z archives should preserve ADD (auto-extraction)"
        );
    }

    #[test]
    fn test_convert_add_to_copy_if_local_empty_line() {
        let line = "";
        let result = convert_add_to_copy_if_local(line);
        assert_eq!(result, line, "Empty line should be unchanged");
    }

    #[test]
    fn test_convert_add_to_copy_if_local_malformed_no_args() {
        let line = "ADD";
        let result = convert_add_to_copy_if_local(line);
        assert_eq!(
            result, line,
            "Malformed ADD (no arguments) should be unchanged"
        );
    }

    #[test]
    fn test_convert_add_to_copy_if_local_with_extra_spaces() {
        let line = "ADD    local_file.txt    /app/";
        let result = convert_add_to_copy_if_local(line);
        assert_eq!(
            result, "COPY    local_file.txt    /app/",
            "Should convert ADD to COPY while preserving spacing"
        );
    }

    #[test]
    fn test_convert_add_to_copy_if_local_non_docker_line() {
        let line = "# This is a comment with ADD in it";
        let result = convert_add_to_copy_if_local(line);
        // Should not convert comment lines
        assert_eq!(result, line, "Comment lines should not be processed");
    }

    // ========================================================================
    // FUNCTION 2: add_no_install_recommends() - 13 tests
    // ========================================================================

    #[test]
    fn test_add_no_install_recommends_happy_path_with_y_flag() {
        let line = "RUN apt-get install -y curl";
        let result = add_no_install_recommends(line);
        assert_eq!(
            result, "RUN apt-get install -y --no-install-recommends curl",
            "Should add --no-install-recommends after -y flag"
        );
    }

    #[test]
    fn test_add_no_install_recommends_without_y_flag() {
        let line = "RUN apt-get install python3";
        let result = add_no_install_recommends(line);
        assert_eq!(
            result, "RUN apt-get install --no-install-recommends python3",
            "Should add --no-install-recommends after install"
        );
    }

    #[test]
    fn test_add_no_install_recommends_already_present() {
        let line = "RUN apt-get install -y --no-install-recommends git";
        let result = add_no_install_recommends(line);
        assert_eq!(result, line, "Should not add flag if already present");
    }

    #[test]
    fn test_add_no_install_recommends_multiple_packages() {
        let line = "RUN apt-get install -y curl wget git";
        let result = add_no_install_recommends(line);
        assert_eq!(
            result, "RUN apt-get install -y --no-install-recommends curl wget git",
            "Should work with multiple packages"
        );
    }

    #[test]
    fn test_add_no_install_recommends_multiple_apt_get_commands() {
        let line = "RUN apt-get update && apt-get install -y curl && apt-get install -y git";
        let result = add_no_install_recommends(line);
        assert!(
            result.contains("--no-install-recommends"),
            "Should add flag to apt-get install commands"
        );
        // Both install commands should get the flag
        let flag_count = result.matches("--no-install-recommends").count();
        assert_eq!(
            flag_count, 2,
            "Should add flag to both apt-get install commands"
        );
    }

    #[test]
    fn test_add_no_install_recommends_apt_install_variant() {
        let line = "RUN apt install -y vim";
        let result = add_no_install_recommends(line);
        // Note: Current implementation only handles "apt-get install", not "apt install"
        // This test documents current behavior
        assert_eq!(result, line, "apt install (not apt-get) not yet supported");
    }

    #[test]
    fn test_add_no_install_recommends_empty_line() {
        let line = "";
        let result = add_no_install_recommends(line);
        assert_eq!(result, line, "Empty line should be unchanged");
    }

    #[test]
    fn test_add_no_install_recommends_no_apt_get() {
        let line = "RUN echo hello";
        let result = add_no_install_recommends(line);
        assert_eq!(result, line, "Non-apt-get commands should be unchanged");
    }

    #[test]
    fn test_add_no_install_recommends_apt_get_update_only() {
        let line = "RUN apt-get update";
        let result = add_no_install_recommends(line);
        assert_eq!(
            result, line,
            "apt-get update (without install) should be unchanged"
        );
    }

    #[test]
    fn test_add_no_install_recommends_with_continuation() {
        let line = "RUN apt-get install -y \\\n    curl \\\n    wget";
        let result = add_no_install_recommends(line);
        assert!(
            result.contains("--no-install-recommends"),
            "Should handle multi-line continuations"
        );
    }

    #[test]
    fn test_add_no_install_recommends_comment_line() {
        let line = "# RUN apt-get install -y curl";
        let result = add_no_install_recommends(line);
        // Should not process comments
        assert_eq!(result, line, "Comment lines should not be processed");
    }

    #[test]
    fn test_add_no_install_recommends_install_at_end() {
        let line = "RUN apt-get install";
        let result = add_no_install_recommends(line);
        assert_eq!(
            result, "RUN apt-get install --no-install-recommends ",
            "Should add flag even if no packages listed"
        );
    }

    #[test]
    fn test_add_no_install_recommends_preserves_other_flags() {
        let line = "RUN apt-get install -y --fix-missing curl";
        let result = add_no_install_recommends(line);
        assert!(
            result.contains("--fix-missing"),
            "Should preserve other flags"
        );
        assert!(
            result.contains("--no-install-recommends"),
            "Should add --no-install-recommends"
        );
    }

    // ========================================================================
    // FUNCTION 3: add_package_manager_cleanup() - 13 tests
    // ========================================================================

    #[test]
    fn test_add_package_manager_cleanup_apt_get_install() {
        let line = "RUN apt-get update && apt-get install -y curl";
        let result = add_package_manager_cleanup(line);
        assert_eq!(
            result, "RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*",
            "Should add apt cleanup after install"
        );
    }

    #[test]
    fn test_add_package_manager_cleanup_apt_install() {
        let line = "RUN apt install -y python3";
        let result = add_package_manager_cleanup(line);
        assert_eq!(
            result, "RUN apt install -y python3 && rm -rf /var/lib/apt/lists/*",
            "Should add apt cleanup for 'apt install' variant"
        );
    }

    #[test]
    fn test_add_package_manager_cleanup_apk_add() {
        let line = "RUN apk add curl";
        let result = add_package_manager_cleanup(line);
        assert_eq!(
            result, "RUN apk add curl && rm -rf /var/cache/apk/*",
            "Should add apk cleanup for Alpine"
        );
    }

    #[test]
    fn test_add_package_manager_cleanup_already_present_apt() {
        let line = "RUN apt-get install -y git && rm -rf /var/lib/apt/lists/*";
        let result = add_package_manager_cleanup(line);
        assert_eq!(result, line, "Should not add cleanup if already present");
    }

    #[test]
    fn test_add_package_manager_cleanup_already_present_apk() {
        let line = "RUN apk add vim && rm -rf /var/cache/apk/*";
        let result = add_package_manager_cleanup(line);
        assert_eq!(
            result, line,
            "Should not add cleanup if already present (apk)"
        );
    }

    #[test]
    fn test_add_package_manager_cleanup_no_package_manager() {
        let line = "RUN echo hello";
        let result = add_package_manager_cleanup(line);
        assert_eq!(
            result, line,
            "Non-package-manager commands should be unchanged"
        );
    }

    #[test]
    fn test_add_package_manager_cleanup_apt_get_update_only() {
        let line = "RUN apt-get update";
        let result = add_package_manager_cleanup(line);
        // update doesn't install packages, so no cleanup needed
        assert_eq!(result, line, "apt-get update alone should be unchanged");
    }

    #[test]
    fn test_add_package_manager_cleanup_empty_line() {
        let line = "";
        let result = add_package_manager_cleanup(line);
        assert_eq!(result, line, "Empty line should be unchanged");
    }

    #[test]
    fn test_add_package_manager_cleanup_comment_line() {
        let line = "# RUN apt-get install curl";
        let result = add_package_manager_cleanup(line);
        assert_eq!(result, line, "Comment lines should not be processed");
    }

    #[test]
    fn test_add_package_manager_cleanup_with_trailing_whitespace() {
        let line = "RUN apt-get install -y wget   ";
        let result = add_package_manager_cleanup(line);
        assert_eq!(
            result, "RUN apt-get install -y wget && rm -rf /var/lib/apt/lists/*",
            "Should trim trailing whitespace before adding cleanup"
        );
    }

    #[test]
    fn test_add_package_manager_cleanup_multiple_commands() {
        let line = "RUN apt-get update && apt-get install -y curl && echo done";
        let result = add_package_manager_cleanup(line);
        assert!(
            result.contains("&& rm -rf /var/lib/apt/lists/*"),
            "Should add cleanup even with multiple commands"
        );
    }

    #[test]
    fn test_add_package_manager_cleanup_apk_add_multiple_packages() {
        let line = "RUN apk add --no-cache curl wget git";
        let result = add_package_manager_cleanup(line);
        assert_eq!(
            result, "RUN apk add --no-cache curl wget git && rm -rf /var/cache/apk/*",
            "Should add cleanup for apk with multiple packages"
        );
    }

    #[test]
    fn test_add_package_manager_cleanup_partial_match_no_install() {
        let line = "RUN apt-get clean";
        let result = add_package_manager_cleanup(line);
        assert_eq!(
            result, line,
            "apt-get clean (not install) should be unchanged"
        );
    }

    // ========================================================================
    // FUNCTION 4: pin_base_image_version() - 13 tests
    // ========================================================================

    #[test]
    fn test_pin_base_image_version_ubuntu_untagged() {
        let line = "FROM ubuntu";
        let result = pin_base_image_version(line);
        assert_eq!(
            result, "FROM ubuntu:22.04",
            "Untagged ubuntu should be pinned to 22.04 LTS"
        );
    }

    #[test]
    fn test_pin_base_image_version_ubuntu_latest() {
        let line = "FROM ubuntu:latest";
        let result = pin_base_image_version(line);
        assert_eq!(
            result, "FROM ubuntu:22.04",
            "ubuntu:latest should be pinned to 22.04 LTS"
        );
    }

    #[test]
    fn test_pin_base_image_version_ubuntu_already_pinned() {
        let line = "FROM ubuntu:20.04";
        let result = pin_base_image_version(line);
        assert_eq!(result, line, "Already pinned ubuntu should be unchanged");
    }

    #[test]
    fn test_pin_base_image_version_debian() {
        let line = "FROM debian";
        let result = pin_base_image_version(line);
        assert_eq!(
            result, "FROM debian:12-slim",
            "Untagged debian should be pinned to 12-slim"
        );
    }

    #[test]
    fn test_pin_base_image_version_alpine() {
        let line = "FROM alpine:latest";
        let result = pin_base_image_version(line);
        assert_eq!(
            result, "FROM alpine:3.19",
            "alpine:latest should be pinned to 3.19"
        );
    }

    #[test]
    fn test_pin_base_image_version_node() {
        let line = "FROM node";
        let result = pin_base_image_version(line);
        assert_eq!(
            result, "FROM node:20-alpine",
            "Untagged node should be pinned to 20-alpine"
        );
    }

    #[test]
    fn test_pin_base_image_version_python() {
        let line = "FROM python:latest";
        let result = pin_base_image_version(line);
        assert_eq!(
            result, "FROM python:3.11-slim",
            "python:latest should be pinned to 3.11-slim"
        );
    }

    #[test]
    fn test_pin_base_image_version_with_registry_prefix() {
        let line = "FROM docker.io/ubuntu";
        let result = pin_base_image_version(line);
        assert_eq!(
            result, "FROM docker.io/ubuntu:22.04",
            "Should preserve registry prefix (docker.io/)"
        );
    }

    #[test]
    fn test_pin_base_image_version_with_as_alias() {
        let line = "FROM ubuntu AS builder";
        let result = pin_base_image_version(line);
        assert_eq!(
            result, "FROM ubuntu:22.04 AS builder",
            "Should preserve AS alias"
        );
    }

    #[test]
    fn test_pin_base_image_version_unknown_image() {
        let line = "FROM mycompany/custom-image";
        let result = pin_base_image_version(line);
        assert_eq!(result, line, "Unknown images should be unchanged");
    }

    #[test]
    fn test_pin_base_image_version_malformed_no_image() {
        let line = "FROM";
        let result = pin_base_image_version(line);
        assert_eq!(
            result, line,
            "Malformed FROM (no image) should be unchanged"
        );
    }

    #[test]
    fn test_pin_base_image_version_empty_line() {
        let line = "";
        let result = pin_base_image_version(line);
        assert_eq!(result, line, "Empty line should be unchanged");
    }

    #[test]
    fn test_pin_base_image_version_rust() {
        let line = "FROM rust:latest";
        let result = pin_base_image_version(line);
        assert_eq!(
            result, "FROM rust:1.75-alpine",
            "rust:latest should be pinned to 1.75-alpine"
        );
    }
}
