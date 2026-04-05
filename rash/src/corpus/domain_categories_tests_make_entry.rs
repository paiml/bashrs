#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::registry::{CorpusFormat, CorpusTier};

    fn make_entry(id: &str, format: CorpusFormat) -> CorpusEntry {
        CorpusEntry::new(
            id,
            "test",
            "test entry",
            format,
            CorpusTier::Trivial,
            "fn main() {}",
            "expected",
        )
    }

    fn make_result(id: &str, transpiled: bool) -> CorpusResult {
        CorpusResult {
            id: id.to_string(),
            transpiled,
            output_contains: transpiled,
            output_exact: transpiled,
            output_behavioral: transpiled,
            has_test: true,
            coverage_ratio: 0.95,
            schema_valid: true,
            lint_clean: transpiled,
            deterministic: true,
            metamorphic_consistent: true,
            cross_shell_agree: transpiled,
            expected_output: None,
            actual_output: if transpiled {
                Some("expected".to_string())
            } else {
                None
            },
            error: if transpiled {
                None
            } else {
                Some("error".to_string())
            },
            error_category: None,
            error_confidence: None,
            decision_trace: None,
        }
    }

    #[test]
    fn test_classify_bash_general() {
        let entry = make_entry("B-001", CorpusFormat::Bash);
        assert_eq!(classify_entry(&entry), DomainCategory::General);
    }

    #[test]
    fn test_classify_shell_config() {
        let entry = make_entry("B-371", CorpusFormat::Bash);
        assert_eq!(classify_entry(&entry), DomainCategory::ShellConfig);
        let entry = make_entry("B-380", CorpusFormat::Bash);
        assert_eq!(classify_entry(&entry), DomainCategory::ShellConfig);
    }

    #[test]
    fn test_classify_one_liners() {
        let entry = make_entry("B-385", CorpusFormat::Bash);
        assert_eq!(classify_entry(&entry), DomainCategory::OneLiners);
    }

    #[test]
    fn test_classify_provability() {
        let entry = make_entry("B-395", CorpusFormat::Bash);
        assert_eq!(classify_entry(&entry), DomainCategory::Provability);
    }

    #[test]
    fn test_classify_unix_tools() {
        let entry = make_entry("B-405", CorpusFormat::Bash);
        assert_eq!(classify_entry(&entry), DomainCategory::UnixTools);
    }

    #[test]
    fn test_classify_lang_integration() {
        let entry = make_entry("B-415", CorpusFormat::Bash);
        assert_eq!(classify_entry(&entry), DomainCategory::LangIntegration);
    }

    #[test]
    fn test_classify_system_tooling() {
        let entry = make_entry("B-425", CorpusFormat::Bash);
        assert_eq!(classify_entry(&entry), DomainCategory::SystemTooling);
    }

    #[test]
    fn test_classify_coreutils() {
        let entry = make_entry("B-431", CorpusFormat::Bash);
        assert_eq!(classify_entry(&entry), DomainCategory::Coreutils);
        let entry = make_entry("B-460", CorpusFormat::Bash);
        assert_eq!(classify_entry(&entry), DomainCategory::Coreutils);
    }

    #[test]
    fn test_classify_regex() {
        let entry = make_entry("B-470", CorpusFormat::Bash);
        assert_eq!(classify_entry(&entry), DomainCategory::RegexPatterns);
        let entry = make_entry("B-490", CorpusFormat::Bash);
        assert_eq!(classify_entry(&entry), DomainCategory::RegexPatterns);
    }

    #[test]
    fn test_classify_non_bash_is_general() {
        let entry = make_entry("M-001", CorpusFormat::Makefile);
        assert_eq!(classify_entry(&entry), DomainCategory::General);
        let entry = make_entry("D-001", CorpusFormat::Dockerfile);
        assert_eq!(classify_entry(&entry), DomainCategory::General);
    }

    #[test]
    fn test_classify_boundary_b370_is_general() {
        let entry = make_entry("B-370", CorpusFormat::Bash);
        assert_eq!(classify_entry(&entry), DomainCategory::General);
    }

    #[test]
    fn test_classify_boundary_b491_is_general() {
        let entry = make_entry("B-491", CorpusFormat::Bash);
        assert_eq!(classify_entry(&entry), DomainCategory::General);
    }

    #[test]
    fn test_domain_category_label() {
        assert_eq!(DomainCategory::ShellConfig.label(), "A: Shell Config");
        assert_eq!(DomainCategory::Coreutils.label(), "G: Coreutils");
    }

    #[test]
    fn test_domain_category_range() {
        assert_eq!(DomainCategory::ShellConfig.range(), Some((371, 380)));
        assert_eq!(DomainCategory::Coreutils.range(), Some((431, 460)));
        assert_eq!(DomainCategory::General.range(), None);
    }

    #[test]
    fn test_domain_category_capacity() {
        assert_eq!(DomainCategory::ShellConfig.capacity(), 10);
        assert_eq!(DomainCategory::Coreutils.capacity(), 30);
        assert_eq!(DomainCategory::RegexPatterns.capacity(), 30);
        assert_eq!(DomainCategory::General.capacity(), 0);
    }

    #[test]
    fn test_all_specific_count() {
        assert_eq!(DomainCategory::all_specific().len(), 8);
    }

    #[test]
    fn test_categorize_empty_corpus() {
        let registry = CorpusRegistry::new();
        let results: Vec<CorpusResult> = vec![];
        let stats = categorize_corpus(&registry, &results);
        // Should have 8 specific categories, all empty
        assert_eq!(stats.len(), 8);
        for s in &stats {
            assert_eq!(s.total, 0);
        }
    }

    #[test]
    fn test_categorize_mixed_entries() {
        let mut registry = CorpusRegistry::new();
        registry
            .entries
            .push(make_entry("B-001", CorpusFormat::Bash));
        registry
            .entries
            .push(make_entry("B-375", CorpusFormat::Bash));
        registry
            .entries
            .push(make_entry("B-450", CorpusFormat::Bash));
        registry
            .entries
            .push(make_entry("M-001", CorpusFormat::Makefile));

        let results = vec![
            make_result("B-001", true),
            make_result("B-375", true),
            make_result("B-450", false),
            make_result("M-001", true),
        ];

        let stats = categorize_corpus(&registry, &results);

        // ShellConfig should have 1 entry
        let config = stats
            .iter()
            .find(|s| s.category == DomainCategory::ShellConfig);
        assert!(config.is_some());
        let config = config.expect("config stat should exist");
        assert_eq!(config.total, 1);
        assert_eq!(config.passed, 1);

        // Coreutils should have 1 entry (failed)
        let core = stats
            .iter()
            .find(|s| s.category == DomainCategory::Coreutils);
        assert!(core.is_some());
        let core = core.expect("coreutils stat should exist");
        assert_eq!(core.total, 1);
        assert_eq!(core.failed, 1);

        // General should have B-001 + M-001
        let gen = stats.iter().find(|s| s.category == DomainCategory::General);
        assert!(gen.is_some());
        let gen = gen.expect("general stat should exist");
        assert_eq!(gen.total, 2);
    }

    #[test]
    fn test_format_categories_report_not_empty() {
        let stats = vec![CategoryStats {
            category: DomainCategory::ShellConfig,
            total: 5,
            capacity: 10,
            passed: 4,
            failed: 1,
            fill_pct: 50.0,
            pass_rate: 80.0,
        }];
        let report = format_categories_report(&stats);
        assert!(report.contains("A: Shell Config"));
        assert!(report.contains("50%"));
        assert!(report.contains("80.0%"));
    }

    #[test]
    fn test_format_quality_matrix_contains_properties() {
        let stats: Vec<CategoryStats> = DomainCategory::all_specific()
            .iter()
            .map(|c| CategoryStats {
                category: *c,
                total: 0,
                capacity: c.capacity(),
                passed: 0,
                failed: 0,
                fill_pct: 0.0,
                pass_rate: 0.0,
            })
            .collect();
        let matrix = format_quality_matrix(&stats);
        assert!(matrix.contains("Idempotent"));
        assert!(matrix.contains("POSIX"));
        assert!(matrix.contains("Pipeline-safe"));
        assert!(matrix.contains("REQ"));
        assert!(matrix.contains("N/A"));
    }

    #[test]
    fn test_quality_properties_count() {
        assert_eq!(QUALITY_PROPERTIES.len(), 10);
    }

    #[test]
    fn test_quality_matrix_posix_all_required() {
        // POSIX should be required for all 8 categories
        let (_, posix_reqs) = QUALITY_PROPERTIES
            .iter()
            .find(|(name, _)| *name == "POSIX")
            .expect("POSIX property should exist");
        for req in posix_reqs {
            assert_eq!(*req, QualityReq::Required);
        }
    }

    #[test]
    fn test_quality_matrix_parity_only_coreutils() {
        // 1:1 parity should only be required for Coreutils (index 6)
        let (_, parity_reqs) = QUALITY_PROPERTIES
            .iter()
            .find(|(name, _)| *name == "1:1 parity")
            .expect("1:1 parity property should exist");
        for (i, req) in parity_reqs.iter().enumerate() {
            if i == 6 {
                assert_eq!(*req, QualityReq::Required);
            } else {
                assert_eq!(*req, QualityReq::NotApplicable);
            }
        }
    }

    #[test]
    fn test_coverage_status_empty() {
        let s = CategoryStats {
            category: DomainCategory::ShellConfig,
            total: 0,
            capacity: 10,
            passed: 0,
            failed: 0,
            fill_pct: 0.0,
            pass_rate: 0.0,
        };
        assert_eq!(coverage_status(&s), "EMPTY");
    }

    #[test]
    fn test_coverage_status_complete() {
        let s = CategoryStats {
            category: DomainCategory::ShellConfig,
            total: 10,
            capacity: 10,
            passed: 10,
            failed: 0,
            fill_pct: 100.0,
            pass_rate: 100.0,
        };
        assert_eq!(coverage_status(&s), "COMPLETE");
    }

    #[test]
    fn test_coverage_status_partial() {
        let s = CategoryStats {
            category: DomainCategory::ShellConfig,
            total: 6,
            capacity: 10,
            passed: 6,
            failed: 0,
            fill_pct: 60.0,
            pass_rate: 100.0,
        };
        assert_eq!(coverage_status(&s), "PARTIAL");
    }

    #[test]
    fn test_coverage_status_sparse() {
        let s = CategoryStats {
            category: DomainCategory::ShellConfig,
            total: 2,
            capacity: 10,
            passed: 2,
            failed: 0,
            fill_pct: 20.0,
            pass_rate: 100.0,
        };
        assert_eq!(coverage_status(&s), "SPARSE");
    }

    #[test]
    fn test_format_domain_coverage_gaps() {
        use crate::corpus::registry::Grade;
        use crate::corpus::runner::{CorpusScore, FormatScore};

        let stats = vec![
            CategoryStats {
                category: DomainCategory::ShellConfig,
                total: 5,
                capacity: 10,
                passed: 5,
                failed: 0,
                fill_pct: 50.0,
                pass_rate: 100.0,
            },
            CategoryStats {
                category: DomainCategory::OneLiners,
                total: 0,
                capacity: 10,
                passed: 0,
                failed: 0,
                fill_pct: 0.0,
                pass_rate: 0.0,
            },
        ];
        let score = CorpusScore {
            total: 900,
            passed: 899,
            failed: 1,
            rate: 0.999,
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
                    passed: 200,
                    rate: 1.0,
                    score: 100.0,
                    grade: Grade::APlus,
                },
            ],
            results: vec![],
        };

        let report = format_domain_coverage(&stats, &score);
        assert!(report.contains("Coverage Gaps"));
        assert!(report.contains("A: Shell Config"));
        assert!(report.contains("B: One-Liners"));
        assert!(report.contains("EMPTY"));
        assert!(report.contains("PARTIAL"));
    }
}
