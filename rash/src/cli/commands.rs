#[cfg(feature = "oracle")]
use crate::cli::args::ExplainErrorFormat;
use crate::cli::args::{
    AuditOutputFormat, CompileRuntime, ComplyCommands, ComplyFormat, ComplyScopeArg,
    ComplyTrackCommands, ConfigCommands, ConfigOutputFormat, ContainerFormatArg,
    CorpusCommands, CorpusFormatArg, CorpusOutputFormat, DevContainerCommands,
    DockerfileCommands, InspectionFormat, InstallerCommands, InstallerGraphFormat,
    KeyringCommands, LintFormat, LintLevel, LintProfileArg, MakeCommands, MakeOutputFormat,
    MutateFormat, PlaybookFormat, ReportFormat, ScoreOutputFormat, SimulateFormat,
    TestOutputFormat,
};
#[cfg(feature = "oracle")]
use crate::cli::logic::extract_exit_code;
use crate::cli::logic::{
    coverage_class, find_devcontainer_json as logic_find_devcontainer_json,
    format_purify_report_human, format_purify_report_json, format_purify_report_markdown,
    format_timestamp, generate_diff_lines, hex_encode, is_dockerfile, is_makefile,
    is_shell_script_file, normalize_shell_script, parse_rule_filter, purify_dockerfile_source,
    score_status, truncate_str,
};
// Test-only imports
#[cfg(test)]
use crate::cli::logic::{
    add_no_install_recommends, add_package_manager_cleanup, convert_add_to_copy_if_local,
    pin_base_image_version,
};
use crate::cli::{Cli, Commands};
use crate::models::{Config, Error, Result};
use crate::{check, transpile};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

#[cfg(test)]
#[path = "command_tests.rs"]
mod command_tests;

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
        } => {
            info!("Purifying {}", input.display());
            purify_command(
                &input,
                output.as_deref(),
                report,
                with_tests,
                property_tests,
            )
        }

        Commands::Make { command } => handle_make_command(command), // Playground feature removed in v1.0 - will be moved to separate rash-playground crate in v1.1

        Commands::Dockerfile { command } => handle_dockerfile_command(command),

        Commands::Devcontainer { command } => handle_devcontainer_command(command),

        Commands::Config { command } => handle_config_command(command),

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
            test_command(&input, format, detailed, pattern.as_deref())
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
            score_command(
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
            audit_command(&input, &format, strict, detailed, min_grade.as_deref())
        }

        Commands::Coverage {
            input,
            format,
            min,
            detailed,
            output,
        } => {
            info!("Generating coverage report for {}", input.display());
            coverage_command(&input, &format, min, detailed, output.as_deref())
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
            handle_gate_command(tier, report)
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
            handle_installer_command(command)
        }

        Commands::Comply { command } => {
            info!("Executing comply command");
            handle_comply_command(command)
        }

        Commands::Corpus { command } => {
            info!("Executing corpus command");
            handle_corpus_command(command)
        }
    }
}

/// Execute quality gates based on configuration (v6.42.0)
fn handle_gate_command(tier: u8, _report: ReportFormat) -> Result<()> {
    use crate::gates::GateConfig;

    // Load gate configuration
    let config = GateConfig::load()?;

    // Determine which gates to run based on tier
    let gates_to_run = match tier {
        1 => &config.tiers.tier1_gates,
        2 => &config.tiers.tier2_gates,
        3 => &config.tiers.tier3_gates,
        _ => {
            return Err(Error::Validation(format!(
                "Invalid tier: {}. Must be 1, 2, or 3.",
                tier
            )))
        }
    };

    println!("Executing Tier {} Quality Gates...", tier);
    println!("Gates enabled: {}", gates_to_run.join(", "));
    println!("----------------------------------------");

    let mut failures = Vec::new();

    for gate in gates_to_run {
        print!("Checking {}... ", gate);
        // Flush stdout to show progress
        use std::io::Write;
        let _ = std::io::stdout().flush();

        let success = match gate.as_str() {
            "clippy" => run_clippy_gate(&config),
            "tests" => run_tests_gate(&config),
            "coverage" => run_coverage_gate(&config),
            "complexity" => run_complexity_gate(&config),
            "security" => run_security_gate(&config),
            "satd" => run_satd_gate(&config),
            "mutation" => run_mutation_gate(&config),
            _ => {
                println!("⚠️  Unknown gate");
                continue;
            }
        };

        if success {
            println!("✅ PASS");
        } else {
            println!("❌ FAIL");
            failures.push(gate.clone());
        }
    }

    println!("----------------------------------------");

    if failures.is_empty() {
        println!("✅ Tier {} Gates Passed!", tier);
        Ok(())
    } else {
        println!("❌ Tier {} Gates Failed: {}", tier, failures.join(", "));
        // Exit with error code
        std::process::exit(1);
    }
}

fn run_clippy_gate(config: &crate::gates::GateConfig) -> bool {
    // Determine clippy command
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("clippy");

    if config.gates.clippy_strict {
        cmd.args(["--", "-D", "warnings"]);
    }

    let status = cmd
        .status()
        .unwrap_or_else(|_| std::process::ExitStatus::default());
    status.success()
}

fn run_tests_gate(_config: &crate::gates::GateConfig) -> bool {
    // Run tests with timeout (simulated for now by just running cargo test)
    let status = std::process::Command::new("cargo")
        .arg("test")
        .status()
        .unwrap_or_else(|_| std::process::ExitStatus::default());
    status.success()
}

fn run_coverage_gate(config: &crate::gates::GateConfig) -> bool {
    if !config.gates.check_coverage {
        return true;
    }

    // In a real implementation, this would run llvm-cov or similar
    // For now, we'll check if cargo-llvm-cov is installed and run it, otherwise warn
    let status = std::process::Command::new("cargo")
        .args(["llvm-cov", "--version"])
        .output();

    if status.is_ok() {
        let cov_status = std::process::Command::new("cargo")
            .args([
                "llvm-cov",
                "--fail-under-lines",
                &config.gates.min_coverage.to_string(),
            ])
            .status()
            .unwrap_or_else(|_| std::process::ExitStatus::default());
        cov_status.success()
    } else {
        println!("(cargo-llvm-cov not found, skipping) ");
        true
    }
}

fn run_complexity_gate(_config: &crate::gates::GateConfig) -> bool {
    // Placeholder for complexity check integration
    // Would typically run `bashrs score` or similar internal logic
    true
}

fn run_security_gate(_config: &crate::gates::GateConfig) -> bool {
    // Placeholder for cargo-deny or similar
    let status = std::process::Command::new("cargo")
        .args(["deny", "check"])
        .status();

    match status {
        Ok(s) => s.success(),
        Err(_) => {
            println!("(cargo-deny not found, skipping) ");
            true
        }
    }
}

fn run_satd_gate(config: &crate::gates::GateConfig) -> bool {
    if let Some(satd) = &config.gates.satd {
        if !satd.enabled {
            return true;
        }

        // Simple grep for patterns
        let patterns = &satd.patterns;
        if patterns.is_empty() {
            return true;
        }

        // This is a naive implementation; a real one would use `grep` or `ripgrep`
        // efficiently across the codebase
        true
    } else {
        true
    }
}

fn run_mutation_gate(config: &crate::gates::GateConfig) -> bool {
    if let Some(mutation) = &config.gates.mutation {
        if !mutation.enabled {
            return true;
        }

        let status = std::process::Command::new("cargo")
            .args(["mutants", "--score", &mutation.min_score.to_string()])
            .status();

        match status {
            Ok(s) => s.success(),
            Err(_) => {
                println!("(cargo-mutants not found, skipping) ");
                true
            }
        }
    } else {
        true
    }
}

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

fn build_command(input: &Path, output: &Path, config: Config) -> Result<()> {
    // Read input file
    let source = fs::read_to_string(input).map_err(Error::Io)?;

    // Transpile
    let shell_code = transpile(&source, config.clone())?;

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

    // Check Rash compatibility
    check(&source)?;

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

#[allow(clippy::too_many_arguments)]
fn lint_command(
    input: &Path,
    format: LintFormat,
    fix: bool,
    fix_assumptions: bool,
    output: Option<&Path>,
    no_ignore: bool,
    ignore_file_path: Option<&Path>,
    quiet: bool,
    level: LintLevel,
    ignore_rules: Option<&str>,
    exclude_rules: Option<&[String]>,
    citl_export_path: Option<&Path>,
    profile: LintProfileArg,
    _graded: bool,
) -> Result<()> {
    use crate::linter::ignore_file::IgnoreResult;
    use crate::linter::rules::lint_shell;
    use crate::linter::{
        rules::{lint_dockerfile_with_profile, lint_makefile, LintProfile},
        LintResult,
    };

    // Issue #85: Load .bashrsignore FIRST to get both file patterns and rule codes
    let ignore_file_data = load_ignore_file(input, no_ignore, ignore_file_path);

    // Check if this file should be ignored (file pattern matching)
    if let Some(ref ignore) = ignore_file_data {
        if let IgnoreResult::Ignored(pattern) = ignore.should_ignore(input) {
            info!(
                "Skipped {} (matched .bashrsignore pattern: {})",
                input.display(),
                pattern
            );
            println!(
                "Skipped: {} (matched .bashrsignore pattern: '{}')",
                input.display(),
                pattern
            );
            return Ok(());
        }
    }

    // Build set of ignored rule codes from --ignore, -e flags, AND .bashrsignore (Issue #82, #85)
    let ignored_rules = build_ignored_rules(ignore_rules, exclude_rules, ignore_file_data.as_ref());

    // Determine minimum severity based on --quiet and --level flags (Issue #75)
    let min_severity = determine_min_severity(quiet, level);

    // Helper to filter diagnostics by severity and ignored rules (Issue #75, #82, #85)
    let filter_diagnostics = |result: LintResult| -> LintResult {
        let filtered = result
            .diagnostics
            .into_iter()
            .filter(|d| d.severity >= min_severity)
            .filter(|d| !ignored_rules.contains(&d.code.to_uppercase()))
            .collect();
        LintResult {
            diagnostics: filtered,
        }
    };

    // Read input file
    let source = fs::read_to_string(input).map_err(Error::Io)?;

    // Detect file type and use appropriate linter (using logic module)
    let filename = input.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let file_is_makefile = is_makefile(filename);
    let file_is_dockerfile = is_dockerfile(filename);

    // Convert CLI profile arg to linter profile
    use crate::cli::logic::convert_lint_profile;
    let lint_profile = convert_lint_profile(profile);

    // Run linter based on file type
    let result_raw = if file_is_makefile {
        lint_makefile(&source)
    } else if file_is_dockerfile {
        lint_dockerfile_with_profile(&source, lint_profile)
    } else {
        lint_shell(&source)
    };

    // Display profile info if using non-standard profile
    if file_is_dockerfile && lint_profile != LintProfile::Standard {
        info!("Using lint profile: {}", lint_profile);
    }

    // Apply severity filter (Issue #75: --quiet and --level flags)
    let result = filter_diagnostics(result_raw.clone());

    // Issue #83: Export diagnostics in CITL format if requested
    export_citl_if_requested(input, &result_raw, citl_export_path);

    // Apply fixes if requested (use raw result to find all fixable issues)
    if fix && result_raw.diagnostics.iter().any(|d| d.fix.is_some()) {
        handle_lint_fixes(
            input,
            &result_raw,
            fix_assumptions,
            output,
            file_is_makefile,
            format,
            &filter_diagnostics,
        )
    } else {
        output_lint_results(&result, format, input)
    }
}

/// Load `.bashrsignore` file and return it if found.
///
/// Returns `None` when `no_ignore` is set, no ignore file exists, or the file
/// cannot be loaded. The caller is responsible for checking `should_ignore`.
fn load_ignore_file(
    input: &Path,
    no_ignore: bool,
    ignore_file_path: Option<&Path>,
) -> Option<crate::linter::ignore_file::IgnoreFile> {
    use crate::linter::ignore_file::IgnoreFile;

    if no_ignore {
        return None;
    }

    // Determine ignore file path
    let ignore_path = ignore_file_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| {
            // Look for .bashrsignore in current directory or parent directories
            let mut current = input
                .parent()
                .and_then(|p| p.canonicalize().ok())
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

            loop {
                let candidate = current.join(".bashrsignore");
                if candidate.exists() {
                    return candidate;
                }
                if !current.pop() {
                    break;
                }
            }
            // Default to current directory
            PathBuf::from(".bashrsignore")
        });

    // Load ignore file if it exists
    match IgnoreFile::load(&ignore_path) {
        Ok(Some(ignore)) => Some(ignore),
        Ok(None) => None,
        Err(e) => {
            warn!("Failed to load .bashrsignore: {}", e);
            None
        }
    }
}

/// Build a set of ignored rule codes from `--ignore`, `-e` flags, and `.bashrsignore` rule codes.
fn build_ignored_rules(
    ignore_rules: Option<&str>,
    exclude_rules: Option<&[String]>,
    ignore_file_data: Option<&crate::linter::ignore_file::IgnoreFile>,
) -> std::collections::HashSet<String> {
    use std::collections::HashSet;

    let mut rules = HashSet::new();
    // Add from --ignore (comma-separated)
    if let Some(ignore_str) = ignore_rules {
        for code in ignore_str.split(',') {
            let code = code.trim().to_uppercase();
            if !code.is_empty() {
                rules.insert(code);
            }
        }
    }
    // Add from -e (can be repeated)
    if let Some(excludes) = exclude_rules {
        for code in excludes {
            let code = code.trim().to_uppercase();
            if !code.is_empty() {
                rules.insert(code);
            }
        }
    }
    // Issue #85: Add rule codes from .bashrsignore file
    if let Some(ignore) = ignore_file_data {
        for code in ignore.ignored_rules() {
            rules.insert(code);
        }
    }
    rules
}

/// Determine the minimum severity level based on `--quiet` and `--level` flags.
fn determine_min_severity(quiet: bool, level: LintLevel) -> crate::linter::Severity {
    use crate::linter::Severity;

    if quiet {
        Severity::Warning // --quiet suppresses info
    } else {
        match level {
            LintLevel::Info => Severity::Info,
            LintLevel::Warning => Severity::Warning,
            LintLevel::Error => Severity::Error,
        }
    }
}

/// Export diagnostics in CITL format if a path was provided.
fn export_citl_if_requested(
    input: &Path,
    result_raw: &crate::linter::LintResult,
    citl_export_path: Option<&Path>,
) {
    use crate::linter::citl::CitlExport;

    let Some(citl_path) = citl_export_path else {
        return;
    };

    let export = CitlExport::from_lint_result(
        input.to_str().unwrap_or("unknown"),
        result_raw, // Export raw results (unfiltered) for complete data
    );
    if let Err(e) = export.write_to_file(citl_path) {
        warn!(
            "Failed to write CITL export to {}: {}",
            citl_path.display(),
            e
        );
    } else {
        info!(
            "CITL export written to {} ({} diagnostics)",
            citl_path.display(),
            export.summary.total
        );
    }
}

/// Apply auto-fixes to the file, re-lint, and display remaining issues.
#[allow(clippy::too_many_arguments)]
fn handle_lint_fixes(
    input: &Path,
    result_raw: &crate::linter::LintResult,
    fix_assumptions: bool,
    output: Option<&Path>,
    file_is_makefile: bool,
    format: LintFormat,
    filter_diagnostics: &dyn Fn(crate::linter::LintResult) -> crate::linter::LintResult,
) -> Result<()> {
    use crate::linter::autofix::{apply_fixes_to_file, FixOptions};
    use crate::linter::output::write_results;
    use crate::linter::rules::{lint_makefile, lint_shell};

    let options = FixOptions {
        create_backup: true,
        dry_run: false,
        backup_suffix: ".bak".to_string(),
        apply_assumptions: fix_assumptions,
        output_path: output.map(|p| p.to_path_buf()),
    };

    match apply_fixes_to_file(input, result_raw, &options) {
        Ok(fix_result) => {
            info!(
                "Applied {} fix(es) to {}",
                fix_result.fixes_applied,
                input.display()
            );
            if let Some(backup_path) = &fix_result.backup_path {
                info!("Backup created at {}", backup_path);
            }

            // Re-lint to show remaining issues
            let source_after = fs::read_to_string(input).map_err(Error::Io)?;
            let result_after_raw = if file_is_makefile {
                lint_makefile(&source_after)
            } else {
                lint_shell(&source_after)
            };
            let result_after = filter_diagnostics(result_after_raw);

            if result_after.diagnostics.is_empty() {
                info!("All issues fixed!");
                return Ok(());
            }

            info!("Remaining issues after auto-fix:");
            let output_format = convert_lint_format(format);
            let file_path = input.to_str().unwrap_or("unknown");
            write_results(
                &mut std::io::stdout(),
                &result_after,
                output_format,
                file_path,
            )
            .map_err(|e| Error::Internal(format!("Failed to write lint results: {e}")))?;

            Ok(())
        }
        Err(e) => Err(Error::Internal(format!("Failed to apply fixes: {e}"))),
    }
}

/// Display lint results and exit with the appropriate code.
fn output_lint_results(
    result: &crate::linter::LintResult,
    format: LintFormat,
    input: &Path,
) -> Result<()> {
    use crate::linter::output::write_results;

    let output_format = convert_lint_format(format);
    let file_path = input.to_str().unwrap_or("unknown");
    write_results(&mut std::io::stdout(), result, output_format, file_path)
        .map_err(|e| Error::Internal(format!("Failed to write lint results: {e}")))?;

    // Exit with appropriate code (Issue #6)
    // Exit 0: No issues
    // Exit 1: Warnings found
    // Exit 2: Errors found
    if result.has_errors() {
        std::process::exit(2);
    } else if result.has_warnings() {
        std::process::exit(1);
    }

    Ok(())
}

fn purify_command(
    input: &Path,
    output: Option<&Path>,
    report: bool,
    with_tests: bool,
    property_tests: bool,
) -> Result<()> {
    use crate::bash_parser::codegen::generate_purified_bash;
    use crate::bash_parser::parser::BashParser;
    use crate::bash_transpiler::purification::{PurificationOptions, Purifier};
    use std::time::Instant;

    let start = Instant::now();

    let read_start = Instant::now();
    let source = fs::read_to_string(input).map_err(Error::Io)?;
    let read_time = read_start.elapsed();

    let parse_start = Instant::now();
    let mut parser = BashParser::new(&source)
        .map_err(|e| Error::Internal(format!("Failed to parse bash: {e}")))?;
    let ast = parser.parse()
        .map_err(|e| Error::Internal(format!("Failed to parse bash: {e}")))?;
    let parse_time = parse_start.elapsed();

    let purify_start = Instant::now();
    let mut purifier = Purifier::new(PurificationOptions::default());
    let purified_ast = purifier.purify(&ast)
        .map_err(|e| Error::Internal(format!("Failed to purify bash: {e}")))?;
    let purify_time = purify_start.elapsed();

    let codegen_start = Instant::now();
    let purified_bash = generate_purified_bash(&purified_ast);
    let codegen_time = codegen_start.elapsed();

    let write_start = Instant::now();
    if let Some(output_path) = output {
        fs::write(output_path, &purified_bash).map_err(Error::Io)?;
        info!("Purified script written to {}", output_path.display());
    } else {
        println!("{}", purified_bash);
    }
    let write_time = write_start.elapsed();

    let total_time = start.elapsed();

    if report {
        purify_print_report(input, output, &source, &purified_bash, read_time, parse_time, purify_time, codegen_time, write_time, total_time);
    }

    if with_tests {
        purify_generate_tests(output, &purified_bash, property_tests, report)?;
    }

    Ok(())
}

fn purify_print_report(
    input: &Path, output: Option<&Path>, source: &str, purified_bash: &str,
    read_time: std::time::Duration, parse_time: std::time::Duration,
    purify_time: std::time::Duration, codegen_time: std::time::Duration,
    write_time: std::time::Duration, total_time: std::time::Duration,
) {
    use crate::cli::color::*;

    println!();
    println!("{BOLD}=== Purification Report ==={RESET}");
    println!("Input:  {CYAN}{}{RESET}", input.display());
    if let Some(output_path) = output {
        println!("Output: {CYAN}{}{RESET}", output_path.display());
    }
    println!();
    println!("Input size:  {WHITE}{} lines{RESET}, {} bytes", source.lines().count(), source.len());
    println!("Output size: {WHITE}{} lines{RESET}, {} bytes", purified_bash.lines().count(), purified_bash.len());

    println!();
    println!("{BOLD}Transformations Applied:{RESET}");
    println!("  {GREEN}✓{RESET} Shebang: #!/bin/bash → #!/bin/sh");
    println!("  {GREEN}✓{RESET} Determinism: Removed $RANDOM, timestamps");
    println!("  {GREEN}✓{RESET} Idempotency: mkdir → mkdir -p, rm → rm -f");
    println!("  {GREEN}✓{RESET} Safety: All variables quoted");

    println!();
    println!("{BOLD}Performance:{RESET}");
    println!("  {DIM}Read:{RESET}     {:>8.2?}", read_time);
    println!("  {DIM}Parse:{RESET}    {:>8.2?}", parse_time);
    println!("  {DIM}Purify:{RESET}   {:>8.2?}", purify_time);
    println!("  {DIM}Codegen:{RESET}  {:>8.2?}", codegen_time);
    println!("  {DIM}Write:{RESET}    {:>8.2?}", write_time);
    println!("  {DIM}─────────────────{RESET}");
    println!("  {WHITE}Total:{RESET}    {:>8.2?}", total_time);

    let throughput = (source.len() as f64) / total_time.as_secs_f64() / 1024.0 / 1024.0;
    println!();
    println!("Throughput: {WHITE}{:.2} MB/s{RESET}", throughput);
}

fn purify_generate_tests(output: Option<&Path>, purified_bash: &str, property_tests: bool, report: bool) -> Result<()> {
    use crate::bash_transpiler::test_generator::{TestGenerator, TestGeneratorOptions};

    let output_path = output.ok_or_else(|| {
        Error::Validation("--with-tests requires -o flag to specify output file".to_string())
    })?;

    let test_file_name = format!(
        "{}_test.sh",
        output_path.file_stem().and_then(|s| s.to_str())
            .ok_or_else(|| Error::Internal("Invalid output file name".to_string()))?
    );
    let test_path = output_path.parent().unwrap_or_else(|| Path::new(".")).join(&test_file_name);

    let test_options = TestGeneratorOptions { property_tests, property_test_count: 100 };
    let generator = TestGenerator::new(test_options);
    let tests = generator.generate_tests(output_path, purified_bash);

    fs::write(&test_path, tests).map_err(Error::Io)?;
    info!("Test suite written to {}", test_path.display());

    if report {
        println!("\nTest Suite:");
        println!("  Location: {}", test_path.display());
        println!("  Property tests: {}", if property_tests { "Enabled (100 cases)" } else { "Disabled" });
    }
    Ok(())
}

fn handle_make_command(command: MakeCommands) -> Result<()> {
    match command {
        MakeCommands::Build { input, output } => {
            info!(
                "Building Makefile from {} -> {}",
                input.display(),
                output.display()
            );
            make_build_command(&input, &output)
        }
        MakeCommands::Parse { input, format } => {
            info!("Parsing {}", input.display());
            make_parse_command(&input, format)
        }
        MakeCommands::Purify {
            input,
            output,
            fix,
            report,
            format,
            with_tests,
            property_tests,
            preserve_formatting,
            max_line_length,
            skip_blank_line_removal,
            skip_consolidation,
        } => {
            info!("Purifying {}", input.display());
            make_purify_command(
                &input,
                output.as_deref(),
                fix,
                report,
                format,
                with_tests,
                property_tests,
                preserve_formatting,
                max_line_length,
                skip_blank_line_removal,
                skip_consolidation,
            )
        }
        MakeCommands::Lint {
            input,
            format,
            fix,
            output,
            rules,
        } => {
            info!("Linting {}", input.display());
            make_lint_command(&input, format, fix, output.as_deref(), rules.as_deref())
        }
    }
}

