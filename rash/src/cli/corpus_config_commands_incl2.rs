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
    entrenar_format: bool,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::conversations::{generate_batch, to_entrenar_jsonl, to_jsonl};
    use crate::corpus::registry::CorpusRegistry;

    let registry = CorpusRegistry::load_full();
    let max = limit.unwrap_or(registry.entries.len());

    // v12: Transpile each entry to shell/Makefile/Dockerfile output first.
    // generate_batch receives (id, shell_code) not (id, rust_source).
    // This fixes the data quality issue where v3 conversations contained 85% Rust code.
    let config = crate::Config::default();
    let transpiled_entries: Vec<(String, String)> = registry
        .entries
        .iter()
        .take(max)
        .map(|e| {
            let shell_output = match e.format {
                crate::corpus::registry::CorpusFormat::Bash => crate::transpile(&e.input, &config)
                    .map(|s| crate::corpus::dataset::strip_shell_preamble(&s))
                    .unwrap_or_else(|_| e.input.clone()),
                crate::corpus::registry::CorpusFormat::Makefile => {
                    crate::transpile_makefile(&e.input, &config).unwrap_or_else(|_| e.input.clone())
                }
                crate::corpus::registry::CorpusFormat::Dockerfile => {
                    crate::transpile_dockerfile(&e.input, &config)
                        .unwrap_or_else(|_| e.input.clone())
                }
            };
            (e.id.clone(), shell_output)
        })
        .collect();

    let batch: Vec<(&str, &str)> = transpiled_entries
        .iter()
        .map(|(id, shell)| (id.as_str(), shell.as_str()))
        .collect();

    let format_name = if entrenar_format {
        "entrenar"
    } else {
        "chatml"
    };
    eprintln!(
        "{BOLD}Generating conversations from {} corpus entries (seed={seed}, format={format_name})...{RESET}",
        batch.len()
    );

    let (conversations, report) = generate_batch(&batch, seed);
    let jsonl = if entrenar_format {
        to_entrenar_jsonl(&conversations)
    } else {
        to_jsonl(&conversations)
    };

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
    eprintln!("  Type D %:    {:.1}% (target: >=30%)", report.type_d_pct);
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


include!("corpus_config_commands_incl2_incl2.rs");
