//! Score output formatting functions extracted from score_commands.rs.
//!
//! Contains all print/output functions for score results in human, JSON,
//! and Markdown formats for both bash scripts and Dockerfiles.

use super::RuntimeScore;
use crate::cli::logic::score_status;
use std::path::Path;

/// Print human-readable runtime score
pub(crate) fn print_human_runtime_score(
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
pub(crate) fn print_combined_grade(
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
pub(crate) fn print_json_dockerfile_score_with_runtime(
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
pub(crate) fn print_markdown_runtime_score(rt: &RuntimeScore) {
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
pub(crate) fn print_human_score_results(
    score: &crate::bash_quality::scoring::QualityScore,
    detailed: bool,
) {
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
        "C+" | "C" => {
            println!("{YELLOW}⚠ Average code quality. Consider addressing suggestions.{RESET}");
        }
        "D" => println!("{RED}⚠ Below average. Multiple improvements needed.{RESET}"),
        "F" => {
            println!("{BRIGHT_RED}✗ Poor code quality. Significant improvements required.{RESET}");
        }
        _ => {}
    }
}

/// Print JSON score results
pub(crate) fn print_json_score_results(score: &crate::bash_quality::scoring::QualityScore) {
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
pub(crate) fn print_markdown_score_results(
    score: &crate::bash_quality::scoring::QualityScore,
    input: &Path,
) {
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

/// Print human-readable Dockerfile score results
pub(crate) fn print_human_dockerfile_score_results(
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
            println!(
                "{:<21} {sc}{:.1}/10.0{RESET}  {DIM}({weight}){RESET}",
                name, val
            );
        };
        dim_line("Safety:", score.safety, "30% weight");
        dim_line("Complexity:", score.complexity, "25% weight");
        dim_line(
            "Layer Optimization:",
            score.layer_optimization,
            "20% weight",
        );
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
pub(crate) fn print_markdown_dockerfile_score_results(
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
