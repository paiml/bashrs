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

        // Check for potential secrets
        if trimmed.contains("PASSWORD")
            || trimmed.contains("SECRET")
            || trimmed.contains("API_KEY")
            || trimmed.contains("TOKEN")
        {
            potential_secrets = true;
        }
    }

    let mut score: f64 = 5.0; // Start at neutral

    // Issue #13: FROM scratch images get bonus for minimal attack surface
    if is_final_stage_scratch {
        score += 4.0; // Scratch = high security (no OS layer)
    } else {
        // Regular images: USER directive (4 points)
        if has_user_directive {
            score += 4.0;
        } else {
            score -= 2.0; // Penalty for running as root
        }
    }

    // Using COPY instead of ADD (1 point)
    if uses_copy_not_add {
        score += 1.0;
    }

    // No bad permissions (maintain score)
    if has_bad_permissions {
        score -= 2.0;
    }

    // No exposed secrets (maintain score)
    if potential_secrets {
        score -= 1.0;
    }

    score.clamp(0.0, 10.0)
}

/// Detect if the final stage is FROM scratch (Issue #13)
fn detect_final_stage_scratch(source: &str) -> bool {
    let mut last_from_is_scratch = false;

    for line in source.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Detect FROM directive
        if let Some(stripped) = trimmed.strip_prefix("FROM ") {
            let from_image = stripped.trim();
            // Check if this FROM is for scratch
            last_from_is_scratch = from_image.starts_with("scratch");
        }
    }

    last_from_is_scratch
}

/// Calculate grade from numeric score
fn calculate_grade(score: f64) -> String {
    match score {
        s if s >= 9.5 => "A+".to_string(),
        s if s >= 9.0 => "A".to_string(),
        s if s >= 8.5 => "B+".to_string(),
        s if s >= 8.0 => "B".to_string(),
        s if s >= 7.5 => "C+".to_string(),
        s if s >= 7.0 => "C".to_string(),
        s if s >= 6.0 => "D".to_string(),
        _ => "F".to_string(),
    }
}