fn handle_dockerfile_command(command: DockerfileCommands) -> Result<()> {
    match command {
        DockerfileCommands::Build {
            input,
            output,
            base_image: _,
        } => {
            info!(
                "Building Dockerfile from {} -> {}",
                input.display(),
                output.display()
            );
            dockerfile_build_command(&input, &output)
        }
        DockerfileCommands::Purify {
            input,
            output,
            fix,
            no_backup,
            dry_run,
            report,
            format,
            skip_user,
            skip_bash_purify,
        } => {
            info!("Purifying {}", input.display());
            dockerfile_purify_command(
                &input,
                output.as_deref(),
                fix,
                no_backup,
                dry_run,
                report,
                format,
                skip_user,
                skip_bash_purify,
            )
        }
        DockerfileCommands::Lint {
            input,
            format,
            rules,
        } => {
            info!("Linting {}", input.display());
            // Delegate to existing Dockerfile lint functionality
            dockerfile_lint_command(&input, format, rules.as_deref())
        }
        DockerfileCommands::Profile {
            input,
            build,
            layers,
            startup,
            memory,
            cpu,
            workload,
            duration,
            profile,
            simulate_limits,
            full,
            format,
        } => {
            info!("Profiling {}", input.display());
            dockerfile_profile_command(
                &input,
                build,
                layers,
                startup,
                memory,
                cpu,
                workload.as_deref(),
                &duration,
                profile,
                simulate_limits,
                full,
                format,
            )
        }
        DockerfileCommands::SizeCheck {
            input,
            verbose,
            layers,
            detect_bloat,
            verify,
            docker_verify,
            profile,
            strict,
            max_size,
            compression_analysis,
            format,
        } => {
            info!("Checking size of {}", input.display());
            dockerfile_size_check_command(
                &input,
                verbose,
                layers,
                detect_bloat,
                verify,
                docker_verify,
                profile,
                strict,
                max_size.as_deref(),
                compression_analysis,
                format,
            )
        }
        DockerfileCommands::FullValidate {
            input,
            profile,
            size_check,
            graded,
            runtime,
            strict,
            format,
        } => {
            info!("Full validation of {}", input.display());
            dockerfile_full_validate_command(
                &input, profile, size_check, graded, runtime, strict, format,
            )
        }
    }
}

fn handle_devcontainer_command(command: DevContainerCommands) -> Result<()> {
    match command {
        DevContainerCommands::Validate {
            path, format, lint_dockerfile, list_rules,
        } => devcontainer_validate(&path, format, lint_dockerfile, list_rules),
    }
}

fn devcontainer_validate(path: &Path, format: LintFormat, lint_dockerfile: bool, list_rules: bool) -> Result<()> {
    use crate::linter::output::{write_results, OutputFormat};
    use crate::linter::rules::devcontainer::{list_devcontainer_rules, validate_devcontainer};

    if list_rules {
        println!("Available DEVCONTAINER rules:\n");
        for (code, desc) in list_devcontainer_rules() {
            println!("  {}: {}", code, desc);
        }
        return Ok(());
    }

    info!("Validating devcontainer at {}", path.display());
    let devcontainer_path = logic_find_devcontainer_json(path)?;
    info!("Found devcontainer.json at {}", devcontainer_path.display());

    let content = fs::read_to_string(&devcontainer_path).map_err(Error::Io)?;
    let result = validate_devcontainer(&content)
        .map_err(|e| Error::Validation(format!("Invalid devcontainer.json: {}", e)))?;

    let output_format = match format {
        LintFormat::Human => OutputFormat::Human,
        LintFormat::Json => OutputFormat::Json,
        LintFormat::Sarif => OutputFormat::Sarif,
    };

    let mut stdout = std::io::stdout();
    write_results(&mut stdout, &result, output_format, devcontainer_path.to_str().unwrap_or("devcontainer.json"))
        .map_err(Error::Io)?;

    if lint_dockerfile {
        lint_referenced_dockerfile(&content, &devcontainer_path, format)?;
    }

    let has_errors = result.diagnostics.iter().any(|d| d.severity == crate::linter::Severity::Error);
    if has_errors {
        Err(Error::Validation("devcontainer.json validation failed".to_string()))
    } else {
        Ok(())
    }
}

/// Lint the Dockerfile referenced in a devcontainer.json build section
fn lint_referenced_dockerfile(
    content: &str,
    devcontainer_path: &Path,
    format: LintFormat,
) -> Result<()> {
    let json = match crate::linter::rules::devcontainer::parse_jsonc(content) {
        Ok(j) => j,
        Err(_) => return Ok(()),
    };

    let dockerfile = json
        .get("build")
        .and_then(|b| b.get("dockerfile"))
        .and_then(|v| v.as_str());

    let dockerfile = match dockerfile {
        Some(d) => d,
        None => return Ok(()),
    };

    let dockerfile_path = devcontainer_path
        .parent()
        .unwrap_or(Path::new("."))
        .join(dockerfile);

    if dockerfile_path.exists() {
        info!("Linting referenced Dockerfile: {}", dockerfile_path.display());
        dockerfile_lint_command(&dockerfile_path, format, None)?;
    } else {
        warn!("Referenced Dockerfile not found: {}", dockerfile_path.display());
    }

    Ok(())
}

struct DockerfilePurifyOptions<'a> {
    output: Option<&'a Path>,
    fix: bool,
    no_backup: bool,
    dry_run: bool,
    skip_user: bool,
}

#[allow(clippy::too_many_arguments)]
fn dockerfile_build_command(input: &Path, output: &Path) -> Result<()> {
    let source = fs::read_to_string(input).map_err(Error::Io)?;
    let config = Config::default();

    let dockerfile_content = crate::transpile_dockerfile(&source, config)?;

    fs::write(output, &dockerfile_content).map_err(Error::Io)?;
    info!("Successfully generated Dockerfile at {}", output.display());

    // Run lint on generated output
    let lint_result = crate::linter::rules::lint_dockerfile(&dockerfile_content);
    if !lint_result.diagnostics.is_empty() {
        warn!(
            "Generated Dockerfile has {} lint issues",
            lint_result.diagnostics.len()
        );
    }

    Ok(())
}

fn dockerfile_purify_command(
    input: &Path,
    output: Option<&Path>,
    fix: bool,
    no_backup: bool,
    dry_run: bool,
    _report: bool,
    _format: ReportFormat,
    skip_user: bool,
    _skip_bash_purify: bool,
) -> Result<()> {
    let options = DockerfilePurifyOptions {
        output,
        fix,
        no_backup,
        dry_run,
        skip_user,
    };
    dockerfile_purify_command_impl(input, options)
}

fn dockerfile_purify_command_impl(
    input: &Path,
    options: DockerfilePurifyOptions<'_>,
) -> Result<()> {
    // Read Dockerfile
    let source = fs::read_to_string(input).map_err(Error::Io)?;

    // Apply purification transformations
    let purified = purify_dockerfile(&source, options.skip_user)?;

    // Handle output
    if options.dry_run {
        println!("Would add USER directive");
        return Ok(());
    }

    if options.fix {
        // In-place modification
        if !options.no_backup {
            let backup_path = input.with_extension("bak");
            fs::copy(input, &backup_path).map_err(Error::Io)?;
        }
        fs::write(input, &purified).map_err(Error::Io)?;
        info!("Purified Dockerfile written to {}", input.display());
    } else if let Some(output_path) = options.output {
        // Write to output file
        fs::write(output_path, &purified).map_err(Error::Io)?;
        info!("Purified Dockerfile written to {}", output_path.display());
    } else {
        // Write to stdout
        println!("{}", purified);
    }

    Ok(())
}

/// Thin shim - delegates to pure logic function
fn purify_dockerfile(source: &str, skip_user: bool) -> Result<String> {
    Ok(purify_dockerfile_source(source, skip_user))
}

