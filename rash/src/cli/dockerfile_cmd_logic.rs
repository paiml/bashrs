//! Pure logic extracted from Dockerfile CLI commands (no I/O).
//!
//! See also `dockerfile_logic` for: `purify_dockerfile`, `parse_size_limit`,
//! `size_check_limit_check`, `estimate_build_time`.

use crate::cli::args::LintProfileArg;
use crate::linter::docker_profiler::{PlatformProfile, SizeEstimate};
use crate::linter::rules::LintProfile;
use crate::linter::Diagnostic;
use std::collections::HashSet;

/// `LintProfileArg` -> `PlatformProfile`. Defaults to `Standard`.
pub(crate) fn resolve_platform_profile(arg: Option<&LintProfileArg>) -> PlatformProfile {
    match arg {
        Some(LintProfileArg::Coursera) => PlatformProfile::Coursera,
        _ => PlatformProfile::Standard,
    }
}

/// `LintProfileArg` -> linter `LintProfile`. Defaults to `Standard`.
pub(crate) fn resolve_lint_profile(arg: Option<&LintProfileArg>) -> LintProfile {
    match arg {
        Some(LintProfileArg::Coursera) => LintProfile::Coursera,
        Some(LintProfileArg::DevContainer) => LintProfile::DevContainer,
        _ => LintProfile::Standard,
    }
}

