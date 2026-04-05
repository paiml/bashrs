fn test_CORPUS_029_tier4_dockerfile_transpilation() {
    let registry = bashrs::corpus::CorpusRegistry::load_all_with_adversarial();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run_format(&registry, bashrs::corpus::CorpusFormat::Dockerfile);

    for result in &score.results {
        if !result.transpiled {
            eprintln!(
                "TIER4 FALSIFIER: {} - {}",
                result.id,
                result.error.as_deref().unwrap_or("unknown")
            );
        }
    }

    eprintln!(
        "Dockerfile T1-T4: {}/{} passed ({:.1}%), score: {:.1}, grade: {}",
        score.passed,
        score.total,
        score.rate * 100.0,
        score.score,
        score.grade
    );
}

#[test]
fn test_CORPUS_030_tier4_aggregate_score() {
    let registry = bashrs::corpus::CorpusRegistry::load_all_with_adversarial();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run(&registry);

    eprintln!("\n=== CORPUS QUALITY REPORT (Tier 1-4, Adversarial) ===");
    eprintln!("Total entries: {}", score.total);
    eprintln!("Passed: {}", score.passed);
    eprintln!("Failed: {}", score.failed);
    eprintln!("Rate: {:.1}%", score.rate * 100.0);
    eprintln!("Score: {:.1}/100", score.score);
    eprintln!("Grade: {}", score.grade);
    eprintln!("====================================================\n");

    for result in &score.results {
        if !result.transpiled {
            eprintln!(
                "  FALSIFIER: {} - {}",
                result.id,
                result.error.as_deref().unwrap_or("unknown")
            );
        }
    }

    assert!(score.total > 85, "Should run more than 85 entries");
}

// =============================================================================
// Tier 5 Corpus Tests (Production - real-world patterns)
// =============================================================================

#[test]
fn test_CORPUS_031_tier5_loads_full_corpus() {
    let registry = bashrs::corpus::CorpusRegistry::load_full();
    assert!(
        registry.len() > 110,
        "Full corpus should have more than 110 entries"
    );
}

#[test]
fn test_CORPUS_032_tier5_bash_transpilation() {
    let registry = bashrs::corpus::CorpusRegistry::load_full();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run_format(&registry, bashrs::corpus::CorpusFormat::Bash);

    for result in &score.results {
        if !result.transpiled {
            eprintln!(
                "TIER5 FALSIFIER: {} - {}",
                result.id,
                result.error.as_deref().unwrap_or("unknown")
            );
        }
    }

    eprintln!(
        "Bash FULL: {}/{} passed ({:.1}%), score: {:.1}, grade: {}",
        score.passed,
        score.total,
        score.rate * 100.0,
        score.score,
        score.grade
    );
}

#[test]
fn test_CORPUS_033_tier5_makefile_transpilation() {
    let registry = bashrs::corpus::CorpusRegistry::load_full();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run_format(&registry, bashrs::corpus::CorpusFormat::Makefile);

    for result in &score.results {
        if !result.transpiled {
            eprintln!(
                "TIER5 FALSIFIER: {} - {}",
                result.id,
                result.error.as_deref().unwrap_or("unknown")
            );
        }
    }

    eprintln!(
        "Makefile FULL: {}/{} passed ({:.1}%), score: {:.1}, grade: {}",
        score.passed,
        score.total,
        score.rate * 100.0,
        score.score,
        score.grade
    );
}

#[test]
fn test_CORPUS_034_tier5_dockerfile_transpilation() {
    let registry = bashrs::corpus::CorpusRegistry::load_full();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run_format(&registry, bashrs::corpus::CorpusFormat::Dockerfile);

    for result in &score.results {
        if !result.transpiled {
            eprintln!(
                "TIER5 FALSIFIER: {} - {}",
                result.id,
                result.error.as_deref().unwrap_or("unknown")
            );
        }
    }

    eprintln!(
        "Dockerfile FULL: {}/{} passed ({:.1}%), score: {:.1}, grade: {}",
        score.passed,
        score.total,
        score.rate * 100.0,
        score.score,
        score.grade
    );
}

