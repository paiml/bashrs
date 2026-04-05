//! Docker image profiling and size verification
//!
//! Provides functionality for:
//! - Image size estimation from Dockerfile analysis
//! - Layer-by-layer size breakdown
//! - Bloat pattern detection
//! - Runtime profiling (requires Docker daemon)
//! - Platform-specific constraint validation (Coursera, etc.)

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use std::collections::HashMap;
use std::process::Command;

#[cfg(test)]
#[path = "docker_profiler_tests.rs"]
mod tests;

/// Known base image sizes (approximate, in bytes)
const BASE_IMAGE_SIZES: &[(&str, u64)] = &[
    // Alpine variants
    ("alpine:latest", 7_000_000),
    ("alpine:3", 7_000_000),
    ("alpine:3.18", 7_000_000),
    ("alpine:3.19", 7_000_000),
    // Ubuntu variants
    ("ubuntu:latest", 78_000_000),
    ("ubuntu:22.04", 78_000_000),
    ("ubuntu:24.04", 80_000_000),
    // Python variants
    ("python:3.11", 1_000_000_000),
    ("python:3.11-slim", 150_000_000),
    ("python:3.11-alpine", 50_000_000),
    ("python:3.12", 1_000_000_000),
    ("python:3.12-slim", 150_000_000),
    // Node.js variants
    ("node:18", 1_000_000_000),
    ("node:18-slim", 200_000_000),
    ("node:18-alpine", 170_000_000),
    ("node:20", 1_100_000_000),
    ("node:20-slim", 210_000_000),
    ("node:20-alpine", 180_000_000),
    // Jupyter variants
    ("jupyter/base-notebook", 1_500_000_000),
    ("jupyter/base-notebook:latest", 1_500_000_000),
    ("jupyter/scipy-notebook", 3_000_000_000),
    ("jupyter/scipy-notebook:latest", 3_000_000_000),
    ("jupyter/datascience-notebook", 5_000_000_000),
    // NVIDIA CUDA variants
    ("nvidia/cuda:12.0-devel-ubuntu22.04", 8_500_000_000),
    ("nvidia/cuda:12.0-runtime-ubuntu22.04", 4_000_000_000),
    ("nvidia/cuda:12.0-base-ubuntu22.04", 500_000_000),
    // Docker variants
    ("docker:latest", 400_000_000),
    ("docker:dind", 450_000_000),
    // Nginx
    ("nginx:latest", 140_000_000),
    ("nginx:alpine", 40_000_000),
];

/// Known package sizes (approximate, in bytes)
const PACKAGE_SIZES: &[(&str, u64)] = &[
    // APT packages
    ("build-essential", 250_000_000),
    ("cmake", 50_000_000),
    ("git", 50_000_000),
    ("curl", 10_000_000),
    ("wget", 5_000_000),
    ("vim", 35_000_000),
    ("python3", 50_000_000),
    ("python3-pip", 20_000_000),
    ("nodejs", 100_000_000),
    ("npm", 50_000_000),
    // Python packages (pip)
    ("numpy", 50_000_000),
    ("pandas", 100_000_000),
    ("scipy", 100_000_000),
    ("matplotlib", 50_000_000),
    ("seaborn", 10_000_000),
    ("scikit-learn", 80_000_000),
    ("tensorflow", 1_500_000_000),
    ("torch", 2_000_000_000),
    ("torchvision", 500_000_000),
    ("pytorch", 2_000_000_000),
    ("transformers", 500_000_000),
    ("datasets", 100_000_000),
    ("jupyter", 100_000_000),
    ("notebook", 50_000_000),
    // Node packages
    ("@angular/cli", 300_000_000),
    ("create-react-app", 200_000_000),
    ("lodash", 5_000_000),
];

/// Size estimation result
#[derive(Debug, Clone)]
pub struct SizeEstimate {
    /// Base image size in bytes
    pub base_image_size: u64,
    /// Name of base image
    pub base_image: String,
    /// Per-layer size estimates
    pub layer_estimates: Vec<LayerEstimate>,
    /// Total estimated size in bytes
    pub total_estimated: u64,
    /// Detected bloat patterns
    pub bloat_patterns: Vec<BloatPattern>,
    /// Warnings generated during analysis
    pub warnings: Vec<String>,
}

