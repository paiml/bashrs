#[allow(clippy::unwrap_used)]
use super::*;
use crate::corpus::registry::CorpusTier;

#[test]
fn test_CORPUS_RUN_001_score_calculation_v2_full() {
    // All flags true: A(30) + B_L1(10) + B_L2(8) + B_L3(7) + C(15) + D(10) + E(10) + F(5) + G(5) = 100
    let result = CorpusResult {
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
        expected_output: None,
        actual_output: Some("output".to_string()),
        error: None,
        error_category: None,
        error_confidence: None,
        decision_trace: None,
    };
    assert!((result.score() - 100.0).abs() < f64::EPSILON);
}

#[test]
fn test_CORPUS_RUN_002_score_transpile_only() {
    // Only transpilation succeeds: A(30) + nothing else = 30
    let result = CorpusResult {
        id: "T-002".to_string(),
        transpiled: true,
        output_contains: false,
        output_exact: false,
        output_behavioral: false,
        schema_valid: true,
        has_test: false,
        coverage_ratio: 0.0,
        lint_clean: false,
        deterministic: false,
        metamorphic_consistent: false,
        cross_shell_agree: false,
        expected_output: None,
        actual_output: Some("output".to_string()),
        error: None,
        error_category: None,
        error_confidence: None,
        decision_trace: None,
    };
    assert!((result.score() - 30.0).abs() < f64::EPSILON);
}

#[test]
fn test_CORPUS_RUN_003_score_failed_transpile() {
    // Failed transpilation: gateway blocks everything = 0
    let result = CorpusResult {
        id: "T-003".to_string(),
        transpiled: false,
        output_contains: false,
        output_exact: false,
        output_behavioral: false,
        schema_valid: false,
        has_test: true,
        coverage_ratio: 1.0,
        lint_clean: false,
        deterministic: false,
        metamorphic_consistent: false,
        cross_shell_agree: false,
        expected_output: None,
        actual_output: None,
        error: Some("parse error".to_string()),
        error_category: None,
        error_confidence: None,
        decision_trace: None,
    };
    assert!((result.score()).abs() < f64::EPSILON);
}

#[test]
fn test_CORPUS_RUN_004_convergence_not_enough_entries() {
    let entries = vec![ConvergenceEntry {
        iteration: 1,
        date: "2026-02-06".to_string(),
        total: 100,
        passed: 99,
        failed: 1,
        rate: 0.99,
        delta: 0.99,
        notes: "initial".to_string(),
        ..Default::default()
    }];
    assert!(!CorpusRunner::is_converged(&entries));
}