/// Filter diagnostics by comma-separated rule codes (`None` keeps all).
pub(crate) fn filter_lint_diagnostics(
    diagnostics: Vec<Diagnostic>,
    rules: Option<&str>,
) -> Vec<Diagnostic> {
    match rules {
        Some(rule_filter) => {
            let allowed: HashSet<&str> = rule_filter.split(',').collect();
            diagnostics
                .into_iter()
                .filter(|d| allowed.contains(d.code.as_str()))
                .collect()
        }
        None => diagnostics,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProfileSections {
    pub build: bool,
    pub layers: bool,
    pub startup: bool,
    pub memory: bool,
    pub cpu: bool,
    pub simulate_limits: bool,
}

/// Merge individual boolean flags with the `--full` override.
pub(crate) fn resolve_profile_sections(
    build: bool,
    layers: bool,
    startup: bool,
    memory: bool,
    cpu: bool,
    simulate_limits: bool,
    full: bool,
) -> ProfileSections {
    if full {
        ProfileSections {
            build: true,
            layers: true,
            startup: true,
            memory: true,
            cpu: true,
            simulate_limits,
        }
    } else {
        ProfileSections {
            build,
            layers,
            startup,
            memory,
            cpu,
            simulate_limits,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CourseraValidationResult {
    pub size_ok: bool,
    pub estimated_gb: f64,
    pub max_size_gb: f64,
}

/// Validate image estimate against Coursera constraints. `None` if not Coursera.
pub(crate) fn validate_coursera_size(
    estimate: &SizeEstimate,
    platform: &PlatformProfile,
) -> Option<CourseraValidationResult> {
    if *platform != PlatformProfile::Coursera {
        return None;
    }
    let max_size_gb = platform.max_size_bytes() as f64 / 1_000_000_000.0;
    let estimated_gb = estimate.total_estimated as f64 / 1_000_000_000.0;
    let size_ok = estimate.total_estimated < platform.max_size_bytes();
    Some(CourseraValidationResult { size_ok, estimated_gb, max_size_gb })
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LayerDetail {
    pub layer_num: usize,
    pub instruction: String,
    pub line: usize,
    pub cached: bool,
    pub notes: Option<String>,
}

/// Extract layer details from a `SizeEstimate` for display.
pub(crate) fn extract_layer_details(estimate: &SizeEstimate) -> Vec<LayerDetail> {
    estimate
        .layer_estimates
        .iter()
        .map(|layer| LayerDetail {
            layer_num: layer.layer_num,
            instruction: layer.instruction.clone(),
            line: layer.line,
            cached: layer.cached,
            notes: layer.notes.clone(),
        })
        .collect()
}

/// Profile data for JSON serialisation.
#[derive(Debug, Clone)]
pub(crate) struct ProfileData {
    pub platform_label: String,
    pub layer_count: usize,
    pub build_time_estimate: String,
    pub base_image: String,
    pub base_image_bytes: u64,
    pub total_estimated_bytes: u64,
    pub bloat_pattern_count: usize,
    pub max_size_bytes: u64,
    pub max_memory_bytes: u64,
    pub max_startup_ms: u64,
}

/// Build profile data from a size estimate and platform profile (no I/O).
pub(crate) fn build_profile_data(
    estimate: &SizeEstimate,
    platform: &PlatformProfile,
    build_time: &str,
) -> ProfileData {
    ProfileData {
        platform_label: format!("{:?}", platform),
        layer_count: estimate.layer_estimates.len(),
        build_time_estimate: build_time.to_string(),
        base_image: estimate.base_image.clone(),
        base_image_bytes: estimate.base_image_size,
        total_estimated_bytes: estimate.total_estimated,
        bloat_pattern_count: estimate.bloat_patterns.len(),
        max_size_bytes: platform.max_size_bytes(),
        max_memory_bytes: platform.max_memory_bytes(),
        max_startup_ms: platform.max_startup_ms(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedValidationProfiles {
    pub lint_profile: LintProfile,
    pub platform_profile: PlatformProfile,
}

/// Resolve lint and platform profiles from a single optional CLI argument.
pub(crate) fn resolve_validation_profiles(
    arg: Option<&LintProfileArg>,
) -> ResolvedValidationProfiles {
    ResolvedValidationProfiles {
        lint_profile: resolve_lint_profile(arg),
        platform_profile: resolve_platform_profile(arg),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiagnosticCounts {
    pub errors: usize,
    pub warnings: usize,
    pub total: usize,
}

/// Count errors and warnings in a slice of diagnostics.
pub(crate) fn count_diagnostics(diagnostics: &[Diagnostic]) -> DiagnosticCounts {
    let errors = diagnostics
        .iter()
        .filter(|d| d.severity == crate::linter::Severity::Error)
        .count();
    let warnings = diagnostics
        .iter()
        .filter(|d| d.severity == crate::linter::Severity::Warning)
        .count();
    DiagnosticCounts { errors, warnings, total: diagnostics.len() }
}

/// Format bytes as a human-readable GB string with two decimal places.
pub(crate) fn format_bytes_as_gb(bytes: u64) -> String {
    format!("{:.2}GB", bytes as f64 / 1_000_000_000.0)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::linter::docker_profiler::LayerEstimate;
    use crate::linter::{Fix, Severity, Span};

    fn est(total: u64, layers: Vec<LayerEstimate>) -> SizeEstimate {
        SizeEstimate {
            base_image_size: 0,
            base_image: "scratch".to_string(),
            layer_estimates: layers,
            total_estimated: total,
            bloat_patterns: vec![],
            warnings: vec![],
        }
    }

    fn layer(instr: &str, line: usize, cached: bool) -> LayerEstimate {
        LayerEstimate {
            layer_num: 1,
            instruction: instr.to_string(),
            content: String::new(),
            line,
            estimated_size: 0,
            cached,
            notes: None,
        }
    }

    fn diag(code: &str, sev: Severity) -> Diagnostic {
        Diagnostic::new(code, sev, "msg", Span::new(1, 1, 1, 1))
    }

    // -- resolve_platform_profile --

    #[test]
    fn test_DOCKERFILE_CMD_001_resolve_platform_none() {
        assert_eq!(resolve_platform_profile(None), PlatformProfile::Standard);
    }

    #[test]
    fn test_DOCKERFILE_CMD_002_resolve_platform_coursera() {
        assert_eq!(resolve_platform_profile(Some(&LintProfileArg::Coursera)), PlatformProfile::Coursera);
    }

    #[test]
    fn test_DOCKERFILE_CMD_003_resolve_platform_standard() {
        assert_eq!(resolve_platform_profile(Some(&LintProfileArg::Standard)), PlatformProfile::Standard);
    }

    #[test]
    fn test_DOCKERFILE_CMD_004_resolve_platform_devcontainer() {
        assert_eq!(resolve_platform_profile(Some(&LintProfileArg::DevContainer)), PlatformProfile::Standard);
    }

    // -- resolve_lint_profile --

    #[test]
    fn test_DOCKERFILE_CMD_005_resolve_lint_none() {
        assert_eq!(resolve_lint_profile(None), LintProfile::Standard);
    }

    #[test]
    fn test_DOCKERFILE_CMD_006_resolve_lint_coursera() {
        assert_eq!(resolve_lint_profile(Some(&LintProfileArg::Coursera)), LintProfile::Coursera);
    }

    #[test]
    fn test_DOCKERFILE_CMD_007_resolve_lint_devcontainer() {
        assert_eq!(resolve_lint_profile(Some(&LintProfileArg::DevContainer)), LintProfile::DevContainer);
    }

    #[test]
    fn test_DOCKERFILE_CMD_008_resolve_lint_standard() {
        assert_eq!(resolve_lint_profile(Some(&LintProfileArg::Standard)), LintProfile::Standard);
    }

    // -- filter_lint_diagnostics --

    #[test]
    fn test_DOCKERFILE_CMD_009_filter_none_passes_all() {
        let d = vec![diag("DL001", Severity::Error), diag("DL002", Severity::Warning)];
        assert_eq!(filter_lint_diagnostics(d, None).len(), 2);
    }

    #[test]
    fn test_DOCKERFILE_CMD_010_filter_single_rule() {
        let d = vec![diag("DL001", Severity::Error), diag("DL002", Severity::Warning)];
        let r = filter_lint_diagnostics(d, Some("DL001"));
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].code, "DL001");
    }

    #[test]
    fn test_DOCKERFILE_CMD_011_filter_multiple_rules() {
        let d = vec![diag("DL001", Severity::Error), diag("DL002", Severity::Warning), diag("DL003", Severity::Error)];
        let r = filter_lint_diagnostics(d, Some("DL001,DL003"));
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn test_DOCKERFILE_CMD_012_filter_no_match() {
        let d = vec![diag("DL001", Severity::Error)];
        assert!(filter_lint_diagnostics(d, Some("DL999")).is_empty());
    }

    #[test]
    fn test_DOCKERFILE_CMD_013_filter_empty_input() {
        assert!(filter_lint_diagnostics(vec![], Some("DL001")).is_empty());
    }

    // -- resolve_profile_sections --

    #[test]
    fn test_DOCKERFILE_CMD_014_sections_full_overrides() {
        let s = resolve_profile_sections(false, false, false, false, false, false, true);
        assert!(s.build && s.layers && s.startup && s.memory && s.cpu && !s.simulate_limits);
    }

    #[test]
    fn test_DOCKERFILE_CMD_015_sections_individual() {
        let s = resolve_profile_sections(true, false, true, false, true, false, false);
        assert!(s.build && !s.layers && s.startup && !s.memory && s.cpu);
    }

    #[test]
    fn test_DOCKERFILE_CMD_016_sections_all_false() {
        let s = resolve_profile_sections(false, false, false, false, false, false, false);
        assert!(!s.build && !s.layers && !s.startup && !s.memory && !s.cpu);
    }

    #[test]
    fn test_DOCKERFILE_CMD_017_sections_full_simulate() {
        let s = resolve_profile_sections(false, false, false, false, false, true, true);
        assert!(s.build && s.simulate_limits);
    }

    // -- validate_coursera_size --

    #[test]
    fn test_DOCKERFILE_CMD_018_coursera_standard_none() {
        assert!(validate_coursera_size(&est(5_000_000_000, vec![]), &PlatformProfile::Standard).is_none());
    }

    #[test]
    fn test_DOCKERFILE_CMD_019_coursera_within() {
        let r = validate_coursera_size(&est(5_000_000_000, vec![]), &PlatformProfile::Coursera).unwrap();
        assert!(r.size_ok);
        assert!((r.estimated_gb - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_DOCKERFILE_CMD_020_coursera_exceeds() {
        let r = validate_coursera_size(&est(12_000_000_000, vec![]), &PlatformProfile::Coursera).unwrap();
        assert!(!r.size_ok);
    }

    #[test]
    fn test_DOCKERFILE_CMD_021_coursera_max_gb() {
        let r = validate_coursera_size(&est(5_000_000_000, vec![]), &PlatformProfile::Coursera).unwrap();
        assert!(r.max_size_gb > 0.0);
    }

    // -- extract_layer_details --

    #[test]
    fn test_DOCKERFILE_CMD_022_layers_empty() {
        assert!(extract_layer_details(&est(0, vec![])).is_empty());
    }

    #[test]
    fn test_DOCKERFILE_CMD_023_layers_preserves() {
        let d = extract_layer_details(&est(0, vec![layer("RUN", 5, false), layer("COPY", 10, true)]));
        assert_eq!(d.len(), 2);
        assert_eq!(d[0].instruction, "RUN");
        assert!(d[1].cached);
    }

    #[test]
    fn test_DOCKERFILE_CMD_024_layers_notes() {
        let mut l = layer("RUN", 1, false);
        l.notes = Some("pkg".to_string());
        let d = extract_layer_details(&est(0, vec![l]));
        assert_eq!(d[0].notes.as_deref(), Some("pkg"));
    }

    // -- build_profile_data --

    #[test]
    fn test_DOCKERFILE_CMD_025_profile_standard() {
        let d = build_profile_data(&est(500_000_000, vec![]), &PlatformProfile::Standard, "~5s");
        assert_eq!(d.platform_label, "Standard");
        assert_eq!(d.total_estimated_bytes, 500_000_000);
    }

    #[test]
    fn test_DOCKERFILE_CMD_026_profile_coursera() {
        let d = build_profile_data(
            &est(2_000_000_000, vec![layer("RUN", 1, false), layer("COPY", 3, true)]),
            &PlatformProfile::Coursera,
            "~20s",
        );
        assert_eq!(d.layer_count, 2);
        assert!(d.max_size_bytes > 0);
    }

    #[test]
    fn test_DOCKERFILE_CMD_027_profile_base_image() {
        let mut e = est(0, vec![]);
        e.base_image = "ubuntu:22.04".to_string();
        e.base_image_size = 77_000_000;
        let d = build_profile_data(&e, &PlatformProfile::Standard, "~0s");
        assert_eq!(d.base_image, "ubuntu:22.04");
        assert_eq!(d.base_image_bytes, 77_000_000);
    }

    // -- resolve_validation_profiles --

    #[test]
    fn test_DOCKERFILE_CMD_028_validation_none() {
        let p = resolve_validation_profiles(None);
        assert_eq!(p.lint_profile, LintProfile::Standard);
        assert_eq!(p.platform_profile, PlatformProfile::Standard);
    }

    #[test]
    fn test_DOCKERFILE_CMD_029_validation_coursera() {
        let p = resolve_validation_profiles(Some(&LintProfileArg::Coursera));
        assert_eq!(p.lint_profile, LintProfile::Coursera);
        assert_eq!(p.platform_profile, PlatformProfile::Coursera);
    }

    #[test]
    fn test_DOCKERFILE_CMD_030_validation_devcontainer() {
        let p = resolve_validation_profiles(Some(&LintProfileArg::DevContainer));
        assert_eq!(p.lint_profile, LintProfile::DevContainer);
        assert_eq!(p.platform_profile, PlatformProfile::Standard);
    }

    // -- count_diagnostics --

    #[test]
    fn test_DOCKERFILE_CMD_031_count_empty() {
        let c = count_diagnostics(&[]);
        assert_eq!((c.errors, c.warnings, c.total), (0, 0, 0));
    }

    #[test]
    fn test_DOCKERFILE_CMD_032_count_mixed() {
        let d = [diag("E", Severity::Error), diag("W", Severity::Warning), diag("E2", Severity::Error), diag("I", Severity::Info)];
        let c = count_diagnostics(&d);
        assert_eq!((c.errors, c.warnings, c.total), (2, 1, 4));
    }

    #[test]
    fn test_DOCKERFILE_CMD_033_count_all_errors() {
        let c = count_diagnostics(&[diag("E1", Severity::Error), diag("E2", Severity::Error)]);
        assert_eq!((c.errors, c.warnings), (2, 0));
    }

    #[test]
    fn test_DOCKERFILE_CMD_034_count_all_warnings() {
        let c = count_diagnostics(&[diag("W1", Severity::Warning), diag("W2", Severity::Warning), diag("W3", Severity::Warning)]);
        assert_eq!((c.errors, c.warnings), (0, 3));
    }

    // -- format_bytes_as_gb --

    #[test]
    fn test_DOCKERFILE_CMD_035_format_zero() { assert_eq!(format_bytes_as_gb(0), "0.00GB"); }

    #[test]
    fn test_DOCKERFILE_CMD_036_format_1gb() { assert_eq!(format_bytes_as_gb(1_000_000_000), "1.00GB"); }

    #[test]
    fn test_DOCKERFILE_CMD_037_format_frac() { assert_eq!(format_bytes_as_gb(1_500_000_000), "1.50GB"); }

    #[test]
    fn test_DOCKERFILE_CMD_038_format_small() { assert_eq!(format_bytes_as_gb(500_000), "0.00GB"); }

    #[test]
    fn test_DOCKERFILE_CMD_039_format_large() { assert_eq!(format_bytes_as_gb(10_000_000_000), "10.00GB"); }

    // -- filter preserves fix --

    #[test]
    fn test_DOCKERFILE_CMD_040_filter_preserves_fix() {
        let mut d = diag("DL001", Severity::Error);
        d.fix = Some(Fix::new("use COPY instead"));
        let r = filter_lint_diagnostics(vec![d], Some("DL001"));
        assert!(r[0].fix.is_some());
    }
}
