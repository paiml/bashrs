#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::Span;

    fn sample_lint_result() -> LintResult {
        LintResult {
            diagnostics: vec![
                Diagnostic {
                    code: "SC2086".to_string(),
                    severity: Severity::Warning,
                    message: "Double quote to prevent globbing".to_string(),
                    span: Span::new(10, 1, 10, 20),
                    fix: Some(crate::linter::Fix::new("\"$var\"")),
                },
                Diagnostic {
                    code: "SC2086".to_string(),
                    severity: Severity::Warning,
                    message: "Double quote to prevent globbing".to_string(),
                    span: Span::new(15, 1, 15, 20),
                    fix: Some(crate::linter::Fix::new("\"$other\"")),
                },
                Diagnostic {
                    code: "DET001".to_string(),
                    severity: Severity::Warning,
                    message: "Non-deterministic $RANDOM".to_string(),
                    span: Span::new(20, 5, 20, 15),
                    fix: None,
                },
                Diagnostic {
                    code: "SEC010".to_string(),
                    severity: Severity::Error,
                    message: "Hardcoded path /tmp".to_string(),
                    span: Span::new(25, 1, 25, 10),
                    fix: Some(crate::linter::Fix::new("${TMPDIR:-/tmp}")),
                },
            ],
        }
    }

    #[test]
    fn test_ml_013_histogram_bar() {
        let bar = histogram_bar(50.0, 100.0, 10);
        assert_eq!(bar.chars().count(), 10);
        assert!(bar.contains('█'));
        assert!(bar.contains('░'));

        let full_bar = histogram_bar(100.0, 100.0, 10);
        assert!(!full_bar.contains('░'));

        let empty_bar = histogram_bar(0.0, 100.0, 10);
        assert!(!empty_bar.contains('█'));
    }

    #[test]
    fn test_ml_014_rich_report_creation() {
        let result = sample_lint_result();
        let report = RichLintReport::from_lint_result("test.sh", &result, "echo $var");

        assert_eq!(report.total_issues, 4);
        assert_eq!(report.errors, 1);
        assert_eq!(report.warnings, 3);
        assert!(!report.clusters.is_empty());
    }

    #[test]
    fn test_ml_014_rich_report_clustering() {
        let result = sample_lint_result();
        let report = RichLintReport::from_lint_result("test.sh", &result, "echo $var");

        // SC2086 appears twice, should be biggest cluster
        let sc2086 = report.clusters.iter().find(|c| c.error_code == "SC2086");
        assert!(sc2086.is_some());
        assert_eq!(sc2086.expect("found").count, 2);
    }

    #[test]
    fn test_ml_014_rich_report_render() {
        let result = sample_lint_result();
        let report = RichLintReport::from_lint_result("test.sh", &result, "echo $var");

        let rendered = report.render(80);

        assert!(rendered.contains("BASHRS LINT REPORT"));
        assert!(rendered.contains("test.sh"));
        assert!(rendered.contains("SUMMARY"));
        assert!(rendered.contains("ERROR CLUSTERS"));
        assert!(rendered.contains("SC2086"));
    }

    #[test]
    fn test_ml_006_sbfl_report() {
        use crate::quality::sbfl::CoverageData;

        let rankings = vec![
            SuspiciousnessRanking {
                location: "SC2086".to_string(),
                score: 0.94,
                coverage: CoverageData::new(2, 31, 8, 0),
                rank: 1,
            },
            SuspiciousnessRanking {
                location: "DET001".to_string(),
                score: 0.72,
                coverage: CoverageData::new(8, 12, 2, 0),
                rank: 2,
            },
        ];

        let report = sbfl_report(&rankings, SbflFormula::Ochiai, 80);

        assert!(report.contains("FAULT LOCALIZATION"));
        assert!(report.contains("Ochiai"));
        assert!(report.contains("SC2086"));
        assert!(report.contains("DET001"));
    }

    #[test]
    fn test_error_cluster_percentage() {
        let cluster = ErrorCluster {
            error_code: "SC2086".to_string(),
            count: 25,
            category: ShellErrorCategory::MissingQuotes,
            diagnostics: vec![],
            fix_confidence: 0.94,
            auto_fixable: true,
        };

        assert!((cluster.percentage(100) - 25.0).abs() < 0.01);
        assert!((cluster.percentage(50) - 50.0).abs() < 0.01);
        assert!((cluster.percentage(0) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_ml_014_auto_fixable_counting() {
        let result = sample_lint_result();
        let report = RichLintReport::from_lint_result("test.sh", &result, "echo $var");

        // 3 diagnostics have fixes, 1 doesn't
        assert_eq!(report.auto_fixable_count, 3);
        assert_eq!(report.manual_count, 1);
    }

    #[test]
    fn test_ml_014_confidence_calculation() {
        let result = sample_lint_result();
        let report = RichLintReport::from_lint_result("test.sh", &result, "echo $var");

        // Confidence should be between 0 and 1
        assert!(report.overall_confidence >= 0.0);
        assert!(report.overall_confidence <= 1.0);
    }
}
