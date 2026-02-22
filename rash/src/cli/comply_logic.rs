//! Comply command pure logic helpers.
//!
//! This module contains stateless, I/O-free functions extracted from
//! `commands.rs` for the `comply` subcommand family. All functions here
//! are unit-testable without filesystem access (except `comply_load_or_default`,
//! which touches the filesystem but contains no printing).

use std::path::Path;

use crate::cli::args::ComplyScopeArg;
use crate::models::{Error, Result};

/// Load comply config from `path`, or return a built-in default.
///
/// Does not print anything; filesystem access only.
pub(crate) fn comply_load_or_default(path: &Path) -> crate::comply::config::ComplyConfig {
    use crate::comply::config::ComplyConfig;
    let version = env!("CARGO_PKG_VERSION");
    if ComplyConfig::exists(path) {
        ComplyConfig::load(path).unwrap_or_else(|| ComplyConfig::new_default(version))
    } else {
        ComplyConfig::new_default(version)
    }
}

/// Map a CLI scope argument to the internal `Scope` enum used by the runner.
///
/// `ComplyScopeArg::All` maps to `None` (meaning "no scope filter").
#[must_use]
pub(crate) fn comply_scope_filter(
    scope: Option<ComplyScopeArg>,
) -> Option<crate::comply::config::Scope> {
    scope.and_then(|s| match s {
        ComplyScopeArg::Project => Some(crate::comply::config::Scope::Project),
        ComplyScopeArg::User => Some(crate::comply::config::Scope::User),
        ComplyScopeArg::System => Some(crate::comply::config::Scope::System),
        ComplyScopeArg::All => None,
    })
}

/// Apply a CLI scope argument to the mutable `ComplyConfig` scope flags.
pub(crate) fn apply_comply_scope(
    config: &mut crate::comply::config::ComplyConfig,
    scope: ComplyScopeArg,
) {
    match scope {
        ComplyScopeArg::Project => {
            config.scopes.user = false;
            config.scopes.system = false;
        }
        ComplyScopeArg::User => {
            config.scopes.user = true;
            config.scopes.system = false;
        }
        ComplyScopeArg::System => {
            config.scopes.user = false;
            config.scopes.system = true;
        }
        ComplyScopeArg::All => {
            config.scopes.user = true;
            config.scopes.system = true;
        }
    }
}

/// Enable all rule flags in a `ComplyConfig` (strict mode).
pub(crate) fn apply_comply_strict(config: &mut crate::comply::config::ComplyConfig) {
    config.rules.posix = true;
    config.rules.determinism = true;
    config.rules.idempotency = true;
    config.rules.security = true;
    config.rules.quoting = true;
    config.rules.shellcheck = true;
    config.rules.makefile_safety = true;
    config.rules.dockerfile_best = true;
    config.rules.config_hygiene = true;
    config.rules.pzsh_budget = "10ms".to_string();
}

/// Enforce `min_score` and `max_violations` thresholds from `comply.toml`.
///
/// Returns `Err` if either threshold is exceeded.
pub(crate) fn comply_enforce_thresholds(
    score: &crate::comply::scoring::ProjectScore,
    config: &crate::comply::config::ComplyConfig,
) -> Result<()> {
    use crate::comply::scoring::Grade;

    if config.thresholds.min_score > 0 && score.score < f64::from(config.thresholds.min_score) {
        return Err(Error::Validation(format!(
            "Score {:.0} below config threshold {} (grade {})",
            score.score, config.thresholds.min_score, score.grade
        )));
    }
    if config.thresholds.max_violations > 0 {
        let total_violations: usize = score.artifact_scores.iter().map(|a| a.violations).sum();
        if total_violations > config.thresholds.max_violations as usize {
            return Err(Error::Validation(format!(
                "Violations {} exceed config max {} (grade {})",
                total_violations, config.thresholds.max_violations, score.grade
            )));
        }
    }
    // Suppress unused import warning — Grade is used for formatting above.
    let _ = Grade::F;
    Ok(())
}

/// Convert a CLI scope argument to the internal `Scope` enum.
///
/// `ComplyScopeArg::All` falls back to `Scope::Project`; callers that need
/// to handle `All` specially should do so before calling this function.
#[must_use]
pub(crate) fn comply_scope_to_internal(scope: ComplyScopeArg) -> crate::comply::config::Scope {
    use crate::comply::config::Scope;
    match scope {
        ComplyScopeArg::Project => Scope::Project,
        ComplyScopeArg::User => Scope::User,
        ComplyScopeArg::System => Scope::System,
        ComplyScopeArg::All => Scope::Project, // fallback, caller should handle All
    }
}

