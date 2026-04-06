//! Dockerfile Quality Scoring (Issue #10)
//!
//! Dockerfile-specific quality metrics instead of general bash script metrics.
//!
//! ## Scoring Dimensions (Weighted)
//!
//! 1. **Safety (30%)**: set -euo pipefail usage, error handling
//! 2. **Complexity (25%)**: RUN command simplicity, script length
//! 3. **Layer Optimization (20%)**: combined commands, cache cleanup in same layer
//! 4. **Determinism (15%)**: version pinning, specific tags (not :latest)
//! 5. **Security (10%)**: non-root user, no credential exposure
//!
//! ## Grade Scale
//!
//! - A+ (9.5-10): Excellent
//! - A  (9.0-9.5): Very Good
//! - B+ (8.5-9.0): Good
//! - B  (8.0-8.5): Above Average
//! - C+ (7.5-8.0): Average
//! - C  (7.0-7.5): Below Average
//! - D  (6.0-7.0): Poor
//! - F  (<6.0): Failing

use serde::{Deserialize, Serialize};

/// Dockerfile quality score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerfileQualityScore {
    /// Overall grade (A+ to F)
    pub grade: String,

    /// Numeric score (0.0 - 10.0)
    pub score: f64,

    /// Safety score (0.0 - 10.0)
    pub safety: f64,

    /// Complexity score (0.0 - 10.0)
    pub complexity: f64,

    /// Layer optimization score (0.0 - 10.0)
    pub layer_optimization: f64,

    /// Determinism score (0.0 - 10.0)
    pub determinism: f64,

    /// Security score (0.0 - 10.0)
    pub security: f64,

    /// Improvement suggestions
    pub suggestions: Vec<String>,
}

impl DockerfileQualityScore {
    /// Create new Dockerfile quality score
    pub fn new() -> Self {
        Self {
            grade: String::from("F"),
            score: 0.0,
            safety: 0.0,
            complexity: 0.0,
            layer_optimization: 0.0,
            determinism: 0.0,
            security: 0.0,
            suggestions: Vec::new(),
        }
    }
}

impl Default for DockerfileQualityScore {
    fn default() -> Self {
        Self::new()
    }
}

/// Lint penalty breakdown
struct LintPenalty {
    determinism: f64,
    security: f64,
    layer: f64,
}

/// Calculate scoring penalties based on Dockerfile lint results (Issue #19)
///
/// Penalties are intentionally small to avoid overwhelming existing scoring.
/// The main value is in the detailed suggestions shown to users.
fn calculate_lint_penalty(lint_results: &crate::linter::LintResult) -> LintPenalty {
    let mut penalty = LintPenalty {
        determinism: 0.0,
        security: 0.0,
        layer: 0.0,
    };

    for diag in &lint_results.diagnostics {
        match diag.code.as_str() {
            "DOCKER001" => penalty.security += 0.5, // Missing USER (security issue, but scratch is OK)
            "DOCKER002" => penalty.determinism += 0.2, // Unpinned base image (common, minor penalty)
            "DOCKER003" => penalty.layer += 0.2,       // Missing apt cleanup
            "DOCKER004" => penalty.security += 0.5,    // Invalid COPY --from
            "DOCKER005" => penalty.layer += 0.1, // Missing --no-install-recommends (info level)
            "DOCKER006" => penalty.security += 0.05, // Use COPY not ADD (very minor)
            _ => {}
        }
    }

    penalty
}

/// Score a Dockerfile for quality
///
/// Returns Dockerfile-specific quality score with grade, numeric score, and suggestions.
/// Now integrates Issue #19 Dockerfile linting rules for accurate scoring.
pub fn score_dockerfile(source: &str) -> Result<DockerfileQualityScore, String> {
    let mut score = DockerfileQualityScore::new();

    // Run Dockerfile linting (Issue #19) to get accurate diagnostics
    use crate::linter::rules::lint_dockerfile;
    let lint_results = lint_dockerfile(source);

    // Apply lint penalties to scores
    let lint_penalty = calculate_lint_penalty(&lint_results);

    // Calculate each dimension
    score.safety = calculate_safety_score(source);
    score.complexity = calculate_complexity_score(source);
    score.layer_optimization = calculate_layer_optimization_score(source);
    score.determinism = calculate_determinism_score(source);
    score.security = calculate_security_score(source);

    // Apply lint penalties to relevant dimensions
    score.determinism = (score.determinism - lint_penalty.determinism).max(0.0);
    score.security = (score.security - lint_penalty.security).max(0.0);
    score.layer_optimization = (score.layer_optimization - lint_penalty.layer).max(0.0);

    // Calculate overall score (weighted average per Issue #10 spec)
    score.score = (score.safety * 0.30)
        + (score.complexity * 0.25)
        + (score.layer_optimization * 0.20)
        + (score.determinism * 0.15)
        + (score.security * 0.10);

    // Assign grade
    score.grade = calculate_grade(score.score);

    // Generate suggestions
    score.suggestions = generate_suggestions(source, &score);

    // Add lint-based suggestions (Issue #19)
    for diag in &lint_results.diagnostics {
        score
            .suggestions
            .push(format!("Line {}: {}", diag.span.start_line, diag.message));
    }

    Ok(score)
}

