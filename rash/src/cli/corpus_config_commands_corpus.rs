pub(crate) fn corpus_ssc_report(json: bool, gate: bool) -> Result<()> {
    use crate::corpus::ssc_report::generate_ssc_report;

    eprintln!("Generating SSC v11 readiness report...");
    let report = generate_ssc_report();
    corpus_ssc_report_with(&report, json, gate)
}

/// PMAT-257: body of `corpus_ssc_report`, split so a test can pass a
/// synthetic `SscStatusReport` instead of building one from
/// `CorpusRegistry::load_full()` (which `generate_ssc_report` scores
/// end-to-end and costs minutes).
pub(crate) fn corpus_ssc_report_with(
    report: &crate::corpus::ssc_report::SscStatusReport,
    json: bool,
    gate: bool,
) -> Result<()> {
    use crate::corpus::ssc_report::{format_ssc_report, SscStatus};

    if json {
        let json_str = serde_json::to_string_pretty(report)
            .map_err(|e| Error::Validation(format!("JSON serialization failed: {e}")))?;
        println!("{json_str}");
    } else {
        print!("{}", format_ssc_report(report));
    }

    if report.overall_ready {
        eprintln!("All sections ready for classifier training.");
    } else {
        eprintln!("Some sections need attention before classifier training.");
    }

    if gate {
        let failures: Vec<&str> = report
            .sections
            .iter()
            .filter(|s| s.status == SscStatus::Fail)
            .map(|s| s.name.as_str())
            .collect();
        if !failures.is_empty() {
            return Err(Error::Validation(format!(
                "SSC gate failed: {} section(s) not ready: {}",
                failures.len(),
                failures.join(", ")
            )));
        }
    }

    Ok(())
}

pub(crate) fn corpus_model_card(output: Option<PathBuf>) -> Result<()> {
    use crate::corpus::model_card;
    let card = model_card::generate_model_card();
    corpus_model_card_with(&card, output)
}

/// PMAT-257: body of `corpus_model_card`, split so a test can pass an
/// already-generated card string instead of building one from
/// `CorpusRegistry::load_full()`.
pub(crate) fn corpus_model_card_with(card: &str, output: Option<PathBuf>) -> Result<()> {
    use crate::cli::color::*;

    eprintln!("{BOLD}Generating HuggingFace model card...{RESET}");

    match output {
        Some(path) => {
            std::fs::write(&path, card).map_err(|e| {
                Error::Validation(format!("Failed to write {}: {e}", path.display()))
            })?;
            eprintln!(
                "{GREEN}\u{2713}{RESET} Model card written to {}",
                path.display()
            );
        }
        None => {
            print!("{card}");
        }
    }

    Ok(())
}

pub(crate) fn corpus_training_config(output: Option<PathBuf>, json: bool) -> Result<()> {
    use crate::corpus::training_config;
    let config = training_config::generate_training_config();
    corpus_training_config_with(&config, output, json)
}

/// PMAT-257: body of `corpus_training_config`, split so a test can pass an
/// already-generated `TrainingConfig` instead of building one from
/// `CorpusRegistry::load_full()`.
pub(crate) fn corpus_training_config_with(
    config: &crate::corpus::training_config::TrainingConfig,
    output: Option<PathBuf>,
    json: bool,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::training_config;

    eprintln!("{BOLD}Generating entrenar training configuration...{RESET}");

    let data = if json {
        training_config::format_json(config)
    } else {
        training_config::format_yaml(config)
    };

    match output {
        Some(path) => {
            std::fs::write(&path, &data).map_err(|e| {
                Error::Validation(format!("Failed to write {}: {e}", path.display()))
            })?;
            eprintln!(
                "{GREEN}\u{2713}{RESET} Training config written to {} ({} format)",
                path.display(),
                if json { "JSON" } else { "YAML" }
            );
        }
        None => {
            print!("{data}");
        }
    }

    Ok(())
}

pub(crate) fn corpus_publish_dataset(output: PathBuf) -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    corpus_publish_dataset_with(&CorpusRegistry::load_full(), output)
}

