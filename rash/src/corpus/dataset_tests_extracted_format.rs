
    #[test]
    fn test_format_dataset_info() {
        let entries = vec![make_entry("B-001", CorpusFormat::Bash)];
        let registry = CorpusRegistry { entries };
        let info = dataset_info(&registry);
        let table = format_dataset_info(&info);
        assert!(table.contains("Dataset Schema"));
        assert!(table.contains("id"));
        assert!(table.contains("string"));
        assert!(table.contains("float64"));
    }

    #[test]
    fn test_publish_checks_all_pass() {
        let score = CorpusScore {
            total: 900,
            passed: 900,
            failed: 0,
            rate: 1.0,
            score: 99.9,
            grade: crate::corpus::registry::Grade::APlus,
            format_scores: vec![
                crate::corpus::runner::FormatScore {
                    format: CorpusFormat::Bash,
                    total: 500,
                    passed: 500,
                    rate: 1.0,
                    score: 99.7,
                    grade: crate::corpus::registry::Grade::APlus,
                },
                crate::corpus::runner::FormatScore {
                    format: CorpusFormat::Makefile,
                    total: 200,
                    passed: 200,
                    rate: 1.0,
                    score: 100.0,
                    grade: crate::corpus::registry::Grade::APlus,
                },
                crate::corpus::runner::FormatScore {
                    format: CorpusFormat::Dockerfile,
                    total: 200,
                    passed: 200,
                    rate: 1.0,
                    score: 100.0,
                    grade: crate::corpus::registry::Grade::APlus,
                },
            ],
            results: vec![],
        };

        let checks = check_publish_readiness(&score);
        assert!(checks.iter().all(|c| c.passed));
    }

    #[test]
    fn test_publish_checks_some_fail() {
        let score = CorpusScore {
            total: 50,
            passed: 45,
            failed: 5,
            rate: 0.90,
            score: 85.0,
            grade: crate::corpus::registry::Grade::B,
            format_scores: vec![],
            results: vec![],
        };

        let checks = check_publish_readiness(&score);
        assert!(!checks.iter().all(|c| c.passed));
        // rate < 99%, score < 90, no formats, failed > 0, size < 100
        let failed_count = checks.iter().filter(|c| !c.passed).count();
        assert!(failed_count >= 3);
    }

    #[test]
    fn test_format_publish_checks() {
        let checks = vec![PublishCheck {
            name: "Test check",
            passed: true,
            value: "ok".to_string(),
        }];
        let table = format_publish_checks(&checks);
        assert!(table.contains("Test check"));
        assert!(table.contains("PASS"));
        assert!(table.contains("Ready to publish"));
    }

    #[test]
    fn test_format_publish_checks_failure() {
        let checks = vec![PublishCheck {
            name: "Failing check",
            passed: false,
            value: "bad".to_string(),
        }];
        let table = format_publish_checks(&checks);
        assert!(table.contains("FAIL"));
        assert!(table.contains("check(s) failed"));
    }

    #[test]
    fn test_days_to_ymd_epoch() {
        assert_eq!(days_to_ymd(0), (1970, 1, 1));
    }

    #[test]
    fn test_days_to_ymd_known_date() {
        // 2026-02-09 is day 20,493 since epoch
        let (y, m, d) = days_to_ymd(20_493);
        assert_eq!(y, 2026);
        assert_eq!(m, 2);
        assert_eq!(d, 9);
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2025));
    }

    #[test]
    fn test_current_date_format() {
        let date = current_date();
        assert_eq!(date.len(), 10);
        assert_eq!(&date[4..5], "-");
        assert_eq!(&date[7..8], "-");
    }

    #[test]
    fn test_dataset_row_serializes() {
        let row = DatasetRow {
            id: "B-001".into(),
            name: "test".into(),
            tier: 1,
            format: "bash".into(),
            input_rust: "fn main() {}".into(),
            expected_output: "#!/bin/sh\n".into(),
            actual_output: "#!/bin/sh\n".into(),
            transpiled: true,
            output_correct: true,
            lint_clean: true,
            deterministic: true,
            score: 100.0,
            grade: "A+".into(),
            safety_index: 0,
            safety_label: "safe".into(),
            bashrs_version: "6.61.0".into(),
            commit_sha: "abc1234".into(),
            date: "2026-02-09".into(),
        };

        let json = serde_json::to_string(&row);
        assert!(json.is_ok());
        let s = json.expect("serialization should succeed");
        assert!(s.contains("B-001"));
        assert!(s.contains("safety_index"));
        assert!(s.contains("safe"));
    }

    // ── Safety label derivation tests ───────────────────────────────

    #[test]
    fn test_derive_safety_label_safe() {
        // Clean transpiled output with quoted vars → safe (0)
        let script = "#!/bin/sh\necho \"hello world\"\nmkdir -p \"$HOME/tmp\"\n";
        assert_eq!(derive_safety_label(script, true, true, true), 0);
    }

    #[test]
    fn test_derive_safety_label_not_transpiled() {
        // Failed transpilation → unsafe (4)
        assert_eq!(derive_safety_label("", false, true, true), 4);
    }

    #[test]
    fn test_derive_safety_label_not_lint_clean() {
        // Lint failures → unsafe (4)
        assert_eq!(derive_safety_label("echo ok", true, false, true), 4);
    }

    #[test]
    fn test_derive_safety_label_not_deterministic() {
        // Non-deterministic → class 2
        assert_eq!(derive_safety_label("echo ok", true, true, false), 2);
    }

    #[test]
    fn test_derive_safety_label_non_idempotent_mkdir() {
        // mkdir without -p → non-idempotent (3)
        let script = "#!/bin/sh\nmkdir /tmp/build\n";
        assert_eq!(derive_safety_label(script, true, true, true), 3);
    }

    #[test]
    fn test_derive_safety_label_idempotent_mkdir() {
        // mkdir -p → safe (0)
        let script = "#!/bin/sh\nmkdir -p /tmp/build\n";
        assert_eq!(derive_safety_label(script, true, true, true), 0);
    }

    #[test]
    fn test_derive_safety_label_non_idempotent_rm() {
        // rm without -f → non-idempotent (3)
        let script = "#!/bin/sh\nrm /tmp/file\n";
        assert_eq!(derive_safety_label(script, true, true, true), 3);
    }

    #[test]
    fn test_derive_safety_label_idempotent_rm() {
        // rm -f → safe (0)
        let script = "#!/bin/sh\nrm -f /tmp/file\n";
        assert_eq!(derive_safety_label(script, true, true, true), 0);
    }

    #[test]
    fn test_derive_safety_label_non_idempotent_ln() {
        // ln -s without -f → non-idempotent (3)
        let script = "#!/bin/sh\nln -s /a /b\n";
        assert_eq!(derive_safety_label(script, true, true, true), 3);
    }

    #[test]
    fn test_derive_safety_label_unquoted_var() {
        // Unquoted $VAR → needs-quoting (1)
        let script = "#!/bin/sh\necho $HOME\n";
        assert_eq!(derive_safety_label(script, true, true, true), 1);
    }

    #[test]
    fn test_derive_safety_label_quoted_var() {
        // Quoted "$VAR" → safe (0)
        let script = "#!/bin/sh\necho \"$HOME\"\n";
        assert_eq!(derive_safety_label(script, true, true, true), 0);
    }

    #[test]
    fn test_derive_safety_label_single_quoted_var() {
        // Single-quoted '$VAR' → safe (0) — no expansion in single quotes
        let script = "#!/bin/sh\necho '$HOME'\n";
        assert_eq!(derive_safety_label(script, true, true, true), 0);
    }

    #[test]
    fn test_derive_safety_label_priority_unsafe_over_nondeterministic() {
        // Not lint clean AND not deterministic → unsafe (4) wins
        assert_eq!(derive_safety_label("echo ok", true, false, false), 4);
    }

    #[test]
    fn test_derive_safety_label_priority_nondeterministic_over_non_idempotent() {
        // Non-deterministic AND has mkdir → non-deterministic (2) wins
        let script = "#!/bin/sh\nmkdir /tmp/build\n";
        assert_eq!(derive_safety_label(script, true, true, false), 2);
    }

    #[test]
    fn test_has_non_idempotent_pattern_comments_ignored() {
        assert!(!has_non_idempotent_pattern("# mkdir /tmp/build\n"));
        assert!(!has_non_idempotent_pattern("  # rm file\n"));
    }

    #[test]
    fn test_line_has_unquoted_var_basic() {
        assert!(line_has_unquoted_var("echo $HOME"));
        assert!(line_has_unquoted_var("echo ${HOME}"));
        assert!(!line_has_unquoted_var("echo \"$HOME\""));
        assert!(!line_has_unquoted_var("echo '$HOME'"));
        assert!(!line_has_unquoted_var("echo hello"));
    }

    #[test]
    fn test_line_has_unquoted_var_dollar_special() {
        // $? $# $0 etc. are special — not flagged (no alpha/underscore/brace after $)
        assert!(!line_has_unquoted_var("echo $?"));
        assert!(!line_has_unquoted_var("echo $#"));
    }

    // ── Classification export tests ─────────────────────────────────

    #[test]
    fn test_export_classification_jsonl() {
        let entry = make_entry("B-001", CorpusFormat::Bash);
        let result = make_result("B-001", true);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc1234", "2026-02-09");

        let output = export_classification_jsonl(&[row]);
        assert!(output.contains("\"input\""));
        assert!(output.contains("\"label\""));
        // Should be valid JSON
        let parsed: serde_json::Value =
            serde_json::from_str(&output).expect("classification JSONL should be valid JSON");
        assert!(parsed.get("input").is_some());
        assert!(parsed.get("label").is_some());
    }

    #[test]
    fn test_export_classification_jsonl_includes_failed_as_unsafe() {
        let entry = make_entry("B-002", CorpusFormat::Bash);
        let result = make_result("B-002", false);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc1234", "2026-02-09");

        let output = export_classification_jsonl(&[row]);
        assert!(
            !output.is_empty(),
            "Failed entries should be included as unsafe (label 1)"
        );
        let parsed: serde_json::Value = serde_json::from_str(&output).expect("valid JSON");
        assert_eq!(parsed.get("label").and_then(|v| v.as_u64()), Some(1));
    }

    #[test]
    fn test_export_classification_jsonl_multiple() {
        let rows: Vec<DatasetRow> = ["B-001", "B-002", "B-003"]
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let entry = make_entry(id, CorpusFormat::Bash);
                let result = make_result(id, i != 1); // B-002 fails
                build_row(&entry, Some(&result), "6.61.0", "abc", "2026-02-09")
            })
            .collect();

        let output = export_classification_jsonl(&rows);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(
            lines.len(),
            3,
            "All entries included (binary: safe=0, unsafe=1)"
        );
    }

    #[test]
    fn test_classification_row_serializes() {
        let cr = ClassificationRow {
            input: "#!/bin/sh\necho ok\n".into(),
            label: 0,
        };
        let json = serde_json::to_string(&cr).expect("should serialize");
        assert!(json.contains("\"input\""));
        assert!(json.contains("\"label\":0"));
    }

    #[test]
    fn test_safety_labels_count() {
        assert_eq!(SAFETY_LABELS.len(), 5);
        assert_eq!(SAFETY_LABELS[0], "safe");
        assert_eq!(SAFETY_LABELS[4], "unsafe");
    }

    #[test]
    fn test_build_row_includes_safety() {
        let entry = make_entry("B-001", CorpusFormat::Bash);
        let result = make_result("B-001", true);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc1234", "2026-02-09");

        // Transpiled, lint clean, deterministic, no unsafe patterns → safe
        assert_eq!(row.safety_label, "safe");
        assert_eq!(row.safety_index, 0);
    }

    #[test]
    fn test_build_row_failed_is_unsafe() {
        let entry = make_entry("B-002", CorpusFormat::Bash);
        let result = make_result("B-002", false);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc1234", "2026-02-09");

        assert_eq!(row.safety_label, "unsafe");
        assert_eq!(row.safety_index, 4);
    }

    #[test]
    fn test_csv_includes_safety_fields() {
        let entry = make_entry("B-001", CorpusFormat::Bash);
        let result = make_result("B-001", true);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc1234", "2026-02-09");

        let output = export_csv(&[row]);
        assert!(output.contains("safety_index"));
        assert!(output.contains("safety_label"));
    }

    // ── Multi-label classification tests (SSC-021) ──────────────────

    #[test]
    fn test_derive_multi_label_safe() {
        let labels = derive_multi_label("#!/bin/sh\necho \"hello\"\n", true, true, true);
        assert_eq!(
            labels,
            [1.0, 0.0, 0.0, 0.0, 0.0],
            "Clean script should be safe only"
        );
    }

    #[test]
    fn test_derive_multi_label_unsafe() {
        let labels = derive_multi_label("#!/bin/sh\necho hello\n", false, false, true);
        assert_eq!(labels[4], 1.0, "Not transpiled → unsafe");
    }

    #[test]
    fn test_derive_multi_label_nondet() {
        let labels = derive_multi_label("#!/bin/sh\necho \"hello\"\n", true, true, false);
        assert_eq!(labels[2], 1.0, "Non-deterministic should be set");
        assert_eq!(labels[0], 0.0, "Safe should NOT be set when nondet");
    }

    #[test]
    fn test_derive_multi_label_nonidempotent_and_unquoted() {
        let labels = derive_multi_label("mkdir $HOME/build\n", true, true, true);
        assert_eq!(labels[3], 1.0, "Non-idempotent pattern should be set");
        assert_eq!(labels[1], 1.0, "Needs-quoting should also be set");
        assert_eq!(labels[0], 0.0, "Safe should NOT be set");
    }

    #[test]
    fn test_derive_multi_label_multiple_issues() {
        // Not deterministic + has unquoted var → classes 1 and 2
        let labels = derive_multi_label("echo $HOME\n", true, true, false);
        assert_eq!(labels[1], 1.0, "Needs-quoting should be set");
        assert_eq!(labels[2], 1.0, "Non-deterministic should be set");
        assert_eq!(labels[0], 0.0, "Safe should NOT be set");
    }

    #[test]
    fn test_multi_label_row_serializes() {
        let ml = MultiLabelClassificationRow {
            input: "echo $HOME\n".into(),
            labels: [0.0, 1.0, 1.0, 0.0, 0.0],
        };
        let json = serde_json::to_string(&ml).expect("should serialize");
        assert!(json.contains("\"labels\""));
        assert!(json.contains("[0.0,1.0,1.0,0.0,0.0]"));
    }

    #[test]
    fn test_export_multi_label_classification_jsonl() {
        let entry = make_entry("B-001", CorpusFormat::Bash);
        let result = make_result("B-001", true);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc", "2026-02-09");

        let output = export_multi_label_classification_jsonl(&[row]);
        assert!(output.contains("\"input\""));
        assert!(output.contains("\"labels\""));
    }

    #[test]
    fn test_export_multi_label_skips_failed() {
        let entry = make_entry("B-002", CorpusFormat::Bash);
        let result = make_result("B-002", false);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc", "2026-02-09");

        let output = export_multi_label_classification_jsonl(&[row]);
        assert!(output.is_empty(), "Failed entries should not appear");
    }
}
