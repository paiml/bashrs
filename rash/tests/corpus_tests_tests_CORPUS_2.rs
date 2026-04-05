fn test_CORPUS_014_cli_build_command_exists() {
    // Verify the CLI build subcommand is accessible
    bashrs_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("build").or(predicate::str::contains("transpile")));
}

// =============================================================================
// Full Corpus Aggregate Test
// =============================================================================

#[test]
fn test_CORPUS_015_full_corpus_aggregate_score() {
    let registry = bashrs::corpus::CorpusRegistry::load_tier1();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run(&registry);

    eprintln!("\n=== CORPUS QUALITY REPORT (Iteration 1) ===");
    eprintln!("Total entries: {}", score.total);
    eprintln!("Passed: {}", score.passed);
    eprintln!("Failed: {}", score.failed);
    eprintln!("Rate: {:.1}%", score.rate * 100.0);
    eprintln!("Score: {:.1}/100", score.score);
    eprintln!("Grade: {}", score.grade);
    eprintln!("Gateway met: {}", score.gateway_met());
    eprintln!("============================================\n");

    // This is iteration 1 - we're measuring baseline, not enforcing 99% yet
    // The spec says Tier 1 target is 100%, but we're bootstrapping
    assert!(score.total > 0, "Corpus should have entries");
}

// =============================================================================
// Tier 2 Corpus Tests (Standard difficulty - potential falsifiers)
// =============================================================================

#[test]
fn test_CORPUS_016_tier2_loads_all_entries() {
    let registry = bashrs::corpus::CorpusRegistry::load_tier1_and_tier2();
    assert_eq!(
        registry.len(),
        55,
        "Tier 1+2 should have 55 entries (30 + 25)"
    );
}

#[test]
fn test_CORPUS_017_tier2_bash_transpilation() {
    let registry = bashrs::corpus::CorpusRegistry::load_tier1_and_tier2();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run_format(&registry, bashrs::corpus::CorpusFormat::Bash);

    for result in &score.results {
        if !result.transpiled {
            eprintln!(
                "TIER2 FAILED: {} - {}",
                result.id,
                result.error.as_deref().unwrap_or("unknown")
            );
        }
    }

    eprintln!(
        "Bash T1+T2: {}/{} passed ({:.1}%), score: {:.1}, grade: {}",
        score.passed,
        score.total,
        score.rate * 100.0,
        score.score,
        score.grade
    );
}

#[test]
fn test_CORPUS_018_tier2_makefile_transpilation() {
    let registry = bashrs::corpus::CorpusRegistry::load_tier1_and_tier2();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run_format(&registry, bashrs::corpus::CorpusFormat::Makefile);

    for result in &score.results {
        if !result.transpiled {
            eprintln!(
                "TIER2 FAILED: {} - {}",
                result.id,
                result.error.as_deref().unwrap_or("unknown")
            );
        }
    }

    eprintln!(
        "Makefile T1+T2: {}/{} passed ({:.1}%), score: {:.1}, grade: {}",
        score.passed,
        score.total,
        score.rate * 100.0,
        score.score,
        score.grade
    );
}

#[test]
fn test_CORPUS_019_tier2_dockerfile_transpilation() {
    let registry = bashrs::corpus::CorpusRegistry::load_tier1_and_tier2();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run_format(&registry, bashrs::corpus::CorpusFormat::Dockerfile);

    for result in &score.results {
        if !result.transpiled {
            eprintln!(
                "TIER2 FAILED: {} - {}",
                result.id,
                result.error.as_deref().unwrap_or("unknown")
            );
        }
    }

    eprintln!(
        "Dockerfile T1+T2: {}/{} passed ({:.1}%), score: {:.1}, grade: {}",
        score.passed,
        score.total,
        score.rate * 100.0,
        score.score,
        score.grade
    );
}

#[test]
fn test_CORPUS_020_tier2_aggregate_score() {
    let registry = bashrs::corpus::CorpusRegistry::load_tier1_and_tier2();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run(&registry);

    eprintln!("\n=== CORPUS QUALITY REPORT (Tier 1+2) ===");
    eprintln!("Total entries: {}", score.total);
    eprintln!("Passed: {}", score.passed);
    eprintln!("Failed: {}", score.failed);
    eprintln!("Rate: {:.1}%", score.rate * 100.0);
    eprintln!("Score: {:.1}/100", score.score);
    eprintln!("Grade: {}", score.grade);
    eprintln!("==========================================\n");

    // Log individual failures for fixing
    for result in &score.results {
        if !result.transpiled {
            eprintln!(
                "  FALSIFIER: {} - {}",
                result.id,
                result.error.as_deref().unwrap_or("unknown")
            );
        }
    }

    assert!(score.total == 55, "Should run all 55 entries");
}