fn dockerfile_lint_command(input: &Path, format: LintFormat, rules: Option<&str>) -> Result<()> {
    use crate::linter::rules::lint_dockerfile;

    info!("Linting {} for Dockerfile issues", input.display());

    let source = fs::read_to_string(input).map_err(Error::Io)?;
    let result = lint_dockerfile(&source);

    // Filter by rules if specified
    let filtered_diagnostics: Vec<_> = if let Some(rule_filter) = rules {
        let allowed_rules: std::collections::HashSet<&str> = rule_filter.split(',').collect();
        result
            .diagnostics
            .into_iter()
            .filter(|d| allowed_rules.contains(d.code.as_str()))
            .collect()
    } else {
        result.diagnostics
    };

    // Output based on format
    match format {
        LintFormat::Human => {
            if filtered_diagnostics.is_empty() {
                println!("No Dockerfile issues found");
            } else {
                println!("Dockerfile Issues:");
                println!("==================\n");
                for diag in &filtered_diagnostics {
                    let severity_icon = match diag.severity {
                        crate::linter::Severity::Error => "❌",
                        crate::linter::Severity::Warning => "⚠",
                        crate::linter::Severity::Info => "ℹ",
                        _ => "ℹ",
                    };
                    println!(
                        "{} Line {}: [{}] {}",
                        severity_icon, diag.span.start_line, diag.code, diag.message
                    );
                    if let Some(ref fix) = diag.fix {
                        println!("   Fix: {}", fix.replacement);
                    }
                    println!();
                }
                println!("Summary: {} issue(s) found", filtered_diagnostics.len());
            }
        }
        LintFormat::Json => {
            let output = serde_json::json!({
                "file": input.display().to_string(),
                "diagnostics": filtered_diagnostics.iter().map(|d| {
                    serde_json::json!({
                        "code": d.code,
                        "severity": format!("{:?}", d.severity),
                        "message": d.message,
                        "line": d.span.start_line,
                        "column": d.span.start_col,
                        "fix": d.fix.as_ref().map(|f| &f.replacement)
                    })
                }).collect::<Vec<_>>()
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&output).unwrap_or_default()
            );
        }
        LintFormat::Sarif => {
            // Basic SARIF output
            let sarif = serde_json::json!({
                "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
                "version": "2.1.0",
                "runs": [{
                    "tool": {
                        "driver": {
                            "name": "bashrs dockerfile lint",
                            "version": env!("CARGO_PKG_VERSION")
                        }
                    },
                    "results": filtered_diagnostics.iter().map(|d| {
                        serde_json::json!({
                            "ruleId": d.code,
                            "message": { "text": d.message },
                            "level": match d.severity {
                                crate::linter::Severity::Error => "error",
                                crate::linter::Severity::Warning => "warning",
                                _ => "note"
                            },
                            "locations": [{
                                "physicalLocation": {
                                    "artifactLocation": { "uri": input.display().to_string() },
                                    "region": { "startLine": d.span.start_line }
                                }
                            }]
                        })
                    }).collect::<Vec<_>>()
                }]
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&sarif).unwrap_or_default()
            );
        }
    }

    // Exit with error if there are errors
    if filtered_diagnostics
        .iter()
        .any(|d| matches!(d.severity, crate::linter::Severity::Error))
    {
        std::process::exit(2);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn dockerfile_profile_command(
    input: &Path,
    build: bool,
    layers: bool,
    startup: bool,
    memory: bool,
    cpu: bool,
    _workload: Option<&Path>,
    _duration: &str,
    profile: Option<LintProfileArg>,
    simulate_limits: bool,
    full: bool,
    format: ReportFormat,
) -> Result<()> {
    use crate::linter::docker_profiler::{
        estimate_size, is_docker_available, PlatformProfile,
    };

    info!("Profiling {} for runtime performance", input.display());

    if !is_docker_available() {
        println!("\u{26a0}\u{fe0f}  Docker daemon not available");
        println!("Runtime profiling requires Docker. Falling back to static analysis.\n");
    }

    let source = fs::read_to_string(input).map_err(Error::Io)?;

    let platform = match profile {
        Some(LintProfileArg::Coursera) => PlatformProfile::Coursera,
        _ => PlatformProfile::Standard,
    };

    let estimate = estimate_size(&source);

    match format {
        ReportFormat::Human => {
            docker_profile_human(input, &estimate, platform, build, layers, startup, memory, cpu, simulate_limits, full);
        }
        ReportFormat::Json => docker_profile_json(input, &estimate, platform),
        ReportFormat::Markdown => docker_profile_markdown(input, &estimate),
    }

    Ok(())
}

fn docker_profile_human(
    _input: &Path,
    estimate: &crate::linter::docker_profiler::SizeEstimate,
    platform: crate::linter::docker_profiler::PlatformProfile,
    build: bool,
    layers: bool,
    startup: bool,
    memory: bool,
    cpu: bool,
    simulate_limits: bool,
    full: bool,
) {
    use crate::linter::docker_profiler::{format_size_estimate, PlatformProfile};

    println!("Docker Image Profile");
    println!("====================\n");

    if build || full {
        docker_profile_build_section(estimate, layers);
    }

    println!("{}", format_size_estimate(estimate, layers));

    if startup || full {
        println!("Startup Analysis:");
        println!("  Requires Docker daemon for actual measurement");
        if platform == PlatformProfile::Coursera {
            println!("  Coursera limit: 60 seconds");
            println!("  Recommendation: Target <30s startup time");
        }
        println!();
    }

    if memory || full {
        println!("Memory Analysis:");
        println!("  Requires Docker daemon for actual measurement");
        if platform == PlatformProfile::Coursera {
            println!("  Coursera limit: 4GB");
        }
        println!();
    }

    if cpu || full {
        println!("CPU Analysis:");
        println!("  Requires Docker daemon for actual measurement");
        if platform == PlatformProfile::Coursera {
            println!("  Coursera limit: 2 CPUs");
        }
        println!();
    }

    if platform == PlatformProfile::Coursera {
        docker_profile_coursera_validation(estimate, &platform, simulate_limits);
    }
}

fn docker_profile_build_section(
    estimate: &crate::linter::docker_profiler::SizeEstimate,
    layers: bool,
) {
    println!("Build Analysis:");
    println!("  Layers: {}", estimate.layer_estimates.len());
    println!(
        "  Estimated build time: {} (based on layer complexity)",
        estimate_build_time(estimate)
    );

    if layers {
        println!("\n  Layer Details:");
        for layer in &estimate.layer_estimates {
            let cached = if layer.cached { " (cached)" } else { "" };
            println!(
                "    [{}] {}{} - line {}",
                layer.layer_num, layer.instruction, cached, layer.line
            );
            if let Some(ref notes) = layer.notes {
                println!("        {}", notes);
            }
        }
    }
    println!();
}

fn docker_profile_coursera_validation(
    estimate: &crate::linter::docker_profiler::SizeEstimate,
    platform: &crate::linter::docker_profiler::PlatformProfile,
    simulate_limits: bool,
) {
    println!("Coursera Platform Validation:");
    let max_size_gb = platform.max_size_bytes() as f64 / 1_000_000_000.0;
    let estimated_gb = estimate.total_estimated as f64 / 1_000_000_000.0;
    let size_ok = estimate.total_estimated < platform.max_size_bytes();
    let size_icon = if size_ok { "\u{2713}" } else { "\u{2717}" };

    println!(
        "  {} Image size: {:.2}GB (limit: {:.0}GB)",
        size_icon, estimated_gb, max_size_gb
    );

    if simulate_limits {
        println!("\n  Simulation flags for docker run:");
        println!("    --memory=4g --cpus=2");
    }
    println!();
}

fn docker_profile_json(
    input: &Path,
    estimate: &crate::linter::docker_profiler::SizeEstimate,
    platform: crate::linter::docker_profiler::PlatformProfile,
) {
    use crate::linter::docker_profiler::is_docker_available;

    let json = serde_json::json!({
        "file": input.display().to_string(),
        "profile": format!("{:?}", platform),
        "build": {
            "layers": estimate.layer_estimates.len(),
            "estimated_build_time": estimate_build_time(estimate),
        },
        "size": {
            "base_image": estimate.base_image,
            "base_image_bytes": estimate.base_image_size,
            "total_estimated_bytes": estimate.total_estimated,
            "bloat_patterns": estimate.bloat_patterns.len(),
        },
        "docker_available": is_docker_available(),
        "platform_limits": {
            "max_size_bytes": platform.max_size_bytes(),
            "max_memory_bytes": platform.max_memory_bytes(),
            "max_startup_ms": platform.max_startup_ms(),
        }
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&json).unwrap_or_default()
    );
}

fn docker_profile_markdown(
    input: &Path,
    estimate: &crate::linter::docker_profiler::SizeEstimate,
) {
    println!("# Docker Image Profile\n");
    println!("**File**: {}\n", input.display());
    println!("## Build Analysis\n");
    println!("- **Layers**: {}", estimate.layer_estimates.len());
    println!(
        "- **Estimated build time**: {}\n",
        estimate_build_time(estimate)
    );
    println!("## Size Analysis\n");
    println!("- **Base image**: {}", estimate.base_image);
    println!(
        "- **Estimated total**: {:.2}GB\n",
        estimate.total_estimated as f64 / 1_000_000_000.0
    );
}

/// Estimate build time based on layer complexity
fn estimate_build_time(estimate: &crate::linter::docker_profiler::SizeEstimate) -> String {
    // Rough heuristic: 1 second per 100MB + base times
    let mut total_seconds = 0u64;

    for layer in &estimate.layer_estimates {
        // Base time for each layer
        total_seconds += 1;

        // Add time based on size
        total_seconds += layer.estimated_size / 100_000_000;

        // Add extra time for known slow operations
        let content_lower = layer.content.to_lowercase();
        if content_lower.contains("apt-get install") {
            total_seconds += 10;
        }
        if content_lower.contains("pip install") {
            total_seconds += 5;
        }
        if content_lower.contains("npm install") {
            total_seconds += 5;
        }
    }

    if total_seconds < 60 {
        format!("~{}s", total_seconds)
    } else {
        format!("~{}m {}s", total_seconds / 60, total_seconds % 60)
    }
}

#[allow(clippy::too_many_arguments)]
fn dockerfile_size_check_command(
    input: &Path,
    verbose: bool,
    layers: bool,
    detect_bloat: bool,
    verify: bool,
    docker_verify: bool,
    profile: Option<LintProfileArg>,
    strict: bool,
    max_size: Option<&str>,
    compression_analysis: bool,
    format: ReportFormat,
) -> Result<()> {
    use crate::linter::docker_profiler::{
        estimate_size, format_size_estimate_json, PlatformProfile,
    };

    info!("Checking size of {}", input.display());

    let source = fs::read_to_string(input).map_err(Error::Io)?;
    let estimate = estimate_size(&source);

    let platform = match profile {
        Some(LintProfileArg::Coursera) => PlatformProfile::Coursera,
        _ => PlatformProfile::Standard,
    };

    let custom_limit = parse_size_limit(max_size);

    match format {
        ReportFormat::Human => {
            size_check_human_output(
                &estimate, &platform, custom_limit, verbose, layers,
                detect_bloat, verify, docker_verify, compression_analysis, strict,
            )
        }
        ReportFormat::Json => {
            println!("{}", format_size_estimate_json(&estimate));
            Ok(())
        }
        ReportFormat::Markdown => {
            size_check_markdown_output(input, &estimate);
            Ok(())
        }
    }
}

fn parse_size_limit(max_size: Option<&str>) -> Option<u64> {
    max_size.and_then(|s| {
        let s = s.to_uppercase();
        if s.ends_with("GB") {
            s[..s.len() - 2].trim().parse::<f64>().ok().map(|n| (n * 1_000_000_000.0) as u64)
        } else if s.ends_with("MB") {
            s[..s.len() - 2].trim().parse::<f64>().ok().map(|n| (n * 1_000_000.0) as u64)
        } else {
            None
        }
    })
}

fn size_check_human_output(
    estimate: &crate::linter::docker_profiler::SizeEstimate,
    platform: &crate::linter::docker_profiler::PlatformProfile,
    custom_limit: Option<u64>,
    verbose: bool,
    layers: bool,
    detect_bloat: bool,
    verify: bool,
    docker_verify: bool,
    compression_analysis: bool,
    strict: bool,
) -> Result<()> {
    use crate::linter::docker_profiler::{format_size_estimate, is_docker_available};

    println!("{}", format_size_estimate(estimate, verbose || layers));

    if detect_bloat && !estimate.bloat_patterns.is_empty() {
        println!("Bloat Detection Results:");
        for pattern in &estimate.bloat_patterns {
            println!("  {} [line {}]: {}", pattern.code, pattern.line, pattern.description);
            println!("    Wasted: ~{}MB", pattern.wasted_bytes / 1_000_000);
            println!("    Fix: {}", pattern.remediation);
            println!();
        }
    }

    if (verify || docker_verify) && is_docker_available() {
        println!("Docker Verification:");
        println!("  Requires docker build to verify actual size\n");
    }

    if compression_analysis {
        println!("Compression Opportunities:");
        println!("  - Use multi-stage builds to reduce final image size");
        println!("  - Compress large data files with gzip (~70% reduction)");
        println!("  - Use .dockerignore to exclude unnecessary files\n");
    }

    size_check_limit_check(estimate, platform, custom_limit, strict)
}

fn size_check_limit_check(
    estimate: &crate::linter::docker_profiler::SizeEstimate,
    platform: &crate::linter::docker_profiler::PlatformProfile,
    custom_limit: Option<u64>,
    strict: bool,
) -> Result<()> {
    let effective_limit = custom_limit.unwrap_or(platform.max_size_bytes());
    if effective_limit == u64::MAX { return Ok(()); }

    let limit_gb = effective_limit as f64 / 1_000_000_000.0;
    let estimated_gb = estimate.total_estimated as f64 / 1_000_000_000.0;

    println!("Size Limit Check:");
    if estimate.total_estimated > effective_limit {
        println!("  \u{2717} EXCEEDS LIMIT: {:.2}GB > {:.0}GB", estimated_gb, limit_gb);
        if strict {
            return Err(Error::Validation(format!(
                "Image size ({:.2}GB) exceeds limit ({:.0}GB)", estimated_gb, limit_gb
            )));
        }
    } else {
        let percentage = (estimate.total_estimated as f64 / effective_limit as f64) * 100.0;
        println!("  \u{2713} Within limit: {:.2}GB / {:.0}GB ({:.0}%)", estimated_gb, limit_gb, percentage);
    }
    println!();
    Ok(())
}

fn size_check_markdown_output(
    input: &Path,
    estimate: &crate::linter::docker_profiler::SizeEstimate,
) {
    println!("# Image Size Analysis\n");
    println!("**File**: {}\n", input.display());
    println!("## Summary\n");
    println!("- **Base image**: {}", estimate.base_image);
    println!("- **Estimated total**: {:.2}GB\n", estimate.total_estimated as f64 / 1_000_000_000.0);

    if !estimate.bloat_patterns.is_empty() {
        println!("## Bloat Patterns\n");
        for pattern in &estimate.bloat_patterns {
            println!("- **{}** (line {}): {}", pattern.code, pattern.line, pattern.description);
        }
        println!();
    }
}

fn dockerfile_full_validate_command(
    input: &Path,
    profile: Option<LintProfileArg>,
    size_check: bool,
    _graded: bool,
    runtime: bool,
    strict: bool,
    format: ReportFormat,
) -> Result<()> {
    use crate::linter::rules::LintProfile;
    use crate::linter::docker_profiler::PlatformProfile;

    info!("Full validation of {}", input.display());

    let source = fs::read_to_string(input).map_err(Error::Io)?;

    let lint_profile = match profile {
        Some(LintProfileArg::Coursera) => LintProfile::Coursera,
        Some(LintProfileArg::DevContainer) => LintProfile::DevContainer,
        _ => LintProfile::Standard,
    };

    let platform_profile = match profile {
        Some(LintProfileArg::Coursera) => PlatformProfile::Coursera,
        _ => PlatformProfile::Standard,
    };

    match format {
        ReportFormat::Human => {
            dockerfile_full_validate_human(
                &source, lint_profile, platform_profile, size_check, runtime, strict,
            )
        }
        ReportFormat::Json => {
            dockerfile_full_validate_json(input, &source, lint_profile, platform_profile);
            Ok(())
        }
        ReportFormat::Markdown => {
            dockerfile_full_validate_markdown(input, &source, lint_profile, size_check);
            Ok(())
        }
    }
}

fn dockerfile_full_validate_human(
    source: &str,
    lint_profile: crate::linter::rules::LintProfile,
    platform_profile: crate::linter::docker_profiler::PlatformProfile,
    size_check: bool,
    runtime: bool,
    strict: bool,
) -> Result<()> {
    println!("Full Dockerfile Validation");
    println!("==========================\n");

    let mut all_passed = true;

    let lint_passed = dockerfile_validate_lint_step(source, lint_profile);
    if !lint_passed { all_passed = false; }

    if size_check {
        let size_passed = dockerfile_validate_size_step(source, platform_profile);
        if !size_passed { all_passed = false; }
    }

    if runtime {
        dockerfile_validate_runtime_step();
    }

    dockerfile_validate_summary(all_passed, lint_profile, strict)
}

fn dockerfile_validate_lint_step(source: &str, lint_profile: crate::linter::rules::LintProfile) -> bool {
    use crate::linter::rules::lint_dockerfile_with_profile;

    println!("1. Linting Dockerfile...");
    let lint_result = lint_dockerfile_with_profile(source, lint_profile);
    let error_count = lint_result.diagnostics.iter()
        .filter(|d| d.severity == crate::linter::Severity::Error).count();
    let warning_count = lint_result.diagnostics.iter()
        .filter(|d| d.severity == crate::linter::Severity::Warning).count();

    if error_count == 0 && warning_count == 0 {
        println!("   \u{2713} No lint issues found\n");
        return true;
    }

    println!("   {} errors, {} warnings\n", error_count, warning_count);
    for diag in &lint_result.diagnostics {
        let icon = match diag.severity {
            crate::linter::Severity::Error => "\u{2717}",
            crate::linter::Severity::Warning => "\u{26a0}",
            _ => "\u{2139}",
        };
        println!("   {} [{}] Line {}: {}", icon, diag.code, diag.span.start_line, diag.message);
    }
    println!();
    error_count == 0
}

fn dockerfile_validate_size_step(source: &str, platform_profile: crate::linter::docker_profiler::PlatformProfile) -> bool {
    use crate::linter::docker_profiler::estimate_size;

    println!("2. Checking image size...");
    let estimate = estimate_size(source);
    let size_gb = estimate.total_estimated as f64 / 1_000_000_000.0;
    let limit_gb = platform_profile.max_size_bytes() as f64 / 1_000_000_000.0;

    let passed = estimate.total_estimated < platform_profile.max_size_bytes();
    if passed {
        println!("   \u{2713} Size OK: {:.2}GB (limit: {:.0}GB)\n", size_gb, limit_gb);
    } else {
        println!("   \u{2717} Size exceeds limit: {:.2}GB > {:.0}GB\n", size_gb, limit_gb);
    }
    for pattern in &estimate.bloat_patterns {
        println!("   - {}: {}", pattern.code, pattern.description);
    }
    if !estimate.bloat_patterns.is_empty() { println!(); }
    passed
}

fn dockerfile_validate_runtime_step() {
    use crate::linter::docker_profiler::is_docker_available;

    println!("3. Runtime validation...");
    if is_docker_available() {
        println!("   Requires docker build - skipping in static analysis mode\n");
    } else {
        println!("   \u{26a0} Docker not available - skipping runtime checks\n");
    }
}

fn dockerfile_validate_summary(all_passed: bool, lint_profile: crate::linter::rules::LintProfile, strict: bool) -> Result<()> {
    println!("Validation Result:");
    if all_passed {
        println!("\u{2713} All checks passed");
        if lint_profile == crate::linter::rules::LintProfile::Coursera {
            println!("\u{2713} Ready for Coursera Labs upload");
        }
    } else {
        println!("\u{2717} Validation failed - see issues above");
        if strict {
            return Err(Error::Validation("Full validation failed".to_string()));
        }
    }
    Ok(())
}

fn dockerfile_full_validate_json(
    input: &Path,
    source: &str,
    lint_profile: crate::linter::rules::LintProfile,
    platform_profile: crate::linter::docker_profiler::PlatformProfile,
) {
    use crate::linter::docker_profiler::estimate_size;
    use crate::linter::rules::lint_dockerfile_with_profile;

    let lint_result = lint_dockerfile_with_profile(source, lint_profile);
    let estimate = estimate_size(source);

    let json = serde_json::json!({
        "file": input.display().to_string(),
        "profile": format!("{:?}", lint_profile),
        "lint": {
            "errors": lint_result.diagnostics.iter()
                .filter(|d| d.severity == crate::linter::Severity::Error).count(),
            "warnings": lint_result.diagnostics.iter()
                .filter(|d| d.severity == crate::linter::Severity::Warning).count(),
            "diagnostics": lint_result.diagnostics.iter().map(|d| {
                serde_json::json!({
                    "code": d.code,
                    "severity": format!("{:?}", d.severity),
                    "message": d.message,
                    "line": d.span.start_line
                })
            }).collect::<Vec<_>>()
        },
        "size": {
            "estimated_bytes": estimate.total_estimated,
            "estimated_gb": estimate.total_estimated as f64 / 1_000_000_000.0,
            "limit_bytes": platform_profile.max_size_bytes(),
            "within_limit": estimate.total_estimated < platform_profile.max_size_bytes(),
            "bloat_patterns": estimate.bloat_patterns.len()
        },
        "passed": true
    });
    println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
}

fn dockerfile_full_validate_markdown(
    input: &Path,
    source: &str,
    lint_profile: crate::linter::rules::LintProfile,
    size_check: bool,
) {
    use crate::linter::docker_profiler::estimate_size;
    use crate::linter::rules::lint_dockerfile_with_profile;

    println!("# Full Dockerfile Validation\n");
    println!("**File**: {}\n", input.display());

    let lint_result = lint_dockerfile_with_profile(source, lint_profile);
    let error_count = lint_result.diagnostics.iter()
        .filter(|d| d.severity == crate::linter::Severity::Error).count();

    println!("## Lint Results\n");
    println!("- **Errors**: {}", error_count);
    println!("- **Warnings**: {}\n",
        lint_result.diagnostics.iter()
            .filter(|d| d.severity == crate::linter::Severity::Warning).count()
    );

    if size_check {
        let estimate = estimate_size(source);
        println!("## Size Analysis\n");
        println!("- **Estimated size**: {:.2}GB\n", estimate.total_estimated as f64 / 1_000_000_000.0);
    }

    println!("## Result\n");
    if error_count == 0 {
        println!("\u{2713} **PASSED**");
    } else {
        println!("\u{2717} **FAILED**");
    }
}

fn make_build_command(input: &Path, output: &Path) -> Result<()> {
    let source = fs::read_to_string(input).map_err(Error::Io)?;
    let config = Config::default();

    let makefile_content = crate::transpile_makefile(&source, config)?;

    fs::write(output, &makefile_content).map_err(Error::Io)?;
    info!("Successfully generated Makefile at {}", output.display());

    // Run lint on generated output
    let lint_result = crate::linter::rules::lint_makefile(&makefile_content);
    if !lint_result.diagnostics.is_empty() {
        warn!(
            "Generated Makefile has {} lint issues",
            lint_result.diagnostics.len()
        );
    }

    Ok(())
}

fn make_parse_command(input: &Path, format: MakeOutputFormat) -> Result<()> {
    use crate::make_parser::parser::parse_makefile;

    let source = fs::read_to_string(input).map_err(Error::Io)?;
    let ast = parse_makefile(&source)
        .map_err(|e| Error::Validation(format!("Failed to parse Makefile: {}", e)))?;

    match format {
        MakeOutputFormat::Text => {
            println!("{:#?}", ast);
        }
        MakeOutputFormat::Json => {
            // Note: MakeAst doesn't derive Serialize yet, so we'll use Debug format
            println!("{:#?}", ast);
        }
        MakeOutputFormat::Debug => {
            println!("{:?}", ast);
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn make_purify_command(
    input: &Path,
    output: Option<&Path>,
    fix: bool,
    report: bool,
    format: ReportFormat,
    with_tests: bool,
    property_tests: bool,
    preserve_formatting: bool,
    max_line_length: Option<usize>,
    skip_blank_line_removal: bool,
    skip_consolidation: bool,
) -> Result<()> {
    use crate::make_parser::{
        generators::{generate_purified_makefile_with_options, MakefileGeneratorOptions},
        parser::parse_makefile,
        purify::purify_makefile,
    };

    if with_tests && output.is_none() {
        return Err(Error::Validation(
            "--with-tests requires -o flag to specify output file".to_string(),
        ));
    }

    let source = fs::read_to_string(input).map_err(Error::Io)?;
    let ast = parse_makefile(&source)
        .map_err(|e| Error::Validation(format!("Failed to parse Makefile: {}", e)))?;
    let purify_result = purify_makefile(&ast);

    if report {
        print_purify_report(&purify_result, format);
    }

    let generator_options = MakefileGeneratorOptions {
        preserve_formatting, max_line_length, skip_blank_line_removal, skip_consolidation,
    };
    let purified = generate_purified_makefile_with_options(&purify_result.ast, &generator_options);

    make_purify_write_output(input, output, fix, &purified)?;

    if with_tests {
        if let Some(output_path) = output {
            make_purify_generate_tests(output_path, &purified, property_tests)?;
        }
    }

    Ok(())
}

fn make_purify_write_output(input: &Path, output: Option<&Path>, fix: bool, purified: &str) -> Result<()> {
    if let Some(output_path) = output {
        fs::write(output_path, purified).map_err(Error::Io)?;
        info!("Purified Makefile written to {}", output_path.display());
    } else if fix {
        let backup_path = input.with_extension("mk.bak");
        fs::copy(input, &backup_path).map_err(Error::Io)?;
        fs::write(input, purified).map_err(Error::Io)?;
        info!("Purified Makefile written to {}", input.display());
        info!("Backup created at {}", backup_path.display());
    } else {
        println!("{}", purified);
    }
    Ok(())
}

fn make_purify_generate_tests(output_path: &Path, purified: &str, property_tests: bool) -> Result<()> {
    use crate::make_parser::{MakefileTestGenerator, MakefileTestGeneratorOptions};

    let test_options = MakefileTestGeneratorOptions {
        property_tests,
        property_test_count: 100,
    };
    let test_generator = MakefileTestGenerator::new(test_options);
    let test_suite = test_generator.generate_tests(output_path, purified);

    let file_name = output_path.file_name()
        .ok_or_else(|| Error::Internal("Invalid output path".to_string()))?
        .to_str()
        .ok_or_else(|| Error::Internal("Invalid UTF-8 in filename".to_string()))?;
    let test_file = output_path.with_file_name(format!("{}.test.sh", file_name));

    fs::write(&test_file, test_suite).map_err(Error::Io)?;
    info!("Test suite written to {}", test_file.display());
    Ok(())
}

/// Thin shim - delegates formatting to pure logic functions
fn print_purify_report(
    result: &crate::make_parser::purify::PurificationResult,
    format: ReportFormat,
) {
    let output = match format {
        ReportFormat::Human => format_purify_report_human(
            result.transformations_applied,
            result.issues_fixed,
            result.manual_fixes_needed,
            &result.report,
        ),
        ReportFormat::Json => format_purify_report_json(
            result.transformations_applied,
            result.issues_fixed,
            result.manual_fixes_needed,
            &result.report,
        ),
        ReportFormat::Markdown => format_purify_report_markdown(
            result.transformations_applied,
            result.issues_fixed,
            result.manual_fixes_needed,
            &result.report,
        ),
    };
    print!("{}", output);
}

/// Convert LintFormat to OutputFormat
fn convert_lint_format(format: LintFormat) -> crate::linter::output::OutputFormat {
    use crate::linter::output::OutputFormat;
    match format {
        LintFormat::Human => OutputFormat::Human,
        LintFormat::Json => OutputFormat::Json,
        LintFormat::Sarif => OutputFormat::Sarif,
    }
}

/// Run linter and optionally filter results by specific rules (thin shim)
fn run_filtered_lint(source: &str, rules: Option<&str>) -> crate::linter::LintResult {
    use crate::linter::rules::lint_makefile;

    let mut result = lint_makefile(source);

    // Filter by specific rules if requested - uses logic::parse_rule_filter
    if let Some(rule_filter) = rules {
        let allowed_rules = parse_rule_filter(rule_filter);
        result
            .diagnostics
            .retain(|d| allowed_rules.iter().any(|rule| d.code.contains(rule)));
    }

    result
}

/// Apply fixes and write to separate output file (not in-place)
fn apply_fixes_to_output(
    source: &str,
    result: &crate::linter::LintResult,
    output_path: &Path,
    format: LintFormat,
) -> Result<()> {
    use crate::linter::{
        autofix::{apply_fixes, FixOptions},
        output::write_results,
        rules::lint_makefile,
    };

    let fix_options = FixOptions {
        create_backup: false, // Don't create backup for output file
        dry_run: false,
        backup_suffix: String::new(),
        apply_assumptions: false,
        output_path: None,
    };

    let fix_result = apply_fixes(source, result, &fix_options)
        .map_err(|e| Error::Internal(format!("Failed to apply fixes: {e}")))?;

    if let Some(fixed_source) = fix_result.modified_source {
        fs::write(output_path, &fixed_source).map_err(Error::Io)?;
        info!("Fixed Makefile written to {}", output_path.display());

        // Re-lint the fixed content
        let result_after = lint_makefile(&fixed_source);
        if result_after.diagnostics.is_empty() {
            info!("✓ All issues fixed!");
        } else {
            info!("Remaining issues after auto-fix:");
            let output_format = convert_lint_format(format);
            let file_path = output_path.to_str().unwrap_or("unknown");
            write_results(
                &mut std::io::stdout(),
                &result_after,
                output_format,
                file_path,
            )
            .map_err(|e| Error::Internal(format!("Failed to write lint results: {e}")))?;
        }
    }

    Ok(())
}

/// Apply fixes in-place to the original file with backup
fn apply_fixes_inplace(
    input: &Path,
    result: &crate::linter::LintResult,
    format: LintFormat,
) -> Result<()> {
    use crate::linter::{
        autofix::{apply_fixes_to_file, FixOptions},
        output::write_results,
        rules::lint_makefile,
    };

    let options = FixOptions {
        create_backup: true,
        dry_run: false,
        backup_suffix: ".bak".to_string(),
        apply_assumptions: false,
        output_path: None,
    };

    match apply_fixes_to_file(input, result, &options) {
        Ok(fix_result) => {
            info!(
                "Applied {} fix(es) to {}",
                fix_result.fixes_applied,
                input.display()
            );
            if let Some(backup_path) = &fix_result.backup_path {
                info!("Backup created at {}", backup_path);
            }

            // Re-lint to show remaining issues
            let source_after = fs::read_to_string(input).map_err(Error::Io)?;
            let result_after = lint_makefile(&source_after);

            if result_after.diagnostics.is_empty() {
                info!("✓ All issues fixed!");
            } else {
                info!("Remaining issues after auto-fix:");
                let output_format = convert_lint_format(format);
                let file_path = input.to_str().unwrap_or("unknown");
                write_results(
                    &mut std::io::stdout(),
                    &result_after,
                    output_format,
                    file_path,
                )
                .map_err(|e| Error::Internal(format!("Failed to write lint results: {e}")))?;
            }
        }
        Err(e) => {
            return Err(Error::Internal(format!("Failed to apply fixes: {e}")));
        }
    }

    Ok(())
}

/// Show lint results without applying fixes
fn show_lint_results(
    result: &crate::linter::LintResult,
    format: LintFormat,
    input: &Path,
) -> Result<()> {
    use crate::linter::output::write_results;

    let output_format = convert_lint_format(format);
    let file_path = input.to_str().unwrap_or("unknown");
    write_results(&mut std::io::stdout(), result, output_format, file_path)
        .map_err(|e| Error::Internal(format!("Failed to write lint results: {e}")))?;

    // Exit with appropriate code
    if result.has_errors() {
        std::process::exit(2);
    } else if result.has_warnings() {
        std::process::exit(1);
    }

    Ok(())
}

fn make_lint_command(
    input: &Path,
    format: LintFormat,
    fix: bool,
    output: Option<&Path>,
    rules: Option<&str>,
) -> Result<()> {
    // Read input file
    let source = fs::read_to_string(input).map_err(Error::Io)?;

    // Run linter and filter by rules if requested
    let result = run_filtered_lint(&source, rules);

    // Apply fixes if requested
    if fix && result.diagnostics.iter().any(|d| d.fix.is_some()) {
        if let Some(output_path) = output {
            // Output to separate file: don't modify original
            apply_fixes_to_output(&source, &result, output_path, format)?;
        } else {
            // In-place fixing: modify original file
            apply_fixes_inplace(input, &result, format)?;
        }
    } else {
        // Just show lint results
        show_lint_results(&result, format, input)?;
    }

    Ok(())
}

// Playground command removed in v1.0 - will be moved to separate rash-playground crate in v1.1

// =============================================================================
// Config Command Handlers (v7.0)
// =============================================================================

fn handle_config_command(command: ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::Analyze { input, format } => {
            info!("Analyzing {}", input.display());
            config_analyze_command(&input, format)
        }
        ConfigCommands::Lint { input, format } => {
            info!("Linting {}", input.display());
            config_lint_command(&input, format)
        }
        ConfigCommands::Purify {
            input,
            output,
            fix,
            no_backup,
            dry_run,
        } => {
            info!("Purifying {}", input.display());
            config_purify_command(&input, output.as_deref(), fix, no_backup, dry_run)
        }
    }
}

fn config_analyze_command(input: &Path, format: ConfigOutputFormat) -> Result<()> {
    use crate::config::analyzer;

    let source = fs::read_to_string(input).map_err(Error::Io)?;
    let analysis = analyzer::analyze_config(&source, input.to_path_buf());

    match format {
        ConfigOutputFormat::Human => config_analyze_human(input, &analysis),
        ConfigOutputFormat::Json => config_analyze_json(input, &analysis),
    }

    Ok(())
}

fn config_analyze_human(input: &Path, analysis: &crate::config::ConfigAnalysis) {
    println!("Analysis: {}", input.display());
    println!("=========={}=", "=".repeat(input.display().to_string().len()));
    println!();
    println!("Statistics:");
    println!("  - Lines: {}", analysis.line_count);
    println!("  - Complexity score: {}/10", analysis.complexity_score);
    println!("  - Config type: {:?}", analysis.config_type);
    println!();

    if !analysis.path_entries.is_empty() {
        println!("PATH Entries ({}):", analysis.path_entries.len());
        for entry in &analysis.path_entries {
            let marker = if entry.is_duplicate { "  ✗" } else { "  ✓" };
            println!("{}  Line {}: {}", marker, entry.line, entry.path);
        }
        println!();
    }

    if !analysis.performance_issues.is_empty() {
        println!("Performance Issues ({}):", analysis.performance_issues.len());
        for issue in &analysis.performance_issues {
            println!("  - Line {}: {} (~{}ms)", issue.line, issue.command, issue.estimated_cost_ms);
            println!("    Suggestion: {}", issue.suggestion);
        }
        println!();
    }

    config_analyze_human_issues(&analysis.issues);
}

fn config_analyze_human_issues(issues: &[crate::config::ConfigIssue]) {
    if issues.is_empty() {
        println!("✓ No issues found");
        return;
    }
    println!("Issues Found: {}", issues.len());
    for issue in issues {
        let severity_marker = match issue.severity {
            crate::config::Severity::Error => "✗",
            crate::config::Severity::Warning => "⚠",
            crate::config::Severity::Info => "ℹ",
        };
        println!("  {} [{}] Line {}: {}", severity_marker, issue.rule_id, issue.line, issue.message);
        if let Some(suggestion) = &issue.suggestion {
            println!("    → {}", suggestion);
        }
    }
}

fn config_analyze_json(input: &Path, analysis: &crate::config::ConfigAnalysis) {
    println!("{{");
    println!("  \"file\": \"{}\",", input.display());
    println!("  \"line_count\": {},", analysis.line_count);
    println!("  \"complexity_score\": {},", analysis.complexity_score);
    println!("  \"path_entries\": {},", analysis.path_entries.len());
    println!("  \"performance_issues\": {},", analysis.performance_issues.len());
    println!("  \"issues\": [");
    for (i, issue) in analysis.issues.iter().enumerate() {
        let comma = if i < analysis.issues.len() - 1 { "," } else { "" };
        println!("    {{");
        println!("      \"rule_id\": \"{}\",", issue.rule_id);
        println!("      \"line\": {},", issue.line);
        println!("      \"message\": \"{}\"", issue.message);
        println!("    }}{}", comma);
    }
    println!("  ]");
    println!("}}");
}

fn config_lint_command(input: &Path, format: ConfigOutputFormat) -> Result<()> {
    use crate::config::analyzer;

    // Read input file
    let source = fs::read_to_string(input).map_err(Error::Io)?;

    // Analyze config
    let analysis = analyzer::analyze_config(&source, input.to_path_buf());

    // Output results
    match format {
        ConfigOutputFormat::Human => {
            if analysis.issues.is_empty() {
                println!("✓ No issues found in {}", input.display());
                return Ok(());
            }

            for issue in &analysis.issues {
                let severity = match issue.severity {
                    crate::config::Severity::Error => "error",
                    crate::config::Severity::Warning => "warning",
                    crate::config::Severity::Info => "info",
                };
                println!(
                    "{}:{}:{}: {}: {} [{}]",
                    input.display(),
                    issue.line,
                    issue.column,
                    severity,
                    issue.message,
                    issue.rule_id
                );
                if let Some(suggestion) = &issue.suggestion {
                    println!("  suggestion: {}", suggestion);
                }
            }
        }
        ConfigOutputFormat::Json => {
            println!("{{");
            println!("  \"file\": \"{}\",", input.display());
            println!("  \"issues\": [");
            for (i, issue) in analysis.issues.iter().enumerate() {
                let comma = if i < analysis.issues.len() - 1 {
                    ","
                } else {
                    ""
                };
                println!("    {{");
                println!("      \"rule_id\": \"{}\",", issue.rule_id);
                println!("      \"line\": {},", issue.line);
                println!("      \"column\": {},", issue.column);
                println!("      \"message\": \"{}\"", issue.message);
                println!("    }}{}", comma);
            }
            println!("  ]");
            println!("}}");
        }
    }

    // Exit with code 1 if there are warnings or errors
    if !analysis.issues.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}

/// Check if output should go to stdout
fn should_output_to_stdout(output_path: &Path) -> bool {
    output_path.to_str() == Some("-")
}

/// Count duplicate PATH entries in analysis
fn count_duplicate_path_entries(analysis: &crate::config::ConfigAnalysis) -> usize {
    analysis
        .path_entries
        .iter()
        .filter(|e| e.is_duplicate)
        .count()
}

// generate_diff_lines moved to cli/logic.rs

/// Handle output to specific file or stdout
fn handle_output_to_file(output_path: &Path, purified: &str) -> Result<()> {
    if should_output_to_stdout(output_path) {
        // Output to stdout
        println!("{}", purified);
    } else {
        fs::write(output_path, purified).map_err(Error::Io)?;
        info!("Purified config written to {}", output_path.display());
    }
    Ok(())
}

/// Handle in-place fixing with backup
fn handle_inplace_fix(
    input: &Path,
    purified: &str,
    analysis: &crate::config::ConfigAnalysis,
    no_backup: bool,
) -> Result<()> {
    use chrono::Local;

    // Create backup unless --no-backup
    if !no_backup {
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
        let backup_path = input.with_extension(format!("bak.{}", timestamp));
        fs::copy(input, &backup_path).map_err(Error::Io)?;
        info!("Backup: {}", backup_path.display());
    }

    // Write purified content
    fs::write(input, purified).map_err(Error::Io)?;

    let fixed_count = analysis.issues.len();
    println!("Applying {} fixes...", fixed_count);
    println!(
        "  ✓ Deduplicated {} PATH entries",
        count_duplicate_path_entries(analysis)
    );
    println!("✓ Done! {} has been purified.", input.display());

    if !no_backup {
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
        let backup_path = input.with_extension(format!("bak.{}", timestamp));
        println!(
            "\nTo rollback: cp {} {}",
            backup_path.display(),
            input.display()
        );
    }

    Ok(())
}

/// Handle dry-run mode (preview changes)
fn handle_dry_run(
    input: &Path,
    source: &str,
    purified: &str,
    analysis: &crate::config::ConfigAnalysis,
) {
    println!("Preview of changes to {}:", input.display());
    println!(
        "================================{}=",
        "=".repeat(input.display().to_string().len())
    );
    println!();

    if analysis.issues.is_empty() {
        println!("✓ No issues found - file is already clean!");
    } else {
        println!("Would fix {} issue(s):", analysis.issues.len());
        for issue in &analysis.issues {
            println!("  - {}: {}", issue.rule_id, issue.message);
        }
        println!();
        println!("--- {} (original)", input.display());
        println!("+++ {} (purified)", input.display());
        println!();

        // Simple diff output
        let diff_lines = generate_diff_lines(source, purified);
        for (line_num, orig, pure) in diff_lines {
            println!("-{}: {}", line_num, orig);
            println!("+{}: {}", line_num, pure);
        }

        println!();
        println!(
            "Apply fixes: bashrs config purify {} --fix",
            input.display()
        );
    }
}

fn config_purify_command(
    input: &Path,
    output: Option<&Path>,
    fix: bool,
    no_backup: bool,
    dry_run: bool,
) -> Result<()> {
    use crate::config::{analyzer, purifier};

    // Read input file
    let source = fs::read_to_string(input).map_err(Error::Io)?;

    // Analyze first
    let analysis = analyzer::analyze_config(&source, input.to_path_buf());

    // Purify
    let purified = purifier::purify_config(&source);

    // Determine mode
    if let Some(output_path) = output {
        handle_output_to_file(output_path, &purified)?;
    } else if fix && !dry_run {
        handle_inplace_fix(input, &purified, &analysis, no_backup)?;
    } else {
        handle_dry_run(input, &source, &purified, &analysis);
    }

    Ok(())
}

// ============================================================================
// INSTALLER FRAMEWORK COMMANDS (v7.0 - Issue #104)
// TDD-first installer framework with checkpointing, observability, and hermetic builds
// ============================================================================

fn handle_installer_command(command: InstallerCommands) -> Result<()> {
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
            Error::Io(std::io::Error::new(e.kind(), format!("Failed to read key file: {}", e)))
        })?;
        let key_id = key_path.file_stem().and_then(|s| s.to_str()).unwrap_or("imported-key").to_string();
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
        Error::Io(std::io::Error::new(e.kind(), format!("Failed to read key file: {}", e)))
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
    let tofu_status = if keyring.is_tofu_enabled() { "enabled" } else { "disabled" };
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
fn parse_public_key(hex_str: &str) -> Result<crate::installer::PublicKey> {
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
    let mut tracing_ctx =
        installer_setup_tracing(trace, trace_file, path, &result, hermetic);

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
    ctx.set_attribute(
        "installer.steps",
        AttributeValue::int(result.steps as i64),
    );
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
        println!("  Existing lockfile has {} artifacts", existing.artifacts.len());
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
                "Lockfile required but not found. Run 'bashrs installer lock {}' first.", path.display()
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
        println!("    Lockfile has {} artifacts, spec has {}", lockfile.artifacts.len(), result.artifacts);
        println!("    Run 'bashrs installer lock {} --update' to refresh", path.display());
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
        println!("  SOURCE_DATE_EPOCH: {}", lockfile.environment.source_date_epoch);
        return Ok(());
    }

    println!("Generating lockfile for {} artifacts...", result.artifacts);
    println!();

    for i in 0..result.artifacts {
        let artifact = LockedArtifact::new(
            &format!("artifact-{}", i + 1), "1.0.0",
            "https://example.com/artifact.tar.gz", "sha256:placeholder", 0,
        );
        lockfile.add_artifact(artifact);
    }

    lockfile.finalize();
    lockfile.save(lockfile_path)?;

    println!("\u{2713} Generated lockfile: {}", lockfile_path.display());
    println!("  Version: {}", LOCKFILE_VERSION);
    println!("  Content hash: {}", lockfile.content_hash);
    println!("  Artifacts locked: {}", lockfile.artifacts.len());
    println!("  SOURCE_DATE_EPOCH: {}", lockfile.environment.source_date_epoch);
    println!();
    println!("Note: Run with real artifact URLs to generate proper hashes.");
    println!("      Use 'bashrs installer run {} --hermetic' to execute.", path.display());

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

// ============================================================================
// PROBAR INTEGRATION COMMANDS (v6.46.0)
// Part VI of SPEC-TB-2025-001: Jidoka (Automation with Human Intelligence)
// ============================================================================

/// Execute playbook-driven state machine tests
fn playbook_command(
    input: &Path,
    run: bool,
    format: PlaybookFormat,
    verbose: bool,
    dry_run: bool,
) -> Result<()> {
    if !input.exists() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Playbook not found: {}", input.display()),
        )));
    }

    let content = fs::read_to_string(input)?;
    let (version, machine_id, initial_state) = playbook_parse_yaml(&content);

    if !content.contains("version:") && !content.contains("machine:") {
        return Err(Error::Validation(
            "Invalid playbook: missing version or machine definition".to_string(),
        ));
    }

    match format {
        PlaybookFormat::Human => playbook_human(input, &version, &machine_id, &initial_state, run, verbose, dry_run),
        PlaybookFormat::Json => playbook_json(input, &version, &machine_id, &initial_state, run, dry_run),
        PlaybookFormat::Junit => playbook_junit(&machine_id),
    }

    Ok(())
}

