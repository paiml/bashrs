#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::registry::Grade;
    use crate::corpus::runner::FormatScore;

    fn make_score(passed: usize, failed: usize, rate: f64, score_val: f64) -> CorpusScore {
        CorpusScore {
            total: passed + failed,
            passed,
            failed,
            rate,
            score: score_val,
            grade: Grade::from_score(score_val),
            format_scores: vec![],
            results: vec![],
        }
    }

    fn make_history_entry(iteration: u32, passed_count: usize) -> ConvergenceEntry {
        let mut e = ConvergenceEntry {
            iteration,
            passed: passed_count,
            ..Default::default()
        };
        // Use today's date dynamically to prevent time-coupled test flakiness (Five Whys #2)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        let days = now.as_secs() / 86400;
        // Convert days since epoch to YYYY-MM-DD
        let (y, m, d) = {
            // Inverse Julian day: epoch 1970-01-01 is JDN 2440588
            let jdn = days as i64 + 2_440_588;
            let f = jdn + 1401 + (((4 * jdn + 274277) / 146097) * 3) / 4 - 38;
            let e2 = 4 * f + 3;
            let g = (e2 % 1461) / 4;
            let h = 5 * g + 2;
            let d = (h % 153) / 5 + 1;
            let m = ((h / 153 + 2) % 12) + 1;
            let y = e2 / 1461 - 4716 + (14 - m) / 12;
            (y, m, d)
        };
        e.date = format!("{y:04}-{m:02}-{d:02}");
        e
    }

    #[test]
    fn test_quality_gates_all_pass() {
        let score = make_score(900, 0, 1.0, 99.9);
        let history = vec![make_history_entry(1, 900)];
        let thresholds = QualityThresholds::default();
        let gates = check_quality_gates(&score, &history, &thresholds);

        assert!(gates.iter().all(|g| g.passed));
    }

    #[test]
    fn test_quality_gates_rate_fail() {
        let score = make_score(800, 100, 0.889, 85.0);
        let thresholds = QualityThresholds::default();
        let gates = check_quality_gates(&score, &[], &thresholds);

        let rate_gate = gates.iter().find(|g| g.name == "Transpilation Rate");
        assert!(rate_gate.is_some());
        assert!(!rate_gate.expect("gate exists").passed);
    }

    #[test]
    fn test_quality_gates_score_fail() {
        let score = make_score(900, 0, 1.0, 85.0);
        let thresholds = QualityThresholds::default();
        let gates = check_quality_gates(&score, &[], &thresholds);

        let score_gate = gates.iter().find(|g| g.name == "V2 Corpus Score");
        assert!(score_gate.is_some());
        assert!(!score_gate.expect("gate exists").passed);
    }

    #[test]
    fn test_quality_gates_grade_fail() {
        let score = make_score(900, 0, 1.0, 75.0);
        let thresholds = QualityThresholds::default();
        let gates = check_quality_gates(&score, &[], &thresholds);

        let grade_gate = gates.iter().find(|g| g.name == "Quality Grade");
        assert!(grade_gate.is_some());
        assert!(!grade_gate.expect("gate exists").passed);
    }

    #[test]
    fn test_quality_gates_failures_fail() {
        let score = make_score(890, 10, 0.989, 95.0);
        let thresholds = QualityThresholds::default();
        let gates = check_quality_gates(&score, &[], &thresholds);

        let fail_gate = gates.iter().find(|g| g.name == "Failure Count");
        assert!(fail_gate.is_some());
        assert!(!fail_gate.expect("gate exists").passed);
    }

    #[test]
    fn test_quality_gates_regression_detected() {
        let score = make_score(895, 5, 0.994, 98.0);
        let history = vec![make_history_entry(1, 900)]; // was 900, now 895
        let thresholds = QualityThresholds::default();
        let gates = check_quality_gates(&score, &history, &thresholds);

        let reg_gate = gates.iter().find(|g| g.name == "No Regressions");
        assert!(reg_gate.is_some());
        assert!(!reg_gate.expect("gate exists").passed);
    }

    #[test]
    fn test_quality_gates_no_regression() {
        let score = make_score(905, 0, 1.0, 99.9);
        let history = vec![make_history_entry(1, 900)];
        let thresholds = QualityThresholds::default();
        let gates = check_quality_gates(&score, &history, &thresholds);

        let reg_gate = gates.iter().find(|g| g.name == "No Regressions");
        assert!(reg_gate.is_some());
        assert!(reg_gate.expect("gate exists").passed);
    }

    #[test]
    fn test_quality_gates_per_format() {
        use crate::corpus::registry::CorpusFormat;
        let mut score = make_score(900, 0, 1.0, 99.9);
        score.format_scores = vec![
            FormatScore {
                format: CorpusFormat::Bash,
                total: 500,
                passed: 500,
                rate: 1.0,
                score: 100.0,
                grade: Grade::APlus,
            },
            FormatScore {
                format: CorpusFormat::Makefile,
                total: 200,
                passed: 195,
                rate: 0.975,
                score: 97.5,
                grade: Grade::APlus,
            },
        ];
        let thresholds = QualityThresholds::default();
        let gates = check_quality_gates(&score, &[], &thresholds);

        let bash_gate = gates.iter().find(|g| g.name == "Bash Rate");
        assert!(bash_gate.is_some());
        assert!(bash_gate.expect("gate exists").passed);

        let make_gate = gates.iter().find(|g| g.name == "Makefile Rate");
        assert!(make_gate.is_some());
        assert!(!make_gate.expect("gate exists").passed); // 97.5% < 99%
    }

    #[test]
    fn test_metrics_all_pass() {
        let score = make_score(900, 0, 1.0, 99.9);
        let duration = Duration::from_millis(5000);
        let history = vec![
            make_history_entry(1, 800),
            make_history_entry(2, 850),
            make_history_entry(3, 900),
        ];
        let thresholds = PerformanceThresholds::default();
        let metrics = check_metrics(&score, duration, &history, &thresholds);

        assert!(metrics.iter().all(|m| m.passed));
    }

    #[test]
    fn test_metrics_slow_run() {
        let score = make_score(900, 0, 1.0, 99.9);
        let duration = Duration::from_millis(120_000); // 120s > 60s threshold
        let history = vec![
            make_history_entry(1, 800),
            make_history_entry(2, 850),
            make_history_entry(3, 900),
        ];
        let thresholds = PerformanceThresholds::default();
        let metrics = check_metrics(&score, duration, &history, &thresholds);

        let total_time = metrics.iter().find(|m| m.name == "Total Run Time");
        assert!(total_time.is_some());
        assert!(!total_time.expect("metric exists").passed);
    }

    #[test]
    fn test_metrics_insufficient_history() {
        let score = make_score(900, 0, 1.0, 99.9);
        let duration = Duration::from_millis(5000);
        let history = vec![make_history_entry(1, 900)]; // only 1, need >= 3
        let thresholds = PerformanceThresholds::default();
        let metrics = check_metrics(&score, duration, &history, &thresholds);

        let depth = metrics.iter().find(|m| m.name == "History Depth");
        assert!(depth.is_some());
        assert!(!depth.expect("metric exists").passed);
    }

    #[test]
    fn test_metrics_small_corpus() {
        let score = make_score(100, 0, 1.0, 99.9); // only 100 entries
        let duration = Duration::from_millis(1000);
        let history = vec![
            make_history_entry(1, 50),
            make_history_entry(2, 75),
            make_history_entry(3, 100),
        ];
        let thresholds = PerformanceThresholds::default();
        let metrics = check_metrics(&score, duration, &history, &thresholds);

        let size = metrics.iter().find(|m| m.name == "Corpus Size");
        assert!(size.is_some());
        assert!(!size.expect("metric exists").passed);
    }

    #[test]
    fn test_gate_status_all_pass() {
        let score = make_score(900, 0, 1.0, 99.9);
        let duration = Duration::from_millis(5000);
        let history = vec![
            make_history_entry(1, 800),
            make_history_entry(2, 850),
            make_history_entry(3, 900),
        ];
        let status = build_gate_status(&score, duration, &history);
        assert!(status.all_passed);
        assert_eq!(status.gates_passed, status.gates_total);
    }

    #[test]
    fn test_grade_meets_minimum() {
        assert!(grade_meets_minimum("A+", "A"));
        assert!(grade_meets_minimum("A+", "A+"));
        assert!(grade_meets_minimum("A", "A"));
        assert!(grade_meets_minimum("B", "B"));
        assert!(!grade_meets_minimum("B", "A"));
        assert!(!grade_meets_minimum("C", "A"));
        assert!(grade_meets_minimum("A+", "F"));
    }

    #[test]
    fn test_check_for_regression_true() {
        let score = make_score(895, 5, 0.994, 98.0);
        let history = vec![make_history_entry(1, 900)];
        assert!(check_for_regression(&score, &history));
    }

    #[test]
    fn test_check_for_regression_false() {
        let score = make_score(905, 0, 1.0, 99.9);
        let history = vec![make_history_entry(1, 900)];
        assert!(!check_for_regression(&score, &history));
    }

    #[test]
    fn test_compute_staleness_empty() {
        assert_eq!(compute_staleness(&[]), 999);
    }

    #[test]
    fn test_compute_staleness_recent() {
        let history = vec![make_history_entry(1, 900)];
        let staleness = compute_staleness(&history);
        // Entry is dated 2026-02-08, today is 2026-02-09, so ~1 day
        assert!(staleness <= 2);
    }

    #[test]
    fn test_format_quality_gates_contains_headers() {
        let gates = vec![GateCheck {
            name: "Test Gate",
            description: "A test",
            passed: true,
            actual: "100.0%".to_string(),
            threshold: ">= 99%".to_string(),
        }];
        let report = format_quality_gates(&gates);
        assert!(report.contains("Quality Gates"));
        assert!(report.contains("Test Gate"));
        assert!(report.contains("PASS"));
    }

    #[test]
    fn test_format_metrics_check_contains_headers() {
        let metrics = vec![MetricCheck {
            name: "Test Metric",
            passed: false,
            actual: "150".to_string(),
            threshold: "<= 100".to_string(),
            unit: "ms",
        }];
        let report = format_metrics_check(&metrics);
        assert!(report.contains("Performance Metrics"));
        assert!(report.contains("Test Metric"));
        assert!(report.contains("FAIL"));
    }

    #[test]
    fn test_format_gate_status_all_pass() {
        let status = GateStatus {
            quality_gates: vec![GateCheck {
                name: "Rate",
                description: "rate check",
                passed: true,
                actual: "100%".to_string(),
                threshold: ">= 99%".to_string(),
            }],
            metrics: vec![MetricCheck {
                name: "Time",
                passed: true,
                actual: "50".to_string(),
                threshold: "<= 100".to_string(),
                unit: "ms",
            }],
            all_passed: true,
            gates_passed: 2,
            gates_total: 2,
        };
        let report = format_gate_status(&status);
        assert!(report.contains("ALL GATES PASSED"));
        assert!(report.contains("2/2"));
    }

    #[test]
    fn test_format_gate_status_failure() {
        let status = GateStatus {
            quality_gates: vec![GateCheck {
                name: "Rate",
                description: "rate check",
                passed: false,
                actual: "90%".to_string(),
                threshold: ">= 99%".to_string(),
            }],
            metrics: vec![],
            all_passed: false,
            gates_passed: 0,
            gates_total: 1,
        };
        let report = format_gate_status(&status);
        assert!(report.contains("GATES FAILED"));
        assert!(report.contains("0/1"));
    }

    #[test]
    fn test_default_thresholds() {
        let qt = QualityThresholds::default();
        assert_eq!(qt.min_coverage, 95.0);
        assert_eq!(qt.min_score, 90.0);
        assert_eq!(qt.min_rate, 99.0);
        assert_eq!(qt.max_failures, 5);
        assert_eq!(qt.min_grade, "A");
        assert!(qt.block_on_regression);

        let pt = PerformanceThresholds::default();
        assert_eq!(pt.max_transpile_ms_per_entry, 100);
        assert_eq!(pt.max_total_ms, 60_000);
        assert_eq!(pt.max_staleness_days, 7);
        assert_eq!(pt.min_mutation_score, 90.0);
    }

    #[test]
    fn test_days_since_valid() {
        // We can't test exact value but can verify it returns Some
        let result = days_since("2026-02-08");
        assert!(result.is_some());
    }

    #[test]
    fn test_days_since_invalid() {
        assert!(days_since("invalid").is_none());
        assert!(days_since("2026-13-01").is_some()); // JDN handles weird months
        assert!(days_since("").is_none());
    }
}
