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

include!("runner_tests_tests_CORPUS.rs");