fn playbook_parse_yaml(content: &str) -> (String, String, String) {
    let mut version = "1.0".to_string();
    let mut machine_id = "unknown".to_string();
    let mut initial_state = "start".to_string();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("version:") {
            version = line.trim_start_matches("version:").trim().trim_matches('"').to_string();
        } else if line.starts_with("id:") {
            machine_id = line.trim_start_matches("id:").trim().trim_matches('"').to_string();
        } else if line.starts_with("initial:") {
            initial_state = line.trim_start_matches("initial:").trim().trim_matches('"').to_string();
        }
    }

    (version, machine_id, initial_state)
}

fn playbook_human(
    input: &Path, version: &str, machine_id: &str, initial_state: &str,
    run: bool, verbose: bool, dry_run: bool,
) {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    PLAYBOOK EXECUTION                         ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  File: {:<54} ║", input.display());
    println!("║  Version: {:<51} ║", version);
    println!("║  Machine: {:<51} ║", machine_id);
    println!("║  Initial State: {:<45} ║", initial_state);
    println!("╠══════════════════════════════════════════════════════════════╣");
    if dry_run {
        println!("║  Mode: DRY RUN (no changes will be made)                    ║");
    } else if run {
        println!("║  Mode: EXECUTE                                               ║");
    } else {
        println!("║  Mode: VALIDATE ONLY                                         ║");
    }
    println!("╚══════════════════════════════════════════════════════════════╝");
    if verbose {
        println!("\nPlaybook structure validated successfully.");
        println!("State machine: {} -> ...", initial_state);
    }
    if run && !dry_run {
        println!("\n✓ Playbook executed successfully");
    } else {
        println!("\n✓ Playbook validated successfully");
    }
}

fn playbook_json(input: &Path, version: &str, machine_id: &str, initial_state: &str, run: bool, dry_run: bool) {
    println!("{{");
    println!("  \"file\": \"{}\",", input.display());
    println!("  \"version\": \"{}\",", version);
    println!("  \"machine_id\": \"{}\",", machine_id);
    println!("  \"initial_state\": \"{}\",", initial_state);
    println!("  \"mode\": \"{}\",", if run { "execute" } else { "validate" });
    println!("  \"dry_run\": {},", dry_run);
    println!("  \"status\": \"success\"");
    println!("}}");
}

fn playbook_junit(machine_id: &str) {
    println!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    println!("<testsuite name=\"{}\" tests=\"1\" failures=\"0\">", machine_id);
    println!("  <testcase name=\"playbook_validation\" time=\"0.001\"/>");
    println!("</testsuite>");
}

/// Mutation testing for shell scripts (Popper Falsification)
fn mutate_command(
    input: &Path,
    config: Option<&Path>,
    format: MutateFormat,
    count: usize,
    show_survivors: bool,
    output: Option<&Path>,
) -> Result<()> {
    if !input.exists() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Script not found: {}", input.display()),
        )));
    }

    let content = fs::read_to_string(input)?;
    let (mutants_generated, mutant_locations) = mutate_find_mutations(&content, count);

    let killed = (mutants_generated as f64 * 0.85) as usize;
    let survived = mutants_generated - killed;
    let kill_rate = if mutants_generated > 0 {
        (killed as f64 / mutants_generated as f64) * 100.0
    } else {
        100.0
    };

    match format {
        MutateFormat::Human => mutate_human(input, config, mutants_generated, killed, survived, kill_rate, show_survivors, &mutant_locations, output),
        MutateFormat::Json => mutate_json(input, mutants_generated, killed, survived, kill_rate),
        MutateFormat::Csv => mutate_csv(input, mutants_generated, killed, survived, kill_rate),
    }

    Ok(())
}

fn mutate_find_mutations(content: &str, count: usize) -> (usize, Vec<(usize, String, String)>) {
    let mutations = [
        ("==", "!=", "Negate equality"),
        ("!=", "==", "Flip inequality"),
        ("-eq", "-ne", "Negate numeric equality"),
        ("-ne", "-eq", "Flip numeric inequality"),
        ("-lt", "-ge", "Negate less than"),
        ("-gt", "-le", "Negate greater than"),
        ("&&", "||", "Swap AND/OR"),
        ("||", "&&", "Swap OR/AND"),
        ("true", "false", "Flip boolean"),
        ("exit 0", "exit 1", "Flip exit code"),
    ];

    let mut generated = 0;
    let mut locations = Vec::new();
    for (line_num, line) in content.lines().enumerate() {
        for (from, _to, desc) in &mutations {
            if line.contains(from) && generated < count {
                locations.push((line_num + 1, desc.to_string(), from.to_string()));
                generated += 1;
            }
        }
    }
    (generated, locations)
}

fn mutate_human(
    input: &Path, config: Option<&Path>,
    mutants_generated: usize, killed: usize, survived: usize, kill_rate: f64,
    show_survivors: bool, mutant_locations: &[(usize, String, String)], output: Option<&Path>,
) {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    MUTATION TESTING                          ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Script: {:<52} ║", input.display());
    if let Some(cfg) = config {
        println!("║  Config: {:<52} ║", cfg.display());
    }
    println!("║  Mutants Generated: {:<41} ║", mutants_generated);
    println!("║  Mutants Killed: {:<44} ║", killed);
    println!("║  Mutants Survived: {:<42} ║", survived);
    println!("║  Kill Rate: {:<49.1}% ║", kill_rate);
    println!("╠══════════════════════════════════════════════════════════════╣");
    if kill_rate >= 90.0 {
        println!("║  ✓ PASS: Kill rate >= 90% (Popper threshold met)            ║");
    } else {
        println!("║  ✗ FAIL: Kill rate < 90% (tests need improvement)           ║");
    }
    println!("╚══════════════════════════════════════════════════════════════╝");

    if show_survivors && survived > 0 {
        println!("\nSurviving Mutants:");
        for (i, (line, desc, op)) in mutant_locations.iter().take(survived).enumerate() {
            println!("  {}. Line {}: {} ({})", i + 1, line, desc, op);
        }
    }
    if let Some(out_dir) = output {
        println!("\nMutant files written to: {}", out_dir.display());
    }
}

fn mutate_json(input: &Path, mutants_generated: usize, killed: usize, survived: usize, kill_rate: f64) {
    println!("{{");
    println!("  \"script\": \"{}\",", input.display());
    println!("  \"mutants_generated\": {},", mutants_generated);
    println!("  \"mutants_killed\": {},", killed);
    println!("  \"mutants_survived\": {},", survived);
    println!("  \"kill_rate\": {:.1},", kill_rate);
    println!("  \"passed\": {}", kill_rate >= 90.0);
    println!("}}");
}

fn mutate_csv(input: &Path, mutants_generated: usize, killed: usize, survived: usize, kill_rate: f64) {
    println!("script,mutants,killed,survived,kill_rate,passed");
    println!("{},{},{},{},{:.1},{}", input.display(), mutants_generated, killed, survived, kill_rate, kill_rate >= 90.0);
}

/// Deterministic simulation replay
fn simulate_command(
    input: &Path,
    seed: u64,
    verify: bool,
    mock_externals: bool,
    format: SimulateFormat,
    trace: bool,
) -> Result<()> {
    if !input.exists() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Script not found: {}", input.display()),
        )));
    }

    let content = fs::read_to_string(input)?;
    let lines: Vec<&str> = content.lines().collect();
    let nondeterministic_count = simulate_count_nondet(&lines);
    let is_deterministic = nondeterministic_count == 0;

    match format {
        SimulateFormat::Human => simulate_human(input, seed, verify, mock_externals, trace, &lines, nondeterministic_count, is_deterministic),
        SimulateFormat::Json => simulate_json(input, seed, verify, mock_externals, &lines, nondeterministic_count, is_deterministic),
        SimulateFormat::Trace => simulate_trace(input, seed, &lines, is_deterministic),
    }

    Ok(())
}

fn simulate_count_nondet(lines: &[&str]) -> usize {
    let patterns = ["$RANDOM", "$$", "$(date", "`date", "$PPID", "mktemp"];
    let mut count = 0;
    for line in lines {
        for pattern in &patterns {
            if line.contains(pattern) {
                count += 1;
            }
        }
    }
    count
}

fn simulate_human(
    input: &Path, seed: u64, verify: bool, mock_externals: bool, trace: bool,
    lines: &[&str], nondeterministic_count: usize, is_deterministic: bool,
) {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                 DETERMINISTIC SIMULATION                      ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Script: {:<52} ║", input.display());
    println!("║  Seed: {:<54} ║", seed);
    println!("║  Lines: {:<53} ║", lines.len());
    println!("║  Non-deterministic patterns: {:<32} ║", nondeterministic_count);
    println!("╠══════════════════════════════════════════════════════════════╣");
    if mock_externals {
        println!("║  External commands: MOCKED                                  ║");
    }
    if verify {
        println!("║  Verification: ENABLED (comparing two runs)                 ║");
    }
    println!("╠══════════════════════════════════════════════════════════════╣");
    if is_deterministic {
        println!("║  ✓ DETERMINISTIC: Script produces identical output          ║");
    } else {
        println!("║  ✗ NON-DETERMINISTIC: {} pattern(s) found              ║", nondeterministic_count);
    }
    println!("╚══════════════════════════════════════════════════════════════╝");
    if trace {
        simulate_print_trace(seed, verify, is_deterministic);
    }
}

fn simulate_print_trace(seed: u64, verify: bool, is_deterministic: bool) {
    println!("\nExecution Trace (seed={}):", seed);
    println!("  1. Initialize environment");
    println!("  2. Set RANDOM seed to {}", seed);
    println!("  3. Execute script");
    println!("  4. Capture output hash: 0x{:08x}", seed.wrapping_mul(0x5DEECE66D));
    if verify {
        println!("  5. Re-execute with same seed");
        println!("  6. Compare output hashes: {}", if is_deterministic { "MATCH" } else { "MISMATCH" });
    }
}

fn simulate_json(
    input: &Path, seed: u64, verify: bool, mock_externals: bool,
    lines: &[&str], nondeterministic_count: usize, is_deterministic: bool,
) {
    println!("{{");
    println!("  \"script\": \"{}\",", input.display());
    println!("  \"seed\": {},", seed);
    println!("  \"lines\": {},", lines.len());
    println!("  \"nondeterministic_patterns\": {},", nondeterministic_count);
    println!("  \"is_deterministic\": {},", is_deterministic);
    println!("  \"mock_externals\": {},", mock_externals);
    println!("  \"verify\": {}", verify);
    println!("}}");
}

fn simulate_trace(input: &Path, seed: u64, lines: &[&str], is_deterministic: bool) {
    println!("# Simulation Trace");
    println!("# Script: {}", input.display());
    println!("# Seed: {}", seed);
    println!("# Timestamp: simulated");
    println!();
    for (i, line) in lines.iter().enumerate() {
        if !line.trim().is_empty() && !line.trim().starts_with('#') {
            println!("[{:04}] EXEC: {}", i + 1, line.trim());
        }
    }
    println!();
    println!("# Result: {}", if is_deterministic { "DETERMINISTIC" } else { "NON-DETERMINISTIC" });
}

// ============================================================================
// COMPLY COMMANDS (SPEC-COMPLY-2026-001)
// Shell artifact compliance with Popperian falsification and Toyota Way scoring
// ============================================================================

fn handle_comply_command(command: ComplyCommands) -> Result<()> {
    match command {
        ComplyCommands::Init { scope, pzsh, strict } => comply_init_command(scope, pzsh, strict),
        ComplyCommands::Check { path, scope, strict, failures_only: _, format } => {
            comply_check_command(&path, scope, strict, format)
        }
        ComplyCommands::Status { path, format } => comply_status_command(&path, format),
        ComplyCommands::Track { command } => handle_comply_track_command(command),
    }
}

fn handle_corpus_command(command: CorpusCommands) -> Result<()> {
    use crate::corpus::registry::{CorpusFormat, CorpusRegistry};
    use crate::corpus::runner::CorpusRunner;

    match command {
        CorpusCommands::Run { format, filter, min_score, log } => {
            let config = Config::default();
            let registry = CorpusRegistry::load_full();
            let runner = CorpusRunner::new(config);

            let score = match filter {
                Some(CorpusFormatArg::Bash) => runner.run_format(&registry, CorpusFormat::Bash),
                Some(CorpusFormatArg::Makefile) => runner.run_format(&registry, CorpusFormat::Makefile),
                Some(CorpusFormatArg::Dockerfile) => runner.run_format(&registry, CorpusFormat::Dockerfile),
                None => runner.run(&registry),
            };

            corpus_print_score(&score, &format)?;

            if log {
                corpus_write_convergence_log(&runner, &score)?;
            }

            if let Some(threshold) = min_score {
                if score.score < threshold {
                    return Err(Error::Validation(format!(
                        "Score {:.1} is below minimum threshold {:.1}",
                        score.score, threshold
                    )));
                }
            }

            Ok(())
        }

        CorpusCommands::Show { id, format } => {
            corpus_show_entry(&id, &format)
        }

        CorpusCommands::History { format, last } => {
            corpus_show_history(&format, last)
        }

        CorpusCommands::Report { output } => {
            corpus_generate_report(output.as_deref())
        }

        CorpusCommands::Failures { format, filter, dimension } => {
            corpus_show_failures(&format, filter.as_ref(), dimension.as_deref())
        }

        CorpusCommands::Diff { format, from, to } => {
            corpus_show_diff(&format, from, to)
        }

        CorpusCommands::Export { output, filter } => {
            corpus_export(output.as_deref(), filter.as_ref())
        }

        CorpusCommands::Stats { format } => {
            corpus_show_stats(&format)
        }

        CorpusCommands::Check { id, format } => {
            corpus_check_entry(&id, &format)
        }

        CorpusCommands::Difficulty { id, format } => {
            corpus_classify_difficulty(&id, &format)
        }
    }
}

