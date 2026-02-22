use crate::cli::args::{ComplyCommands, ComplyFormat, ComplyScopeArg, ComplyTrackCommands};
use crate::models::{Error, Result};
use std::path::Path;
use tracing::info;

pub(crate) fn handle_comply_command(command: ComplyCommands) -> Result<()> {
    match command {
        ComplyCommands::Init {
            scope,
            pzsh,
            strict,
        } => comply_init_command(scope, pzsh, strict),
        ComplyCommands::Check {
            path,
            scope,
            strict,
            failures_only,
            min_score,
            format,
        } => comply_check_command(&path, scope, strict, failures_only, min_score, format),
        ComplyCommands::Status { path, format } => comply_status_command(&path, format),
        ComplyCommands::Track { command } => handle_comply_track_command(command),
        ComplyCommands::Rules { format } => comply_rules_command(format),
    }
}

fn comply_load_or_default(path: &Path) -> crate::comply::config::ComplyConfig {
    use crate::comply::config::ComplyConfig;
    let version = env!("CARGO_PKG_VERSION");
    if ComplyConfig::exists(path) {
        ComplyConfig::load(path).unwrap_or_else(|| ComplyConfig::new_default(version))
    } else {
        ComplyConfig::new_default(version)
    }
}

fn comply_scope_filter(scope: Option<ComplyScopeArg>) -> Option<crate::comply::config::Scope> {
    scope.and_then(|s| match s {
        ComplyScopeArg::Project => Some(crate::comply::config::Scope::Project),
        ComplyScopeArg::User => Some(crate::comply::config::Scope::User),
        ComplyScopeArg::System => Some(crate::comply::config::Scope::System),
        ComplyScopeArg::All => None,
    })
}

fn comply_init_command(scope: ComplyScopeArg, pzsh: bool, strict: bool) -> Result<()> {
    use crate::comply::config::ComplyConfig;

    info!("Initializing comply manifest");

    if ComplyConfig::exists(Path::new(".")) {
        return Err(Error::Validation(
            ".bashrs/comply.toml already exists. Delete it first to reinitialize.".into(),
        ));
    }

    let mut config = ComplyConfig::new_default(env!("CARGO_PKG_VERSION"));
    apply_comply_scope(&mut config, scope);

    if pzsh {
        config.integration.pzsh = "enabled".to_string();
    }
    if strict {
        apply_comply_strict(&mut config);
    }

    config
        .save(Path::new("."))
        .map_err(|e| Error::Internal(format!("Failed to save comply.toml: {e}")))?;

    println!("Initialized .bashrs/comply.toml");
    println!(
        "  Scopes: project={} user={} system={}",
        config.scopes.project, config.scopes.user, config.scopes.system
    );
    if pzsh {
        println!("  pzsh integration: enabled");
    }
    if strict {
        println!("  Mode: strict (all rules enforced)");
    }

    Ok(())
}

