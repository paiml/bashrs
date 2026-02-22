//! Score command functions extracted from commands.rs.
//!
//! Handles `rash score` subcommand: scoring bash scripts and Dockerfiles
//! for quality, then outputting results in human, JSON, or Markdown format.

#[path = "score_output_commands.rs"]
mod score_output;
use score_output::*;

use crate::cli::args::{LintProfileArg, ScoreOutputFormat};
use crate::models::{Error, Result};
use std::fs;
use std::path::Path;

/// Score a bash script for quality
pub(crate) fn score_command(
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
pub(crate) struct RuntimeScore {
    /// Overall runtime score (0-100)
    pub(crate) score: f64,
    /// Image size in bytes
    pub(crate) estimated_size: u64,
    /// Size score component (0-100)
    pub(crate) size_score: f64,
    /// Layer optimization score (0-100)
    pub(crate) layer_score: f64,
    /// Number of bloat patterns detected
    pub(crate) bloat_count: usize,
    /// Whether Docker is available for actual measurement
    pub(crate) docker_available: bool,
    /// Suggestions for improvement
    pub(crate) suggestions: Vec<String>,
}

impl RuntimeScore {
    pub(crate) fn new(
        estimate: &crate::linter::docker_profiler::SizeEstimate,
        profile: crate::linter::docker_profiler::PlatformProfile,
        docker_available: bool,
    ) -> Self {
        let max_size = profile.max_size_bytes();
        let size_score = Self::calculate_size_score(estimate.total_estimated, max_size);
        let layer_count = estimate.layer_estimates.len();
        let bloat_count = estimate.bloat_patterns.len();
        let layer_score = Self::calculate_layer_score(layer_count, bloat_count);
        let suggestions =
            Self::build_suggestions(&estimate.bloat_patterns, layer_count, estimate.total_estimated, max_size);
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

    fn calculate_size_score(total_estimated: u64, max_size: u64) -> f64 {
        if max_size == u64::MAX {
            let five_gb = 5_000_000_000u64;
            if total_estimated < five_gb {
                100.0
            } else {
                let ratio = total_estimated as f64 / five_gb as f64;
                (100.0 / ratio).clamp(0.0, 100.0)
            }
        } else {
            let ratio = total_estimated as f64 / max_size as f64;
            if ratio > 1.0 {
                0.0
            } else if ratio > 0.8 {
                (1.0 - ratio) * 500.0
            } else {
                100.0 - (ratio * 50.0)
            }
        }
    }

    fn calculate_layer_score(layer_count: usize, bloat_count: usize) -> f64 {
        let base = if layer_count <= 5 {
            100.0
        } else if layer_count <= 10 {
            80.0
        } else {
            60.0
        };
        (base - (bloat_count as f64 * 20.0)).max(0.0)
    }

    fn build_suggestions(
        bloat_patterns: &[crate::linter::docker_profiler::BloatPattern],
        layer_count: usize,
        total_estimated: u64,
        max_size: u64,
    ) -> Vec<String> {
        let mut suggestions: Vec<String> = bloat_patterns
            .iter()
            .map(|p| format!("{}: {}", p.code, p.remediation))
            .collect();
        if layer_count > 10 {
            suggestions.push("Consider combining RUN commands to reduce layer count".to_string());
        }
        if total_estimated > max_size {
            suggestions.push(format!(
                "Image size ({:.1}GB) exceeds limit ({:.1}GB) - use smaller base image or multi-stage build",
                total_estimated as f64 / 1_000_000_000.0,
                max_size as f64 / 1_000_000_000.0
            ));
        }
        suggestions
    }

    pub(crate) fn grade(&self) -> &'static str {
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