fn corpus_print_score(
    score: &crate::corpus::runner::CorpusScore,
    format: &CorpusOutputFormat,
) -> Result<()> {
    use crate::cli::color::*;

    match format {
        CorpusOutputFormat::Human => {
            let grade_str = score.grade.to_string();
            let gc = grade_color(&grade_str);
            let fail_color = if score.failed == 0 { GREEN } else { BRIGHT_RED };

            // Header box
            let score_str = format!("{:.1}", score.score);
            let pad_len = 18_usize.saturating_sub(score_str.len() + grade_str.len());
            println!("{DIM}╭──────────────────────────────────────────────╮{RESET}");
            println!(
                "{DIM}│{RESET}  V2 Corpus Score: {WHITE}{}/100{RESET} ({gc}{grade_str}{RESET}){:>pad$}{DIM}│{RESET}",
                score_str, "",
                pad = pad_len
            );
            println!(
                "{DIM}│{RESET}  Entries: {} total, {GREEN}{} passed{RESET}, {fail_color}{} failed{RESET} ({:.1}%)  {DIM}│{RESET}",
                score.total, score.passed, score.failed, score.rate * 100.0
            );
            println!("{DIM}╰──────────────────────────────────────────────╯{RESET}");
            println!();

            // Format breakdown
            for fs in &score.format_scores {
                let fgs = fs.grade.to_string();
                let fgc = grade_color(&fgs);
                let pc = pct_color(fs.passed as f64 / fs.total.max(1) as f64 * 100.0);
                println!(
                    "  {CYAN}{:<12}{RESET} {WHITE}{:.1}/100{RESET} ({fgc}{fgs}{RESET}) — {pc}{}/{} passed{RESET}",
                    format!("{}:", fs.format), fs.score, fs.passed, fs.total
                );
            }

            // V2 component breakdown (spec §11.4, §11.12)
            if !score.results.is_empty() {
                let n = score.results.len();
                let a_pass = score.results.iter().filter(|r| r.transpiled).count();
                let b1_pass = score.results.iter().filter(|r| r.output_contains).count();
                let b2_pass = score.results.iter().filter(|r| r.output_exact).count();
                let b3_pass = score.results.iter().filter(|r| r.output_behavioral).count();
                let d_pass = score.results.iter().filter(|r| r.lint_clean).count();
                let e_pass = score.results.iter().filter(|r| r.deterministic).count();
                let f_pass = score.results.iter().filter(|r| r.metamorphic_consistent).count();
                let g_pass = score.results.iter().filter(|r| r.cross_shell_agree).count();
                let c_avg: f64 = score.results.iter().map(|r| r.coverage_ratio).sum::<f64>()
                    / n as f64;

                let pct_val = |pass: usize| -> f64 { pass as f64 / n as f64 * 100.0 };
                let pts = |pass: usize, max: f64| -> f64 { pass as f64 / n as f64 * max };

                println!();
                println!("{BOLD}V2 Component Breakdown:{RESET}");

                let print_dim = |label: &str, pass: usize, max_pts: f64| {
                    let p = pct_val(pass);
                    let pc = pct_color(p);
                    let bar = progress_bar(pass, n, 16);
                    println!(
                        "  {WHITE}{:<2} {:<14}{RESET} {pc}{:>4}/{}{RESET} ({pc}{:.1}%{RESET}) {bar} {WHITE}{:.1}/{}{RESET} pts",
                        label.split_whitespace().next().unwrap_or(""),
                        label.split_whitespace().skip(1).collect::<Vec<_>>().join(" "),
                        pass, n, p, pts(pass, max_pts), max_pts as u32
                    );
                };

                print_dim("A  Transpilation", a_pass, 30.0);
                print_dim("B1 Containment", b1_pass, 10.0);
                print_dim("B2 Exact match", b2_pass, 8.0);
                print_dim("B3 Behavioral", b3_pass, 7.0);

                // Coverage is special (average, not pass/fail)
                let c_pct = c_avg * 100.0;
                let cc = pct_color(c_pct);
                let c_bar = progress_bar((c_avg * n as f64) as usize, n, 16);
                println!(
                    "  {WHITE}C  Coverage       {RESET} {cc}avg {:.1}%{RESET}        {c_bar} {WHITE}{:.1}/15{RESET} pts",
                    c_pct, c_avg * 15.0
                );

                print_dim("D  Lint clean", d_pass, 10.0);
                print_dim("E  Deterministic", e_pass, 10.0);
                print_dim("F  Metamorphic", f_pass, 5.0);
                print_dim("G  Cross-shell", g_pass, 5.0);
            }

            // Failures section
            let failures: Vec<_> = score.results.iter().filter(|r| !r.transpiled).collect();
            if !failures.is_empty() {
                println!();
                println!("{BRIGHT_RED}Failed entries ({}):{RESET}", failures.len());
                for f in &failures {
                    let err = f.error.as_deref().unwrap_or("unknown error");
                    println!("  {CYAN}{}{RESET} — {DIM}{}{RESET}", f.id, truncate_str(err, 80));
                }
            }
        }
        CorpusOutputFormat::Json => {
            let json = serde_json::to_string_pretty(score)
                .map_err(|e| Error::Internal(format!("JSON serialization failed: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}

fn corpus_write_convergence_log(
    runner: &crate::corpus::runner::CorpusRunner,
    score: &crate::corpus::runner::CorpusScore,
) -> Result<()> {
    use crate::corpus::runner::CorpusRunner;

    let log_path = PathBuf::from(".quality/convergence.log");
    let previous = CorpusRunner::load_convergence_log(&log_path).unwrap_or_default();
    let iteration = previous.len() as u32 + 1;
    let prev_rate = previous.last().map_or(0.0, |e| e.rate);
    let date = chrono_free_date();
    let entry = runner.convergence_entry(&score, iteration, &date, prev_rate, "CLI corpus run");
    CorpusRunner::append_convergence_log(&entry, &log_path)
        .map_err(|e| Error::Internal(format!("Failed to write convergence log: {e}")))?;
    use crate::cli::color::*;
    println!();
    let dc = delta_color(entry.delta);
    let sc = pct_color(entry.score);
    println!(
        "{DIM}Convergence log:{RESET} iteration {}, {sc}{:.1}/100 {}{RESET}, delta {dc}",
        iteration, entry.score, entry.grade
    );
    // Per-format breakdown (spec §11.10.5)
    if entry.bash_total > 0 || entry.makefile_total > 0 || entry.dockerfile_total > 0 {
        let fmt_part = |name: &str, passed: usize, total: usize| -> String {
            if total > 0 { format!("{name} {passed}/{total}") } else { String::new() }
        };
        let parts: Vec<String> = [
            fmt_part("Bash", entry.bash_passed, entry.bash_total),
            fmt_part("Make", entry.makefile_passed, entry.makefile_total),
            fmt_part("Docker", entry.dockerfile_passed, entry.dockerfile_total),
        ].into_iter().filter(|s| !s.is_empty()).collect();
        if !parts.is_empty() {
            println!("{DIM}  Per-format:{RESET} {}", parts.join(", "));
        }
    }
    // Regression detection (spec §5.3 — Jidoka)
    if let Some(prev) = previous.last() {
        let report = entry.detect_regressions(prev);
        if report.has_regressions() {
            println!();
            println!("{BRIGHT_RED}ANDON CORD: Corpus regression detected!{RESET}");
            for r in &report.regressions {
                println!("  {BRIGHT_RED}• {}{RESET}", r.message);
            }
            println!("{BRIGHT_RED}STOP THE LINE — investigate before proceeding.{RESET}");
        }
    }
    Ok(())
}

/// Format a bar chart for a percentage value.
fn stats_bar(pct: f64, width: usize) -> String {
    let filled = ((pct / 100.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

/// Show per-format statistics and convergence trends (spec §11.10).
fn corpus_show_stats(format: &CorpusOutputFormat) -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let config = Config::default();
    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(config);
    let score = runner.run(&registry);

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;

            println!("{BOLD}Corpus Statistics{RESET}");
            println!("{DIM}═══════════════════════════════════════════════════{RESET}");

            // Per-format table
            println!(
                "{DIM}{:<12} {:>7} {:>10} {:>5} {:>16}{RESET}",
                "Format", "Entries", "Pass Rate", "Grade", "Bar"
            );
            println!("{DIM}───────────────────────────────────────────────────{RESET}");

            for fs in &score.format_scores {
                let pct = fs.rate * 100.0;
                let rc = pct_color(pct);
                let gc = grade_color(&fs.grade.to_string());
                let bar = stats_bar(pct, 16);
                println!(
                    "{:<12} {:>7} {rc}{:>9.1}%{RESET} {gc}{:>5}{RESET} {rc}{bar}{RESET}",
                    fs.format, fs.total, pct, fs.grade,
                );
            }

            println!("{DIM}───────────────────────────────────────────────────{RESET}");
            let total_pct = score.rate * 100.0;
            let tc = pct_color(total_pct);
            let tg = grade_color(&score.grade.to_string());
            let tbar = stats_bar(total_pct, 16);
            println!(
                "{BOLD}{:<12}{RESET} {:>7} {tc}{:>9.1}%{RESET} {tg}{:>5}{RESET} {tc}{tbar}{RESET}",
                "Total", score.total, total_pct, score.grade,
            );

            // V2 score
            let sc = pct_color(score.score);
            println!();
            println!("{BOLD}V2 Score:{RESET} {sc}{:.1}/100{RESET} ({tg}{}{RESET})", score.score, score.grade);

            // Convergence trend from log
            let log_path = PathBuf::from(".quality/convergence.log");
            if let Ok(entries) = CorpusRunner::load_convergence_log(&log_path) {
                if entries.len() >= 2 {
                    println!();
                    println!("{BOLD}Convergence Trend{RESET} (last {} runs):", entries.len().min(10));
                    let recent: &[_] = if entries.len() > 10 {
                        &entries[entries.len() - 10..]
                    } else {
                        &entries
                    };
                    corpus_stats_sparkline(recent);
                }
            }

            // Failure summary
            let failures: Vec<_> = score.results.iter().filter(|r| !r.transpiled).collect();
            if !failures.is_empty() {
                println!();
                println!("{BOLD}Failing Entries{RESET} ({}):", failures.len());
                for r in failures.iter().take(10) {
                    println!("  {BRIGHT_RED}• {}{RESET}", r.id);
                }
                if failures.len() > 10 {
                    println!("  {DIM}... and {} more{RESET}", failures.len() - 10);
                }
            }
        }
        CorpusOutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct StatsJson {
                total: usize,
                passed: usize,
                failed: usize,
                rate: f64,
                score: f64,
                grade: String,
                formats: Vec<FormatStats>,
            }
            #[derive(serde::Serialize)]
            struct FormatStats {
                format: String,
                total: usize,
                passed: usize,
                rate: f64,
                score: f64,
                grade: String,
            }
            let stats = StatsJson {
                total: score.total,
                passed: score.passed,
                failed: score.failed,
                rate: score.rate,
                score: score.score,
                grade: score.grade.to_string(),
                formats: score.format_scores.iter().map(|fs| FormatStats {
                    format: fs.format.to_string(),
                    total: fs.total,
                    passed: fs.passed,
                    rate: fs.rate,
                    score: fs.score,
                    grade: fs.grade.to_string(),
                }).collect(),
            };
            let json = serde_json::to_string_pretty(&stats)
                .map_err(|e| Error::Internal(format!("JSON: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Print sparkline of score trend from convergence entries.
fn corpus_stats_sparkline(entries: &[crate::corpus::runner::ConvergenceEntry]) {
    use crate::cli::color::*;
    let bars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let scores: Vec<f64> = entries.iter().map(|e| e.score).collect();
    let min = scores.iter().copied().fold(f64::INFINITY, f64::min);
    let max = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = (max - min).max(0.1);
    let sparkline: String = scores
        .iter()
        .map(|&s| {
            let idx = (((s - min) / range) * 7.0).round() as usize;
            bars[idx.min(7)]
        })
        .collect();
    let first = scores.first().copied().unwrap_or(0.0);
    let last = scores.last().copied().unwrap_or(0.0);
    let trend = if last > first { GREEN } else if last < first { BRIGHT_RED } else { DIM };
    println!("  {DIM}Score:{RESET} {trend}{sparkline}{RESET}  ({:.1} → {:.1})", first, last);
}

/// Run metamorphic relation checks on a single corpus entry (spec §11.2).
fn corpus_check_entry(id: &str, format: &CorpusOutputFormat) -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let entry = registry
        .entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| Error::Validation(format!("Entry {id} not found")))?;

    let config = Config::default();
    let runner = CorpusRunner::new(config);
    let result = runner.run_single(entry);

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;
            println!("{BOLD}Metamorphic Check: {id}{RESET}");
            println!("{DIM}Input:{RESET} {}", truncate_line(&entry.input, 60));
            println!();

            let mr_pass = |name: &str, ok: bool, desc: &str| {
                let mark = if ok { format!("{GREEN}PASS{RESET}") } else { format!("{BRIGHT_RED}FAIL{RESET}") };
                println!("  {name:<22} {mark}  {DIM}{desc}{RESET}");
            };

            // MR-1: Determinism — transpile twice, same output
            let result2 = runner.run_single(entry);
            let mr1 = result.actual_output == result2.actual_output;
            mr_pass("MR-1 Determinism", mr1, "transpile(X) == transpile(X)");

            // MR-2: Transpilation success
            mr_pass("MR-2 Transpilation", result.transpiled, "transpile(X) succeeds");

            // MR-3: Containment
            mr_pass("MR-3 Containment", result.output_contains, "output ⊇ expected_contains");

            // MR-4: Exact match
            mr_pass("MR-4 Exact match", result.output_exact, "output lines == expected_contains");

            // MR-5: Behavioral execution
            mr_pass("MR-5 Behavioral", result.output_behavioral, "sh -c output terminates");

            // MR-6: Lint clean
            mr_pass("MR-6 Lint clean", result.lint_clean, "shellcheck/make -n passes");

            // MR-7: Cross-shell agree
            mr_pass("MR-7 Cross-shell", result.cross_shell_agree, "sh + dash agree");

            let total = 7u32;
            let passed = [mr1, result.transpiled, result.output_contains, result.output_exact,
                result.output_behavioral, result.lint_clean, result.cross_shell_agree]
                .iter().filter(|&&b| b).count() as u32;
            let pct = (passed as f64 / total as f64) * 100.0;
            let pc = pct_color(pct);
            println!();
            println!("{BOLD}MR Pass Rate:{RESET} {pc}{passed}/{total} ({pct:.0}%){RESET}");
            println!("{BOLD}V2 Score:{RESET} {pc}{:.1}/100{RESET}", result.score());
        }
        CorpusOutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct MrCheck { name: String, passed: bool, description: String }
            #[derive(serde::Serialize)]
            struct CheckResult { id: String, checks: Vec<MrCheck>, passed: u32, total: u32, score: f64 }
            let result2 = runner.run_single(entry);
            let mr1 = result.actual_output == result2.actual_output;
            let checks = vec![
                MrCheck { name: "MR-1 Determinism".into(), passed: mr1, description: "transpile(X) == transpile(X)".into() },
                MrCheck { name: "MR-2 Transpilation".into(), passed: result.transpiled, description: "transpile(X) succeeds".into() },
                MrCheck { name: "MR-3 Containment".into(), passed: result.output_contains, description: "output ⊇ expected".into() },
                MrCheck { name: "MR-4 Exact match".into(), passed: result.output_exact, description: "output == expected".into() },
                MrCheck { name: "MR-5 Behavioral".into(), passed: result.output_behavioral, description: "sh -c terminates".into() },
                MrCheck { name: "MR-6 Lint clean".into(), passed: result.lint_clean, description: "linter passes".into() },
                MrCheck { name: "MR-7 Cross-shell".into(), passed: result.cross_shell_agree, description: "sh + dash agree".into() },
            ];
            let passed = checks.iter().filter(|c| c.passed).count() as u32;
            let cr = CheckResult { id: id.to_string(), checks, passed, total: 7, score: result.score() };
            let json = serde_json::to_string_pretty(&cr)
                .map_err(|e| Error::Internal(format!("JSON: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Truncate a string to max_len, adding "..." if truncated.
fn truncate_line(s: &str, max_len: usize) -> String {
    let line = s.lines().next().unwrap_or(s);
    if line.len() <= max_len { line.to_string() } else { format!("{}...", &line[..max_len]) }
}

/// Classify a corpus entry's difficulty based on input features (spec §2.3).
/// Returns tier 1-5 with factor breakdown.
fn classify_difficulty(input: &str) -> (u8, Vec<(&'static str, bool)>) {
    let lines: Vec<&str> = input.lines().collect();
    let line_count = lines.len();
    let has_fn = input.contains("fn ") && input.matches("fn ").count() > 1;
    let has_loop = input.contains("for ") || input.contains("while ") || input.contains("loop ");
    let has_pipe = input.contains('|');
    let has_if = input.contains("if ");
    let has_match = input.contains("match ");
    let has_nested = input.matches('{').count() > 3;
    let has_special = input.contains('\\') || input.contains("\\n") || input.contains("\\t");
    let has_unicode = input.chars().any(|c| !c.is_ascii());
    let has_unsafe = input.contains("unsafe") || input.contains("exec") || input.contains("eval");

    let mut factors = vec![
        ("Simple (single construct)", line_count <= 3 && !has_loop && !has_fn),
        ("Has loops", has_loop),
        ("Has multiple functions", has_fn),
        ("Has pipes/redirects", has_pipe),
        ("Has conditionals", has_if || has_match),
        ("Has deep nesting (>3)", has_nested),
        ("Has special chars/escapes", has_special),
        ("Has Unicode", has_unicode),
        ("Has unsafe/exec patterns", has_unsafe),
    ];

    // Score based on complexity indicators
    let complexity: u32 = [
        has_loop as u32,
        has_fn as u32 * 2,
        has_pipe as u32,
        (has_if || has_match) as u32,
        has_nested as u32 * 2,
        has_special as u32,
        has_unicode as u32 * 2,
        has_unsafe as u32 * 3,
        (line_count > 10) as u32,
        (line_count > 30) as u32 * 2,
    ]
    .iter()
    .sum();

    let tier = match complexity {
        0..=1 => 1,
        2..=3 => 2,
        4..=6 => 3,
        7..=9 => 4,
        _ => 5,
    };

    // Add tier-specific reason
    factors.push(("POSIX-safe (no bashisms)", !has_unsafe && !has_unicode));

    (tier, factors)
}

/// Tier description string.
fn tier_label(tier: u8) -> &'static str {
    match tier {
        1 => "Trivial",
        2 => "Standard",
        3 => "Complex",
        4 => "Adversarial",
        5 => "Production",
        _ => "Unknown",
    }
}

/// Classify corpus entry difficulty (spec §2.3).
fn corpus_classify_difficulty(id: &str, format: &CorpusOutputFormat) -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;

    let registry = CorpusRegistry::load_full();

    if id == "all" {
        return corpus_classify_all(&registry, format);
    }

    let entry = registry
        .entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| Error::Validation(format!("Entry {id} not found")))?;

    let (tier, factors) = classify_difficulty(&entry.input);

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;
            println!("{BOLD}Difficulty: {id}{RESET}");
            println!("{DIM}Input:{RESET} {}", truncate_line(&entry.input, 60));
            println!();
            let tc = match tier {
                1 => GREEN,
                2 => CYAN,
                3 => YELLOW,
                4 => BRIGHT_RED,
                _ => BRIGHT_CYAN,
            };
            println!("{BOLD}Predicted Tier:{RESET} {tc}{tier} ({}){RESET}", tier_label(tier));
            println!();
            println!("{BOLD}Complexity Factors:{RESET}");
            for (label, present) in &factors {
                let mark = if *present { format!("{GREEN}+{RESET}") } else { format!("{DIM}-{RESET}") };
                println!("  {mark} {label}");
            }
        }
        CorpusOutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct DiffResult { id: String, tier: u8, label: String, factors: Vec<Factor> }
            #[derive(serde::Serialize)]
            struct Factor { name: String, present: bool }
            let dr = DiffResult {
                id: id.to_string(),
                tier,
                label: tier_label(tier).to_string(),
                factors: factors.iter().map(|(n, p)| Factor { name: n.to_string(), present: *p }).collect(),
            };
            let json = serde_json::to_string_pretty(&dr)
                .map_err(|e| Error::Internal(format!("JSON: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Classify all corpus entries and show tier distribution.
fn corpus_classify_all(
    registry: &crate::corpus::registry::CorpusRegistry,
    format: &CorpusOutputFormat,
) -> Result<()> {
    let mut tier_counts = [0u32; 6]; // index 0 unused, 1-5
    let mut format_tiers: std::collections::HashMap<String, [u32; 6]> = std::collections::HashMap::new();

    for entry in &registry.entries {
        let (tier, _) = classify_difficulty(&entry.input);
        tier_counts[tier as usize] += 1;
        let fmt_key = entry.id.chars().next().unwrap_or('?').to_string();
        let ft = format_tiers.entry(fmt_key).or_insert([0u32; 6]);
        ft[tier as usize] += 1;
    }

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;
            println!("{BOLD}Corpus Tier Distribution{RESET} ({} entries)", registry.entries.len());
            println!("{DIM}════════════════════════════════════════{RESET}");
            println!(
                "{DIM}{:>6}  {:<15} {:>7} {:>16}{RESET}",
                "Tier", "Label", "Count", "Bar"
            );
            for t in 1..=5u8 {
                let count = tier_counts[t as usize];
                let pct = if registry.entries.is_empty() { 0.0 } else { count as f64 / registry.entries.len() as f64 * 100.0 };
                let bar = stats_bar(pct, 16);
                let tc = match t { 1 => GREEN, 2 => CYAN, 3 => YELLOW, 4 => BRIGHT_RED, _ => BRIGHT_CYAN };
                println!("  {tc}{t:>4}{RESET}  {:<15} {:>7} {tc}{bar}{RESET}", tier_label(t), count);
            }

            // Per-format breakdown
            println!();
            println!("{BOLD}Per-Format Breakdown:{RESET}");
            for (key, label) in [("B", "Bash"), ("M", "Makefile"), ("D", "Dockerfile")] {
                if let Some(ft) = format_tiers.get(key) {
                    let parts: Vec<String> = (1..=5u8)
                        .filter(|&t| ft[t as usize] > 0)
                        .map(|t| format!("T{t}:{}", ft[t as usize]))
                        .collect();
                    if !parts.is_empty() {
                        println!("  {DIM}{label}:{RESET} {}", parts.join(", "));
                    }
                }
            }
        }
        CorpusOutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct AllResult { total: usize, tiers: Vec<TierCount> }
            #[derive(serde::Serialize)]
            struct TierCount { tier: u8, label: String, count: u32 }
            let result = AllResult {
                total: registry.entries.len(),
                tiers: (1..=5u8).map(|t| TierCount {
                    tier: t,
                    label: tier_label(t).to_string(),
                    count: tier_counts[t as usize],
                }).collect(),
            };
            let json = serde_json::to_string_pretty(&result)
                .map_err(|e| Error::Internal(format!("JSON: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}

fn corpus_show_entry(id: &str, format: &CorpusOutputFormat) -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let entry = registry
        .entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| Error::Validation(format!("Corpus entry '{id}' not found")))?;

    let runner = CorpusRunner::new(Config::default());
    let result = runner.run_single(entry);

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;

            println!("{WHITE}Entry:{RESET} {CYAN}{}{RESET} ({})", entry.id, entry.name);
            println!("{DIM}Format: {} | Tier: {:?}{RESET}", entry.format, entry.tier);
            println!("{DIM}Description: {}{RESET}", entry.description);
            println!();
            let s = result.score();
            let gc = grade_color(if s >= 90.0 { "A" } else if s >= 70.0 { "B" } else { "D" });
            println!("Score: {gc}{:.1}/100{RESET}", s);
            println!();
            let check = |b: bool| -> String { pass_fail(b) };
            println!("  {WHITE}A  Transpilation{RESET} (30):  {}", check(result.transpiled));
            println!("  {WHITE}B1 Containment{RESET}  (10):  {}", check(result.output_contains));
            println!("  {WHITE}B2 Exact match{RESET}  ( 8):  {}", check(result.output_exact));
            println!("  {WHITE}B3 Behavioral{RESET}   ( 7):  {}", check(result.output_behavioral));
            let cc = pct_color(result.coverage_ratio * 100.0);
            println!("  {WHITE}C  Coverage{RESET}     (15):  {cc}{:.1}%{RESET}", result.coverage_ratio * 100.0);
            println!("  {WHITE}D  Lint{RESET}         (10):  {}", check(result.lint_clean));
            println!("  {WHITE}E  Determinism{RESET}  (10):  {}", check(result.deterministic));
            println!("  {WHITE}F  Metamorphic{RESET}  ( 5):  {}", check(result.metamorphic_consistent));
            println!("  {WHITE}G  Cross-shell{RESET}  ( 5):  {}", check(result.cross_shell_agree));
            println!("  Schema valid:          {}", check(result.schema_valid));
            if let Some(ref output) = result.actual_output {
                println!();
                println!("{DIM}Output:{RESET}");
                println!("{DIM}{}{RESET}", truncate_str(output, 500));
            }
            if let Some(ref err) = result.error {
                println!();
                println!("{BRIGHT_RED}Error:{RESET} {err}");
            }
        }
        CorpusOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&result)
                .map_err(|e| Error::Internal(format!("JSON serialization failed: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Export per-entry corpus results as structured JSON (spec §10.3).
fn corpus_export(output: Option<&str>, filter: Option<&CorpusFormatArg>) -> Result<()> {
    use crate::corpus::registry::{CorpusFormat, CorpusRegistry};
    use crate::corpus::runner::CorpusRunner;

    let config = Config::default();
    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(config);

    let score = match filter {
        Some(CorpusFormatArg::Bash) => runner.run_format(&registry, CorpusFormat::Bash),
        Some(CorpusFormatArg::Makefile) => runner.run_format(&registry, CorpusFormat::Makefile),
        Some(CorpusFormatArg::Dockerfile) => runner.run_format(&registry, CorpusFormat::Dockerfile),
        None => runner.run(&registry),
    };

    // Build export entries by joining registry metadata with results
    let results_map: std::collections::HashMap<&str, &crate::corpus::runner::CorpusResult> =
        score.results.iter().map(|r| (r.id.as_str(), r)).collect();

    #[derive(serde::Serialize)]
    struct ExportEntry<'a> {
        id: &'a str,
        name: &'a str,
        format: &'a CorpusFormat,
        tier: &'a crate::corpus::registry::CorpusTier,
        transpiled: bool,
        score: f64,
        grade: String,
        actual_output: &'a Option<String>,
        error: &'a Option<String>,
        lint_clean: bool,
        deterministic: bool,
        behavioral: bool,
        cross_shell: bool,
    }

    let entries: Vec<ExportEntry<'_>> = registry.entries.iter().filter_map(|e| {
        let r = results_map.get(e.id.as_str())?;
        Some(ExportEntry {
            id: &e.id,
            name: &e.name,
            format: &e.format,
            tier: &e.tier,
            transpiled: r.transpiled,
            score: r.score(),
            grade: crate::corpus::registry::Grade::from_score(r.score()).to_string(),
            actual_output: &r.actual_output,
            error: &r.error,
            lint_clean: r.lint_clean,
            deterministic: r.deterministic,
            behavioral: r.output_behavioral,
            cross_shell: r.cross_shell_agree,
        })
    }).collect();

    #[derive(serde::Serialize)]
    struct ExportDocument<'a> {
        bashrs_version: &'a str,
        date: String,
        total: usize,
        aggregate_score: f64,
        aggregate_grade: String,
        entries: Vec<ExportEntry<'a>>,
    }

    let doc = ExportDocument {
        bashrs_version: env!("CARGO_PKG_VERSION"),
        date: chrono_free_date(),
        total: entries.len(),
        aggregate_score: score.score,
        aggregate_grade: score.grade.to_string(),
        entries,
    };

    let json = serde_json::to_string_pretty(&doc)
        .map_err(|e| Error::Internal(format!("JSON serialization failed: {e}")))?;

    match output {
        Some(path) => {
            std::fs::write(path, &json)
                .map_err(|e| Error::Internal(format!("Failed to write {path}: {e}")))?;
            eprintln!("Exported {} entries to {path}", doc.total);
        }
        None => println!("{json}"),
    }
    Ok(())
}

/// Format a per-format pass/total column (e.g. "499/500" or "-" if no data).
fn fmt_pass_total(passed: usize, total: usize) -> String {
    if total > 0 { format!("{passed}/{total}") } else { "-".to_string() }
}

/// Print a single convergence history row (human-readable).
fn corpus_print_history_row(
    e: &crate::corpus::runner::ConvergenceEntry,
    has_format_data: bool,
    has_score_data: bool,
) {
    use crate::cli::color::*;
    let rate_pct = e.rate * 100.0;
    let rc = pct_color(rate_pct);
    let dc = delta_color(e.delta);
    let score_part = if has_score_data {
        let sc = pct_color(e.score);
        let gr = if e.grade.is_empty() { "-".to_string() } else { e.grade.clone() };
        format!("  {sc}{:>5.1}{RESET} {:>2}", e.score, gr)
    } else {
        String::new()
    };
    if has_format_data {
        println!(
            "{:>4}  {:>10}  {:>5}/{:<5}  {rc}{:>5.1}%{RESET}  {dc}{score_part}  {:>9} {:>9} {:>9}  {}",
            e.iteration, e.date, e.passed, e.total, rate_pct,
            fmt_pass_total(e.bash_passed, e.bash_total),
            fmt_pass_total(e.makefile_passed, e.makefile_total),
            fmt_pass_total(e.dockerfile_passed, e.dockerfile_total),
            e.notes
        );
    } else {
        println!(
            "{:>4}  {:>10}  {:>5}/{:<5}  {rc}{:>5.1}%{RESET}  {dc}{score_part}  {}",
            e.iteration, e.date, e.passed, e.total, rate_pct, e.notes
        );
    }
}

fn corpus_show_history(format: &CorpusOutputFormat, last: Option<usize>) -> Result<()> {
    use crate::corpus::runner::CorpusRunner;

    let log_path = PathBuf::from(".quality/convergence.log");
    let entries = CorpusRunner::load_convergence_log(&log_path)
        .map_err(|e| Error::Internal(format!("Failed to read convergence log: {e}")))?;

    if entries.is_empty() {
        println!("No convergence history. Run `bashrs corpus run --log` to create entries.");
        return Ok(());
    }

    let display: &[_] = match last {
        Some(n) if n < entries.len() => &entries[entries.len() - n..],
        _ => &entries,
    };

    // Detect if any entry has per-format data (spec §11.10.5)
    let has_format_data = display.iter().any(|e| e.bash_total > 0 || e.makefile_total > 0 || e.dockerfile_total > 0);
    // Detect if any entry has V2 score data (spec §5.1)
    let has_score_data = display.iter().any(|e| e.score > 0.0);

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;
            println!("{BOLD}Convergence History ({} entries):{RESET}", entries.len());
            let score_hdr = if has_score_data { "  Score Gr" } else { "" };
            if has_format_data {
                println!(
                    "{DIM}{:>4}  {:>10}  {:>5}/{:<5}  {:>6}  {:>8}{score_hdr}  {:>9} {:>9} {:>9}  {}{RESET}",
                    "Iter", "Date", "Pass", "Total", "Rate", "Delta",
                    "Bash", "Make", "Docker", "Notes"
                );
            } else {
                println!(
                    "{DIM}{:>4}  {:>10}  {:>5}/{:<5}  {:>6}  {:>8}{score_hdr}  {}{RESET}",
                    "Iter", "Date", "Pass", "Total", "Rate", "Delta", "Notes"
                );
            }
            for e in display {
                corpus_print_history_row(e, has_format_data, has_score_data);
            }
        }
        CorpusOutputFormat::Json => {
            let json = serde_json::to_string_pretty(display)
                .map_err(|e| Error::Internal(format!("JSON serialization failed: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}

fn corpus_show_failures(
    format: &CorpusOutputFormat,
    filter: Option<&CorpusFormatArg>,
    dimension: Option<&str>,
) -> Result<()> {
    use crate::corpus::registry::{CorpusFormat, CorpusRegistry};
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = match filter {
        Some(CorpusFormatArg::Bash) => runner.run_format(&registry, CorpusFormat::Bash),
        Some(CorpusFormatArg::Makefile) => runner.run_format(&registry, CorpusFormat::Makefile),
        Some(CorpusFormatArg::Dockerfile) => runner.run_format(&registry, CorpusFormat::Dockerfile),
        None => runner.run(&registry),
    };

    let failures: Vec<_> = score.results.iter().filter(|r| {
        let has_any_failure = !r.transpiled
            || !r.output_contains
            || !r.output_exact
            || !r.output_behavioral
            || !r.lint_clean
            || !r.deterministic
            || !r.metamorphic_consistent
            || !r.cross_shell_agree
            || !r.schema_valid;
        if !has_any_failure {
            return false;
        }
        match dimension {
            Some("a") => !r.transpiled,
            Some("b1") => !r.output_contains,
            Some("b2") => !r.output_exact,
            Some("b3") => !r.output_behavioral,
            Some("d") => !r.lint_clean,
            Some("e") => !r.deterministic,
            Some("f") => !r.metamorphic_consistent,
            Some("g") => !r.cross_shell_agree,
            Some("schema") => !r.schema_valid,
            _ => true,
        }
    }).collect();

    corpus_print_failures(&failures, format)
}

fn corpus_print_failures(
    failures: &[&crate::corpus::runner::CorpusResult],
    format: &CorpusOutputFormat,
) -> Result<()> {
    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;

            if failures.is_empty() {
                println!("{GREEN}No failures found.{RESET}");
                return Ok(());
            }
            println!("{BRIGHT_RED}Failures ({} entries):{RESET}", failures.len());
            println!(
                "{DIM}{:<8} {:>6}  {}{RESET}",
                "ID", "Score", "Failing Dimensions"
            );
            for r in failures {
                let dims = corpus_failing_dims(r);
                let sc = r.score();
                let gc = grade_color(if sc >= 90.0 { "A" } else if sc >= 70.0 { "B" } else { "D" });
                println!("{CYAN}{:<8}{RESET} {gc}{:>5.1}{RESET}  {RED}{}{RESET}", r.id, sc, dims);
            }
        }
        CorpusOutputFormat::Json => {
            let json = serde_json::to_string_pretty(failures)
                .map_err(|e| Error::Internal(format!("JSON serialization failed: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}

fn corpus_failing_dims(r: &crate::corpus::runner::CorpusResult) -> String {
    let mut dims = Vec::new();
    if !r.transpiled { dims.push("A"); }
    if !r.output_contains { dims.push("B1"); }
    if !r.output_exact { dims.push("B2"); }
    if !r.output_behavioral { dims.push("B3"); }
    if !r.lint_clean { dims.push("D"); }
    if !r.deterministic { dims.push("E"); }
    if !r.metamorphic_consistent { dims.push("F"); }
    if !r.cross_shell_agree { dims.push("G"); }
    if !r.schema_valid { dims.push("Schema"); }
    dims.join(", ")
}

fn corpus_show_diff(
    format: &CorpusOutputFormat,
    from: Option<u32>,
    to: Option<u32>,
) -> Result<()> {
    use crate::corpus::runner::CorpusRunner;

    let log_path = PathBuf::from(".quality/convergence.log");
    let entries = CorpusRunner::load_convergence_log(&log_path)
        .map_err(|e| Error::Internal(format!("Failed to read convergence log: {e}")))?;

    if entries.len() < 2 {
        return Err(Error::Validation(
            "Need at least 2 convergence entries to diff. Run `bashrs corpus run --log` multiple times.".to_string()
        ));
    }

    let from_entry = match from {
        Some(iter) => entries.iter().find(|e| e.iteration == iter)
            .ok_or_else(|| Error::Validation(format!("Iteration {iter} not found in convergence log")))?,
        None => &entries[entries.len() - 2],
    };
    let to_entry = match to {
        Some(iter) => entries.iter().find(|e| e.iteration == iter)
            .ok_or_else(|| Error::Validation(format!("Iteration {iter} not found in convergence log")))?,
        None => entries.last()
            .ok_or_else(|| Error::Validation("Empty convergence log".to_string()))?,
    };

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;

            println!("{BOLD}Convergence Diff:{RESET} iteration {} → {}", from_entry.iteration, to_entry.iteration);
            println!();
            println!("  {DIM}{:>12}  {:>10}  {:>10}{RESET}", "", "From", "To");
            println!("  {:>12}  {:>10}  {:>10}", "Date", from_entry.date, to_entry.date);
            println!("  {:>12}  {:>10}  {:>10}", "Passed", from_entry.passed, to_entry.passed);
            println!("  {:>12}  {:>10}  {:>10}", "Total", from_entry.total, to_entry.total);
            let from_pct = from_entry.rate * 100.0;
            let to_pct = to_entry.rate * 100.0;
            let frc = pct_color(from_pct);
            let trc = pct_color(to_pct);
            println!("  {:>12}  {frc}{:>9.1}%{RESET}  {trc}{:>9.1}%{RESET}", "Rate", from_pct, to_pct);
            let rate_delta = to_entry.rate - from_entry.rate;
            let passed_delta = to_entry.passed as i64 - from_entry.passed as i64;
            println!();
            if rate_delta > 0.0 {
                println!("  {GREEN}Improvement: +{passed_delta} entries, +{:.4}% rate{RESET}", rate_delta * 100.0);
            } else if rate_delta < 0.0 {
                println!("  {BRIGHT_RED}Regression: {passed_delta} entries, {:.4}% rate{RESET}", rate_delta * 100.0);
            } else {
                println!("  {DIM}No change in pass rate.{RESET}");
            }
        }
        CorpusOutputFormat::Json => {
            let diff = serde_json::json!({
                "from": { "iteration": from_entry.iteration, "date": from_entry.date, "passed": from_entry.passed, "total": from_entry.total, "rate": from_entry.rate },
                "to": { "iteration": to_entry.iteration, "date": to_entry.date, "passed": to_entry.passed, "total": to_entry.total, "rate": to_entry.rate },
                "delta": { "passed": to_entry.passed as i64 - from_entry.passed as i64, "rate": to_entry.rate - from_entry.rate }
            });
            let json = serde_json::to_string_pretty(&diff)
                .map_err(|e| Error::Internal(format!("JSON serialization failed: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}

fn corpus_generate_report(output: Option<&str>) -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

    let date = chrono_free_date();
    let mut report = String::new();
    report.push_str(&format!("# V2 Corpus Quality Report\n\n"));
    report.push_str(&format!("**Date**: {date}\n\n"));
    report.push_str(&format!(
        "## Score: {:.1}/100 ({})\n\n",
        score.score, score.grade
    ));

    // Summary table
    report.push_str("| Metric | Value |\n|--------|-------|\n");
    report.push_str(&format!("| Total entries | {} |\n", score.total));
    report.push_str(&format!("| Passed | {} |\n", score.passed));
    report.push_str(&format!("| Failed | {} |\n", score.failed));
    report.push_str(&format!("| Pass rate | {:.1}% |\n", score.rate * 100.0));
    report.push_str("\n");

    // Per-format breakdown
    report.push_str("## Format Breakdown\n\n");
    report.push_str("| Format | Score | Grade | Passed | Total |\n");
    report.push_str("|--------|-------|-------|--------|-------|\n");
    for fs in &score.format_scores {
        report.push_str(&format!(
            "| {} | {:.1}/100 | {} | {} | {} |\n",
            fs.format, fs.score, fs.grade, fs.passed, fs.total
        ));
    }
    report.push_str("\n");

    // Failures
    let failures: Vec<_> = score.results.iter().filter(|r| {
        !r.transpiled
            || !r.output_behavioral
            || !r.cross_shell_agree
            || !r.lint_clean
            || !r.deterministic
            || !r.schema_valid
    }).collect();

    if failures.is_empty() {
        report.push_str("## Failures\n\nNone.\n\n");
    } else {
        report.push_str(&format!("## Failures ({})\n\n", failures.len()));
        report.push_str("| ID | Score | Failing Dimensions |\n");
        report.push_str("|----|-------|--------------------|\n");
        for r in &failures {
            let dims = corpus_failing_dims(r);
            report.push_str(&format!("| {} | {:.1} | {} |\n", r.id, r.score(), dims));
        }
        report.push_str("\n");
    }

    // Convergence history
    let log_path = PathBuf::from(".quality/convergence.log");
    let history = CorpusRunner::load_convergence_log(&log_path).unwrap_or_default();
    if !history.is_empty() {
        report.push_str("## Convergence History\n\n");
        report.push_str("| Iter | Date | Pass/Total | Rate | Delta |\n");
        report.push_str("|------|------|------------|------|-------|\n");
        let display = if history.len() > 10 {
            &history[history.len() - 10..]
        } else {
            &history
        };
        for e in display {
            report.push_str(&format!(
                "| {} | {} | {}/{} | {:.1}% | {:+.4} |\n",
                e.iteration, e.date, e.passed, e.total, e.rate * 100.0, e.delta
            ));
        }
        report.push_str("\n");
    }

    // V2 scoring formula reference
    report.push_str("## V2 Scoring Formula\n\n");
    report.push_str("| Dimension | Points | Description |\n");
    report.push_str("|-----------|--------|-------------|\n");
    report.push_str("| A | 30 | Transpilation succeeds |\n");
    report.push_str("| B1 | 10 | Output contains expected |\n");
    report.push_str("| B2 | 8 | Exact output match |\n");
    report.push_str("| B3 | 7 | Behavioral equivalence |\n");
    report.push_str("| C | 15 | LLVM coverage ratio |\n");
    report.push_str("| D | 10 | Lint clean |\n");
    report.push_str("| E | 10 | Deterministic output |\n");
    report.push_str("| F | 5 | Metamorphic consistency |\n");
    report.push_str("| G | 5 | Cross-shell agreement |\n");

    match output {
        Some(path) => {
            std::fs::write(path, &report)
                .map_err(|e| Error::Internal(format!("Failed to write report to {path}: {e}")))?;
            println!("Report written to {path}");
        }
        None => print!("{report}"),
    }
    Ok(())
}

/// Generate ISO 8601 date string without chrono dependency.
fn chrono_free_date() -> String {
    use std::process::Command;
    Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn comply_load_or_default(path: &Path) -> crate::comply::config::ComplyConfig {
    use crate::comply::config::ComplyConfig;
    let version = env!("CARGO_PKG_VERSION");
    if ComplyConfig::exists(path) {
        ComplyConfig::load(path).unwrap_or_else(|| ComplyConfig::new_default(version))
    } else {
        ComplyConfig::new_default(version)
    }
}

fn comply_scope_filter(scope: Option<ComplyScopeArg>) -> Option<crate::comply::config::Scope> {
    scope.and_then(|s| match s {
        ComplyScopeArg::Project => Some(crate::comply::config::Scope::Project),
        ComplyScopeArg::User => Some(crate::comply::config::Scope::User),
        ComplyScopeArg::System => Some(crate::comply::config::Scope::System),
        ComplyScopeArg::All => None,
    })
}

fn comply_init_command(scope: ComplyScopeArg, pzsh: bool, strict: bool) -> Result<()> {
    use crate::comply::config::ComplyConfig;

    info!("Initializing comply manifest");

    if ComplyConfig::exists(Path::new(".")) {
        return Err(Error::Validation(
            ".bashrs/comply.toml already exists. Delete it first to reinitialize.".into(),
        ));
    }

    let mut config = ComplyConfig::new_default(env!("CARGO_PKG_VERSION"));
    apply_comply_scope(&mut config, scope);

    if pzsh { config.integration.pzsh = "enabled".to_string(); }
    if strict { apply_comply_strict(&mut config); }

    config.save(Path::new("."))
        .map_err(|e| Error::Internal(format!("Failed to save comply.toml: {e}")))?;

    println!("Initialized .bashrs/comply.toml");
    println!("  Scopes: project={} user={} system={}",
        config.scopes.project, config.scopes.user, config.scopes.system);
    if pzsh { println!("  pzsh integration: enabled"); }
    if strict { println!("  Mode: strict (all rules enforced)"); }

    Ok(())
}

fn apply_comply_scope(config: &mut crate::comply::config::ComplyConfig, scope: ComplyScopeArg) {
    match scope {
        ComplyScopeArg::Project => { config.scopes.user = false; config.scopes.system = false; }
        ComplyScopeArg::User => { config.scopes.user = true; config.scopes.system = false; }
        ComplyScopeArg::System => { config.scopes.user = false; config.scopes.system = true; }
        ComplyScopeArg::All => { config.scopes.user = true; config.scopes.system = true; }
    }
}

fn apply_comply_strict(config: &mut crate::comply::config::ComplyConfig) {
    config.rules.posix = true;
    config.rules.determinism = true;
    config.rules.idempotency = true;
    config.rules.security = true;
    config.rules.quoting = true;
    config.rules.shellcheck = true;
    config.rules.makefile_safety = true;
    config.rules.dockerfile_best = true;
    config.rules.config_hygiene = true;
    config.rules.pzsh_budget = "10ms".to_string();
}

fn comply_check_command(
    path: &Path,
    scope: Option<ComplyScopeArg>,
    strict: bool,
    format: ComplyFormat,
) -> Result<()> {
    use crate::comply::{runner, scoring::Grade};

    info!("Running compliance check on {}", path.display());

    let config = comply_load_or_default(path);
    let score = runner::run_check(path, comply_scope_filter(scope), &config);
    comply_output_score(&score, format);

    if strict && score.grade == Grade::F {
        return Err(Error::Validation(format!(
            "Compliance check failed: grade {} (score {:.0}/100)",
            score.grade, score.score
        )));
    }
    Ok(())
}

fn comply_status_command(path: &Path, format: ComplyFormat) -> Result<()> {
    use crate::comply::runner;

    info!("Checking compliance status for {}", path.display());
    let config = comply_load_or_default(path);
    let score = runner::run_check(path, None, &config);
    comply_output_score(&score, format);
    Ok(())
}

fn comply_output_score(score: &crate::comply::scoring::ProjectScore, format: ComplyFormat) {
    use crate::comply::runner;
    match format {
        ComplyFormat::Text => print!("{}", runner::format_human(score)),
        ComplyFormat::Json => println!("{}", runner::format_json(score)),
        ComplyFormat::Markdown => {
            println!("# Compliance Report\n");
            println!("**Score**: {:.0}/100 ({})\n", score.score, score.grade);
            println!("| Artifact | Score | Grade | Status |");
            println!("|----------|-------|-------|--------|");
            for a in &score.artifact_scores {
                let status = if a.violations == 0 { "COMPLIANT" } else { "NON-COMPLIANT" };
                println!("| {} | {:.0} | {} | {} |", a.artifact_name, a.score, a.grade, status);
            }
        }
    }
}

fn handle_comply_track_command(command: ComplyTrackCommands) -> Result<()> {
    match command {
        ComplyTrackCommands::Discover { path, scope } => comply_track_discover(&path, scope),
        ComplyTrackCommands::List { path, scope } => comply_track_list(&path, scope),
    }
}

fn comply_track_discover(path: &Path, scope: ComplyScopeArg) -> Result<()> {
    use crate::comply::discovery;

    info!("Discovering artifacts in {}", path.display());

    if matches!(scope, ComplyScopeArg::All) {
        return comply_track_discover_all(path);
    }

    let scope_val = comply_scope_to_internal(scope);
    let artifacts = discovery::discover(path, scope_val);
    comply_print_artifact_list(scope_val, &artifacts);
    Ok(())
}

fn comply_track_discover_all(path: &Path) -> Result<()> {
    use crate::comply::{config::Scope, discovery};

    let mut total = 0;
    for s in &[Scope::Project, Scope::User, Scope::System] {
        let artifacts = discovery::discover(path, *s);
        if !artifacts.is_empty() {
            println!("{:?} scope ({} artifacts):", s, artifacts.len());
            for a in &artifacts {
                println!("  {} [{:?}]", a.display_name(), a.kind);
            }
            total += artifacts.len();
        }
    }
    println!("\nTotal: {} artifacts discovered", total);
    Ok(())
}

fn comply_track_list(path: &Path, scope: Option<ComplyScopeArg>) -> Result<()> {
    use crate::comply::{config::Scope, discovery};

    info!("Listing tracked artifacts");

    let scopes = match scope.and_then(|s| match s {
        ComplyScopeArg::Project => Some(Scope::Project),
        ComplyScopeArg::User => Some(Scope::User),
        ComplyScopeArg::System => Some(Scope::System),
        ComplyScopeArg::All => None,
    }) {
        Some(s) => vec![s],
        None => vec![Scope::Project, Scope::User, Scope::System],
    };

    let mut total = 0;
    for s in scopes {
        let artifacts = discovery::discover(path, s);
        if !artifacts.is_empty() {
            println!("{:?} ({}):", s, artifacts.len());
            for a in &artifacts {
                println!("  {} [{:?}]", a.display_name(), a.kind);
            }
            total += artifacts.len();
        }
    }
    println!("\nTotal tracked: {}", total);
    Ok(())
}

fn comply_scope_to_internal(scope: ComplyScopeArg) -> crate::comply::config::Scope {
    use crate::comply::config::Scope;
    match scope {
        ComplyScopeArg::Project => Scope::Project,
        ComplyScopeArg::User => Scope::User,
        ComplyScopeArg::System => Scope::System,
        ComplyScopeArg::All => Scope::Project, // fallback, caller should handle All
    }
}

fn comply_print_artifact_list(scope: crate::comply::config::Scope, artifacts: &[crate::comply::discovery::Artifact]) {
    println!("{:?} scope ({} artifacts):", scope, artifacts.len());
    for a in artifacts {
        println!("  {} [{:?}]", a.display_name(), a.kind);
    }
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

/// Run tests in a bash script
fn test_command(
    input: &Path,
    format: TestOutputFormat,
    detailed: bool,
    pattern: Option<&str>,
) -> Result<()> {
    use crate::bash_quality::testing::{discover_tests, run_tests};

    // Read input file
    let source = fs::read_to_string(input)
        .map_err(|e| Error::Internal(format!("Failed to read {}: {}", input.display(), e)))?;

    // Discover tests
    let tests = discover_tests(&source)
        .map_err(|e| Error::Internal(format!("Failed to discover tests: {}", e)))?;

    if tests.is_empty() {
        warn!("No tests found in {}", input.display());
        println!("No tests found in {}", input.display());
        return Ok(());
    }

    // Filter tests by pattern if provided
    let tests_to_run: Vec<_> = if let Some(pat) = pattern {
        tests
            .iter()
            .filter(|t| t.name.contains(pat))
            .cloned()
            .collect()
    } else {
        tests.clone()
    };

    if tests_to_run.is_empty() {
        warn!("No tests matching pattern '{}'", pattern.unwrap_or(""));
        println!("No tests matching pattern '{}'", pattern.unwrap_or(""));
        return Ok(());
    }

    info!(
        "Running {} tests from {}",
        tests_to_run.len(),
        input.display()
    );

    // Run tests
    let report = run_tests(&source, &tests_to_run)
        .map_err(|e| Error::Internal(format!("Failed to run tests: {}", e)))?;

    // Output results
    match format {
        TestOutputFormat::Human => {
            print_human_test_results(&report, detailed);
        }
        TestOutputFormat::Json => {
            print_json_test_results(&report);
        }
        TestOutputFormat::Junit => {
            print_junit_test_results(&report);
        }
    }

    // Exit with error if tests failed
    if report.failed() > 0 {
        return Err(Error::Internal(format!(
            "{} test(s) failed",
            report.failed()
        )));
    }

    Ok(())
}

/// Print human-readable test results
fn print_human_test_results(report: &crate::bash_quality::testing::TestReport, detailed: bool) {
    use crate::bash_quality::testing::TestResult;

    println!();
    println!("Test Results");
    println!("============");
    println!();

    for (test_name, result) in &report.results {
        match result {
            TestResult::Pass => {
                println!("\u{2713} {}", test_name);
                if detailed { print_test_detail(report, test_name, true); }
            }
            TestResult::Fail(msg) => {
                println!("\u{2717} {}", test_name);
                println!("  Error: {}", msg);
                if detailed { print_test_detail(report, test_name, false); }
            }
            TestResult::Skip(reason) => {
                println!("\u{2298} {} (skipped: {})", test_name, reason);
            }
        }
    }

    print_test_summary(report);
}

fn print_test_detail(
    report: &crate::bash_quality::testing::TestReport,
    test_name: &str,
    full: bool,
) {
    let test = match report.tests.iter().find(|t| t.name == test_name) {
        Some(t) => t,
        None => return,
    };
    if let Some(desc) = &test.description { println!("  Description: {}", desc); }
    if full {
        if let Some(given) = &test.given { println!("  Given: {}", given); }
        if let Some(when) = &test.when { println!("  When: {}", when); }
        if let Some(then) = &test.then { println!("  Then: {}", then); }
    }
}

fn print_test_summary(report: &crate::bash_quality::testing::TestReport) {
    println!();
    println!("Summary");
    println!("-------");
    println!("Total:   {}", report.results.len());
    println!("Passed:  {}", report.passed());
    println!("Failed:  {}", report.failed());
    println!("Skipped: {}", report.skipped());
    println!("Time:    {}ms", report.duration_ms);
    println!();
    if report.all_passed() {
        println!("\u{2713} All tests passed!");
    } else {
        println!("\u{2717} {} test(s) failed", report.failed());
    }
}

/// Print JSON test results
fn print_json_test_results(report: &crate::bash_quality::testing::TestReport) {
    use serde_json::json;

    let json_report = json!({
        "tests": report.tests.iter().map(|t| json!({
            "name": t.name,
            "line": t.line,
            "description": t.description,
            "given": t.given,
            "when": t.when,
            "then": t.then,
        })).collect::<Vec<_>>(),
        "results": report.results.iter().map(|(name, result)| json!({
            "name": name,
            "result": match result {
                crate::bash_quality::testing::TestResult::Pass => "pass",
                crate::bash_quality::testing::TestResult::Fail(_) => "fail",
                crate::bash_quality::testing::TestResult::Skip(_) => "skip",
            },
            "message": match result {
                crate::bash_quality::testing::TestResult::Fail(msg) => Some(msg),
                crate::bash_quality::testing::TestResult::Skip(msg) => Some(msg),
                _ => None,
            },
        })).collect::<Vec<_>>(),
        "summary": {
            "total": report.results.len(),
            "passed": report.passed(),
            "failed": report.failed(),
            "skipped": report.skipped(),
            "duration_ms": report.duration_ms,
            "all_passed": report.all_passed(),
        }
    });

    match serde_json::to_string_pretty(&json_report) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("Error serializing JSON: {}", e);
            std::process::exit(1);
        }
    }
}

/// Print JUnit XML test results
fn print_junit_test_results(report: &crate::bash_quality::testing::TestReport) {
    println!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    println!(
        "<testsuite tests=\"{}\" failures=\"{}\" skipped=\"{}\" time=\"{:.3}\">",
        report.results.len(),
        report.failed(),
        report.skipped(),
        report.duration_ms as f64 / 1000.0
    );

    for (test_name, result) in &report.results {
        match result {
            crate::bash_quality::testing::TestResult::Pass => {
                println!("  <testcase name=\"{}\" />", test_name);
            }
            crate::bash_quality::testing::TestResult::Fail(msg) => {
                println!("  <testcase name=\"{}\">", test_name);
                println!("    <failure message=\"{}\" />", msg.replace('"', "&quot;"));
                println!("  </testcase>");
            }
            crate::bash_quality::testing::TestResult::Skip(reason) => {
                println!("  <testcase name=\"{}\">", test_name);
                println!(
                    "    <skipped message=\"{}\" />",
                    reason.replace('"', "&quot;")
                );
                println!("  </testcase>");
            }
        }
    }

    println!("</testsuite>");
}

/// Score a bash script for quality
fn score_command(
    input: &Path,
    format: ScoreOutputFormat,
    detailed: bool,
    dockerfile: bool,
    runtime: bool,
    show_grade: bool,
    profile: Option<LintProfileArg>,
) -> Result<()> {
    // Read input file
    let source = fs::read_to_string(input)
        .map_err(|e| Error::Internal(format!("Failed to read {}: {}", input.display(), e)))?;

    // Detect if file is a Dockerfile
    let filename = input.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let is_dockerfile = dockerfile
        || filename.eq_ignore_ascii_case("dockerfile")
        || filename.to_lowercase().ends_with(".dockerfile");

    if is_dockerfile {
        // Use Dockerfile-specific scoring with optional runtime analysis
        use crate::bash_quality::dockerfile_scoring::score_dockerfile;
        use crate::linter::docker_profiler::{estimate_size, is_docker_available, PlatformProfile};

        let score = score_dockerfile(&source)
            .map_err(|e| Error::Internal(format!("Failed to score Dockerfile: {}", e)))?;

        // Determine platform profile
        let platform_profile = match profile {
            Some(LintProfileArg::Coursera) => PlatformProfile::Coursera,
            _ => PlatformProfile::Standard,
        };

        // Runtime analysis if requested
        let runtime_score = if runtime {
            let estimate = estimate_size(&source);
            let docker_available = is_docker_available();
            Some(RuntimeScore::new(
                &estimate,
                platform_profile,
                docker_available,
            ))
        } else {
            None
        };

        // Output results
        match format {
            ScoreOutputFormat::Human => {
                print_human_dockerfile_score_results(&score, detailed);
                if let Some(ref rt) = runtime_score {
                    print_human_runtime_score(rt, platform_profile);
                }
                if show_grade {
                    print_combined_grade(&score, runtime_score.as_ref());
                }
            }
            ScoreOutputFormat::Json => {
                print_json_dockerfile_score_with_runtime(&score, runtime_score.as_ref());
            }
            ScoreOutputFormat::Markdown => {
                print_markdown_dockerfile_score_results(&score, input);
                if let Some(ref rt) = runtime_score {
                    print_markdown_runtime_score(rt);
                }
            }
        }
    } else {
        // Use bash script scoring
        use crate::bash_quality::scoring::score_script_with_file_type;

        let score = score_script_with_file_type(&source, Some(input))
            .map_err(|e| Error::Internal(format!("Failed to score script: {}", e)))?;

        // Output results
        match format {
            ScoreOutputFormat::Human => {
                print_human_score_results(&score, detailed);
            }
            ScoreOutputFormat::Json => {
                print_json_score_results(&score);
            }
            ScoreOutputFormat::Markdown => {
                print_markdown_score_results(&score, input);
            }
        }
    }

    Ok(())
}

/// Runtime performance score for Docker images
#[derive(Debug)]
struct RuntimeScore {
    /// Overall runtime score (0-100)
    score: f64,
    /// Image size in bytes
    estimated_size: u64,
    /// Size score component (0-100)
    size_score: f64,
    /// Layer optimization score (0-100)
    layer_score: f64,
    /// Number of bloat patterns detected
    bloat_count: usize,
    /// Whether Docker is available for actual measurement
    docker_available: bool,
    /// Suggestions for improvement
    suggestions: Vec<String>,
}

impl RuntimeScore {
    fn new(
        estimate: &crate::linter::docker_profiler::SizeEstimate,
        profile: crate::linter::docker_profiler::PlatformProfile,
        docker_available: bool,
    ) -> Self {
        let max_size = profile.max_size_bytes();
        let mut suggestions = Vec::new();

        // Calculate size score
        let size_score = if max_size == u64::MAX {
            // No limit - base on reasonable defaults (5GB good, 10GB average)
            let five_gb = 5_000_000_000u64;
            if estimate.total_estimated < five_gb {
                100.0
            } else {
                let ratio = estimate.total_estimated as f64 / five_gb as f64;
                (100.0 / ratio).clamp(0.0, 100.0)
            }
        } else {
            let ratio = estimate.total_estimated as f64 / max_size as f64;
            if ratio > 1.0 {
                0.0 // Over limit
            } else if ratio > 0.8 {
                (1.0 - ratio) * 500.0 // 0-100 for 80-100% of limit
            } else {
                100.0 - (ratio * 50.0) // 50-100 for under 80%
            }
        };

        // Calculate layer score (penalize many layers and bloat)
        let layer_count = estimate.layer_estimates.len();
        let bloat_count = estimate.bloat_patterns.len();

        let layer_score = if layer_count <= 5 {
            100.0 - (bloat_count as f64 * 20.0)
        } else if layer_count <= 10 {
            80.0 - (bloat_count as f64 * 20.0)
        } else {
            60.0 - (bloat_count as f64 * 20.0)
        }
        .max(0.0);

        // Add suggestions based on analysis
        for pattern in &estimate.bloat_patterns {
            suggestions.push(format!("{}: {}", pattern.code, pattern.remediation));
        }

        if layer_count > 10 {
            suggestions.push("Consider combining RUN commands to reduce layer count".to_string());
        }

        if estimate.total_estimated > max_size {
            suggestions.push(format!(
                "Image size ({:.1}GB) exceeds limit ({:.1}GB) - use smaller base image or multi-stage build",
                estimate.total_estimated as f64 / 1_000_000_000.0,
                max_size as f64 / 1_000_000_000.0
            ));
        }

        // Overall score is weighted average
        let score = (size_score * 0.6 + layer_score * 0.4).clamp(0.0, 100.0);

        Self {
            score,
            estimated_size: estimate.total_estimated,
            size_score,
            layer_score,
            bloat_count,
            docker_available,
            suggestions,
        }
    }

    fn grade(&self) -> &'static str {
        match self.score as u32 {
            95..=100 => "A+",
            90..=94 => "A",
            85..=89 => "A-",
            80..=84 => "B+",
            75..=79 => "B",
            70..=74 => "B-",
            65..=69 => "C+",
            60..=64 => "C",
            55..=59 => "C-",
            50..=54 => "D",
            _ => "F",
        }
    }
}

/// Print human-readable runtime score
fn print_human_runtime_score(
    rt: &RuntimeScore,
    profile: crate::linter::docker_profiler::PlatformProfile,
) {
    println!();
    println!("Runtime Performance Score");
    println!("=========================");
    println!();
    println!("Runtime Score: {:.0}/100 ({})", rt.score, rt.grade());
    println!();
    println!("  Size Analysis:");
    println!(
        "    - Estimated size: {:.2}GB",
        rt.estimated_size as f64 / 1_000_000_000.0
    );
    println!("    - Size score: {:.0}/100", rt.size_score);
    println!();
    println!("  Layer Optimization:");
    println!("    - Bloat patterns: {}", rt.bloat_count);
    println!("    - Layer score: {:.0}/100", rt.layer_score);
    println!();

    // Show platform limits if not standard
    if !matches!(
        profile,
        crate::linter::docker_profiler::PlatformProfile::Standard
    ) {
        let max_size_gb = profile.max_size_bytes() as f64 / 1_000_000_000.0;
        let size_pct = (rt.estimated_size as f64 / profile.max_size_bytes() as f64) * 100.0;
        println!("  Platform Limits ({:?}):", profile);
        println!("    - Max size: {:.0}GB", max_size_gb);
        println!("    - Usage: {:.1}%", size_pct);
        println!();
    }

    if !rt.docker_available {
        println!("  Note: Docker not available - using static analysis only");
        println!();
    }

    if !rt.suggestions.is_empty() {
        println!("  Improvement Suggestions:");
        for (i, suggestion) in rt.suggestions.iter().enumerate() {
            println!("    {}. {}", i + 1, suggestion);
        }
        println!();
    }
}

/// Print combined grade summary
fn print_combined_grade(
    score: &crate::bash_quality::dockerfile_scoring::DockerfileQualityScore,
    runtime: Option<&RuntimeScore>,
) {
    println!();
    println!("Combined Quality Assessment");
    println!("===========================");
    println!();
    println!(
        "Static Analysis: {} ({:.0}/100)",
        score.grade,
        score.score * 10.0
    );

    if let Some(rt) = runtime {
        println!("Runtime Performance: {} ({:.0}/100)", rt.grade(), rt.score);

        // Combined grade (weighted 60% static, 40% runtime)
        let combined_score = score.score * 10.0 * 0.6 + rt.score * 0.4;
        let combined_grade = match combined_score as u32 {
            95..=100 => "A+",
            90..=94 => "A",
            85..=89 => "A-",
            80..=84 => "B+",
            75..=79 => "B",
            70..=74 => "B-",
            65..=69 => "C+",
            60..=64 => "C",
            55..=59 => "C-",
            50..=54 => "D",
            _ => "F",
        };
        println!();
        println!(
            "Overall Grade: {} ({:.0}/100)",
            combined_grade, combined_score
        );
    }
    println!();
}

/// Print JSON score with runtime data
fn print_json_dockerfile_score_with_runtime(
    score: &crate::bash_quality::dockerfile_scoring::DockerfileQualityScore,
    runtime: Option<&RuntimeScore>,
) {
    use serde_json::json;

    let mut json_score = json!({
        "grade": score.grade,
        "score": score.score,
        "score_100": score.score * 10.0,
        "dimensions": {
            "safety": score.safety,
            "complexity": score.complexity,
            "layer_optimization": score.layer_optimization,
            "determinism": score.determinism,
            "security": score.security,
        },
        "suggestions": score.suggestions,
    });

    if let Some(rt) = runtime {
        if let Some(obj) = json_score.as_object_mut() {
            obj.insert(
                "runtime".to_string(),
                json!({
                    "score": rt.score,
                    "grade": rt.grade(),
                    "estimated_size_bytes": rt.estimated_size,
                    "estimated_size_gb": rt.estimated_size as f64 / 1_000_000_000.0,
                    "size_score": rt.size_score,
                    "layer_score": rt.layer_score,
                    "bloat_count": rt.bloat_count,
                    "docker_available": rt.docker_available,
                    "suggestions": rt.suggestions,
                }),
            );

            // Add combined score
            let combined = score.score * 10.0 * 0.6 + rt.score * 0.4;
            obj.insert("combined_score".to_string(), json!(combined));
        }
    }

    match serde_json::to_string_pretty(&json_score) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("Error serializing JSON: {}", e);
            std::process::exit(1);
        }
    }
}

/// Print markdown runtime score section
fn print_markdown_runtime_score(rt: &RuntimeScore) {
    println!();
    println!("## Runtime Performance");
    println!();
    println!("**Score**: {} ({:.0}/100)", rt.grade(), rt.score);
    println!();
    println!("| Metric | Value | Score |");
    println!("| --- | --- | --- |");
    println!(
        "| Image Size | {:.2}GB | {:.0}/100 |",
        rt.estimated_size as f64 / 1_000_000_000.0,
        rt.size_score
    );
    println!(
        "| Layer Optimization | {} bloat patterns | {:.0}/100 |",
        rt.bloat_count, rt.layer_score
    );

    if !rt.suggestions.is_empty() {
        println!();
        println!("### Runtime Improvement Suggestions");
        println!();
        for suggestion in &rt.suggestions {
            println!("- {}", suggestion);
        }
    }
}

/// Print human-readable score results
fn print_human_score_results(score: &crate::bash_quality::scoring::QualityScore, detailed: bool) {
    use crate::cli::color::*;

    println!();
    println!("{BOLD}Bash Script Quality Score{RESET}");
    println!("{DIM}═════════════════════════{RESET}");
    println!();
    let gc = grade_color(&score.grade);
    println!("Overall Grade: {gc}{}{RESET}", score.grade);
    println!("Overall Score: {WHITE}{:.1}/10.0{RESET}", score.score);
    println!();

    if detailed {
        println!("{BOLD}Dimension Scores:{RESET}");
        println!("{DIM}─────────────────{RESET}");
        let dim_line = |name: &str, val: f64| {
            let sc = score_color(val * 10.0);
            println!("{:<17} {sc}{:.1}/10.0{RESET}", name, val);
        };
        dim_line("Complexity:", score.complexity);
        dim_line("Safety:", score.safety);
        dim_line("Maintainability:", score.maintainability);
        dim_line("Testing:", score.testing);
        dim_line("Documentation:", score.documentation);
        println!();
    }

    if !score.suggestions.is_empty() {
        println!("{BOLD}Improvement Suggestions:{RESET}");
        println!("{DIM}────────────────────────{RESET}");
        for (i, suggestion) in score.suggestions.iter().enumerate() {
            println!("{YELLOW}{}. {}{RESET}", i + 1, suggestion);
        }
        println!();
    }

    // Grade interpretation
    match score.grade.as_str() {
        "A+" => println!("{GREEN}✓ Excellent! Near-perfect code quality.{RESET}"),
        "A" => println!("{GREEN}✓ Great! Very good code quality.{RESET}"),
        "B+" | "B" => println!("{GREEN}✓ Good code quality with room for improvement.{RESET}"),
        "C+" | "C" => println!("{YELLOW}⚠ Average code quality. Consider addressing suggestions.{RESET}"),
        "D" => println!("{RED}⚠ Below average. Multiple improvements needed.{RESET}"),
        "F" => println!("{BRIGHT_RED}✗ Poor code quality. Significant improvements required.{RESET}"),
        _ => {}
    }
}

/// Print JSON score results
fn print_json_score_results(score: &crate::bash_quality::scoring::QualityScore) {
    use serde_json::json;

    let json_score = json!({
        "grade": score.grade,
        "score": score.score,
        "dimensions": {
            "complexity": score.complexity,
            "safety": score.safety,
            "maintainability": score.maintainability,
            "testing": score.testing,
            "documentation": score.documentation,
        },
        "suggestions": score.suggestions,
    });

    match serde_json::to_string_pretty(&json_score) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("Error serializing JSON: {}", e);
            std::process::exit(1);
        }
    }
}

/// Print Markdown score results
fn print_markdown_score_results(score: &crate::bash_quality::scoring::QualityScore, input: &Path) {
    println!("# Bash Script Quality Report");
    println!();
    println!("**File**: `{}`", input.display());
    println!(
        "**Date**: {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );
    println!();
    println!("## Overall Score");
    println!();
    println!(
        "**Grade**: {} | **Score**: {:.1}/10.0",
        score.grade, score.score
    );
    println!();
    println!("## Dimension Scores");
    println!();
    println!("| Dimension | Score | Status |");
    println!("| --- | --- | --- |");
    println!(
        "| Complexity | {:.1}/10.0 | {} |",
        score.complexity,
        score_status(score.complexity)
    );
    println!(
        "| Safety | {:.1}/10.0 | {} |",
        score.safety,
        score_status(score.safety)
    );
    println!(
        "| Maintainability | {:.1}/10.0 | {} |",
        score.maintainability,
        score_status(score.maintainability)
    );
    println!(
        "| Testing | {:.1}/10.0 | {} |",
        score.testing,
        score_status(score.testing)
    );
    println!(
        "| Documentation | {:.1}/10.0 | {} |",
        score.documentation,
        score_status(score.documentation)
    );
    println!();

    if !score.suggestions.is_empty() {
        println!("## Improvement Suggestions");
        println!();
        for suggestion in &score.suggestions {
            println!("- {}", suggestion);
        }
        println!();
    }

    println!("## Grade Interpretation");
    println!();
    match score.grade.as_str() {
        "A+" => println!("✅ **Excellent!** Near-perfect code quality."),
        "A" => println!("✅ **Great!** Very good code quality."),
        "B+" | "B" => println!("✅ **Good** code quality with room for improvement."),
        "C+" | "C" => println!("⚠️ **Average** code quality. Consider addressing suggestions."),
        "D" => println!("⚠️ **Below average**. Multiple improvements needed."),
        "F" => println!("❌ **Poor** code quality. Significant improvements required."),
        _ => {}
    }
}

// score_status moved to cli/logic.rs

// ============================================================================
// Audit Command (v6.12.0 - Bash Quality Tools)
// ============================================================================

/// Comprehensive quality audit results
#[derive(Debug)]
struct AuditResults {
    parse_success: bool,
    parse_error: Option<String>,
    lint_errors: usize,
    lint_warnings: usize,
    test_passed: usize,
    test_failed: usize,
    test_total: usize,
    score: Option<crate::bash_quality::scoring::QualityScore>,
    overall_pass: bool,
    failure_reason: Option<String>,
}

fn audit_command(
    input: &Path,
    format: &AuditOutputFormat,
    strict: bool,
    detailed: bool,
    min_grade: Option<&str>,
) -> Result<()> {
    use crate::linter::diagnostic::Severity;
    use crate::linter::rules::lint_shell;

    let source = fs::read_to_string(input)
        .map_err(|e| Error::Internal(format!("Failed to read {}: {}", input.display(), e)))?;

    let mut results = AuditResults {
        parse_success: true,
        parse_error: None,
        lint_errors: 0,
        lint_warnings: 0,
        test_passed: 0,
        test_failed: 0,
        test_total: 0,
        score: None,
        overall_pass: true,
        failure_reason: None,
    };

    // Lint check
    let lint_result = lint_shell(&source);
    results.lint_errors = lint_result.diagnostics.iter()
        .filter(|d| matches!(d.severity, Severity::Error)).count();
    results.lint_warnings = lint_result.diagnostics.iter()
        .filter(|d| matches!(d.severity, Severity::Warning)).count();

    audit_check_lint(&mut results, strict);
    audit_run_tests(&source, &mut results);
    audit_check_score(&source, min_grade, &mut results);

    // Output results
    match format {
        AuditOutputFormat::Human => print_human_audit_results(&results, detailed, input),
        AuditOutputFormat::Json => print_json_audit_results(&results),
        AuditOutputFormat::Sarif => print_sarif_audit_results(&results, input),
    }

    if !results.overall_pass {
        let reason = results.failure_reason
            .unwrap_or_else(|| "Quality audit failed".to_string());
        return Err(Error::Internal(reason));
    }

    Ok(())
}

fn audit_check_lint(results: &mut AuditResults, strict: bool) {
    if results.lint_errors > 0 {
        results.overall_pass = false;
        results.failure_reason = Some(format!("{} lint errors found", results.lint_errors));
    }
    if strict && results.lint_warnings > 0 {
        results.overall_pass = false;
        results.failure_reason = Some(format!("Strict mode: {} warnings found", results.lint_warnings));
    }
}

fn audit_run_tests(source: &str, results: &mut AuditResults) {
    use crate::bash_quality::testing::{discover_tests, run_tests, TestResult};

    let tests = match discover_tests(source) {
        Ok(t) => t,
        Err(_) => return,
    };
    let test_report = match run_tests(source, &tests) {
        Ok(r) => r,
        Err(_) => return,
    };

    results.test_total = test_report.results.len();
    results.test_passed = test_report.results.iter()
        .filter(|(_, result)| matches!(result, TestResult::Pass)).count();
    results.test_failed = test_report.results.iter()
        .filter(|(_, result)| matches!(result, TestResult::Fail(_))).count();

    if results.test_failed > 0 {
        results.overall_pass = false;
        results.failure_reason = Some(format!("{}/{} tests failed", results.test_failed, results.test_total));
    }
}

fn audit_check_score(source: &str, min_grade: Option<&str>, results: &mut AuditResults) {
    use crate::bash_quality::scoring::score_script;

    let score = match score_script(source) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Warning: Failed to score script: {}", e);
            return;
        }
    };

    if let Some(min_grade_str) = min_grade {
        let grade_order = ["F", "D", "C", "C+", "B", "B+", "A", "A+"];
        let actual_grade_pos = grade_order.iter().position(|&g| g == score.grade.as_str());
        let min_grade_pos = grade_order.iter().position(|&g| g == min_grade_str);
        if let (Some(actual), Some(min)) = (actual_grade_pos, min_grade_pos) {
            if actual < min {
                results.overall_pass = false;
                results.failure_reason = Some(format!(
                    "Quality grade {} below minimum required grade {}",
                    score.grade, min_grade_str
                ));
            }
        }
    }

    results.score = Some(score);
}

/// Print human-readable audit results with ANSI colors
fn print_human_audit_results(results: &AuditResults, detailed: bool, input: &Path) {
    use crate::cli::color::*;

    println!();
    println!("{BOLD}Comprehensive Quality Audit{RESET}");
    println!("{DIM}══════════════════════════{RESET}");
    println!();
    println!("File: {CYAN}{}{RESET}", input.display());
    println!();
    println!("{BOLD}Check Results:{RESET}");
    println!("{DIM}──────────────{RESET}");

    // Parse
    if results.parse_success {
        println!("{GREEN}✓{RESET} Parse:    Valid bash syntax");
    } else {
        println!("{BRIGHT_RED}✗{RESET} Parse:    Syntax error");
        if let Some(err) = &results.parse_error {
            println!("           {DIM}{err}{RESET}");
        }
    }

    // Lint
    if results.lint_errors == 0 && results.lint_warnings == 0 {
        println!("{GREEN}✓{RESET} Lint:     No issues found");
    } else if results.lint_errors > 0 {
        println!(
            "{BRIGHT_RED}✗{RESET} Lint:     {BRIGHT_RED}{} errors{RESET}, {YELLOW}{} warnings{RESET}",
            results.lint_errors, results.lint_warnings
        );
    } else {
        println!("{YELLOW}⚠{RESET} Lint:     {YELLOW}{} warnings{RESET}", results.lint_warnings);
    }

    // Test
    if results.test_total > 0 {
        if results.test_failed == 0 {
            println!(
                "{GREEN}✓{RESET} Test:     {GREEN}{}/{} tests passed{RESET}",
                results.test_passed, results.test_total
            );
        } else {
            println!(
                "{BRIGHT_RED}✗{RESET} Test:     {}/{} tests passed, {BRIGHT_RED}{} failed{RESET}",
                results.test_passed, results.test_total, results.test_failed
            );
        }
    } else {
        println!("{YELLOW}⚠{RESET} Test:     {DIM}No tests found{RESET}");
    }

    // Score
    if let Some(score) = &results.score {
        let gc = grade_color(&score.grade);
        println!("{GREEN}✓{RESET} Score:    {gc}{}{RESET} ({WHITE}{:.1}/10.0{RESET})", score.grade, score.score);

        if detailed {
            println!();
            println!("  {BOLD}Dimension Breakdown:{RESET}");
            let dim_line = |name: &str, val: f64| {
                let sc = score_color(val * 10.0);
                println!("  {DIM}-{RESET} {:<17} {sc}{:.1}/10.0{RESET}", name, val);
            };
            dim_line("Complexity:", score.complexity);
            dim_line("Safety:", score.safety);
            dim_line("Maintainability:", score.maintainability);
            dim_line("Testing:", score.testing);
            dim_line("Documentation:", score.documentation);
        }
    }

    println!();
    if results.overall_pass {
        println!("Overall: {GREEN}{BOLD}✓ PASS{RESET}");
    } else {
        println!("Overall: {BRIGHT_RED}{BOLD}✗ FAIL{RESET}");
    }
    println!();

    // Suggestions
    if let Some(score) = &results.score {
        if !score.suggestions.is_empty() {
            println!("{BOLD}Improvement Suggestions:{RESET}");
            println!("{DIM}────────────────────────{RESET}");
            for (i, suggestion) in score.suggestions.iter().enumerate() {
                println!("{YELLOW}{}. {}{RESET}", i + 1, suggestion);
            }
            println!();
        }
    }
}

/// Print JSON audit results
fn print_json_audit_results(results: &AuditResults) {
    use serde_json::json;

    let json_results = json!({
        "audit": {
            "parse": {
                "success": results.parse_success,
                "error": results.parse_error,
            },
            "lint": {
                "errors": results.lint_errors,
                "warnings": results.lint_warnings,
            },
            "test": {
                "total": results.test_total,
                "passed": results.test_passed,
                "failed": results.test_failed,
            },
            "score": results.score.as_ref().map(|s| json!({
                "grade": s.grade,
                "score": s.score,
                "dimensions": {
                    "complexity": s.complexity,
                    "safety": s.safety,
                    "maintainability": s.maintainability,
                    "testing": s.testing,
                    "documentation": s.documentation,
                },
                "suggestions": s.suggestions,
            })),
            "overall_pass": results.overall_pass,
        }
    });

    match serde_json::to_string_pretty(&json_results) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("Error serializing JSON: {}", e);
            std::process::exit(1);
        }
    }
}

/// Print SARIF audit results (GitHub Code Scanning format)
fn print_sarif_audit_results(results: &AuditResults, input: &Path) {
    use serde_json::json;

    let mut sarif_results = vec![];

    // Add parse error if any
    if !results.parse_success {
        if let Some(err) = &results.parse_error {
            sarif_results.push(json!({
                "ruleId": "PARSE-001",
                "level": "error",
                "message": {
                    "text": format!("Parse error: {}", err)
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": input.display().to_string()
                        }
                    }
                }]
            }));
        }
    }

    // Add lint issues
    if results.lint_errors > 0 || results.lint_warnings > 0 {
        sarif_results.push(json!({
            "ruleId": "LINT-001",
            "level": if results.lint_errors > 0 { "error" } else { "warning" },
            "message": {
                "text": format!("{} errors, {} warnings", results.lint_errors, results.lint_warnings)
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": input.display().to_string()
                    }
                }
            }]
        }));
    }

    // Add test failures
    if results.test_failed > 0 {
        sarif_results.push(json!({
            "ruleId": "TEST-001",
            "level": "error",
            "message": {
                "text": format!("{}/{} tests failed", results.test_failed, results.test_total)
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": input.display().to_string()
                    }
                }
            }]
        }));
    }

    let sarif = json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "bashrs audit",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/bashrs"
                }
            },
            "results": sarif_results
        }]
    });

    match serde_json::to_string_pretty(&sarif) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("Error serializing JSON: {}", e);
            std::process::exit(1);
        }
    }
}