/// Per-layer size estimate
#[derive(Debug, Clone)]
pub struct LayerEstimate {
    /// Layer number (1-indexed)
    pub layer_num: usize,
    /// Instruction type (FROM, RUN, COPY, etc.)
    pub instruction: String,
    /// The full instruction content
    pub content: String,
    /// Line number in Dockerfile
    pub line: usize,
    /// Estimated size in bytes
    pub estimated_size: u64,
    /// Whether this layer is cached (for build profiling)
    pub cached: bool,
    /// Notes about the estimate
    pub notes: Option<String>,
}

/// Detected bloat pattern
#[derive(Debug, Clone)]
pub struct BloatPattern {
    /// Pattern code (SIZE001, SIZE002, etc.)
    pub code: String,
    /// Description of the bloat
    pub description: String,
    /// Line number where detected
    pub line: usize,
    /// Estimated wasted bytes
    pub wasted_bytes: u64,
    /// Suggested fix
    pub remediation: String,
}

/// Runtime profile results
#[derive(Debug, Clone)]
pub struct RuntimeProfile {
    /// Build time in milliseconds
    pub build_time_ms: Option<u64>,
    /// Per-layer build times
    pub layer_times: Vec<LayerTiming>,
    /// Container startup time to healthy state (ms)
    pub startup_time_ms: Option<u64>,
    /// Peak memory usage in bytes
    pub peak_memory_bytes: Option<u64>,
    /// Average memory usage in bytes
    pub avg_memory_bytes: Option<u64>,
    /// Peak CPU percentage
    pub peak_cpu_percent: Option<f64>,
    /// Average CPU percentage
    pub avg_cpu_percent: Option<f64>,
    /// Actual image size in bytes (from docker images)
    pub actual_size_bytes: Option<u64>,
    /// Whether Docker daemon is available
    pub docker_available: bool,
}

/// Per-layer build timing
#[derive(Debug, Clone)]
pub struct LayerTiming {
    /// Layer number
    pub layer_num: usize,
    /// Instruction
    pub instruction: String,
    /// Build time in milliseconds
    pub time_ms: u64,
    /// Whether cached
    pub cached: bool,
}

/// Platform-specific constraints
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformProfile {
    /// No specific platform constraints
    Standard,
    /// Coursera Labs constraints (10GB limit, 4GB RAM, 1-minute startup)
    Coursera,
}

impl PlatformProfile {
    /// Maximum image size in bytes
    pub fn max_size_bytes(&self) -> u64 {
        match self {
            PlatformProfile::Standard => u64::MAX,
            PlatformProfile::Coursera => 10 * 1024 * 1024 * 1024, // 10GB
        }
    }

    /// Maximum memory in bytes
    pub fn max_memory_bytes(&self) -> u64 {
        match self {
            PlatformProfile::Standard => u64::MAX,
            PlatformProfile::Coursera => 4 * 1024 * 1024 * 1024, // 4GB
        }
    }

    /// Maximum startup time in milliseconds
    pub fn max_startup_ms(&self) -> u64 {
        match self {
            PlatformProfile::Standard => u64::MAX,
            PlatformProfile::Coursera => 60_000, // 1 minute
        }
    }

    /// Warning threshold for size (percentage of max)
    pub fn size_warning_threshold(&self) -> f64 {
        0.80 // Warn at 80% of limit
    }
}