// ─── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::comply::config::{ComplyConfig, Scope};
    use crate::comply::scoring::{ArtifactScore, Grade, ProjectScore};

    // Helper: build a minimal ProjectScore for threshold tests.
    fn make_score(score: f64, violations_per_artifact: &[usize]) -> ProjectScore {
        let artifact_scores: Vec<ArtifactScore> = violations_per_artifact
            .iter()
            .enumerate()
            .map(|(i, &v)| ArtifactScore {
                artifact_name: format!("artifact_{i}"),
                score,
                grade: Grade::from_score(score),
                rules_tested: 1,
                rules_passed: if v == 0 { 1 } else { 0 },
                violations: v,
                results: vec![],
            })
            .collect();
        let total = artifact_scores.len();
        let compliant = artifact_scores.iter().filter(|a| a.violations == 0).count();
        ProjectScore {
            total_artifacts: total,
            compliant_artifacts: compliant,
            score,
            grade: Grade::from_score(score),
            total_falsification_attempts: total,
            successful_falsifications: violations_per_artifact.iter().sum(),
            artifact_scores,
        }
    }

    // ── comply_scope_filter ───────────────────────────────────────────────────

    #[test]
    fn test_comply_logic_001_scope_filter_none_returns_none() {
        assert_eq!(comply_scope_filter(None), None);
    }

    #[test]
    fn test_comply_logic_002_scope_filter_project_returns_project() {
        assert_eq!(
            comply_scope_filter(Some(ComplyScopeArg::Project)),
            Some(Scope::Project)
        );
    }

    #[test]
    fn test_comply_logic_003_scope_filter_user_returns_user() {
        assert_eq!(
            comply_scope_filter(Some(ComplyScopeArg::User)),
            Some(Scope::User)
        );
    }

    #[test]
    fn test_comply_logic_004_scope_filter_system_returns_system() {
        assert_eq!(
            comply_scope_filter(Some(ComplyScopeArg::System)),
            Some(Scope::System)
        );
    }

    #[test]
    fn test_comply_logic_005_scope_filter_all_returns_none() {
        // All means "no filter" — runner will scan every scope.
        assert_eq!(comply_scope_filter(Some(ComplyScopeArg::All)), None);
    }

    // ── comply_scope_to_internal ──────────────────────────────────────────────

    #[test]
    fn test_comply_logic_006_scope_to_internal_project() {
        assert_eq!(comply_scope_to_internal(ComplyScopeArg::Project), Scope::Project);
    }

    #[test]
    fn test_comply_logic_007_scope_to_internal_user() {
        assert_eq!(comply_scope_to_internal(ComplyScopeArg::User), Scope::User);
    }

    #[test]
    fn test_comply_logic_008_scope_to_internal_system() {
        assert_eq!(comply_scope_to_internal(ComplyScopeArg::System), Scope::System);
    }

    #[test]
    fn test_comply_logic_009_scope_to_internal_all_fallback_is_project() {
        // Documented fallback: All → Project (caller must handle All before delegating).
        assert_eq!(comply_scope_to_internal(ComplyScopeArg::All), Scope::Project);
    }

    // ── apply_comply_scope ────────────────────────────────────────────────────

    #[test]
    fn test_comply_logic_010_apply_scope_project_clears_user_and_system() {
        let mut cfg = ComplyConfig::new_default("0.0.0");
        cfg.scopes.user = true;
        cfg.scopes.system = true;
        apply_comply_scope(&mut cfg, ComplyScopeArg::Project);
        assert!(!cfg.scopes.user, "user should be false for Project scope");
        assert!(!cfg.scopes.system, "system should be false for Project scope");
    }

    #[test]
    fn test_comply_logic_011_apply_scope_user_sets_user_clears_system() {
        let mut cfg = ComplyConfig::new_default("0.0.0");
        apply_comply_scope(&mut cfg, ComplyScopeArg::User);
        assert!(cfg.scopes.user);
        assert!(!cfg.scopes.system);
    }

    #[test]
    fn test_comply_logic_012_apply_scope_system_clears_user_sets_system() {
        let mut cfg = ComplyConfig::new_default("0.0.0");
        apply_comply_scope(&mut cfg, ComplyScopeArg::System);
        assert!(!cfg.scopes.user);
        assert!(cfg.scopes.system);
    }

    #[test]
    fn test_comply_logic_013_apply_scope_all_enables_user_and_system() {
        let mut cfg = ComplyConfig::new_default("0.0.0");
        apply_comply_scope(&mut cfg, ComplyScopeArg::All);
        assert!(cfg.scopes.user);
        assert!(cfg.scopes.system);
    }

    // ── apply_comply_strict ───────────────────────────────────────────────────

    #[test]
    fn test_comply_logic_014_apply_strict_enables_all_rules() {
        let mut cfg = ComplyConfig::new_default("0.0.0");
        // Disable everything first to confirm the function actually turns them on.
        cfg.rules.posix = false;
        cfg.rules.determinism = false;
        cfg.rules.idempotency = false;
        cfg.rules.security = false;
        cfg.rules.quoting = false;
        cfg.rules.shellcheck = false;
        cfg.rules.makefile_safety = false;
        cfg.rules.dockerfile_best = false;
        cfg.rules.config_hygiene = false;
        cfg.rules.pzsh_budget = "off".to_string();

        apply_comply_strict(&mut cfg);

        assert!(cfg.rules.posix);
        assert!(cfg.rules.determinism);
        assert!(cfg.rules.idempotency);
        assert!(cfg.rules.security);
        assert!(cfg.rules.quoting);
        assert!(cfg.rules.shellcheck);
        assert!(cfg.rules.makefile_safety);
        assert!(cfg.rules.dockerfile_best);
        assert!(cfg.rules.config_hygiene);
        assert_eq!(cfg.rules.pzsh_budget, "10ms");
    }

    // ── comply_enforce_thresholds ─────────────────────────────────────────────

    #[test]
    fn test_comply_logic_015_enforce_thresholds_passes_when_score_above_min() {
        let score = make_score(90.0, &[0]);
        let mut cfg = ComplyConfig::new_default("0.0.0");
        cfg.thresholds.min_score = 80;
        cfg.thresholds.max_violations = 0;
        assert!(comply_enforce_thresholds(&score, &cfg).is_ok());
    }

    #[test]
    fn test_comply_logic_016_enforce_thresholds_fails_when_score_below_min() {
        let score = make_score(50.0, &[0]);
        let mut cfg = ComplyConfig::new_default("0.0.0");
        cfg.thresholds.min_score = 80;
        cfg.thresholds.max_violations = 0;
        let result = comply_enforce_thresholds(&score, &cfg);
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("below config threshold"), "error: {msg}");
    }

    #[test]
    fn test_comply_logic_017_enforce_thresholds_passes_when_min_score_zero() {
        // min_score = 0 means "no threshold"
        let score = make_score(10.0, &[0]);
        let mut cfg = ComplyConfig::new_default("0.0.0");
        cfg.thresholds.min_score = 0;
        cfg.thresholds.max_violations = 0;
        assert!(comply_enforce_thresholds(&score, &cfg).is_ok());
    }

    #[test]
    fn test_comply_logic_018_enforce_thresholds_fails_when_violations_exceed_max() {
        // 3 total violations, max_violations = 2
        let score = make_score(70.0, &[1, 2]);
        let mut cfg = ComplyConfig::new_default("0.0.0");
        cfg.thresholds.min_score = 0;
        cfg.thresholds.max_violations = 2;
        let result = comply_enforce_thresholds(&score, &cfg);
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("exceed config max"), "error: {msg}");
    }

    #[test]
    fn test_comply_logic_019_enforce_thresholds_passes_when_violations_within_max() {
        let score = make_score(70.0, &[1]);
        let mut cfg = ComplyConfig::new_default("0.0.0");
        cfg.thresholds.min_score = 0;
        cfg.thresholds.max_violations = 5;
        assert!(comply_enforce_thresholds(&score, &cfg).is_ok());
    }

    #[test]
    fn test_comply_logic_020_enforce_thresholds_max_violations_zero_means_no_limit() {
        // max_violations = 0 disables that check (matches the `> 0` guard)
        let score = make_score(70.0, &[100]);
        let mut cfg = ComplyConfig::new_default("0.0.0");
        cfg.thresholds.min_score = 0;
        cfg.thresholds.max_violations = 0;
        assert!(comply_enforce_thresholds(&score, &cfg).is_ok());
    }

    // ── comply_load_or_default ────────────────────────────────────────────────

    #[test]
    fn test_comply_logic_021_load_or_default_returns_default_for_nonexistent_path() {
        // A path that definitely has no .bashrs/comply.toml
        let tmp = std::env::temp_dir().join("comply_logic_test_nonexistent_12345");
        let cfg = comply_load_or_default(&tmp);
        // Default has min_score = 80 (see ComplyConfig::new_default)
        assert_eq!(cfg.thresholds.min_score, 80);
        assert!(cfg.scopes.project);
        assert!(!cfg.scopes.user);
        assert!(!cfg.scopes.system);
    }
}
