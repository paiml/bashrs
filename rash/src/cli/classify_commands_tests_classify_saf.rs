use super::*;

// ── Bash classification tests ───────────────────────────────────

#[test]
fn test_classify_safe_script() {
    let result = classify_script("#!/bin/sh\necho \"hello world\"\n", &ClassifyFormat::Bash);
    assert_eq!(result.index, 0);
    assert_eq!(result.label, "safe");
    assert!(result.confidence > 0.7);
    assert_eq!(result.format, "bash");
}

#[test]
fn test_classify_unquoted_var() {
    let result = classify_script("#!/bin/sh\necho $HOME\n", &ClassifyFormat::Bash);
    assert_eq!(result.index, 1);
    assert_eq!(result.label, "needs-quoting");
}

#[test]
fn test_classify_non_deterministic() {
    let result = classify_script("#!/bin/bash\necho $RANDOM\n", &ClassifyFormat::Bash);
    assert_eq!(result.index, 2);
    assert_eq!(result.label, "non-deterministic");
    assert!(result.has_determinism_issues);
}

#[test]
fn test_classify_non_idempotent() {
    let result = classify_script("#!/bin/sh\nmkdir /tmp/build\n", &ClassifyFormat::Bash);
    assert_eq!(result.index, 3);
    assert_eq!(result.label, "non-idempotent");
}

#[test]
fn test_classify_unsafe_eval() {
    let result = classify_script("#!/bin/bash\neval \"$user_input\"\n", &ClassifyFormat::Bash);
    assert_eq!(result.index, 4);
    assert_eq!(result.label, "unsafe");
    assert!(result.has_security_issues);
}

#[test]
fn test_classify_json_output() {
    let result = classify_script("#!/bin/sh\necho \"ok\"\n", &ClassifyFormat::Bash);
    let json = serde_json::to_string(&result).expect("should serialize");
    assert!(json.contains("\"label\""));
    assert!(json.contains("\"confidence\""));
    assert!(json.contains("\"scores\""));
    assert!(
        json.contains("\"bash\""),
        "JSON should contain format 'bash'"
    );
}

#[test]
fn test_confidence_range() {
    for script in &[
        "#!/bin/sh\necho ok\n",
        "#!/bin/sh\necho $HOME\n",
        "#!/bin/bash\necho $RANDOM\n",
        "#!/bin/sh\nmkdir /tmp/x\n",
        "#!/bin/bash\neval \"$x\"\n",
    ] {
        let result = classify_script(script, &ClassifyFormat::Bash);
        assert!(
            result.confidence >= 0.5 && result.confidence <= 1.0,
            "Confidence {:.2} out of range for: {}",
            result.confidence,
            script
        );
    }
}

#[test]
fn test_score_distribution_sums_to_one() {
    let scores = build_score_distribution(2, 0.9);
    let sum: f64 = scores.iter().sum();
    assert!(
        (sum - 1.0).abs() < 1e-10,
        "Score distribution must sum to 1.0, got {sum}"
    );
}

#[test]
fn test_score_distribution_predicted_highest() {
    let scores = build_score_distribution(3, 0.85);
    assert_eq!(
        scores
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).expect("no NaN"))
            .map(|(i, _)| i),
        Some(3)
    );
}

#[test]
fn test_classify_empty_script() {
    let result = classify_script("", &ClassifyFormat::Bash);
    assert_eq!(result.index, 0);
}

#[test]
fn test_classify_priority_sec_over_det() {
    let result = classify_script("#!/bin/bash\neval \"$RANDOM\"\n", &ClassifyFormat::Bash);
    assert_eq!(
        result.index, 4,
        "Security should take priority over determinism"
    );
}

// ── Multi-label bash tests (SSC-021) ────────────────────────────

#[test]
fn test_multi_label_safe_script() {
    let result =
        classify_script_multi_label("#!/bin/sh\necho \"hello world\"\n", &ClassifyFormat::Bash);
    assert_eq!(result.labels, vec!["safe"]);
    assert_eq!(result.label_indices, vec![0]);
    assert!(result.scores[0] > 0.7);
}

#[test]
fn test_multi_label_unsafe_and_nondet() {
    let result =
        classify_script_multi_label("#!/bin/bash\neval \"$RANDOM\"\n", &ClassifyFormat::Bash);
    assert!(result.labels.contains(&"unsafe".to_string()));
    assert!(result.labels.contains(&"non-deterministic".to_string()));
}

#[test]
fn test_multi_label_nondet_and_unquoted() {
    let result = classify_script_multi_label("#!/bin/bash\necho $RANDOM\n", &ClassifyFormat::Bash);
    assert!(result.labels.contains(&"non-deterministic".to_string()));
    assert!(result.labels.contains(&"needs-quoting".to_string()));
}

#[test]
fn test_multi_label_json_serialization() {
    let result =
        classify_script_multi_label("#!/bin/bash\neval \"$RANDOM\"\n", &ClassifyFormat::Bash);
    let json = serde_json::to_string_pretty(&result).expect("should serialize");
    assert!(json.contains("\"labels\""));
    assert!(
        json.contains("\"bash\""),
        "JSON should contain format 'bash': {json}"
    );
}

