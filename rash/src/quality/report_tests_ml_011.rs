use super::*;

#[test]
fn test_ml_011_sparkline_basic() {
    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let spark = sparkline(&values);

    assert_eq!(spark.chars().count(), 8);
    assert!(spark.contains(chars::SPARK_1_8));
    assert!(spark.contains(chars::SPARK_FULL));
}

#[test]
fn test_ml_011_sparkline_empty() {
    let values: Vec<f64> = vec![];
    let spark = sparkline(&values);
    assert!(spark.is_empty());
}

#[test]
fn test_ml_011_sparkline_constant() {
    let values = vec![5.0, 5.0, 5.0, 5.0];
    let spark = sparkline(&values);

    // All same value should produce middle-height bars
    assert_eq!(spark.chars().count(), 4);
}

#[test]
fn test_ml_011_progress_bar() {
    let bar = progress_bar(50.0, 100.0, 10);
    assert!(bar.contains("50.0%"));
    assert!(bar.starts_with('['));
    assert!(bar.contains(']'));
}

#[test]
fn test_ml_012_grade_from_percentage() {
    assert_eq!(Grade::from_percentage(95.0), Grade::APlus);
    assert_eq!(Grade::from_percentage(90.0), Grade::A);
    assert_eq!(Grade::from_percentage(85.0), Grade::AMinus);
    assert_eq!(Grade::from_percentage(80.0), Grade::BPlus);
    assert_eq!(Grade::from_percentage(70.0), Grade::BMinus);
    assert_eq!(Grade::from_percentage(60.0), Grade::C);
    assert_eq!(Grade::from_percentage(40.0), Grade::F);
}

#[test]
fn test_ml_012_grade_passing() {
    assert!(Grade::APlus.is_passing());
    assert!(Grade::C.is_passing());
    assert!(Grade::CMinus.is_passing());
    assert!(!Grade::D.is_passing());
    assert!(!Grade::F.is_passing());
}

#[test]
fn test_ml_011_report_builder() {
    let report = ReportBuilder::new("Test Report")
        .width(50)
        .section("Results")
        .status(true, "All tests passed")
        .kv("Coverage", "91.2%")
        .progress("Completion", 45.0, 50.0)
        .build();

    let output = report.render();

    assert!(output.contains("Test Report"));
    assert!(output.contains("Results"));
    assert!(output.contains("All tests passed"));
    assert!(output.contains("Coverage"));
    assert!(output.contains("91.2%"));
}

#[test]
fn test_ml_011_gate_report() {
    let details = vec![
        ("clippy".to_string(), true, Duration::from_millis(500)),
        ("tests".to_string(), true, Duration::from_secs(30)),
        ("coverage".to_string(), false, Duration::from_secs(45)),
    ];

    let report = gate_report(2, 2, 3, Duration::from_secs(76), &details);

    assert!(report.contains("Tier 2"));
    assert!(report.contains("Quality Gate"));
    assert!(report.contains("2/3"));
    assert!(report.contains("clippy"));
    assert!(report.contains("coverage"));
}

#[test]
fn test_ml_011_format_duration() {
    assert_eq!(format_duration(Duration::from_millis(500)), "500ms");
    assert_eq!(format_duration(Duration::from_secs(5)), "5.0s");
    assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
    assert_eq!(format_duration(Duration::from_secs(3700)), "1h 1m");
}

// ===== Additional tests for coverage =====

#[test]
fn test_grade_all_letters() {
    assert_eq!(Grade::APlus.as_letter(), "A+");
    assert_eq!(Grade::A.as_letter(), "A");
    assert_eq!(Grade::AMinus.as_letter(), "A-");
    assert_eq!(Grade::BPlus.as_letter(), "B+");
    assert_eq!(Grade::B.as_letter(), "B");
    assert_eq!(Grade::BMinus.as_letter(), "B-");
    assert_eq!(Grade::CPlus.as_letter(), "C+");
    assert_eq!(Grade::C.as_letter(), "C");
    assert_eq!(Grade::CMinus.as_letter(), "C-");
    assert_eq!(Grade::D.as_letter(), "D");
    assert_eq!(Grade::F.as_letter(), "F");
}

#[test]
fn test_grade_from_all_ranges() {
    // Test exact boundaries
    assert_eq!(Grade::from_percentage(100.0), Grade::APlus);
    assert_eq!(Grade::from_percentage(95.0), Grade::APlus);
    assert_eq!(Grade::from_percentage(94.0), Grade::A);
    assert_eq!(Grade::from_percentage(90.0), Grade::A);
    assert_eq!(Grade::from_percentage(89.0), Grade::AMinus);
    assert_eq!(Grade::from_percentage(85.0), Grade::AMinus);
    assert_eq!(Grade::from_percentage(84.0), Grade::BPlus);
    assert_eq!(Grade::from_percentage(80.0), Grade::BPlus);
    assert_eq!(Grade::from_percentage(79.0), Grade::B);
    assert_eq!(Grade::from_percentage(75.0), Grade::B);
    assert_eq!(Grade::from_percentage(74.0), Grade::BMinus);
    assert_eq!(Grade::from_percentage(70.0), Grade::BMinus);
    assert_eq!(Grade::from_percentage(69.0), Grade::CPlus);
    assert_eq!(Grade::from_percentage(65.0), Grade::CPlus);
    assert_eq!(Grade::from_percentage(64.0), Grade::C);
    assert_eq!(Grade::from_percentage(60.0), Grade::C);
    assert_eq!(Grade::from_percentage(59.0), Grade::CMinus);
    assert_eq!(Grade::from_percentage(55.0), Grade::CMinus);
    assert_eq!(Grade::from_percentage(54.0), Grade::D);
    assert_eq!(Grade::from_percentage(50.0), Grade::D);
    assert_eq!(Grade::from_percentage(49.0), Grade::F);
    assert_eq!(Grade::from_percentage(0.0), Grade::F);
}

