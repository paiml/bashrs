#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_commit_ast_transform() {
        assert_eq!(
            classify_commit("fix: handle nested quotes in command substitution"),
            OipCategory::AstTransform
        );
        assert_eq!(
            classify_commit("fix: parser crash on heredoc with tabs"),
            OipCategory::AstTransform
        );
        assert_eq!(
            classify_commit("fix: emit brace group correctly"),
            OipCategory::AstTransform
        );
    }

    #[test]
    fn test_classify_commit_operator_precedence() {
        assert_eq!(
            classify_commit("fix: arithmetic precedence for nested expressions"),
            OipCategory::OperatorPrecedence
        );
        assert_eq!(
            classify_commit("fix: operator parenthesization in compound expressions"),
            OipCategory::OperatorPrecedence
        );
    }

    #[test]
    fn test_classify_commit_security() {
        assert_eq!(
            classify_commit("fix: quoting issue in variable expansion"),
            OipCategory::SecurityVulnerabilities
        );
        assert_eq!(
            classify_commit("fix: command injection via unescaped input"),
            OipCategory::SecurityVulnerabilities
        );
    }

    #[test]
    fn test_classify_commit_idempotency() {
        assert_eq!(
            classify_commit("fix: idempotency violation in mkdir"),
            OipCategory::IdempotencyViolation
        );
    }

    #[test]
    fn test_classify_commit_comprehension() {
        assert_eq!(
            classify_commit("fix: iterator early exit in accumulator"),
            OipCategory::ComprehensionBugs
        );
    }

    #[test]
    fn test_classify_commit_config() {
        assert_eq!(
            classify_commit("fix: env var config loading with default value"),
            OipCategory::ConfigurationErrors
        );
    }

    #[test]
    fn test_classify_commit_integration() {
        assert_eq!(
            classify_commit("fix: cross-shell compatibility for case statement"),
            OipCategory::IntegrationFailures
        );
        assert_eq!(
            classify_commit("fix: POSIX compliance for test command"),
            OipCategory::IntegrationFailures
        );
    }

    #[test]
    fn test_classify_commit_false_positive() {
        assert_eq!(
            classify_commit("fix: false positive on SC2171 rule"),
            OipCategory::FalsePositives
        );
    }

    #[test]
    fn test_classify_commit_type_errors() {
        assert_eq!(
            classify_commit("fix: missing type u16 support in codegen"),
            OipCategory::TypeErrors
        );
    }

    #[test]
    fn test_classify_commit_performance() {
        assert_eq!(
            classify_commit("fix: performance regression in optimizer"),
            OipCategory::Performance
        );
    }

    #[test]
    fn test_classify_commit_documentation() {
        assert_eq!(
            classify_commit("fix: doc comment typo"),
            OipCategory::Documentation
        );
    }

    #[test]
    fn test_classify_commit_test() {
        assert_eq!(
            classify_commit("fix: test suite flaky assertion in CI"),
            OipCategory::TestInfrastructure
        );
    }

    #[test]
    fn test_classify_commit_other() {
        assert_eq!(
            classify_commit("fix: miscellaneous cleanup"),
            OipCategory::Other
        );
    }

    #[test]
    fn test_parse_fix_commits() {
        let log = "abc1234|2026-02-08|fix: handle nested quotes in parser\ndef5678|2026-02-07|fix: arithmetic precedence bug";
        let commits = parse_fix_commits(log);
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].hash, "abc1234");
        assert_eq!(commits[0].date, "2026-02-08");
        assert_eq!(commits[0].category, OipCategory::AstTransform);
        assert_eq!(commits[1].category, OipCategory::OperatorPrecedence);
    }

    #[test]
    fn test_parse_fix_commits_empty() {
        let commits = parse_fix_commits("");
        assert!(commits.is_empty());
    }

    #[test]
    fn test_parse_fix_commits_malformed() {
        let log = "abc1234|no second pipe";
        let commits = parse_fix_commits(log);
        assert!(commits.is_empty());
    }

    #[test]
    fn test_category_distribution() {
        let commits = vec![
            FixCommit {
                hash: "a".to_string(),
                date: "2026-01-01".to_string(),
                message: "fix: parser".to_string(),
                category: OipCategory::AstTransform,
                files_changed: 1,
                has_corpus_entry: false,
            },
            FixCommit {
                hash: "b".to_string(),
                date: "2026-01-02".to_string(),
                message: "fix: parser 2".to_string(),
                category: OipCategory::AstTransform,
                files_changed: 1,
                has_corpus_entry: false,
            },
            FixCommit {
                hash: "c".to_string(),
                date: "2026-01-03".to_string(),
                message: "fix: quoting".to_string(),
                category: OipCategory::SecurityVulnerabilities,
                files_changed: 1,
                has_corpus_entry: false,
            },
        ];
        let dist = category_distribution(&commits);
        assert_eq!(dist[0].0, OipCategory::AstTransform);
        assert_eq!(dist[0].1, 2);
        assert_eq!(dist[1].0, OipCategory::SecurityVulnerabilities);
        assert_eq!(dist[1].1, 1);
    }

    #[test]
    fn test_category_priority() {
        assert_eq!(
            category_priority(OipCategory::AstTransform),
            GapPriority::High
        );
        assert_eq!(
            category_priority(OipCategory::SecurityVulnerabilities),
            GapPriority::High
        );
        assert_eq!(
            category_priority(OipCategory::IdempotencyViolation),
            GapPriority::Medium
        );
        assert_eq!(
            category_priority(OipCategory::Documentation),
            GapPriority::Low
        );
    }

    #[test]
    fn test_find_fix_gaps() {
        let commits = vec![
            FixCommit {
                hash: "a".to_string(),
                date: "2026-01-01".to_string(),
                message: "fix: parser crash".to_string(),
                category: OipCategory::AstTransform,
                files_changed: 1,
                has_corpus_entry: false,
            },
            FixCommit {
                hash: "b".to_string(),
                date: "2026-01-02".to_string(),
                message: "fix: doc typo".to_string(),
                category: OipCategory::Documentation,
                files_changed: 1,
                has_corpus_entry: false,
            },
        ];
        let descriptions = vec![];
        let gaps = find_fix_gaps(&commits, &descriptions);
        // Documentation fix should be filtered out (Low priority)
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].suggested_id, "B-501");
        assert_eq!(gaps[0].priority, GapPriority::High);
    }

    #[test]
    fn test_find_fix_gaps_with_corpus_entry() {
        let commits = vec![FixCommit {
            hash: "a".to_string(),
            date: "2026-01-01".to_string(),
            message: "fix: parser crash".to_string(),
            category: OipCategory::AstTransform,
            files_changed: 1,
            has_corpus_entry: true,
        }];
        let descriptions = vec![];
        let gaps = find_fix_gaps(&commits, &descriptions);
        assert!(gaps.is_empty());
    }

    #[test]
    fn test_has_matching_corpus_entry() {
        let descriptions = vec![
            "Nested quoting in command substitution".to_string(),
            "Variable assignment with arithmetic".to_string(),
        ];
        assert!(has_matching_corpus_entry(
            "fix: handle nested quoting in command substitution",
            &descriptions
        ));
        assert!(!has_matching_corpus_entry(
            "fix: random unrelated fix",
            &descriptions
        ));
    }

    #[test]
    fn test_has_matching_corpus_entry_empty() {
        assert!(!has_matching_corpus_entry("fix: something", &[]));
    }

    #[test]
    fn test_format_mine_table() {
        let commits = vec![FixCommit {
            hash: "abc1234".to_string(),
            date: "2026-02-08".to_string(),
            message: "fix: parser crash on heredoc".to_string(),
            category: OipCategory::AstTransform,
            files_changed: 3,
            has_corpus_entry: true,
        }];
        let table = format_mine_table(&commits);
        assert!(table.contains("OIP Fix Pattern Mining"));
        assert!(table.contains("abc1234"));
        assert!(table.contains("ASTTransform"));
        assert!(table.contains("\u{2713}")); // checkmark
        assert!(table.contains("1 fix commits"));
    }

    #[test]
    fn test_format_mine_table_empty() {
        let table = format_mine_table(&[]);
        assert!(table.contains("0 fix commits"));
    }

    #[test]
    fn test_format_fix_gaps_table() {
        let gaps = vec![FixGap {
            commit: FixCommit {
                hash: "abc1234".to_string(),
                date: "2026-02-08".to_string(),
                message: "fix: parser crash".to_string(),
                category: OipCategory::AstTransform,
                files_changed: 1,
                has_corpus_entry: false,
            },
            suggested_id: "B-501".to_string(),
            suggested_description: "Regression test for ASTTransform fix".to_string(),
            priority: GapPriority::High,
        }];
        let table = format_fix_gaps_table(&gaps);
        assert!(table.contains("Fix-Driven Corpus Gaps"));
        assert!(table.contains("B-501"));
        assert!(table.contains("HIGH"));
        assert!(table.contains("1 gaps total"));
    }

    #[test]
    fn test_format_fix_gaps_table_empty() {
        let table = format_fix_gaps_table(&[]);
        assert!(table.contains("0 gaps total"));
    }

    #[test]
    fn test_format_org_patterns_table() {
        let patterns = known_org_patterns();
        let table = format_org_patterns_table(&patterns);
        assert!(table.contains("Cross-Project Defect Patterns"));
        assert!(table.contains("Off-by-one"));
        assert!(table.contains("String escaping"));
        assert!(table.contains("Precedence"));
    }

    #[test]
    fn test_known_org_patterns_not_empty() {
        let patterns = known_org_patterns();
        assert!(!patterns.is_empty());
        assert!(patterns.len() >= 8);
    }

    #[test]
    fn test_truncate_message_short() {
        assert_eq!(truncate_message("short", 10), "short");
    }

    #[test]
    fn test_truncate_message_long() {
        let result = truncate_message("this is a very long message", 10);
        // 9 ASCII chars + 3-byte ellipsis character = 12 bytes
        assert!(result.len() <= 12);
        assert!(result.ends_with('\u{2026}'));
    }

    #[test]
    fn test_oip_category_display() {
        assert_eq!(format!("{}", OipCategory::AstTransform), "ASTTransform");
        assert_eq!(
            format!("{}", OipCategory::SecurityVulnerabilities),
            "SecurityVulnerabilities"
        );
        assert_eq!(format!("{}", OipCategory::Other), "Other");
    }

    #[test]
    fn test_gap_priority_display() {
        assert_eq!(format!("{}", GapPriority::High), "HIGH");
        assert_eq!(format!("{}", GapPriority::Medium), "MEDIUM");
        assert_eq!(format!("{}", GapPriority::Low), "LOW");
    }

    #[test]
    fn test_category_distribution_empty() {
        let dist = category_distribution(&[]);
        assert!(dist.is_empty());
    }
}