/// PMAT-257: body of `corpus_publish_dataset`, split so a test can pass a
/// small synthetic registry instead of lint-scoring the full corpus via
/// `corpus_baseline_entries()` / `generate_model_card()` / `generate_training_config()`.
pub(crate) fn corpus_publish_dataset_with(
    registry: &crate::corpus::registry::CorpusRegistry,
    output: PathBuf,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::baselines::corpus_baseline_entries_from;
    use crate::corpus::dataset::{split_and_validate, ClassificationRow};
    use crate::corpus::model_card;
    use crate::corpus::training_config;

    eprintln!("{BOLD}Building HuggingFace-ready dataset...{RESET}");

    // Create output directory
    std::fs::create_dir_all(&output)
        .map_err(|e| Error::Validation(format!("Cannot create {}: {e}", output.display())))?;

    // Step 1: Split dataset
    let owned = corpus_baseline_entries_from(registry);
    let total = owned.len();
    let rows: Vec<ClassificationRow> = owned
        .into_iter()
        .map(|(input, label)| ClassificationRow { input, label })
        .collect();
    let result = split_and_validate(rows, 2);

    // Step 2: Write split files
    write_split_file(&output, "train", &result.train)?;
    write_split_file(&output, "val", &result.val)?;
    write_split_file(&output, "test", &result.test)?;

    // Step 3: Write model card (README.md)
    let card = model_card::generate_model_card_from(registry);
    let readme_path = output.join("README.md");
    std::fs::write(&readme_path, &card).map_err(|e| {
        Error::Validation(format!("Failed to write {}: {e}", readme_path.display()))
    })?;

    // Step 4: Write training config
    let config = training_config::generate_training_config_from(registry);
    let config_path = output.join("training_config.yaml");
    std::fs::write(&config_path, training_config::format_yaml(&config)).map_err(|e| {
        Error::Validation(format!("Failed to write {}: {e}", config_path.display()))
    })?;

    // Summary
    eprintln!(
        "\n{GREEN}\u{2713}{RESET} {BOLD}Dataset published to {}{RESET}",
        output.display()
    );
    eprintln!("  README.md        \u{2014} HuggingFace model card");
    eprintln!("  train.jsonl      \u{2014} {} entries", result.train.len());
    eprintln!("  val.jsonl        \u{2014} {} entries", result.val.len());
    eprintln!("  test.jsonl       \u{2014} {} entries", result.test.len());
    eprintln!("  training_config.yaml \u{2014} entrenar config");
    eprintln!("  Total: {total} entries\n");
    eprintln!(
        "To publish: `huggingface-cli upload paiml/shell-safety-classifier {}`",
        output.display()
    );

    Ok(())
}

fn write_split_file(
    dir: &Path,
    name: &str,
    rows: &[crate::corpus::dataset::ClassificationRow],
) -> Result<()> {
    use std::fmt::Write as _;

    let path = dir.join(format!("{name}.jsonl"));
    let mut out = String::new();
    for row in rows {
        let escaped = row
            .input
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n");
        let _ = writeln!(out, r#"{{"input":"{}","label":{}}}"#, escaped, row.label);
    }
    std::fs::write(&path, out)
        .map_err(|e| Error::Validation(format!("Failed to write {}: {e}", path.display())))?;
    Ok(())
}
/// Publish HuggingFace-ready conversation dataset (S6.6).
///
/// Generates conversations from full corpus, writes JSONL + dataset README.
pub(crate) fn corpus_publish_conversations(output: PathBuf, seed: u64) -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    corpus_publish_conversations_with(&CorpusRegistry::load_full(), output, seed)
}

/// PMAT-257: body of `corpus_publish_conversations`, split so a test can
/// pass a small synthetic registry instead of `CorpusRegistry::load_full()`.
pub(crate) fn corpus_publish_conversations_with(
    registry: &crate::corpus::registry::CorpusRegistry,
    output: PathBuf,
    seed: u64,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::conversations::{generate_batch, generate_dataset_readme, to_jsonl};

    eprintln!("{BOLD}Building conversation dataset (seed={seed})...{RESET}");

    let batch: Vec<(&str, &str)> = registry
        .entries
        .iter()
        .map(|e| (e.id.as_str(), e.input.as_str()))
        .collect();

    let (conversations, report) = generate_batch(&batch, seed);

    // Create output directory
    std::fs::create_dir_all(&output)
        .map_err(|e| Error::Validation(format!("Cannot create {}: {e}", output.display())))?;

    // Write conversations JSONL
    let jsonl = to_jsonl(&conversations);
    let jsonl_path = output.join("conversations.jsonl");
    std::fs::write(&jsonl_path, &jsonl)
        .map_err(|e| Error::Validation(format!("Failed to write {}: {e}", jsonl_path.display())))?;

    // Write dataset README
    let readme = generate_dataset_readme(&report);
    let readme_path = output.join("README.md");
    std::fs::write(&readme_path, &readme).map_err(|e| {
        Error::Validation(format!("Failed to write {}: {e}", readme_path.display()))
    })?;

    // Summary
    eprintln!(
        "\n{GREEN}\u{2713}{RESET} {BOLD}Conversation dataset published to {}{RESET}",
        output.display()
    );
    eprintln!("  README.md            \u{2014} HuggingFace dataset card");
    eprintln!(
        "  conversations.jsonl  \u{2014} {} conversations",
        conversations.len()
    );
    eprintln!();
    eprintln!("{BOLD}Quality Report:{RESET}");
    eprintln!(
        "  Type A (classify): {} | Type B (fix): {} | Type C (debug): {} | Type D (safe): {}",
        report.type_a_count, report.type_b_count, report.type_c_count, report.type_d_count
    );
    eprintln!("  Type D %:    {:.1}% (target: >=30%)", report.type_d_pct);
    eprintln!("  Empty:       {}", report.empty_responses);
    eprintln!(
        "  Status:      {}",
        if report.passed {
            format!("{GREEN}PASSED{RESET}")
        } else {
            format!("{RED}FAILED{RESET}")
        }
    );
    eprintln!();
    eprintln!(
        "To publish: `huggingface-cli upload paiml/shell-safety-conversations {}`",
        output.display()
    );

    Ok(())
}
