#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explain_safe_script() {
        let report = generate_explanation("#!/bin/sh\necho hello\n", &ClassifyFormat::Bash);
        assert_eq!(report.risk_level, "safe");
        assert!(report.categories.is_empty());
        assert!(report.summary.contains("passes all safety checks"));
    }

    #[test]
    fn test_explain_eval_security() {
        let report =
            generate_explanation("#!/bin/bash\neval \"$user_input\"\n", &ClassifyFormat::Bash);
        assert_eq!(report.risk_level, "high");
        assert!(report.categories.iter().any(|c| c.category == "Security"));
        let sec = report
            .categories
            .iter()
            .find(|c| c.category == "Security")
            .expect("should have security category");
        assert!(sec.findings.iter().any(|f| f.code.starts_with("SEC")));
    }

    #[test]
    fn test_explain_nondeterministic() {
        let report = generate_explanation("#!/bin/bash\necho $RANDOM\n", &ClassifyFormat::Bash);
        assert!(report.risk_level == "medium" || report.risk_level == "high");
        assert!(report
            .categories
            .iter()
            .any(|c| c.category == "Determinism"));
    }

    #[test]
    fn test_explain_idempotency() {
        let report = generate_explanation("#!/bin/sh\nmkdir /tmp/test\n", &ClassifyFormat::Bash);
        assert!(report
            .categories
            .iter()
            .any(|c| c.category == "Idempotency"));
    }

    #[test]
    fn test_explain_json_serializable() {
        let report = generate_explanation("#!/bin/sh\necho ok\n", &ClassifyFormat::Bash);
        let json = serde_json::to_string_pretty(&report);
        assert!(json.is_ok());
        let s = json.expect("should serialize");
        assert!(s.contains("\"verdict\""));
        assert!(s.contains("\"risk_level\""));
        assert!(s.contains("\"recommendations\""));
    }

    #[test]
    fn test_explain_makefile() {
        let report = generate_explanation(
            ".PHONY: build\nbuild:\n\techo ok\n",
            &ClassifyFormat::Makefile,
        );
        assert_eq!(report.format, "makefile");
    }

    #[test]
    fn test_explain_dockerfile() {
        let report = generate_explanation(
            "FROM alpine:3.18\nUSER nobody\nCOPY app /app\n",
            &ClassifyFormat::Dockerfile,
        );
        assert_eq!(report.format, "dockerfile");
    }

    #[test]
    fn test_explain_critical_risk() {
        // Multiple security issues should escalate to critical
        let report = generate_explanation(
            "#!/bin/bash\neval \"$1\"\neval \"$2\"\neval \"$3\"\n",
            &ClassifyFormat::Bash,
        );
        assert_eq!(report.risk_level, "critical");
    }

    #[test]
    fn test_explain_why_coverage() {
        // Verify common rule codes return meaningful explanations
        for code in &[
            "SEC001", "SEC002", "SEC003", "SEC004", "SEC005", "SEC006", "SEC007", "SEC008",
            "SEC010", "SEC016", "SEC019", "SEC020", "SEC021", "SEC022", "SEC023", "SEC024",
            "DET001", "DET002", "DET003", "DET004", "IDEM001", "IDEM002", "IDEM003",
        ] {
            let why = explain_why(code);
            assert!(
                !why.contains("unexpected behavior"),
                "{code} should have a specific explanation"
            );
        }
    }

    #[test]
    fn test_explain_fix_coverage() {
        for code in &[
            "SEC001", "SEC002", "SEC003", "SEC004", "SEC005", "SEC006", "SEC007", "SEC008",
            "SEC010", "SEC016", "SEC019", "SEC020", "SEC021", "SEC022", "SEC023", "SEC024",
            "DET001", "DET002", "DET003", "DET004", "IDEM001", "IDEM002", "IDEM003",
        ] {
            let fix = explain_fix(code);
            assert!(
                !fix.contains("Review the flagged line"),
                "{code} should have a specific fix"
            );
        }
    }

    #[test]
    fn test_recommendations_include_purify() {
        let report = generate_explanation("#!/bin/bash\necho $RANDOM\n", &ClassifyFormat::Bash);
        assert!(report.recommendations.iter().any(|r| r.contains("purify")));
    }

    #[test]
    fn test_detect_format_extensions() {
        assert!(matches!(
            detect_format(Path::new("test.sh")),
            ClassifyFormat::Bash
        ));
        assert!(matches!(
            detect_format(Path::new("Makefile")),
            ClassifyFormat::Makefile
        ));
        assert!(matches!(
            detect_format(Path::new("Dockerfile")),
            ClassifyFormat::Dockerfile
        ));
    }
}
