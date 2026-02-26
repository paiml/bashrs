//! Coverage tests for `format_human`, `run_check`, and related functions
//! in `rash/src/comply/runner.rs`.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

#[cfg(test)]
mod tests {
    use crate::comply::config::{ComplyConfig, Scope};
    use crate::comply::runner;
    use crate::comply::scoring::{ArtifactScore, Grade, ProjectScore};

    // ── Helpers ──

    fn default_config() -> ComplyConfig {
        ComplyConfig::new_default("7.1.0")
    }

    fn make_project_score(total: usize, compliant: usize, score: f64) -> ProjectScore {
        let grade = Grade::from_score(score);
        ProjectScore {
            total_artifacts: total,
            compliant_artifacts: compliant,
            score,
            grade,
            total_falsification_attempts: total * 5,
            successful_falsifications: total - compliant,
            artifact_scores: vec![],
        }
    }

    fn make_artifact_score(name: &str, score: f64, violations: usize) -> ArtifactScore {
        ArtifactScore {
            artifact_name: name.to_string(),
            score,
            grade: Grade::from_score(score),
            rules_tested: 5,
            rules_passed: 5 - violations,
            violations,
            results: vec![],
        }
    }

    // ── format_human: basic structure ──

    #[test]
    fn test_format_human_contains_header() {
        let score = make_project_score(0, 0, 100.0);
        let output = runner::format_human(&score);
        assert!(
            output.contains("COMPLIANCE CHECK"),
            "Header missing: {output}"
        );
        assert!(output.contains("Jidoka"), "Jidoka layer label missing");
    }

    #[test]
    fn test_format_human_contains_artifact_header() {
        let score = make_project_score(2, 2, 100.0);
        let output = runner::format_human(&score);
        assert!(
            output.contains("Artifact"),
            "Artifact column header missing"
        );
        assert!(output.contains("Score"), "Score column header missing");
    }

    #[test]
    fn test_format_human_contains_overall_line() {
        let score = make_project_score(3, 2, 85.0);
        let output = runner::format_human(&score);
        assert!(
            output.contains("Overall"),
            "Overall summary missing: {output}"
        );
    }

    #[test]
    fn test_format_human_contains_grade() {
        let score = make_project_score(2, 2, 97.0);
        let output = runner::format_human(&score);
        assert!(output.contains("Grade"), "Grade field missing: {output}");
        assert!(output.contains("A+"), "Grade value A+ missing: {output}");
    }

    #[test]
    fn test_format_human_zero_artifacts() {
        let score = make_project_score(0, 0, 100.0);
        let output = runner::format_human(&score);
        // Should not panic, should still produce output with 0 artifacts
        assert!(
            output.contains("0"),
            "Zero artifact count not shown: {output}"
        );
    }

    #[test]
    fn test_format_human_compliant_artifact_shows_plus_sign() {
        let mut score = make_project_score(1, 1, 100.0);
        score.artifact_scores = vec![make_artifact_score("Makefile", 100.0, 0)];
        let output = runner::format_human(&score);
        // Compliant artifacts get "+" icon
        assert!(
            output.contains("+ COMPLIANT") || output.contains("+"),
            "Compliant icon missing"
        );
    }

    #[test]
    fn test_format_human_noncompliant_artifact_shows_exclamation() {
        let mut score = make_project_score(1, 0, 50.0);
        score.artifact_scores = vec![make_artifact_score("bad.sh", 50.0, 2)];
        let output = runner::format_human(&score);
        // Non-compliant artifacts get "!" icon
        assert!(
            output.contains("! NON-COMPLIANT") || output.contains("!"),
            "Non-compliant icon missing: {output}"
        );
    }

    #[test]
    fn test_format_human_falsification_attempts_shown() {
        let score = make_project_score(3, 3, 100.0);
        let output = runner::format_human(&score);
        assert!(
            output.contains("Falsification"),
            "Falsification info missing: {output}"
        );
    }

