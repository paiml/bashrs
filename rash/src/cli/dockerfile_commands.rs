use crate::cli::args::{DockerfileCommands, LintFormat, LintProfileArg, ReportFormat};
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

pub(crate) fn dockerfile_purify_command(
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
pub(crate) fn purify_dockerfile(source: &str, skip_user: bool) -> Result<String> {
    Ok(purify_dockerfile_source(source, skip_user))
}

// ---------------------------------------------------------------------------
// dockerfile_lint_command
// ---------------------------------------------------------------------------

pub(crate) fn dockerfile_lint_command(input: &Path, format: LintFormat, rules: Option<&str>) -> Result<()> {
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

// ---------------------------------------------------------------------------
// dockerfile_profile_command
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub(crate) fn dockerfile_profile_command(
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
    use crate::linter::docker_profiler::{estimate_size, is_docker_available, PlatformProfile};

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
            docker_profile_human(
                input,
                &estimate,
                platform,
                build,
                layers,
                startup,
                memory,
                cpu,
                simulate_limits,
                full,
            );
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

fn docker_profile_markdown(input: &Path, estimate: &crate::linter::docker_profiler::SizeEstimate) {
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

// ---------------------------------------------------------------------------
// estimate_build_time
// ---------------------------------------------------------------------------

/// Estimate build time based on layer complexity
pub(crate) fn estimate_build_time(estimate: &crate::linter::docker_profiler::SizeEstimate) -> String {
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

// ---------------------------------------------------------------------------
// dockerfile_size_check_command
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub(crate) fn dockerfile_size_check_command(
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
        ReportFormat::Human => size_check_human_output(
            &estimate,
            &platform,
            custom_limit,
            verbose,
            layers,
            detect_bloat,
            verify,
            docker_verify,
            compression_analysis,
            strict,
        ),
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
            s[..s.len() - 2]
                .trim()
                .parse::<f64>()
                .ok()
                .map(|n| (n * 1_000_000_000.0) as u64)
        } else if s.ends_with("MB") {
            s[..s.len() - 2]
                .trim()
                .parse::<f64>()
                .ok()
                .map(|n| (n * 1_000_000.0) as u64)
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
            println!(
                "  {} [line {}]: {}",
                pattern.code, pattern.line, pattern.description
            );
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
    if effective_limit == u64::MAX {
        return Ok(());
    }

    let limit_gb = effective_limit as f64 / 1_000_000_000.0;
    let estimated_gb = estimate.total_estimated as f64 / 1_000_000_000.0;

    println!("Size Limit Check:");
    if estimate.total_estimated > effective_limit {
        println!(
            "  \u{2717} EXCEEDS LIMIT: {:.2}GB > {:.0}GB",
            estimated_gb, limit_gb
        );
        if strict {
            return Err(Error::Validation(format!(
                "Image size ({:.2}GB) exceeds limit ({:.0}GB)",
                estimated_gb, limit_gb
            )));
        }
    } else {
        let percentage = (estimate.total_estimated as f64 / effective_limit as f64) * 100.0;
        println!(
            "  \u{2713} Within limit: {:.2}GB / {:.0}GB ({:.0}%)",
            estimated_gb, limit_gb, percentage
        );
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
    println!(
        "- **Estimated total**: {:.2}GB\n",
        estimate.total_estimated as f64 / 1_000_000_000.0
    );

    if !estimate.bloat_patterns.is_empty() {
        println!("## Bloat Patterns\n");
        for pattern in &estimate.bloat_patterns {
            println!(
                "- **{}** (line {}): {}",
                pattern.code, pattern.line, pattern.description
            );
        }
        println!();
    }
}

// ---------------------------------------------------------------------------
// dockerfile_full_validate_command
// ---------------------------------------------------------------------------

pub(crate) fn dockerfile_full_validate_command(
    input: &Path,
    profile: Option<LintProfileArg>,
    size_check: bool,
    _graded: bool,
    runtime: bool,
    strict: bool,
    format: ReportFormat,
) -> Result<()> {
    use crate::linter::docker_profiler::PlatformProfile;
    use crate::linter::rules::LintProfile;

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
        ReportFormat::Human => dockerfile_full_validate_human(
            &source,
            lint_profile,
            platform_profile,
            size_check,
            runtime,
            strict,
        ),
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
    if !lint_passed {
        all_passed = false;
    }

    if size_check {
        let size_passed = dockerfile_validate_size_step(source, platform_profile);
        if !size_passed {
            all_passed = false;
        }
    }

    if runtime {
        dockerfile_validate_runtime_step();
    }

    dockerfile_validate_summary(all_passed, lint_profile, strict)
}

fn dockerfile_validate_lint_step(
    source: &str,
    lint_profile: crate::linter::rules::LintProfile,
) -> bool {
    use crate::linter::rules::lint_dockerfile_with_profile;

    println!("1. Linting Dockerfile...");
    let lint_result = lint_dockerfile_with_profile(source, lint_profile);
    let error_count = lint_result
        .diagnostics
        .iter()
        .filter(|d| d.severity == crate::linter::Severity::Error)
        .count();
    let warning_count = lint_result
        .diagnostics
        .iter()
        .filter(|d| d.severity == crate::linter::Severity::Warning)
        .count();

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
        println!(
            "   {} [{}] Line {}: {}",
            icon, diag.code, diag.span.start_line, diag.message
        );
    }
    println!();
    error_count == 0
}

fn dockerfile_validate_size_step(
    source: &str,
    platform_profile: crate::linter::docker_profiler::PlatformProfile,
) -> bool {
    use crate::linter::docker_profiler::estimate_size;

    println!("2. Checking image size...");
    let estimate = estimate_size(source);
    let size_gb = estimate.total_estimated as f64 / 1_000_000_000.0;
    let limit_gb = platform_profile.max_size_bytes() as f64 / 1_000_000_000.0;

    let passed = estimate.total_estimated < platform_profile.max_size_bytes();
    if passed {
        println!(
            "   \u{2713} Size OK: {:.2}GB (limit: {:.0}GB)\n",
            size_gb, limit_gb
        );
    } else {
        println!(
            "   \u{2717} Size exceeds limit: {:.2}GB > {:.0}GB\n",
            size_gb, limit_gb
        );
    }
    for pattern in &estimate.bloat_patterns {
        println!("   - {}: {}", pattern.code, pattern.description);
    }
    if !estimate.bloat_patterns.is_empty() {
        println!();
    }
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

fn dockerfile_validate_summary(
    all_passed: bool,
    lint_profile: crate::linter::rules::LintProfile,
    strict: bool,
) -> Result<()> {
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
    println!(
        "{}",
        serde_json::to_string_pretty(&json).unwrap_or_default()
    );
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
    let error_count = lint_result
        .diagnostics
        .iter()
        .filter(|d| d.severity == crate::linter::Severity::Error)
        .count();

    println!("## Lint Results\n");
    println!("- **Errors**: {}", error_count);
    println!(
        "- **Warnings**: {}\n",
        lint_result
            .diagnostics
            .iter()
            .filter(|d| d.severity == crate::linter::Severity::Warning)
            .count()
    );

    if size_check {
        let estimate = estimate_size(source);
        println!("## Size Analysis\n");
        println!(
            "- **Estimated size**: {:.2}GB\n",
            estimate.total_estimated as f64 / 1_000_000_000.0
        );
    }

    println!("## Result\n");
    if error_count == 0 {
        println!("\u{2713} **PASSED**");
    } else {
        println!("\u{2717} **FAILED**");
    }
}

// ---------------------------------------------------------------------------
// parse_public_key
// ---------------------------------------------------------------------------

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