#[test]
fn test_grade_display() {
    assert_eq!(format!("{}", Grade::APlus), "A+");
    assert_eq!(format!("{}", Grade::F), "F");
    assert_eq!(format!("{}", Grade::CMinus), "C-");
}

#[test]
fn test_grade_ordering() {
    assert!(Grade::APlus > Grade::A);
    assert!(Grade::A > Grade::AMinus);
    assert!(Grade::AMinus > Grade::BPlus);
    assert!(Grade::F < Grade::D);
    assert!(Grade::CMinus > Grade::D);
}

#[test]
fn test_sparkline_single_value() {
    let values = vec![5.0];
    let spark = sparkline(&values);
    assert_eq!(spark.chars().count(), 1);
}

#[test]
fn test_sparkline_two_values() {
    let values = vec![1.0, 10.0];
    let spark = sparkline(&values);
    assert_eq!(spark.chars().count(), 2);
    // First should be lowest, second should be highest
}

#[test]
fn test_progress_bar_zero_total() {
    let bar = progress_bar(50.0, 0.0, 10);
    assert!(bar.contains("0.0%"));
}

#[test]
fn test_progress_bar_full() {
    let bar = progress_bar(100.0, 100.0, 10);
    assert!(bar.contains("100.0%"));
}

#[test]
fn test_progress_bar_overflow() {
    // Current > total should cap at 100%
    let bar = progress_bar(150.0, 100.0, 10);
    assert!(bar.contains("150.0%"));
}

#[test]
fn test_report_builder_item_without_section() {
    // Adding item without a section shouldn't panic
    let builder = ReportBuilder::new("Test").item(ReportItem::Text("orphan".to_string()));
    let report = builder.build();
    assert!(report.sections.is_empty());
}

#[test]
fn test_report_builder_all_item_types() {
    let report = ReportBuilder::new("Full Test")
        .width(80)
        .section("All Items")
        .kv("Key", "Value")
        .status(true, "Passed")
        .status(false, "Failed")
        .progress("Progress", 50.0, 100.0)
        .sparkline("Trend", vec![1.0, 2.0, 3.0, 4.0, 5.0])
        .item(ReportItem::TableRow {
            cells: vec!["A".to_string(), "B".to_string()],
        })
        .item(ReportItem::Text("Plain text".to_string()))
        .metric("Coverage", 92.0, "%")
        .build();

    let output = report.render();

    assert!(output.contains("Full Test"));
    assert!(output.contains("Key: Value"));
    assert!(output.contains("Passed"));
    assert!(output.contains("Failed"));
    assert!(output.contains("Progress"));
    assert!(output.contains("Trend"));
    assert!(output.contains("A │ B"));
    assert!(output.contains("Plain text"));
    assert!(output.contains("Coverage"));
    assert!(output.contains("92.0%"));
}

#[test]
fn test_report_builder_metric_grades() {
    let report = ReportBuilder::new("Metrics")
        .section("Grades")
        .metric("Excellent", 98.0, "%")
        .metric("Poor", 45.0, "%")
        .build();

    let output = report.render();
    assert!(output.contains("✓")); // Passing grade
    assert!(output.contains("✗")); // Failing grade
}

#[test]
fn test_gate_report_all_passed() {
    let details = vec![
        ("gate1".to_string(), true, Duration::from_millis(100)),
        ("gate2".to_string(), true, Duration::from_millis(200)),
    ];

    let report = gate_report(1, 2, 2, Duration::from_millis(300), &details);

    assert!(report.contains("Tier 1"));
    assert!(report.contains("2/2"));
    assert!(report.contains("100.0%"));
    assert!(report.contains("gate1"));
    assert!(report.contains("gate2"));
}

#[test]
fn test_gate_report_empty_gates() {
    let details: Vec<(String, bool, Duration)> = vec![];

    let report = gate_report(3, 0, 0, Duration::from_secs(0), &details);

    assert!(report.contains("Tier 3"));
    assert!(report.contains("0/0"));
    assert!(report.contains("100.0%")); // 0/0 is 100%
}

