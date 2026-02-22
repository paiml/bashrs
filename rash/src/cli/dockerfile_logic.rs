//! Pure logic functions for Dockerfile operations.
//!
//! This module contains functions extracted from `commands.rs` that have no
//! side-effects (no I/O, no process spawning). They can be tested in isolation
//! without touching the filesystem.

use crate::cli::logic::purify_dockerfile_source;
use crate::linter::docker_profiler::{PlatformProfile, SizeEstimate};
use crate::models::{Error, Result};

// ---------------------------------------------------------------------------
// Purification
// ---------------------------------------------------------------------------

/// Purify a Dockerfile source string.
///
/// Applies all determinism / security transformations from
/// [`purify_dockerfile_source`] and returns the result.
///
/// # Arguments
/// * `source`    – Raw Dockerfile text.
/// * `skip_user` – When `true`, skip the "add non-root USER" transformation.
pub(crate) fn purify_dockerfile(source: &str, skip_user: bool) -> Result<String> {
    Ok(purify_dockerfile_source(source, skip_user))
}

// ---------------------------------------------------------------------------
// Size parsing
// ---------------------------------------------------------------------------

/// Parse a human-readable size string ("10GB", "500MB") into bytes.
///
/// Returns `None` when `max_size` is `None` or the string is not recognised.
pub(crate) fn parse_size_limit(max_size: Option<&str>) -> Option<u64> {
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

// ---------------------------------------------------------------------------
// Size limit checking
// ---------------------------------------------------------------------------

/// Check whether the estimated image size exceeds the effective limit.
///
/// When `strict` is `true` and the size is over the limit, returns
/// `Err(Error::Validation(...))`. Prints a human-readable table entry.
///
/// When `effective_limit == u64::MAX` (i.e. `PlatformProfile::Standard`) the
/// function returns `Ok(())` immediately.
pub(crate) fn size_check_limit_check(
    estimate: &SizeEstimate,
    platform: &PlatformProfile,
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

// ---------------------------------------------------------------------------
// Build time estimation
// ---------------------------------------------------------------------------

/// Estimate the Docker build time as a human-readable string.
///
/// Uses a rough heuristic:
/// * 1 second base per layer
/// * 1 second per 100 MB of layer content
/// * +10 seconds for `apt-get install` layers
/// * +5 seconds for `pip install` layers
/// * +5 seconds for `npm install` layers
pub(crate) fn estimate_build_time(estimate: &SizeEstimate) -> String {
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
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::linter::docker_profiler::LayerEstimate;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn make_estimate(total_bytes: u64, layers: Vec<LayerEstimate>) -> SizeEstimate {
        SizeEstimate {
            base_image_size: 0,
            base_image: "scratch".to_string(),
            layer_estimates: layers,
            total_estimated: total_bytes,
            bloat_patterns: vec![],
            warnings: vec![],
        }
    }

    fn make_layer(content: &str, size: u64) -> LayerEstimate {
        LayerEstimate {
            layer_num: 1,
            instruction: "RUN".to_string(),
            content: content.to_string(),
            line: 1,
            estimated_size: size,
            cached: false,
            notes: None,
        }
    }

    // -----------------------------------------------------------------------
    // parse_size_limit
    // -----------------------------------------------------------------------

    #[test]
    fn test_DOCKERFILE_LOGIC_001_parse_size_limit_none_input() {
        assert_eq!(parse_size_limit(None), None);
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_002_parse_size_limit_gb_whole() {
        assert_eq!(parse_size_limit(Some("10GB")), Some(10_000_000_000));
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_003_parse_size_limit_gb_fractional() {
        let result = parse_size_limit(Some("1.5GB")).unwrap();
        assert_eq!(result, 1_500_000_000);
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_004_parse_size_limit_mb_whole() {
        assert_eq!(parse_size_limit(Some("500MB")), Some(500_000_000));
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_005_parse_size_limit_mb_fractional() {
        let result = parse_size_limit(Some("0.5MB")).unwrap();
        assert_eq!(result, 500_000);
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_006_parse_size_limit_lowercase_gb() {
        // Parsing is case-insensitive
        assert_eq!(parse_size_limit(Some("2gb")), Some(2_000_000_000));
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_007_parse_size_limit_lowercase_mb() {
        assert_eq!(parse_size_limit(Some("100mb")), Some(100_000_000));
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_008_parse_size_limit_invalid_unit() {
        // "KB" is not a supported unit
        assert_eq!(parse_size_limit(Some("100KB")), None);
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_009_parse_size_limit_plain_number() {
        // A bare number without a unit is not supported
        assert_eq!(parse_size_limit(Some("1024")), None);
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_010_parse_size_limit_empty_string() {
        assert_eq!(parse_size_limit(Some("")), None);
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_011_parse_size_limit_gb_with_space() {
        // Leading/trailing whitespace inside the numeric part should be trimmed
        let result = parse_size_limit(Some("5 GB"));
        assert_eq!(result, Some(5_000_000_000));
    }

    // -----------------------------------------------------------------------
    // size_check_limit_check
    // -----------------------------------------------------------------------

    #[test]
    fn test_DOCKERFILE_LOGIC_012_size_check_standard_platform_unlimited() {
        // PlatformProfile::Standard has MAX limit – always passes
        let estimate = make_estimate(999_999_999_999, vec![]);
        let result = size_check_limit_check(&estimate, &PlatformProfile::Standard, None, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_013_size_check_within_limit_non_strict() {
        let estimate = make_estimate(5_000_000_000, vec![]);
        let result = size_check_limit_check(
            &estimate,
            &PlatformProfile::Coursera,
            None,
            false,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_014_size_check_within_limit_strict() {
        let estimate = make_estimate(5_000_000_000, vec![]);
        let result = size_check_limit_check(
            &estimate,
            &PlatformProfile::Coursera,
            None,
            true,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_015_size_check_exceeds_limit_non_strict() {
        // Over 10 GB (Coursera limit) but strict=false → Ok
        let estimate = make_estimate(12_000_000_000, vec![]);
        let result = size_check_limit_check(
            &estimate,
            &PlatformProfile::Coursera,
            None,
            false,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_016_size_check_exceeds_limit_strict() {
        let estimate = make_estimate(12_000_000_000, vec![]);
        let result = size_check_limit_check(
            &estimate,
            &PlatformProfile::Coursera,
            None,
            true,
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("exceeds limit"));
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_017_size_check_custom_limit_respected() {
        // Custom limit of 1 GB, estimate is 2 GB → error in strict mode
        let estimate = make_estimate(2_000_000_000, vec![]);
        let result = size_check_limit_check(
            &estimate,
            &PlatformProfile::Standard, // would be unlimited without custom
            Some(1_000_000_000),
            true,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_018_size_check_custom_limit_passes() {
        let estimate = make_estimate(500_000_000, vec![]);
        let result = size_check_limit_check(
            &estimate,
            &PlatformProfile::Standard,
            Some(1_000_000_000),
            true,
        );
        assert!(result.is_ok());
    }

    // -----------------------------------------------------------------------
    // estimate_build_time
    // -----------------------------------------------------------------------

    #[test]
    fn test_DOCKERFILE_LOGIC_019_estimate_build_time_empty_layers() {
        let estimate = make_estimate(0, vec![]);
        let result = estimate_build_time(&estimate);
        assert_eq!(result, "~0s");
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_020_estimate_build_time_single_small_layer() {
        let layers = vec![make_layer("COPY . /app", 10_000_000)]; // 10 MB → 0 extra seconds
        let estimate = make_estimate(10_000_000, layers);
        let result = estimate_build_time(&estimate);
        // 1 base + 0 size-based = 1 second, under 60
        assert_eq!(result, "~1s");
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_021_estimate_build_time_apt_get_install() {
        let layers = vec![make_layer("RUN apt-get install -y curl", 50_000_000)];
        let estimate = make_estimate(50_000_000, layers);
        let result = estimate_build_time(&estimate);
        // 1 base + 0 size-based + 10 apt = 11 seconds
        assert_eq!(result, "~11s");
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_022_estimate_build_time_pip_install() {
        let layers = vec![make_layer("RUN pip install numpy", 50_000_000)];
        let estimate = make_estimate(50_000_000, layers);
        let result = estimate_build_time(&estimate);
        // 1 base + 0 + 5 pip = 6 seconds
        assert_eq!(result, "~6s");
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_023_estimate_build_time_npm_install() {
        let layers = vec![make_layer("RUN npm install", 50_000_000)];
        let estimate = make_estimate(50_000_000, layers);
        let result = estimate_build_time(&estimate);
        // 1 base + 0 + 5 npm = 6 seconds
        assert_eq!(result, "~6s");
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_024_estimate_build_time_large_layer_over_minute() {
        // 6 GB layer → 60 size seconds
        let layers = vec![make_layer("COPY model /model", 6_000_000_000)];
        let estimate = make_estimate(6_000_000_000, layers);
        let result = estimate_build_time(&estimate);
        // 1 base + 60 = 61 seconds → 1m 1s
        assert_eq!(result, "~1m 1s");
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_025_estimate_build_time_multiple_layers() {
        let layers = vec![
            make_layer("RUN apt-get install -y git", 50_000_000),
            make_layer("RUN pip install torch", 2_000_000_000),
            make_layer("RUN npm install", 200_000_000),
        ];
        let estimate = make_estimate(2_250_000_000, layers);
        let result = estimate_build_time(&estimate);
        // Layer 1: 1 + 0 + 10 = 11
        // Layer 2: 1 + 20 + 5 = 26
        // Layer 3: 1 + 2 + 5 = 8
        // Total = 45 s → under 60
        assert_eq!(result, "~45s");
    }

    // -----------------------------------------------------------------------
    // purify_dockerfile
    // -----------------------------------------------------------------------

    #[test]
    fn test_DOCKERFILE_LOGIC_026_purify_dockerfile_basic_passthrough() {
        let source = "FROM alpine:3.18\nCMD [\"sh\"]\n";
        let result = purify_dockerfile(source, false);
        assert!(result.is_ok());
        // Result should contain the FROM line
        let output = result.unwrap();
        assert!(output.contains("FROM alpine:3.18"));
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_027_purify_dockerfile_adds_user_directive() {
        // A Dockerfile without a USER directive but with CMD should get one added (skip_user=false)
        let source = "FROM ubuntu:22.04\nRUN apt-get update\nCMD [\"bash\"]\n";
        let result = purify_dockerfile(source, false);
        assert!(result.is_ok());
        // The purified output should contain a USER directive (inserted before CMD)
        let output = result.unwrap();
        assert!(output.contains("USER"), "Expected USER directive to be added before CMD");
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_028_purify_dockerfile_skip_user() {
        // When skip_user=true the USER directive should NOT be forced in
        let source = "FROM ubuntu:22.04\nRUN apt-get update\n";
        let without_skip = purify_dockerfile(source, false).unwrap();
        let with_skip = purify_dockerfile(source, true).unwrap();
        // Both are valid results; with skip_user the USER line may or may not
        // appear, but the output must at minimum contain the FROM line.
        assert!(with_skip.contains("FROM ubuntu:22.04"));
        // When skip is on the output should have fewer or equal USER occurrences
        let user_count_without = without_skip.matches("USER").count();
        let user_count_with = with_skip.matches("USER").count();
        assert!(user_count_with <= user_count_without);
    }

    #[test]
    fn test_DOCKERFILE_LOGIC_029_purify_dockerfile_empty_source() {
        let result = purify_dockerfile("", false);
        assert!(result.is_ok());
    }
}
