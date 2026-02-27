#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::cli::args::{CorpusFormatArg, CorpusOutputFormat};

// ============================================================================
// CONSOLIDATED SLOW TEST: One test calls all corpus functions sequentially
// to avoid parallel corpus loading overhead (each function internally calls
// load_full() + runner.run()). This single test covers ~2,600 lines in 7 modules.
// ============================================================================

#[test]
#[ignore] // Takes 60+ seconds: loads 17k corpus entries, runs full transpiler
fn test_cov_corpus_all_sequential() {
    // --- corpus_metrics_commands (376 lines) ---
    let _ = super::corpus_metrics_commands::corpus_topk(3);
    let _ = super::corpus_metrics_commands::corpus_format_cmp();
    let _ = super::corpus_metrics_commands::corpus_stability();
    let _ = super::corpus_metrics_commands::corpus_version();
    let _ = super::corpus_metrics_commands::corpus_rate();
    let _ = super::corpus_metrics_commands::corpus_dist();
    let _ = super::corpus_metrics_commands::corpus_trace("B-001");
    let _ = super::corpus_metrics_commands::corpus_trace("ZZZZZ-999");
    let _ = super::corpus_metrics_commands::corpus_suspicious(3);

    // --- corpus_gate_commands (344 lines) ---
    let fmt_human = CorpusOutputFormat::Human;
    let fmt_json = CorpusOutputFormat::Json;
    let filter_bash = CorpusFormatArg::Bash;
    let filter_make = CorpusFormatArg::Makefile;
    let filter_docker = CorpusFormatArg::Dockerfile;

    let _ = super::corpus_gate_commands::corpus_errors(&fmt_human, None);
    let _ = super::corpus_gate_commands::corpus_errors(&fmt_json, None);
    let _ = super::corpus_gate_commands::corpus_errors(&fmt_human, Some(&filter_bash));
    let _ = super::corpus_gate_commands::corpus_errors(&fmt_human, Some(&filter_make));
    let _ = super::corpus_gate_commands::corpus_errors(&fmt_human, Some(&filter_docker));
    let _ = super::corpus_gate_commands::corpus_sample(3, None);
    let _ = super::corpus_gate_commands::corpus_sample(2, Some(&filter_bash));
    let _ = super::corpus_gate_commands::corpus_completeness();
    let _ = super::corpus_gate_commands::corpus_gate(50.0, 5000);
    super::corpus_gate_commands::gate_print_check("Test passes", true);
    super::corpus_gate_commands::gate_print_check("Test fails", false);
    let _ = super::corpus_gate_commands::corpus_outliers(2.0, None);
    let _ = super::corpus_gate_commands::corpus_outliers(2.0, Some(&filter_bash));
    let _ = super::corpus_gate_commands::corpus_matrix();

    // --- corpus_compare_commands (330 lines) ---
    let _ = super::corpus_compare_commands::corpus_health();
    let _ = super::corpus_compare_commands::corpus_compare("B-001", "B-002");
    let _ = super::corpus_compare_commands::corpus_compare("B-001", "ZZZZZ-999");
    let _ = super::corpus_compare_commands::corpus_density();
    let _ = super::corpus_compare_commands::corpus_perf(None);
    let _ = super::corpus_compare_commands::corpus_perf(Some(&filter_make));
    let _ = super::corpus_compare_commands::corpus_citl(None);
    let _ = super::corpus_compare_commands::corpus_citl(Some(&filter_docker));
    let _ = super::corpus_compare_commands::corpus_streak();

    // --- corpus_weight_commands (321 lines) ---
    let _ = super::corpus_weight_commands::corpus_weight();
    let _ = super::corpus_weight_commands::corpus_format_report(&fmt_human);
    let _ = super::corpus_weight_commands::corpus_format_report(&fmt_json);
    let _ = super::corpus_weight_commands::corpus_budget();
    let _ = super::corpus_weight_commands::corpus_entropy();
    let _ = super::corpus_weight_commands::corpus_todo();

    // --- corpus_convergence_commands (307 lines) ---
    let _ = super::corpus_convergence_commands::corpus_converge_table();
    let _ = super::corpus_convergence_commands::corpus_converge_diff(None, None);
    let _ = super::corpus_convergence_commands::corpus_converge_status();
    let _ = super::corpus_convergence_commands::corpus_mine(5);
    let _ = super::corpus_convergence_commands::corpus_fix_gaps(5);
    let _ = super::corpus_convergence_commands::corpus_org_patterns();
    let _ = super::corpus_convergence_commands::corpus_schema_validate();
    let _ = super::corpus_convergence_commands::corpus_grammar_errors();
    let _ = super::corpus_convergence_commands::corpus_format_grammar(CorpusFormatArg::Bash);
    let _ = super::corpus_convergence_commands::corpus_format_grammar(CorpusFormatArg::Makefile);
    let _ = super::corpus_convergence_commands::corpus_format_grammar(CorpusFormatArg::Dockerfile);

    // --- corpus_time_commands (309 lines) ---
    let _ = super::corpus_time_commands::corpus_timeline();
    let _ = super::corpus_time_commands::corpus_drift();
    let _ = super::corpus_time_commands::corpus_slow(3, None);
    let _ = super::corpus_time_commands::corpus_slow(2, Some(&filter_bash));
    let _ = super::corpus_time_commands::corpus_tags();

    // --- corpus_b2_commands - slow parts (307 lines) ---
    let _ = super::corpus_b2_commands::corpus_diagnose_b2(None, 5);
    let _ = super::corpus_b2_commands::corpus_fix_b2(false);
}

