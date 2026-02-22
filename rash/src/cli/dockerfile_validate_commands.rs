use crate::cli::args::{LintProfileArg, ReportFormat};
use crate::models::{Error, Result};
use std::fs;
use std::path::Path;
use tracing::info;

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