    #[test]
    fn test_format_human_contains_separator_lines() {
        let score = make_project_score(0, 0, 100.0);
        let output = runner::format_human(&score);
        // Should have separator characters
        assert!(output.contains("═"), "Top separator missing: {output}");
        assert!(output.contains("─"), "Inner separator missing: {output}");
    }

    #[test]
    fn test_format_human_long_artifact_name_truncated() {
        let long_name = "a".repeat(50);
        let mut score = make_project_score(1, 1, 100.0);
        score.artifact_scores = vec![make_artifact_score(&long_name, 100.0, 0)];
        let output = runner::format_human(&score);
        // Name should be truncated (max 35 chars + "...")
        assert!(
            output.contains("..."),
            "Long artifact name should be truncated: {output}"
        );
    }

    #[test]
    fn test_format_human_short_name_not_truncated() {
        let short_name = "Makefile";
        let mut score = make_project_score(1, 1, 100.0);
        score.artifact_scores = vec![make_artifact_score(short_name, 100.0, 0)];
        let output = runner::format_human(&score);
        assert!(
            output.contains("Makefile"),
            "Short name should appear as-is"
        );
    }

    #[test]
    fn test_format_human_grade_f_shown() {
        let score = make_project_score(1, 0, 40.0);
        let output = runner::format_human(&score);
        assert!(
            output.contains("F"),
            "Grade F should be shown for low score"
        );
    }

    #[test]
    fn test_format_human_grade_b_shown() {
        let score = make_project_score(1, 0, 75.0);
        let output = runner::format_human(&score);
        assert!(
            output.contains("B"),
            "Grade B should be shown for 75.0 score"
        );
    }

    // ── format_human_failures_only ──

    #[test]
    fn test_format_human_failures_only_no_violations() {
        let mut score = make_project_score(2, 2, 100.0);
        score.artifact_scores = vec![
            make_artifact_score("a.sh", 100.0, 0),
            make_artifact_score("b.sh", 100.0, 0),
        ];
        let output = runner::format_human_failures_only(&score);
        assert!(
            output.contains("No violations"),
            "Should say no violations: {output}"
        );
    }

    #[test]
    fn test_format_human_failures_only_with_violation() {
        let mut score = make_project_score(2, 1, 80.0);
        score.artifact_scores = vec![
            make_artifact_score("good.sh", 100.0, 0),
            make_artifact_score("bad.sh", 60.0, 1),
        ];
        let output = runner::format_human_failures_only(&score);
        // Should show bad.sh but not good.sh
        assert!(
            output.contains("bad.sh"),
            "Should show non-compliant artifact"
        );
        assert!(
            !output.contains("good.sh"),
            "Should NOT show compliant artifact in failures-only"
        );
    }

    #[test]
    fn test_format_human_failures_only_contains_header() {
        let score = make_project_score(0, 0, 100.0);
        let output = runner::format_human_failures_only(&score);
        assert!(
            output.contains("Failures Only"),
            "Header should mention failures: {output}"
        );
    }

    // ── format_json ──

    #[test]
    fn test_format_json_is_valid_structure() {
        let score = make_project_score(0, 0, 100.0);
        let output = runner::format_json(&score);
        assert!(
            output.contains("bashrs-comply-check-v1"),
            "Schema field missing"
        );
        assert!(
            output.contains("total_artifacts"),
            "total_artifacts missing"
        );
        assert!(output.contains("score"), "score field missing");
        assert!(output.contains("grade"), "grade field missing");
    }

    #[test]
    fn test_format_json_with_artifacts() {
        let mut score = make_project_score(1, 1, 100.0);
        score.artifact_scores = vec![make_artifact_score("Makefile", 100.0, 0)];
        let output = runner::format_json(&score);
        assert!(
            output.contains("Makefile"),
            "Artifact name should be in JSON"
        );
        assert!(output.contains("100"), "Score should be in JSON");
    }

    // ── run_check ──

    #[test]
    fn test_run_check_nonexistent_path() {
        let config = default_config();
        let score = runner::run_check(std::path::Path::new("/nonexistent/path"), None, &config);
        // No artifacts found → vacuously perfect
        assert_eq!(score.total_artifacts, 0, "Nonexistent path: 0 artifacts");
    }