// ============================================================================
// FAST UNIT TESTS: These don't load the corpus (pure helper functions)
// ============================================================================

#[test]
fn test_cov_collect_trace_coverage() {
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;
    use crate::models::Config;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let mut small_registry = registry;
    small_registry.entries.truncate(5);
    let _ = super::corpus_metrics_commands::collect_trace_coverage(&small_registry, &runner);
}

#[test]
fn test_cov_gate_print_check_pass() {
    super::corpus_gate_commands::gate_print_check("Test check passes", true);
}

#[test]
fn test_cov_gate_print_check_fail() {
    super::corpus_gate_commands::gate_print_check("Test check fails", false);
}

#[test]
fn test_cov_percentile() {
    let sorted = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let p50 = super::corpus_compare_commands::percentile(&sorted, 50.0);
    assert!((p50 - 3.0).abs() < 0.01);
}

#[test]
fn test_cov_percentile_empty() {
    let sorted: Vec<f64> = vec![];
    let p50 = super::corpus_compare_commands::percentile(&sorted, 50.0);
    assert!((p50 - 0.0).abs() < 0.01);
}

#[test]
fn test_cov_drift_print_format() {
    super::corpus_time_commands::drift_print_format("TestFmt", 90, 100, 90.0, 95, 100, 95.0);
}

#[test]
fn test_cov_drift_print_format_empty() {
    super::corpus_time_commands::drift_print_format("Empty", 0, 0, 0.0, 0, 0, 0.0);
}

#[test]
fn test_cov_classify_b2_only_false_positive() {
    let (cat, _best) = super::corpus_b2_commands::classify_b2_only(
        "echo hello",
        "echo hello\nsome other line\n",
    );
    assert_eq!(cat, "false_positive");
}

#[test]
fn test_cov_classify_b2_only_echo_to_printf() {
    let (cat, _best) = super::corpus_b2_commands::classify_b2_only(
        "echo hello",
        "printf hello world\necho hello world\n",
    );
    assert!(!cat.is_empty());
}

#[test]
fn test_cov_classify_b2_only_multiline_mismatch() {
    let (cat, _best) = super::corpus_b2_commands::classify_b2_only(
        "something unique",
        "completely different output\n",
    );
    assert_eq!(cat, "multiline_mismatch");
}

#[test]
fn test_cov_classify_b2_only_line_wider() {
    let (cat, _best) = super::corpus_b2_commands::classify_b2_only(
        "echo",
        "echo hello world from the shell\n",
    );
    assert_eq!(cat, "line_wider");
}

#[test]
fn test_cov_classify_b1b2() {
    let cat = super::corpus_b2_commands::classify_b1b2("echo hello", "printf '%s\\n' hello\n");
    assert!(!cat.is_empty());
}

#[test]
fn test_cov_classify_b1b2_empty_expected() {
    let cat = super::corpus_b2_commands::classify_b1b2("", "some output\n");
    assert_eq!(cat, "empty_expected");
}

#[test]
fn test_cov_classify_b1b2_no_output() {
    let cat = super::corpus_b2_commands::classify_b1b2("echo hello", "");
    assert_eq!(cat, "echo_missing");
}

#[test]
fn test_cov_classify_b1b2_no_output_non_echo() {
    let cat = super::corpus_b2_commands::classify_b1b2("mkdir -p /tmp/foo", "");
    assert_eq!(cat, "no_output");
}

#[test]
fn test_cov_classify_b1b2_partial_match() {
    let cat = super::corpus_b2_commands::classify_b1b2(
        "echo hello world",
        "echo hello planet\n",
    );
    assert!(!cat.is_empty());
}

#[test]
fn test_cov_print_b2_category() {
    let items = vec![
        ("B-001".to_string(), "echo hello".to_string(), "echo hello world".to_string()),
        ("B-002".to_string(), "echo foo".to_string(), "echo foo bar".to_string()),
    ];
    super::corpus_b2_commands::print_b2_category("test_cat", &items, 5);
}

#[test]
fn test_cov_find_best_b2_replacement_substring() {
    let result = super::corpus_b2_commands::find_best_b2_replacement(
        "echo",
        "echo hello world\nset -e\n",
        "B-001",
    );
    assert!(result.is_some());
    assert!(result.unwrap().contains("echo"));
}

