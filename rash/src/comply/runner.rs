//! Compliance runner — orchestrates checks across artifacts and rules

use super::config::{ComplyConfig, Scope};
use super::discovery::{self, Artifact};
use super::rules::{self, RuleId, RuleResult};
use super::scoring::{self, ProjectScore};
use std::path::Path;

/// Run compliance check for a project
pub fn run_check(
    project_path: &Path,
    scope: Option<Scope>,
    config: &ComplyConfig,
) -> ProjectScore {
    let artifacts = collect_artifacts(project_path, scope, config);
    let mut artifact_scores = Vec::new();

    for artifact in &artifacts {
        let content = match read_artifact_content(project_path, artifact) {
            Some(c) => c,
            None => continue,
        };

        let rules = RuleId::applicable_rules(artifact.kind);
        let enabled_rules: Vec<RuleId> = rules
            .into_iter()
            .filter(|r| is_rule_enabled(r, config))
            .collect();

        let results: Vec<RuleResult> = enabled_rules
            .iter()
            .map(|rule| rules::check_rule(*rule, &content, artifact))
            .collect();

        let name = artifact.display_name();
        let score = scoring::compute_artifact_score(&name, &results);
        artifact_scores.push(score);
    }

    scoring::compute_project_score(artifact_scores)
}

/// Collect artifacts based on scope and config
fn collect_artifacts(
    project_path: &Path,
    scope: Option<Scope>,
    config: &ComplyConfig,
) -> Vec<Artifact> {
    match scope {
        Some(Scope::Project) => discovery::discover(project_path, Scope::Project),
        Some(Scope::User) => {
            if config.scopes.user {
                discovery::discover(project_path, Scope::User)
            } else {
                vec![]
            }
        }
        Some(Scope::System) => {
            if config.scopes.system {
                discovery::discover(project_path, Scope::System)
            } else {
                vec![]
            }
        }
        None => {
            let mut artifacts = Vec::new();
            if config.scopes.project {
                artifacts.extend(discovery::discover(project_path, Scope::Project));
            }
            if config.scopes.user {
                artifacts.extend(discovery::discover(project_path, Scope::User));
            }
            if config.scopes.system {
                artifacts.extend(discovery::discover(project_path, Scope::System));
            }
            artifacts
        }
    }
}

fn read_artifact_content(project_path: &Path, artifact: &Artifact) -> Option<String> {
    let full_path = match artifact.scope {
        Scope::Project => project_path.join(&artifact.path),
        _ => artifact.path.clone(),
    };
    std::fs::read_to_string(full_path).ok()
}

fn is_rule_enabled(rule: &RuleId, config: &ComplyConfig) -> bool {
    match rule {
        RuleId::Posix => config.rules.posix,
        RuleId::Determinism => config.rules.determinism,
        RuleId::Idempotency => config.rules.idempotency,
        RuleId::Security => config.rules.security,
        RuleId::Quoting => config.rules.quoting,
        RuleId::ShellCheck => config.rules.shellcheck,
        RuleId::MakefileSafety => config.rules.makefile_safety,
        RuleId::DockerfileBest => config.rules.dockerfile_best,
        RuleId::ConfigHygiene => config.rules.config_hygiene,
        RuleId::PzshBudget => config.rules.pzsh_budget != "disabled",
    }
}

/// Format check results for human output
pub fn format_human(score: &ProjectScore) -> String {
    let mut out = String::new();

    out.push_str(
        "═══════════════════════════════════════════════════════════\n",
    );
    out.push_str(
        "  COMPLIANCE CHECK — Layer 1 (Jidoka)\n",
    );
    out.push_str(
        "═══════════════════════════════════════════════════════════\n\n",
    );

    // pzsh detection
    if let Some(pzsh) = discovery::detect_pzsh() {
        out.push_str(&format!(
            "Scope: project ({} artifacts) | pzsh: {} (integrated)\n\n",
            score.total_artifacts, pzsh.version
        ));
    } else {
        out.push_str(&format!(
            "Scope: project ({} artifacts) | pzsh: not found\n\n",
            score.total_artifacts
        ));
    }

    out.push_str(&format!(
        " {:<35} {:>5}  {}\n",
        "Artifact", "Score", "Status"
    ));
    out.push_str(&format!("{}\n", "─".repeat(57)));

    for artifact_score in &score.artifact_scores {
        let status = if artifact_score.violations == 0 {
            "COMPLIANT"
        } else {
            "NON-COMPLIANT"
        };
        let icon = if artifact_score.violations == 0 {
            "+"
        } else {
            "!"
        };

        out.push_str(&format!(
            " {:<35} {:>3.0}    {} {}\n",
            truncate(&artifact_score.artifact_name, 35),
            artifact_score.score,
            icon,
            status
        ));

        // Show violations
        for result in &artifact_score.results {
            for v in &result.violations {
                out.push_str(&format!("   {}\n", v));
            }
        }
    }

    out.push_str(&format!("\n{}\n", "─".repeat(57)));
    out.push_str(&format!(
        " Overall: {:.0}/100 ({}/{} compliant)\n",
        score.score, score.compliant_artifacts, score.total_artifacts
    ));
    out.push_str(&format!(" Grade: {}\n", score.grade));
    out.push_str(&format!(
        " Falsification attempts: {} ({} artifacts x rules)\n",
        score.total_falsification_attempts, score.total_artifacts
    ));
    out.push_str(&format!(
        " Falsifications succeeded: {}\n",
        score.successful_falsifications
    ));
    out.push_str(
        "═══════════════════════════════════════════════════════════\n",
    );

    out
}

/// Format check results as JSON
pub fn format_json(score: &ProjectScore) -> String {
    let mut artifacts = Vec::new();
    for a in &score.artifact_scores {
        let violations: Vec<String> = a
            .results
            .iter()
            .flat_map(|r| r.violations.iter().map(|v| v.to_string()))
            .collect();
        artifacts.push(format!(
            r#"    {{"name":"{}","score":{:.1},"grade":"{}","violations":{}}}"#,
            a.artifact_name,
            a.score,
            a.grade,
            serde_json_array(&violations)
        ));
    }

    format!(
        r#"{{"schema":"bashrs-comply-check-v1","total_artifacts":{},"compliant_artifacts":{},"score":{:.1},"grade":"{}","falsification_attempts":{},"successful_falsifications":{},"artifacts":[
{}
]}}"#,
        score.total_artifacts,
        score.compliant_artifacts,
        score.score,
        score.grade,
        score.total_falsification_attempts,
        score.successful_falsifications,
        artifacts.join(",\n")
    )
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

fn serde_json_array(items: &[String]) -> String {
    let quoted: Vec<String> = items.iter().map(|s| format!("\"{}\"", s.replace('"', "\\\""))).collect();
    format!("[{}]", quoted.join(","))
}
