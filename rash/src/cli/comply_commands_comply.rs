fn comply_track_list(path: &Path, scope: Option<ComplyScopeArg>) -> Result<()> {
    use crate::comply::{config::Scope, discovery};

    info!("Listing tracked artifacts");

    let scopes = match scope.and_then(|s| match s {
        ComplyScopeArg::Project => Some(Scope::Project),
        ComplyScopeArg::User => Some(Scope::User),
        ComplyScopeArg::System => Some(Scope::System),
        ComplyScopeArg::All => None,
    }) {
        Some(s) => vec![s],
        None => vec![Scope::Project, Scope::User, Scope::System],
    };

    let mut total = 0;
    for s in scopes {
        let artifacts = discovery::discover(path, s);
        if !artifacts.is_empty() {
            println!("{:?} ({}):", s, artifacts.len());
            for a in &artifacts {
                println!("  {} [{:?}]", a.display_name(), a.kind);
            }
            total += artifacts.len();
        }
    }
    println!("\nTotal tracked: {}", total);
    Ok(())
}

fn comply_scope_to_internal(scope: ComplyScopeArg) -> crate::comply::config::Scope {
    use crate::comply::config::Scope;
    match scope {
        ComplyScopeArg::Project => Scope::Project,
        ComplyScopeArg::User => Scope::User,
        ComplyScopeArg::System => Scope::System,
        ComplyScopeArg::All => Scope::Project, // fallback, caller should handle All
    }
}

fn comply_print_artifact_list(
    scope: crate::comply::config::Scope,
    artifacts: &[crate::comply::discovery::Artifact],
) {
    println!("{:?} scope ({} artifacts):", scope, artifacts.len());
    for a in artifacts {
        println!("  {} [{:?}]", a.display_name(), a.kind);
    }
}

// ============================================================================
// Phase 2: comply report
// ============================================================================

fn comply_report_command(
    path: &Path,
    format: ComplyFormat,
    output: Option<&Path>,
    scope: Option<ComplyScopeArg>,
) -> Result<()> {
    use crate::comply::runner;

    info!("Generating compliance report for {}", path.display());

    let config = comply_load_or_default(path);
    let score = runner::run_check(path, comply_scope_filter(scope), &config);

    let report = match format {
        ComplyFormat::Json => comply_report_json(&score),
        ComplyFormat::Markdown | ComplyFormat::Text => comply_report_markdown(&score),
    };

    if let Some(out_path) = output {
        std::fs::write(out_path, &report)
            .map_err(|e| Error::Internal(format!("Failed to write report: {e}")))?;
        println!("Report written to {}", out_path.display());
    } else {
        println!("{report}");
    }

    Ok(())
}

fn comply_report_markdown(score: &crate::comply::scoring::ProjectScore) -> String {
    let mut md = String::new();
    md.push_str("# Compliance Report\n\n");
    md.push_str(&format!(
        "**Grade**: {} | **Score**: {:.0}/100 | **Artifacts**: {}/{} compliant\n\n",
        score.grade, score.score, score.compliant_artifacts, score.total_artifacts
    ));
    md.push_str(&format!(
        "**Falsification**: {} attempts, {} succeeded\n\n",
        score.total_falsification_attempts, score.successful_falsifications
    ));

    // Artifact table
    md.push_str("## Artifacts\n\n");
    md.push_str("| Artifact | Score | Grade | Violations |\n");
    md.push_str("|----------|-------|-------|------------|\n");
    for a in &score.artifact_scores {
        let status = if a.violations == 0 {
            "COMPLIANT"
        } else {
            "NON-COMPLIANT"
        };
        md.push_str(&format!(
            "| {} | {:.0} | {} | {} ({}) |\n",
            a.artifact_name, a.score, a.grade, a.violations, status
        ));
    }

    // Findings
    let non_compliant: Vec<_> = score
        .artifact_scores
        .iter()
        .filter(|a| a.violations > 0)
        .collect();
    if !non_compliant.is_empty() {
        md.push_str("\n## Findings\n\n");
        for a in non_compliant {
            md.push_str(&format!("### {}\n\n", a.artifact_name));
            for r in &a.results {
                if !r.passed {
                    for v in &r.violations {
                        let line = v.line.unwrap_or(0);
                        md.push_str(&format!(
                            "- **{:?}** (line {}): {}\n",
                            v.rule, line, v.message
                        ));
                    }
                }
            }
            md.push('\n');
        }
    }

    md
}

