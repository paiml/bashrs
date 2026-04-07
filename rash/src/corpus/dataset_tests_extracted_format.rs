
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

    // ── validate_export tests ────────────────────────────────────────

    #[test]
    fn test_validate_export_all_classes_present() {
        let rows = vec![
            ClassificationRow { input: "echo safe".to_string(), label: 0 },
            ClassificationRow { input: "eval $x".to_string(), label: 1 },
        ];
        let v = validate_export(&rows, 2);
        assert!(v.passed, "Should pass with all classes present: {:?}", v.errors);
        assert_eq!(v.total, 2);
        assert_eq!(v.num_classes, 2);
    }

    #[test]
    fn test_validate_export_missing_class() {
        let rows = vec![
            ClassificationRow { input: "echo safe1".to_string(), label: 0 },
            ClassificationRow { input: "echo safe2".to_string(), label: 0 },
        ];
        let v = validate_export(&rows, 2);
        assert!(!v.passed, "Should fail with missing class 1");
        assert!(v.errors.iter().any(|e| e.contains("missing classes")));
    }

    #[test]
    fn test_validate_export_extreme_imbalance() {
        // 96 safe + 4 unsafe = 96% dominance -> error
        let mut rows: Vec<ClassificationRow> = (0..96)
            .map(|i| ClassificationRow { input: format!("safe_script_{i}"), label: 0 })
            .collect();
        rows.extend((0..4).map(|i| ClassificationRow {
            input: format!("unsafe_script_{i}"),
            label: 1,
        }));
        let v = validate_export(&rows, 2);
        assert!(!v.passed, "Should fail with >95% dominance");
        assert!(v.errors.iter().any(|e| e.contains("extreme class imbalance")));
    }

    #[test]
    fn test_validate_export_moderate_imbalance_warning() {
        // 90 safe + 10 unsafe = 90% dominance -> warning but passes
        let mut rows: Vec<ClassificationRow> = (0..90)
            .map(|i| ClassificationRow { input: format!("safe_code_{i}"), label: 0 })
            .collect();
        rows.extend((0..10).map(|i| ClassificationRow {
            input: format!("unsafe_code_{i}"),
            label: 1,
        }));
        let v = validate_export(&rows, 2);
        // May or may not pass depending on preamble check
        // But should have imbalance warning
        assert!(v.warnings.iter().any(|w| w.contains("class imbalance")));
    }

    #[test]
    fn test_validate_export_preamble_contamination() {
        let rows = vec![
            ClassificationRow { input: "#!/bin/sh\necho safe".to_string(), label: 0 },
            ClassificationRow { input: "eval $x".to_string(), label: 1 },
        ];
        let v = validate_export(&rows, 2);
        assert!(!v.passed, "Should fail with preamble contamination");
        assert!(v.errors.iter().any(|e| e.contains("preamble contamination")));
    }

    #[test]
    fn test_validate_export_preamble_set_euf() {
        let rows = vec![
            ClassificationRow { input: "set -euf pipefail\necho hi".to_string(), label: 0 },
            ClassificationRow { input: "bad cmd".to_string(), label: 1 },
        ];
        let v = validate_export(&rows, 2);
        assert!(!v.passed);
        assert!(v.errors.iter().any(|e| e.contains("preamble")));
    }

    #[test]
    fn test_validate_export_preamble_ifs() {
        let rows = vec![
            ClassificationRow { input: "IFS=' \\t\\n'\necho hi".to_string(), label: 0 },
            ClassificationRow { input: "bad cmd".to_string(), label: 1 },
        ];
        let v = validate_export(&rows, 2);
        assert!(!v.passed);
        assert!(v.errors.iter().any(|e| e.contains("preamble")));
    }

    #[test]
    fn test_validate_export_preamble_export_lc_all() {
        let rows = vec![
            ClassificationRow { input: "export LC_ALL=C\necho hi".to_string(), label: 0 },
            ClassificationRow { input: "bad cmd".to_string(), label: 1 },
        ];
        let v = validate_export(&rows, 2);
        assert!(!v.passed);
    }

    #[test]
    fn test_validate_export_trivial_inputs_warning() {
        let rows = vec![
            ClassificationRow { input: "ab".to_string(), label: 0 },
            ClassificationRow { input: "eval $dangerous_cmd".to_string(), label: 1 },
        ];
        let v = validate_export(&rows, 2);
        assert!(v.warnings.iter().any(|w| w.contains("trivial inputs")));
    }

    #[test]
    fn test_validate_export_length_confound_error() {
        // Class 0: very short inputs, class 1: very long inputs -> length confound
        let mut rows: Vec<ClassificationRow> = (0..10)
            .map(|i| ClassificationRow { input: format!("s{i}x"), label: 0 })
            .collect();
        rows.extend((0..10).map(|i| ClassificationRow {
            input: format!("x{}", "y".repeat(200 + i)),
            label: 1,
        }));
        let v = validate_export(&rows, 2);
        assert!(
            v.errors.iter().any(|e| e.contains("length confound"))
                || v.warnings.iter().any(|w| w.contains("length spread")),
            "Should detect length confound or spread: errors={:?} warnings={:?}",
            v.errors,
            v.warnings
        );
    }

    #[test]
    fn test_validate_export_length_spread_warning() {
        // Class 0: avg ~5 chars, class 1: avg ~30 chars -> 6x ratio -> warning
        let mut rows: Vec<ClassificationRow> = (0..10)
            .map(|i| ClassificationRow { input: format!("ab{i}cd"), label: 0 })
            .collect();
        rows.extend((0..10).map(|i| ClassificationRow {
            input: format!("x{}{i}", "z".repeat(30)),
            label: 1,
        }));
        let v = validate_export(&rows, 2);
        // 6x ratio should trigger warning (>5x) but not error (<10x)
        let has_length_issue = v.errors.iter().any(|e| e.contains("length"))
            || v.warnings.iter().any(|w| w.contains("length"));
        assert!(has_length_issue, "Should detect length spread");
    }

    #[test]
    fn test_validate_export_clean_passes() {
        let rows = vec![
            ClassificationRow { input: "echo hello world".to_string(), label: 0 },
            ClassificationRow { input: "echo goodbye world".to_string(), label: 0 },
            ClassificationRow { input: "eval dangerous cmd".to_string(), label: 1 },
            ClassificationRow { input: "eval another bad cmd".to_string(), label: 1 },
        ];
        let v = validate_export(&rows, 2);
        assert!(v.passed, "Clean data should pass: {:?}", v.errors);
    }

    // ── ExportValidation Display tests ───────────────────────────────

    #[test]
    fn test_export_validation_display_pass() {
        let v = ExportValidation {
            passed: true,
            total: 100,
            num_classes: 2,
            class_counts: [80, 20, 0, 0, 0],
            errors: vec![],
            warnings: vec![],
        };
        let display = format!("{v}");
        assert!(display.contains("PASS"));
        assert!(display.contains("100 samples"));
        assert!(display.contains("2 classes"));
        assert!(display.contains("Class 0:"));
        assert!(display.contains("Class 1:"));
    }

    #[test]
    fn test_export_validation_display_fail_with_errors() {
        let v = ExportValidation {
            passed: false,
            total: 50,
            num_classes: 1,
            class_counts: [50, 0, 0, 0, 0],
            errors: vec!["missing classes [1]".to_string()],
            warnings: vec!["trivial inputs: 2 samples have <3 chars".to_string()],
        };
        let display = format!("{v}");
        assert!(display.contains("FAIL"));
        assert!(display.contains("ERROR: missing classes"));
        assert!(display.contains("WARN: trivial inputs"));
    }

    // ── Split and SplitResult tests ──────────────────────────────────

    #[test]
    fn test_split_display() {
        assert_eq!(format!("{}", Split::Train), "train");
        assert_eq!(format!("{}", Split::Val), "val");
        assert_eq!(format!("{}", Split::Test), "test");
    }

    #[test]
    fn test_split_and_validate_basic() {
        let rows: Vec<ClassificationRow> = (0..100)
            .map(|i| ClassificationRow {
                input: format!("echo script number {i} with unique content"),
                label: if i % 5 == 0 { 1 } else { 0 },
            })
            .collect();
        let result = split_and_validate(rows, 2);
        let total = result.train.len() + result.val.len() + result.test.len();
        assert_eq!(total, 100);
        // Hash-based split should give roughly 80/10/10
        assert!(result.train.len() > 50, "Train should be majority: {}", result.train.len());
        assert!(result.val.len() > 0, "Val should have entries");
        assert!(result.test.len() > 0, "Test should have entries");
    }

    #[test]
    fn test_split_and_validate_deterministic() {
        let rows1: Vec<ClassificationRow> = (0..50)
            .map(|i| ClassificationRow {
                input: format!("deterministic_test_script_{i}"),
                label: if i % 3 == 0 { 1 } else { 0 },
            })
            .collect();
        let rows2 = rows1.clone();

        let result1 = split_and_validate(rows1, 2);
        let result2 = split_and_validate(rows2, 2);

        assert_eq!(result1.train.len(), result2.train.len());
        assert_eq!(result1.val.len(), result2.val.len());
        assert_eq!(result1.test.len(), result2.test.len());
    }

    #[test]
    fn test_split_result_display() {
        let rows: Vec<ClassificationRow> = (0..20)
            .map(|i| ClassificationRow {
                input: format!("display test script {i}"),
                label: if i % 4 == 0 { 1 } else { 0 },
            })
            .collect();
        let result = split_and_validate(rows, 2);
        let display = format!("{result}");
        assert!(display.contains("Split Result"));
        assert!(display.contains("train:"));
        assert!(display.contains("val:"));
        assert!(display.contains("test:"));
    }

    #[test]
    fn test_split_result_display_with_validation_errors() {
        // All same class -> missing class error
        let rows: Vec<ClassificationRow> = (0..10)
            .map(|i| ClassificationRow {
                input: format!("only safe script {i}"),
                label: 0,
            })
            .collect();
        let result = split_and_validate(rows, 2);
        let display = format!("{result}");
        assert!(display.contains("ERROR:") || display.contains("FAIL"));
    }

    // ── assign_split and fnv1a_hash tests ────────────────────────────

    #[test]
    fn test_assign_split_consistent() {
        let split1 = assign_split("echo hello world");
        let split2 = assign_split("echo hello world");
        assert_eq!(split1, split2);
    }

    #[test]
    fn test_assign_split_different_inputs_differ() {
        // Different inputs should map to different splits (probabilistic but
        // with enough diversity we expect at least 2 distinct splits)
        let inputs: Vec<String> = (0..100).map(|i| format!("unique script {i}")).collect();
        let mut has_train = false;
        let mut has_val = false;
        let mut has_test = false;
        for input in &inputs {
            match assign_split(input) {
                Split::Train => has_train = true,
                Split::Val => has_val = true,
                Split::Test => has_test = true,
            }
        }
        let distinct = [has_train, has_val, has_test].iter().filter(|&&b| b).count();
        assert!(distinct >= 2, "Should produce at least 2 distinct splits with 100 inputs");
    }

    #[test]
    fn test_fnv1a_hash_deterministic() {
        let h1 = fnv1a_hash(b"test input");
        let h2 = fnv1a_hash(b"test input");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_fnv1a_hash_different_inputs() {
        let h1 = fnv1a_hash(b"hello");
        let h2 = fnv1a_hash(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_fnv1a_hash_empty() {
        let h = fnv1a_hash(b"");
        // FNV-1a with empty input is just the offset basis
        assert_eq!(h, 0xcbf29ce484222325);
    }

    // ── strip_shell_preamble tests ───────────────────────────────────

    #[test]
    fn test_strip_shell_preamble_removes_shebang() {
        let script = "#!/bin/sh\necho hello\n";
        let stripped = strip_shell_preamble(script);
        assert!(!stripped.contains("#!/bin/sh"));
        assert!(stripped.contains("echo hello"));
    }

    #[test]
    fn test_strip_shell_preamble_removes_set() {
        let script = "#!/bin/sh\nset -euf\necho hello\n";
        let stripped = strip_shell_preamble(script);
        assert!(!stripped.contains("set -euf"));
        assert!(stripped.contains("echo hello"));
    }

    #[test]
    fn test_strip_shell_preamble_removes_main_wrapper() {
        let script = "#!/bin/sh\nmain() {\n  echo hello\n}\nmain \"$@\"\n";
        let stripped = strip_shell_preamble(script);
        assert!(!stripped.contains("main()"));
        assert!(!stripped.contains("main \"$@\""));
        assert!(stripped.contains("echo hello"));
    }

    #[test]
    fn test_strip_shell_preamble_preserves_body() {
        let script = "echo line1\necho line2\n";
        let stripped = strip_shell_preamble(script);
        assert!(stripped.contains("echo line1"));
        assert!(stripped.contains("echo line2"));
    }

    #[test]
    fn test_strip_shell_preamble_empty_input() {
        // All lines are preamble -> returns original
        let script = "#!/bin/sh\n# comment\n";
        let stripped = strip_shell_preamble(script);
        // Should fallback to original since stripping produces empty body
        assert!(!stripped.is_empty());
    }

    #[test]
    fn test_strip_shell_preamble_removes_trap() {
        let script = "#!/bin/sh\ntrap 'rm -rf /tmp/x' EXIT\necho hi\n";
        let stripped = strip_shell_preamble(script);
        assert!(!stripped.contains("trap"));
        assert!(stripped.contains("echo hi"));
    }

    #[test]
    fn test_strip_shell_preamble_removes_export() {
        let script = "export LC_ALL=C\necho hello\n";
        let stripped = strip_shell_preamble(script);
        assert!(!stripped.contains("export"));
        assert!(stripped.contains("echo hello"));
    }

    #[test]
    fn test_strip_shell_preamble_removes_ifs() {
        let script = "IFS=' \\t\\n'\necho hello\n";
        let stripped = strip_shell_preamble(script);
        assert!(!stripped.contains("IFS="));
        assert!(stripped.contains("echo hello"));
    }

    // ── is_shell_preamble tests ──────────────────────────────────────

    #[test]
    fn test_is_shell_preamble_empty() {
        assert!(is_shell_preamble(""));
    }

    #[test]
    fn test_is_shell_preamble_comment() {
        assert!(is_shell_preamble("# Generated by Rash"));
        assert!(is_shell_preamble("#!/bin/sh"));
    }

    #[test]
    fn test_is_shell_preamble_set() {
        assert!(is_shell_preamble("set -euf"));
    }

    #[test]
    fn test_is_shell_preamble_main_call() {
        assert!(is_shell_preamble("main \"$@\""));
    }

    #[test]
    fn test_is_shell_preamble_not_preamble() {
        assert!(!is_shell_preamble("echo hello"));
        assert!(!is_shell_preamble("mkdir -p /tmp/build"));
        assert!(!is_shell_preamble("for i in 1 2 3; do"));
    }

    // ── classify_single tests ────────────────────────────────────────

    #[test]
    fn test_classify_single_safe() {
        let cr = classify_single("echo hello world", true, true, true);
        assert_eq!(cr.label, 0);
        assert!(!cr.input.is_empty());
    }

    #[test]
    fn test_classify_single_unsafe_not_transpiled() {
        let cr = classify_single("echo hello", false, true, true);
        assert_eq!(cr.label, 1);
    }

    #[test]
    fn test_classify_single_unsafe_not_lint_clean() {
        let cr = classify_single("echo hello", true, false, true);
        assert_eq!(cr.label, 1);
    }

    #[test]
    fn test_classify_single_unsafe_not_deterministic() {
        let cr = classify_single("echo hello", true, true, false);
        assert_eq!(cr.label, 1);
    }

    #[test]
    fn test_classify_single_strips_preamble() {
        let cr = classify_single("#!/bin/sh\nset -euf\necho hello", true, true, true);
        assert!(!cr.input.contains("#!/bin/sh"));
        assert!(!cr.input.contains("set -euf"));
        assert!(cr.input.contains("echo hello"));
    }

    // ── line_has_unquoted_var edge cases ─────────────────────────────

    #[test]
    fn test_line_has_unquoted_var_escaped_dollar() {
        assert!(!line_has_unquoted_var("echo \\$HOME"));
    }

    #[test]
    fn test_line_has_unquoted_var_in_double_quotes() {
        assert!(!line_has_unquoted_var("echo \"$HOME is here\""));
    }

    #[test]
    fn test_line_has_unquoted_var_mixed_quotes() {
        // Outside quotes: $USER should be detected
        assert!(line_has_unquoted_var("echo '$HOME' $USER"));
    }

    #[test]
    fn test_line_has_unquoted_var_brace_form() {
        assert!(line_has_unquoted_var("echo ${HOME}"));
        assert!(!line_has_unquoted_var("echo \"${HOME}\""));
    }

    #[test]
    fn test_line_has_unquoted_var_underscore_prefix() {
        assert!(line_has_unquoted_var("echo $_VAR"));
    }