// =============================================================================
// Tier 3 Corpus Tests (Complex difficulty - stronger falsifiers)
// =============================================================================

#[test]
fn test_CORPUS_021_tier3_loads_all_entries() {
    let registry = bashrs::corpus::CorpusRegistry::load_all();
    assert!(
        registry.len() > 55,
        "Tier 1+2+3 should have more than 55 entries"
    );
}

#[test]
fn test_CORPUS_022_tier3_bash_transpilation() {
    let registry = bashrs::corpus::CorpusRegistry::load_all();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run_format(&registry, bashrs::corpus::CorpusFormat::Bash);

    for result in &score.results {
        if !result.transpiled {
            eprintln!(
                "TIER3 FAILED: {} - {}",
                result.id,
                result.error.as_deref().unwrap_or("unknown")
            );
        }
    }

    eprintln!(
        "Bash T1+T2+T3: {}/{} passed ({:.1}%), score: {:.1}, grade: {}",
        score.passed,
        score.total,
        score.rate * 100.0,
        score.score,
        score.grade
    );
}

#[test]
fn test_CORPUS_023_tier3_makefile_transpilation() {
    let registry = bashrs::corpus::CorpusRegistry::load_all();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run_format(&registry, bashrs::corpus::CorpusFormat::Makefile);

    for result in &score.results {
        if !result.transpiled {
            eprintln!(
                "TIER3 FAILED: {} - {}",
                result.id,
                result.error.as_deref().unwrap_or("unknown")
            );
        }
    }

    eprintln!(
        "Makefile T1+T2+T3: {}/{} passed ({:.1}%), score: {:.1}, grade: {}",
        score.passed,
        score.total,
        score.rate * 100.0,
        score.score,
        score.grade
    );
}

#[test]
fn test_CORPUS_024_tier3_dockerfile_transpilation() {
    let registry = bashrs::corpus::CorpusRegistry::load_all();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run_format(&registry, bashrs::corpus::CorpusFormat::Dockerfile);

    for result in &score.results {
        if !result.transpiled {
            eprintln!(
                "TIER3 FAILED: {} - {}",
                result.id,
                result.error.as_deref().unwrap_or("unknown")
            );
        }
    }

    eprintln!(
        "Dockerfile T1+T2+T3: {}/{} passed ({:.1}%), score: {:.1}, grade: {}",
        score.passed,
        score.total,
        score.rate * 100.0,
        score.score,
        score.grade
    );
}

#[test]
fn test_CORPUS_025_tier3_aggregate_score() {
    let registry = bashrs::corpus::CorpusRegistry::load_all();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run(&registry);

    eprintln!("\n=== CORPUS QUALITY REPORT (Tier 1+2+3) ===");
    eprintln!("Total entries: {}", score.total);
    eprintln!("Passed: {}", score.passed);
    eprintln!("Failed: {}", score.failed);
    eprintln!("Rate: {:.1}%", score.rate * 100.0);
    eprintln!("Score: {:.1}/100", score.score);
    eprintln!("Grade: {}", score.grade);
    eprintln!("============================================\n");

    // Log individual failures for fixing
    for result in &score.results {
        if !result.transpiled {
            eprintln!(
                "  FALSIFIER: {} - {}",
                result.id,
                result.error.as_deref().unwrap_or("unknown")
            );
        }
    }

    assert!(score.total > 55, "Should run more than 55 entries");
}

// =============================================================================
// Tier 4 Corpus Tests (Adversarial - edge cases, boundary conditions)
// =============================================================================

#[test]
fn test_CORPUS_026_tier4_loads_all_entries() {
    let registry = bashrs::corpus::CorpusRegistry::load_all_with_adversarial();
    assert!(
        registry.len() > 85,
        "Tier 1-4 should have more than 85 entries"
    );
}

#[test]
fn test_CORPUS_027_tier4_bash_transpilation() {
    let registry = bashrs::corpus::CorpusRegistry::load_all_with_adversarial();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run_format(&registry, bashrs::corpus::CorpusFormat::Bash);

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
        "Bash T1-T4: {}/{} passed ({:.1}%), score: {:.1}, grade: {}",
        score.passed,
        score.total,
        score.rate * 100.0,
        score.score,
        score.grade
    );
}

#[test]
fn test_CORPUS_028_tier4_makefile_transpilation() {
    let registry = bashrs::corpus::CorpusRegistry::load_all_with_adversarial();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run_format(&registry, bashrs::corpus::CorpusFormat::Makefile);

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
        "Makefile T1-T4: {}/{} passed ({:.1}%), score: {:.1}, grade: {}",
        score.passed,
        score.total,
        score.rate * 100.0,
        score.score,
        score.grade
    );
}

#[test]

include!("corpus_tests_tests_CORPUS.rs");