fn comply_report_json(score: &crate::comply::scoring::ProjectScore) -> String {
    let artifacts: Vec<serde_json::Value> = score
        .artifact_scores
        .iter()
        .map(|a| {
            let violations: Vec<serde_json::Value> = a
                .results
                .iter()
                .filter(|r| !r.passed)
                .flat_map(|r| {
                    r.violations.iter().map(|v| {
                        serde_json::json!({
                            "code": format!("{:?}", v.rule),
                            "line": v.line.unwrap_or(0),
                            "message": v.message,
                        })
                    })
                })
                .collect();
            serde_json::json!({
                "name": a.artifact_name,
                "score": a.score,
                "grade": format!("{}", a.grade),
                "violations": a.violations,
                "findings": violations,
            })
        })
        .collect();

    let report = serde_json::json!({
        "grade": format!("{}", score.grade),
        "score": score.score,
        "total_artifacts": score.total_artifacts,
        "compliant_artifacts": score.compliant_artifacts,
        "falsification_attempts": score.total_falsification_attempts,
        "successful_falsifications": score.successful_falsifications,
        "artifacts": artifacts,
    });

    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
}

// ============================================================================
// Phase 2: comply enforce
// ============================================================================

fn comply_enforce_command(tier: u8, uninstall: bool) -> Result<()> {
    comply_enforce_command_with(Path::new(".git/hooks"), tier, uninstall)
}

fn comply_enforce_command_with(hooks_dir: &Path, tier: u8, uninstall: bool) -> Result<()> {
    if !hooks_dir.exists() {
        return Err(Error::Validation(
            "Not a git repository (no .git/hooks directory)".into(),
        ));
    }

    let hook_path = hooks_dir.join("pre-commit");

    if uninstall {
        if hook_path.exists() {
            let content = std::fs::read_to_string(&hook_path).unwrap_or_default();
            if content.contains("bashrs comply") {
                std::fs::remove_file(&hook_path)
                    .map_err(|e| Error::Internal(format!("Failed to remove hook: {e}")))?;
                println!("Removed comply pre-commit hook");
            } else {
                println!("Pre-commit hook exists but is not a comply hook — skipping");
            }
        } else {
            println!("No pre-commit hook found");
        }
        return Ok(());
    }

    if hook_path.exists() {
        let content = std::fs::read_to_string(&hook_path).unwrap_or_default();
        if !content.contains("bashrs comply") {
            return Err(Error::Validation(
                "Pre-commit hook already exists (not a comply hook). Remove it first or use --uninstall.".into(),
            ));
        }
    }

    let tier_args = match tier {
        1 => "--failures-only",
        2 => "",
        3 => "--strict",
        _ => "--failures-only",
    };

    let hook_content = format!(
        "#!/bin/sh\n\
        # bashrs comply enforcement hook (tier {tier})\n\
        # Installed by: bashrs comply enforce --tier {tier}\n\
        # Remove with: bashrs comply enforce --uninstall\n\n\
        bashrs comply check {tier_args} --strict 2>/dev/null\n\
        exit $?\n"
    );

    std::fs::write(&hook_path, hook_content)
        .map_err(|e| Error::Internal(format!("Failed to write hook: {e}")))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&hook_path, perms)
            .map_err(|e| Error::Internal(format!("Failed to set hook permissions: {e}")))?;
    }

    println!("Installed comply pre-commit hook (tier {tier})");
    println!("  Hook: {}", hook_path.display());
    println!("  Remove: bashrs comply enforce --uninstall");
    Ok(())
}

// ============================================================================
// Phase 2: comply diff
// ============================================================================