fn apply_comply_scope(config: &mut crate::comply::config::ComplyConfig, scope: ComplyScopeArg) {
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

fn apply_comply_strict(config: &mut crate::comply::config::ComplyConfig) {
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

fn comply_check_command(
    path: &Path,
    scope: Option<ComplyScopeArg>,
    strict: bool,
    failures_only: bool,
    min_score: Option<u32>,
    format: ComplyFormat,
) -> Result<()> {
    use crate::comply::{runner, scoring::Grade};

    info!("Running compliance check on {}", path.display());

    let has_config = crate::comply::config::ComplyConfig::exists(path);
    let config = comply_load_or_default(path);
    let score = runner::run_check(path, comply_scope_filter(scope), &config);
    comply_output_score(&score, format, failures_only);

    // --strict: exit non-zero on grade F
    if strict && score.grade == Grade::F {
        return Err(Error::Validation(format!(
            "Compliance check failed: grade {} (score {:.0}/100)",
            score.grade, score.score
        )));
    }

    // --min-score N: exit non-zero if score < N
    if let Some(min) = min_score {
        if score.score < f64::from(min) {
            return Err(Error::Validation(format!(
                "Score {:.0} below minimum {} (grade {})",
                score.score, min, score.grade
            )));
        }
    }

    // Config thresholds: only enforce when explicit comply.toml exists
    if has_config {
        comply_enforce_thresholds(&score, &config)?;
    }

    Ok(())
}

/// Enforce thresholds from comply.toml (min_score, max_violations)
fn comply_enforce_thresholds(
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
    // Suppress unused warning — Grade is used for formatting
    let _ = Grade::F;
    Ok(())
}

fn comply_status_command(path: &Path, format: ComplyFormat) -> Result<()> {
    use crate::comply::runner;

    info!("Checking compliance status for {}", path.display());
    let config = comply_load_or_default(path);
    let score = runner::run_check(path, None, &config);
    comply_output_score(&score, format, false);
    Ok(())
}

fn comply_rules_command(format: ComplyFormat) -> Result<()> {
    use crate::comply::rules::RuleId;

    match format {
        ComplyFormat::Text => {
            println!("═══════════════════════════════════════════════════════════");
            println!("  COMPLIANCE RULES — 10 Falsifiable Hypotheses");
            println!("═══════════════════════════════════════════════════════════\n");
            println!(" {:<12} {:<22} {:>6}  Applies To", "Code", "Name", "Weight");
            println!("{}", "─".repeat(70));

            for rule in RuleId::all() {
                println!(
                    " {:<12} {:<22} {:>4}    {}",
                    rule.code(),
                    rule.name(),
                    rule.weight(),
                    rule.applies_to().join(", ")
                );
                println!("              {}", rule.description());
                println!();
            }

            let total_weight: u32 = RuleId::all().iter().map(|r| r.weight()).sum();
            println!("{}", "─".repeat(70));
            println!(
                " {} rules | total weight: {} | suppress: # comply:disable=COMPLY-NNN",
                RuleId::all().len(),
                total_weight
            );
            println!("═══════════════════════════════════════════════════════════");
        }
        ComplyFormat::Json => {
            let rules: Vec<String> = RuleId::all()
                .iter()
                .map(|r| {
                    format!(
                        r#"  {{"code":"{}","name":"{}","weight":{},"applies_to":[{}],"description":"{}"}}"#,
                        r.code(),
                        r.name(),
                        r.weight(),
                        r.applies_to()
                            .iter()
                            .map(|s| format!("\"{}\"", s))
                            .collect::<Vec<_>>()
                            .join(","),
                        r.description()
                    )
                })
                .collect();
            println!(
                "{{\"schema\":\"bashrs-comply-rules-v1\",\"rules\":[\n{}\n]}}",
                rules.join(",\n")
            );
        }
        ComplyFormat::Markdown => {
            println!("# Compliance Rules\n");
            println!("| Code | Name | Weight | Applies To | Description |");
            println!("|------|------|--------|------------|-------------|");
            for rule in RuleId::all() {
                println!(
                    "| {} | {} | {} | {} | {} |",
                    rule.code(),
                    rule.name(),
                    rule.weight(),
                    rule.applies_to().join(", "),
                    rule.description()
                );
            }
            println!("\n*Suppress with: `# comply:disable=COMPLY-NNN`*");
        }
    }
    Ok(())
}

fn comply_output_score(
    score: &crate::comply::scoring::ProjectScore,
    format: ComplyFormat,
    failures_only: bool,
) {
    use crate::comply::runner;
    match format {
        ComplyFormat::Text => {
            if failures_only {
                print!("{}", runner::format_human_failures_only(score));
            } else {
                print!("{}", runner::format_human(score));
            }
        }
        ComplyFormat::Json => println!("{}", runner::format_json(score)),
        ComplyFormat::Markdown => {
            println!("# Compliance Report\n");
            println!("**Score**: {:.0}/100 ({})\n", score.score, score.grade);
            println!("| Artifact | Score | Grade | Status |");
            println!("|----------|-------|-------|--------|");
            for a in &score.artifact_scores {
                if failures_only && a.violations == 0 {
                    continue;
                }
                let status = if a.violations == 0 {
                    "COMPLIANT"
                } else {
                    "NON-COMPLIANT"
                };
                println!(
                    "| {} | {:.0} | {} | {} |",
                    a.artifact_name, a.score, a.grade, status
                );
            }
        }
    }
}

fn handle_comply_track_command(command: ComplyTrackCommands) -> Result<()> {
    match command {
        ComplyTrackCommands::Discover { path, scope } => comply_track_discover(&path, scope),
        ComplyTrackCommands::List { path, scope } => comply_track_list(&path, scope),
    }
}

fn comply_track_discover(path: &Path, scope: ComplyScopeArg) -> Result<()> {
    use crate::comply::discovery;

    info!("Discovering artifacts in {}", path.display());

    if matches!(scope, ComplyScopeArg::All) {
        return comply_track_discover_all(path);
    }

    let scope_val = comply_scope_to_internal(scope);
    let artifacts = discovery::discover(path, scope_val);
    comply_print_artifact_list(scope_val, &artifacts);
    Ok(())
}

fn comply_track_discover_all(path: &Path) -> Result<()> {
    use crate::comply::{config::Scope, discovery};

    let mut total = 0;
    for s in &[Scope::Project, Scope::User, Scope::System] {
        let artifacts = discovery::discover(path, *s);
        if !artifacts.is_empty() {
            println!("{:?} scope ({} artifacts):", s, artifacts.len());
            for a in &artifacts {
                println!("  {} [{:?}]", a.display_name(), a.kind);
            }
            total += artifacts.len();
        }
    }
    println!("\nTotal: {} artifacts discovered", total);
    Ok(())
}

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