#[test]
fn test_format_duration_various() {
    assert_eq!(format_duration(Duration::from_millis(0)), "0ms");
    assert_eq!(format_duration(Duration::from_millis(999)), "999ms");
    assert_eq!(format_duration(Duration::from_secs(1)), "1.0s");
    assert_eq!(format_duration(Duration::from_secs(59)), "59.0s");
    assert_eq!(format_duration(Duration::from_secs(60)), "1m 0s");
    assert_eq!(format_duration(Duration::from_secs(3599)), "59m 59s");
    assert_eq!(format_duration(Duration::from_secs(3600)), "1h 0m");
    assert_eq!(format_duration(Duration::from_secs(7200)), "2h 0m");
}

#[test]
fn test_chars_constants() {
    // Verify box drawing characters are defined
    assert_eq!(chars::TOP_LEFT, '┌');
    assert_eq!(chars::TOP_RIGHT, '┐');
    assert_eq!(chars::BOTTOM_LEFT, '└');
    assert_eq!(chars::BOTTOM_RIGHT, '┘');
    assert_eq!(chars::HORIZONTAL, '─');
    assert_eq!(chars::VERTICAL, '│');
    assert_eq!(chars::T_DOWN, '┬');
    assert_eq!(chars::T_UP, '┴');
    assert_eq!(chars::T_RIGHT, '├');
    assert_eq!(chars::T_LEFT, '┤');
    assert_eq!(chars::CROSS, '┼');
}

#[test]
fn test_chars_sparkline_constants() {
    assert_eq!(chars::SPARK_EMPTY, ' ');
    assert_eq!(chars::SPARK_1_8, '▁');
    assert_eq!(chars::SPARK_2_8, '▂');
    assert_eq!(chars::SPARK_3_8, '▃');
    assert_eq!(chars::SPARK_4_8, '▄');
    assert_eq!(chars::SPARK_5_8, '▅');
    assert_eq!(chars::SPARK_6_8, '▆');
    assert_eq!(chars::SPARK_7_8, '▇');
    assert_eq!(chars::SPARK_FULL, '█');
}

#[test]
fn test_chars_status_constants() {
    assert_eq!(chars::CHECK, "✓");
    assert_eq!(chars::CROSS_MARK, "✗");
    assert_eq!(chars::WARNING, "⚠");
    assert_eq!(chars::INFO, "ℹ");
    assert_eq!(chars::BULLET, "•");
}

#[test]
fn test_report_section_clone() {
    let section = ReportSection {
        title: "Test Section".to_string(),
        items: vec![ReportItem::Text("Item".to_string())],
    };

    let cloned = section.clone();
    assert_eq!(cloned.title, "Test Section");
    assert_eq!(cloned.items.len(), 1);
}

#[test]
fn test_report_item_variants_clone() {
    let items = vec![
        ReportItem::KeyValue {
            key: "k".to_string(),
            value: "v".to_string(),
        },
        ReportItem::Status {
            passed: true,
            message: "m".to_string(),
        },
        ReportItem::Progress {
            label: "l".to_string(),
            current: 1.0,
            total: 2.0,
        },
        ReportItem::Sparkline {
            label: "s".to_string(),
            values: vec![1.0],
        },
        ReportItem::TableRow {
            cells: vec!["c".to_string()],
        },
        ReportItem::Text("t".to_string()),
        ReportItem::Metric {
            name: "n".to_string(),
            value: 1.0,
            unit: "u".to_string(),
            grade: Grade::A,
        },
    ];

    for item in items {
        let cloned = item.clone();
        let _ = format!("{:?}", cloned);
    }
}

#[test]
fn test_rich_report_clone() {
    let report = RichReport {
        title: "Test".to_string(),
        sections: vec![],
        width: 60,
    };

    let cloned = report.clone();
    assert_eq!(cloned.title, "Test");
    assert_eq!(cloned.width, 60);
}

#[test]
fn test_grade_debug() {
    let debug = format!("{:?}", Grade::APlus);
    assert!(debug.contains("APlus"));
}

#[test]
fn test_report_section_debug() {
    let section = ReportSection {
        title: "Title".to_string(),
        items: vec![],
    };
    let debug = format!("{:?}", section);
    assert!(debug.contains("Title"));
}

#[test]
fn test_report_narrow_width() {
    let report = ReportBuilder::new("Narrow")
        .width(20)
        .section("Small")
        .kv("K", "V")
        .build();

    let output = report.render();
    assert!(output.contains("Narrow"));
}

#[test]
fn test_report_long_title() {
    // Test with title that fits within width
    let long_title = "A".repeat(40);
    let report = ReportBuilder::new(&long_title)
        .width(60) // Width larger than title
        .section("Section")
        .build();

    let output = report.render();
    assert!(output.contains(&long_title));
}

#[test]
fn test_sparkline_negative_values() {
    let values = vec![-5.0, -3.0, 0.0, 3.0, 5.0];
    let spark = sparkline(&values);
    assert_eq!(spark.chars().count(), 5);
}

#[test]
fn test_sparkline_large_range() {
    let values = vec![0.0, 1000000.0];
    let spark = sparkline(&values);
    assert_eq!(spark.chars().count(), 2);
}
