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
