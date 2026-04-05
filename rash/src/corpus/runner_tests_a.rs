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