#[test]
fn test_CORPUS_035_tier5_full_aggregate_score() {
    let registry = bashrs::corpus::CorpusRegistry::load_full();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run(&registry);

    eprintln!("\n=== FULL CORPUS QUALITY REPORT (Tiers 1-5) ===");
    eprintln!("Total entries: {}", score.total);
    eprintln!("Passed: {}", score.passed);
    eprintln!("Failed: {}", score.failed);
    eprintln!("Rate: {:.1}%", score.rate * 100.0);
    eprintln!("Score: {:.1}/100", score.score);
    eprintln!("Grade: {}", score.grade);
    eprintln!("================================================\n");

    for result in &score.results {
        if !result.transpiled {
            eprintln!(
                "  FALSIFIER: {} - {}",
                result.id,
                result.error.as_deref().unwrap_or("unknown")
            );
        }
    }

    assert!(
        score.total > 110,
        "Full corpus should run more than 110 entries"
    );
}

// =============================================================================
// V2 Falsification: Component-Level Diagnostic
// =============================================================================

#[test]
fn test_CORPUS_036_v2_component_breakdown() {
    let registry = bashrs::corpus::CorpusRegistry::load_full();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run(&registry);

    let total = score.results.len();
    let transpiled = score.results.iter().filter(|r| r.transpiled).count();
    let contains = score.results.iter().filter(|r| r.output_contains).count();
    let exact = score.results.iter().filter(|r| r.output_exact).count();
    let behavioral = score.results.iter().filter(|r| r.output_behavioral).count();
    let schema = score.results.iter().filter(|r| r.schema_valid).count();
    let has_test = score.results.iter().filter(|r| r.has_test).count();
    let avg_coverage: f64 =
        score.results.iter().map(|r| r.coverage_ratio).sum::<f64>() / total as f64;
    let lint = score.results.iter().filter(|r| r.lint_clean).count();
    let determ = score.results.iter().filter(|r| r.deterministic).count();
    let metamorphic = score
        .results
        .iter()
        .filter(|r| r.metamorphic_consistent)
        .count();
    let cross_shell = score.results.iter().filter(|r| r.cross_shell_agree).count();

    eprintln!("\n=== V2 COMPONENT FALSIFICATION REPORT ===");
    eprintln!(
        "A  Transpilation:  {}/{} ({:.1}%) → {:.0}/30 pts",
        transpiled,
        total,
        transpiled as f64 / total as f64 * 100.0,
        transpiled as f64 / total as f64 * 30.0
    );
    eprintln!(
        "B1 Containment:    {}/{} ({:.1}%) → {:.0}/10 pts",
        contains,
        total,
        contains as f64 / total as f64 * 100.0,
        contains as f64 / total as f64 * 10.0
    );
    eprintln!(
        "B2 Exact match:    {}/{} ({:.1}%) → {:.0}/8 pts",
        exact,
        total,
        exact as f64 / total as f64 * 100.0,
        exact as f64 / total as f64 * 8.0
    );
    eprintln!(
        "B3 Behavioral:     {}/{} ({:.1}%) → {:.0}/7 pts",
        behavioral,
        total,
        behavioral as f64 / total as f64 * 100.0,
        behavioral as f64 / total as f64 * 7.0
    );
    eprintln!(
        "   Schema valid:   {}/{} ({:.1}%)",
        schema,
        total,
        schema as f64 / total as f64 * 100.0
    );
    eprintln!(
        "C  Coverage (V2-8): avg {:.1}% → {:.1}/15 pts (has_test: {}/{})",
        avg_coverage * 100.0,
        avg_coverage * 15.0,
        has_test,
        total
    );
    eprintln!(
        "D  Lint clean:     {}/{} ({:.1}%) → {:.0}/10 pts",
        lint,
        total,
        lint as f64 / total as f64 * 100.0,
        lint as f64 / total as f64 * 10.0
    );
    eprintln!(
        "E  Deterministic:  {}/{} ({:.1}%) → {:.0}/10 pts",
        determ,
        total,
        determ as f64 / total as f64 * 100.0,
        determ as f64 / total as f64 * 10.0
    );
    eprintln!(
        "F  Metamorphic:    {}/{} ({:.1}%) → {:.0}/5 pts",
        metamorphic,
        total,
        metamorphic as f64 / total as f64 * 100.0,
        metamorphic as f64 / total as f64 * 5.0
    );
    eprintln!(
        "G  Cross-shell:    {}/{} ({:.1}%) → {:.0}/5 pts",
        cross_shell,
        total,
        cross_shell as f64 / total as f64 * 100.0,
        cross_shell as f64 / total as f64 * 5.0
    );
    eprintln!(
        "\n   Aggregate v2 score: {:.1}/100 ({})",
        score.score, score.grade
    );

    // Per-format breakdown
    for fs in &score.format_scores {
        eprintln!(
            "   Format {}: {}/{} ({:.1}%), score: {:.1}, grade: {}",
            fs.format,
            fs.passed,
            fs.total,
            fs.rate * 100.0,
            fs.score,
            fs.grade
        );
    }

    log_v2_failures(&score);

    // Persist convergence log
    let log_path = std::path::Path::new(".quality/convergence.log");
    let previous = bashrs::corpus::CorpusRunner::load_convergence_log(log_path).unwrap_or_default();
    let iteration = previous.len() as u32 + 1;
    let previous_rate = previous.last().map_or(0.0, |e| e.rate);
    let entry = runner.convergence_entry(
        &score,
        iteration,
        &chrono_date_today(),
        previous_rate,
        &format!("v2 score {:.1}/100 ({})", score.score, score.grade),
    );
    if let Err(e) = bashrs::corpus::CorpusRunner::append_convergence_log(&entry, log_path) {
        eprintln!("WARNING: Failed to write convergence log: {}", e);
    }
}

