use super::*;
use crate::cli::args::LintLevel;

// ===== BUILD IGNORED RULES TESTS =====

#[test]
fn test_build_ignored_rules_from_ignore_str() {
    let rules = build_ignored_rules(Some("SEC001,DET002"), None, &[]);
    assert!(rules.contains("SEC001"));
    assert!(rules.contains("DET002"));
    assert_eq!(rules.len(), 2);
}

#[test]
fn test_build_ignored_rules_from_exclude() {
    let excludes = vec!["sec003".to_string(), "det004".to_string()];
    let rules = build_ignored_rules(None, Some(&excludes), &[]);
    assert!(rules.contains("SEC003"));
    assert!(rules.contains("DET004"));
}

#[test]
fn test_build_ignored_rules_from_file() {
    let file_rules = vec!["IDEM001".to_string(), "IDEM002".to_string()];
    let rules = build_ignored_rules(None, None, &file_rules);
    assert!(rules.contains("IDEM001"));
    assert!(rules.contains("IDEM002"));
}

#[test]
fn test_build_ignored_rules_combined() {
    let excludes = vec!["det001".to_string()];
    let file_rules = vec!["IDEM001".to_string()];
    let rules = build_ignored_rules(Some("SEC001"), Some(&excludes), &file_rules);
    assert!(rules.contains("SEC001"));
    assert!(rules.contains("DET001"));
    assert!(rules.contains("IDEM001"));
    assert_eq!(rules.len(), 3);
}

#[test]
fn test_build_ignored_rules_handles_whitespace() {
    let rules = build_ignored_rules(Some(" SEC001 , DET002 "), None, &[]);
    assert!(rules.contains("SEC001"));
    assert!(rules.contains("DET002"));
}

// ===== DETERMINE MIN SEVERITY TESTS =====

#[test]
fn test_determine_min_severity_quiet() {
    let severity = determine_min_severity(true, LintLevel::Info);
    assert_eq!(severity, Severity::Warning);
}

#[test]
fn test_determine_min_severity_level_info() {
    let severity = determine_min_severity(false, LintLevel::Info);
    assert_eq!(severity, Severity::Info);
}

#[test]
fn test_determine_min_severity_level_warning() {
    let severity = determine_min_severity(false, LintLevel::Warning);
    assert_eq!(severity, Severity::Warning);
}

#[test]
fn test_determine_min_severity_level_error() {
    let severity = determine_min_severity(false, LintLevel::Error);
    assert_eq!(severity, Severity::Error);
}

// ===== LINT PROCESS TESTS =====

#[test]
fn test_file_type_from_filename_shell() {
    assert_eq!(FileType::from_filename("script.sh"), FileType::Shell);
    assert_eq!(FileType::from_filename("test.bash"), FileType::Shell);
    assert_eq!(FileType::from_filename("run"), FileType::Shell);
}

#[test]
fn test_file_type_from_filename_makefile() {
    assert_eq!(FileType::from_filename("Makefile"), FileType::Makefile);
    assert_eq!(FileType::from_filename("makefile"), FileType::Makefile);
    assert_eq!(FileType::from_filename("GNUmakefile"), FileType::Makefile);
}

#[test]
fn test_file_type_from_filename_dockerfile() {
    assert_eq!(FileType::from_filename("Dockerfile"), FileType::Dockerfile);
    assert_eq!(
        FileType::from_filename("Dockerfile.prod"),
        FileType::Dockerfile
    );
}

#[test]
fn test_lint_options_default() {
    let options = LintOptions::default();
    assert!(!options.quiet);
    assert_eq!(options.level, LintLevel::Info);
    assert!(options.ignore_rules.is_empty());
    assert!(!options.fix);
}

#[test]
fn test_process_lint_basic() {
    let source = "echo hello world";
    let options = LintOptions::default();
    let result = process_lint(source, "test.sh", &options);

    assert_eq!(result.file_type, FileType::Shell);
    // Results depend on what the linter finds
}

#[test]
fn test_process_lint_with_ignored_rules() {
    let source = "echo hello world";
    let mut options = LintOptions::default();
    options.ignore_rules.insert("SC2086".to_string());
    let result = process_lint(source, "test.sh", &options);

    // Verify ignored rules are filtered out
    for diag in &result.diagnostics {
        assert_ne!(diag.code, "SC2086");
    }
}

#[test]
fn test_process_lint_quiet_mode() {
    let source = "echo hello world";
    let options = LintOptions {
        quiet: true,
        ..Default::default()
    };
    let result = process_lint(source, "test.sh", &options);

    // Info-level diagnostics should be filtered
    for diag in &result.diagnostics {
        assert!(diag.severity >= crate::linter::Severity::Warning);
    }
}

// ===== PURIFY PROCESS TESTS =====

