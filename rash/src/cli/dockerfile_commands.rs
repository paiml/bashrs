use crate::cli::args::{DockerfileCommands, LintFormat};
use crate::cli::logic::purify_dockerfile_source;
use crate::models::{Config, Error, Result};
use std::fs;
use std::path::Path;
use tracing::{info, warn};

// ---------------------------------------------------------------------------
// handle_dockerfile_command  (entry point, called from commands.rs)
// ---------------------------------------------------------------------------

pub(crate) fn handle_dockerfile_command(command: DockerfileCommands) -> Result<()> {
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
            let _ = (report, format, skip_bash_purify); // consumed by CLI args
            dockerfile_purify_command(DockerfilePurifyCommandArgs {
                input: &input,
                output: output.as_deref(),
                fix,
                no_backup,
                dry_run,
                skip_user,
            })
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
            super::dockerfile_profile_commands::dockerfile_profile_command(
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
            super::dockerfile_profile_commands::dockerfile_size_check_command(
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
            super::dockerfile_validate_commands::dockerfile_full_validate_command(
                &input, profile, size_check, graded, runtime, strict, format,
            )
        }
    }
}

// ---------------------------------------------------------------------------
// DockerfilePurifyOptions
// ---------------------------------------------------------------------------

struct DockerfilePurifyOptions<'a> {
    output: Option<&'a Path>,
    fix: bool,
    no_backup: bool,
    dry_run: bool,
    skip_user: bool,
}

// ---------------------------------------------------------------------------
// dockerfile_build_command
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// dockerfile_purify_command
// ---------------------------------------------------------------------------

pub(crate) struct DockerfilePurifyCommandArgs<'a> {
    pub input: &'a Path,
    pub output: Option<&'a Path>,
    pub fix: bool,
    pub no_backup: bool,
    pub dry_run: bool,
    pub skip_user: bool,
}

pub(crate) fn dockerfile_purify_command(args: DockerfilePurifyCommandArgs<'_>) -> Result<()> {
    let options = DockerfilePurifyOptions {
        output: args.output,
        fix: args.fix,
        no_backup: args.no_backup,
        dry_run: args.dry_run,
        skip_user: args.skip_user,
    };
    dockerfile_purify_command_impl(args.input, options)
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
pub(crate) fn purify_dockerfile(source: &str, skip_user: bool) -> Result<String> {
    Ok(purify_dockerfile_source(source, skip_user))
}

// ---------------------------------------------------------------------------
// dockerfile_lint_command
// ---------------------------------------------------------------------------

fn severity_icon(severity: crate::linter::Severity) -> &'static str {
    match severity {
        crate::linter::Severity::Error => "❌",
        crate::linter::Severity::Warning => "⚠",
        _ => "ℹ",
    }
}

fn print_dockerfile_lint_human(diagnostics: &[crate::linter::Diagnostic]) {
    if diagnostics.is_empty() {
        println!("No Dockerfile issues found");
        return;
    }
    println!("Dockerfile Issues:");
    println!("==================\n");
    for diag in diagnostics {
        println!(
            "{} Line {}: [{}] {}",
            severity_icon(diag.severity),
            diag.span.start_line,
            diag.code,
            diag.message
        );
        if let Some(ref fix) = diag.fix {
            println!("   Fix: {}", fix.replacement);
        }
        println!();
    }
    println!("Summary: {} issue(s) found", diagnostics.len());
}

fn print_dockerfile_lint_json(input: &Path, diagnostics: &[crate::linter::Diagnostic]) {
    let output = serde_json::json!({
        "file": input.display().to_string(),
        "diagnostics": diagnostics.iter().map(|d| {
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

fn print_dockerfile_lint_sarif(input: &Path, diagnostics: &[crate::linter::Diagnostic]) {
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
            "results": diagnostics.iter().map(|d| {
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

pub(crate) fn dockerfile_lint_command(
    input: &Path,
    format: LintFormat,
    rules: Option<&str>,
) -> Result<()> {
    use crate::linter::rules::lint_dockerfile;

    info!("Linting {} for Dockerfile issues", input.display());

    let source = fs::read_to_string(input).map_err(Error::Io)?;
    let result = lint_dockerfile(&source);

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

    match format {
        LintFormat::Human => print_dockerfile_lint_human(&filtered_diagnostics),
        LintFormat::Json => print_dockerfile_lint_json(input, &filtered_diagnostics),
        LintFormat::Sarif => print_dockerfile_lint_sarif(input, &filtered_diagnostics),
    }

    if filtered_diagnostics
        .iter()
        .any(|d| matches!(d.severity, crate::linter::Severity::Error))
    {
        std::process::exit(2);
    }

    Ok(())
}
