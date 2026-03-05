//! Corpus configuration: domain analysis, tier management, quality gates, dataset export, and publishing.

use crate::cli::args::DatasetExportFormat;
use crate::models::{Config, Error, Result};
use std::path::PathBuf;

pub(crate) fn corpus_domain_categories() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::domain_categories;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);
    let stats = domain_categories::categorize_corpus(&registry, &score.results);

    println!("{BOLD}Domain-Specific Corpus Categories (\u{00a7}11.11){RESET}");
    println!();

    let report = domain_categories::format_categories_report(&stats);
    for line in report.lines().skip(2) {
        let colored = line
            .replace("COMPLETE", &format!("{GREEN}COMPLETE{RESET}"))
            .replace("EMPTY", &format!("{RED}EMPTY{RESET}"))
            .replace("SPARSE", &format!("{YELLOW}SPARSE{RESET}"))
            .replace("PARTIAL", &format!("{CYAN}PARTIAL{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

pub(crate) fn corpus_domain_coverage() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::domain_categories;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);
    let stats = domain_categories::categorize_corpus(&registry, &score.results);

    println!("{BOLD}Domain Coverage Analysis (\u{00a7}11.11){RESET}");
    println!();

    let report = domain_categories::format_domain_coverage(&stats, &score);
    for line in report.lines().skip(2) {
        let colored = line
            .replace("COMPLETE", &format!("{GREEN}COMPLETE{RESET}"))
            .replace("EMPTY", &format!("{RED}EMPTY{RESET}"))
            .replace("SPARSE", &format!("{YELLOW}SPARSE{RESET}"))
            .replace("PARTIAL", &format!("{CYAN}PARTIAL{RESET}"))
            .replace("Coverage Gaps:", &format!("{YELLOW}Coverage Gaps:{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

pub(crate) fn corpus_domain_matrix() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::domain_categories;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);
    let stats = domain_categories::categorize_corpus(&registry, &score.results);

    println!("{BOLD}Cross-Category Quality Matrix (\u{00a7}11.11.9){RESET}");
    println!();

    let report = domain_categories::format_quality_matrix(&stats);
    for line in report.lines().skip(2) {
        let colored = line
            .replace("REQ", &format!("{GREEN}REQ{RESET}"))
            .replace("N/A", &format!("{DIM}N/A{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

pub(crate) fn corpus_tier_weights() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;
    use crate::corpus::tier_analysis;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);
    let analysis = tier_analysis::analyze_tiers(&registry, &score);

    println!("{BOLD}Tier-Weighted Corpus Scoring (\u{00a7}4.3){RESET}");
    println!();

    let report = tier_analysis::format_tier_weights(&analysis);
    for line in report.lines().skip(2) {
        let colored = line
            .replace("100.0%", &format!("{GREEN}100.0%{RESET}"))
            .replace("Weighted Score:", &format!("{BOLD}Weighted Score:{RESET}"))
            .replace("Weight Effect:", &format!("{BOLD}Weight Effect:{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

pub(crate) fn corpus_tier_analysis() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;
    use crate::corpus::tier_analysis;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);
    let analysis = tier_analysis::analyze_tiers(&registry, &score);

    println!("{BOLD}Tier Difficulty Analysis (\u{00a7}4.3){RESET}");
    println!();

    let report = tier_analysis::format_tier_analysis(&analysis);
    for line in report.lines().skip(2) {
        let colored = line
            .replace("No difference", &format!("{GREEN}No difference{RESET}"))
            .replace("Distribution:", &format!("{BOLD}Distribution:{RESET}"))
            .replace(
                "Scoring Comparison:",
                &format!("{BOLD}Scoring Comparison:{RESET}"),
            )
            .replace("Weight Impact", &format!("{BOLD}Weight Impact{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

pub(crate) fn corpus_tier_targets() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;
    use crate::corpus::tier_analysis;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);
    let analysis = tier_analysis::analyze_tiers(&registry, &score);

    println!("{BOLD}Tier Target Rate Comparison (\u{00a7}2.3/\u{00a7}4.3){RESET}");
    println!();

    let report = tier_analysis::format_tier_targets(&analysis);
    for line in report.lines().skip(2) {
        let colored = line
            .replace("PASS", &format!("{GREEN}PASS{RESET}"))
            .replace("FAIL", &format!("{RED}FAIL{RESET}"))
            .replace("ALL TARGETS MET", &format!("{GREEN}ALL TARGETS MET{RESET}"))
            .replace("TARGETS NOT MET", &format!("{RED}TARGETS NOT MET{RESET}"))
            .replace("COMFORTABLE", &format!("{GREEN}COMFORTABLE{RESET}"))
            .replace("AT RISK", &format!("{YELLOW}AT RISK{RESET}"))
            .replace("MARGINAL", &format!("{YELLOW}MARGINAL{RESET}"))
            .replace("BELOW TARGET", &format!("{RED}BELOW TARGET{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

pub(crate) fn corpus_quality_gates() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::quality_gates;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let log_path = PathBuf::from(".quality/convergence.log");
    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);
    let history = CorpusRunner::load_convergence_log(&log_path).unwrap_or_default();
    let thresholds = quality_gates::QualityThresholds::default();
    let gates = quality_gates::check_quality_gates(&score, &history, &thresholds);

    println!("{BOLD}Corpus Quality Gates (\u{00a7}9 / \u{00a7}8.1){RESET}");
    println!();

    let report = quality_gates::format_quality_gates(&gates);
    for line in report.lines().skip(2) {
        let colored = line
            .replace("PASS", &format!("{GREEN}PASS{RESET}"))
            .replace("FAIL", &format!("{RED}FAIL{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

pub(crate) fn corpus_metrics_check() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::quality_gates;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let log_path = PathBuf::from(".quality/convergence.log");
    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let start = std::time::Instant::now();
    let score = runner.run(&registry);
    let duration = start.elapsed();
    let history = CorpusRunner::load_convergence_log(&log_path).unwrap_or_default();
    let thresholds = quality_gates::PerformanceThresholds::default();
    let metrics = quality_gates::check_metrics(&score, duration, &history, &thresholds);

    println!("{BOLD}Corpus Performance Metrics (\u{00a7}9 / \u{00a7}8.2){RESET}");
    println!();

    let report = quality_gates::format_metrics_check(&metrics);
    for line in report.lines().skip(2) {
        let colored = line
            .replace("PASS", &format!("{GREEN}PASS{RESET}"))
            .replace("FAIL", &format!("{RED}FAIL{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

pub(crate) fn corpus_gate_status_cmd() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::quality_gates;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let log_path = PathBuf::from(".quality/convergence.log");
    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let start = std::time::Instant::now();
    let score = runner.run(&registry);
    let duration = start.elapsed();
    let history = CorpusRunner::load_convergence_log(&log_path).unwrap_or_default();
    let status = quality_gates::build_gate_status(&score, duration, &history);

    println!("{BOLD}Corpus Gate Status Summary (\u{00a7}9){RESET}");
    println!();

    let report = quality_gates::format_gate_status(&status);
    for line in report.lines().skip(2) {
        let colored = line
            .replace("\u{2713}", &format!("{GREEN}\u{2713}{RESET}"))
            .replace("\u{2717}", &format!("{RED}\u{2717}{RESET}"))
            .replace(
                "ALL GATES PASSED",
                &format!("{GREEN}ALL GATES PASSED{RESET}"),
            )
            .replace("GATES FAILED", &format!("{RED}GATES FAILED{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

pub(crate) fn corpus_export_dataset(
    format: DatasetExportFormat,
    output: Option<std::path::PathBuf>,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::dataset::{self, ExportFormat};

    let export_fmt = match format {
        DatasetExportFormat::Json => ExportFormat::Json,
        DatasetExportFormat::Jsonl => ExportFormat::JsonLines,
        DatasetExportFormat::Csv => ExportFormat::Csv,
        DatasetExportFormat::Classification => ExportFormat::Classification,
        DatasetExportFormat::MultiLabelClassification => ExportFormat::MultiLabelClassification,
    };

    let (score, data) = dataset::run_and_export(export_fmt);

    match output {
        Some(path) => {
            std::fs::write(&path, &data).map_err(|e| {
                Error::Validation(format!("Failed to write {}: {e}", path.display()))
            })?;
            println!(
                "{GREEN}\u{2713}{RESET} Exported {} entries to {} ({} format)",
                score.total,
                path.display(),
                export_fmt,
            );
        }
        None => {
            print!("{data}");
        }
    }

    Ok(())
}

pub(crate) fn corpus_dataset_info() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::dataset;
    use crate::corpus::registry::CorpusRegistry;

    let registry = CorpusRegistry::load_full();
    let info = dataset::dataset_info(&registry);

    println!("{BOLD}Corpus Dataset Info (\u{00a7}10.3){RESET}");
    println!();

    let table = dataset::format_dataset_info(&info);
    for line in table.lines() {
        let colored = line
            .replace("bash", &format!("{CYAN}bash{RESET}"))
            .replace("makefile", &format!("{YELLOW}makefile{RESET}"))
            .replace("dockerfile", &format!("{GREEN}dockerfile{RESET}"))
            .replace("string", &format!("{DIM}string{RESET}"))
            .replace("bool", &format!("{DIM}bool{RESET}"))
            .replace("float64", &format!("{DIM}float64{RESET}"))
            .replace("int32", &format!("{DIM}int32{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

pub(crate) fn corpus_publish_check() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::dataset;
    use crate::corpus::registry::{CorpusFormat, CorpusRegistry};
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

    let checks = dataset::check_publish_readiness(&score);

    println!("{BOLD}Hugging Face Publish Readiness (\u{00a7}10.3){RESET}");
    println!();

    let table = dataset::format_publish_checks(&checks);
    for line in table.lines() {
        let colored = line
            .replace("\u{2713} PASS", &format!("{GREEN}\u{2713} PASS{RESET}"))
            .replace("\u{2717} FAIL", &format!("{RED}\u{2717} FAIL{RESET}"))
            .replace(
                "Ready to publish",
                &format!("{GREEN}Ready to publish{RESET}"),
            )
            .replace("check(s) failed", &format!("{RED}check(s) failed{RESET}"));
        println!("  {colored}");
    }

    // Show target HF repos
    println!();
    println!("  {BOLD}Target Repositories:{RESET}");
    for (repo, fmt) in &[
        ("paiml/bashrs-corpus-bash", "Bash"),
        ("paiml/bashrs-corpus-makefile", "Makefile"),
        ("paiml/bashrs-corpus-dockerfile", "Dockerfile"),
        ("paiml/bashrs-convergence", "Convergence"),
    ] {
        let count = match *fmt {
            "Bash" => registry
                .entries
                .iter()
                .filter(|e| e.format == CorpusFormat::Bash)
                .count(),
            "Makefile" => registry
                .entries
                .iter()
                .filter(|e| e.format == CorpusFormat::Makefile)
                .count(),
            "Dockerfile" => registry
                .entries
                .iter()
                .filter(|e| e.format == CorpusFormat::Dockerfile)
                .count(),
            _ => 0,
        };
        if count > 0 {
            println!("    {CYAN}{repo}{RESET} ({count} entries)");
        } else {
            println!("    {CYAN}{repo}{RESET}");
        }
    }

    Ok(())
}

/// Generate synthetic conversations from corpus entries (SSC v11 Section 6).
pub(crate) fn corpus_generate_conversations(
    output: Option<PathBuf>,
    seed: u64,
    limit: Option<usize>,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::conversations::{generate_batch, to_jsonl};
    use crate::corpus::registry::CorpusRegistry;

    let registry = CorpusRegistry::load_full();
    let max = limit.unwrap_or(registry.entries.len());

    let batch: Vec<(&str, &str)> = registry
        .entries
        .iter()
        .take(max)
        .map(|e| (e.id.as_str(), e.input.as_str()))
        .collect();

    eprintln!(
        "{BOLD}Generating conversations from {} corpus entries (seed={seed})...{RESET}",
        batch.len()
    );

    let (conversations, report) = generate_batch(&batch, seed);
    let jsonl = to_jsonl(&conversations);

    match output {
        Some(ref path) => {
            std::fs::write(path, &jsonl).map_err(Error::Io)?;
            eprintln!(
                "{GREEN}Wrote {} conversations to {}{RESET}",
                conversations.len(),
                path.display()
            );
        }
        None => {
            print!("{jsonl}");
        }
    }

    eprintln!();
    eprintln!("{BOLD}Quality Report:{RESET}");
    eprintln!("  Total:       {}", report.total);
    eprintln!(
        "  Type A (classify): {} | Type B (fix): {} | Type C (debug): {} | Type D (safe): {}",
        report.type_a_count, report.type_b_count, report.type_c_count, report.type_d_count
    );
    eprintln!(
        "  Type D %:    {:.1}% (target: >=30%)",
        report.type_d_pct
    );
    eprintln!(
        "  Citations:   {:.0}%",
        report.rule_citation_accuracy * 100.0
    );
    eprintln!(
        "  Variants OK: {}",
        if report.variant_distribution_ok {
            format!("{GREEN}yes{RESET}")
        } else {
            format!("{RED}no{RESET}")
        }
    );
    eprintln!(
        "  Overall:     {}",
        if report.passed {
            format!("{GREEN}PASSED{RESET}")
        } else {
            format!("{YELLOW}FAILED{RESET}")
        }
    );

    Ok(())
}

/// Run all three baseline classifiers (SSC v11 S5.5).
pub(crate) fn corpus_baselines() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::baselines::{corpus_baseline_entries, run_all_baselines};
    use crate::corpus::evaluation::{format_comparison, format_report};

    eprintln!("{BOLD}Building baseline entries from corpus...{RESET}");

    let owned = corpus_baseline_entries();
    let entries: Vec<(&str, u8)> = owned.iter().map(|(s, l)| (s.as_str(), *l)).collect();

    let safe_count = entries.iter().filter(|(_, l)| *l == 0).count();
    let unsafe_count = entries.iter().filter(|(_, l)| *l == 1).count();
    eprintln!(
        "  Dataset: {} entries ({} safe, {} unsafe)",
        entries.len(),
        safe_count,
        unsafe_count
    );
    eprintln!();

    let reports = run_all_baselines(&entries);

    // Side-by-side comparison
    println!("{BOLD}=== SSC v11 Baseline Comparison (Section 5.5) ==={RESET}\n");
    print!("{}", format_comparison(&reports));
    println!();

    // Detailed per-baseline reports
    for report in &reports {
        println!("{BOLD}--- {} ---{RESET}", report.name);
        print!("{}", format_report(report));
        println!();
    }

    // Contract C-CLF-001 thresholds
    println!("{BOLD}Contract C-CLF-001 Thresholds:{RESET}");
    println!("  MCC CI lower > 0.2");
    println!("  Accuracy > 93.5%");
    println!("  Generalization >= 50%");
    println!();
    println!("Any ML classifier must beat ALL three baselines on MCC.");

    Ok(())
}

/// Audit label accuracy (SSC v11 S5.3, C-LABEL-001).
pub(crate) fn corpus_label_audit(limit: usize) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::label_audit::run_corpus_label_audit;

    eprintln!("{BOLD}Running label audit (C-LABEL-001, limit={limit})...{RESET}");

    let report = run_corpus_label_audit(limit);

    println!("{BOLD}=== SSC v11 Label Audit (Section 5.3, C-LABEL-001) ==={RESET}\n");
    println!("Audited {} unsafe labels:", report.total_audited);
    println!(
        "  Genuinely unsafe: {} ({:.1}%)",
        report.genuinely_unsafe, report.accuracy_pct
    );
    println!("  False positives:  {}", report.false_positives);
    println!("  Target:           >= 90% (C-LABEL-001)");
    println!(
        "  Status:           {}",
        if report.passed {
            format!("{GREEN}PASSED{RESET}")
        } else {
            format!("{RED}FAILED{RESET}")
        }
    );

    // Show false positives
    let false_pos: Vec<_> = report
        .results
        .iter()
        .filter(|r| !r.genuinely_unsafe)
        .collect();

    if !false_pos.is_empty() {
        println!("\n{BOLD}--- False Positives ---{RESET}\n");
        for r in false_pos.iter().take(10) {
            println!("  {} — {}", r.entry_id, r.reason);
            let preview = if r.script.len() > 60 {
                format!("{}...", &r.script[..60])
            } else {
                r.script.clone()
            };
            println!("    Script: {preview}");
        }
    }

    Ok(())
}

/// Run out-of-distribution generalization tests (SSC v11 S5.6).
pub(crate) fn corpus_generalization_tests() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::generalization_tests::{
        generalization_test_entries, GENERALIZATION_TARGET_PCT,
    };
    use crate::linter::lint_shell;

    println!("{BOLD}=== SSC v11 Generalization Tests (Section 5.6) ==={RESET}\n");

    let entries = generalization_test_entries();
    let mut caught = 0;
    let mut missed = Vec::new();

    for (script, category) in &entries {
        let result = lint_shell(script);
        let has_finding = result
            .diagnostics
            .iter()
            .any(|d| d.code.starts_with("SEC") || d.code.starts_with("DET"));
        if has_finding {
            caught += 1;
        } else {
            missed.push((*script, *category));
        }
    }

    let total = entries.len();
    let pct = caught as f64 / total as f64 * 100.0;
    let passed = pct >= GENERALIZATION_TARGET_PCT;

    println!("Total OOD scripts: {total}");
    println!("Caught by linter:  {caught} ({pct:.1}%)");
    println!("Missed:            {}", total - caught);
    println!("Target:            >= {GENERALIZATION_TARGET_PCT}%");
    println!(
        "Status:            {}",
        if passed {
            format!("{GREEN}PASSED{RESET}")
        } else {
            format!("{RED}FAILED{RESET}")
        }
    );

    if !missed.is_empty() {
        println!("\n{BOLD}--- Missed Scripts ---{RESET}\n");
        for (script, category) in &missed {
            let preview = if script.len() > 60 {
                format!("{}...", &script[..60])
            } else {
                (*script).to_string()
            };
            println!("  [{category}] {preview}");
        }
    }

    // Category breakdown
    println!("\n{BOLD}--- Category Breakdown ---{RESET}\n");
    let categories = [
        "injection",
        "non-determinism",
        "race-condition",
        "privilege",
        "exfiltration",
        "destructive",
    ];
    for cat in &categories {
        let cat_total = entries.iter().filter(|(_, c)| c == cat).count();
        let cat_caught = entries
            .iter()
            .filter(|(s, c)| {
                c == cat && {
                    let r = lint_shell(s);
                    r.diagnostics
                        .iter()
                        .any(|d| d.code.starts_with("SEC") || d.code.starts_with("DET"))
                }
            })
            .count();
        println!("  {cat:<20} {cat_caught}/{cat_total}");
    }

    Ok(())
}