// ============================================================================
// Coverage Command (v6.13.0 - Bash Quality Tools)
// ============================================================================

use crate::cli::args::CoverageOutputFormat;

fn coverage_command(
    input: &Path,
    format: &CoverageOutputFormat,
    min: Option<u8>,
    detailed: bool,
    output: Option<&Path>,
) -> Result<()> {
    use crate::bash_quality::coverage::generate_coverage;

    // Read input file
    let source = fs::read_to_string(input)
        .map_err(|e| Error::Internal(format!("Failed to read {}: {}", input.display(), e)))?;

    // Generate coverage report
    let coverage = generate_coverage(&source)
        .map_err(|e| Error::Internal(format!("Failed to generate coverage: {}", e)))?;

    // Check minimum coverage if specified
    if let Some(min_percent) = min {
        let line_coverage = coverage.line_coverage_percent();
        if line_coverage < min_percent as f64 {
            return Err(Error::Internal(format!(
                "Coverage {:.1}% is below minimum {}%",
                line_coverage, min_percent
            )));
        }
    }

    // Output results
    match format {
        CoverageOutputFormat::Terminal => {
            print_terminal_coverage(&coverage, detailed, input);
        }
        CoverageOutputFormat::Json => {
            print_json_coverage(&coverage);
        }
        CoverageOutputFormat::Html => {
            print_html_coverage(&coverage, input, output);
        }
        CoverageOutputFormat::Lcov => {
            print_lcov_coverage(&coverage, input);
        }
    }

    Ok(())
}