/// Get today's date as YYYY-MM-DD string without chrono dependency.
fn chrono_date_today() -> String {
    // Use system time to derive ISO date
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Simple days-since-epoch calculation
    let days = secs / 86400;
    // Gregorian calendar from days since 1970-01-01
    let (year, month, day) = days_to_ymd(days);
    format!("{year:04}-{month:02}-{day:02}")
}

/// Convert days since Unix epoch to (year, month, day).
fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Log failure details for each v2 component.
fn log_v2_failures(score: &bashrs::corpus::CorpusScore) {
    let failure_categories: Vec<(&str, Vec<&bashrs::corpus::CorpusResult>)> = vec![
        (
            "SCHEMA GATE FAILURES",
            score
                .results
                .iter()
                .filter(|r| r.transpiled && !r.schema_valid)
                .collect(),
        ),
        (
            "CONTAINMENT PASS / EXACT MATCH FAIL (B1=ok, B2=fail)",
            score
                .results
                .iter()
                .filter(|r| r.output_contains && !r.output_exact)
                .collect(),
        ),
        (
            "BEHAVIORAL FAILURES (B3)",
            score
                .results
                .iter()
                .filter(|r| r.transpiled && !r.output_behavioral)
                .collect(),
        ),
        (
            "METAMORPHIC RELATION FAILURES",
            score
                .results
                .iter()
                .filter(|r| r.transpiled && !r.metamorphic_consistent)
                .collect(),
        ),
        (
            "CROSS-SHELL AGREEMENT FAILURES",
            score
                .results
                .iter()
                .filter(|r| r.transpiled && !r.cross_shell_agree)
                .collect(),
        ),
        (
            "LINT FAILURES",
            score
                .results
                .iter()
                .filter(|r| r.transpiled && !r.lint_clean)
                .collect(),
        ),
    ];

    for (label, failures) in &failure_categories {
        if !failures.is_empty() {
            eprintln!("\n--- {} ---", label);
            for r in &failures[..failures.len().min(20)] {
                eprintln!("  {}", r.id);
            }
            if failures.len() > 20 {
                eprintln!("  ... and {} more", failures.len() - 20);
            }
        }
    }

    eprintln!("==========================================\n");
}

#[test]

include!("corpus_tests_corpus_037.rs");