fn comply_diff_command(path: &Path, _since_last: bool) -> Result<()> {
    use crate::comply::runner;

    info!("Computing compliance diff for {}", path.display());

    let config = comply_load_or_default(path);
    let current = runner::run_check(path, None, &config);

    // Load previous score from .bashrs/comply-last.json
    let last_path = path.join(".bashrs").join("comply-last.json");
    let previous = if last_path.exists() {
        let content = std::fs::read_to_string(&last_path).unwrap_or_default();
        serde_json::from_str::<ComplyDiffSnapshot>(&content).ok()
    } else {
        None
    };

    // Save current snapshot for next diff
    let snapshot = ComplyDiffSnapshot {
        score: current.score,
        grade: format!("{}", current.grade),
        artifacts: current
            .artifact_scores
            .iter()
            .map(|a| ComplyDiffArtifact {
                name: a.artifact_name.clone(),
                score: a.score,
                violations: a.violations,
            })
            .collect(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&snapshot) {
        let dir = path.join(".bashrs");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(&last_path, json);
    }

    match previous {
        None => {
            println!("No previous compliance snapshot found.");
            println!(
                "Current: {:.0}/100 (grade {}), {}/{} compliant",
                current.score, current.grade, current.compliant_artifacts, current.total_artifacts
            );
            println!("\nSnapshot saved. Run again to see diff.");
        }
        Some(prev) => {
            let score_delta = current.score - prev.score;
            let direction = if score_delta > 0.0 { "+" } else { "" };
            println!("Compliance Diff");
            println!(
                "  Score: {:.0} -> {:.0} ({}{:.0})",
                prev.score, current.score, direction, score_delta
            );
            println!("  Grade: {} -> {}", prev.grade, current.grade);

            // Find new and fixed violations
            let prev_artifacts: std::collections::HashMap<_, _> = prev
                .artifacts
                .iter()
                .map(|a| (a.name.as_str(), a))
                .collect();

            for a in &current.artifact_scores {
                let prev_v = prev_artifacts
                    .get(a.artifact_name.as_str())
                    .map(|p| p.violations)
                    .unwrap_or(0);
                if a.violations != prev_v {
                    let delta = a.violations as i64 - prev_v as i64;
                    let sym = if delta > 0 { "+" } else { "" };
                    println!(
                        "  {} violations: {} -> {} ({}{delta})",
                        a.artifact_name, prev_v, a.violations, sym
                    );
                }
            }
        }
    }

    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ComplyDiffSnapshot {
    score: f64,
    grade: String,
    artifacts: Vec<ComplyDiffArtifact>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ComplyDiffArtifact {
    name: String,
    score: f64,
    violations: usize,
}

// PMAT-257: every function below either takes an explicit path/tempdir-scoped
// hooks_dir or operates on plain in-memory structs (ProjectScore, ArtifactScore).
// None of them touch CorpusRegistry/CorpusRunner, so all fixtures below are
// tempfile::TempDir-scoped and never read/write the real repo or $HOME.
#[cfg(test)]
mod pmat257_cov_tests {
    use super::*;
    use crate::comply::discovery::{Artifact, ArtifactKind};
    use crate::comply::rules::{RuleId, RuleResult, Violation};
    use crate::comply::scoring::{ArtifactScore, Grade, ProjectScore};

    fn write_file(dir: &std::path::Path, name: &str, content: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, content).expect("write fixture");
        path
    }

    // ---- comply_track_list ----

    #[test]
    fn test_PMAT257_cov_track_list_project_scope_with_artifacts() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        write_file(dir.path(), "deploy.sh", "#!/bin/sh\necho hi\n");
        write_file(dir.path(), "Makefile", "all:\n\techo hi\n");
        comply_track_list(dir.path(), Some(ComplyScopeArg::Project))
            .expect("tracking a project with artifacts must succeed");
    }

    #[test]
    fn test_PMAT257_cov_track_list_empty_project_scope() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        comply_track_list(dir.path(), Some(ComplyScopeArg::Project))
            .expect("tracking an empty project must still succeed");
    }

    #[test]
    fn test_PMAT257_cov_track_list_none_scope_covers_all_three() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        write_file(dir.path(), "run.sh", "#!/bin/sh\necho hi\n");
        // scope=None takes the "all scopes" branch (project + user + system);
        // user/system discovery is a read-only existence check of fixed paths.
        comply_track_list(dir.path(), None).expect("tracking with no scope filter must succeed");
    }

    // ---- comply_scope_to_internal ----

    #[test]
    fn test_PMAT257_cov_scope_to_internal_all_variants() {
        use crate::comply::config::Scope;
        assert_eq!(
            comply_scope_to_internal(ComplyScopeArg::Project),
            Scope::Project
        );
        assert_eq!(comply_scope_to_internal(ComplyScopeArg::User), Scope::User);
        assert_eq!(
            comply_scope_to_internal(ComplyScopeArg::System),
            Scope::System
        );
        // All has no direct internal scope; falls back to Project.
        assert_eq!(
            comply_scope_to_internal(ComplyScopeArg::All),
            Scope::Project
        );
    }

    // ---- comply_print_artifact_list ----

    #[test]
    fn test_PMAT257_cov_print_artifact_list_empty() {
        use crate::comply::config::Scope;
        comply_print_artifact_list(Scope::Project, &[]);
    }

    #[test]
    fn test_PMAT257_cov_print_artifact_list_with_entries() {
        use crate::comply::config::Scope;
        let artifacts = vec![Artifact::new(
            std::path::PathBuf::from("build.sh"),
            Scope::Project,
            ArtifactKind::ShellScript,
        )];
        comply_print_artifact_list(Scope::Project, &artifacts);
    }

    // ---- comply_report_command ----

    #[test]
    fn test_PMAT257_cov_report_command_text_to_stdout() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        write_file(dir.path(), "ok.sh", "#!/bin/sh\necho hi\n");
        comply_report_command(dir.path(), ComplyFormat::Text, None, None)
            .expect("text report to stdout must succeed");
    }

    #[test]
    fn test_PMAT257_cov_report_command_json_to_file() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        write_file(dir.path(), "ok.sh", "#!/bin/sh\necho hi\n");
        let out = dir.path().join("report.json");
        comply_report_command(
            dir.path(),
            ComplyFormat::Json,
            Some(out.as_path()),
            Some(ComplyScopeArg::Project),
        )
        .expect("json report to a file must succeed");
        let content = std::fs::read_to_string(&out).expect("report file must exist");
        assert!(
            content.contains("\"grade\""),
            "json report must include a grade field"
        );
    }

    #[test]
    fn test_PMAT257_cov_report_command_markdown_to_file() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        write_file(dir.path(), "ok.sh", "#!/bin/sh\necho hi\n");
        let out = dir.path().join("report.md");
        comply_report_command(
            dir.path(),
            ComplyFormat::Markdown,
            Some(out.as_path()),
            None,
        )
        .expect("markdown report to a file must succeed");
        let content = std::fs::read_to_string(&out).expect("report file must exist");
        assert!(content.starts_with("# Compliance Report"));
    }

    // ---- comply_report_markdown / comply_report_json ----

    fn sample_project_score(violations: usize) -> ProjectScore {
        // One decision, made once: compliant or not.
        let compliant = violations == 0;
        let (score, grade, passed) = if compliant {
            (100.0, Grade::APlus, 1)
        } else {
            (40.0, Grade::F, 0)
        };
        let results = if compliant {
            vec![]
        } else {
            vec![RuleResult {
                rule: RuleId::Determinism,
                passed: false,
                violations: vec![Violation {
                    rule: RuleId::Determinism,
                    line: Some(3),
                    message: "uses $RANDOM".to_string(),
                }],
            }]
        };
        let artifact = ArtifactScore {
            artifact_name: "deploy.sh".to_string(),
            score,
            grade,
            rules_tested: 1,
            rules_passed: passed,
            violations,
            results,
        };
        ProjectScore {
            total_artifacts: 1,
            compliant_artifacts: passed,
            score,
            grade,
            total_falsification_attempts: 1,
            successful_falsifications: violations,
            artifact_scores: vec![artifact],
        }
    }

    #[test]
    fn test_PMAT257_cov_report_markdown_compliant_has_no_findings() {
        let score = sample_project_score(0);
        let md = comply_report_markdown(&score);
        assert!(md.contains("COMPLIANT"));
        assert!(!md.contains("## Findings"));
    }

    #[test]
    fn test_PMAT257_cov_report_markdown_non_compliant_lists_findings() {
        let score = sample_project_score(1);
        let md = comply_report_markdown(&score);
        assert!(md.contains("NON-COMPLIANT"));
        assert!(md.contains("## Findings"));
        assert!(md.contains("uses $RANDOM"));
    }

    #[test]
    fn test_PMAT257_cov_report_json_includes_findings() {
        let score = sample_project_score(1);
        let json = comply_report_json(&score);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(parsed["grade"], "F");
        assert_eq!(
            parsed["artifacts"][0]["findings"][0]["message"],
            "uses $RANDOM"
        );
    }

    // ---- comply_enforce_command_with ----

    #[test]
    fn test_PMAT257_cov_enforce_missing_hooks_dir_is_an_error() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let hooks_dir = dir.path().join("does-not-exist");
        let err = comply_enforce_command_with(&hooks_dir, 1, false)
            .expect_err("a missing hooks dir must be an error");
        assert!(format!("{err}").contains("Not a git repository"));
    }

    #[test]
    fn test_PMAT257_cov_enforce_installs_hook_each_tier() {
        for tier in [1u8, 2, 3, 9] {
            let dir = tempfile::TempDir::new().expect("tempdir");
            let hooks_dir = dir.path().join(".git/hooks");
            std::fs::create_dir_all(&hooks_dir).expect("mkdir hooks");
            comply_enforce_command_with(&hooks_dir, tier, false)
                .unwrap_or_else(|e| panic!("installing tier {tier} hook must succeed: {e}"));
            let hook_path = hooks_dir.join("pre-commit");
            let content = std::fs::read_to_string(&hook_path).expect("hook must exist");
            assert!(content.contains("bashrs comply check"));
        }
    }

    #[test]
    fn test_PMAT257_cov_enforce_refuses_foreign_hook() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let hooks_dir = dir.path().join(".git/hooks");
        std::fs::create_dir_all(&hooks_dir).expect("mkdir hooks");
        write_file(&hooks_dir, "pre-commit", "#!/bin/sh\necho not-comply\n");
        let err = comply_enforce_command_with(&hooks_dir, 1, false)
            .expect_err("a foreign pre-commit hook must not be overwritten");
        assert!(format!("{err}").contains("already exists"));
    }

    #[test]
    fn test_PMAT257_cov_enforce_overwrites_existing_comply_hook() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let hooks_dir = dir.path().join(".git/hooks");
        std::fs::create_dir_all(&hooks_dir).expect("mkdir hooks");
        write_file(&hooks_dir, "pre-commit", "#!/bin/sh\nbashrs comply check\n");
        comply_enforce_command_with(&hooks_dir, 2, false)
            .expect("reinstalling over an existing comply hook must succeed");
    }

    #[test]
    fn test_PMAT257_cov_enforce_uninstall_removes_comply_hook() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let hooks_dir = dir.path().join(".git/hooks");
        std::fs::create_dir_all(&hooks_dir).expect("mkdir hooks");
        let hook_path = write_file(&hooks_dir, "pre-commit", "#!/bin/sh\nbashrs comply check\n");
        comply_enforce_command_with(&hooks_dir, 1, true)
            .expect("uninstalling a comply hook must succeed");
        assert!(!hook_path.exists(), "comply hook must be removed");
    }

    #[test]
    fn test_PMAT257_cov_enforce_uninstall_leaves_foreign_hook() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let hooks_dir = dir.path().join(".git/hooks");
        std::fs::create_dir_all(&hooks_dir).expect("mkdir hooks");
        let hook_path = write_file(&hooks_dir, "pre-commit", "#!/bin/sh\necho not-comply\n");
        comply_enforce_command_with(&hooks_dir, 1, true)
            .expect("uninstall on a foreign hook must not error");
        assert!(hook_path.exists(), "foreign hook must be left alone");
    }

    #[test]
    fn test_PMAT257_cov_enforce_uninstall_no_hook_present() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let hooks_dir = dir.path().join(".git/hooks");
        std::fs::create_dir_all(&hooks_dir).expect("mkdir hooks");
        comply_enforce_command_with(&hooks_dir, 1, true)
            .expect("uninstall with no existing hook must succeed");
    }

    // ---- comply_diff_command ----

    #[test]
    fn test_PMAT257_cov_diff_command_first_run_has_no_previous_snapshot() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        write_file(dir.path(), "ok.sh", "#!/bin/sh\necho hi\n");
        comply_diff_command(dir.path(), false)
            .expect("first diff run must succeed and save a snapshot");
        let snapshot_path = dir.path().join(".bashrs").join("comply-last.json");
        assert!(
            snapshot_path.exists(),
            "diff must persist a snapshot for next run"
        );
    }

    #[test]
    fn test_PMAT257_cov_diff_command_second_run_reports_delta() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        write_file(dir.path(), "ok.sh", "#!/bin/sh\necho hi\n");
        let bashrs_dir = dir.path().join(".bashrs");
        std::fs::create_dir_all(&bashrs_dir).expect("mkdir .bashrs");
        let snapshot = ComplyDiffSnapshot {
            score: 10.0,
            grade: "F".to_string(),
            artifacts: vec![ComplyDiffArtifact {
                name: "ok.sh".to_string(),
                score: 10.0,
                violations: 5,
            }],
        };
        let json = serde_json::to_string_pretty(&snapshot).expect("serialize snapshot");
        std::fs::write(bashrs_dir.join("comply-last.json"), json).expect("write snapshot");

        comply_diff_command(dir.path(), false)
            .expect("second diff run against a saved snapshot must succeed");
    }
}
