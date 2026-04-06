#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusTier};

    fn make_entry(id: &str, format: CorpusFormat) -> CorpusEntry {
        CorpusEntry {
            id: id.to_string(),
            name: format!("test-{id}"),
            description: "Test entry".to_string(),
            format,
            tier: CorpusTier::Trivial,
            input: "fn main() {}".to_string(),
            expected_output: "#!/bin/sh\necho ok\n".to_string(),
            shellcheck: true,
            deterministic: true,
            idempotent: true,
        }
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
            deterministic: transpiled,
            metamorphic_consistent: transpiled,
            cross_shell_agree: transpiled,
            expected_output: None,
            actual_output: if transpiled {
                Some("#!/bin/sh\necho ok\n".to_string())
            } else {
                None
            },
            error: if transpiled {
                None
            } else {
                Some("test error".to_string())
            },
            error_category: None,
            error_confidence: None,
            decision_trace: None,
        }
    }

    #[test]
    fn test_export_format_display() {
        assert_eq!(format!("{}", ExportFormat::JsonLines), "jsonl");
        assert_eq!(format!("{}", ExportFormat::Csv), "csv");
        assert_eq!(format!("{}", ExportFormat::Json), "json");
        assert_eq!(
            format!("{}", ExportFormat::Classification),
            "classification"
        );
        assert_eq!(
            format!("{}", ExportFormat::MultiLabelClassification),
            "multi-label-classification"
        );
    }

    #[test]
    fn test_score_to_grade() {
        assert_eq!(score_to_grade(100.0), "A+");
        assert_eq!(score_to_grade(97.0), "A+");
        assert_eq!(score_to_grade(93.0), "A");
        assert_eq!(score_to_grade(90.0), "A-");
        assert_eq!(score_to_grade(85.0), "B");
        assert_eq!(score_to_grade(75.0), "C");
        assert_eq!(score_to_grade(65.0), "D");
        assert_eq!(score_to_grade(50.0), "F");
    }

    #[test]
    fn test_csv_escape_plain() {
        assert_eq!(csv_escape("hello"), "hello");
    }

    #[test]
    fn test_csv_escape_comma() {
        assert_eq!(csv_escape("hello,world"), "\"hello,world\"");
    }

    #[test]
    fn test_csv_escape_quotes() {
        assert_eq!(csv_escape("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn test_csv_escape_newline() {
        assert_eq!(csv_escape("line1\nline2"), "\"line1\nline2\"");
    }

    #[test]
    fn test_build_row_with_result() {
        let entry = make_entry("B-001", CorpusFormat::Bash);
        let result = make_result("B-001", true);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc1234", "2026-02-09");

        assert_eq!(row.id, "B-001");
        assert!(row.transpiled);
        assert!(row.output_correct);
        assert!(row.lint_clean);
        assert!(row.deterministic);
        assert_eq!(row.bashrs_version, "6.61.0");
        assert!(row.score > 0.0);
    }

    #[test]
    fn test_build_row_without_result() {
        let entry = make_entry("B-002", CorpusFormat::Bash);
        let row = build_row(&entry, None, "6.61.0", "abc1234", "2026-02-09");

        assert_eq!(row.id, "B-002");
        assert!(!row.transpiled);
        assert!(!row.output_correct);
        assert_eq!(row.score, 0.0);
        assert_eq!(row.grade, "F");
    }

    #[test]
    fn test_build_row_failed_result() {
        let entry = make_entry("B-003", CorpusFormat::Bash);
        let result = make_result("B-003", false);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc1234", "2026-02-09");

        assert!(!row.transpiled);
        assert!(!row.output_correct);
        assert_eq!(row.grade, "F");
    }

    #[test]
    fn test_export_jsonl() {
        let entry = make_entry("B-001", CorpusFormat::Bash);
        let result = make_result("B-001", true);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc1234", "2026-02-09");

        let output = export_jsonl(&[row]);
        assert!(output.contains("\"id\":\"B-001\""));
        assert!(output.contains("\"transpiled\":true"));
        assert!(!output.contains('\n') || output.lines().count() == 1);
    }

    #[test]
    fn test_export_json() {
        let entry = make_entry("B-001", CorpusFormat::Bash);
        let result = make_result("B-001", true);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc1234", "2026-02-09");

        let output = export_json(&[row]);
        assert!(output.starts_with('['));
        assert!(output.contains("\"id\": \"B-001\""));
    }

    #[test]
    fn test_export_csv() {
        let entry = make_entry("B-001", CorpusFormat::Bash);
        let result = make_result("B-001", true);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc1234", "2026-02-09");

        let output = export_csv(&[row]);
        assert!(output.starts_with("id,name,tier,format"));
        assert!(output.contains("B-001"));
        assert!(output.contains("bash"));
    }

    #[test]
    fn test_export_csv_multiple_rows() {
        let rows: Vec<DatasetRow> = ["B-001", "M-001"]
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let fmt = if i == 0 {
                    CorpusFormat::Bash
                } else {
                    CorpusFormat::Makefile
                };
                let entry = make_entry(id, fmt);
                let result = make_result(id, true);
                build_row(&entry, Some(&result), "6.61.0", "abc", "2026-02-09")
            })
            .collect();

        let output = export_csv(&rows);
        assert_eq!(output.lines().count(), 3); // header + 2 rows
    }

    #[test]
    fn test_dataset_info() {
        let entries = vec![
            make_entry("B-001", CorpusFormat::Bash),
            make_entry("M-001", CorpusFormat::Makefile),
            make_entry("D-001", CorpusFormat::Dockerfile),
        ];
        let registry = CorpusRegistry { entries };
        let info = dataset_info(&registry);
        assert_eq!(info.total_entries, 3);
        assert_eq!(info.format_counts.len(), 3);
        assert_eq!(info.schema_fields.len(), 18);
    }

    include!("dataset_tests_extracted_format.rs");
}
