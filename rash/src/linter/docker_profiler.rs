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

/// Look up base image size from known sizes
fn lookup_base_image_size(image: &str) -> u64 {
    // Try exact match first
    for (name, size) in BASE_IMAGE_SIZES {
        if image == *name {
            return *size;
        }
    }

    // Try prefix match (for tagged versions)
    let image_base = image.split(':').next().unwrap_or(image);
    for (name, size) in BASE_IMAGE_SIZES {
        let name_base = name.split(':').next().unwrap_or(name);
        if image_base == name_base {
            return *size;
        }
    }

    0
}

/// Estimate size of a RUN layer
fn estimate_run_layer_size(cmd: &str, line: usize) -> (u64, Option<String>, Vec<BloatPattern>) {
    let mut total: u64 = 0;
    let mut notes = Vec::new();
    let mut bloat = Vec::new();

    // Check for apt-get install
    if cmd.contains("apt-get install") || cmd.contains("apt install") {
        // Extract package names
        let packages = extract_apt_packages(cmd);
        for pkg in &packages {
            let pkg_size = lookup_package_size(pkg);
            if pkg_size > 0 {
                total += pkg_size;
                notes.push(format!("{}: ~{}MB", pkg, pkg_size / 1_000_000));
            }
        }

        // Check for missing --no-install-recommends
        if !cmd.contains("--no-install-recommends") && !packages.is_empty() {
            bloat.push(BloatPattern {
                code: "SIZE002".to_string(),
                description: "apt-get install without --no-install-recommends".to_string(),
                line,
                wasted_bytes: 100_000_000, // ~100MB typically
                remediation: "Add '--no-install-recommends' to apt-get install".to_string(),
            });
        }
    }

    // Check for pip install
    if cmd.contains("pip install") || cmd.contains("pip3 install") {
        let packages = extract_pip_packages(cmd);
        for pkg in &packages {
            let pkg_size = lookup_package_size(pkg);
            if pkg_size > 0 {
                total += pkg_size;
                notes.push(format!("{}: ~{}MB", pkg, pkg_size / 1_000_000));
            }
        }

        // Check for missing --no-cache-dir
        if !cmd.contains("--no-cache-dir") {
            bloat.push(BloatPattern {
                code: "SIZE003".to_string(),
                description: "pip install without --no-cache-dir".to_string(),
                line,
                wasted_bytes: 50_000_000, // ~50MB typically
                remediation: "Add '--no-cache-dir' to pip install".to_string(),
            });
        }
    }

    // Check for npm install
    if cmd.contains("npm install") || cmd.contains("npm i ") {
        // NPM packages can be large
        total += 200_000_000; // ~200MB base estimate
        notes.push("npm dependencies".to_string());

        // Check for node_modules
        if !cmd.contains("--production") && !cmd.contains("ci") {
            bloat.push(BloatPattern {
                code: "SIZE004".to_string(),
                description: "npm install includes dev dependencies".to_string(),
                line,
                wasted_bytes: 100_000_000,
                remediation: "Use 'npm ci --only=production' for smaller image".to_string(),
            });
        }
    }

    // Default minimum estimate for RUN commands with no detected packages
    if total == 0 {
        total = 10_000_000; // 10MB default for misc RUN commands
    }

    let notes_str = if notes.is_empty() {
        None
    } else {
        Some(notes.join(", "))
    };

    (total, notes_str, bloat)
}

/// Extract package names from apt-get install command
fn extract_apt_packages(cmd: &str) -> Vec<String> {
    let mut packages = Vec::new();

    // Find the install command and extract packages
    if let Some(idx) = cmd.find("install") {
        let after_install = &cmd[idx + 7..];
        for word in after_install.split_whitespace() {
            // Skip flags
            if word.starts_with('-') || word.starts_with('\\') {
                continue;
            }
            // Skip operators
            if word == "&&" || word == "||" || word == ";" {
                break;
            }
            // Skip -y which might come after install
            if word == "-y" {
                continue;
            }
            packages.push(word.to_string());
        }
    }

    packages
}

