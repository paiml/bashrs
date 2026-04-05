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

/// Validate tokenizer quality on shell constructs (SSC v11 S5.2, C-TOK-001).
pub(crate) fn corpus_tokenizer_validation() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::tokenizer_validation::{run_validation, shell_constructs};

    println!("{BOLD}=== SSC v11 Tokenizer Validation (Section 5.2, C-TOK-001) ==={RESET}\n");

    let constructs = shell_constructs();
    println!("Shell constructs: {}\n", constructs.len());

    // Use whitespace tokenizer as baseline (real BPE plugs in via entrenar)
    let report = run_validation(|construct| {
        construct
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    });

    println!("Tokenizer:        whitespace (baseline)");
    println!(
        "Acceptable:       {} ({:.1}%)",
        report.acceptable_count, report.acceptable_pct
    );
    println!("Unacceptable:     {}", report.unacceptable_count);
    println!("Target:           >= 70% (C-TOK-001)");
    println!(
        "Status:           {}",
        if report.passed {
            format!("{GREEN}PASSED{RESET}")
        } else {
            format!("{RED}FAILED{RESET}")
        }
    );

    // Show failures
    let failures: Vec<_> = report.results.iter().filter(|r| !r.acceptable).collect();
    if !failures.is_empty() {
        println!("\n{BOLD}--- Failed Constructs ---{RESET}\n");
        for r in &failures {
            println!("  {} {:30} — {}", r.id, r.construct, r.reason);
        }
    }

    println!("\nNote: This uses a whitespace tokenizer as baseline.");
    println!("Plug in a real BPE tokenizer via entrenar for production validation.");

    Ok(())
}

/// Run all SSC contract validations (pre-training gate).
pub(crate) fn corpus_validate_contracts() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::contract_validation::run_all_contracts;

    eprintln!("{BOLD}Running SSC v11 contract validation (pre-training gate)...{RESET}\n");

    let report = run_all_contracts();

    println!("{BOLD}=== SSC v11 Contract Validation ==={RESET}\n");

    for c in &report.contracts {
        let status = if c.passed {
            format!("{GREEN}PASS{RESET}")
        } else {
            format!("{RED}FAIL{RESET}")
        };
        println!(
            "  [{status}] {:<15} {:<25} value={:.1} threshold={:.1}",
            c.id, c.name, c.value, c.threshold
        );
        println!("         {}", c.detail);
    }

    println!();
    println!(
        "{BOLD}Result: {}/{} contracts passed{RESET}",
        report.passed_count,
        report.contracts.len()
    );

    if report.all_passed {
        println!("{GREEN}All contracts passed. Ready for classifier training.{RESET}");
    } else {
        println!("{RED}Some contracts failed. Fix before proceeding to training.{RESET}");
    }

    Ok(())
}