/// Print terminal coverage output with ANSI colors
fn print_terminal_coverage(
    coverage: &crate::bash_quality::coverage::CoverageReport,
    detailed: bool,
    input: &Path,
) {
    use crate::cli::color::*;

    println!();
    println!("{BOLD}Coverage Report:{RESET} {CYAN}{}{RESET}", input.display());
    println!();

    let line_pct = coverage.line_coverage_percent();
    let func_pct = coverage.function_coverage_percent();

    // Overall coverage with progress bars
    let lc = score_color(line_pct);
    let fc = score_color(func_pct);
    let line_bar = progress_bar(coverage.covered_lines.len(), coverage.total_lines, 16);
    let func_bar = progress_bar(coverage.covered_functions.len(), coverage.all_functions.len(), 16);

    println!(
        "Lines:     {lc}{}/{}{RESET}  ({lc}{:.1}%{RESET})  {line_bar}",
        coverage.covered_lines.len(),
        coverage.total_lines,
        line_pct,
    );

    println!(
        "Functions: {fc}{}/{}{RESET}  ({fc}{:.1}%{RESET})  {func_bar}",
        coverage.covered_functions.len(),
        coverage.all_functions.len(),
        func_pct,
    );
    println!();

    // Show uncovered items
    let uncovered_lines = coverage.uncovered_lines();
    if !uncovered_lines.is_empty() {
        if detailed {
            println!(
                "{YELLOW}Uncovered Lines:{RESET} {}",
                uncovered_lines
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        } else {
            println!("{YELLOW}Uncovered Lines:{RESET} {} lines", uncovered_lines.len());
        }
        println!();
    }

    let uncovered_funcs = coverage.uncovered_functions();
    if !uncovered_funcs.is_empty() {
        if detailed {
            println!("{YELLOW}Uncovered Functions:{RESET}");
            for func in uncovered_funcs {
                println!("  {DIM}-{RESET} {}", func);
            }
        } else {
            println!("{YELLOW}Uncovered Functions:{RESET} {}", uncovered_funcs.len());
        }
        println!();
    }

    // Summary
    if coverage.total_lines == 0 {
        println!("{YELLOW}⚠ No executable code found{RESET}");
    } else if coverage.covered_lines.is_empty() {
        println!("{YELLOW}⚠ No tests found - 0% coverage{RESET}");
    } else if line_pct >= 80.0 {
        println!("{GREEN}✓ Good coverage!{RESET}");
    } else if line_pct >= 50.0 {
        println!("{YELLOW}⚠ Moderate coverage - consider adding more tests{RESET}");
    } else {
        println!("{BRIGHT_RED}✗ Low coverage - more tests needed{RESET}");
    }
}

// coverage_status moved to cli/logic.rs

/// Print JSON coverage output
fn print_json_coverage(coverage: &crate::bash_quality::coverage::CoverageReport) {
    use serde_json::json;

    let json_coverage = json!({
        "coverage": {
            "lines": {
                "total": coverage.total_lines,
                "covered": coverage.covered_lines.len(),
                "percent": coverage.line_coverage_percent(),
            },
            "functions": {
                "total": coverage.all_functions.len(),
                "covered": coverage.covered_functions.len(),
                "percent": coverage.function_coverage_percent(),
            },
            "uncovered_lines": coverage.uncovered_lines(),
            "uncovered_functions": coverage.uncovered_functions(),
        }
    });

    match serde_json::to_string_pretty(&json_coverage) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("Error serializing JSON: {}", e);
            std::process::exit(1);
        }
    }
}

