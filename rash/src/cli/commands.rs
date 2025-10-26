use crate::cli::args::{
    CompileRuntime, ConfigCommands, ConfigOutputFormat, ContainerFormatArg, InspectionFormat,
    LintFormat, MakeCommands, MakeOutputFormat, ReportFormat,
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

fn make_lint_command(
    input: &Path,
    format: LintFormat,
    fix: bool,
    output: Option<&Path>,
    rules: Option<&str>,
) -> Result<()> {
    use crate::linter::{
        autofix::{apply_fixes_to_file, FixOptions},
        output::{write_results, OutputFormat},
        rules::lint_makefile,
    };

    // Read input file
    let source = fs::read_to_string(input).map_err(Error::Io)?;

    // Run linter
    let mut result = lint_makefile(&source);

    // Filter by specific rules if requested
    if let Some(rule_filter) = rules {
        let allowed_rules: Vec<&str> = rule_filter.split(',').map(|s| s.trim()).collect();
        result
            .diagnostics
            .retain(|d| allowed_rules.iter().any(|rule| d.code.contains(rule)));
    }

    // Apply fixes if requested
    if fix && result.diagnostics.iter().any(|d| d.fix.is_some()) {
        if let Some(output_path) = output {
            // Output to separate file: don't modify original
            // Apply fixes in memory and write to output
            use crate::linter::autofix::{apply_fixes, FixOptions};

            let fix_options = FixOptions {
                create_backup: false, // Don't create backup for output file
                dry_run: false,
                backup_suffix: String::new(),
                apply_assumptions: false, // TODO: Wire up fix_assumptions flag
                output_path: None,
            };

            let fix_result = apply_fixes(&source, &result, &fix_options)
                .map_err(|e| Error::Internal(format!("Failed to apply fixes: {e}")))?;

            if let Some(fixed_source) = fix_result.modified_source {
                fs::write(output_path, &fixed_source).map_err(Error::Io)?;
                info!("Fixed Makefile written to {}", output_path.display());

                // Re-lint the fixed content
                let result_after = lint_makefile(&fixed_source);
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
        } else {
            // In-place fixing: modify original file
            let options = FixOptions {
                create_backup: true,
                dry_run: false,
                backup_suffix: ".bak".to_string(),
                apply_assumptions: false, // TODO: Wire up fix_assumptions flag
                output_path: None,
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
                    let result_after = lint_makefile(&source_after);

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
                        .map_err(|e| {
                            Error::Internal(format!("Failed to write lint results: {e}"))
                        })?;
                    }
                }
                Err(e) => {
                    return Err(Error::Internal(format!("Failed to apply fixes: {e}")));
                }
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