    #[test]
    fn test_run_check_project_scope_only() {
        let config = default_config();
        let score = runner::run_check(
            std::path::Path::new("/nonexistent/path"),
            Some(Scope::Project),
            &config,
        );
        assert_eq!(score.total_artifacts, 0);
    }

    #[test]
    fn test_run_check_user_scope_disabled_gives_empty() {
        let mut config = default_config();
        config.scopes.user = false;
        let score = runner::run_check(
            std::path::Path::new("/nonexistent"),
            Some(Scope::User),
            &config,
        );
        assert_eq!(
            score.total_artifacts, 0,
            "User scope disabled should give 0"
        );
    }

    #[test]
    fn test_run_check_system_scope_disabled_gives_empty() {
        let mut config = default_config();
        config.scopes.system = false;
        let score = runner::run_check(
            std::path::Path::new("/nonexistent"),
            Some(Scope::System),
            &config,
        );
        assert_eq!(
            score.total_artifacts, 0,
            "System scope disabled should give 0"
        );
    }

    #[test]
    fn test_run_check_none_scope_uses_all_enabled_scopes() {
        let config = default_config();
        // None scope = combine all enabled; nonexistent path → 0 artifacts but no panic
        let score = runner::run_check(std::path::Path::new("/nonexistent"), None, &config);
        assert_eq!(score.total_artifacts, 0);
    }

    #[test]
    fn test_run_check_returns_project_score_type() {
        let config = default_config();
        let score = runner::run_check(std::path::Path::new("/nonexistent"), None, &config);
        // Grade should be A+ for empty project (vacuously compliant)
        assert_eq!(score.grade, Grade::APlus);
        assert_eq!(score.score, 100.0);
    }

    // ── parse_suppressions ──

    #[test]
    fn test_parse_suppressions_empty_content() {
        let sup = runner::parse_suppressions("");
        assert!(sup.file_level.is_empty());
        assert!(sup.line_level.is_empty());
    }

    #[test]
    fn test_parse_suppressions_file_level() {
        let content = "# comply:disable=COMPLY-002\n#!/bin/sh\necho hello\n";
        let sup = runner::parse_suppressions(content);
        assert!(sup.file_level.contains(&"COMPLY-002".to_string()));
    }

    #[test]
    fn test_parse_suppressions_line_level() {
        let content = "#!/bin/sh\nSESSION=$RANDOM  # comply:disable=COMPLY-002\n";
        let sup = runner::parse_suppressions(content);
        // Line 2 should have the suppression
        assert!(sup.line_level.contains_key(&2));
    }

    #[test]
    fn test_parse_suppressions_invalid_marker_ignored() {
        let content = "# no-comply:disable=COMPLY-002\n";
        let sup = runner::parse_suppressions(content);
        assert!(sup.file_level.is_empty());
    }

    // ── extract_disable_rules ──

    #[test]
    fn test_extract_disable_rules_single_rule() {
        let result = runner::extract_disable_rules("# comply:disable=COMPLY-001");
        assert_eq!(result, Some(vec!["COMPLY-001".to_string()]));
    }

    #[test]
    fn test_extract_disable_rules_multiple_rules() {
        let result = runner::extract_disable_rules("# comply:disable=COMPLY-001,COMPLY-003");
        let rules = result.expect("Should parse multiple rules");
        assert!(rules.contains(&"COMPLY-001".to_string()));
        assert!(rules.contains(&"COMPLY-003".to_string()));
    }

    #[test]
    fn test_extract_disable_rules_no_marker_returns_none() {
        let result = runner::extract_disable_rules("# just a comment");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_disable_rules_non_comply_prefix_filtered() {
        let result = runner::extract_disable_rules("# comply:disable=NOTCOMPLY-001");
        assert!(
            result.is_none(),
            "Non COMPLY- prefixed rules should be filtered"
        );
    }

    #[test]
    fn test_extract_disable_rules_not_preceded_by_hash() {
        let result = runner::extract_disable_rules("code comply:disable=COMPLY-001");
        assert!(result.is_none(), "Must be preceded by # to be valid");
    }
}