/// Estimate image size from Dockerfile content
pub fn estimate_size(dockerfile: &str) -> SizeEstimate {
    let mut base_image = String::new();
    let mut base_image_size: u64 = 0;
    let mut layer_estimates = Vec::new();
    let mut bloat_patterns = Vec::new();
    let mut warnings = Vec::new();
    let mut layer_num = 0;
    let mut apt_update_without_clean = false;
    let mut apt_update_line = 0;

    for (line_idx, line) in dockerfile.lines().enumerate() {
        let line_num = line_idx + 1;
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let upper = trimmed.to_uppercase();

        // Parse FROM instruction
        if upper.starts_with("FROM ") {
            layer_num += 1;
            let image = trimmed[5..].split_whitespace().next().unwrap_or("");
            base_image = image.to_string();
            base_image_size = lookup_base_image_size(image);

            if base_image_size == 0 {
                warnings.push(format!(
                    "Unknown base image '{}' - size estimate may be inaccurate",
                    image
                ));
                // Default estimate for unknown images
                base_image_size = 500_000_000; // 500MB default
            }

            layer_estimates.push(LayerEstimate {
                layer_num,
                instruction: "FROM".to_string(),
                content: trimmed.to_string(),
                line: line_num,
                estimated_size: base_image_size,
                cached: true, // Base images are typically cached
                notes: Some(format!("Base image: {}", image)),
            });
        }
        // Parse RUN instruction
        else if upper.starts_with("RUN ") {
            layer_num += 1;
            let cmd = &trimmed[4..].trim();
            let (estimated, notes, bloat) = estimate_run_layer_size(cmd, line_num);

            // Track apt-get update without clean
            if cmd.contains("apt-get update") && !cmd.contains("rm -rf /var/lib/apt/lists") {
                apt_update_without_clean = true;
                apt_update_line = line_num;
            }

            // Check for cleanup in same command
            if cmd.contains("rm -rf /var/lib/apt/lists") {
                apt_update_without_clean = false;
            }

            layer_estimates.push(LayerEstimate {
                layer_num,
                instruction: "RUN".to_string(),
                content: trimmed.to_string(),
                line: line_num,
                estimated_size: estimated,
                cached: false,
                notes,
            });

            bloat_patterns.extend(bloat);
        }
        // Parse COPY/ADD instructions
        else if upper.starts_with("COPY ") || upper.starts_with("ADD ") {
            layer_num += 1;
            let instruction = if upper.starts_with("COPY") {
                "COPY"
            } else {
                "ADD"
            };

            // Can't estimate COPY/ADD size without build context
            layer_estimates.push(LayerEstimate {
                layer_num,
                instruction: instruction.to_string(),
                content: trimmed.to_string(),
                line: line_num,
                estimated_size: 0, // Unknown
                cached: false,
                notes: Some("Size depends on build context".to_string()),
            });
        }
        // Other instructions that don't add layers
        else if upper.starts_with("ENV ")
            || upper.starts_with("WORKDIR ")
            || upper.starts_with("EXPOSE ")
            || upper.starts_with("USER ")
            || upper.starts_with("ARG ")
            || upper.starts_with("LABEL ")
            || upper.starts_with("CMD ")
            || upper.starts_with("ENTRYPOINT ")
            || upper.starts_with("HEALTHCHECK ")
        {
            // These create metadata layers, minimal size
            layer_estimates.push(LayerEstimate {
                layer_num: layer_num + 1,
                instruction: upper.split_whitespace().next().unwrap_or("").to_string(),
                content: trimmed.to_string(),
                line: line_num,
                estimated_size: 0, // Metadata only
                cached: true,
                notes: Some("Metadata layer".to_string()),
            });
        }
    }

    // Add bloat pattern for apt-get update without cleanup
    if apt_update_without_clean {
        bloat_patterns.push(BloatPattern {
            code: "SIZE001".to_string(),
            description: "apt cache not cleaned after install".to_string(),
            line: apt_update_line,
            wasted_bytes: 200_000_000, // ~200MB
            remediation: "Add '&& rm -rf /var/lib/apt/lists/*' after apt-get install".to_string(),
        });
    }

    // Calculate total
    let total_estimated: u64 = layer_estimates.iter().map(|l| l.estimated_size).sum();

    SizeEstimate {
        base_image_size,
        base_image,
        layer_estimates,
        total_estimated,
        bloat_patterns,
        warnings,
    }
}

include!("docker_profiler_lookup.rs");