#[test]
fn test_cov_find_best_b2_replacement_no_match() {
    let result = super::corpus_b2_commands::find_best_b2_replacement(
        "zzz_unique_string",
        "completely different\n",
        "B-001",
    );
    let _ = result;
}

#[test]
fn test_cov_extract_main_body_bash() {
    let actual = "#!/bin/sh\nset -e\nmain() {\necho hello\n}\nmain \"$@\"";
    let body = super::corpus_b2_commands::extract_main_body(actual, "B-001");
    assert!(!body.is_empty());
}

#[test]
fn test_cov_extract_main_body_dockerfile() {
    let actual = "# comment\nFROM ubuntu\nRUN echo hi\n";
    let body = super::corpus_b2_commands::extract_main_body(actual, "D-001");
    assert!(!body.is_empty());
}

#[test]
fn test_cov_extract_main_body_makefile() {
    let actual = "# comment\nall: build\n\t@echo building\n";
    let body = super::corpus_b2_commands::extract_main_body(actual, "M-001");
    assert!(!body.is_empty());
}

#[test]
fn test_cov_extract_noncomment_lines() {
    let lines = super::corpus_b2_commands::extract_noncomment_lines("# comment\nFROM ubuntu\n\nRUN echo hi\n");
    assert_eq!(lines.len(), 2);
}

#[test]
fn test_cov_is_bash_preamble() {
    assert!(super::corpus_b2_commands::is_bash_preamble(""));
    assert!(super::corpus_b2_commands::is_bash_preamble("#!/bin/sh"));
    assert!(super::corpus_b2_commands::is_bash_preamble("set -euf"));
    assert!(super::corpus_b2_commands::is_bash_preamble("IFS=$'\\n'"));
    assert!(super::corpus_b2_commands::is_bash_preamble("export PATH=/usr/bin"));
    assert!(super::corpus_b2_commands::is_bash_preamble("trap cleanup EXIT"));
    assert!(super::corpus_b2_commands::is_bash_preamble("main \"$@\""));
    assert!(!super::corpus_b2_commands::is_bash_preamble("echo hello"));
}

#[test]
fn test_cov_extract_bash_main_body() {
    let script = "#!/bin/sh\nset -e\nrash_println() {\nprintf '%s\\n' \"$@\"\n}\nmain() {\necho hello\necho world\n}\nmain \"$@\"";
    let body = super::corpus_b2_commands::extract_bash_main_body(script);
    assert!(body.iter().any(|l| l.contains("echo")));
}

#[test]
fn test_cov_advance_bash_body_state() {
    use super::corpus_b2_commands::BashBodyState;

    let mut out = Vec::new();

    let state = super::corpus_b2_commands::advance_bash_body_state("rash_println() {", BashBodyState::Before, &mut out);
    assert!(state == BashBodyState::InFuncDef);

    let state = super::corpus_b2_commands::advance_bash_body_state("}", BashBodyState::InFuncDef, &mut out);
    assert!(state == BashBodyState::Before);

    let state = super::corpus_b2_commands::advance_bash_body_state("main() {", BashBodyState::Before, &mut out);
    assert!(state == BashBodyState::InMain);

    let state = super::corpus_b2_commands::advance_bash_body_state("echo hello", BashBodyState::InMain, &mut out);
    assert!(state == BashBodyState::InMain);
    assert_eq!(out.len(), 1);

    let state = super::corpus_b2_commands::advance_bash_body_state("}", BashBodyState::InMain, &mut out);
    assert!(state == BashBodyState::Before);
}

#[test]
fn test_cov_find_best_token_match() {
    let lines = vec![
        "echo hello world".to_string(),
        "printf format string".to_string(),
        "set -e".to_string(),
    ];
    let result = super::corpus_b2_commands::find_best_token_match("echo hello", &lines);
    assert!(result.is_some());
    assert!(result.unwrap().contains("echo"));
}

#[test]
fn test_cov_find_best_token_match_no_overlap() {
    let lines = vec![
        "aaaa=1".to_string(),
        "bbbb=2".to_string(),
    ];
    let result = super::corpus_b2_commands::find_best_token_match("zzzz unique", &lines);
    assert!(result.is_some());
}

#[test]
fn test_cov_find_best_token_match_empty() {
    let lines: Vec<String> = vec![];
    let result = super::corpus_b2_commands::find_best_token_match("echo hello", &lines);
    assert!(result.is_none());
}

#[test]
fn test_cov_collect_b2_fixes() {
    use crate::corpus::registry::Grade;
    use crate::corpus::runner::CorpusScore;

    let score = CorpusScore {
        total: 0,
        passed: 0,
        failed: 0,
        rate: 0.0,
        score: 0.0,
        grade: Grade::F,
        results: vec![],
        format_scores: vec![],
    };
    let fixes = super::corpus_b2_commands::collect_b2_fixes(&score);
    assert!(fixes.is_empty());
}