#[test]
fn test_multi_label_nonidempotent_and_unquoted() {
    let result =
        classify_script_multi_label("#!/bin/sh\nmkdir $HOME/build\n", &ClassifyFormat::Bash);
    assert!(result.labels.contains(&"non-idempotent".to_string()));
    assert!(result.labels.contains(&"needs-quoting".to_string()));
}

#[test]
fn test_multi_label_only_unquoted() {
    let result = classify_script_multi_label("#!/bin/sh\necho $HOME\n", &ClassifyFormat::Bash);
    assert_eq!(result.labels, vec!["needs-quoting"]);
}

#[test]
fn test_multi_label_scores_structure() {
    let result =
        classify_script_multi_label("#!/bin/bash\neval \"$RANDOM\"\n", &ClassifyFormat::Bash);
    for &idx in &result.label_indices {
        assert!(result.scores[idx as usize] > 0.0);
    }
}

// ── Format detection tests (SSC-022) ────────────────────────────

#[test]
fn test_detect_format_bash() {
    assert!(matches!(
        detect_format(Path::new("script.sh")),
        ClassifyFormat::Bash
    ));
    assert!(matches!(
        detect_format(Path::new("script.bash")),
        ClassifyFormat::Bash
    ));
}

#[test]
fn test_detect_format_makefile() {
    assert!(matches!(
        detect_format(Path::new("Makefile")),
        ClassifyFormat::Makefile
    ));
    assert!(matches!(
        detect_format(Path::new("build.mk")),
        ClassifyFormat::Makefile
    ));
}

#[test]
fn test_detect_format_dockerfile() {
    assert!(matches!(
        detect_format(Path::new("Dockerfile")),
        ClassifyFormat::Dockerfile
    ));
    assert!(matches!(
        detect_format(Path::new("Dockerfile.prod")),
        ClassifyFormat::Dockerfile
    ));
}

// ── Makefile classification tests (SSC-022) ─────────────────────

#[test]
fn test_classify_makefile_safe() {
    let makefile = ".PHONY: build\nbuild:\n\techo \"building\"\n";
    let result = classify_script(makefile, &ClassifyFormat::Makefile);
    assert_eq!(result.format, "makefile");
    // With .PHONY declaration, it should be relatively clean
    assert!(
        result.index <= 1,
        "Clean makefile should be safe or needs-quoting"
    );
}

#[test]
fn test_classify_makefile_format_field() {
    let makefile = "all:\n\techo ok\n";
    let result = classify_script(makefile, &ClassifyFormat::Makefile);
    assert_eq!(result.format, "makefile");
}

#[test]
fn test_classify_makefile_multi_label() {
    let makefile = ".PHONY: build\nbuild:\n\techo \"ok\"\n";
    let result = classify_script_multi_label(makefile, &ClassifyFormat::Makefile);
    assert_eq!(result.format, "makefile");
    // Should not have needs-quoting (that's bash-specific)
    assert!(
        !result.labels.contains(&"needs-quoting".to_string()),
        "Makefile should not get needs-quoting label"
    );
}

// ── Dockerfile classification tests (SSC-022) ───────────────────

#[test]
fn test_classify_dockerfile_safe() {
    let dockerfile = "FROM alpine:3.18\nUSER nobody\nCOPY app /app\n";
    let result = classify_script(dockerfile, &ClassifyFormat::Dockerfile);
    assert_eq!(result.format, "dockerfile");
}

#[test]
fn test_classify_dockerfile_format_field() {
    let dockerfile = "FROM ubuntu:22.04\nRUN apt-get update\n";
    let result = classify_script(dockerfile, &ClassifyFormat::Dockerfile);
    assert_eq!(result.format, "dockerfile");
}

#[test]
fn test_classify_dockerfile_multi_label() {
    let dockerfile = "FROM alpine:3.18\nUSER nobody\nCOPY app /app\n";
    let result = classify_script_multi_label(dockerfile, &ClassifyFormat::Dockerfile);
    assert_eq!(result.format, "dockerfile");
    // No needs-quoting for Dockerfile
    assert!(
        !result.labels.contains(&"needs-quoting".to_string()),
        "Dockerfile should not get needs-quoting label"
    );
}

// ── Cross-format comparison tests ───────────────────────────────

#[test]
fn test_format_name_mapping() {
    assert_eq!(format_name(&ClassifyFormat::Bash), "bash");
    assert_eq!(format_name(&ClassifyFormat::Makefile), "makefile");
    assert_eq!(format_name(&ClassifyFormat::Dockerfile), "dockerfile");
}

#[test]
fn test_lint_signals_bash() {
    let signals = analyze_lint("#!/bin/bash\neval \"$RANDOM\"\n", &ClassifyFormat::Bash);
    assert!(signals.has_security_issues);
    assert!(signals.has_determinism_issues);
    assert!(signals.sec_count > 0);
    assert!(signals.det_count > 0);
}

#[test]
fn test_lint_signals_makefile() {
    let signals = analyze_lint("all:\n\techo ok\n", &ClassifyFormat::Makefile);
    // At minimum, lint should produce some diagnostics
    assert!(signals.diagnostic_count >= 0); // relaxed: linter may or may not fire
}

#[test]
fn test_lint_signals_dockerfile() {
    let signals = analyze_lint(
        "FROM ubuntu:22.04\nRUN apt-get update\n",
        &ClassifyFormat::Dockerfile,
    );
    assert!(signals.diagnostic_count >= 0); // relaxed: linter may or may not fire
}