/// Extract package names from pip install command
fn extract_pip_packages(cmd: &str) -> Vec<String> {
    let mut packages = Vec::new();

    // Find pip install and extract packages
    let install_patterns = ["pip install", "pip3 install"];
    for pattern in &install_patterns {
        if let Some(idx) = cmd.find(pattern) {
            let after_install = &cmd[idx + pattern.len()..];
            for word in after_install.split_whitespace() {
                // Skip flags
                if word.starts_with('-') || word.starts_with('\\') {
                    continue;
                }
                // Skip operators
                if word == "&&" || word == "||" || word == ";" {
                    break;
                }
                // Skip requirement files
                if word.ends_with(".txt") {
                    continue;
                }
                packages.push(word.to_string());
            }
        }
    }

    packages
}

/// Look up package size from known sizes
fn lookup_package_size(package: &str) -> u64 {
    let package_lower = package.to_lowercase();

    for (name, size) in PACKAGE_SIZES {
        if package_lower == *name || package_lower.contains(name) {
            return *size;
        }
    }

    0
}

/// Check if Docker daemon is available
pub fn is_docker_available() -> bool {
    Command::new("docker")
        .arg("info")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get actual image size from Docker
pub fn get_docker_image_size(image_name: &str) -> Option<u64> {
    let output = Command::new("docker")
        .args(["images", image_name, "--format", "{{.Size}}"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let size_str = String::from_utf8_lossy(&output.stdout);
    parse_docker_size(size_str.trim())
}

/// Parse Docker size string (e.g., "1.5GB", "500MB")
fn parse_docker_size(size_str: &str) -> Option<u64> {
    let size_str = size_str.to_uppercase();

    // Try to parse number and unit
    let (num_str, multiplier) = if size_str.ends_with("GB") {
        (&size_str[..size_str.len() - 2], 1_000_000_000u64)
    } else if size_str.ends_with("MB") {
        (&size_str[..size_str.len() - 2], 1_000_000u64)
    } else if size_str.ends_with("KB") {
        (&size_str[..size_str.len() - 2], 1_000u64)
    } else if size_str.ends_with('B') {
        (&size_str[..size_str.len() - 1], 1u64)
    } else {
        return None;
    };

    num_str
        .trim()
        .parse::<f64>()
        .ok()
        .map(|n| (n * multiplier as f64) as u64)
}

/// Generate lint result from size estimate
pub fn size_estimate_to_lint_result(
    estimate: &SizeEstimate,
    profile: PlatformProfile,
    strict: bool,
) -> LintResult {
    let mut result = LintResult::new();

    // Add warnings from analysis
    for warning in &estimate.warnings {
        let span = Span::new(1, 1, 1, 1);
        result.add(Diagnostic::new("SIZE-INFO", Severity::Info, warning, span));
    }

    // Add bloat patterns as warnings
    for bloat in &estimate.bloat_patterns {
        let span = Span::new(bloat.line, 1, bloat.line, 1);
        let mut diag = Diagnostic::new(
            bloat.code.clone(),
            Severity::Warning,
            format!(
                "{} (~{}MB wasted)",
                bloat.description,
                bloat.wasted_bytes / 1_000_000
            ),
            span,
        );
        diag.fix = Some(Fix::new(bloat.remediation.clone()));
        result.add(diag);
    }

    // Check against platform limits
    let max_size = profile.max_size_bytes();
    let warning_threshold = (max_size as f64 * profile.size_warning_threshold()) as u64;

    if estimate.total_estimated > max_size {
        let severity = if strict {
            Severity::Error
        } else {
            Severity::Warning
        };
        let span = Span::new(1, 1, 1, 1);
        let mut diag = Diagnostic::new(
            "SIZE-LIMIT",
            severity,
            format!(
                "Estimated image size ({:.1}GB) exceeds platform limit ({:.1}GB)",
                estimate.total_estimated as f64 / 1_000_000_000.0,
                max_size as f64 / 1_000_000_000.0
            ),
            span,
        );
        diag.fix = Some(Fix::new(
            "Consider using a smaller base image or multi-stage build",
        ));
        result.add(diag);
    } else if estimate.total_estimated > warning_threshold {
        let span = Span::new(1, 1, 1, 1);
        let mut diag = Diagnostic::new(
            "SIZE-WARNING",
            Severity::Warning,
            format!(
                "Estimated image size ({:.1}GB) approaching platform limit ({:.1}GB)",
                estimate.total_estimated as f64 / 1_000_000_000.0,
                max_size as f64 / 1_000_000_000.0
            ),
            span,
        );
        diag.fix = Some(Fix::new("Consider optimizations to reduce image size"));
        result.add(diag);
    }

    result
}

/// Format size estimate as human-readable output
pub fn format_size_estimate(estimate: &SizeEstimate, verbose: bool) -> String {
    let mut output = String::new();

    output.push_str("Image Size Analysis\n");
    output.push_str("===================\n\n");

    // Base image
    output.push_str(&format!(
        "Base image: {} (~{:.1}GB)\n\n",
        estimate.base_image,
        estimate.base_image_size as f64 / 1_000_000_000.0
    ));

    // Layer breakdown
    if verbose {
        output.push_str("Layer Breakdown:\n");
        for layer in &estimate.layer_estimates {
            let size_str = if layer.estimated_size == 0 {
                "unknown".to_string()
            } else {
                format!("~{:.1}MB", layer.estimated_size as f64 / 1_000_000.0)
            };

            output.push_str(&format!(
                "  [{}] {} ({}) - line {}\n",
                layer.layer_num, layer.instruction, size_str, layer.line
            ));

            if let Some(notes) = &layer.notes {
                output.push_str(&format!("      {}\n", notes));
            }
        }
        output.push('\n');
    }

    // Total
    output.push_str(&format!(
        "Estimated total: {:.2}GB\n\n",
        estimate.total_estimated as f64 / 1_000_000_000.0
    ));

    // Bloat patterns
    if !estimate.bloat_patterns.is_empty() {
        output.push_str("Optimization Opportunities:\n");
        for bloat in &estimate.bloat_patterns {
            output.push_str(&format!(
                "  {} [line {}]: {} (~{}MB)\n",
                bloat.code,
                bloat.line,
                bloat.description,
                bloat.wasted_bytes / 1_000_000
            ));
            output.push_str(&format!("    Fix: {}\n", bloat.remediation));
        }
        output.push('\n');
    }

    // Warnings
    if !estimate.warnings.is_empty() {
        output.push_str("Warnings:\n");
        for warning in &estimate.warnings {
            output.push_str(&format!("  - {}\n", warning));
        }
    }

    output
}

/// Format size estimate as JSON
pub fn format_size_estimate_json(estimate: &SizeEstimate) -> String {
    let layers: Vec<HashMap<&str, serde_json::Value>> = estimate
        .layer_estimates
        .iter()
        .map(|l| {
            let mut map = HashMap::new();
            map.insert("layer_num", serde_json::json!(l.layer_num));
            map.insert("instruction", serde_json::json!(l.instruction));
            map.insert("line", serde_json::json!(l.line));
            map.insert("estimated_bytes", serde_json::json!(l.estimated_size));
            map.insert("notes", serde_json::json!(l.notes));
            map
        })
        .collect();

    let bloat: Vec<HashMap<&str, serde_json::Value>> = estimate
        .bloat_patterns
        .iter()
        .map(|b| {
            let mut map = HashMap::new();
            map.insert("code", serde_json::json!(b.code));
            map.insert("description", serde_json::json!(b.description));
            map.insert("line", serde_json::json!(b.line));
            map.insert("wasted_bytes", serde_json::json!(b.wasted_bytes));
            map.insert("remediation", serde_json::json!(b.remediation));
            map
        })
        .collect();

    let json = serde_json::json!({
        "base_image": estimate.base_image,
        "base_image_bytes": estimate.base_image_size,
        "total_estimated_bytes": estimate.total_estimated,
        "total_estimated_gb": estimate.total_estimated as f64 / 1_000_000_000.0,
        "layers": layers,
        "bloat_patterns": bloat,
        "warnings": estimate.warnings,
    });

    serde_json::to_string_pretty(&json).unwrap_or_else(|_| "{}".to_string())
}

/// List all SIZE rules
pub fn list_size_rules() -> Vec<(&'static str, &'static str)> {
    vec![
        ("SIZE001", "apt cache not cleaned after install"),
        ("SIZE002", "apt-get install without --no-install-recommends"),
        ("SIZE003", "pip install without --no-cache-dir"),
        ("SIZE004", "npm install includes dev dependencies"),
        ("SIZE-LIMIT", "Image size exceeds platform limit"),
        ("SIZE-WARNING", "Image size approaching platform limit"),
    ]
}