/// Calculate safety score (30% weight)
///
/// Checks for:
/// - `set -euo pipefail` in RUN commands
/// - Error handling (|| return, || exit)
/// - Proper error propagation
fn calculate_safety_score(source: &str) -> f64 {
    if source.trim().is_empty() {
        return 0.0;
    }

    let mut run_commands = 0;
    let mut safe_run_commands = 0;
    let mut has_error_handling = false;

    for line in source.lines() {
        let trimmed = line.trim();

        // Track RUN commands
        if trimmed.starts_with("RUN ") {
            run_commands += 1;

            // Check for set -euo pipefail
            if trimmed.contains("set -euo pipefail")
                || trimmed.contains("set -e") && trimmed.contains("set -o pipefail")
            {
                safe_run_commands += 1;
                has_error_handling = true;
            }

            // Check for error handling
            if trimmed.contains("|| exit") || trimmed.contains("|| return") {
                has_error_handling = true;
            }
        }
    }

    if run_commands == 0 {
        return 5.0; // Neutral score if no RUN commands
    }

    let safety_ratio = safe_run_commands as f64 / run_commands as f64;

    let mut score: f64 = match safety_ratio {
        r if r >= 0.8 => 10.0, // 80%+ safe commands
        r if r >= 0.6 => 8.0,  // 60-79% safe
        r if r >= 0.4 => 6.0,  // 40-59% safe
        r if r >= 0.2 => 4.0,  // 20-39% safe
        _ => 2.0,              // <20% safe
    };

    // Bonus for having any error handling
    if has_error_handling {
        score += 1.0;
    }

    score.min(10.0)
}

/// Calculate complexity score (25% weight)
///
/// Checks for:
/// - Number of RUN commands (fewer is better - encourages layer optimization)
/// - Length of RUN commands
/// - Multi-line command complexity
fn calculate_complexity_score(source: &str) -> f64 {
    let lines: Vec<&str> = source.lines().collect();

    if lines.is_empty() {
        return 0.0;
    }

    let mut run_count = 0;
    let mut longest_run = 0;
    let mut current_run_lines = 0;
    let mut in_run = false;

    for line in &lines {
        let trimmed = line.trim();

        if trimmed.starts_with("RUN ") {
            run_count += 1;
            in_run = true;
            current_run_lines = 1;
        } else if in_run {
            if trimmed.ends_with('\\') {
                current_run_lines += 1;
            } else {
                if current_run_lines > 0 {
                    current_run_lines += 1;
                    if current_run_lines > longest_run {
                        longest_run = current_run_lines;
                    }
                }
                in_run = false;
                current_run_lines = 0;
            }
        }
    }

    // Finalize if still in RUN
    if in_run && current_run_lines > 0 && current_run_lines > longest_run {
        longest_run = current_run_lines;
    }

    // Score based on RUN command count (fewer is better)
    let run_score = match run_count {
        0 => 5.0,       // Neutral
        1..=3 => 10.0,  // Excellent (combined commands)
        4..=6 => 8.0,   // Good
        7..=10 => 6.0,  // Average
        11..=15 => 4.0, // Below average
        _ => 2.0,       // Poor (too many layers)
    };

    // Score based on longest RUN command
    let length_score = match longest_run {
        0 => 5.0,       // Neutral
        1..=5 => 10.0,  // Simple
        6..=10 => 8.0,  // Moderate
        11..=20 => 6.0, // Complex
        21..=30 => 4.0, // Very complex
        _ => 2.0,       // Extremely complex
    };

    f64::midpoint(run_score, length_score)
}

