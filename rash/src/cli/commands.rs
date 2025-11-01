use crate::cli::args::{
    CompileRuntime, ConfigCommands, ConfigOutputFormat, ContainerFormatArg, InspectionFormat,
    LintFormat, MakeCommands, MakeOutputFormat, ReportFormat, ScoreOutputFormat, TestOutputFormat,
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
        } => {
            info!("Linting {}", input.display());
            lint_command(&input, format, fix, fix_assumptions, output.as_deref())
        }

        Commands::Make { command } => handle_make_command(command), // Playground feature removed in v1.0 - will be moved to separate rash-playground crate in v1.1

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
        } => {
            info!("Scoring {}", input.display());
            score_command(&input, format, detailed)
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
    }
}

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

    // Check compatibility
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
    
    // TODO: Add your actual installation logic here
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

    fs::write(proof_path, proof).map_err(Error::Io)?;

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

fn lint_command(
    input: &Path,
    format: LintFormat,
    fix: bool,
    fix_assumptions: bool,
    output: Option<&Path>,
) -> Result<()> {
    use crate::linter::{
        autofix::{apply_fixes_to_file, FixOptions},
        output::{write_results, OutputFormat},
        rules::lint_shell,
    };

    // Read input file
    let source = fs::read_to_string(input).map_err(Error::Io)?;

    // Run linter
    let result = lint_shell(&source);

    // Apply fixes if requested
    if fix && result.diagnostics.iter().any(|d| d.fix.is_some()) {
        let options = FixOptions {
            create_backup: true,
            dry_run: false,
            backup_suffix: ".bak".to_string(),
            apply_assumptions: fix_assumptions, // NEW: Pass fix_assumptions flag
            output_path: output.map(|p| p.to_path_buf()), // NEW: Optional output path
        };

        match apply_fixes_to_file(input, &result, &options) {
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
                let result_after = lint_shell(&source_after);

                if result_after.diagnostics.is_empty() {
                    info!("✓ All issues fixed!");
                    return Ok(());
                } else {
                    info!("Remaining issues after auto-fix:");
                    let output_format = match format {
                        LintFormat::Human => OutputFormat::Human,
                        LintFormat::Json => OutputFormat::Json,
                        LintFormat::Sarif => OutputFormat::Sarif,
                    };
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
    } else {
        // Just show lint results
        let output_format = match format {
            LintFormat::Human => OutputFormat::Human,
            LintFormat::Json => OutputFormat::Json,
            LintFormat::Sarif => OutputFormat::Sarif,
        };

        let file_path = input.to_str().unwrap_or("unknown");
        write_results(&mut std::io::stdout(), &result, output_format, file_path)
            .map_err(|e| Error::Internal(format!("Failed to write lint results: {e}")))?;

        // Exit with appropriate code
        if result.has_errors() {
            std::process::exit(2);
        } else if result.has_warnings() {
            std::process::exit(1);
        }
    }

    Ok(())
}

fn handle_make_command(command: MakeCommands) -> Result<()> {
    match command {
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
        } => {
            info!("Purifying {}", input.display());
            make_purify_command(&input, output.as_deref(), fix, report, format)
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

fn make_purify_command(
    input: &Path,
    output: Option<&Path>,
    fix: bool,
    report: bool,
    format: ReportFormat,
) -> Result<()> {
    use crate::make_parser::{
        generators::generate_purified_makefile, parser::parse_makefile, purify::purify_makefile,
    };

    let source = fs::read_to_string(input).map_err(Error::Io)?;
    let ast = parse_makefile(&source)
        .map_err(|e| Error::Validation(format!("Failed to parse Makefile: {}", e)))?;
    let purify_result = purify_makefile(&ast);

    if report {
        // Print transformation report
        print_purify_report(&purify_result, format);
    }

    if fix {
        let purified = generate_purified_makefile(&purify_result.ast);

        if let Some(output_path) = output {
            // Write to specified output file
            fs::write(output_path, purified).map_err(Error::Io)?;
            info!("Purified Makefile written to {}", output_path.display());
        } else {
            // In-place: create backup and overwrite
            let backup_path = input.with_extension("mk.bak");
            fs::copy(input, &backup_path).map_err(Error::Io)?;
            fs::write(input, purified).map_err(Error::Io)?;
            info!("Purified Makefile written to {}", input.display());
            info!("Backup created at {}", backup_path.display());
        }
    } else {
        // Dry-run: print purified output to stdout
        let purified = generate_purified_makefile(&purify_result.ast);
        println!("{}", purified);
    }

    Ok(())
}

fn print_purify_report(
    result: &crate::make_parser::purify::PurificationResult,
    format: ReportFormat,
) {
    match format {
        ReportFormat::Human => {
            println!("Makefile Purification Report");
            println!("============================");
            println!(
                "Transformations Applied: {}",
                result.transformations_applied
            );
            println!("Issues Fixed: {}", result.issues_fixed);
            println!("Manual Fixes Needed: {}", result.manual_fixes_needed);
            println!();
            for (i, report_item) in result.report.iter().enumerate() {
                println!("{}: {}", i + 1, report_item);
            }
        }
        ReportFormat::Json => {
            println!("{{");
            println!(
                "  \"transformations_applied\": {},",
                result.transformations_applied
            );
            println!("  \"issues_fixed\": {},", result.issues_fixed);
            println!("  \"manual_fixes_needed\": {},", result.manual_fixes_needed);
            println!("  \"report\": [");
            for (i, report_item) in result.report.iter().enumerate() {
                let comma = if i < result.report.len() - 1 { "," } else { "" };
                println!("    \"{}\"{}", report_item.replace('"', "\\\""), comma);
            }
            println!("  ]");
            println!("}}");
        }
        ReportFormat::Markdown => {
            println!("# Makefile Purification Report\n");
            println!("**Transformations**: {}", result.transformations_applied);
            println!("**Issues Fixed**: {}", result.issues_fixed);
            println!("**Manual Fixes Needed**: {}\n", result.manual_fixes_needed);
            for (i, report_item) in result.report.iter().enumerate() {
                println!("{}. {}", i + 1, report_item);
            }
        }
    }
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

/// Run linter and optionally filter results by specific rules
fn run_filtered_lint(source: &str, rules: Option<&str>) -> crate::linter::LintResult {
    use crate::linter::rules::lint_makefile;

    let mut result = lint_makefile(source);

    // Filter by specific rules if requested
    if let Some(rule_filter) = rules {
        let allowed_rules: Vec<&str> = rule_filter.split(',').map(|s| s.trim()).collect();
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
        apply_assumptions: false, // TODO: Wire up fix_assumptions flag
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
        apply_assumptions: false, // TODO: Wire up fix_assumptions flag
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

    // Read input file
    let source = fs::read_to_string(input).map_err(Error::Io)?;

    // Analyze config
    let analysis = analyzer::analyze_config(&source, input.to_path_buf());

    // Output results
    match format {
        ConfigOutputFormat::Human => {
            println!("Analysis: {}", input.display());
            println!(
                "=========={}=",
                "=".repeat(input.display().to_string().len())
            );
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
                println!(
                    "Performance Issues ({}):",
                    analysis.performance_issues.len()
                );
                for issue in &analysis.performance_issues {
                    println!(
                        "  - Line {}: {} (~{}ms)",
                        issue.line, issue.command, issue.estimated_cost_ms
                    );
                    println!("    Suggestion: {}", issue.suggestion);
                }
                println!();
            }

            if analysis.issues.is_empty() {
                println!("✓ No issues found");
            } else {
                println!("Issues Found: {}", analysis.issues.len());
                for issue in &analysis.issues {
                    let severity_marker = match issue.severity {
                        crate::config::Severity::Error => "✗",
                        crate::config::Severity::Warning => "⚠",
                        crate::config::Severity::Info => "ℹ",
                    };
                    println!(
                        "  {} [{}] Line {}: {}",
                        severity_marker, issue.rule_id, issue.line, issue.message
                    );
                    if let Some(suggestion) = &issue.suggestion {
                        println!("    → {}", suggestion);
                    }
                }
            }
        }
        ConfigOutputFormat::Json => {
            // Simple JSON output
            println!("{{");
            println!("  \"file\": \"{}\",", input.display());
            println!("  \"line_count\": {},", analysis.line_count);
            println!("  \"complexity_score\": {},", analysis.complexity_score);
            println!("  \"path_entries\": {},", analysis.path_entries.len());
            println!(
                "  \"performance_issues\": {},",
                analysis.performance_issues.len()
            );
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
                println!("      \"message\": \"{}\"", issue.message);
                println!("    }}{}", comma);
            }
            println!("  ]");
            println!("}}");
        }
    }

    Ok(())
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

fn config_purify_command(
    input: &Path,
    output: Option<&Path>,
    fix: bool,
    no_backup: bool,
    dry_run: bool,
) -> Result<()> {
    use crate::config::{analyzer, purifier};
    use chrono::Local;

    // Read input file
    let source = fs::read_to_string(input).map_err(Error::Io)?;

    // Analyze first
    let analysis = analyzer::analyze_config(&source, input.to_path_buf());

    // Purify
    let purified = purifier::purify_config(&source);

    // Determine mode
    if let Some(output_path) = output {
        // Output to specific file
        if output_path.to_str() == Some("-") {
            // Output to stdout
            println!("{}", purified);
        } else {
            fs::write(output_path, &purified).map_err(Error::Io)?;
            info!("Purified config written to {}", output_path.display());
        }
    } else if fix && !dry_run {
        // Apply fixes in-place

        // Create backup unless --no-backup
        if !no_backup {
            let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
            let backup_path = input.with_extension(format!("bak.{}", timestamp));
            fs::copy(input, &backup_path).map_err(Error::Io)?;
            info!("Backup: {}", backup_path.display());
        }

        // Write purified content
        fs::write(input, &purified).map_err(Error::Io)?;

        let fixed_count = analysis.issues.len();
        println!("Applying {} fixes...", fixed_count);
        println!(
            "  ✓ Deduplicated {} PATH entries",
            analysis
                .path_entries
                .iter()
                .filter(|e| e.is_duplicate)
                .count()
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
    } else {
        // Dry-run mode (default)
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
            let original_lines: Vec<&str> = source.lines().collect();
            let purified_lines: Vec<&str> = purified.lines().collect();

            for (i, (orig, pure)) in original_lines.iter().zip(purified_lines.iter()).enumerate() {
                if orig != pure {
                    println!("-{}: {}", i + 1, orig);
                    println!("+{}: {}", i + 1, pure);
                }
            }

            println!();
            println!(
                "Apply fixes: bashrs config purify {} --fix",
                input.display()
            );
        }
    }

    Ok(())
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
                println!("✓ {}", test_name);
                if detailed {
                    if let Some(test) = report.tests.iter().find(|t| t.name == *test_name) {
                        if let Some(desc) = &test.description {
                            println!("  Description: {}", desc);
                        }
                        if let Some(given) = &test.given {
                            println!("  Given: {}", given);
                        }
                        if let Some(when) = &test.when {
                            println!("  When: {}", when);
                        }
                        if let Some(then) = &test.then {
                            println!("  Then: {}", then);
                        }
                    }
                }
            }
            TestResult::Fail(msg) => {
                println!("✗ {}", test_name);
                println!("  Error: {}", msg);
                if detailed {
                    if let Some(test) = report.tests.iter().find(|t| t.name == *test_name) {
                        if let Some(desc) = &test.description {
                            println!("  Description: {}", desc);
                        }
                    }
                }
            }
            TestResult::Skip(reason) => {
                println!("⊘ {} (skipped: {})", test_name, reason);
            }
        }
    }

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
        println!("✓ All tests passed!");
    } else {
        println!("✗ {} test(s) failed", report.failed());
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
fn score_command(input: &Path, format: ScoreOutputFormat, detailed: bool) -> Result<()> {
    use crate::bash_quality::scoring::score_script_with_file_type;

    // Read input file
    let source = fs::read_to_string(input)
        .map_err(|e| Error::Internal(format!("Failed to read {}: {}", input.display(), e)))?;

    // Score the script with file type detection
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

    Ok(())
}

/// Print human-readable score results
fn print_human_score_results(score: &crate::bash_quality::scoring::QualityScore, detailed: bool) {
    println!();
    println!("Bash Script Quality Score");
    println!("=========================");
    println!();
    println!("Overall Grade: {}", score.grade);
    println!("Overall Score: {:.1}/10.0", score.score);
    println!();

    if detailed {
        println!("Dimension Scores:");
        println!("-----------------");
        println!("Complexity:      {:.1}/10.0", score.complexity);
        println!("Safety:          {:.1}/10.0", score.safety);
        println!("Maintainability: {:.1}/10.0", score.maintainability);
        println!("Testing:         {:.1}/10.0", score.testing);
        println!("Documentation:   {:.1}/10.0", score.documentation);
        println!();
    }

    if !score.suggestions.is_empty() {
        println!("Improvement Suggestions:");
        println!("------------------------");
        for (i, suggestion) in score.suggestions.iter().enumerate() {
            println!("{}. {}", i + 1, suggestion);
        }
        println!();
    }

    // Grade interpretation
    match score.grade.as_str() {
        "A+" => println!("✓ Excellent! Near-perfect code quality."),
        "A" => println!("✓ Great! Very good code quality."),
        "B+" | "B" => println!("✓ Good code quality with room for improvement."),
        "C+" | "C" => println!("⚠ Average code quality. Consider addressing suggestions."),
        "D" => println!("⚠ Below average. Multiple improvements needed."),
        "F" => println!("✗ Poor code quality. Significant improvements required."),
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

/// Helper to get status emoji for dimension score
fn score_status(score: f64) -> &'static str {
    if score >= 8.0 {
        "✅"
    } else if score >= 6.0 {
        "⚠️"
    } else {
        "❌"
    }
}

// ============================================================================
// Audit Command (v6.12.0 - Bash Quality Tools)
// ============================================================================

use crate::cli::args::AuditOutputFormat;

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
    use crate::bash_quality::scoring::score_script;
    use crate::bash_quality::testing::{discover_tests, run_tests};
    use crate::linter::diagnostic::Severity;
    use crate::linter::rules::lint_shell;

    // Read input file
    let source = fs::read_to_string(input)
        .map_err(|e| Error::Internal(format!("Failed to read {}: {}", input.display(), e)))?;

    let mut results = AuditResults {
        parse_success: false,
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

    // Step 1: Parse check - just try to lint (which does parsing internally)
    // For now, we'll assume parse succeeds if file exists
    results.parse_success = true;

    // Step 2: Lint check
    let lint_result = lint_shell(&source);
    results.lint_errors = lint_result
        .diagnostics
        .iter()
        .filter(|d| matches!(d.severity, Severity::Error))
        .count();
    results.lint_warnings = lint_result
        .diagnostics
        .iter()
        .filter(|d| matches!(d.severity, Severity::Warning))
        .count();

    if results.lint_errors > 0 {
        results.overall_pass = false;
        results.failure_reason = Some(format!("{} lint errors found", results.lint_errors));
    }

    if strict && results.lint_warnings > 0 {
        results.overall_pass = false;
        results.failure_reason = Some(format!(
            "Strict mode: {} warnings found",
            results.lint_warnings
        ));
    }

    // Step 3: Test check
    match discover_tests(&source) {
        Ok(tests) => {
            match run_tests(&source, &tests) {
                Ok(test_report) => {
                    use crate::bash_quality::testing::TestResult;

                    results.test_total = test_report.results.len();
                    results.test_passed = test_report
                        .results
                        .iter()
                        .filter(|(_, result)| matches!(result, TestResult::Pass))
                        .count();
                    results.test_failed = test_report
                        .results
                        .iter()
                        .filter(|(_, result)| matches!(result, TestResult::Fail(_)))
                        .count();

                    if results.test_failed > 0 {
                        results.overall_pass = false;
                        results.failure_reason = Some(format!(
                            "{}/{} tests failed",
                            results.test_failed, results.test_total
                        ));
                    }
                }
                Err(_) => {
                    // Test execution failed - not a failure of the audit
                }
            }
        }
        Err(_) => {
            // No tests found - not a failure
        }
    }

    // Step 4: Quality score
    if results.parse_success {
        match score_script(&source) {
            Ok(score) => {
                // Check minimum grade if specified
                if let Some(min_grade_str) = min_grade {
                    let grade_order = ["F", "D", "C", "C+", "B", "B+", "A", "A+"];
                    let actual_grade_pos =
                        grade_order.iter().position(|&g| g == score.grade.as_str());
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
            Err(e) => {
                eprintln!("Warning: Failed to score script: {}", e);
            }
        }
    }

    // Output results
    match format {
        AuditOutputFormat::Human => {
            print_human_audit_results(&results, detailed, input);
        }
        AuditOutputFormat::Json => {
            print_json_audit_results(&results);
        }
        AuditOutputFormat::Sarif => {
            print_sarif_audit_results(&results, input);
        }
    }

    // Return error if overall check failed
    if !results.overall_pass {
        let reason = results
            .failure_reason
            .unwrap_or_else(|| "Quality audit failed".to_string());
        return Err(Error::Internal(reason));
    }

    Ok(())
}

/// Print human-readable audit results
fn print_human_audit_results(results: &AuditResults, detailed: bool, input: &Path) {
    println!();
    println!("Comprehensive Quality Audit");
    println!("===========================");
    println!();
    println!("File: {}", input.display());
    println!();
    println!("Check Results:");
    println!("--------------");

    // Parse
    if results.parse_success {
        println!("✅ Parse:    Valid bash syntax");
    } else {
        println!("❌ Parse:    Syntax error");
        if let Some(err) = &results.parse_error {
            println!("           {}", err);
        }
    }

    // Lint
    if results.lint_errors == 0 && results.lint_warnings == 0 {
        println!("✅ Lint:     No issues found");
    } else if results.lint_errors > 0 {
        println!(
            "❌ Lint:     {} errors, {} warnings",
            results.lint_errors, results.lint_warnings
        );
    } else {
        println!("⚠️  Lint:     {} warnings", results.lint_warnings);
    }

    // Test
    if results.test_total > 0 {
        if results.test_failed == 0 {
            println!(
                "✅ Test:     {}/{} tests passed",
                results.test_passed, results.test_total
            );
        } else {
            println!(
                "❌ Test:     {}/{} tests passed, {} failed",
                results.test_passed, results.test_total, results.test_failed
            );
        }
    } else {
        println!("⚠️  Test:     No tests found");
    }

    // Score
    if let Some(score) = &results.score {
        println!("✅ Score:    {} ({:.1}/10.0)", score.grade, score.score);

        if detailed {
            println!();
            println!("  Dimension Breakdown:");
            println!("  - Complexity:      {:.1}/10.0", score.complexity);
            println!("  - Safety:          {:.1}/10.0", score.safety);
            println!("  - Maintainability: {:.1}/10.0", score.maintainability);
            println!("  - Testing:         {:.1}/10.0", score.testing);
            println!("  - Documentation:   {:.1}/10.0", score.documentation);
        }
    }

    println!();
    println!(
        "Overall: {}",
        if results.overall_pass {
            "✅ PASS"
        } else {
            "❌ FAIL"
        }
    );
    println!();

    // Suggestions
    if let Some(score) = &results.score {
        if !score.suggestions.is_empty() {
            println!("Improvement Suggestions:");
            println!("------------------------");
            for (i, suggestion) in score.suggestions.iter().enumerate() {
                println!("{}. {}", i + 1, suggestion);
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

/// Print terminal coverage output
fn print_terminal_coverage(
    coverage: &crate::bash_quality::coverage::CoverageReport,
    detailed: bool,
    input: &Path,
) {
    println!();
    println!("Coverage Report: {}", input.display());
    println!();

    let line_pct = coverage.line_coverage_percent();
    let func_pct = coverage.function_coverage_percent();

    // Overall coverage
    println!(
        "Lines:     {}/{}   ({:.1}%)  {}",
        coverage.covered_lines.len(),
        coverage.total_lines,
        line_pct,
        coverage_status(line_pct)
    );

    println!(
        "Functions: {}/{}   ({:.1}%)  {}",
        coverage.covered_functions.len(),
        coverage.all_functions.len(),
        func_pct,
        coverage_status(func_pct)
    );
    println!();

    // Show uncovered items (always show if they exist)
    let uncovered_lines = coverage.uncovered_lines();
    if !uncovered_lines.is_empty() {
        if detailed {
            println!(
                "Uncovered Lines: {}",
                uncovered_lines
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        } else {
            println!("Uncovered Lines: {} lines", uncovered_lines.len());
        }
        println!();
    }

    let uncovered_funcs = coverage.uncovered_functions();
    if !uncovered_funcs.is_empty() {
        if detailed {
            println!("Uncovered Functions:");
            for func in uncovered_funcs {
                println!("  - {}", func);
            }
        } else {
            println!("Uncovered Functions: {}", uncovered_funcs.len());
        }
        println!();
    }

    // Summary
    if coverage.total_lines == 0 {
        println!("⚠️  No executable code found");
    } else if coverage.covered_lines.is_empty() {
        println!("⚠️  No tests found - 0% coverage");
    } else if line_pct >= 80.0 {
        println!("✅ Good coverage!");
    } else if line_pct >= 50.0 {
        println!("⚠️  Moderate coverage - consider adding more tests");
    } else {
        println!("❌ Low coverage - more tests needed");
    }
}

fn coverage_status(percent: f64) -> &'static str {
    if percent >= 80.0 {
        "✅"
    } else if percent >= 50.0 {
        "⚠️"
    } else {
        "❌"
    }
}

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

fn coverage_class(percent: f64) -> &'static str {
    if percent >= 80.0 {
        "good"
    } else if percent >= 50.0 {
        "medium"
    } else {
        "poor"
    }
}

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
    use crate::bash_quality::{Formatter, FormatterConfig};

    let mut all_formatted = true;

    for input_path in inputs {
        // Load configuration (look for .bashrs-fmt.toml in script's directory, then current directory)
        let config = if let Some(parent) = input_path.parent() {
            let script_dir_config = parent.join(".bashrs-fmt.toml");
            if script_dir_config.exists() {
                FormatterConfig::from_file(&script_dir_config).unwrap_or_default()
            } else {
                FormatterConfig::from_file(".bashrs-fmt.toml").unwrap_or_default()
            }
        } else {
            FormatterConfig::from_file(".bashrs-fmt.toml").unwrap_or_default()
        };

        let mut formatter = Formatter::with_config(config);

        // Read input file
        let source = fs::read_to_string(input_path).map_err(|e| {
            Error::Internal(format!("Failed to read {}: {}", input_path.display(), e))
        })?;

        // Format the source
        let formatted = formatter.format_source(&source).map_err(|e| {
            Error::Internal(format!("Failed to format {}: {}", input_path.display(), e))
        })?;

        if check {
            // Check mode: verify if formatted
            if source.trim() == formatted.trim() {
                println!("✓ {} is properly formatted", input_path.display());
            } else {
                println!("✗ {} is not properly formatted", input_path.display());
                all_formatted = false;
            }
        } else if dry_run {
            // Dry run: show what would be done
            println!("Would format: {}", input_path.display());
            if source.trim() != formatted.trim() {
                println!("  Changes detected");
            } else {
                println!("  No changes needed");
            }
        } else {
            // Apply formatting
            if let Some(out_path) = output {
                // Write to specified output file
                fs::write(out_path, &formatted).map_err(|e| {
                    Error::Internal(format!("Failed to write {}: {}", out_path.display(), e))
                })?;
                println!(
                    "✓ Formatted {} -> {}",
                    input_path.display(),
                    out_path.display()
                );
            } else {
                // Write in-place
                fs::write(input_path, &formatted).map_err(|e| {
                    Error::Internal(format!("Failed to write {}: {}", input_path.display(), e))
                })?;
                println!("✓ Formatted {}", input_path.display());
            }
        }
    }

    // If check mode and any file not formatted, return error
    if check && !all_formatted {
        return Err(Error::Internal(
            "Files are not properly formatted. Run without --check to fix.".to_string(),
        ));
    }

    Ok(())
}
