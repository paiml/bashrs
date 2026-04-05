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
    let hooks_dir = Path::new(".git/hooks");
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
