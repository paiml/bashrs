#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_entry(
        iteration: u32,
        bash: (usize, usize),
        makefile: (usize, usize),
        dockerfile: (usize, usize),
        score: f64,
        grade: &str,
    ) -> ConvergenceEntry {
        let total = bash.1 + makefile.1 + dockerfile.1;
        let passed = bash.0 + makefile.0 + dockerfile.0;
        ConvergenceEntry {
            iteration,
            date: "2026-02-08".to_string(),
            total,
            passed,
            failed: total - passed,
            rate: pass_rate(passed, total),
            delta: 0.0,
            notes: String::new(),
            bash_passed: bash.0,
            bash_total: bash.1,
            makefile_passed: makefile.0,
            makefile_total: makefile.1,
            dockerfile_passed: dockerfile.0,
            dockerfile_total: dockerfile.1,
            score,
            grade: grade.to_string(),
            bash_score: 0.0,
            makefile_score: 0.0,
            dockerfile_score: 0.0,
            lint_passed: 0,
            lint_rate: 0.0,
        }
    }

    #[test]
    fn test_convergence_table_renders_header() {
        let entries = vec![make_entry(
            1,
            (500, 500),
            (200, 200),
            (200, 200),
            99.9,
            "A+",
        )];
        let table = format_convergence_table(&entries);
        assert!(table.contains("Multi-Corpus Convergence Table"));
        assert!(table.contains("Bash (500)"));
        assert!(table.contains("Makefile (200)"));
        assert!(table.contains("Dockerfile (200)"));
        assert!(table.contains("500/500"));
        assert!(table.contains("200/200"));
        assert!(table.contains("99.9"));
        assert!(table.contains("A+"));
    }

    #[test]
    fn test_convergence_table_multiple_entries() {
        let entries = vec![
            make_entry(1, (490, 500), (195, 200), (198, 200), 98.0, "A"),
            make_entry(2, (500, 500), (200, 200), (200, 200), 99.9, "A+"),
        ];
        let table = format_convergence_table(&entries);
        assert!(table.contains("#1"));
        assert!(table.contains("#2"));
        assert!(table.contains("490/500"));
        assert!(table.contains("500/500"));
        assert!(table.contains("2 iterations total"));
    }

    #[test]
    fn test_convergence_table_empty() {
        let entries: Vec<ConvergenceEntry> = vec![];
        let table = format_convergence_table(&entries);
        assert!(table.contains("0 iterations total"));
    }

    #[test]
    fn test_convergence_table_legacy_entry_no_format_data() {
        let mut entry = make_entry(1, (0, 0), (0, 0), (0, 0), 95.0, "A");
        entry.passed = 800;
        entry.total = 900;
        let table = format_convergence_table(&[entry]);
        assert!(table.contains("800/900"));
    }

    #[test]
    fn test_convergence_status_all_stable() {
        let entries = vec![
            make_entry(1, (500, 500), (200, 200), (200, 200), 99.9, "A+"),
            make_entry(2, (500, 500), (200, 200), (200, 200), 99.9, "A+"),
            make_entry(3, (500, 500), (200, 200), (200, 200), 99.9, "A+"),
        ];
        let statuses = convergence_status(&entries);
        assert_eq!(statuses.len(), 3);
        for s in &statuses {
            assert_eq!(s.trend, Trend::Stable);
            assert_eq!(s.iterations_stable, 3);
            assert!((s.current_rate - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn test_convergence_status_improving() {
        let entries = vec![
            make_entry(1, (490, 500), (200, 200), (200, 200), 98.0, "A"),
            make_entry(2, (495, 500), (200, 200), (200, 200), 99.0, "A"),
            make_entry(3, (500, 500), (200, 200), (200, 200), 99.9, "A+"),
        ];
        let statuses = convergence_status(&entries);
        let bash = &statuses[0];
        assert_eq!(bash.trend, Trend::Improving);
    }

    #[test]
    fn test_convergence_status_regressing() {
        let entries = vec![
            make_entry(1, (500, 500), (200, 200), (200, 200), 99.9, "A+"),
            make_entry(2, (500, 500), (200, 200), (200, 200), 99.9, "A+"),
            make_entry(3, (500, 500), (195, 200), (200, 200), 99.0, "A"),
        ];
        let statuses = convergence_status(&entries);
        let makefile = &statuses[1];
        assert_eq!(makefile.trend, Trend::Regressing);
    }

    #[test]
    fn test_convergence_status_empty() {
        let entries: Vec<ConvergenceEntry> = vec![];
        let statuses = convergence_status(&entries);
        assert_eq!(statuses.len(), 3);
        for s in &statuses {
            assert_eq!(s.trend, Trend::Stable);
            assert!((s.current_rate - 0.0).abs() < 1e-9);
        }
    }

    #[test]
    fn test_convergence_status_single_entry() {
        let entries = vec![make_entry(
            1,
            (500, 500),
            (200, 200),
            (200, 200),
            99.9,
            "A+",
        )];
        let statuses = convergence_status(&entries);
        for s in &statuses {
            assert_eq!(s.trend, Trend::Stable);
            assert_eq!(s.iterations_stable, 1);
        }
    }

    #[test]
    fn test_compare_iterations_no_change() {
        let a = make_entry(1, (500, 500), (200, 200), (200, 200), 99.9, "A+");
        let b = make_entry(2, (500, 500), (200, 200), (200, 200), 99.9, "A+");
        let diff = compare_iterations(&a, &b);
        assert_eq!(diff.from_iter, 1);
        assert_eq!(diff.to_iter, 2);
        assert!(diff.bash_delta.abs() < 1e-9);
        assert!(diff.makefile_delta.abs() < 1e-9);
        assert!(diff.dockerfile_delta.abs() < 1e-9);
        assert!(diff.total_delta.abs() < 1e-9);
        assert!(diff.score_delta.abs() < 1e-9);
    }

    #[test]
    fn test_compare_iterations_with_change() {
        let a = make_entry(1, (490, 500), (190, 200), (195, 200), 95.0, "A");
        let b = make_entry(2, (500, 500), (200, 200), (200, 200), 99.9, "A+");
        let diff = compare_iterations(&a, &b);
        assert!((diff.bash_delta - 2.0).abs() < 0.01); // 98% -> 100% = +2pp
        assert!((diff.makefile_delta - 5.0).abs() < 0.01); // 95% -> 100% = +5pp
        assert!((diff.dockerfile_delta - 2.5).abs() < 0.01); // 97.5% -> 100% = +2.5pp
        assert!((diff.score_delta - 4.9).abs() < 0.01);
    }

    #[test]
    fn test_format_delta_zero() {
        assert_eq!(format_delta(0.0), "\u{2192} 0.0%");
    }

    #[test]
    fn test_format_delta_positive() {
        let s = format_delta(2.5);
        assert!(s.contains("+2.5%"));
    }

    #[test]
    fn test_format_delta_negative() {
        let s = format_delta(-1.3);
        assert!(s.contains("-1.3%"));
    }

    #[test]
    fn test_format_score_delta_zero() {
        assert_eq!(format_score_delta(0.0), "\u{2192} 0.0");
    }

    #[test]
    fn test_format_iteration_diff_table() {
        let a = make_entry(5, (500, 500), (200, 200), (200, 200), 99.9, "A+");
        let b = make_entry(6, (500, 500), (200, 200), (200, 200), 99.9, "A+");
        let diff = compare_iterations(&a, &b);
        let table = format_iteration_diff(&diff);
        assert!(table.contains("Convergence Diff"));
        assert!(table.contains("#5"));
        assert!(table.contains("#6"));
        assert!(table.contains("Bash"));
        assert!(table.contains("Makefile"));
        assert!(table.contains("Dockerfile"));
        assert!(table.contains("Total"));
        assert!(table.contains("Score"));
    }

    #[test]
    fn test_format_convergence_status_converged() {
        let statuses = vec![
            FormatConvergenceStatus {
                format: "Bash",
                current_rate: 1.0,
                trend: Trend::Stable,
                iterations_stable: 3,
            },
            FormatConvergenceStatus {
                format: "Makefile",
                current_rate: 1.0,
                trend: Trend::Stable,
                iterations_stable: 3,
            },
            FormatConvergenceStatus {
                format: "Dockerfile",
                current_rate: 1.0,
                trend: Trend::Stable,
                iterations_stable: 3,
            },
        ];
        let output = format_convergence_status(&statuses);
        assert!(output.contains("CONVERGED"));
        assert!(output.contains("100.0%"));
        assert!(output.contains("Stable"));
    }

    #[test]
    fn test_format_convergence_status_regressing() {
        let statuses = vec![
            FormatConvergenceStatus {
                format: "Bash",
                current_rate: 0.98,
                trend: Trend::Regressing,
                iterations_stable: 1,
            },
            FormatConvergenceStatus {
                format: "Makefile",
                current_rate: 1.0,
                trend: Trend::Stable,
                iterations_stable: 3,
            },
            FormatConvergenceStatus {
                format: "Dockerfile",
                current_rate: 1.0,
                trend: Trend::Stable,
                iterations_stable: 3,
            },
        ];
        let output = format_convergence_status(&statuses);
        assert!(output.contains("REGRESSING"));
        assert!(output.contains("Bash"));
    }

    #[test]
    fn test_format_convergence_status_improving() {
        let statuses = vec![
            FormatConvergenceStatus {
                format: "Bash",
                current_rate: 0.99,
                trend: Trend::Improving,
                iterations_stable: 1,
            },
            FormatConvergenceStatus {
                format: "Makefile",
                current_rate: 1.0,
                trend: Trend::Stable,
                iterations_stable: 2,
            },
            FormatConvergenceStatus {
                format: "Dockerfile",
                current_rate: 1.0,
                trend: Trend::Stable,
                iterations_stable: 2,
            },
        ];
        let output = format_convergence_status(&statuses);
        assert!(output.contains("IMPROVING"));
    }

    #[test]
    fn test_detect_trend_stable_pair() {
        let rates = vec![1.0, 1.0];
        let (trend, count) = detect_trend(&rates);
        assert_eq!(trend, Trend::Stable);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_detect_trend_improving() {
        let rates = vec![0.9, 0.95, 1.0];
        let (trend, _) = detect_trend(&rates);
        assert_eq!(trend, Trend::Improving);
    }

    #[test]
    fn test_detect_trend_regressing() {
        let rates = vec![1.0, 0.95, 0.9];
        let (trend, _) = detect_trend(&rates);
        assert_eq!(trend, Trend::Regressing);
    }

    #[test]
    fn test_detect_trend_single() {
        let rates = vec![0.99];
        let (trend, count) = detect_trend(&rates);
        assert_eq!(trend, Trend::Stable);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_detect_trend_empty() {
        let rates: Vec<f64> = vec![];
        let (trend, count) = detect_trend(&rates);
        assert_eq!(trend, Trend::Stable);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_detect_trend_stable_after_change() {
        let rates = vec![0.9, 1.0, 1.0, 1.0];
        let (trend, count) = detect_trend(&rates);
        assert_eq!(trend, Trend::Stable);
        assert_eq!(count, 3); // last 3 values are identical (including anchor)
    }

    #[test]
    fn test_pass_rate_zero_total() {
        assert!((pass_rate(0, 0) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_pass_rate_normal() {
        assert!((pass_rate(490, 500) - 0.98).abs() < 1e-9);
    }

    #[test]
    fn test_trend_display() {
        assert_eq!(format!("{}", Trend::Improving), "Improving");
        assert_eq!(format!("{}", Trend::Stable), "Stable");
        assert_eq!(format!("{}", Trend::Regressing), "Regressing");
    }
}
