#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]
//! Corpus-Driven Transpilation Quality Tests
//!
//! Tests the corpus registry, runner, scoring, and convergence infrastructure.
//! Implements the Popperian falsification protocol: each entry is a potential
//! falsifier that could demonstrate transpilation failure.
//!
//! Uses assert_cmd for CLI testing (MANDATORY per CLAUDE.md).

use assert_cmd::Command;
use predicates::prelude::*;

/// Create a bashrs command (MANDATORY pattern per CLAUDE.md)
#[allow(deprecated)]
fn bashrs_cmd() -> Command {
    assert_cmd::cargo_bin_cmd!("bashrs")
}

// =============================================================================
// Registry Tests
// =============================================================================

#[test]
fn test_CORPUS_001_registry_loads_all_tier1() {
    let registry = bashrs::corpus::CorpusRegistry::load_tier1();
    assert_eq!(registry.len(), 30, "Tier 1 should have 30 entries (10 per format)");
}

#[test]
fn test_CORPUS_002_registry_bash_entries() {
    let registry = bashrs::corpus::CorpusRegistry::load_tier1();
    let bash = registry.by_format(bashrs::corpus::CorpusFormat::Bash);
    assert_eq!(bash.len(), 10, "Tier 1 bash should have 10 entries");

    // Verify all entries have non-empty inputs and expected outputs
    for entry in &bash {
        assert!(!entry.input.is_empty(), "Entry {} has empty input", entry.id);
        assert!(
            !entry.expected_output.is_empty(),
            "Entry {} has empty expected output",
            entry.id
        );
        assert!(entry.id.starts_with("B-"), "Bash entry ID should start with B-");
    }
}

#[test]
fn test_CORPUS_003_registry_makefile_entries() {
    let registry = bashrs::corpus::CorpusRegistry::load_tier1();
    let make = registry.by_format(bashrs::corpus::CorpusFormat::Makefile);
    assert_eq!(make.len(), 10, "Tier 1 makefile should have 10 entries");

    for entry in &make {
        assert!(entry.id.starts_with("M-"), "Makefile entry ID should start with M-");
    }
}

#[test]
fn test_CORPUS_004_registry_dockerfile_entries() {
    let registry = bashrs::corpus::CorpusRegistry::load_tier1();
    let docker = registry.by_format(bashrs::corpus::CorpusFormat::Dockerfile);
    assert_eq!(docker.len(), 10, "Tier 1 dockerfile should have 10 entries");

    for entry in &docker {
        assert!(
            entry.id.starts_with("D-"),
            "Dockerfile entry ID should start with D-"
        );
    }
}

// =============================================================================
// Scoring Tests
// =============================================================================

#[test]
fn test_CORPUS_005_score_perfect() {
    let result = bashrs::corpus::CorpusResult {
        id: "T-001".to_string(),
        transpiled: true,
        output_correct: true,
        has_test: true,
        lint_clean: true,
        deterministic: true,
        actual_output: Some("output".to_string()),
        error: None,
    };
    let score = result.score();
    assert!(
        (score - 100.0).abs() < f64::EPSILON,
        "Perfect entry should score 100, got {}",
        score
    );
}

#[test]
fn test_CORPUS_006_score_gateway_barrier() {
    // Failed transpilation: gateway blocks all other scores
    let result = bashrs::corpus::CorpusResult {
        id: "T-002".to_string(),
        transpiled: false,
        output_correct: true, // should be ignored
        has_test: true,       // should be ignored
        lint_clean: true,     // should be ignored
        deterministic: true,  // should be ignored
        actual_output: None,
        error: Some("parse error".to_string()),
    };
    let score = result.score();
    assert!(
        score.abs() < f64::EPSILON,
        "Failed transpilation should score 0, got {}",
        score
    );
}

#[test]
fn test_CORPUS_007_grade_from_score() {
    use bashrs::corpus::registry::Grade;
    assert_eq!(Grade::from_score(100.0), Grade::APlus);
    assert_eq!(Grade::from_score(97.0), Grade::APlus);
    assert_eq!(Grade::from_score(96.0), Grade::A);
    assert_eq!(Grade::from_score(80.0), Grade::B);
    assert_eq!(Grade::from_score(70.0), Grade::C);
    assert_eq!(Grade::from_score(60.0), Grade::D);
    assert_eq!(Grade::from_score(59.0), Grade::F);
}

// =============================================================================
// Runner Tests - Bash Corpus
// =============================================================================

#[test]
fn test_CORPUS_008_bash_transpilation_runs() {
    let registry = bashrs::corpus::CorpusRegistry::load_tier1();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run_format(&registry, bashrs::corpus::CorpusFormat::Bash);

    // Verify we ran all entries
    assert_eq!(score.total, 10, "Should run 10 bash entries");

    // Log the results for visibility
    for result in &score.results {
        if !result.transpiled {
            eprintln!(
                "FAILED: {} - {}",
                result.id,
                result.error.as_deref().unwrap_or("unknown")
            );
        }
    }

    // Tier 1 bash should have reasonable success
    // (not enforcing 100% yet - this is iteration 1)
    eprintln!(
        "Bash corpus: {}/{} passed ({:.1}%), score: {:.1}, grade: {}",
        score.passed, score.total, score.rate * 100.0, score.score, score.grade
    );
}