/// Calculate layer optimization score (20% weight)
///
/// Checks for:
/// - Combined commands with && instead of separate RUN statements
/// - Cache cleanup in same layer (rm -rf /var/cache/apk/*, apt clean)
/// - --no-cache flag for package managers
/// - Multi-stage builds
fn calculate_layer_optimization_score(source: &str) -> f64 {
    if source.trim().is_empty() {
        return 0.0;
    }

    let mut has_combined_commands = false;
    let mut has_cache_cleanup = false;
    let mut has_no_cache_flag = false;
    let mut has_multistage = false;
    let mut run_count = 0;

    for line in source.lines() {
        let trimmed = line.trim();

        // Count RUN commands
        if trimmed.starts_with("RUN ") {
            run_count += 1;

            // Check for combined commands
            if trimmed.contains("&&") {
                has_combined_commands = true;
            }

            // Check for cache cleanup in same layer
            if trimmed.contains("rm -rf /var/cache/apk/*")
                || trimmed.contains("apt-get clean")
                || trimmed.contains("yum clean all")
                || trimmed.contains("rm -rf /var/lib/apt/lists/*")
            {
                has_cache_cleanup = true;
            }

            // Check for --no-cache flag
            if trimmed.contains("--no-cache") {
                has_no_cache_flag = true;
            }
        }

        // Check for multi-stage builds (FROM ... AS ...)
        if trimmed.starts_with("FROM ") && trimmed.contains(" AS ") {
            has_multistage = true;
        }
    }

    let mut score: f64 = 0.0;

    // Base score for having any optimizations
    if run_count > 0 {
        score += 2.0;
    }

    // Combined commands (4 points)
    if has_combined_commands {
        score += 4.0;
    }

    // Cache cleanup (3 points)
    if has_cache_cleanup || has_no_cache_flag {
        score += 3.0;
    }

    // Multi-stage builds (1 point bonus)
    if has_multistage {
        score += 1.0;
    }

    score.min(10.0)
}

/// Calculate determinism score (15% weight)
///
/// Checks for:
/// - Version pinning in package installs (curl=8.2.1-r0)
/// - Specific image tags (not :latest)
/// - Pinned base images (alpine:3.18 not alpine:latest)
fn calculate_determinism_score(source: &str) -> f64 {
    if source.trim().is_empty() {
        return 0.0;
    }

    let (has_pinned_base_image, uses_latest_tag, package_installs, pinned_packages) =
        scan_determinism_indicators(source);

    let score = score_base_image_pinning(has_pinned_base_image, uses_latest_tag)
        + score_package_pinning(package_installs, pinned_packages);

    score.min(10.0)
}

/// Scan Dockerfile lines for determinism indicators (base image tags, package pinning)
fn scan_determinism_indicators(source: &str) -> (bool, bool, u32, u32) {
    let mut has_pinned_base_image = false;
    let mut uses_latest_tag = false;
    let mut package_installs = 0u32;
    let mut pinned_packages = 0u32;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("FROM ") {
            if trimmed.contains(":latest") || (!trimmed.contains(':') && !trimmed.contains('@')) {
                uses_latest_tag = true;
            } else if trimmed.contains(':') {
                has_pinned_base_image = true;
            }
        }

        if trimmed.starts_with("RUN ") {
            let is_pkg_install = trimmed.contains("apk add")
                || trimmed.contains("apt-get install")
                || trimmed.contains("yum install");
            if is_pkg_install {
                package_installs += 1;
                if trimmed.contains('=') && (trimmed.contains("apk add") || trimmed.contains("apt"))
                {
                    pinned_packages += 1;
                }
            }
        }
    }

    (
        has_pinned_base_image,
        uses_latest_tag,
        package_installs,
        pinned_packages,
    )
}

/// Score base image pinning (0-5 points)
fn score_base_image_pinning(has_pinned: bool, uses_latest: bool) -> f64 {
    match (has_pinned, uses_latest) {
        (true, false) => 5.0,
        (true, true) => 3.0,
        (false, false) => 1.0,
        (false, true) => 0.0,
    }
}

/// Score package version pinning (0-5 points)
fn score_package_pinning(installs: u32, pinned: u32) -> f64 {
    if installs > 0 {
        (pinned as f64 / installs as f64) * 5.0
    } else {
        2.5 // Neutral if no packages
    }
}

/// Calculate security score (10% weight)
///
/// Checks for:
/// - USER directive (non-root)
/// - Avoiding ADD (use COPY instead)
/// - No exposed secrets or credentials
/// - Reasonable file permissions (not 777)
/// - FROM scratch detection (Issue #13)
fn calculate_security_score(source: &str) -> f64 {
    if source.trim().is_empty() {
        return 0.0;
    }

    // Issue #13: Detect if final stage is FROM scratch
    let is_final_stage_scratch = detect_final_stage_scratch(source);

    let mut has_user_directive = false;
    let mut uses_copy_not_add = true;
    let mut has_bad_permissions = false;
    let mut potential_secrets = false;

    for line in source.lines() {
        let trimmed = line.trim();

        // Check for USER directive
        if trimmed.starts_with("USER ") && !trimmed.contains("USER root") {
            has_user_directive = true;
        }

        // Check for ADD (should prefer COPY)
        if trimmed.starts_with("ADD ") && !trimmed.contains(".tar") {
            uses_copy_not_add = false;
        }

        // Check for bad permissions (777, 666)
        if trimmed.contains("chmod 777") || trimmed.contains("chmod 666") {
            has_bad_permissions = true;
        }


}
}

        include!("dockerfile_scoring_part2_incl2.rs");