#[test]
fn test_process_purify_bash_simple() {
    let source = "echo hello";
    let result = process_purify_bash(source);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(!result.purified_source.is_empty());
}

#[test]
fn test_process_purify_bash_mkdir() {
    let source = "mkdir /tmp/test";
    let result = process_purify_bash(source).unwrap();

    // Purification should output something with mkdir
    assert!(result.purified_source.contains("mkdir"));
    // The output should be valid
    assert!(!result.purified_source.is_empty());
}

#[test]
fn test_process_purify_bash_stats() {
    let source = "echo hello\necho world";
    let result = process_purify_bash(source).unwrap();

    assert_eq!(result.stats.input_lines, 2);
    assert!(result.stats.input_bytes > 0);
    assert!(result.stats.total_time_ns > 0);
}

#[test]
fn test_transformation_rule_detection() {
    assert!(detect_transformation_rule("mkdir /tmp", "mkdir -p /tmp").contains("IDEM001"));
    assert!(detect_transformation_rule("rm /tmp/file", "rm -f /tmp/file").contains("IDEM002"));
    assert!(detect_transformation_rule("ln file link", "ln -sf file link").contains("IDEM003"));
    assert!(detect_transformation_rule("echo $RANDOM", "echo 42").contains("DET001"));
}

// ===== PURIFICATION STATS TESTS =====

#[test]
fn test_purification_stats_throughput() {
    let stats = PurificationStats {
        input_lines: 100,
        input_bytes: 1024 * 1024, // 1 MB
        output_lines: 90,
        output_bytes: 900 * 1024,
        read_time_ns: 1_000_000,
        parse_time_ns: 5_000_000,
        purify_time_ns: 3_000_000,
        codegen_time_ns: 2_000_000,
        write_time_ns: 1_000_000,
        total_time_ns: 1_000_000_000, // 1 second
    };

    let throughput = stats.throughput_mb_s();
    assert!((throughput - 1.0).abs() < 0.01); // ~1 MB/s
}

#[test]
fn test_purification_stats_format_report() {
    let stats = PurificationStats {
        input_lines: 10,
        input_bytes: 100,
        output_lines: 8,
        output_bytes: 80,
        read_time_ns: 1_000,
        parse_time_ns: 2_000,
        purify_time_ns: 3_000,
        codegen_time_ns: 4_000,
        write_time_ns: 5_000,
        total_time_ns: 15_000,
    };

    let report = stats.format_report("input.sh", Some("output.sh"));
    assert!(report.contains("input.sh"));
    assert!(report.contains("output.sh"));
    assert!(report.contains("10 lines"));
    assert!(report.contains("100 bytes"));
}

// ===== LINT FILTERING TESTS =====

#[test]
fn test_parse_rule_filter() {
    let rules = parse_rule_filter("SEC001,DET002,IDEM003");
    assert_eq!(rules, vec!["SEC001", "DET002", "IDEM003"]);
}

#[test]
fn test_parse_rule_filter_with_spaces() {
    let rules = parse_rule_filter(" SEC001 , DET002 , IDEM003 ");
    assert_eq!(rules, vec!["SEC001", "DET002", "IDEM003"]);
}

#[test]
fn test_parse_rule_filter_single() {
    let rules = parse_rule_filter("SEC001");
    assert_eq!(rules, vec!["SEC001"]);
}

// ===== RULE CODE TESTS =====

#[test]
fn test_parse_rule_codes() {
    let codes = parse_rule_codes("sec001,det002,idem003");
    assert_eq!(codes, vec!["SEC001", "DET002", "IDEM003"]);
}

#[test]
fn test_parse_rule_codes_with_spaces() {
    let codes = parse_rule_codes(" sec001 , det002 ");
    assert_eq!(codes, vec!["SEC001", "DET002"]);
}

#[test]
fn test_parse_rule_codes_empty() {
    let codes = parse_rule_codes("");
    assert!(codes.is_empty());
}

#[test]
fn test_diagnostic_matches_rules() {
    let rules = vec!["SEC".to_string(), "DET".to_string()];
    assert!(diagnostic_matches_rules("SEC001", &rules));
    assert!(diagnostic_matches_rules("DET002", &rules));
    assert!(!diagnostic_matches_rules("IDEM001", &rules));
}

// ===== SEVERITY ICON TESTS =====

#[test]
fn test_severity_icon() {
    assert_eq!(severity_icon("error"), "❌");
    assert_eq!(severity_icon("Error"), "❌");
    assert_eq!(severity_icon("ERROR"), "❌");
    assert_eq!(severity_icon("warning"), "⚠");
    assert_eq!(severity_icon("info"), "ℹ");
    assert_eq!(severity_icon("hint"), "💡");
    assert_eq!(severity_icon("unknown"), "•");
}