/// Generate improvement suggestions
fn generate_suggestions(source: &str, score: &DockerfileQualityScore) -> Vec<String> {
    let mut suggestions = Vec::new();

    // Issue #13: Detect FROM scratch for appropriate suggestions
    let is_final_stage_scratch = detect_final_stage_scratch(source);

    // Safety suggestions
    if score.safety < 7.0 {
        let mut has_pipefail = false;
        for line in source.lines() {
            if line.contains("set -euo pipefail") {
                has_pipefail = true;
                break;
            }
        }

        if !has_pipefail {
            suggestions.push(
                "Add 'set -euo pipefail &&' at the beginning of RUN commands for better error handling".to_string()
            );
        }
    }

    // Layer optimization suggestions
    if score.layer_optimization < 7.0 {
        suggestions.push("Combine RUN commands with && to reduce image layers".to_string());
        suggestions.push(
            "Clean up package manager cache in the same layer (rm -rf /var/cache/apk/*)"
                .to_string(),
        );
        suggestions.push("Consider using --no-cache flag for package managers".to_string());
    }

    // Determinism suggestions
    if score.determinism < 7.0 {
        let has_latest = source.contains(":latest");
        let has_version_pinning = source.contains('=');

        if has_latest || !has_version_pinning {
            suggestions
                .push("Pin package versions for reproducibility (e.g., curl=8.2.1-r0)".to_string());
            suggestions
                .push("Use specific image tags instead of :latest (e.g., alpine:3.18)".to_string());
        }
    }

    // Security suggestions (Issue #13: skip USER directive for FROM scratch)
    if score.security < 7.0 {
        let has_user = source.contains("USER ");

        // Only suggest USER directive for non-scratch images
        if !is_final_stage_scratch && (!has_user || source.contains("USER root")) {
            suggestions.push("Add USER directive to run container as non-root user".to_string());
            suggestions.push("Create a dedicated user with adduser/addgroup".to_string());
        }
    }

    // Complexity suggestions
    if score.complexity < 7.0 {
        suggestions
            .push("Reduce the number of separate RUN commands by combining them".to_string());
        suggestions
            .push("Consider using multi-stage builds to reduce final image size".to_string());
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_empty_dockerfile() {
        let source = "";
        let score = score_dockerfile(source).unwrap();
        assert_eq!(score.grade, "F");
        assert!(score.score < 6.0);
    }

    #[test]
    fn test_score_good_dockerfile() {
        let source = r#"FROM alpine:3.18

RUN set -euo pipefail && \
    apk add --no-cache curl=8.2.1-r0 && \
    rm -rf /var/cache/apk/*

USER nobody
"#;
        let score = score_dockerfile(source).unwrap();
        assert!(score.score >= 7.0, "Good Dockerfile should score >= 7.0");
        assert!(score.safety >= 7.0);
        assert!(score.determinism >= 7.0);
    }

    #[test]
    fn test_score_excellent_dockerfile() {
        let source = r#"FROM alpine:3.18 AS builder

RUN set -euo pipefail && \
    apk add --no-cache \
        curl=8.2.1-r0 \
        bash=5.2.15-r5 && \
    rm -rf /var/cache/apk/*

FROM alpine:3.18

RUN set -euo pipefail && \
    apk add --no-cache ca-certificates=20230506-r0 && \
    rm -rf /var/cache/apk/* && \
    adduser -D appuser

USER appuser
"#;
        let score = score_dockerfile(source).unwrap();
        assert!(
            score.score >= 8.0,
            "Excellent Dockerfile should score >= 8.0"
        );
        assert!(matches!(score.grade.as_str(), "A" | "A+" | "B" | "B+"));
    }

    #[test]
    fn test_score_bad_dockerfile() {
        let source = r#"FROM alpine

RUN apk update
RUN apk upgrade
RUN apk add curl

CMD /app.sh
"#;
        let score = score_dockerfile(source).unwrap();
        assert!(score.score < 6.0, "Bad Dockerfile should score < 6.0");
        assert!(matches!(score.grade.as_str(), "D" | "F"));
        assert!(!score.suggestions.is_empty());
    }

    #[test]
    fn test_detects_pipefail() {
        let with_pipefail = r#"FROM alpine:3.18
RUN set -euo pipefail && apk add curl
"#;
        let without_pipefail = r#"FROM alpine:3.18
RUN apk add curl
"#;

        let score_with = score_dockerfile(with_pipefail).unwrap();
        let score_without = score_dockerfile(without_pipefail).unwrap();

        assert!(score_with.safety > score_without.safety);
    }

    #[test]
    fn test_detects_version_pinning() {
        let pinned = r#"FROM alpine:3.18
RUN apk add curl=8.2.1-r0
"#;
        let unpinned = r#"FROM alpine:latest
RUN apk add curl
"#;

        let score_pinned = score_dockerfile(pinned).unwrap();
        let score_unpinned = score_dockerfile(unpinned).unwrap();

        assert!(score_pinned.determinism > score_unpinned.determinism);
    }

    #[test]
    fn test_detects_cache_cleanup() {
        let with_cleanup = r#"FROM alpine:3.18
RUN apk add --no-cache curl && rm -rf /var/cache/apk/*
"#;
        let without_cleanup = r#"FROM alpine:3.18
RUN apk add curl
"#;

        let score_with = score_dockerfile(with_cleanup).unwrap();
        let score_without = score_dockerfile(without_cleanup).unwrap();

        assert!(score_with.layer_optimization > score_without.layer_optimization);
    }

    #[test]
    fn test_detects_user_directive() {
        let with_user = r#"FROM alpine:3.18
RUN adduser -D appuser
USER appuser
"#;
        let without_user = r#"FROM alpine:3.18
RUN apk add curl
"#;

        let score_with = score_dockerfile(with_user).unwrap();
        let score_without = score_dockerfile(without_user).unwrap();

        assert!(score_with.security > score_without.security);
    }

    #[test]
    fn test_multistage_build_bonus() {
        let multistage = r#"FROM alpine:3.18 AS builder
RUN apk add curl
FROM alpine:3.18
COPY --from=builder /app /app
"#;
        let single_stage = r#"FROM alpine:3.18
RUN apk add curl
"#;

        let score_multi = score_dockerfile(multistage).unwrap();
        let score_single = score_dockerfile(single_stage).unwrap();

        assert!(score_multi.layer_optimization >= score_single.layer_optimization);
    }

    // Issue #13: FROM scratch images should not be penalized for missing USER directive
    #[test]
    fn test_ISSUE_13_scratch_image_security_score() {
        let scratch_dockerfile = r#"FROM scratch
COPY --from=builder /build/binary /binary
ENTRYPOINT ["/binary"]
"#;
        let score = score_dockerfile(scratch_dockerfile).unwrap();

        // Scratch images should get high security score (no OS layer = minimal attack surface)
        assert!(
            score.security >= 8.0,
            "FROM scratch should score >= 8.0 security (got {})",
            score.security
        );

        // Should NOT suggest adding USER directive for scratch images
        let has_user_suggestion = score
            .suggestions
            .iter()
            .any(|s| s.contains("USER") || s.contains("non-root"));

        assert!(
            !has_user_suggestion,
            "FROM scratch should not suggest USER directive"
        );
    }

    #[test]
    fn test_ISSUE_13_multistage_scratch_final_stage() {
        let multistage_scratch = r#"FROM alpine:3.18 AS builder
RUN apk add --no-cache curl && \
    curl -o /binary https://example.com/binary

FROM scratch
COPY --from=builder /binary /binary
ENTRYPOINT ["/binary"]
"#;
        let score = score_dockerfile(multistage_scratch).unwrap();

        // Final stage is scratch - should have high security score
        assert!(
            score.security >= 8.0,
            "Multi-stage with FROM scratch final should score >= 8.0 security (got {})",
            score.security
        );

        // Should NOT suggest USER directive
        let has_user_suggestion = score
            .suggestions
            .iter()
            .any(|s| s.contains("USER") || s.contains("non-root"));

        assert!(
            !has_user_suggestion,
            "Multi-stage scratch should not suggest USER directive"
        );
    }

    #[test]
    fn test_ISSUE_13_regular_image_still_requires_user() {
        let regular_dockerfile = r#"FROM alpine:3.18
RUN apk add curl
CMD ["/app"]
"#;
        let score = score_dockerfile(regular_dockerfile).unwrap();

        // Regular images should still be penalized for missing USER
        assert!(
            score.security < 8.0,
            "Regular image without USER should score < 8.0 security (got {})",
            score.security
        );

        // Should suggest USER directive for regular images
        let has_user_suggestion = score
            .suggestions
            .iter()
            .any(|s| s.contains("USER") || s.contains("non-root"));

        assert!(
            has_user_suggestion,
            "Regular image should suggest USER directive"
        );
    }
}

// ============================================================================
// Property Tests (EXTREME TDD requirement - 100+ cases)
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Property: Scoring should never panic on any input
    proptest! {
        #[test]
        fn prop_score_never_panics(dockerfile in ".*{0,1000}") {
            let _ = score_dockerfile(&dockerfile);
        }
    }

    // Property: Score should always be in valid range [0.0, 10.0]
    proptest! {
        #[test]
        fn prop_score_in_valid_range(dockerfile in "FROM.*\n.*{0,500}") {
            if let Ok(score) = score_dockerfile(&dockerfile) {
                prop_assert!(score.score >= 0.0 && score.score <= 10.0);
                prop_assert!(score.safety >= 0.0 && score.safety <= 10.0);
                prop_assert!(score.complexity >= 0.0 && score.complexity <= 10.0);
                prop_assert!(score.layer_optimization >= 0.0 && score.layer_optimization <= 10.0);
                prop_assert!(score.determinism >= 0.0 && score.determinism <= 10.0);
                prop_assert!(score.security >= 0.0 && score.security <= 10.0);
            }
        }
    }

    // Property: Adding set -euo pipefail should never decrease safety score
    proptest! {
        #[test]
        fn prop_pipefail_improves_safety(base_commands in prop::collection::vec("apk add [a-z]+", 1..5)) {
            let without_pipefail = format!("FROM alpine:3.18\n{}",
                base_commands.iter().map(|c| format!("RUN {}", c)).collect::<Vec<_>>().join("\n")
            );

            let with_pipefail = format!("FROM alpine:3.18\n{}",
                base_commands.iter().map(|c| format!("RUN set -euo pipefail && {}", c)).collect::<Vec<_>>().join("\n")
            );

            let score_without = score_dockerfile(&without_pipefail).expect("should score");
            let score_with = score_dockerfile(&with_pipefail).expect("should score");

            prop_assert!(score_with.safety >= score_without.safety,
                "Adding pipefail should improve or maintain safety score");
        }
    }

    // Property: Version pinning should improve determinism
    proptest! {
        #[test]
        fn prop_version_pinning_improves_determinism(packages in prop::collection::vec("[a-z]{3,10}", 1..5)) {
            let without_pinning = format!("FROM alpine:3.18\nRUN apk add {}",
                packages.join(" ")
            );

            let with_pinning = format!("FROM alpine:3.18\nRUN apk add {}",
                packages.iter().map(|p| format!("{}=1.0.0-r0", p)).collect::<Vec<_>>().join(" ")
            );

            let score_without = score_dockerfile(&without_pinning).expect("should score");
            let score_with = score_dockerfile(&with_pinning).expect("should score");

            prop_assert!(score_with.determinism >= score_without.determinism,
                "Version pinning should improve determinism score");
        }
    }

    // Property: Adding USER directive should improve security
    proptest! {
        #[test]
        fn prop_user_directive_improves_security(base_tag in "(3\\.[0-9]{1,2}|latest)") {
            let without_user = format!("FROM alpine:{}\nRUN apk add curl", base_tag);
            let with_user = format!("FROM alpine:{}\nRUN apk add curl\nUSER nobody", base_tag);

            let score_without = score_dockerfile(&without_user).expect("should score");
            let score_with = score_dockerfile(&with_user).expect("should score");

            prop_assert!(score_with.security >= score_without.security,
                "Adding USER directive should improve security score");
        }
    }

    // Property: Combining RUN commands should improve layer optimization
    proptest! {
        #[test]
        fn prop_combined_commands_improve_layering(commands in prop::collection::vec("apk add [a-z]+", 2..4)) {
            let separate = format!("FROM alpine:3.18\n{}",
                commands.iter().map(|c| format!("RUN {}", c)).collect::<Vec<_>>().join("\n")
            );

            let combined = format!("FROM alpine:3.18\nRUN {}",
                commands.join(" && ")
            );

            let score_separate = score_dockerfile(&separate).expect("should score");
            let score_combined = score_dockerfile(&combined).expect("should score");

            prop_assert!(score_combined.layer_optimization >= score_separate.layer_optimization,
                "Combining commands should improve layer optimization");
        }
    }

    // Property: Adding cache cleanup should improve layer optimization
    proptest! {
        #[test]
        fn prop_cache_cleanup_improves_layering(packages in prop::collection::vec("[a-z]{3,8}", 1..3)) {
            let without_cleanup = format!("FROM alpine:3.18\nRUN apk add {}", packages.join(" "));
            let with_cleanup = format!("FROM alpine:3.18\nRUN apk add {} && rm -rf /var/cache/apk/*", packages.join(" "));

            let score_without = score_dockerfile(&without_cleanup).expect("should score");
            let score_with = score_dockerfile(&with_cleanup).expect("should score");

            prop_assert!(score_with.layer_optimization >= score_without.layer_optimization,
                "Cache cleanup should improve layer optimization");
        }
    }

    // Property: Grade should be consistent with score
    proptest! {
        #[test]
        fn prop_grade_consistent_with_score(dockerfile in "FROM [a-z]+:[0-9\\.]+\n.*{0,200}") {
            if let Ok(result) = score_dockerfile(&dockerfile) {
                let expected_grade = calculate_grade(result.score);
                prop_assert_eq!(result.grade, expected_grade,
                    "Grade should match score value");
            }
        }
    }

    // Property: Score is deterministic (same input = same output)
    proptest! {
        #[test]
        fn prop_score_deterministic(dockerfile in "FROM.*\n.*{0,300}") {
            if let (Ok(score1), Ok(score2)) = (score_dockerfile(&dockerfile), score_dockerfile(&dockerfile)) {
                prop_assert_eq!(score1.score, score2.score, "Score should be deterministic");
                prop_assert_eq!(score1.grade, score2.grade, "Grade should be deterministic");
            }
        }
    }

    // Property: Empty or minimal Dockerfiles should get low scores
    proptest! {
        #[test]
        fn prop_minimal_dockerfile_low_score(base_image in "[a-z]+") {
            let minimal = format!("FROM {}", base_image);
            let score = score_dockerfile(&minimal).expect("should score");

            prop_assert!(score.score < 8.0,
                "Minimal Dockerfile should not score highly");
        }
    }

    // Property: Using :latest tag should decrease determinism
    proptest! {
        #[test]
        fn prop_latest_tag_reduces_determinism(base_image in "[a-z]{3,10}") {
            let with_latest = format!("FROM {}:latest\nRUN apk add curl", base_image);
            let with_version = format!("FROM {}:3.18\nRUN apk add curl", base_image);

            let score_latest = score_dockerfile(&with_latest).expect("should score");
            let score_versioned = score_dockerfile(&with_version).expect("should score");

            prop_assert!(score_versioned.determinism >= score_latest.determinism,
                "Specific version should have better determinism than :latest");
        }
    }
}
