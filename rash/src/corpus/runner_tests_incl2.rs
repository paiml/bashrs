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

include!("runner_tests_incl2_incl2.rs");
