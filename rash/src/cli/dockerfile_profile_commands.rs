use crate::cli::args::{LintProfileArg, ReportFormat};
use crate::models::{Error, Result};
use std::fs;
use std::path::Path;
use tracing::info;

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