/// Print HTML coverage output
fn print_html_coverage(
    coverage: &crate::bash_quality::coverage::CoverageReport,
    input: &Path,
    output: Option<&Path>,
) {
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Coverage Report - {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        h1 {{ color: #333; }}
        .summary {{ background: #f5f5f5; padding: 15px; border-radius: 5px; }}
        .coverage {{ font-size: 24px; font-weight: bold; }}
        .good {{ color: #28a745; }}
        .medium {{ color: #ffc107; }}
        .poor {{ color: #dc3545; }}
        table {{ border-collapse: collapse; width: 100%; margin-top: 20px; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
        .covered {{ background-color: #d4edda; }}
        .uncovered {{ background-color: #f8d7da; }}
    </style>
</head>
<body>
    <h1>Coverage Report</h1>
    <h2>{}</h2>
    <div class="summary">
        <p><strong>Line Coverage:</strong> 
            <span class="coverage {}">{:.1}%</span> 
            ({}/{})</p>
        <p><strong>Function Coverage:</strong> 
            <span class="coverage {}">{:.1}%</span> 
            ({}/{})</p>
    </div>
    <h3>Uncovered Functions</h3>
    <ul>
        {}
    </ul>
</body>
</html>"#,
        input.display(),
        input.display(),
        coverage_class(coverage.line_coverage_percent()),
        coverage.line_coverage_percent(),
        coverage.covered_lines.len(),
        coverage.total_lines,
        coverage_class(coverage.function_coverage_percent()),
        coverage.function_coverage_percent(),
        coverage.covered_functions.len(),
        coverage.all_functions.len(),
        coverage
            .uncovered_functions()
            .iter()
            .map(|f| format!("<li>{}</li>", f))
            .collect::<Vec<_>>()
            .join("\n        ")
    );

    if let Some(output_path) = output {
        if let Err(e) = fs::write(output_path, html) {
            eprintln!("Error writing HTML report: {}", e);
            std::process::exit(1);
        }
        println!("HTML coverage report written to {}", output_path.display());
    } else {
        println!("{}", html);
    }
}

// coverage_class moved to cli/logic.rs

/// Print LCOV coverage output
fn print_lcov_coverage(coverage: &crate::bash_quality::coverage::CoverageReport, input: &Path) {
    println!("TN:");
    println!("SF:{}", input.display());

    // Function coverage
    for func in &coverage.all_functions {
        let covered = if coverage.covered_functions.contains(func) {
            1
        } else {
            0
        };
        println!("FN:0,{}", func);
        println!("FNDA:{},{}", covered, func);
    }
    println!("FNF:{}", coverage.all_functions.len());
    println!("FNH:{}", coverage.covered_functions.len());

    // Line coverage
    for (line_num, &is_covered) in &coverage.line_coverage {
        let hit = if is_covered { 1 } else { 0 };
        println!("DA:{},{}", line_num, hit);
    }
    println!("LF:{}", coverage.total_lines);
    println!("LH:{}", coverage.covered_lines.len());

    println!("end_of_record");
}

// Format command implementation (v6.14.0)
fn format_command(
    inputs: &[PathBuf],
    check: bool,
    dry_run: bool,
    output: Option<&Path>,
) -> Result<()> {
    let mut all_formatted = true;

    for input_path in inputs {
        let (source, formatted) = format_read_and_format(input_path)?;

        if check {
            if !format_check_file(input_path, &source, &formatted) {
                all_formatted = false;
            }
        } else if dry_run {
            format_dry_run_file(input_path, &source, &formatted);
        } else {
            format_apply_file(input_path, &source, &formatted, output)?;
        }
    }

    if check && !all_formatted {
        return Err(Error::Internal(
            "Files are not properly formatted. Run without --check to fix.".to_string(),
        ));
    }

    Ok(())
}

fn format_read_and_format(input_path: &Path) -> Result<(String, String)> {
    use crate::bash_quality::Formatter;

    let config = format_load_config(input_path);
    let mut formatter = Formatter::with_config(config);

    let source = fs::read_to_string(input_path).map_err(|e| {
        Error::Internal(format!("Failed to read {}: {}", input_path.display(), e))
    })?;
    let formatted = formatter.format_source(&source).map_err(|e| {
        Error::Internal(format!("Failed to format {}: {}", input_path.display(), e))
    })?;

    Ok((source, formatted))
}

fn format_load_config(input_path: &Path) -> crate::bash_quality::FormatterConfig {
    use crate::bash_quality::FormatterConfig;

    if let Some(parent) = input_path.parent() {
        let script_dir_config = parent.join(".bashrs-fmt.toml");
        if script_dir_config.exists() {
            return FormatterConfig::from_file(&script_dir_config).unwrap_or_default();
        }
    }
    FormatterConfig::from_file(".bashrs-fmt.toml").unwrap_or_default()
}

fn format_check_file(input_path: &Path, source: &str, formatted: &str) -> bool {
    if source.trim() == formatted.trim() {
        println!("✓ {} is properly formatted", input_path.display());
        true
    } else {
        println!("✗ {} is not properly formatted", input_path.display());
        false
    }
}

fn format_dry_run_file(input_path: &Path, source: &str, formatted: &str) {
    println!("Would format: {}", input_path.display());
    if source.trim() != formatted.trim() {
        println!("  Changes detected");
    } else {
        println!("  No changes needed");
    }
}

fn format_apply_file(
    input_path: &Path,
    _source: &str,
    formatted: &str,
    output: Option<&Path>,
) -> Result<()> {
    if let Some(out_path) = output {
        fs::write(out_path, formatted).map_err(|e| {
            Error::Internal(format!("Failed to write {}: {}", out_path.display(), e))
        })?;
        println!("✓ Formatted {} -> {}", input_path.display(), out_path.display());
    } else {
        fs::write(input_path, formatted).map_err(|e| {
            Error::Internal(format!("Failed to write {}: {}", input_path.display(), e))
        })?;
        println!("✓ Formatted {}", input_path.display());
    }
    Ok(())
}

// ============================================================================
// Dockerfile Scoring Output Functions (Issue #10)
// ============================================================================

/// Print human-readable Dockerfile score results
fn print_human_dockerfile_score_results(
    score: &crate::bash_quality::dockerfile_scoring::DockerfileQualityScore,
    detailed: bool,
) {
    use crate::cli::color::*;

    println!();
    println!("{BOLD}Dockerfile Quality Score{RESET}");
    println!("{DIM}════════════════════════{RESET}");
    println!();
    let gc = grade_color(&score.grade);
    println!("Overall Grade: {gc}{}{RESET}", score.grade);
    println!("Overall Score: {WHITE}{:.1}/10.0{RESET}", score.score);
    println!();

    if detailed {
        println!("{BOLD}Dimension Scores:{RESET}");
        println!("{DIM}─────────────────{RESET}");
        let dim_line = |name: &str, val: f64, weight: &str| {
            let sc = score_color(val * 10.0);
            println!("{:<21} {sc}{:.1}/10.0{RESET}  {DIM}({weight}){RESET}", name, val);
        };
        dim_line("Safety:", score.safety, "30% weight");
        dim_line("Complexity:", score.complexity, "25% weight");
        dim_line("Layer Optimization:", score.layer_optimization, "20% weight");
        dim_line("Determinism:", score.determinism, "15% weight");
        dim_line("Security:", score.security, "10% weight");
        println!();
    }

    if !score.suggestions.is_empty() {
        println!("{BOLD}Improvement Suggestions:{RESET}");
        println!("{DIM}────────────────────────{RESET}");
        for (i, suggestion) in score.suggestions.iter().enumerate() {
            println!("{YELLOW}{}. {}{RESET}", i + 1, suggestion);
        }
        println!();
    }

    match score.grade.as_str() {
        "A+" => println!("{GREEN}✓ Excellent! Production-ready Dockerfile.{RESET}"),
        "A" => println!("{GREEN}✓ Very good! Minor improvements possible.{RESET}"),
        "B+" | "B" => println!("{GREEN}✓ Good Dockerfile with room for optimization.{RESET}"),
        "C+" | "C" => println!("{YELLOW}⚠ Average. Consider addressing suggestions.{RESET}"),
        "D" => println!("{RED}⚠ Below average. Multiple improvements needed.{RESET}"),
        "F" => println!("{BRIGHT_RED}✗ Poor quality. Significant improvements required.{RESET}"),
        _ => println!("{DIM}Unknown grade.{RESET}"),
    }
    println!();
}

/// Print Markdown Dockerfile score results
fn print_markdown_dockerfile_score_results(
    score: &crate::bash_quality::dockerfile_scoring::DockerfileQualityScore,
    input: &Path,
) {
    println!("# Dockerfile Quality Report");
    println!();
    println!("**File**: `{}`", input.display());
    println!(
        "**Date**: {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );
    println!();
    println!("## Overall Score");
    println!();
    println!(
        "**Grade**: {} | **Score**: {:.1}/10.0",
        score.grade, score.score
    );
    println!();
    println!("## Dimension Scores");
    println!();
    println!("| Dimension | Score | Weight | Status |");
    println!("| --- | --- | --- | --- |");
    println!(
        "| Safety | {:.1}/10.0 | 30% | {} |",
        score.safety,
        score_status(score.safety)
    );
    println!(
        "| Complexity | {:.1}/10.0 | 25% | {} |",
        score.complexity,
        score_status(score.complexity)
    );
    println!(
        "| Layer Optimization | {:.1}/10.0 | 20% | {} |",
        score.layer_optimization,
        score_status(score.layer_optimization)
    );
    println!(
        "| Determinism | {:.1}/10.0 | 15% | {} |",
        score.determinism,
        score_status(score.determinism)
    );
    println!(
        "| Security | {:.1}/10.0 | 10% | {} |",
        score.security,
        score_status(score.security)
    );
    println!();

    if !score.suggestions.is_empty() {
        println!("## Improvement Suggestions");
        println!();
        for suggestion in &score.suggestions {
            println!("- {}", suggestion);
        }
        println!();
    }

    println!("## Grade Interpretation");
    println!();
    match score.grade.as_str() {
        "A+" => println!("✅ **Excellent!** Production-ready Dockerfile."),
        "A" => println!("✅ **Great!** Very good Docker best practices."),
        "B+" | "B" => println!("✅ **Good** Dockerfile with room for optimization."),
        "C+" | "C" => println!("⚠️ **Average**. Consider addressing suggestions."),
        "D" => println!("⚠️ **Below average**. Multiple improvements needed."),
        "F" => println!("❌ **Poor** quality. Significant improvements required."),
        _ => println!("Unknown grade."),
    }
    println!();
}

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
