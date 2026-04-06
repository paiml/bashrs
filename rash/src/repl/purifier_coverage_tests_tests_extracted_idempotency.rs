
    /// mkdir -p branch returns two alternatives.
    #[test]
    fn test_idempotency_alternatives_mkdir_p() {
        let alts = generate_idempotency_alternatives("mkdir → mkdir -p");
        assert_eq!(alts.len(), 2);
        assert!(alts[0].approach.contains("Check before"));
        assert!(alts[1].approach.contains("mkdir"));
        // First alternative has pros and cons
        assert!(!alts[0].pros.is_empty());
        assert!(!alts[0].cons.is_empty());
    }

    /// rm -f branch returns two alternatives.
    #[test]
    fn test_idempotency_alternatives_rm_f() {
        let alts = generate_idempotency_alternatives("rm → rm -f");
        assert_eq!(alts.len(), 2);
        // Verify example code is non-empty
        for alt in &alts {
            assert!(!alt.example.is_empty());
            assert!(!alt.when_to_use.is_empty());
        }
    }

    /// ln -sf branch returns two alternatives.
    #[test]
    fn test_idempotency_alternatives_ln_sf() {
        let alts = generate_idempotency_alternatives("ln -s → ln -sf");
        assert_eq!(alts.len(), 2);
        for alt in &alts {
            assert!(!alt.approach.is_empty());
        }
    }

    /// Unknown transformation returns the default alternative.
    #[test]
    fn test_idempotency_alternatives_default() {
        let alts = generate_idempotency_alternatives("some unknown operation");
        assert_eq!(alts.len(), 1);
        assert!(alts[0].approach.contains("idempotency check"));
    }

    // ── generate_determinism_alternatives ────────────────────────────────────

    #[test]
    fn test_determinism_alternatives_random() {
        let alts = generate_determinism_alternatives("Remove $RANDOM");
        assert_eq!(alts.len(), 4);
        assert!(alts[0].approach.contains("UUID") || alts[0].approach.contains("uuid"));
    }

    #[test]
    fn test_determinism_alternatives_timestamp() {
        let alts = generate_determinism_alternatives("Remove timestamps");
        assert_eq!(alts.len(), 3);
    }

    #[test]
    fn test_determinism_alternatives_default() {
        let alts = generate_determinism_alternatives("unknown determinism issue");
        assert_eq!(alts.len(), 1);
        assert!(alts[0].pros.contains(&"Fully deterministic".to_string()));
    }

    // ── generate_safety_alternatives ─────────────────────────────────────────

    #[test]
    fn test_safety_alternatives_quoting() {
        let alts = generate_safety_alternatives("Quote variables");
        assert_eq!(alts.len(), 3);
    }

    #[test]
    fn test_safety_alternatives_default() {
        let alts = generate_safety_alternatives("unknown safety issue");
        assert_eq!(alts.len(), 1);
    }

    // ── format_alternatives ───────────────────────────────────────────────────

    #[test]
    fn test_format_alternatives_empty() {
        let output = format_alternatives(&[]);
        assert!(output.is_empty());
    }

    #[test]
    fn test_format_alternatives_single() {
        let alt = Alternative::new("My approach", "example code", "when to use this")
            .add_pro("Good thing")
            .add_con("Bad thing");
        let output = format_alternatives(&[alt]);
        assert!(output.contains("Alternative Approaches"));
        assert!(output.contains("My approach"));
        assert!(output.contains("example code"));
        assert!(output.contains("Good thing"));
        assert!(output.contains("Bad thing"));
    }

    #[test]
    fn test_format_alternatives_multiple() {
        let alts = generate_idempotency_alternatives("mkdir → mkdir -p");
        let output = format_alternatives(&alts);
        assert!(output.contains("1."));
        assert!(output.contains("2."));
    }

    // ── generate_idempotency_rationale ────────────────────────────────────────

    #[test]
    fn test_idempotency_rationale_mkdir_p() {
        let r = generate_idempotency_rationale("mkdir → mkdir -p");
        assert_eq!(r.severity, SafetySeverity::High);
        assert!(!r.failures_eliminated.is_empty());
        assert!(!r.impact_without_fix.is_empty());
    }

    #[test]
    fn test_idempotency_rationale_rm_f() {
        let r = generate_idempotency_rationale("rm → rm -f");
        assert_eq!(r.severity, SafetySeverity::High);
        assert!(!r.failures_eliminated.is_empty());
    }

    #[test]
    fn test_idempotency_rationale_ln_sf() {
        let r = generate_idempotency_rationale("ln -s → ln -sf");
        assert_eq!(r.severity, SafetySeverity::High);
    }

    #[test]
    fn test_idempotency_rationale_default() {
        let r = generate_idempotency_rationale("unknown operation");
        assert_eq!(r.severity, SafetySeverity::Medium);
    }

    // ── generate_determinism_rationale ────────────────────────────────────────

    #[test]
    fn test_determinism_rationale_random() {
        let r = generate_determinism_rationale("Remove $RANDOM");
        assert_eq!(r.severity, SafetySeverity::Critical);
        assert!(!r.vulnerabilities_prevented.is_empty());
        assert!(!r.failures_eliminated.is_empty());
    }

    #[test]
    fn test_determinism_rationale_timestamp() {
        let r = generate_determinism_rationale("Remove timestamps (date)");
        assert_eq!(r.severity, SafetySeverity::High);
        assert!(!r.vulnerabilities_prevented.is_empty());
    }

    #[test]
    fn test_determinism_rationale_default() {
        let r = generate_determinism_rationale("unknown non-determinism");
        assert_eq!(r.severity, SafetySeverity::Medium);
    }

    // ── generate_safety_rationale ─────────────────────────────────────────────

    #[test]
    fn test_safety_rationale_quoting() {
        let r = generate_safety_rationale("Quote variables for safety");
        assert_eq!(r.severity, SafetySeverity::Critical);
        assert!(!r.vulnerabilities_prevented.is_empty());
        assert!(!r.attack_vectors_closed.is_empty());
    }

    #[test]
    fn test_safety_rationale_default() {
        let r = generate_safety_rationale("unknown safety transformation");
        assert_eq!(r.severity, SafetySeverity::Medium);
    }

    // ── format_safety_rationale ───────────────────────────────────────────────

    #[test]
    fn test_format_safety_rationale_critical() {
        let r = SafetyRationale::new()
            .add_vulnerability("Injection")
            .add_failure("Script crash")
            .add_attack_vector("Metachar injection")
            .with_impact("Bad things happen")
            .with_severity(SafetySeverity::Critical);
        let output = format_safety_rationale(&r);
        assert!(output.contains("CRITICAL"));
        assert!(output.contains("Vulnerabilities Prevented"));
        assert!(output.contains("Failures Eliminated"));
        assert!(output.contains("Attack Vectors Closed"));
        assert!(output.contains("Impact Without Fix"));
        assert!(output.contains("Injection"));
    }

    #[test]
    fn test_format_safety_rationale_high() {
        let r = SafetyRationale::new()
            .add_failure("Crash")
            .with_severity(SafetySeverity::High);
        let output = format_safety_rationale(&r);
        assert!(output.contains("HIGH"));
    }

    #[test]
    fn test_format_safety_rationale_medium() {
        let r = SafetyRationale::new().with_severity(SafetySeverity::Medium);
        let output = format_safety_rationale(&r);
        assert!(output.contains("MEDIUM"));
    }

    #[test]
    fn test_format_safety_rationale_low() {
        let r = SafetyRationale::new(); // default is Low
        let output = format_safety_rationale(&r);
        assert!(output.contains("LOW"));
    }

    // ── SafetyRationale builder ───────────────────────────────────────────────

    #[test]
    fn test_safety_rationale_default_impl() {
        let r: SafetyRationale = Default::default();
        assert!(r.vulnerabilities_prevented.is_empty());
        assert!(r.failures_eliminated.is_empty());
        assert!(r.attack_vectors_closed.is_empty());
        assert!(r.impact_without_fix.is_empty());
        assert_eq!(r.severity, SafetySeverity::Low);
    }

    // ── TransformationExplanation builder ────────────────────────────────────

    #[test]
    fn test_transformation_with_safety_rationale() {
        let rationale = SafetyRationale::new()
            .add_vulnerability("Injection")
            .with_severity(SafetySeverity::Critical);
        let explanation = TransformationExplanation::new(
            TransformationCategory::Safety,
            "Quote variables",
            "echo $v",
            "echo \"$v\"",
            "Added quotes",
            "Prevents injection",
        )
        .with_safety_rationale(rationale.clone());
        assert_eq!(explanation.safety_rationale, rationale);
    }

    #[test]
    fn test_transformation_with_alternatives() {
        let alts = generate_idempotency_alternatives("mkdir → mkdir -p");
        let explanation = TransformationExplanation::new(
            TransformationCategory::Idempotency,
            "mkdir → mkdir -p",
            "mkdir /tmp",
            "mkdir -p /tmp",
            "Added -p",
            "Idempotent",
        )
        .with_alternatives(alts.clone());
        assert_eq!(explanation.alternatives.len(), alts.len());
    }

    // ── format_purified_lint_result (non-context variant) ────────────────────

    #[test]
    fn test_format_purified_lint_result_with_idem_and_sec() {
        let lr = make_lint_result_with(&["IDEM001", "SEC001"]);
        let result = PurifiedLintResult::new("code".to_string(), lr);
        let formatted = format_purified_lint_result(&result);
        assert!(formatted.contains("IDEM"));
        assert!(formatted.contains("SEC"));
        assert!(formatted.contains("critical violation"));
    }

    // ── PurifiedLintResult accessors ─────────────────────────────────────────

    #[test]
    fn test_purified_lint_result_det_violations_accessor() {
        let lr = make_lint_result_with(&["DET001", "DET002", "SEC001"]);
        let result = PurifiedLintResult::new("x".to_string(), lr);
        assert_eq!(result.det_violations().len(), 2);
        assert_eq!(result.sec_violations().len(), 1);
        assert_eq!(result.idem_violations().len(), 0);
    }

    #[test]
    fn test_purified_lint_result_critical_violations_count() {
        let lr = make_lint_result_with(&["DET001", "IDEM001", "SEC001", "SC2086"]);
        let result = PurifiedLintResult::new("x".to_string(), lr);
        // SC2086 is not critical
        assert_eq!(result.critical_violations(), 3);
        assert!(!result.is_clean);
    }

    // ── PurificationError ─────────────────────────────────────────────────────

    #[test]
    fn test_purification_error_std_error() {
        let lr = make_lint_result_with(&["DET001"]);
        let result = PurifiedLintResult::new("x".to_string(), lr);
        let err = PurificationError::new(&result);
        let std_err: &dyn std::error::Error = &err;
        assert!(!std_err.to_string().is_empty());
    }

    // ── explain_purification_changes (simple variant) ─────────────────────────

    #[test]
    fn test_explain_changes_with_rm() {
        let result = explain_purification_changes("rm /tmp/test_file");
        assert!(result.is_ok());
    }

    #[test]
    fn test_explain_changes_fallback_message() {
        // Code that purifier changes but has no specific pattern match:
        // we just verify no panic and returns Ok
        let result = explain_purification_changes("echo $RANDOM");
        assert!(result.is_ok());
    }

    // ── format_transformation_report with line number ─────────────────────────

    #[test]
    fn test_format_transformation_report_with_line_number() {
        let explanation = TransformationExplanation::new(
            TransformationCategory::Determinism,
            "Remove $RANDOM",
            "echo $RANDOM",
            "echo 42",
            "Removed RANDOM",
            "Determinism",
        )
        .with_line_number(7);

        let report = format_transformation_report(&[explanation]);
        assert!(report.contains("DETERMINISM"));
        assert!(report.contains("Line: 7"));
        assert!(report.contains("Remove $RANDOM"));
    }

    #[test]
    fn test_format_transformation_report_multiple_entries() {
        let explanations = vec![
            TransformationExplanation::new(
                TransformationCategory::Idempotency,
                "mkdir → mkdir -p",
                "mkdir /tmp",
                "mkdir -p /tmp",
                "Added -p",
                "Idempotent",
            ),
            TransformationExplanation::new(
                TransformationCategory::Safety,
                "Quote variables",
                "echo $v",
                "echo \"$v\"",
                "Added quotes",
                "Safety",
            ),
        ];
        let report = format_transformation_report(&explanations);
        assert!(report.contains("IDEMPOTENCY"));
        assert!(report.contains("SAFETY"));
    }

    // ── purify_and_lint integration ───────────────────────────────────────────

    #[test]
    fn test_purify_and_lint_mkdir_result() {
        let result = purify_and_lint("mkdir /tmp/test_dir");
        assert!(result.is_ok());
        let plr = result.unwrap();
        assert!(plr.purified_code.contains("mkdir"));
    }

    // ── explain_purification_changes_detailed coverage ──────────────────────

    #[test]
    fn test_explain_detailed_no_changes() {
        // Code that is already purified should return empty explanations
        let result = explain_purification_changes_detailed("echo hello");
        assert!(result.is_ok());
        let explanations = result.unwrap();
        assert!(explanations.is_empty());
    }

    #[test]
    fn test_explain_detailed_mkdir_idempotency() {
        let result = explain_purification_changes_detailed("mkdir /tmp/test");
        assert!(result.is_ok());
        let explanations = result.unwrap();
        let has_mkdir = explanations.iter().any(|e| {
            matches!(e.category, TransformationCategory::Idempotency) && e.title.contains("mkdir")
        });
        assert!(has_mkdir, "Should detect mkdir → mkdir -p transformation");
    }

    #[test]
    fn test_explain_detailed_rm_idempotency() {
        let result = explain_purification_changes_detailed("rm /tmp/testfile");
        assert!(result.is_ok());
        let explanations = result.unwrap();
        let has_rm = explanations.iter().any(|e| {
            matches!(e.category, TransformationCategory::Idempotency) && e.title.contains("rm")
        });
        assert!(has_rm, "Should detect rm → rm -f transformation");
    }

    #[test]
    fn test_explain_detailed_variable_quoting_safety_category() {
        let result = explain_purification_changes_detailed("echo $HOME");
        assert!(result.is_ok());
        let explanations = result.unwrap();
        let has_quoting = explanations
            .iter()
            .any(|e| matches!(e.category, TransformationCategory::Safety));
        assert!(has_quoting, "Should detect variable quoting transformation");
    }

    #[test]
    fn test_explain_detailed_random_determinism() {
        let result = explain_purification_changes_detailed("echo $RANDOM");
        assert!(result.is_ok());
        let explanations = result.unwrap();
        let has_random = explanations
            .iter()
            .any(|e| matches!(e.category, TransformationCategory::Determinism));
        assert!(
            has_random,
            "Should detect $RANDOM removal as determinism transformation"
        );
    }

    #[test]
    fn test_explain_detailed_ln_idempotency() {
        // Note: the purifier may or may not transform ln -s depending on
        // whether the transpiler handles it. We verify at minimum no error.
        let result = explain_purification_changes_detailed("ln -s /src /dest");
        assert!(result.is_ok());
        // If explanations are present, verify they make sense
        let explanations = result.unwrap();
        for e in &explanations {
            // All explanations should have non-empty fields
            assert!(!e.title.is_empty());
            assert!(!e.what_changed.is_empty());


    }
}
