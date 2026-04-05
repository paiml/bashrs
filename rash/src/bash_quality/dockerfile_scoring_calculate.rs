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
#[path = "dockerfile_scoring_tests_extracted.rs"]
mod tests_extracted;
