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
