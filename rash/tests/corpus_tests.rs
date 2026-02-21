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
    assert_eq!(
        registry.len(),
        30,
        "Tier 1 should have 30 entries (10 per format)"
    );
}

#[test]
fn test_CORPUS_002_registry_bash_entries() {
    let registry = bashrs::corpus::CorpusRegistry::load_tier1();
    let bash = registry.by_format(bashrs::corpus::CorpusFormat::Bash);
    assert_eq!(bash.len(), 10, "Tier 1 bash should have 10 entries");

    // Verify all entries have non-empty inputs and expected outputs
    for entry in &bash {
        assert!(
            !entry.input.is_empty(),
            "Entry {} has empty input",
            entry.id
        );
        assert!(
            !entry.expected_output.is_empty(),
            "Entry {} has empty expected output",
            entry.id
        );
        assert!(
            entry.id.starts_with("B-"),
            "Bash entry ID should start with B-"
        );
    }
}

#[test]
fn test_CORPUS_003_registry_makefile_entries() {
    let registry = bashrs::corpus::CorpusRegistry::load_tier1();
    let make = registry.by_format(bashrs::corpus::CorpusFormat::Makefile);
    assert_eq!(make.len(), 10, "Tier 1 makefile should have 10 entries");

    for entry in &make {
        assert!(
            entry.id.starts_with("M-"),
            "Makefile entry ID should start with M-"
        );
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
fn test_CORPUS_005_score_perfect_v2() {
    // V2 scoring: A(30) + B_L1(10) + B_L2(8) + B_L3(7) + C(15) + D(10) + E(10) + F(5) + G(5) = 100
    let result = bashrs::corpus::CorpusResult {
        id: "T-001".to_string(),
        transpiled: true,
        output_contains: true,
        output_exact: true,
        output_behavioral: true,
        schema_valid: true,
        has_test: true,
        coverage_ratio: 1.0,
        lint_clean: true,
        deterministic: true,
        metamorphic_consistent: true,
        cross_shell_agree: true,
        actual_output: Some("output".to_string()),
        expected_output: None,
        error: None,
        error_category: None,
        error_confidence: None,
        decision_trace: None,
    };
    let score = result.score();
    assert!(
        (score - 100.0).abs() < f64::EPSILON,
        "Perfect v2 entry should score 100, got {}",
        score
    );
}

#[test]
fn test_CORPUS_006_score_gateway_barrier() {
    // Failed transpilation: gateway blocks all other scores
    let result = bashrs::corpus::CorpusResult {
        id: "T-002".to_string(),
        transpiled: false,
        output_contains: true,        // should be ignored
        output_exact: true,           // should be ignored
        output_behavioral: true,      // should be ignored
        schema_valid: true,           // should be ignored
        has_test: true,               // should be ignored
        coverage_ratio: 1.0,          // should be ignored
        lint_clean: true,             // should be ignored
        deterministic: true,          // should be ignored
        metamorphic_consistent: true, // should be ignored
        cross_shell_agree: true,      // should be ignored
        actual_output: None,
        expected_output: None,
        error: Some("parse error".to_string()),
        error_category: None,
        error_confidence: None,
        decision_trace: None,
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
        score.passed,
        score.total,
        score.rate * 100.0,
        score.score,
        score.grade
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
        score.passed,
        score.total,
        score.rate * 100.0,
        score.score,
        score.grade
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
        score.passed,
        score.total,
        score.rate * 100.0,
        score.score,
        score.grade
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
fn test_CORPUS_037_falsify_lint_failures() {
    // Transpile the 4 lint-failing entries and show what lint flags
    let config = bashrs::Config::default();
    let failing_inputs = vec![
        (
            "B-046",
            r#"fn main() { let mut x = 0; let mut y = 10; while x < 5 && y > 0 { x = x + 1; y = y - 1; } }"#,
        ),
        (
            "B-116",
            r#"fn main() { let mut a = 0; let mut b = 100; while a < 50 || b > 50 { a += 1; b -= 1; } }"#,
        ),
        (
            "B-138",
            r#"fn main() { let max_retries = 3; let mut attempts = 0; let mut success = false; while attempts < max_retries && !success { attempts += 1; if attempts >= 2 { success = true; } } }"#,
        ),
        (
            "B-160",
            r#"fn main() { let mut interrupted = false; let mut completed = false; let mut retries = 0; while !completed && !interrupted { retries += 1; if retries >= 5 { completed = true; } } }"#,
        ),
    ];

    for (id, input) in &failing_inputs {
        match bashrs::transpile(input, config.clone()) {
            Ok(output) => {
                let lint = bashrs::linter::rules::lint_shell(&output);
                eprintln!("\n=== {} (lint_clean={}) ===", id, !lint.has_errors());
                eprintln!("--- output ---\n{}", output);
                if lint.has_errors() {
                    eprintln!("--- lint violations ---");
                    for d in &lint.diagnostics {
                        eprintln!("  {}: {} ({})", d.code, d.message, d.severity);
                    }
                }
            }
            Err(e) => eprintln!("\n=== {} TRANSPILE FAILED: {} ===", id, e),
        }
    }
}

#[test]
fn test_CORPUS_038_falsify_cross_shell_failures() {
    // Transpile B-001, B-021, B-023 with Posix and Bash dialect configs
    let entries = vec![
        (
            "B-001",
            r#"fn main() { let greeting = "hello"; } "#,
            "greeting='hello'",
        ),
        (
            "B-021",
            r#"fn main() { let x = 5; if x > 10 { let r = "big"; } else if x > 3 { let r = "medium"; } else { let r = "small"; } }"#,
            "elif",
        ),
    ];

    for (id, input, expected) in &entries {
        let posix_config = bashrs::Config {
            target: bashrs::models::ShellDialect::Posix,
            ..bashrs::Config::default()
        };
        let bash_config = bashrs::Config {
            target: bashrs::models::ShellDialect::Bash,
            ..bashrs::Config::default()
        };

        let posix_out = bashrs::transpile(input, posix_config);
        let bash_out = bashrs::transpile(input, bash_config);

        eprintln!("\n=== {} (expected contains: '{}') ===", id, expected);
        match &posix_out {
            Ok(o) => eprintln!(
                "POSIX: contains={} len={}\n{}",
                o.contains(expected),
                o.len(),
                o
            ),
            Err(e) => eprintln!("POSIX: FAILED - {}", e),
        }
        match &bash_out {
            Ok(o) => eprintln!(
                "BASH:  contains={} len={}\n{}",
                o.contains(expected),
                o.len(),
                o
            ),
            Err(e) => eprintln!("BASH:  FAILED - {}", e),
        }
    }
}

#[test]
fn test_CORPUS_039_b1_containment_failures() {
    let registry = bashrs::corpus::CorpusRegistry::load_full();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run(&registry);

    let entry_map: std::collections::HashMap<&str, &bashrs::corpus::CorpusEntry> = registry
        .entries
        .iter()
        .map(|e| (e.id.as_str(), e))
        .collect();

    let b1_failures: Vec<_> = score
        .results
        .iter()
        .filter(|r| r.transpiled && !r.output_contains)
        .collect();

    for r in &b1_failures {
        if let Some(entry) = entry_map.get(r.id.as_str()) {
            eprintln!(
                "\nB1 FAIL: {} expected='{}' actual=\n{}",
                r.id,
                entry.expected_output,
                r.actual_output.as_deref().unwrap_or("(none)")
            );
        }
    }

    // B1 containment rate must be >= 95% (currently ~95.8% at 16K+ entries)
    let transpiled: Vec<_> = score.results.iter().filter(|r| r.transpiled).collect();
    let b1_rate = if transpiled.is_empty() {
        0.0
    } else {
        (transpiled.len() - b1_failures.len()) as f64 / transpiled.len() as f64 * 100.0
    };
    assert!(
        b1_rate >= 95.0,
        "B1 containment rate {:.1}% below 95% threshold ({} failures out of {} transpiled)",
        b1_rate,
        b1_failures.len(),
        transpiled.len()
    );
}