/// Export dataset with train/val/test splits.
pub(crate) fn corpus_export_splits(output: Option<PathBuf>, input: Option<PathBuf>) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::dataset::{split_and_validate, ClassificationRow};

    let rows: Vec<ClassificationRow> = if let Some(ref input_path) = input {
        // Fast path: read from pre-merged JSONL
        eprintln!("{BOLD}Loading from {}...{RESET}", input_path.display());
        let content = std::fs::read_to_string(input_path)?;
        content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| {
                let v: serde_json::Value = serde_json::from_str(l).ok()?;
                let input_text = v
                    .get("instruction")
                    .or_else(|| v.get("input"))
                    .or_else(|| v.get("unsafe_script"))
                    .or_else(|| v.get("script"))
                    .and_then(|v| v.as_str())?
                    .to_string();
                let label = v.get("label").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
                Some(ClassificationRow {
                    input: input_text,
                    label,
                })
            })
            .collect()
    } else {
        // Slow path: transpile full corpus
        use crate::corpus::baselines::corpus_baseline_entries;
        eprintln!("{BOLD}Building classification dataset from corpus...{RESET}");
        let owned = corpus_baseline_entries();
        owned
            .into_iter()
            .map(|(input, label)| ClassificationRow { input, label })
            .collect()
    };

    let total = rows.len();
    eprintln!("  Total entries: {total}");

    let result = split_and_validate(rows, 2);

    let train_safe = result.train.iter().filter(|r| r.label == 0).count();
    let train_unsafe = result.train.iter().filter(|r| r.label == 1).count();
    let val_safe = result.val.iter().filter(|r| r.label == 0).count();
    let val_unsafe = result.val.iter().filter(|r| r.label == 1).count();
    let test_safe = result.test.iter().filter(|r| r.label == 0).count();
    let test_unsafe = result.test.iter().filter(|r| r.label == 1).count();

    println!("{BOLD}=== SSC v11 Dataset Split (alimentar-compatible) ==={RESET}\n");
    println!(
        "  {:<8} {:>6} {:>6} {:>6}  {:>6}",
        "Split", "Total", "Safe", "Unsafe", "%Unsafe"
    );
    println!("  {}", "-".repeat(45));
    println!(
        "  {:<8} {:>6} {:>6} {:>6}  {:>5.1}%",
        "Train",
        result.train.len(),
        train_safe,
        train_unsafe,
        train_unsafe as f64 / result.train.len() as f64 * 100.0
    );
    println!(
        "  {:<8} {:>6} {:>6} {:>6}  {:>5.1}%",
        "Val",
        result.val.len(),
        val_safe,
        val_unsafe,
        val_unsafe as f64 / result.val.len() as f64 * 100.0
    );
    println!(
        "  {:<8} {:>6} {:>6} {:>6}  {:>5.1}%",
        "Test",
        result.test.len(),
        test_safe,
        test_unsafe,
        test_unsafe as f64 / result.test.len() as f64 * 100.0
    );
    println!(
        "  {:<8} {:>6} {:>6} {:>6}  {:>5.1}%",
        "Total",
        total,
        train_safe + val_safe + test_safe,
        train_unsafe + val_unsafe + test_unsafe,
        (train_unsafe + val_unsafe + test_unsafe) as f64 / total as f64 * 100.0
    );

    // Validation status
    println!(
        "\n  Validation: {}",
        if result.validation.passed {
            format!("{GREEN}PASSED{RESET}")
        } else {
            format!("{RED}FAILED{RESET}")
        }
    );
    for err in &result.validation.errors {
        println!("    - {RED}ERROR{RESET}: {err}");
    }
    for warn in &result.validation.warnings {
        println!("    - {YELLOW}WARN{RESET}: {warn}");
    }

    // Write split files if output dir specified
    if let Some(ref dir) = output {
        std::fs::create_dir_all(dir).map_err(Error::Io)?;

        let write_split = |name: &str, rows: &[ClassificationRow]| -> std::io::Result<()> {
            let path = dir.join(format!("{name}.jsonl"));
            let mut out = String::new();
            for row in rows {
                use std::fmt::Write as _;
                // Use serde_json for correct escaping of all control chars
                let json_input = serde_json::to_string(&row.input)
                    .unwrap_or_else(|_| format!("\"{}\"", row.input));
                // json_input already includes surrounding quotes
                let _ = writeln!(out, r#"{{"input":{},"label":{}}}"#, json_input, row.label);
            }
            std::fs::write(&path, out)?;
            Ok(())
        };

        write_split("train", &result.train).map_err(Error::Io)?;
        write_split("val", &result.val).map_err(Error::Io)?;
        write_split("test", &result.test).map_err(Error::Io)?;

        eprintln!("\n{GREEN}Wrote split files to {}{RESET}", dir.display());
        eprintln!(
            "  {}/train.jsonl ({} entries)",
            dir.display(),
            result.train.len()
        );
        eprintln!(
            "  {}/val.jsonl ({} entries)",
            dir.display(),
            result.val.len()
        );
        eprintln!(
            "  {}/test.jsonl ({} entries)",
            dir.display(),
            result.test.len()
        );
    }

    Ok(())
}


include!("corpus_config_commands_incl2_incl2_incl2.rs");