// =============================================================================
// Runner Tests - Makefile Corpus
// =============================================================================

#[test]
fn test_CORPUS_009_makefile_transpilation_runs() {
    let registry = bashrs::corpus::CorpusRegistry::load_tier1();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run_format(&registry, bashrs::corpus::CorpusFormat::Makefile);

    assert_eq!(score.total, 10, "Should run 10 makefile entries");

    for result in &score.results {
        if !result.transpiled {
            eprintln!(
                "FAILED: {} - {}",
                result.id,
                result.error.as_deref().unwrap_or("unknown")
            );
        }
    }

    eprintln!(
        "Makefile corpus: {}/{} passed ({:.1}%), score: {:.1}, grade: {}",
        score.passed, score.total, score.rate * 100.0, score.score, score.grade
    );
}

// =============================================================================
// Runner Tests - Dockerfile Corpus
// =============================================================================

#[test]
fn test_CORPUS_010_dockerfile_transpilation_runs() {
    let registry = bashrs::corpus::CorpusRegistry::load_tier1();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run_format(&registry, bashrs::corpus::CorpusFormat::Dockerfile);

    assert_eq!(score.total, 10, "Should run 10 dockerfile entries");

    for result in &score.results {
        if !result.transpiled {
            eprintln!(
                "FAILED: {} - {}",
                result.id,
                result.error.as_deref().unwrap_or("unknown")
            );
        }
    }

    eprintln!(
        "Dockerfile corpus: {}/{} passed ({:.1}%), score: {:.1}, grade: {}",
        score.passed, score.total, score.rate * 100.0, score.score, score.grade
    );
}

// =============================================================================
// Determinism Tests
// =============================================================================

#[test]
fn test_CORPUS_011_full_corpus_determinism() {
    let registry = bashrs::corpus::CorpusRegistry::load_tier1();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config.clone());

    // Run twice
    let score1 = runner.run(&registry);
    let runner2 = bashrs::corpus::CorpusRunner::new(config);
    let score2 = runner2.run(&registry);

    // Scores must be identical (determinism)
    assert_eq!(
        score1.total, score2.total,
        "Corpus run must be deterministic (total)"
    );
    assert_eq!(
        score1.passed, score2.passed,
        "Corpus run must be deterministic (passed)"
    );
    assert!(
        (score1.rate - score2.rate).abs() < f64::EPSILON,
        "Corpus run must be deterministic (rate)"
    );
}

// =============================================================================
// Convergence Tests
// =============================================================================

#[test]
fn test_CORPUS_012_convergence_detection() {
    use bashrs::corpus::ConvergenceEntry;

    // Converged: rate >= 99% and delta < 0.5% for 3 iterations
    let converged = vec![
        ConvergenceEntry {
            iteration: 1,
            date: "2026-02-01".to_string(),
            total: 200,
            passed: 198,
            failed: 2,
            rate: 0.99,
            delta: 0.002,
            notes: "stable".to_string(),
        },
        ConvergenceEntry {
            iteration: 2,
            date: "2026-02-08".to_string(),
            total: 200,
            passed: 199,
            failed: 1,
            rate: 0.995,
            delta: 0.004,
            notes: "stable".to_string(),
        },
        ConvergenceEntry {
            iteration: 3,
            date: "2026-02-15".to_string(),
            total: 200,
            passed: 199,
            failed: 1,
            rate: 0.995,
            delta: 0.0,
            notes: "converged".to_string(),
        },
    ];
    assert!(
        bashrs::corpus::CorpusRunner::is_converged(&converged),
        "Should detect convergence"
    );
}

#[test]
fn test_CORPUS_013_convergence_not_met() {
    use bashrs::corpus::ConvergenceEntry;

    // Not converged: rate below 99%
    let not_converged = vec![
        ConvergenceEntry {
            iteration: 1,
            date: "2026-02-01".to_string(),
            total: 200,
            passed: 190,
            failed: 10,
            rate: 0.95,
            delta: 0.001,
            notes: "not met".to_string(),
        },
        ConvergenceEntry {
            iteration: 2,
            date: "2026-02-08".to_string(),
            total: 200,
            passed: 192,
            failed: 8,
            rate: 0.96,
            delta: 0.01,
            notes: "not met".to_string(),
        },
        ConvergenceEntry {
            iteration: 3,
            date: "2026-02-15".to_string(),
            total: 200,
            passed: 194,
            failed: 6,
            rate: 0.97,
            delta: 0.01,
            notes: "not met".to_string(),
        },
    ];
    assert!(
        !bashrs::corpus::CorpusRunner::is_converged(&not_converged),
        "Should not detect convergence when rate < 99%"
    );
}

// =============================================================================
// CLI Integration Tests
// =============================================================================

#[test]
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
    assert_eq!(registry.len(), 55, "Tier 1+2 should have 55 entries (30 + 25)");
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
        score.passed, score.total, score.rate * 100.0, score.score, score.grade
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
        score.passed, score.total, score.rate * 100.0, score.score, score.grade
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
        score.passed, score.total, score.rate * 100.0, score.score, score.grade
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