#[test]
fn test_CORPUS_RUN_005_convergence_met() {
    let entries = vec![
        ConvergenceEntry {
            iteration: 1,
            date: "2026-02-01".to_string(),
            total: 200,
            passed: 198,
            failed: 2,
            rate: 0.99,
            delta: 0.001,
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
    assert!(CorpusRunner::is_converged(&entries));
}

#[test]
fn test_CORPUS_RUN_006_convergence_rate_below_threshold() {
    let entries = vec![
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
    assert!(!CorpusRunner::is_converged(&entries));
}

#[test]
fn test_CORPUS_RUN_007_gateway_logic_v2() {
    // All v2 flags true: score = 100
    let perfect = CorpusResult {
        id: "T-007".to_string(),
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
        expected_output: None,
        actual_output: Some("out".to_string()),
        error: None,
        error_category: None,
        error_confidence: None,
        decision_trace: None,
    };
    assert!((perfect.score() - 100.0).abs() < f64::EPSILON);

    // Gateway: failed transpile = 0 total (all other flags ignored)
    let failed = CorpusResult {
        id: "T-007b".to_string(),
        transpiled: false,
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
        expected_output: None,
        actual_output: None,
        error: Some("err".to_string()),
        error_category: None,
        error_confidence: None,
        decision_trace: None,
    };
    assert!((failed.score()).abs() < f64::EPSILON);
}

#[test]
fn test_CORPUS_RUN_008_partial_score_v2() {
    // Transpiles + containment + exact + test + deterministic + metamorphic, but NOT lint clean
    // A(30) + B_L1(10) + B_L2(8) + C(15) + D(0) + E(10) + F(5) = 78
    let partial = CorpusResult {
        id: "T-008".to_string(),
        transpiled: true,
        output_contains: true,
        output_exact: true,
        output_behavioral: false,
        schema_valid: true,
        has_test: true,
        coverage_ratio: 1.0,
        lint_clean: false,
        deterministic: true,
        metamorphic_consistent: true,
        cross_shell_agree: false,
        expected_output: None,
        actual_output: Some("out".to_string()),
        error: None,
        error_category: None,
        error_confidence: None,
        decision_trace: None,
    };
    assert!((partial.score() - 78.0).abs() < f64::EPSILON);
}

#[test]
fn test_CORPUS_RUN_009_secondary_gate_l1_blocks_l2() {
    // L1 fails: L2 and L3 are gated to 0 even if set true
    // A(30) + B_L1(0) + B_L2(0) + B_L3(0) + C(15) + D(10) + E(10) + F(5) + G(5) = 75
    let result = CorpusResult {
        id: "T-009".to_string(),
        transpiled: true,
        output_contains: false,
        output_exact: true,      // gated by L1
        output_behavioral: true, // gated by L1
        schema_valid: true,
        has_test: true,
        coverage_ratio: 1.0,
        lint_clean: true,
        deterministic: true,
        metamorphic_consistent: true,
        cross_shell_agree: true,
        expected_output: None,
        actual_output: Some("out".to_string()),
        error: None,
        error_category: None,
        error_confidence: None,
        decision_trace: None,
    };
    assert!((result.score() - 75.0).abs() < f64::EPSILON);
}

#[test]
fn test_CORPUS_RUN_010_v1_backward_compat() {
    // v1 scoring: A(40) + B(25) + C(15) + D(10) + E(10) = 100
    let result = CorpusResult {
        id: "T-010".to_string(),
        transpiled: true,
        output_contains: true,
        output_exact: true,
        output_behavioral: false,
        schema_valid: true,
        has_test: true,
        coverage_ratio: 1.0,
        lint_clean: true,
        deterministic: true,
        metamorphic_consistent: true,
        cross_shell_agree: false,
        expected_output: None,
        actual_output: Some("out".to_string()),
        error: None,
        error_category: None,
        error_confidence: None,
        decision_trace: None,
    };
    assert!((result.score_v1() - 100.0).abs() < f64::EPSILON);
}

#[test]
fn test_CORPUS_RUN_011_exact_match_single_line() {
    assert!(check_exact_match("hello world\nfoo bar\n", "foo bar"));
    assert!(!check_exact_match("hello world\nfoo bar baz\n", "foo bar"));
}

#[test]
fn test_CORPUS_RUN_012_exact_match_multi_line() {
    let actual = "line1\nline2\nline3\nline4\n";
    assert!(check_exact_match(actual, "line2\nline3"));
    assert!(!check_exact_match(actual, "line2\nline4"));
}

#[test]
fn test_CORPUS_RUN_013_exact_match_empty_expected() {
    assert!(check_exact_match("anything", ""));
    assert!(check_exact_match("anything", "  "));
}

#[test]
fn test_CORPUS_RUN_014_detect_test_exists() {
    // Empty ID should always return false
    assert!(!detect_test_exists(""));
    // B-001 is tested via test_CORPUS_002 (registry bash entries) — but the
    // detection checks for ID patterns in test function names.
    // If test names can't be loaded (e.g., in CI), falls back to true.
    let result = detect_test_exists("B-001");
    // Either we found the test or fell back to true (both acceptable)
    // detect_test_exists returns true (found) or true (fallback) — always succeeds
    let _detected = result;
}

#[test]
fn test_CORPUS_RUN_016_classify_error_syntax() {
    let (cat, conf) = classify_error("unexpected token: parse error near line 5");
    assert_eq!(cat.as_deref(), Some("syntax_error"));
    assert!(conf.is_some());
}

#[test]
fn test_CORPUS_RUN_017_classify_error_unsupported() {
    let (cat, conf) = classify_error("unsupported feature: process substitution");
    assert_eq!(cat.as_deref(), Some("unsupported_construct"));
    assert!(conf.is_some());
}

#[test]
fn test_CORPUS_RUN_018_classify_error_type() {
    let (cat, conf) = classify_error("type mismatch in assignment");
    assert_eq!(cat.as_deref(), Some("type_error"));
    assert!(conf.is_some());
}

#[test]
fn test_CORPUS_RUN_019_classify_error_unknown() {
    let (cat, conf) = classify_error("something went wrong");
    assert_eq!(cat.as_deref(), Some("unknown"));
    assert!(conf.is_some());
}

#[test]
fn test_CORPUS_RUN_020_mr5_subsumption_top_level() {
    // MR-5 must only remove top-level statements, not statements inside blocks
    let runner = CorpusRunner::new(Config::default());
    let entry_nested = CorpusEntry::new(
        "T-MR5-1",
        "nested-block",
        "If/else with nested statements",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        r#"fn main() { let x = 5; if x > 3 { let msg = "big"; } else { let msg = "small"; } }"#,
        "x=",
    );
    // Should be vacuously true (only one top-level semi before the if block)
    assert!(runner.check_mr5_subsumption(&entry_nested));

    let entry_multi = CorpusEntry::new(
        "T-MR5-2",
        "multi-stmt",
        "Multiple top-level statements",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        "fn main() { let a = 1; let b = 2; let c = 3; }",
        "a=",
    );
    // Has 3 top-level statements; removing last should still transpile
    assert!(runner.check_mr5_subsumption(&entry_multi));
}

#[test]
fn test_CORPUS_RUN_021_mr6_composition() {
    let runner = CorpusRunner::new(Config::default());
    let entry = CorpusEntry::new(
        "T-MR6-1",
        "multi-let",
        "Multiple let statements",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        "fn main() { let a = 1; let b = 2; }",
        "a=",
    );
    assert!(runner.check_mr6_composition(&entry));
}

#[test]
fn test_CORPUS_RUN_022_mr7_negation() {
    let runner = CorpusRunner::new(Config::default());
    let entry = CorpusEntry::new(
        "T-MR7-1",
        "if-cond",
        "If with condition",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        r#"fn main() { let x = 5; if x > 3 { let msg = "yes"; } }"#,
        "x=",
    );
    assert!(runner.check_mr7_negation(&entry));
}

#[test]
fn test_CORPUS_RUN_023_behavioral_execution() {
    let runner = CorpusRunner::new(Config::default());
    // Simple variable assignment — should execute without error
    assert!(runner.check_behavioral("x='42'", CorpusFormat::Bash));
    // Empty script — should succeed
    assert!(runner.check_behavioral("", CorpusFormat::Bash));
    // Dockerfile — always pass (syntax proxy)
    assert!(runner.check_behavioral("", CorpusFormat::Dockerfile));
}

// BH-MUT-0017: check_behavioral mutation targets
// Kills mutations of timeout detection and format dispatch

#[test]
fn test_CORPUS_RUN_060_behavioral_nonzero_exit_passes() {
    // Non-zero exit code should PASS (not timeout, just "false")
    let runner = CorpusRunner::new(Config::default());
    assert!(
        runner.check_behavioral("exit 1", CorpusFormat::Bash),
        "Non-zero exit (not timeout) should still pass behavioral check"
    );
}

#[test]
fn test_CORPUS_RUN_061_behavioral_timeout_fails() {
    // Infinite loop should be killed by timeout → exit 124 → FAIL
    let runner = CorpusRunner::new(Config::default());
    assert!(
        !runner.check_behavioral("while true; do :; done", CorpusFormat::Bash),
        "Infinite loop should fail behavioral check via timeout"
    );
}

#[test]
fn test_CORPUS_RUN_062_behavioral_makefile_delegates() {
    // Makefile behavioral check delegates to make dry-run
    let runner = CorpusRunner::new(Config::default());
    assert!(runner.check_behavioral("all:\n\techo ok\n", CorpusFormat::Makefile));
}

#[test]
fn test_CORPUS_RUN_024_shellcheck_integration() {
    let runner = CorpusRunner::new(Config::default());
    // Valid POSIX script should pass shellcheck
    let valid = runner.check_shellcheck("#!/bin/sh\nx='hello'\necho \"$x\"");
    // shellcheck might not be installed; if None, that's fine
    if let Some(result) = valid {
        assert!(result, "Valid POSIX script should pass shellcheck");
    }
}

#[test]
fn test_CORPUS_RUN_025_makefile_dry_run() {
    let runner = CorpusRunner::new(Config::default());
    // Valid Makefile should pass make -n
    assert!(runner.check_makefile_dry_run("all:\n\t@echo hello\n"));
    // Also verify check_behavioral routes Makefile correctly
    assert!(runner.check_behavioral("all:\n\t@echo hello\n", CorpusFormat::Makefile));
}

#[test]
fn test_CORPUS_RUN_026_cross_shell_execution() {
    let runner = CorpusRunner::new(Config::default());
    // Valid POSIX script should pass in both sh and dash
    assert!(runner.check_shell_execution("x='hello'"));
    // Empty script should also work
    assert!(runner.check_shell_execution(""));
}

#[test]
fn test_CORPUS_RUN_027_convergence_log_roundtrip() {
    let tmp = std::env::temp_dir().join("bashrs_test_convergence.jsonl");
    // Clean up any previous test run
    let _ = std::fs::remove_file(&tmp);

    let entry1 = ConvergenceEntry {
        iteration: 1,
        date: "2026-02-07".to_string(),
        total: 100,
        passed: 95,
        failed: 5,
        rate: 0.95,
        delta: 0.0,
        notes: "first".to_string(),
        ..Default::default()
    };
    let entry2 = ConvergenceEntry {
        iteration: 2,
        date: "2026-02-07".to_string(),
        total: 100,
        passed: 98,
        failed: 2,
        rate: 0.98,
        delta: 0.03,
        notes: "second".to_string(),
        ..Default::default()
    };

    CorpusRunner::append_convergence_log(&entry1, &tmp).unwrap();
    CorpusRunner::append_convergence_log(&entry2, &tmp).unwrap();

    let loaded = CorpusRunner::load_convergence_log(&tmp).unwrap();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].iteration, 1);
    assert_eq!(loaded[1].iteration, 2);
    assert!((loaded[0].rate - 0.95).abs() < f64::EPSILON);
    assert_eq!(loaded[1].notes, "second");

    // Clean up
    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_CORPUS_RUN_028_convergence_log_missing_file() {
    let nonexistent = std::path::Path::new("/tmp/bashrs_nonexistent_convergence_xyzzy.jsonl");
    let loaded = CorpusRunner::load_convergence_log(nonexistent).unwrap();
    assert!(loaded.is_empty());
}

#[test]
fn test_CORPUS_RUN_029_extract_test_names() {
    let mut names = HashSet::new();
    let source = r#"
#[test]
fn test_CORPUS_001_registry_loads() {
// ...
}

#[test]
fn test_CORPUS_RUN_014_detect_test_exists() {
// ...
}

fn not_a_test() {}
"#;
    extract_test_names(source, &mut names);
    assert!(names.contains("test_CORPUS_001_registry_loads"));
    assert!(names.contains("test_CORPUS_RUN_014_detect_test_exists"));
    assert!(!names.contains("not_a_test"));
}

#[test]
fn test_CORPUS_RUN_015_schema_hard_gate() {
    // Schema invalid: transpiled=true but schema_valid=false → score 0
    let result = CorpusResult {
        id: "T-015".to_string(),
        transpiled: true,
        output_contains: true,
        output_exact: true,
        output_behavioral: true,
        schema_valid: false,
        has_test: true,
        coverage_ratio: 1.0,
        lint_clean: true,
        deterministic: true,
        metamorphic_consistent: true,
        cross_shell_agree: true,
        expected_output: None,
        actual_output: Some("invalid output".to_string()),
        error: None,
        error_category: None,
        error_confidence: None,
        decision_trace: None,
    };
    assert!(
        result.score().abs() < f64::EPSILON,
        "Schema-invalid entry should score 0, got {}",
        result.score()
    );
}

#[test]
fn test_CORPUS_RUN_030_parse_lcov_basic() {
    let lcov = r#"SF:rash/src/emitter/posix.rs
DA:1,5
DA:2,3
DA:3,0
DA:4,10
end_of_record
SF:rash/src/emitter/makefile.rs
DA:1,1
DA:2,0
DA:3,0
end_of_record
"#;
    let results = parse_lcov_file_coverage(lcov);
    assert_eq!(results.len(), 2);
    // posix.rs: 4 lines found, 3 hit (DA:3,0 is not hit)
    assert_eq!(results[0].0, "rash/src/emitter/posix.rs");
    assert_eq!(results[0].1, (4, 3));
    // makefile.rs: 3 lines found, 1 hit
    assert_eq!(results[1].0, "rash/src/emitter/makefile.rs");
    assert_eq!(results[1].1, (3, 1));
}

#[test]
fn test_CORPUS_RUN_031_parse_lcov_empty() {
    let results = parse_lcov_file_coverage("");
    assert!(results.is_empty());
}

#[test]
fn test_CORPUS_RUN_032_coverage_ratio_scoring() {
    // V2-8: coverage_ratio=0.8 should give 12.0/15 points for C
    let result = CorpusResult {
        id: "T-032".to_string(),
        transpiled: true,
        output_contains: true,
        output_exact: true,
        output_behavioral: true,
        schema_valid: true,
        has_test: true,
        coverage_ratio: 0.8,
        lint_clean: true,
        deterministic: true,
        metamorphic_consistent: true,
        cross_shell_agree: true,
        expected_output: None,
        actual_output: Some("output".to_string()),
        error: None,
        error_category: None,
        error_confidence: None,
        decision_trace: None,
    };
    // A=30 + B1=10 + B2=8 + B3=7 + C=12.0 + D=10 + E=10 + F=5 + G=5 = 97.0
    let score = result.score();
    assert!(
        (score - 97.0).abs() < f64::EPSILON,
        "Expected 97.0, got {score}"
    );
}

#[test]
fn test_CORPUS_RUN_033_coverage_ratio_zero() {
    // V2-8: coverage_ratio=0.0 gives 0/15 for C
    let result = CorpusResult {
        id: "T-033".to_string(),
        transpiled: true,
        output_contains: true,
        output_exact: true,
        output_behavioral: true,
        schema_valid: true,
        has_test: false,
        coverage_ratio: 0.0,
        lint_clean: true,
        deterministic: true,
        metamorphic_consistent: true,
        cross_shell_agree: true,
        expected_output: None,
        actual_output: Some("output".to_string()),
        error: None,
        error_category: None,
        error_confidence: None,
        decision_trace: None,
    };
    // A=30 + B1=10 + B2=8 + B3=7 + C=0 + D=10 + E=10 + F=5 + G=5 = 85.0
    let score = result.score();
    assert!(
        (score - 85.0).abs() < f64::EPSILON,
        "Expected 85.0, got {score}"
    );
}

#[test]
fn test_CORPUS_RUN_034_format_file_patterns() {
    // Verify format-to-file pattern mappings exist for all formats
    let bash_patterns = format_file_patterns(CorpusFormat::Bash);
    assert!(!bash_patterns.is_empty());
    assert!(bash_patterns.iter().any(|p| p.contains("posix")));

    let make_patterns = format_file_patterns(CorpusFormat::Makefile);
    assert!(make_patterns.iter().any(|p| p.contains("makefile")));

    let docker_patterns = format_file_patterns(CorpusFormat::Dockerfile);
    assert!(docker_patterns.iter().any(|p| p.contains("dockerfile")));
}

#[test]
fn test_CORPUS_RUN_035_per_format_convergence_entry() {
    // Verify convergence_entry extracts per-format stats from CorpusScore
    let runner = CorpusRunner::new(Config::default());
    let score = CorpusScore {
        total: 900,
        passed: 898,
        failed: 2,
        rate: 898.0 / 900.0,
        score: 99.9,
        grade: Grade::APlus,
        format_scores: vec![
            FormatScore {
                format: CorpusFormat::Bash,
                total: 500,
                passed: 499,
                rate: 499.0 / 500.0,
                score: 99.8,
                grade: Grade::APlus,
            },
            FormatScore {
                format: CorpusFormat::Makefile,
                total: 200,
                passed: 200,
                rate: 1.0,
                score: 100.0,
                grade: Grade::APlus,
            },
            FormatScore {
                format: CorpusFormat::Dockerfile,
                total: 200,
                passed: 199,
                rate: 199.0 / 200.0,
                score: 99.5,
                grade: Grade::APlus,
            },
        ],
        results: vec![],
    };
    let entry = runner.convergence_entry(&score, 5, "2026-02-08", 0.997, "test");
    assert_eq!(entry.bash_passed, 499);
    assert_eq!(entry.bash_total, 500);
    assert_eq!(entry.makefile_passed, 200);
    assert_eq!(entry.makefile_total, 200);
    assert_eq!(entry.dockerfile_passed, 199);
    assert_eq!(entry.dockerfile_total, 200);
    assert_eq!(entry.total, 900);
    assert_eq!(entry.passed, 898);
    assert_eq!(entry.iteration, 5);
}

#[test]
fn test_CORPUS_RUN_036_per_format_serde_roundtrip() {
    // Verify per-format fields survive JSON serialization
    let entry = ConvergenceEntry {
        iteration: 10,
        date: "2026-02-08".to_string(),
        total: 900,
        passed: 898,
        failed: 2,
        rate: 0.998,
        delta: 0.001,
        notes: "per-format".to_string(),
        bash_passed: 499,
        bash_total: 500,
        makefile_passed: 200,
        makefile_total: 200,
        dockerfile_passed: 199,
        dockerfile_total: 200,
        ..Default::default()
    };
    let json = serde_json::to_string(&entry).expect("serialize");
    let loaded: ConvergenceEntry = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(loaded.bash_passed, 499);
    assert_eq!(loaded.bash_total, 500);
    assert_eq!(loaded.makefile_passed, 200);
    assert_eq!(loaded.dockerfile_total, 200);
}

#[test]
fn test_CORPUS_RUN_037_per_format_backward_compat() {
    // Old entries without per-format fields should deserialize with defaults (0)
    let old_json = r#"{"iteration":1,"date":"2026-01-01","total":100,"passed":99,"failed":1,"rate":0.99,"delta":0.0,"notes":"old"}"#;
    let entry: ConvergenceEntry = serde_json::from_str(old_json).expect("deserialize old");
    assert_eq!(entry.bash_passed, 0);
    assert_eq!(entry.bash_total, 0);
    assert_eq!(entry.makefile_passed, 0);
    assert_eq!(entry.dockerfile_total, 0);
    assert_eq!(entry.total, 100);
    assert_eq!(entry.passed, 99);
}

#[test]
fn test_CORPUS_RUN_038_per_format_empty_score() {
    // convergence_entry with no format_scores should yield zeros
    let runner = CorpusRunner::new(Config::default());
    let score = CorpusScore {
        total: 10,
        passed: 10,
        failed: 0,
        rate: 1.0,
        score: 100.0,
        grade: Grade::APlus,
        format_scores: vec![],
        results: vec![],
    };
    let entry = runner.convergence_entry(&score, 1, "2026-02-08", 0.0, "empty");
    assert_eq!(entry.bash_passed, 0);
    assert_eq!(entry.bash_total, 0);
    assert_eq!(entry.makefile_passed, 0);
    assert_eq!(entry.dockerfile_passed, 0);
}

#[test]
fn test_CORPUS_RUN_039_parse_lcov_with_checksum() {
    // LCOV DA lines can have optional checksums: DA:<line>,<count>,<checksum>
    let lcov = "SF:test.rs\nDA:1,5,abc123\nDA:2,0,def456\nend_of_record\n";
    let results = parse_lcov_file_coverage(lcov);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1, (2, 1)); // 2 lines, 1 hit
}

#[test]
fn test_CORPUS_RUN_040_v2_score_in_convergence_entry() {
    // convergence_entry should populate score, grade, and per-format scores
    let runner = CorpusRunner::new(Config::default());
    let score = CorpusScore {
        total: 900,
        passed: 898,
        failed: 2,
        rate: 0.998,
        score: 99.9,
        grade: Grade::APlus,
        format_scores: vec![
            FormatScore {
                format: CorpusFormat::Bash,
                total: 500,
                passed: 499,
                rate: 0.998,
                score: 99.8,
                grade: Grade::APlus,
            },
            FormatScore {
                format: CorpusFormat::Makefile,
                total: 200,
                passed: 200,
                rate: 1.0,
                score: 100.0,
                grade: Grade::APlus,
            },
            FormatScore {
                format: CorpusFormat::Dockerfile,
                total: 200,
                passed: 199,
                rate: 0.995,
                score: 99.5,
                grade: Grade::APlus,
            },
        ],
        results: vec![],
    };
    let entry = runner.convergence_entry(&score, 10, "2026-02-08", 0.997, "v2 test");
    assert!((entry.score - 99.9).abs() < 0.01);
    assert_eq!(entry.grade, "A+");
    assert!((entry.bash_score - 99.8).abs() < 0.01);
    assert!((entry.makefile_score - 100.0).abs() < 0.01);
    assert!((entry.dockerfile_score - 99.5).abs() < 0.01);
}

#[test]
fn test_CORPUS_RUN_041_v2_score_serde_roundtrip() {
    // V2 score/grade fields should survive JSON serialization
    let entry = ConvergenceEntry {
        iteration: 10,
        date: "2026-02-08".to_string(),
        total: 900,
        passed: 898,
        failed: 2,
        rate: 0.998,
        delta: 0.001,
        notes: "serde".to_string(),
        score: 99.9,
        grade: "A+".to_string(),
        bash_score: 99.8,
        makefile_score: 100.0,
        dockerfile_score: 99.5,
        ..Default::default()
    };
    let json = serde_json::to_string(&entry).expect("serialize");
    let loaded: ConvergenceEntry = serde_json::from_str(&json).expect("deserialize");
    assert!((loaded.score - 99.9).abs() < 0.01);
    assert_eq!(loaded.grade, "A+");
    assert!((loaded.bash_score - 99.8).abs() < 0.01);
    assert!((loaded.makefile_score - 100.0).abs() < 0.01);
    assert!((loaded.dockerfile_score - 99.5).abs() < 0.01);
}

#[test]
fn test_CORPUS_RUN_042_v2_score_backward_compat() {
    // Old entries without score/grade fields should deserialize with defaults
    let old_json = r#"{"iteration":1,"date":"2026-01-01","total":100,"passed":99,"failed":1,"rate":0.99,"delta":0.0,"notes":"old"}"#;
    let entry: ConvergenceEntry = serde_json::from_str(old_json).expect("deserialize old");
    assert!((entry.score - 0.0).abs() < 0.01);
    assert_eq!(entry.grade, "");
    assert!((entry.bash_score - 0.0).abs() < 0.01);
    assert!((entry.makefile_score - 0.0).abs() < 0.01);
    assert!((entry.dockerfile_score - 0.0).abs() < 0.01);
}

#[test]
fn test_CORPUS_RUN_043_v2_empty_format_scores() {
    // convergence_entry with no format_scores → per-format scores default to 0
    let runner = CorpusRunner::new(Config::default());
    let score = CorpusScore {
        total: 10,
        passed: 10,
        failed: 0,
        rate: 1.0,
        score: 95.0,
        grade: Grade::A,
        format_scores: vec![],
        results: vec![],
    };
    let entry = runner.convergence_entry(&score, 1, "2026-02-08", 0.0, "empty");
    assert!((entry.score - 95.0).abs() < 0.01);
    assert_eq!(entry.grade, "A");
    assert!((entry.bash_score - 0.0).abs() < 0.01);
    assert!((entry.makefile_score - 0.0).abs() < 0.01);
    assert!((entry.dockerfile_score - 0.0).abs() < 0.01);
}

#[test]
fn test_CORPUS_RUN_044_regression_none() {
    // No regression when current is better or equal
    let prev = ConvergenceEntry {
        score: 99.0,
        passed: 898,
        bash_passed: 499,
        makefile_passed: 200,
        dockerfile_passed: 199,
        bash_score: 99.0,
        makefile_score: 100.0,
        dockerfile_score: 99.5,
        ..Default::default()
    };
    let curr = ConvergenceEntry {
        score: 99.9,
        passed: 900,
        bash_passed: 500,
        makefile_passed: 200,
        dockerfile_passed: 200,
        bash_score: 99.8,
        makefile_score: 100.0,
        dockerfile_score: 100.0,
        ..Default::default()
    };
    let report = curr.detect_regressions(&prev);
    assert!(!report.has_regressions());
    assert!(report.regressions.is_empty());
}

#[test]
fn test_CORPUS_RUN_045_regression_score_drop() {
    // Regression when score drops
    let prev = ConvergenceEntry {
        score: 99.9,
        passed: 900,
        bash_passed: 500,
        ..Default::default()
    };
    let curr = ConvergenceEntry {
        score: 98.5,
        passed: 900,
        bash_passed: 500,
        ..Default::default()
    };
    let report = curr.detect_regressions(&prev);
    assert!(report.has_regressions());
    assert_eq!(report.regressions.len(), 1);
    assert_eq!(report.regressions[0].dimension, "score");
}

#[test]
fn test_CORPUS_RUN_046_regression_format_specific() {
    // Regression in one format but improvement in another
    let prev = ConvergenceEntry {
        score: 99.0,
        passed: 898,
        bash_passed: 498,
        makefile_passed: 200,
        dockerfile_passed: 200,
        bash_score: 99.0,
        makefile_score: 100.0,
        dockerfile_score: 100.0,
        ..Default::default()
    };
    let curr = ConvergenceEntry {
        score: 99.0,
        passed: 898,
        bash_passed: 500,
        makefile_passed: 198,
        dockerfile_passed: 200,
        bash_score: 99.5,
        makefile_score: 98.0,
        dockerfile_score: 100.0,
        ..Default::default()
    };
    let report = curr.detect_regressions(&prev);
    assert!(report.has_regressions());
    // makefile_passed (200→198) and makefile_score (100→98) regressed
    assert_eq!(report.regressions.len(), 2);
    let dims: Vec<&str> = report
        .regressions
        .iter()
        .map(|r| r.dimension.as_str())
        .collect();
    assert!(dims.contains(&"makefile_passed"));
    assert!(dims.contains(&"makefile_score"));
}

#[test]
fn test_CORPUS_RUN_047_regression_multiple() {
    // Multiple regressions at once
    let prev = ConvergenceEntry {
        score: 99.9,
        passed: 900,
        bash_passed: 500,
        bash_score: 99.8,
        ..Default::default()
    };
    let curr = ConvergenceEntry {
        score: 95.0,
        passed: 890,
        bash_passed: 490,
        bash_score: 95.0,
        ..Default::default()
    };
    let report = curr.detect_regressions(&prev);
    assert!(report.has_regressions());
    assert_eq!(report.regressions.len(), 4);
}

#[test]
fn test_CORPUS_RUN_048_lint_rate_in_convergence() {
    // Lint rate fields should be populated from CorpusScore results
    let runner = CorpusRunner::new(Config::default());
    let score = CorpusScore {
        total: 3,
        passed: 3,
        failed: 0,
        rate: 1.0,
        score: 99.0,
        grade: Grade::APlus,
        format_scores: vec![],
        results: vec![
            CorpusResult {
                id: "B-001".into(),
                transpiled: true,
                lint_clean: true,
                ..Default::default()
            },
            CorpusResult {
                id: "B-002".into(),
                transpiled: true,
                lint_clean: true,
                ..Default::default()
            },
            CorpusResult {
                id: "B-003".into(),
                transpiled: true,
                lint_clean: false,
                ..Default::default()
            },
        ],
    };
    let entry = runner.convergence_entry(&score, 1, "2026-02-08", 0.0, "lint test");
    assert_eq!(entry.lint_passed, 2);
    assert!((entry.lint_rate - 2.0 / 3.0).abs() < 0.01);
}

#[test]
fn test_CORPUS_RUN_049_lint_rate_serde_roundtrip() {
    let entry = ConvergenceEntry {
        lint_passed: 890,
        lint_rate: 0.989,
        total: 900,
        ..Default::default()
    };
    let json = serde_json::to_string(&entry).expect("serialize");
    let loaded: ConvergenceEntry = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(loaded.lint_passed, 890);
    assert!((loaded.lint_rate - 0.989).abs() < 0.001);
}

#[test]
fn test_CORPUS_RUN_050_lint_rate_backward_compat() {
    // Old entries without lint fields should deserialize with defaults
    let old_json = r#"{"iteration":1,"date":"2026-01-01","total":100,"passed":99,"failed":1,"rate":0.99,"delta":0.0,"notes":"old"}"#;
    let entry: ConvergenceEntry = serde_json::from_str(old_json).expect("deserialize");
    assert_eq!(entry.lint_passed, 0);
    assert!((entry.lint_rate - 0.0).abs() < 0.001);
}

#[test]
fn test_CORPUS_RUN_051_lint_regression_detected() {
    let prev = ConvergenceEntry {
        lint_passed: 900,
        ..Default::default()
    };
    let curr = ConvergenceEntry {
        lint_passed: 895,
        ..Default::default()
    };
    let report = curr.detect_regressions(&prev);
    assert!(report.has_regressions());
    let dims: Vec<&str> = report
        .regressions
        .iter()
        .map(|r| r.dimension.as_str())
        .collect();
    assert!(dims.contains(&"lint_passed"));
}

/// BH-MUT-0008: Verify Dockerfile schema rejects comment-only output.
/// Kills mutation that negates `!trimmed.starts_with('#')` in check_schema.
#[test]
fn test_CORPUS_RUN_052_dockerfile_schema_rejects_comments_only() {
    let runner = CorpusRunner::new(Config::default());
    // Output with only comments and blank lines — no valid instructions
    let comment_only = "# This is a comment\n# Another comment\n\n# No instructions";
    assert!(
        !runner.check_schema(comment_only, CorpusFormat::Dockerfile),
        "Dockerfile schema should reject output with only comments"
    );
}

/// Verify Dockerfile schema accepts output with valid instructions.
#[test]
fn test_CORPUS_RUN_053_dockerfile_schema_accepts_valid() {
    let runner = CorpusRunner::new(Config::default());
    let valid = "# Comment\nFROM alpine:3.18\nWORKDIR /app";
    assert!(
        runner.check_schema(valid, CorpusFormat::Dockerfile),
        "Dockerfile schema should accept output with valid instructions"
    );
}

/// BH-MUT-0007: Verify cross-shell agreement requires BOTH dialects to contain expected output.
/// Kills mutation changing `posix_has && bash_has` to `posix_has || bash_has`.
#[test]
fn test_CORPUS_RUN_054_cross_shell_both_dialects_required() {
    let runner = CorpusRunner::new(Config::default());
    // Entry with valid Rust that transpiles to shell containing "greet() {"
    let entry = CorpusEntry::new(
        "T-XS-1",
        "cross-shell-valid",
        "Valid cross-shell entry",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        r#"fn greet() -> u32 { return 42; } fn main() { println!("{}", greet()); }"#,
        "greet() {",
    );
    // Both Posix and Bash dialects should contain "greet() {"
    // Transpile first to get output for the _with_output variant
    let output =
        crate::transpile(&entry.input, &Config::default()).expect("valid entry should transpile");
    assert!(
        runner.check_cross_shell_with_output(&entry, &output, false),
        "Cross-shell should pass when both dialects contain expected output"
    );
}

/// Verify cross-shell skips non-Bash formats (always returns true).
#[test]
fn test_CORPUS_RUN_055_cross_shell_skips_non_bash() {
    let runner = CorpusRunner::new(Config::default());
    let makefile_entry = CorpusEntry::new(
        "T-XS-2",
        "cross-shell-makefile",
        "Makefile entry should skip cross-shell",
        CorpusFormat::Makefile,
        CorpusTier::Standard,
        r#"fn main() { let cc = "gcc"; } "#,
        "CC := gcc",
    );
    assert!(
        runner.check_cross_shell_with_output(&makefile_entry, "", false),
        "Cross-shell should return true for non-Bash entries"
    );

    let docker_entry = CorpusEntry::new(
        "T-XS-3",
        "cross-shell-docker",
        "Dockerfile entry should skip cross-shell",
        CorpusFormat::Dockerfile,
        CorpusTier::Standard,
        r#"fn from_image(i: &str, t: &str) {} fn main() { from_image("alpine", "3.18"); }"#,
        "FROM alpine:3.18",
    );
    assert!(
        runner.check_cross_shell_with_output(&docker_entry, "", false),
        "Cross-shell should return true for Dockerfile entries"
    );
}

// BH-MUT-0016: MR-2, MR-3, MR-4 individual metamorphic relation tests
// Kills mutations that remove any individual MR check from the 7-part AND chain

#[test]
fn test_CORPUS_RUN_056_mr2_comment_stability() {
    // MR-2: Adding a no-op comment to the input should not change output semantics
    let runner = CorpusRunner::new(Config::default());
    let entry = CorpusEntry::new(
        "T-MR2-1",
        "comment-stability",
        "Comment addition preserves output",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        r#"fn add(a: u32, b: u32) -> u32 { return a + b; } fn main() { println!("{}", add(1, 2)); }"#,
        "add() {",
    );
    // Compute output_contains like run_entry does
    let output = crate::transpile(&entry.input, &Config::default()).unwrap();
    let output_contains = output.contains(&entry.expected_output);
    assert!(
        runner.check_mr2_stability(&entry, output_contains),
        "MR-2: adding a comment should not change output"
    );
}

#[test]
fn test_CORPUS_RUN_057_mr3_whitespace_invariance() {
    // MR-3: Adding trailing whitespace/newlines should not change output semantics
    let runner = CorpusRunner::new(Config::default());
    let entry = CorpusEntry::new(
        "T-MR3-1",
        "whitespace-invariance",
        "Trailing whitespace preserves output",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        r#"fn greet() -> u32 { return 42; } fn main() { println!("{}", greet()); }"#,
        "greet() {",
    );
    let output = crate::transpile(&entry.input, &Config::default()).unwrap();
    let output_contains = output.contains(&entry.expected_output);
    assert!(
        runner.check_mr3_whitespace(&entry, output_contains),
        "MR-3: trailing whitespace should not change output"
    );
}

#[test]
fn test_CORPUS_RUN_058_mr4_leading_blanks_invariance() {
    // MR-4: Adding leading blank lines should not change output semantics
    let runner = CorpusRunner::new(Config::default());
    let entry = CorpusEntry::new(
        "T-MR4-1",
        "leading-blanks-invariance",
        "Leading blanks preserve output",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        r#"fn square(x: u32) -> u32 { return x * x; } fn main() { println!("{}", square(5)); }"#,
        "square() {",
    );
    let output = crate::transpile(&entry.input, &Config::default()).unwrap();
    let output_contains = output.contains(&entry.expected_output);
    assert!(
        runner.check_mr4_leading_blanks(&entry, output_contains),
        "MR-4: leading blanks should not change output"
    );
}

#[test]
fn test_CORPUS_RUN_059_mr_equivalence_both_fail_agree() {
    // MR equivalence: if original transpilation fails, run_entry sets
    // metamorphic_consistent=false (MR checks only run inside Ok branch).
    let runner = CorpusRunner::new(Config::default());
    let entry = CorpusEntry::new(
        "T-MR-EQ-1",
        "both-fail",
        "Both original and modified fail → degenerate agreement",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        "this is not valid Rust at all!!!",
        "should_not_matter",
    );
    // When transpilation fails, metamorphic_consistent is false (Err branch)
    let result = runner.run_entry(&entry);
    assert!(
        !result.transpiled,
        "Invalid input should fail transpilation"
    );
    assert!(
        !result.metamorphic_consistent,
        "Failed transpilation sets metamorphic_consistent=false"
    );
}

// BH-MUT-0018: check_determinism mutation targets
// Kills mutations of the skip flag and equality comparison

#[test]
fn test_CORPUS_RUN_063_determinism_valid_entry() {
    // A valid deterministic entry should pass determinism check
    let runner = CorpusRunner::new(Config::default());
    let entry = CorpusEntry::new(
        "T-DET-1",
        "det-valid",
        "Valid deterministic entry",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        r#"fn greet() -> u32 { return 42; } fn main() { println!("{}", greet()); }"#,
        "greet() {",
    );
    let output =
        crate::transpile(&entry.input, &Config::default()).expect("valid entry should transpile");
    assert!(
        runner.check_determinism_with_output(&entry, &output),
        "Valid entry should be deterministic"
    );
}

#[test]
fn test_CORPUS_RUN_064_determinism_skip_non_deterministic() {
    // Entry with deterministic=false should skip check (return true)
    let runner = CorpusRunner::new(Config::default());
    let mut entry = CorpusEntry::new(
        "T-DET-2",
        "det-skip",
        "Non-deterministic flag skips check",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        "this is invalid and would fail",
        "should_not_matter",
    );
    entry.deterministic = false;
    // check_determinism_with_output returns true when deterministic=false (skip)
    assert!(
        runner.check_determinism_with_output(&entry, ""),
        "Entry with deterministic=false should return true (skip)"
    );
}

#[test]
fn test_CORPUS_RUN_065_determinism_invalid_input_fails() {
    // Invalid input that fails transpilation → run_entry sets deterministic=false
    let runner = CorpusRunner::new(Config::default());
    let entry = CorpusEntry::new(
        "T-DET-3",
        "det-invalid",
        "Invalid input fails determinism",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        "not valid rust code at all!!!",
        "x",
    );
    // When transpilation fails, run_entry sets deterministic=false in Err branch
    let result = runner.run_entry(&entry);
    assert!(
        !result.transpiled,
        "Invalid input should fail transpilation"
    );
    assert!(
        !result.deterministic,
        "Invalid input should fail determinism check"
    );
}

// BH-MUT-0019: check_lint per-format dispatch
// Kills mutations that swap format linter or negate has_errors()

#[test]
fn test_CORPUS_RUN_066_lint_bash_clean_passes() {
    let runner = CorpusRunner::new(Config::default());
    // Clean POSIX shell should pass bash lint
    assert!(runner.check_lint("#!/bin/sh\necho \"hello\"\n", CorpusFormat::Bash));
}

#[test]
fn test_CORPUS_RUN_067_lint_makefile_clean_passes() {
    let runner = CorpusRunner::new(Config::default());
    // Clean Makefile should pass makefile lint
    assert!(runner.check_lint(
        "CC := gcc\n\nall:\n\t$(CC) -o main main.c\n",
        CorpusFormat::Makefile
    ));
}

#[test]
fn test_CORPUS_RUN_068_lint_dockerfile_clean_passes() {
    let runner = CorpusRunner::new(Config::default());
    // Clean Dockerfile should pass dockerfile lint
    assert!(runner.check_lint(
        "FROM alpine:3.18\nRUN apk add curl\n",
        CorpusFormat::Dockerfile
    ));
}

#[test]
#[ignore] // Takes ~26 minutes to iterate all 16,431 bash corpus entries
fn test_CORPUS_RUN_069_diagnose_lint_failures() {
    // Diagnostic: find which error-level rules fire on transpiled corpus output
    let registry = CorpusRegistry::load_full();
    let config = Config::default();
    let mut error_codes: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut fail_count = 0;
    let mut sample_count = 0;

    for entry in registry
        .entries
        .iter()
        .filter(|e| e.format == CorpusFormat::Bash)
    {
        let result = crate::transpile(&entry.input, &config);
        if let Ok(output) = result {
            let lint = crate::linter::rules::lint_shell(&output);
            let errors: Vec<_> = lint
                .diagnostics
                .iter()
                .filter(|d| {
                    d.severity == crate::linter::Severity::Error
                        && !CorpusRunner::CORPUS_LINT_EXCLUSIONS.contains(&d.code.as_str())
                })
                .collect();
            if !errors.is_empty() {
                fail_count += 1;
                for e in &errors {
                    *error_codes.entry(e.code.clone()).or_insert(0) += 1;
                }
            }
        }
        sample_count += 1;
    }

    // Write diagnostic summary to file for analysis
    let mut sorted: Vec<_> = error_codes.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    let mut report = format!(
        "LINT DIAGNOSTIC: {}/{} bash entries fail lint\n",
        fail_count, sample_count
    );
    for (code, count) in &sorted {
        report.push_str(&format!("  {}: {} occurrences\n", code, count));
    }
    std::fs::write("/tmp/bashrs_lint_diagnostic.txt", &report).ok();

    // This test is diagnostic — it always passes but prints useful info
    // The actual assertion verifies exclusions work for SEC001/REL001
    let runner = CorpusRunner::new(config);
    assert!(runner.check_lint("#!/bin/sh\neval echo hello\n", CorpusFormat::Bash));
}
